use std::path::{Path, PathBuf};

use crate::engine::{
    config::{ConfigFormat, ProjectConfig, StandardConfig},
    constants::paths::{resolve_global_config_path, resolve_repo_config_path},
    error::{ErrorCode, Result},
};

#[derive(Debug, Clone, Copy)]
pub enum ConfigTarget {
    Project,
    Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigShowFormat {
    Human,
    Json,
    Toml,
}

pub struct ConfigPathInput<'a> {
    pub cwd: &'a Path,
    pub target: ConfigTarget,
}

pub struct ConfigPathOutput {
    pub path: PathBuf,
}

pub struct ConfigShowInput<'a> {
    pub cwd: &'a Path,
    pub target: ConfigTarget,
    pub explicit_path: Option<&'a Path>,
    pub format: ConfigShowFormat,
}

pub struct ConfigShowOutput {
    pub path: PathBuf,
    pub exists: bool,
    pub content: String,
}

pub fn resolve_config_path(target: ConfigTarget, cwd: &Path) -> Result<PathBuf> {
    match target {
        ConfigTarget::Project => Ok(resolve_repo_config_path(cwd)),
        ConfigTarget::Global => resolve_global_config_path(),
    }
}

pub fn config_path(input: &ConfigPathInput<'_>) -> Result<ConfigPathOutput> {
    Ok(ConfigPathOutput {
        path: resolve_config_path(input.target, input.cwd)?,
    })
}

pub fn config_show(input: &ConfigShowInput<'_>) -> Result<ConfigShowOutput> {
    let path = if let Some(explicit_path) = input.explicit_path {
        explicit_path.to_path_buf()
    } else {
        resolve_config_path(input.target, input.cwd)?
    };

    if !path.exists() {
        return Err(ErrorCode::ConfigUnreadable
            .error()
            .with_context("path", path.display().to_string())
            .with_context("reason", "config file does not exist"));
    }

    let raw = std::fs::read_to_string(&path)?;

    let content = match input.target {
        ConfigTarget::Project => {
            let parsed = ProjectConfig::from_str_as(ConfigFormat::from_path(&path), &raw)?;
            match input.format {
                ConfigShowFormat::Human | ConfigShowFormat::Toml => raw,
                ConfigShowFormat::Json => serde_json::to_string_pretty(&parsed).map_err(|err| {
                    ErrorCode::SerializationFailure
                        .error()
                        .with_context("error", err.to_string())
                })?,
            }
        }
        ConfigTarget::Global => {
            let parsed = StandardConfig::from_str_as(ConfigFormat::from_path(&path), &raw)?;
            match input.format {
                ConfigShowFormat::Human | ConfigShowFormat::Toml => raw,
                ConfigShowFormat::Json => serde_json::to_string_pretty(&parsed).map_err(|err| {
                    ErrorCode::SerializationFailure
                        .error()
                        .with_context("error", err.to_string())
                })?,
            }
        }
    };

    Ok(ConfigShowOutput {
        path,
        exists: true,
        content,
    })
}
