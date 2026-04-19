use crate::engine::models::runtime::ResolvedConfig;

#[derive(Debug, Clone, Default)]
pub struct HooksModel {
    pub pre_commit: bool,
    pub commit_msg: bool,
    pub pre_push: bool,
}

impl HooksModel {
    pub fn from_config(config: &ResolvedConfig) -> Self {
        let base = &config.base;
        Self {
            pre_commit: base.hooks_pre_commit(),
            commit_msg: base.hooks_commit_msg(),
            pre_push: base.hooks_pre_push(),
        }
    }
}
