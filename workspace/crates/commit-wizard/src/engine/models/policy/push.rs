use crate::engine::models::runtime::ResolvedConfig;

#[derive(Debug, Clone)]
pub struct PushModel {
    pub allow_protected: bool,
    pub allow_force_override: bool,
    pub check_commits: bool,
    pub check_branch_policy: bool,
}

impl Default for PushModel {
    fn default() -> Self {
        Self {
            allow_protected: false,
            allow_force_override: false,
            check_commits: true,
            check_branch_policy: true,
        }
    }
}

impl PushModel {
    pub fn from_config(config: &ResolvedConfig) -> Self {
        let base = &config.base;

        Self {
            allow_protected: base.push_allow_protected(),
            allow_force_override: base.push_allow_force(),
            check_commits: base.push_check_commits(),
            check_branch_policy: base.push_check_branch_policy(),
        }
    }
}
