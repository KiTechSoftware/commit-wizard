use crate::engine::{
    config::{BaseConfig, RulesConfig},
    constants::app_config_dir,
    models::policy::Policy,
};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Debug, Clone)]
pub struct RuntimeResolution {
    // the config options we have available, determined by looking for config files in the current directory and parent directories, as well as global config locations
    pub sources: AvailableConfigOptions,
    // the resolved config we will use for this run, determined by merging the available config options based on precedence rules
    pub config: Option<ResolvedConfig>,
    // resolved policy, which is the final set of rules and base config that we will use to validate commits, determined by applying the config options according to the policy defined in the config and the precedence rules
    pub policy: Policy,
}

impl Default for RuntimeResolution {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeResolution {
    pub fn new() -> Self {
        Self {
            sources: AvailableConfigOptions::new(),
            config: None,
            policy: Policy::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    /// the path of the config file that contributed to the resolved config, if any
    pub path: Option<PathBuf>,
    /// the rules used to resolve the config (kept for reference/diagnostics)
    /// Note: rules are merged into `base` during resolution
    pub rules: RulesConfig,
    /// the base config with rules applied - the single source of truth for policy
    /// This is the complete, merged configuration (base + rules) passed to policy engine.
    /// Policy reads from this field only; no side effects or dynamic rule lookups.
    pub base: BaseConfig,
}

#[derive(Debug, Clone)]
pub struct RuntimePaths {
    /// the current working directory, determined at runtime
    pub cwd: PathBuf,
    /// the global paths for config, cache, and state, determined at runtime
    pub global: RuntimeGlobalPaths,
    /// whether we are in a git repository, determined by looking for a .git directory in the current or parent directories
    pub in_git_repo: bool,
    /// default is cwd, but updated if we detect a git repository root
    pub repo_root: Option<PathBuf>,
    /// an optional explicit config path provided by the user, which overrides all other config sources if present
    pub explicit_config_path: Option<PathBuf>,
    /// an optional explicit registry provided by the user, which overrides discovered registries if present
    pub explicit_registry: Option<String>,
    /// an optional explicit registry ref (e.g. git tag or branch) provided by the user, which overrides the registry config if present
    pub explicit_registry_ref: Option<String>,
    /// an optional explicit registry section provided by the user, which overrides discovered registry sections if present
    pub explicit_registry_section: Option<String>,
}

impl Default for RuntimePaths {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePaths {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|e| {
            eprintln!("[warn] Failed to determine current directory: {e} — using '.'");
            PathBuf::from(".")
        });
        let global = RuntimeGlobalPaths::new().unwrap_or_else(|e| {
            eprintln!(
                "[warn] Failed to resolve global paths: {e} — using '.' for config/cache/state"
            );
            RuntimeGlobalPaths {
                config: PathBuf::from("."),
                cache: PathBuf::from("."),
                state: PathBuf::from("."),
            }
        });
        Self {
            cwd,
            global,
            in_git_repo: false,
            repo_root: None,
            explicit_config_path: None,
            explicit_registry: None,
            explicit_registry_ref: None,
            explicit_registry_section: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeGlobalPaths {
    /// the users global configuration directory, determined at runtime
    pub config: PathBuf,
    /// the users global cache directory, determined at runtime
    pub cache: PathBuf,
    /// the users global state directory, determined at runtime
    pub state: PathBuf,
}

impl RuntimeGlobalPaths {
    pub fn new() -> crate::engine::error::Result<Self> {
        use crate::engine::constants::{app_cache_dir, app_state_dir};
        let config = app_config_dir()?;
        let cache = app_cache_dir()?;
        let state = app_state_dir()?;
        Ok(Self {
            config,
            cache,
            state,
        })
    }
}

impl Default for RuntimeGlobalPaths {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            eprintln!(
                "[warn] Failed to resolve global paths: {e} — using '.' for config/cache/state"
            );
            Self {
                config: std::path::PathBuf::from("."),
                cache: std::path::PathBuf::from("."),
                state: std::path::PathBuf::from("."),
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct AvailableConfigOptions {
    pub cli_config: Option<AvailableConfig>,
    /// Config built from `CW_*` environment variables (SRS §4).
    pub env_config: Option<AvailableConfig>,
    pub repo_config: Option<AvailableConfig>,
    pub global_config: Option<AvailableConfig>,
    /// Loaded registries (SRS §5). Each entry holds its resolved config.
    pub registries: Vec<RegistryOptions>,
}

impl Default for AvailableConfigOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl AvailableConfigOptions {
    pub fn new() -> Self {
        Self {
            cli_config: None,
            env_config: None,
            repo_config: None,
            global_config: None,
            registries: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cli_config.is_none()
            && self.env_config.is_none()
            && self.repo_config.is_none()
            && self.global_config.is_none()
            && self.registries.is_empty()
    }

    pub fn has_cli_config(&self) -> bool {
        self.cli_config.is_some()
    }

    pub fn set_cli_config(&mut self, config: AvailableConfig) -> &mut Self {
        self.cli_config = Some(config);
        self
    }

    pub fn has_repo_config(&self) -> bool {
        self.repo_config.is_some()
    }

    pub fn set_repo_config(&mut self, config: AvailableConfig) -> &mut Self {
        self.repo_config = Some(config);
        self
    }

    pub fn has_global_config(&self) -> bool {
        self.global_config.is_some()
    }

    pub fn set_global_config(&mut self, config: AvailableConfig) -> &mut Self {
        self.global_config = Some(config);
        self
    }

    pub fn has_registries(&self) -> bool {
        !self.registries.is_empty()
    }

    pub fn registry_count(&self) -> usize {
        self.registries.len()
    }
}

#[derive(Debug, Clone)]
pub struct RegistryOptions {
    // unique identifier for the registry: url#ref or url#ref/section for sectioned registries
    pub id: String,
    // an identifier for the registry, e.g. "default" or "main"
    // used by git-based registries to determine which branch or tag to use, and by users to specify which registry to use when multiple are available
    pub tag: String,
    // external reference to the registry, e.g. a git URL or a local path
    pub url: String,
    // the git ref (branch, tag, or commit SHA) used to load this registry
    pub r#ref: String,
    // the section within the registry (for sectioned registries), if any
    pub section: Option<String>,
    // if flat registry, the resolved config options from the registry URL, determined by fetching and parsing the registry config file
    pub config: Option<AvailableConfig>,
    // if hierarchical registry, the resolved config options from the registry URL and section, determined by fetching and parsing the registry config file and extracting the specified section
    pub sections: Option<BTreeMap<String, AvailableConfig>>,
    // whether this is the active registry selected via precedence (CLI > ENV > repo > global)
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct AvailableConfig {
    pub rules: Option<RulesConfig>,
    pub base: Option<BaseConfig>,
}

pub fn resolve_available_config(
    base: Option<BaseConfig>,
    rules: Option<RulesConfig>,
) -> AvailableConfig {
    AvailableConfig { base, rules }
}

pub fn resolve_policy(config: Option<&ResolvedConfig>) -> Policy {
    match config {
        Some(cfg) => Policy::from_config(cfg),
        None => Policy::default(),
    }
}
