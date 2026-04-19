use crate::engine::models::runtime::ResolvedConfig;

#[derive(Debug, Clone)]
pub struct BranchModel {
    pub remote: String,
    pub protected_patterns: Vec<String>,
    pub naming_pattern: String,
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
            naming_pattern: "feature/{issue}".to_string(),
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
