use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistrySelection {
    pub name: Option<String>,
    pub url: String,
    pub r#ref: String,
    pub section: Option<String>,
}

impl RegistrySelection {
    pub fn cache_key(&self) -> String {
        format!("{}#{}", self.url, self.r#ref)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterializedRegistry {
    pub selection: RegistrySelection,
    pub cache_path: PathBuf,
    pub resolved_commit: String,
}

#[derive(Debug, Clone)]
pub struct RegistryFiles {
    pub config_toml: String,
    pub rules_toml: String,
}
