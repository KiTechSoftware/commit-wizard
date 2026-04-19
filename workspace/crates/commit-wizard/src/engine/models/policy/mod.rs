use crate::engine::models::runtime::ResolvedConfig;

pub mod ai;
pub mod branch;
pub mod changelog;
pub mod check;
pub mod commit;
pub mod enforcement;
pub mod versioning;
// pub mod pr;
pub mod hooks;
pub mod push;
pub mod release;

#[derive(Debug, Clone, Default)]
pub struct Policy {
    pub commit: commit::CommitModel,
    pub branch: branch::BranchModel,
    pub check: check::CheckModel,
    pub push: push::PushModel,
    pub release: release::ReleaseModel,
    pub ai: ai::AiModel,
    pub changelog: changelog::ChangelogModel,
    pub hooks: hooks::HooksModel,
    pub versioning: versioning::VersioningModel,
}

impl Policy {
    pub fn from_config(config: &ResolvedConfig) -> Self {
        Self {
            commit: commit::CommitModel::from_config(config),
            branch: branch::BranchModel::from_config(config),
            check: check::CheckModel::from_config(config),
            push: push::PushModel::from_config(config),
            release: release::ReleaseModel::from_config(config),
            ai: ai::AiModel::from_config(config),
            changelog: changelog::ChangelogModel::from_config(config),
            hooks: hooks::HooksModel::from_config(config),
            versioning: versioning::VersioningModel::from_config(config),
        }
    }
}
