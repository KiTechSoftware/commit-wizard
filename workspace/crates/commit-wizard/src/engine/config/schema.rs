use serde::{Deserialize, Serialize};

use crate::engine::{ErrorCode, Result};

// external schema for config documents (global, registry, project)
use super::{BaseConfig, RulesConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Versioned<T> {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    #[serde(flatten)]
    pub inner: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBody {
    #[serde(flatten)]
    pub config: BaseConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules: Option<RulesConfig>,
}

// external representation of global and registry config
pub type StandardConfig = Versioned<BaseConfig>;
// external representation of project config
pub type ProjectConfig = Versioned<ProjectBody>;

impl StandardConfig {
    pub fn from_toml_str(input: &str) -> Result<Self> {
        toml::from_str(input).map_err(|err| {
            ErrorCode::ConfigInvalid
                .error()
                .with_context("error", err.to_string())
        })
    }

    pub fn minimal() -> Self {
        Self {
            version: None,
            inner: BaseConfig::minimal(),
        }
    }

    pub fn standard() -> Self {
        Self {
            version: None,
            inner: BaseConfig::standard(),
        }
    }

    pub fn full() -> Self {
        Self {
            version: None,
            inner: BaseConfig::full(),
        }
    }
}

impl ProjectConfig {
    pub fn from_toml_str(input: &str) -> Result<Self> {
        toml::from_str(input).map_err(|err| {
            ErrorCode::ConfigInvalid
                .error()
                .with_context("error", err.to_string())
        })
    }

    pub fn minimal() -> Self {
        Self {
            version: None,
            inner: ProjectBody {
                config: BaseConfig::minimal(),
                rules: None,
            },
        }
    }

    pub fn standard() -> Self {
        Self {
            version: None,
            inner: ProjectBody {
                config: BaseConfig::standard(),
                rules: None,
            },
        }
    }

    pub fn full() -> Self {
        Self {
            version: None,
            inner: ProjectBody {
                config: BaseConfig::full(),
                rules: None,
            },
        }
    }
}
