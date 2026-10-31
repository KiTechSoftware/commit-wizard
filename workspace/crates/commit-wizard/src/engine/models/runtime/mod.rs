use std::path::PathBuf;

use crate::engine::{
    LoggerTrait,
    config::{
        BaseConfig,
        env::build_env_config,
        registry::{RegistrySpec, load_registry, resolve_registry_spec},
        resolver::{resolve_global_configs, resolve_project_configs},
    },
    constants::{competing_project_configs, resolve_project_config_path},
    models::policy::Policy,
};

pub mod mode;
pub mod options;
pub mod resolution;
pub use options::*;
pub use resolution::*;

#[derive(Debug, Clone)]
pub struct Runtime {
    // the mode we are running in, determined at runtime based on args and environment
    mode: mode::RunMode,
    // options that affect how we run, determined at runtime based on args and environment
    options: RuntimeOptions,
    // paths and environment information, determined at runtime
    paths: RuntimePaths,
    // the config options we have available, determined at runtime
    resolution: RuntimeResolution,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn output_config(&self) -> scriba::Config {
        scriba::Config {
            interactive: matches!(self.mode, mode::RunMode::Interactive),
            format: self.options.output_format(),
            color: self.options.output_color(),
            level: self.options.log_level(),
            auto_yes: self.options.auto_yes(),
        }
    }

    // -------------------------
    // getters
    // -------------------------

    pub fn is_ci(&self) -> bool {
        matches!(self.mode, mode::RunMode::Ci)
    }

    pub fn is_non_interactive(&self) -> bool {
        matches!(self.mode, mode::RunMode::NonInteractive)
    }

    pub fn is_interactive(&self) -> bool {
        matches!(self.mode, mode::RunMode::Interactive)
    }

    pub fn mode(&self) -> &mode::RunMode {
        &self.mode
    }

    pub fn options(&self) -> &RuntimeOptions {
        &self.options
    }

    pub fn options_mut(&mut self) -> &mut RuntimeOptions {
        &mut self.options
    }

    pub fn cwd(&self) -> &PathBuf {
        &self.paths.cwd
    }

    pub fn in_git_repo(&self) -> bool {
        self.paths.in_git_repo
    }

    pub fn repo_root(&self) -> &PathBuf {
        self.paths.repo_root.as_ref().unwrap_or(&self.paths.cwd)
    }

    pub fn global_paths(&self) -> &RuntimeGlobalPaths {
        &self.paths.global
    }

    pub fn global_config_path(&self) -> &PathBuf {
        &self.paths.global.config
    }

    pub fn global_cache_path(&self) -> &PathBuf {
        &self.paths.global.cache
    }

    pub fn global_state_path(&self) -> &PathBuf {
        &self.paths.global.state
    }

    pub fn state_file_path(&self) -> PathBuf {
        use crate::engine::constants::STATE_FILE_NAME;
        self.paths.global.state.join(STATE_FILE_NAME)
    }

    pub fn sources(&self) -> &AvailableConfigOptions {
        &self.resolution.sources
    }

    pub fn sources_mut(&mut self) -> &mut AvailableConfigOptions {
        &mut self.resolution.sources
    }

    pub fn config(&self) -> Option<&ResolvedConfig> {
        self.resolution.config.as_ref()
    }

    pub fn config_mut(&mut self) -> Option<&mut ResolvedConfig> {
        self.resolution.config.as_mut()
    }

    pub fn policy(&self) -> &Policy {
        &self.resolution.policy
    }

    pub fn policy_mut(&mut self) -> &mut Policy {
        &mut self.resolution.policy
    }

    // -------------------------
    // setters
    // -------------------------

    pub fn set_mode(&mut self, mode: mode::RunMode) -> &mut Self {
        self.mode = mode;
        self
    }

    pub fn set_dry_run(&mut self, dry_run: bool) -> &mut Self {
        self.options.set_dry_run(dry_run);
        self
    }

    pub fn set_auto_yes(&mut self, auto_yes: bool) -> &mut Self {
        self.options.set_auto_yes(auto_yes);
        self
    }

    pub fn set_force(&mut self, force: bool) -> &mut Self {
        self.options.set_force(force);
        self
    }

    pub fn set_output_envelope(&mut self, envelope: scriba::EnvelopeMode) -> &mut Self {
        self.options.set_output_envelope(envelope);
        self
    }

    pub fn set_output_format(&mut self, format: scriba::Format) -> &mut Self {
        self.options.set_output_format(format);
        self
    }

