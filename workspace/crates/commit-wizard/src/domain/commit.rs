use crate::domain::errors::DomainError;
use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitType {
    Feat,
    Fix,
    Docs,
    Chore,
    Refactor,
    Test,
    Perf,
    Build,
    Ci,
    Style,
}

impl CommitType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommitType::Feat => "feat",
            CommitType::Fix => "fix",
            CommitType::Docs => "docs",
            CommitType::Chore => "chore",
            CommitType::Refactor => "refactor",
            CommitType::Test => "test",
            CommitType::Perf => "perf",
            CommitType::Build => "build",
            CommitType::Ci => "ci",
            CommitType::Style => "style",
        }
    }
}
impl Display for CommitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct CommitMessage {
    r#type: CommitType,
    scope: Option<String>,
    summary: String,
    body: Option<String>,
    breaking: bool,
}

impl CommitMessage {
    pub fn new(
        r#type: CommitType,
        scope: Option<String>,
        summary: String,
        body: Option<String>,
        breaking: bool,
    ) -> Result<Self, DomainError> {
        if summary.trim().is_empty() {
            return Err(DomainError::EmptySummary);
        }
        if summary.chars().count() > 72 {
            return Err(DomainError::SummaryTooLong(72));
        }
        Ok(Self {
            r#type,
            scope: scope.and_then(|s| {
                let t = s.trim().to_string();
                if t.is_empty() { None } else { Some(t) }
            }),
            summary: summary.trim().to_string(),
            body: body.and_then(|b| {
                let t = b.trim().to_string();
                if t.is_empty() { None } else { Some(t) }
            }),
            breaking,
        })
    }

    pub fn render(&self) -> String {
        let header = match &self.scope {
            Some(s) => format!("{}({}): {}", self.r#type, s, self.summary),
            None => format!("{}: {}", self.r#type, self.summary),
        };

        let mut sections = Vec::new();
        if self.breaking {
            sections.push(String::from("BREAKING CHANGE: yes"));
        }
        if let Some(b) = &self.body {
            sections.push(b.clone());
        }

        if sections.is_empty() {
            header
        } else {
            format!("{header}\n\n{}", sections.join("\n\n"))
        }
    }
}
