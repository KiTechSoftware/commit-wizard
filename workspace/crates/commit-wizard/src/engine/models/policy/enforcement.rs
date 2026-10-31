//! The enumerations that settings choose between (scope mode, ticket source, bump level and so on).

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

/// Written in lowercase (`bump = "minor"`); the capitalised spelling older
/// versions of `cw` wrote is still read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BumpLevel {
    #[serde(alias = "None")]
    None,
    #[serde(alias = "Patch")]
    Patch,
    #[serde(alias = "Minor")]
    Minor,
    #[serde(alias = "Major")]
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

#[cfg(test)]
mod bump_level_tests {
    use super::BumpLevel;

    #[derive(serde::Deserialize)]
    struct Holder {
        bump: BumpLevel,
    }

    #[test]
    fn reads_lowercase_and_the_older_capitalised_spelling() {
        for (text, level) in [
            ("patch", BumpLevel::Patch),
            ("Patch", BumpLevel::Patch),
            ("minor", BumpLevel::Minor),
            ("Major", BumpLevel::Major),
            ("none", BumpLevel::None),
        ] {
            let holder: Holder = toml::from_str(&format!("bump = \"{text}\"")).unwrap();
            assert_eq!(holder.bump, level, "{text}");
        }
    }

    #[test]
    fn writes_lowercase() {
        let text = toml::to_string(&toml::toml! { bump = "x" }).unwrap();
        assert!(text.contains("bump"));
        let value = toml::Value::try_from(BumpLevel::Minor).unwrap();
        assert_eq!(value.as_str(), Some("minor"));
    }
}
