use crate::engine::models::runtime::ResolvedConfig;

#[derive(Debug, Clone)]
pub struct ReleaseValidationModel {
    pub require_clean_worktree: bool,
    pub fail_if_tag_exists: bool,
    pub fail_if_release_branch_exists: bool,
}

#[derive(Debug, Clone)]
pub struct ReleaseFinishModel {
    pub tag: bool,
    pub push: bool,
    pub backmerge_branch: String,
}

#[derive(Debug, Clone)]
pub struct ReleaseModel {
    pub enabled: bool,
    pub source_branch: String,
    pub target_branch: String,
    pub branch_format: String,
    pub hotfix_pattern: String,
    pub validation: ReleaseValidationModel,
    pub finish: ReleaseFinishModel,
}

impl Default for ReleaseModel {
    fn default() -> Self {
        Self {
            enabled: false,
            source_branch: "main".to_string(),
            target_branch: "main".to_string(),
            branch_format: "release/{version}".to_string(),
            hotfix_pattern: "hotfix/*".to_string(),
            validation: ReleaseValidationModel {
                require_clean_worktree: true,
                fail_if_tag_exists: true,
                fail_if_release_branch_exists: true,
            },
            finish: ReleaseFinishModel {
                tag: true,
                push: true,
                backmerge_branch: "main".to_string(),
            },
        }
    }
}

impl ReleaseModel {
    pub fn from_config(config: &ResolvedConfig) -> Self {
        let base = &config.base;

        Self {
            enabled: base.release_enabled(),
            source_branch: base.release_source_branch(),
            target_branch: base.release_target_branch(),
            branch_format: base.release_branch_format(),
            hotfix_pattern: base.release_hotfix_pattern(),
            validation: ReleaseValidationModel {
                require_clean_worktree: base.release_require_clean_worktree(),
                fail_if_tag_exists: base.release_fail_if_tag_exists(),
                fail_if_release_branch_exists: base.release_fail_if_release_branch_exists(),
            },
            finish: ReleaseFinishModel {
                tag: base.release_finish_tag(),
                push: base.release_finish_push(),
                backmerge_branch: base.release_finish_backmerge_branch(),
            },
        }
    }
}
