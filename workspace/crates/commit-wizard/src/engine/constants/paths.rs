use crate::engine::{
    constants::{
        CACHE_DIR_NAME, CONFIG_DIR_NAME, CONFIG_FILE_NAME, PROJECT_CONFIG_FILE_NAME,
        PROJECT_CONFIG_FILE_NAME_HIDDEN, RULES_FILE_NAME, STATE_DIR_NAME,
    },
    error::{ErrorCode, Result},
};
use std::{
    env,
    path::{Path, PathBuf},
};

pub fn resolve_repo_config_path(cwd: &Path) -> PathBuf {
    let hidden = cwd.join(PROJECT_CONFIG_FILE_NAME_HIDDEN);
    let visible = cwd.join(PROJECT_CONFIG_FILE_NAME);

    if hidden.exists() {
        hidden
    } else if visible.exists() {
        visible
    } else {
        hidden
    }
}

pub fn resolve_new_repo_config_path(cwd: &Path, hidden: bool) -> PathBuf {
    if hidden {
        cwd.join(PROJECT_CONFIG_FILE_NAME_HIDDEN) // .cwizard.toml
    } else {
        cwd.join(PROJECT_CONFIG_FILE_NAME) // cwizard.toml
    }
}

pub fn resolve_global_config_path() -> Result<PathBuf> {
    Ok(app_config_dir()?.join(CONFIG_FILE_NAME))
}

pub fn resolve_global_rules_path() -> Result<PathBuf> {
    Ok(app_config_dir()?.join(RULES_FILE_NAME))
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
    let hidden = dir.join(PROJECT_CONFIG_FILE_NAME_HIDDEN);
    if hidden.exists() {
        return Some(hidden);
    }

    let visible = dir.join(PROJECT_CONFIG_FILE_NAME);
    if visible.exists() {
        return Some(visible);
    }

    None
}
