//! Policy model for branch rules.

use crate::engine::models::runtime::ResolvedConfig;

#[derive(Debug, Clone)]
pub struct BranchModel {
    pub remote: String,
    pub protected_patterns: Vec<String>,
    pub naming_pattern: String,
    pub require_type: bool,
    pub require_ticket: bool,
    pub enforce_naming: bool,
    pub allowed_targets: Option<Vec<String>>,
}

impl Default for BranchModel {
    fn default() -> Self {
        Self {
            remote: "origin".to_string(),
            protected_patterns: vec![
                "main".to_string(),
                "master".to_string(),
                "release/*".to_string(),
            ],
            naming_pattern: crate::engine::constants::BRANCH_NAMING_PATTERN.to_string(),
            require_type: false,
            require_ticket: false,
            enforce_naming: false,
            allowed_targets: None,
        }
    }
}

impl BranchModel {
    pub fn from_config(config: &ResolvedConfig) -> Self {
        let base = &config.base;

        Self {
            remote: base.branch_remote(),
            protected_patterns: base.branch_protected_patterns(),
            naming_pattern: base.branch_naming_pattern(),
            require_type: base.branch_naming_require_type(),
            require_ticket: base.branch_naming_require_ticket(),
            enforce_naming: base.branch_naming_enforce(),
            allowed_targets: if base.branch_allowed_targets().is_empty() {
                None
            } else {
                Some(base.branch_allowed_targets())
            },
        }
    }

    pub fn is_protected_pattern_configured(&self) -> bool {
        !self.protected_patterns.is_empty()
    }
}
