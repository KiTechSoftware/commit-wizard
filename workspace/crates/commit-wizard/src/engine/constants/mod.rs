pub mod defaults;
pub mod emoji;
pub mod env;
pub mod paths;

pub use defaults::*;
pub use emoji::*;
pub use env::*;
pub use paths::*;

pub const APP_NAME: &str = "commit-wizard";
pub const BIN_NAME: &str = "cw";

pub const CONFIG_FILE_NAME: &str = "config.toml";
pub const RULES_FILE_NAME: &str = "rules.toml";
pub const STATE_FILE_NAME: &str = "state.json";
pub const REGISTRY_BASE_CONFIG_FILE_NAME: &str = "config.toml";
pub const REGISTRY_RULES_CONFIG_FILE_NAME: &str = "rules.toml";
pub const PROJECT_CONFIG_FILE_NAME: &str = "cwizard.toml";
pub const PROJECT_CONFIG_FILE_NAME_HIDDEN: &str = ".cwizard.toml";

pub const CONFIG_DIR_NAME: &str = "cwizard";

pub const COMMIT_FIX_FILE_NAME: &str = ".cw-fix.json";
pub const TEMP_FIX_FILE_NAME: &str = ".cw-fix-session.json";
pub const CACHE_DIR_NAME: &str = "cwizard";
pub const STATE_DIR_NAME: &str = "cwizard";
pub const REGISTRIES_DIR_NAME: &str = "registries";

pub fn default_branch_protected_patterns() -> Vec<String> {
    vec![
        "main".to_string(),
        "master".to_string(),
        "release/*".to_string(),
    ]
}

pub fn default_changelog_group_by() -> Vec<String> {
    vec!["type".to_string()]
}

pub fn default_changelog_section_order() -> Vec<String> {
    vec![
        "feat".to_string(),
        "fix".to_string(),
        "docs".to_string(),
        "style".to_string(),
        "refactor".to_string(),
        "perf".to_string(),
        "test".to_string(),
        "chore".to_string(),
    ]
}
