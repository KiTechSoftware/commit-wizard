use anyhow::Result;
use dialoguer::{Confirm, Input, Select};

use crate::{domain::commit::CommitType, ports::prompt::PromptPort};

#[derive(Default)]
pub struct TtyPrompt;

impl PromptPort for TtyPrompt {
    fn ask_type(&self) -> Result<CommitType> {
        let types = [
            ("feat", CommitType::Feat),
            ("fix", CommitType::Fix),
            ("docs", CommitType::Docs),
            ("chore", CommitType::Chore),
            ("refactor", CommitType::Refactor),
            ("test", CommitType::Test),
            ("perf", CommitType::Perf),
            ("build", CommitType::Build),
            ("ci", CommitType::Ci),
            ("style", CommitType::Style),
        ];
        let labels: Vec<&str> = types.iter().map(|(s, _)| *s).collect();

        let idx = Select::new()
            .with_prompt("Type")
            .items(&labels)
            .default(0)
            .interact()?;

        Ok(types[idx].1)
    }

    fn ask_scope(&self) -> Result<Option<String>> {
        let s: String = Input::new()
            .with_prompt("Scope (optional)")
            .allow_empty(true)
            .interact_text()?;
        let s = s.trim().to_string();
        Ok(if s.is_empty() { None } else { Some(s) })
    }

    fn ask_summary(&self) -> Result<String> {
        let summary: String = Input::new()
            .with_prompt("Summary (max 72 chars)")
            .validate_with(|input: &String| -> Result<(), &str> {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    return Err("summary cannot be empty");
                }
                if trimmed.chars().count() > 72 {
                    return Err("summary too long (max 72 chars)");
                }
                Ok(())
            })
            .interact_text()?;
        Ok(summary.trim().to_string())
    }

    fn ask_body(&self) -> Result<Option<String>> {
        let body: String = Input::new()
            .with_prompt("Body (optional, single line for now)")
            .allow_empty(true)
            .interact_text()?;
        let body = body.trim().to_string();
        Ok(if body.is_empty() { None } else { Some(body) })
    }

    fn confirm_breaking(&self) -> Result<bool> {
        let yes = Confirm::new()
            .with_prompt("Breaking change?")
            .default(false)
            .interact()?;
        Ok(yes)
    }
}
