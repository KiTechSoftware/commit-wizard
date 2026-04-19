use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScopeMode {
    Disabled,
    Optional,
    Required,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketSource {
    Branch,
    Prompt,
    BranchOrPrompt,
    Disabled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommitEnforcementScope {
    AllBranches,
    ProtectedBranches,
    None,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangelogFormat {
    Markdown,
    Json,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum EmojiMode {
    #[default]
    Prepend,
    Append,
    Replace,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BumpLevel {
    None,
    Patch,
    Minor,
    Major,
}

pub fn decide_bump(breaking_keywords: &[String], minor: &[String], patch: &[String]) -> BumpLevel {
    if !breaking_keywords.is_empty() {
        BumpLevel::Major
    } else if !minor.is_empty() {
        BumpLevel::Minor
    } else if !patch.is_empty() {
        BumpLevel::Patch
    } else {
        BumpLevel::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiProvider {
    Copilot,
}

impl AiProvider {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(provider: &str) -> AiProvider {
        match provider.to_lowercase().as_str() {
            "copilot" => AiProvider::Copilot,
            _ => AiProvider::Copilot, // default to Copilot if unknown
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            AiProvider::Copilot => "copilot",
        }
    }
}

impl std::fmt::Display for AiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
