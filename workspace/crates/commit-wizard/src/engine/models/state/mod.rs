pub mod registry;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::engine::{Result, fs};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppState {
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registry: Option<RegistryState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub url: String,
    pub r#ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    pub resolved_commit: String,
    pub cache_path: String,
}

impl AppState {
    /// Creates a new AppState with the current version
    pub fn new() -> Self {
        Self {
            version: 1,
            registry: None,
        }
    }

    /// Loads AppState from the given path
    /// Returns a default state if the file doesn't exist
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            fs::load_json(path)
        } else {
            Ok(Self::new())
        }
    }

    /// Saves AppState to the given path
    pub fn save(&self, path: &Path) -> Result<()> {
        fs::save_json(path, self)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistryState {
    /// Creates a new RegistryState with resolved registry information
    pub fn new(
        name: Option<String>,
        url: String,
        r#ref: String,
        section: Option<String>,
        resolved_commit: String,
        cache_path: PathBuf,
    ) -> Self {
        Self {
            name,
            url,
            r#ref,
            section,
            resolved_commit,
            cache_path: cache_path.to_string_lossy().to_string(),
        }
    }
}