    pub fn set_output_color(&mut self, color: scriba::ColorMode) -> &mut Self {
        self.options.set_output_color(color);
        self
    }

    pub fn set_log_level(&mut self, level: scriba::Level) -> &mut Self {
        self.options.set_log_level(level);
        self
    }

    pub fn set_cwd(&mut self, cwd: PathBuf) -> &mut Self {
        // needs to resolve to full path to ensure consistency when comparing with repo root
        self.paths.cwd = std::fs::canonicalize(&cwd).unwrap_or_else(|e| {
            eprintln!(
                "[warn] Failed to canonicalize cwd {:?}: {e} — using path as-is",
                cwd
            );
            cwd
        });
        self
    }

    pub fn set_in_git_repo(&mut self, in_git_repo: bool) -> &mut Self {
        self.paths.in_git_repo = in_git_repo;
        self
    }

    pub fn set_repo_root(&mut self, repo_root: PathBuf) -> &mut Self {
        // needs to resolve to full path to ensure consistency when comparing with cwd
        self.paths.repo_root = Some(std::fs::canonicalize(&repo_root).unwrap_or_else(|e| {
            eprintln!(
                "[warn] Failed to canonicalize repo_root {:?}: {e} — using path as-is",
                repo_root
            );
            repo_root
        }));
        self
    }

    pub fn set_sources(&mut self, sources: AvailableConfigOptions) -> &mut Self {
        self.resolution.sources = sources;
        self
    }

    pub fn set_config(&mut self, config: ResolvedConfig) -> &mut Self {
        self.resolution.config = Some(config);
        self
    }

    pub fn set_policy(&mut self, policy: Policy) -> &mut Self {
        self.resolution.policy = policy;
        self
    }

    pub fn new() -> Self {
        Self {
            mode: mode::RunMode::Interactive,
            options: RuntimeOptions::new(),
            paths: RuntimePaths::new(),
            resolution: RuntimeResolution {
                sources: AvailableConfigOptions {
                    cli_config: None,
                    env_config: None,
                    repo_config: None,
                    global_config: None,
                    registries: Vec::new(),
                    registry_failures: Vec::new(),
                },
                config: None,
                policy: Policy::default(),
            },
        }
    }

    pub fn resolve_cli_source(&mut self, logger: Option<&dyn LoggerTrait>) {
        if let Some(path) = self.explicit_config_path().cloned() {
            let (base_config, rules_config) = resolve_project_configs(&path, logger);
            self.resolution.sources.cli_config =
                Some(resolve_available_config(base_config, rules_config));
        }
    }

    pub fn resolve_repo_source(&mut self, logger: &dyn LoggerTrait) {
        let cwd = self.cwd().clone();
        let repo_root = self.repo_root().clone();
        let in_git = self.in_git_repo();

        let msg = format!(
            "Resolving repo config: cwd={:?}, repo_root={:?}, in_git_repo={}",
            cwd, repo_root, in_git
        );
        logger.debug(&msg);

        if let Some(path) = resolve_project_config_path(&cwd, Some(&repo_root), in_git, None) {
            let msg = format!("Found config path: {:?}", path);
            logger.debug(&msg);
            let ignored = competing_project_configs(&path);
            if !ignored.is_empty() {
                let names = ignored
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                logger.warn(&format!(
                    "Several project config files found; using {} and ignoring {names}",
                    path.display()
                ));
            }
            let (base_config, rules_config) = resolve_project_configs(&path, Some(logger));
            let msg = format!(
                "Config loaded: base={}, rules={}",
                base_config.is_some(),
                rules_config.is_some()
            );
            logger.debug(&msg);
            self.resolution.sources.repo_config =
                Some(resolve_available_config(base_config, rules_config));
        } else {
            logger.info("No project config found");
        }
    }

    pub fn resolve_global_source(&mut self) {
        let (base_config, rules_config) = resolve_global_configs();
        self.resolution.sources.global_config =
            Some(resolve_available_config(base_config, rules_config));
    }

    pub fn resolve_env_source(&mut self) {
        if let Some(base) = build_env_config() {
            self.resolution.sources.env_config = Some(resolve_available_config(Some(base), None));
        }
    }

