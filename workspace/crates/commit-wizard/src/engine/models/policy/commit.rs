use crate::engine::{
    constants::DEFAULT_COMMIT_BUMP,
    models::{
        policy::enforcement::{BumpLevel, EmojiMode, ScopeMode, TicketSource},
        runtime::ResolvedConfig,
    },
};
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeRequirement {
    Disabled,
    Optional,
    Required,
}

impl From<ScopeMode> for ScopeRequirement {
    fn from(value: ScopeMode) -> Self {
        match value {
            ScopeMode::Disabled => Self::Disabled,
            ScopeMode::Optional => Self::Optional,
            ScopeMode::Required => Self::Required,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommitTypeModel {
    pub key: String,
    pub emoji: Option<String>,
    pub description: Option<String>,
    pub bump: BumpLevel,
    pub section: String,
}

#[derive(Debug, Clone)]
pub struct CommitProtectedModel {
    pub allow: bool,
    pub force: bool,
    pub warn: bool,
}

#[derive(Debug, Clone)]
pub struct TicketPolicy {
    pub enabled: bool,
    pub required: bool,
    pub regex: Option<String>,
    pub source: TicketSource,
}

#[derive(Debug, Clone)]
pub struct HeaderFormatPolicy {
    pub require_scope: bool,
    pub allow_breaking_bang: bool,
}

#[derive(Debug, Clone)]
pub struct CommitModel {
    pub subject_max_length: u32,
    pub use_emojis: bool,
    pub types: Vec<CommitTypeModel>,

    pub scope_requirement: ScopeRequirement,
    pub restrict_scopes_to_defined: bool,
    pub allowed_scopes: Option<Vec<String>>,

    pub require_conventional: bool,
    pub breaking_header_required: bool,
    pub breaking_footer_required: bool,
    pub breaking_footer_key: String,
    pub breaking_footer_keys: Vec<String>,
    pub breaking_emoji: Option<String>,
    pub breaking_emoji_mode: EmojiMode,

    pub ticket: TicketPolicy,
    pub header_format: HeaderFormatPolicy,

    pub protected: CommitProtectedModel,
}

impl Default for CommitModel {
    fn default() -> Self {
        Self {
            subject_max_length: 72,
            use_emojis: false,
            types: vec![
                CommitTypeModel {
                    key: "feat".to_string(),
                    emoji: Some("✨".to_string()),
                    description: Some("A new feature".to_string()),
                    bump: BumpLevel::Minor,
                    section: "Features".to_string(),
                },
                CommitTypeModel {
                    key: "fix".to_string(),
                    emoji: Some("🐛".to_string()),
                    description: Some("A bug fix".to_string()),
                    bump: BumpLevel::Patch,
                    section: "Bug Fixes".to_string(),
                },
            ],
            scope_requirement: ScopeRequirement::Optional,
            restrict_scopes_to_defined: false,
            allowed_scopes: None,
            require_conventional: true,
            breaking_header_required: false,
            breaking_footer_required: false,
            breaking_footer_key: "BREAKING CHANGE".to_string(),
            breaking_footer_keys: vec![
                "BREAKING CHANGE".to_string(),
                "BREAKING-CHANGE".to_string(),
            ],
            breaking_emoji: None,
            breaking_emoji_mode: EmojiMode::Prepend,
            ticket: TicketPolicy {
                enabled: false,
                required: false,
                regex: None,
                source: TicketSource::Disabled,
            },
            header_format: HeaderFormatPolicy {
                require_scope: false,
                allow_breaking_bang: true,
            },
            protected: CommitProtectedModel {
                allow: true,
                force: false,
                warn: true,
            },
        }
    }
}

impl CommitModel {
    pub fn from_config(config: &ResolvedConfig) -> Self {
        let base = &config.base;

        let types_map = base.commit_types();

        let types = types_map
            .into_iter()
            .map(|(key, value)| CommitTypeModel {
                key,
                emoji: value.emoji,
                description: value.description,
                bump: value.bump.unwrap_or(DEFAULT_COMMIT_BUMP),
                section: value.section.unwrap_or_else(|| "Miscellaneous".to_string()),
            })
            .collect();

        let allowed_scopes_vec = base.commit_scope_allowed();
        let allowed_scopes = if allowed_scopes_vec.is_empty() {
            None
        } else {
            Some(allowed_scopes_vec)
        };

        Self {
            subject_max_length: base.commit_subject_max_length(),
            use_emojis: base.commit_use_emojis(),

            types,

            scope_requirement: base.commit_scopes_mode().into(),
            restrict_scopes_to_defined: base.commit_scope_restrict_to_defined(),
            allowed_scopes,

            require_conventional: base.check_require_conventional(),
            breaking_header_required: base.commit_breaking_require_header(),
            breaking_footer_required: base.commit_breaking_require_footer(),
            breaking_footer_key: base.commit_breaking_footer_key(),
            breaking_footer_keys: base.commit_breaking_footer_keys_normalized(),
            breaking_emoji: base.commit_breaking_emoji(),
            breaking_emoji_mode: base.commit_breaking_emoji_mode(),

            ticket: TicketPolicy {
                enabled: base.commit_ticket_required(),
                required: base.commit_ticket_required(),
                regex: base.commit_ticket_pattern(),
                source: base.commit_ticket_source(),
            },

            header_format: HeaderFormatPolicy {
                require_scope: matches!(base.commit_scopes_mode(), ScopeMode::Required),
                allow_breaking_bang: true,
            },

            protected: CommitProtectedModel {
                allow: base.commit_protected_allow(),
                force: base.commit_protected_force(),
                warn: base.commit_protected_warn(),
            },
        }
    }

    pub fn allows_type(&self, value: &str) -> bool {
        self.types.iter().any(|t| t.key == value)
    }

    pub fn find_type(&self, value: &str) -> Option<&CommitTypeModel> {
        self.types.iter().find(|t| t.key == value)
    }

    pub fn allows_scope(&self, value: &str) -> bool {
        match &self.allowed_scopes {
            Some(scopes) => scopes.iter().any(|s| s == value),
            None => true,
        }
    }
}

/// Validates a commit message against conventional commits.
pub fn is_valid_conventional_commit_message(message: &str, allowed_types: &[&str]) -> bool {
    let pattern = format!(r"^({})(\([\w\-]+\))?(!)?: .+", allowed_types.join("|"));
    Regex::new(&pattern).unwrap().is_match(message.trim())
}
