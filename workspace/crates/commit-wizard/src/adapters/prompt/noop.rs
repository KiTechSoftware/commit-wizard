use crate::{domain::commit::CommitType, ports::prompt::PromptPort};
use anyhow::Result;

#[derive(Default)]
pub struct NoopPrompt;

impl PromptPort for NoopPrompt {
    fn ask_type(&self) -> Result<CommitType> {
        Ok(CommitType::Feat)
    }
    fn ask_scope(&self) -> Result<Option<String>> {
        Ok(Some("core".into()))
    }
    fn ask_summary(&self) -> Result<String> {
        Ok("initial wiring".into())
    }
    fn ask_body(&self) -> Result<Option<String>> {
        Ok(None)
    }
    fn confirm_breaking(&self) -> Result<bool> {
        Ok(false)
    }
}
