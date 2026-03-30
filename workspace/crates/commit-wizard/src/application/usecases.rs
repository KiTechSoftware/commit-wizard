use crate::domain::commit::{CommitMessage, CommitType};
use crate::ports::{
    git::{CommitOptions, GitPort},
    prompt::PromptPort,
};
use anyhow::Result;

pub struct CreateCommit<P: PromptPort> {
    prompt: P,
    git: Box<dyn GitPort>,
}

impl<P: PromptPort> CreateCommit<P> {
    pub fn new(prompt: P, git: Box<dyn GitPort>) -> Self {
        Self { prompt, git }
    }

    pub fn run(&self, opts: &CommitOptions) -> Result<()> {
        let ctype: CommitType = self.prompt.ask_type()?;
        let scope: Option<String> = self.prompt.ask_scope()?;
        let summary: String = self.prompt.ask_summary()?;
        let body: Option<String> = self.prompt.ask_body()?;
        let breaking: bool = self.prompt.confirm_breaking()?;

        let msg = CommitMessage::new(ctype, scope, summary, body, breaking)?;
        self.git.commit(&msg.render(), opts)
    }
}
