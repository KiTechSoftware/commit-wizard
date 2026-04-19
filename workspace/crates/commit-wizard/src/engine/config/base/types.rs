use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Deserializer, Serialize};

use crate::engine::models::policy::enforcement::{
    BumpLevel, ChangelogFormat, CommitEnforcementScope, EmojiMode, ScopeMode, TicketSource,
};

/// Deserialize Option<String>, treating empty strings as None
fn deserialize_optional_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    Ok(s.and_then(|s| if s.is_empty() { None } else { Some(s) }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommitConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_max_length: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_emojis: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub types: Option<BTreeMap<String, CommitTypeConfig>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scopes: Option<CommitScopesConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub breaking: Option<BreakingConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protected: Option<CommitProtectedConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticket: Option<TicketConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommitTypeConfig {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub emoji: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bump: Option<BumpLevel>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub section: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommitScopesConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<ScopeMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restrict_to_defined: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definitions: Option<HashMap<String, ScopeDefinition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeDefinition {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub title: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BreakingConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_header: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_footer: Option<bool>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub footer_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub footer_keys: Option<Vec<String>>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub emoji: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji_mode: Option<EmojiMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommitProtectedConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warn: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TicketConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub pattern: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<TicketSource>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub header_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BranchConfig {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub remote: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protected: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub naming: Option<BranchNamingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BranchNamingConfig {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<PrTitleConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<PrBranchConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrTitleConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_conventional: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_ticket: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_mode: Option<ScopeMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrBranchConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_source: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_target: Option<bool>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub source_pattern: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_allowed: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_conventional: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commits: Option<CheckCommitsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckCommitsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enforce_on: Option<CommitEnforcementScope>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PushConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow: Option<PushAllowConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check: Option<PushCheckConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PushAllowConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protected: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PushCheckConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commits: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_policy: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersioningConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangelogConfig {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub output: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<ChangelogFormat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header: Option<ChangelogHeaderConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<ChangelogLayoutConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sections: Option<BTreeMap<String, ChangelogSectionConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangelogHeaderConfig {
    #[serde(rename = "use", default, skip_serializing_if = "Option::is_none")]
    pub use_header: Option<bool>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub title: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangelogSectionConfig {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangelogLayoutConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_by: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section_order: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_order: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_scope: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_empty_sections: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_empty_scopes: Option<bool>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub misc_section: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub unreleased_label: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub date_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub source_branch: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub target_branch: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub branch_format: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub hotfix_pattern: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation: Option<ReleaseValidationConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish: Option<ReleaseFinishConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseValidationConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_clean_worktree: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_if_tag_exists: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_if_release_branch_exists: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseFinishConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub push: Option<bool>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub backmerge_branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HooksConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_commit: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_msg: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_push: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commands: Option<AiCommandsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiCommandsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changelog: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_prepare: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryConfig {
    #[serde(
        rename = "use",
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub use_registry: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub section: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedRegistryConfig {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub url: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub r#ref: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub section: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<String>>,
}
