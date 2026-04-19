use crate::engine::models::{policy::enforcement::CommitEnforcementScope, runtime::ResolvedConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitCheckEnforcement {
    AllBranches,
    ProtectedBranches,
    None,
}

impl From<CommitEnforcementScope> for CommitCheckEnforcement {
    fn from(value: CommitEnforcementScope) -> Self {
        match value {
            CommitEnforcementScope::AllBranches => Self::AllBranches,
            CommitEnforcementScope::ProtectedBranches => Self::ProtectedBranches,
            CommitEnforcementScope::None => Self::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CheckModel {
    pub require_conventional: bool,
    pub commits_enabled: bool,
    pub enforcement: CommitCheckEnforcement,
}

impl Default for CheckModel {
    fn default() -> Self {
        Self {
            require_conventional: true,
            commits_enabled: true,
            enforcement: CommitCheckEnforcement::ProtectedBranches,
        }
    }
}

impl CheckModel {
    pub fn from_config(config: &ResolvedConfig) -> Self {
        let base = &config.base;

        Self {
            require_conventional: base.check_require_conventional(),
            commits_enabled: base.check_commits_enabled(),
            enforcement: base.check_commits_enforce_on().into(),
        }
    }
}
