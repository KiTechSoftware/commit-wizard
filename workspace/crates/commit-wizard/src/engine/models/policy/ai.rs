use crate::engine::models::{policy::enforcement::AiProvider, runtime::ResolvedConfig};

#[derive(Debug, Clone)]
pub struct AiCommandsModel {
    pub commit: bool,
    pub changelog: bool,
    pub release_prepare: bool,
}

#[derive(Debug, Clone)]
pub struct AiModel {
    pub enabled: bool,
    pub provider: AiProvider,
    pub commands: AiCommandsModel,
}

impl Default for AiModel {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: AiProvider::Copilot,
            commands: AiCommandsModel {
                commit: false,
                changelog: false,
                release_prepare: false,
            },
        }
    }
}

impl AiModel {
    pub fn from_config(config: &ResolvedConfig) -> Self {
        let base = &config.base;

        Self {
            enabled: base.ai_enabled(),
            provider: base.ai_provider(),
            commands: AiCommandsModel {
                commit: base.ai_commit_enabled(),
                changelog: base.ai_changelog_enabled(),
                release_prepare: base.ai_release_prepare_enabled(),
            },
        }
    }
}
