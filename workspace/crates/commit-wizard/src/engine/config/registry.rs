/// Registry resolution (SRS §5).
///
/// Resolves registry configuration from CLI flags, ENV variables, and the
/// project/global config. Supports both local-path registries and Git-URL
/// registries (cloned/fetched into the user-level cache directory).
///
/// Precedence: CLI > ENV > config
use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::engine::{
    Error, ErrorCode, LoggerTrait,
    config::{
        BaseConfig, RulesConfig,
        env::get_env_registry_params,
        resolver::{load_rules_config, load_standard_config},
    },
    models::runtime::resolution::{AvailableConfig, resolve_available_config},
};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Fully-resolved registry parameters.
#[derive(Debug, Clone)]
pub struct RegistrySpec {
    pub url: String,
    pub r#ref: String,
    pub section: Option<String>,
}

/// Registry load result with config and resolved commit hash.
#[derive(Debug, Clone)]
pub struct RegistryLoadResult {
    pub config: AvailableConfig,
    pub resolved_commit: String,
}

// ---------------------------------------------------------------------------
// Registry selection
// ---------------------------------------------------------------------------

/// Determine which registry to use, applying CLI > ENV > config precedence.
///
/// Returns `None` when no registry is configured from any source.
pub fn resolve_registry_spec(
    cli_url: Option<&str>,
    cli_ref: Option<&str>,
    cli_section: Option<&str>,
    base_config: Option<&BaseConfig>,
) -> Option<RegistrySpec> {
    // --- URL ---
    let env_params = get_env_registry_params();

    let url = cli_url
        .map(str::to_owned)
        .or_else(|| env_params.url.clone())
        .or_else(|| resolve_url_from_config(base_config))?;

    // --- ref ---
    let resolved_ref = cli_ref
        .map(str::to_owned)
        .or_else(|| env_params.r#ref.clone())
        .or_else(|| resolve_ref_from_config(base_config, &url))
        .unwrap_or_else(|| "HEAD".to_string());

    // --- section ---
    let section = cli_section
        .map(str::to_owned)
        .or_else(|| env_params.section.clone())
        .or_else(|| resolve_section_from_top_level(base_config))
        .or_else(|| resolve_section_from_named_registry(base_config, &url));

    Some(RegistrySpec {
        url,
        r#ref: resolved_ref,
        section,
    })
}

fn resolve_url_from_config(config: Option<&BaseConfig>) -> Option<String> {
    let config = config?;
    let use_name = config.registry_use()?;
    let registries = config.registries_map();
    registries.get(&use_name).and_then(|r| r.url.clone())
}

fn resolve_ref_from_config(config: Option<&BaseConfig>, url: &str) -> Option<String> {
    let config = config?;
    let use_name = config.registry_use()?;
    let registries = config.registries_map();
    // Match by name or by URL
    registries
        .iter()
        .find(|(name, r)| *name == &use_name || r.url.as_deref() == Some(url))
        .and_then(|(_, r)| r.r#ref.clone())
}

fn resolve_section_from_top_level(config: Option<&BaseConfig>) -> Option<String> {
    config?.registry.as_ref().and_then(|r| r.section.clone())
}

fn resolve_section_from_named_registry(config: Option<&BaseConfig>, url: &str) -> Option<String> {
    let config = config?;
    let use_name = config.registry_use()?;
    let registries = config.registries_map();

    registries
        .iter()
        .find(|(name, r)| *name == &use_name || r.url.as_deref() == Some(url))
        .and_then(|(_, r)| {
            // Try single section field first (backward compat)
            r.section
                .clone()
                // Fall back to first section in sections array
                .or_else(|| r.sections.as_ref().and_then(|s| s.first().cloned()))
        })
}

// ---------------------------------------------------------------------------
// Registry loading
// ---------------------------------------------------------------------------

/// Load registry configuration from the given spec.
///
/// Returns both the configuration and the resolved commit hash.
///
/// Supports:
/// - Local paths (starts with `/`, `./`, `../`, or is an existing directory)
///   - Loaded directly from disk, no caching or networking
/// - Git URLs (anything else — cloned/fetched into the cache)
///   - Implements smart-sync (SRS §10.4-10.5):
///     - Uses cache if available (no network call)
///     - Only fetches if remote has changes or ref is not a version tag
///     - Version tags skip fetch entirely (assumed stable)
pub fn load_registry(
    spec: &RegistrySpec,
    cache_dir: &Path,
    state_file: &Path,
    logger: &dyn LoggerTrait,
) -> Result<RegistryLoadResult, Error> {
    logger.debug(&format!(
        "[registry] loading: url={}, ref={}, section={}, local={}",
        spec.url,
        spec.r#ref,
        spec.section.as_deref().unwrap_or("(root)"),
        is_local_path(&spec.url),
    ));
    if is_local_path(&spec.url) {
        load_local_registry(spec, logger)
    } else {
        // Evict stale cache when the ref has changed for the same URL
        evict_stale_cache(spec, state_file, logger);
        load_git_registry(spec, cache_dir, logger)
    }
}

fn is_local_path(url: &str) -> bool {
    url.starts_with('/')
        || url.starts_with("./")
        || url.starts_with("../")
        || url == "."
        || url == ".."
        || Path::new(url).exists()
}

// ---------------------------------------------------------------------------
// Stale cache eviction
// ---------------------------------------------------------------------------

/// Evict the old cache directory when the registry ref has changed for the same URL.
///
/// Reads the previous state from state.json. If the URL matches but the ref has changed,
/// removes the old cache directory so the new ref gets a fresh clone. This prevents
/// ref conflicts and reclaims disk space from orphaned caches.
fn evict_stale_cache(spec: &RegistrySpec, state_file: &Path, logger: &dyn LoggerTrait) {
    use crate::engine::models::state::AppState;

    let state = match AppState::load(state_file) {
        Ok(s) => s,
        Err(_) => return,
    };

    let prev = match state.registry {
        Some(r) => r,
        None => return,
    };

    // Only act when the URL is the same but the ref has changed
    if prev.url != spec.url || prev.r#ref == spec.r#ref {
        return;
    }

    logger.debug(&format!(
        "[registry] ref changed ({} → {}) — evicting old cache",
        prev.r#ref, spec.r#ref,
    ));

    let old_cache = PathBuf::from(&prev.cache_path);
    if old_cache.exists() {
        match std::fs::remove_dir_all(&old_cache) {
            Ok(_) => logger.debug(&format!("[registry] evicted old cache: {:?}", old_cache)),
            Err(e) => logger.warn(&format!(
                "[registry] failed to evict old cache {:?}: {e}",
                old_cache
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Local-path registry
// ---------------------------------------------------------------------------

fn load_local_registry(
    spec: &RegistrySpec,
    logger: &dyn LoggerTrait,
) -> Result<RegistryLoadResult, Error> {
    let base = PathBuf::from(&spec.url);
    let dir = match &spec.section {
        Some(section) => base.join(section),
        None => base,
    };

    logger.debug(&format!("[registry] local path resolved to: {:?}", dir));

    if !dir.exists() {
        logger.warn(&format!("[registry] local path does not exist: {:?}", dir));
        return Err(ErrorCode::RegistrySectionMissing
            .error()
            .with_context("path", dir.display().to_string()));
    }

    let config = read_registry_dir(&dir, logger)?;

    Ok(RegistryLoadResult {
        config,
        // For local registries, we can't get a commit hash, so use a placeholder
        resolved_commit: "local".to_string(),
    })
}

// ---------------------------------------------------------------------------
// Git-URL registry
// ---------------------------------------------------------------------------

fn load_git_registry(
    spec: &RegistrySpec,
    cache_dir: &Path,
    logger: &dyn LoggerTrait,
) -> Result<RegistryLoadResult, Error> {
    let registry_id = registry_cache_id(&spec.url, &spec.r#ref);
    let registry_path = cache_dir.join("registries").join(&registry_id);

    logger.debug(&format!(
        "[registry] cache path: {:?}, exists={}",
        registry_path,
        registry_path.exists()
    ));

    if registry_path.exists() {
        // Cache hit: use it as-is for version tags (immutable), sync for HEAD/branches.
        if is_version_tag(&spec.r#ref) {
            logger.debug(&format!(
                "[registry] cache hit, version tag {} — skipping sync",
                spec.r#ref
            ));
        } else {
            logger.debug(&format!(
                "[registry] cache hit, ref={} — checking for remote changes",
                spec.r#ref
            ));
            maybe_sync_registry(&registry_path, spec, logger)?;
        }
    } else {
        // No cache: clone and checkout (unless it's a version tag, where --branch positions HEAD).
        logger.debug("[registry] no cache — cloning repository");
        clone_registry(&registry_path, spec)?;
        if !is_version_tag(&spec.r#ref) {
            logger.debug(&format!("[registry] checking out ref: {}", spec.r#ref));
            checkout_registry(&registry_path, spec)?;
        }
    }

    let dir = match &spec.section {
        Some(section) => registry_path.join(section),
        None => registry_path.clone(),
    };

    logger.debug(&format!(
        "[registry] reading config from dir: {:?}, exists={}",
        dir,
        dir.exists()
    ));

    if !dir.exists() {
        logger.warn(&format!(
            "[registry] section directory not found: {:?}",
            dir
        ));
        return Err(ErrorCode::RegistrySectionMissing
            .error()
            .with_context("section", spec.section.as_deref().unwrap_or("(root)"))
            .with_context("path", dir.display().to_string()));
    }

    let config = read_registry_dir(&dir, logger)?;
    let resolved_commit = get_resolved_commit(&registry_path)?;
    logger.trace(&format!("[registry] resolved commit: {}", resolved_commit));

    Ok(RegistryLoadResult {
        config,
        resolved_commit,
    })
}

fn registry_cache_id(url: &str, git_ref: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    format!("{url}#{git_ref}").hash(&mut h);
    format!("{:x}", h.finish())
}

/// Returns the cache directory path for a registry, matching the path used during cloning.
/// Section MUST NOT affect the cache path (SRS §7.2).
pub fn registry_cache_path(url: &str, git_ref: &str, cache_dir: &Path) -> PathBuf {
    let id = registry_cache_id(url, git_ref);
    cache_dir.join("registries").join(id)
}

fn run_git(args: &[&str], cwd: &Path) -> Result<(), Error> {
    let output = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .map_err(|e| {
            ErrorCode::GitCommandFailed
                .error()
                .with_context("command", format!("git {}", args.join(" ")))
                .with_context("error", e.to_string())
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        return Err(ErrorCode::RegistrySyncFailed
            .error()
            .with_context("command", format!("git {}", args.join(" ")))
            .with_context("stderr", stderr)
            .with_context("stdout", stdout));
    }
    Ok(())
}

fn clone_registry(dest: &Path, spec: &RegistrySpec) -> Result<(), Error> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            ErrorCode::RegistrySyncFailed
                .error()
                .with_context("action", "create cache directory")
                .with_context("path", parent.display().to_string())
                .with_context("error", e.to_string())
        })?;
    }

    let dest_str = dest.to_str().ok_or_else(|| {
        ErrorCode::RegistrySyncFailed
            .error()
            .with_context("action", "clone registry")
            .with_context("error", "cache path contains non-UTF8 characters")
            .with_context("path", dest.display().to_string())
    })?;

    run_git(
        // For tagged refs use --branch to fetch only that tag, keeping the clone shallow.
        // For branch refs use --no-single-branch so all branches are available.
        &if is_version_tag(&spec.r#ref) {
            vec![
                "clone",
                "--depth",
                "1",
                "--branch",
                &spec.r#ref,
                &spec.url,
                dest_str,
            ]
        } else {
            vec![
                "clone",
                "--depth",
                "1",
                "--no-single-branch",
                &spec.url,
                dest_str,
            ]
        },
        dest.parent().unwrap_or(Path::new(".")),
    )
    .map_err(|e| {
        e.with_context("url", spec.url.clone())
            .with_context("action", "clone registry")
    })
}

/// Smart-sync: fetch and checkout only if the remote has changes (SRS §10.5).
///
/// Fetch is skipped if:
/// - ref is a version tag (e.g., v1.0, v2.3.1) — handled by the caller before this is invoked
/// - remote has no changes (dirty check)
fn maybe_sync_registry(
    dest: &Path,
    spec: &RegistrySpec,
    logger: &dyn LoggerTrait,
) -> Result<(), Error> {
    if has_remote_changes(dest)? {
        logger.info("[registry] remote has changes — fetching");
        fetch_registry(dest, spec)?;
        logger.info(&format!("[registry] checking out ref: {}", spec.r#ref));
        checkout_registry(dest, spec)?;
    } else {
        logger.info("[registry] no remote changes — using cache");
    }
    Ok(())
}

/// Check if a git ref is a version tag (e.g., v1.0, v2.3.1, v1.0-beta).
///
/// Returns true if ref looks like a semantic version tag.
fn is_version_tag(r#ref: &str) -> bool {
    // Match patterns like v1, v1.0, v1.0.0, v1.0-beta, v1.0.0-rc1
    r#ref.starts_with('v')
        && r#ref.len() > 1
        && r#ref[1..].chars().next().map_or(false, char::is_numeric)
}

/// Check if a git repository has remote changes.
///
/// Compares local HEAD with origin/HEAD to determine if sync is needed.
/// Returns true if remote is ahead of local (dirty).
fn has_remote_changes(repo_path: &Path) -> Result<bool, Error> {
    // Fetch to get latest remote refs (but don't checkout)
    let _ = run_git(&["fetch", "--depth", "1", "origin"], repo_path);

    // Get current HEAD
    let local_head = get_resolved_commit(repo_path)?;

    // Try to get remote HEAD (for current branch); if fails, assume no changes
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["rev-parse", "origin/HEAD"])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let remote_head = String::from_utf8_lossy(&o.stdout).trim().to_string();
            // If local and remote HEAD differ, there are changes
            Ok(local_head != remote_head)
        }
        _ => {
            // If we can't get remote HEAD, assume no changes (be conservative)
            Ok(false)
        }
    }
}

fn fetch_registry(dest: &Path, spec: &RegistrySpec) -> Result<(), Error> {
    run_git(&["fetch", "--depth", "1", "origin"], dest).map_err(|e| {
        e.with_context("url", spec.url.clone())
            .with_context("action", "fetch registry updates")
    })
}

fn checkout_registry(dest: &Path, spec: &RegistrySpec) -> Result<(), Error> {
    run_git(&["checkout", &spec.r#ref], dest).map_err(|e| {
        e.with_context("ref", spec.r#ref.clone())
            .with_context("action", "checkout registry ref")
    })
}

fn get_resolved_commit(repo_path: &Path) -> Result<String, Error> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|e| {
            ErrorCode::GitCommandFailed
                .error()
                .with_context("command", "git rev-parse HEAD")
                .with_context("error", e.to_string())
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(ErrorCode::RegistrySyncFailed
            .error()
            .with_context("command", "git rev-parse HEAD")
            .with_context("stderr", stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// ---------------------------------------------------------------------------
// File reading
// ---------------------------------------------------------------------------

fn read_registry_dir(dir: &Path, logger: &dyn LoggerTrait) -> Result<AvailableConfig, Error> {
    let config_path = dir.join("config.toml");
    let rules_path = dir.join("rules.toml");

    logger.debug(&format!(
        "[registry] config.toml: {:?}, exists={}",
        config_path,
        config_path.exists()
    ));
    logger.debug(&format!(
        "[registry] rules.toml: {:?}, exists={}",
        rules_path,
        rules_path.exists()
    ));

    // config.toml is required (SRS §5)
    if !config_path.exists() {
        logger.warn(&format!(
            "[registry] config.toml missing at {:?}",
            config_path
        ));
        return Err(ErrorCode::RegistryInvalid
            .error()
            .with_context("missing_file", config_path.display().to_string()));
    }

    let base: Option<BaseConfig> = load_standard_config(&config_path).map(|sc| {
        use crate::engine::config::resolver::extract_config_from_standard_config;
        extract_config_from_standard_config(&sc)
    });

    match &base {
        Some(b) => logger.trace(&format!(
            "[registry] config.toml parsed ok: commit.types={:?}",
            b.commit
                .as_ref()
                .and_then(|c| c.types.as_ref())
                .map(|t| t.keys().cloned().collect::<Vec<_>>()),
        )),
        None => {
            logger.warn("[registry] config.toml exists but failed to parse — base config is None")
        }
    }

    // rules.toml is optional (SRS §5)
    let rules: Option<RulesConfig> = rules_path
        .exists()
        .then(|| load_rules_config(&rules_path))
        .flatten();

    match &rules {
        Some(_) => logger.debug("[registry] rules.toml parsed ok"),
        None if rules_path.exists() => {
            logger.warn("[registry] rules.toml exists but failed to parse")
        }
        None => logger.debug("[registry] rules.toml not present (optional)"),
    }

    Ok(resolve_available_config(base, rules))
}