    /// Resolve and load all available registries from every config layer,
    /// marking the one selected via precedence (CLI > ENV > repo > global) as active.
    pub fn resolve_registry_source(
        &mut self,
        logger: &dyn LoggerTrait,
    ) -> crate::engine::error::Result<()> {
        let partial_base = self.build_partial_config_for_registry();

        let cli_url = self.explicit_registry().map(String::to_owned);
        let cli_ref = self.explicit_registry_ref().map(String::to_owned);
        let cli_section = self.explicit_registry_section().map(String::to_owned);

        // Determine which registry is ACTIVE (via precedence: CLI > ENV > repo > global)
        let active_spec = resolve_registry_spec(
            cli_url.as_deref(),
            cli_ref.as_deref(),
            cli_section.as_deref(),
            Some(&partial_base),
        );

        // Resolve rule references in the active spec URL so it can be compared with the
        // collected specs (which have already had their URLs resolved from rules).
        let partial_rules = self.build_partial_rules_for_registry();
        let active_spec = active_spec.map(|mut spec| {
            if let Some(rules) = &partial_rules
                && let Ok(resolved) = rules.resolve_string(&spec.url)
            {
                spec.url = resolved;
            }
            spec
        });

        let required = partial_base.registry_required();
        let use_cache = partial_base.registry_cache();

        // A required registry that is named but cannot even be resolved is an error.
        if required && active_spec.is_none() && partial_base.registry_use().is_some() {
            return Err(crate::engine::error::ErrorCode::RegistryUnknown
                .error()
                .with_context("registry", partial_base.registry_use().unwrap_or_default())
                .with_context("reason", "registry is required but not configured"));
        }

        // Collect and deduplicate all registry specs from all config layers
        let mut all_specs = self.collect_all_registry_specs();

        // If the active spec was supplied via CLI or ENV and doesn't appear in the pool
        // (e.g. the user passed --registry for an ad-hoc URL not in the config), inject
        // it so it gets loaded and marked active.
        if let Some(ref a) = active_spec {
            let already_present = all_specs.iter().any(|(_, s)| {
                // Match on URL + ref only; section is resolved separately for active registry
                s.url == a.url && s.r#ref == a.r#ref
            });
            if !already_present {
                all_specs.push(("cli".to_string(), a.clone()));
            }
        }

        // Load each registry and add it to the pool
        let cache_dir = self.global_cache_path().clone();
        let state_file_path = self.state_file_path();

        for (name, spec) in all_specs {
            // For the active registry, use the fully-resolved active_spec (with section)
            // For others, use their spec as-is (which may have no section)
            let spec_to_load = if let Some(ref a) = active_spec {
                if spec.url == a.url && spec.r#ref == a.r#ref {
                    a.clone()
                } else {
                    spec.clone()
                }
            } else {
                spec.clone()
            };

            let is_active = active_spec
                .as_ref()
                .is_some_and(|a| a.url == spec.url && a.r#ref == spec.r#ref);

            // Build the stable registry id: url#ref or url#ref/section
            let registry_id = match &spec_to_load.section {
                Some(section) => {
                    format!("{}##{}/{}", spec_to_load.url, spec_to_load.r#ref, section)
                }
                None => format!("{}##{}", spec_to_load.url, spec_to_load.r#ref),
            };

            match load_registry(
                &spec_to_load,
                &cache_dir,
                &state_file_path,
                use_cache,
                logger,
            ) {
                Ok(result) => {
                    let status = if is_active { "[ACTIVE]" } else { "[available]" };
                    logger.debug(&format!(
                        "Registry loaded: url={}, ref={}, section={} {status}",
                        spec_to_load.url,
                        spec_to_load.r#ref,
                        spec_to_load.section.as_deref().unwrap_or("(root)")
                    ));

                    // Save state for active registry
                    if is_active {
                        use crate::engine::config::registry::registry_cache_path;
                        let cache_path =
                            registry_cache_path(&spec_to_load.url, &spec_to_load.r#ref, &cache_dir);

                        use crate::engine::models::state::{AppState, RegistryState};
                        let mut state = AppState::new();
                        state.registry = Some(RegistryState::new(
                            Some(name.clone()),
                            spec_to_load.url.clone(),
                            spec_to_load.r#ref.clone(),
                            spec_to_load.section.clone(),
                            result.resolved_commit.clone(),
                            cache_path,
                        ));

                        if let Err(e) = state.save(&state_file_path) {
                            logger.warn(&format!("Failed to save registry state: {e}"));
                        }
                    }

                    self.resolution.sources.registries.push(RegistryOptions {
                        id: registry_id,
                        tag: name,
                        url: spec_to_load.url,
                        r#ref: spec_to_load.r#ref,
                        section: spec_to_load.section,
                        config: Some(result.config),
                        sections: None,
                        is_active,
                    });
                }
                Err(e) if is_active && required => {
                    return Err(e.with_context("registry", name.clone()));
                }
                Err(e) => {
                    let message = format!("Registry load failed ({name}): {e}");
                    logger.warn(&message);
                    self.resolution.sources.registry_failures.push(message);
                }
            }
        }

        Ok(())
    }

    /// Build a merged BaseConfig from all layers except CLI (used to resolve the active registry
    /// before the full config is available).
    fn build_partial_config_for_registry(&self) -> BaseConfig {
        let global = self
            .resolution
            .sources
            .global_config
            .as_ref()
            .and_then(|c| c.base.clone());
        let env = self
            .resolution
            .sources
            .env_config
            .as_ref()
            .and_then(|c| c.base.clone());
        let repo = self
            .resolution
            .sources
            .repo_config
            .as_ref()
            .and_then(|c| c.base.clone());
        let cli = self
            .resolution
            .sources
            .cli_config
            .as_ref()
            .and_then(|c| c.base.clone());
        let base = global.unwrap_or_else(BaseConfig::empty);
        let base = if let Some(r) = repo {
            r.merge(base)
        } else {
            base
        };
        let base = if let Some(e) = env {
            e.merge(base)
        } else {
            base
        };
        if let Some(c) = cli {
            c.merge(base)
        } else {
            base
        }
    }

    /// Return the highest-precedence rules available before registries are loaded.
    /// Used to resolve @rules.* references in the active registry spec URL.
    fn build_partial_rules_for_registry(&self) -> Option<crate::engine::config::RulesConfig> {
        self.resolution
            .sources
            .cli_config
            .as_ref()
            .and_then(|c| c.rules.clone())
            .or_else(|| {
                self.resolution
                    .sources
                    .repo_config
                    .as_ref()
                    .and_then(|c| c.rules.clone())
            })
            .or_else(|| {
                self.resolution
                    .sources
                    .global_config
                    .as_ref()
                    .and_then(|c| c.rules.clone())
            })
    }

    /// Collect all uniquely-identifiable registry specs from every config layer (global + repo).
    /// Deduplicates by (name, url) to avoid loading the same registry twice.
    /// Resolves rule references in URLs (e.g., @rules.vars.cw_registry).
    fn collect_all_registry_specs(&self) -> Vec<(String, RegistrySpec)> {
        let mut specs: Vec<(String, RegistrySpec)> = Vec::new();

        for available_config in [
            self.resolution.sources.global_config.as_ref(),
            self.resolution.sources.repo_config.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            if let Some(cfg) = available_config.base.as_ref() {
                for (name, reg) in cfg.registries_map() {
                    if let Some(url) = reg.url {
                        // Resolve rule references in URL if rules are available.
                        // Failure to resolve MUST error (SRS §4.3) — no silent fallback.
                        let resolved_url = if let Some(rules) = &available_config.rules {
                            match rules.resolve_string(&url) {
                                Ok(s) => s,
                                Err(_) => {
                                    // Rule reference could not be resolved — skip this registry
                                    // with a note; error will surface clearly if this was the active one.
                                    continue;
                                }
                            }
                        } else {
                            url.clone()
                        };

                        // Only include explicit section field, NOT sections array
                        // (sections array is only for documentation/validation, not for pool deduplication)
                        specs.push((
                            name,
                            RegistrySpec {
                                url: resolved_url,
                                r#ref: reg.r#ref.unwrap_or_else(|| "HEAD".to_string()),
                                section: reg.section,
                            },
                        ));
                    }
                }
            }
        }

        // Deduplicate: keep first occurrence of each (name, url) pair
        let mut seen = std::collections::HashSet::new();
        specs.retain(|(name, spec)| seen.insert((name.clone(), spec.url.clone())));
        specs
    }

    pub fn resolve_available_sources(
        &mut self,
        logger: &dyn LoggerTrait,
    ) -> crate::engine::error::Result<()> {
        self.resolve_cli_source(Some(logger));
        self.resolve_env_source();
        self.resolve_repo_source(logger);
        self.resolve_global_source();
        self.resolve_registry_source(logger)
    }

    pub fn resolve_active_config(
        &mut self,
        logger: &dyn LoggerTrait,
    ) -> crate::engine::error::Result<()> {
        // Precedence rules:
        // 1. If no config sources: use the built-in defaults.
        // 2. If only global: use global as-is (no merging with defaults).
        // 3-4. If registry/repo/env/cli: registry is base, then repo, env and cli override it
        //      in that order (cli wins).
        // 5-6. If registry/repo/env/cli exists: never use global or defaults as base.
        // 7-9. Rules: cli > repo > registry > global (global only if registry has no rules).

        let global_base = self
            .resolution
            .sources
            .global_config
            .as_ref()
            .and_then(|c| c.base.clone());

        // The layers that take part (Rules 5-6): registry, repo, env and cli. The
        // global config is only used when none of them exists.
        let layers = self.config_layers();
        let has_registry_repo_or_cli = layers
            .iter()
            .any(|layer| layer.kind != ConfigLayerKind::Global);

        if has_registry_repo_or_cli {
            // Rules 3-4: lowest priority first, each layer overriding the ones below it.
            let base = layers
                .into_iter()
                .fold(BaseConfig::empty(), |base, layer| layer.base.merge(base));

            // Rules 7-9: Determine which rules to use.
            // cli > repo > registry > global (global only if registry has no rules).
            let registry_rules = self
                .resolution
                .sources
                .registries
                .iter()
                .find(|r| r.is_active)
                .and_then(|r| r.config.as_ref())
                .and_then(|c| c.rules.clone());

            let rules = self
                .resolution
                .sources
                .cli_config
                .as_ref()
                .and_then(|c| c.rules.clone())
                .or_else(|| {
                    self.resolution
                        .sources
                        .repo_config
                        .as_ref()
                        .and_then(|c| c.rules.clone())
                })
                .or_else(|| registry_rules.clone());

            // Rule 8: Only use global rules if registry doesn't have rules.
            let rules = if registry_rules.is_none() {
                rules.or_else(|| {
                    self.resolution
                        .sources
                        .global_config
                        .as_ref()
                        .and_then(|c| c.rules.clone())
                })
            } else {
                rules
            }
            .unwrap_or_default();

            // Merge rules into base: ResolvedConfig.base becomes the single source of truth
            // with all @rules.* references resolved. Failure to resolve MUST error (SRS §4.3).
            use crate::engine::config::resolver::merge_rules_into_base;
            let base = merge_rules_into_base(base, &rules)?;

            logger.debug(&format!(
                "[config] resolved commit.types: {:?}",
                base.commit
                    .as_ref()
                    .and_then(|c| c.types.as_ref())
                    .map(|t| t.keys().cloned().collect::<Vec<_>>()),
            ));

            let path = self.project_config_path();
            self.resolution.config = Some(ResolvedConfig { path, rules, base });
            self.resolve_policy();
        } else if global_base.is_some() {
            // Rule 2: Only global exists, use it as-is (no merging with defaults).
            let base = global_base.unwrap();
            let rules = self
                .resolution
                .sources
                .global_config
                .as_ref()
                .and_then(|c| c.rules.clone())
                .unwrap_or_default();

            use crate::engine::config::resolver::merge_rules_into_base;
            let base = merge_rules_into_base(base, &rules)?;

            logger.debug(&format!(
                "[config] resolved commit.types: {:?}",
                base.commit
                    .as_ref()
                    .and_then(|c| c.types.as_ref())
                    .map(|t| t.keys().cloned().collect::<Vec<_>>()),
            ));

            let path = self.project_config_path();
            self.resolution.config = Some(ResolvedConfig { path, rules, base });
            self.resolve_policy();
        } else {
            // Rule 1: No config sources at all (including a registry that failed to
            // load): run on the built-in defaults rather than leaving the config
            // unresolved, which every command treats as an error.
            use crate::engine::config::resolver::merge_rules_into_base;
            let rules = crate::engine::config::RulesConfig::default();
            let base = merge_rules_into_base(
                crate::engine::constants::defaults::config::standard_base_config(),
                &rules,
            )?;
            self.resolution.config = Some(ResolvedConfig {
                path: None,
                rules,
                base,
            });
            self.resolve_policy();
        }

        Ok(())
    }

    pub fn resolve_policy(&mut self) {
        let policy = resolve_policy(self.config());
        self.resolution.policy = policy;
    }

    pub fn explicit_config_path(&self) -> Option<&PathBuf> {
        self.paths.explicit_config_path.as_ref()
    }

    pub fn explicit_registry(&self) -> Option<&String> {
        self.paths.explicit_registry.as_ref()
    }

    pub fn explicit_registry_ref(&self) -> Option<&String> {
        self.paths.explicit_registry_ref.as_ref()
    }

    pub fn explicit_registry_section(&self) -> Option<&String> {
        self.paths.explicit_registry_section.as_ref()
    }

    pub fn set_explicit_config_path(&mut self, path: Option<PathBuf>) -> &mut Self {
        self.paths.explicit_config_path = path;
        self
    }

    pub fn set_explicit_registry(&mut self, registry: Option<String>) -> &mut Self {
        self.paths.explicit_registry = registry;
        self
    }

    pub fn set_explicit_registry_ref(&mut self, registry_ref: Option<String>) -> &mut Self {
        self.paths.explicit_registry_ref = registry_ref;
        self
    }

    pub fn set_explicit_registry_section(&mut self, registry_section: Option<String>) -> &mut Self {
        self.paths.explicit_registry_section = registry_section;
        self
    }

    /// The config layers that make up the resolved configuration, lowest
    /// priority first: the active registry, the project config, environment
    /// variables, then command-line flags. The global config is the only layer
    /// when none of those exists.
    pub fn config_layers(&self) -> Vec<ConfigLayer> {
        let sources = &self.resolution.sources;
        let mut layers = Vec::new();

        if let Some(registry) = sources.registries.iter().find(|r| r.is_active)
            && let Some(base) = registry.config.as_ref().and_then(|c| c.base.clone())
        {
            let mut location = format!("{}#{}", registry.url, registry.r#ref);
            if let Some(section) = &registry.section {
                location.push_str(&format!("/{section}"));
            }
            layers.push(ConfigLayer {
                kind: ConfigLayerKind::Registry,
                label: format!("registry:{}", registry.tag),
                detail: location,
                base,
            });
        }
        if let Some(base) = sources.repo_config.as_ref().and_then(|c| c.base.clone()) {
            layers.push(ConfigLayer {
                kind: ConfigLayerKind::Project,
                label: "project".to_string(),
                detail: self
                    .project_config_path()
                    .map(|path| path.display().to_string())
                    .unwrap_or_default(),
                base,
            });
        }
        if let Some(base) = sources.env_config.as_ref().and_then(|c| c.base.clone()) {
            layers.push(ConfigLayer {
                kind: ConfigLayerKind::Env,
                label: "env".to_string(),
                detail: "CW_* environment variables".to_string(),
                base,
            });
        }
        if let Some(base) = sources.cli_config.as_ref().and_then(|c| c.base.clone()) {
            layers.push(ConfigLayer {
                kind: ConfigLayerKind::Cli,
                label: "cli".to_string(),
                detail: "command-line flags".to_string(),
                base,
            });
        }

        // The global config is the fallback when nothing else exists, and
        // otherwise only a lower layer if the project opts in with
        // `[global] inherit = true`.
        let inherit = layers
            .iter()
            .fold(BaseConfig::empty(), |base, layer| {
                layer.base.clone().merge(base)
            })
            .global_inherit();
        if (layers.is_empty() || inherit)
            && let Some(layer) = self.global_layer()
        {
            layers.insert(0, layer);
        }
        layers
    }

    /// Layers that were found but do not take part, each with the reason.
    pub fn skipped_layers(&self) -> Vec<(ConfigLayer, String)> {
        let applied = self.config_layers();
        if applied
            .iter()
            .any(|layer| layer.kind == ConfigLayerKind::Global)
        {
            return Vec::new();
        }
        match self.global_layer() {
            Some(layer) if !applied.is_empty() => vec![(
                layer,
                "not applied: set `[global] inherit = true` in the project config to use it as a lower layer"
                    .to_string(),
            )],
            _ => Vec::new(),
        }
    }

    fn global_layer(&self) -> Option<ConfigLayer> {
        let base = self
            .resolution
            .sources
            .global_config
            .as_ref()
            .and_then(|c| c.base.clone())?;
        Some(ConfigLayer {
            kind: ConfigLayerKind::Global,
            label: "global".to_string(),
            detail: crate::engine::constants::resolve_global_config_path()
                .map(|path| path.display().to_string())
                .unwrap_or_default(),
            base,
        })
    }

    pub fn project_config_path(&self) -> Option<PathBuf> {
        resolve_project_config_path(
            self.cwd(),
            Some(self.repo_root().as_path()),
            self.in_git_repo(),
            self.explicit_config_path().map(|p| p.as_path()),
        )
    }
}
