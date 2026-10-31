//! Where config files live and how they are found, whichever of TOML, YAML or JSON they use.

use crate::engine::{
    constants::{
        CACHE_DIR_NAME, CONFIG_DIR_NAME, CONFIG_FILE_NAME, CONFIG_STEM, PROJECT_CONFIG_FILE_NAME,
        PROJECT_CONFIG_FILE_NAME_HIDDEN, PROJECT_CONFIG_STEM, PROJECT_CONFIG_STEM_HIDDEN,
        RULES_FILE_NAME, RULES_STEM, STATE_DIR_NAME,
    },
    error::{ErrorCode, Result},
};
use std::{
    env,
    path::{Path, PathBuf},
};

use crate::engine::config::ConfigFormat;

/// Config files named `<stem>.<ext>` that exist in `dir`, preferred first.
///
/// Every supported format is considered, in the order TOML, YAML (`.yaml`,
/// `.yml`), JSON. More than one result means the directory holds competing
/// copies of the same config; callers use the first and may warn about the rest.
pub fn config_files_in_dir(dir: &Path, stem: &str) -> Vec<PathBuf> {
    ConfigFormat::ALL
        .iter()
        .flat_map(|format| format.extensions().iter())
        .map(|extension| dir.join(format!("{stem}.{extension}")))
        .filter(|path| path.is_file())
        .collect()
}

/// Other project config files beside `chosen` that are being ignored.
///
/// A directory with both `.cwizard.toml` and `.cwizard.yaml` uses the first by
/// precedence; the rest are returned so the caller can warn about them.
pub fn competing_project_configs(chosen: &Path) -> Vec<PathBuf> {
    let Some(dir) = chosen.parent() else {
        return Vec::new();
    };
    project_config_files_in_dir(dir)
        .into_iter()
        .filter(|path| path != chosen)
        .collect()
}

/// The config file named `<stem>.<ext>` in `dir`, in any supported format.
pub fn find_config_file(dir: &Path, stem: &str) -> Option<PathBuf> {
    config_files_in_dir(dir, stem).into_iter().next()
}

/// Project config files in `dir`: `.cwizard.*` first, then `cwizard.*`.
pub fn project_config_files_in_dir(dir: &Path) -> Vec<PathBuf> {
    let mut files = config_files_in_dir(dir, PROJECT_CONFIG_STEM_HIDDEN);
    files.extend(config_files_in_dir(dir, PROJECT_CONFIG_STEM));
    files
}

pub fn resolve_repo_config_path(cwd: &Path) -> PathBuf {
    project_config_files_in_dir(cwd)
        .into_iter()
        .next()
        .unwrap_or_else(|| cwd.join(PROJECT_CONFIG_FILE_NAME_HIDDEN))
}

pub fn resolve_new_repo_config_path(cwd: &Path, hidden: bool) -> PathBuf {
    if hidden {
        cwd.join(PROJECT_CONFIG_FILE_NAME_HIDDEN) // .cwizard.toml
    } else {
        cwd.join(PROJECT_CONFIG_FILE_NAME) // cwizard.toml
    }
}

/// The global config: `config.toml`, `config.yaml`/`.yml` or `config.json` in
/// the app config directory. When none exists, the TOML path is returned.
pub fn resolve_global_config_path() -> Result<PathBuf> {
    let dir = app_config_dir()?;
    Ok(find_config_file(&dir, CONFIG_STEM).unwrap_or_else(|| dir.join(CONFIG_FILE_NAME)))
}

/// The global rules file, in any supported format.
pub fn resolve_global_rules_path() -> Result<PathBuf> {
    let dir = app_config_dir()?;
    Ok(find_config_file(&dir, RULES_STEM).unwrap_or_else(|| dir.join(RULES_FILE_NAME)))
}

pub fn app_config_dir() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|p| p.join(CONFIG_DIR_NAME))
            .ok_or_else(|| {
                ErrorCode::ConfigInvalid
                    .error()
                    .with_context("reason", "APPDATA is not set")
            })
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(xdg_config_home) = env::var_os("XDG_CONFIG_HOME") {
            return Ok(PathBuf::from(xdg_config_home).join(CONFIG_DIR_NAME));
        }

        env::var_os("HOME")
            .map(PathBuf::from)
            .map(|p| p.join(".config").join(CONFIG_DIR_NAME))
            .ok_or_else(|| {
                ErrorCode::ConfigInvalid
                    .error()
                    .with_context("reason", "neither XDG_CONFIG_HOME nor HOME is set")
            })
    }
}

pub fn app_state_dir() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .map(|p| p.join(STATE_DIR_NAME))
            .ok_or_else(|| {
                ErrorCode::ConfigInvalid
                    .error()
                    .with_context("reason", "LOCALAPPDATA is not set")
            })
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(xdg_state_home) = env::var_os("XDG_STATE_HOME") {
            return Ok(PathBuf::from(xdg_state_home).join(STATE_DIR_NAME));
        }

        env::var_os("HOME")
            .map(PathBuf::from)
            .map(|p| p.join(".local").join("state").join(STATE_DIR_NAME))
            .ok_or_else(|| {
                ErrorCode::ConfigInvalid
                    .error()
                    .with_context("reason", "neither XDG_STATE_HOME nor HOME is set")
            })
    }
}

pub fn app_cache_dir() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .map(|p| p.join(CACHE_DIR_NAME))
            .ok_or_else(|| {
                ErrorCode::ConfigInvalid
                    .error()
                    .with_context("reason", "LOCALAPPDATA is not set")
            })
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(xdg_cache_home) = env::var_os("XDG_CACHE_HOME") {
            return Ok(PathBuf::from(xdg_cache_home).join(CACHE_DIR_NAME));
        }

        env::var_os("HOME")
            .map(PathBuf::from)
            .map(|p| p.join(".cache").join(CACHE_DIR_NAME))
            .ok_or_else(|| {
                ErrorCode::ConfigInvalid
                    .error()
                    .with_context("reason", "neither XDG_CACHE_HOME nor HOME is set")
            })
    }
}

pub fn resolve_project_config_path(
    cwd: &Path,
    repo_root: Option<&Path>,
    in_git_repo: bool,
    explicit_config_path: Option<&Path>,
) -> Option<PathBuf> {
    if let Some(path) = explicit_config_path {
        return Some(path.to_path_buf());
    }

    // Check cwd first — a config here takes precedence over the repo root
    if let Some(path) = find_project_config_in_dir(cwd) {
        return Some(path);
    }

    // Fall back to repo root if we're inside a git repo and cwd isn't the root
    if in_git_repo
        && let Some(root) = repo_root
        && root != cwd
        && let Some(path) = find_project_config_in_dir(root)
    {
        return Some(path);
    }

    None
}

pub fn resolve_new_project_config_path(
    cwd: &Path,
    repo_root: Option<&Path>,
    in_git_repo: bool,
    explicit_config_path: Option<&Path>,
    hidden: bool,
) -> PathBuf {
    if let Some(path) = explicit_config_path {
        return path.to_path_buf();
    }

    let base_dir = if in_git_repo {
        repo_root.unwrap_or(cwd)
    } else {
        cwd
    };

    base_dir.join(if hidden {
        PROJECT_CONFIG_FILE_NAME_HIDDEN
    } else {
        PROJECT_CONFIG_FILE_NAME
    })
}

fn find_project_config_in_dir(dir: &Path) -> Option<PathBuf> {
    project_config_files_in_dir(dir).into_iter().next()
}
