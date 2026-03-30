use crate::domain::commit::CommitType;
use anyhow::Result;

pub trait PromptPort {
    fn ask_type(&self) -> Result<CommitType>;
    fn ask_scope(&self) -> Result<Option<String>>;
    fn ask_summary(&self) -> Result<String>;
    fn ask_body(&self) -> Result<Option<String>>;
    fn confirm_breaking(&self) -> Result<bool>;
}
