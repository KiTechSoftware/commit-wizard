pub mod types;
pub use types::*;

use std::collections::{BTreeMap, HashMap};

use crate::engine::{
    constants::{
        CONFIG_VERSION, defaults, full_base_config, minimal_base_config, standard_base_config,
    },
    models::policy::enforcement::AiProvider,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaseConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<CommitConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<BranchConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pr: Option<PrConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check: Option<CheckConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub push: Option<PushConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub versioning: Option<VersioningConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changelog: Option<ChangelogConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release: Option<ReleaseConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<HooksConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai: Option<AiConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registry: Option<RegistryConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registries: Option<HashMap<String, NamedRegistryConfig>>,
}

impl BaseConfig {
    pub fn empty() -> Self {
        Self {
            commit: None,
            branch: None,
            pr: None,
            check: None,
            push: None,
            versioning: None,
            changelog: None,
            release: None,
            hooks: None,
            ai: None,
            registry: None,
            registries: None,
        }
    }

    /// Merge two configs. `self` has higher priority — its sections win over `lower` for each
    /// top-level section, falling back to `lower` only when `self`'s section is `None`.
    pub fn merge(self, lower: BaseConfig) -> BaseConfig {
        BaseConfig {
            commit: merge_commit_config(self.commit, lower.commit),
            branch: self.branch.or(lower.branch),
            pr: self.pr.or(lower.pr),
            check: self.check.or(lower.check),
            push: self.push.or(lower.push),
            versioning: self.versioning.or(lower.versioning),
            changelog: self.changelog.or(lower.changelog),
            release: self.release.or(lower.release),
            hooks: self.hooks.or(lower.hooks),
            ai: self.ai.or(lower.ai),
            registry: self.registry.or(lower.registry),
            registries: self.registries.or(lower.registries),
        }
    }

    pub fn minimal() -> Self {
        minimal_base_config()
    }

    pub fn standard() -> Self {
        standard_base_config()
    }

    pub fn full() -> Self {
        full_base_config()
    }

    pub fn version(&self) -> u32 {
        CONFIG_VERSION
    }

    // -------------------------------------------------------------------------
    // commit
    // -------------------------------------------------------------------------

    pub fn commit_subject_max_length(&self) -> u32 {
        self.commit
            .as_ref()
            .and_then(|c| c.subject_max_length)
            .unwrap_or(defaults::COMMIT_SUBJECT_MAX_LENGTH)
    }

    pub fn commit_use_emojis(&self) -> bool {
        self.commit
            .as_ref()
            .and_then(|c| c.use_emojis)
            .unwrap_or(defaults::COMMIT_USE_EMOJIS)
    }

    pub fn commit_types(&self) -> BTreeMap<String, CommitTypeConfig> {
        self.commit
            .as_ref()
            .and_then(|c| c.types.clone())
            .unwrap_or_else(defaults::default_commit_types)
    }

    pub fn commit_scopes_mode(&self) -> crate::engine::models::policy::enforcement::ScopeMode {
        self.commit
            .as_ref()
            .and_then(|c| c.scopes.as_ref())
            .and_then(|s| s.mode)
            .unwrap_or(defaults::COMMIT_SCOPE_MODE)
    }

    pub fn commit_scope_restrict_to_defined(&self) -> bool {
        self.commit
            .as_ref()
            .and_then(|c| c.scopes.as_ref())
            .and_then(|s| s.restrict_to_defined)
            .unwrap_or(defaults::COMMIT_SCOPE_RESTRICT_TO_DEFINED)
    }

    pub fn commit_scope_allowed(&self) -> Vec<String> {
        self.commit
            .as_ref()
            .and_then(|c| c.scopes.as_ref())
            .and_then(|s| {
                s.definitions
                    .clone()
                    .map(|defs| defs.keys().cloned().collect())
            })
            .unwrap_or_else(defaults::default_commit_allowed_scopes)
    }

    pub fn commit_breaking_require_header(&self) -> bool {
        self.commit
            .as_ref()
            .and_then(|c| c.breaking.as_ref())
            .and_then(|b| b.require_header)
            .unwrap_or(defaults::COMMIT_BREAKING_REQUIRE_HEADER)
    }

    pub fn commit_breaking_require_footer(&self) -> bool {
        self.commit
            .as_ref()
            .and_then(|c| c.breaking.as_ref())
            .and_then(|b| b.require_footer)
            .unwrap_or(defaults::COMMIT_BREAKING_REQUIRE_FOOTER)
    }

    pub fn commit_breaking_footer_key(&self) -> String {
        self.commit
            .as_ref()
            .and_then(|c| c.breaking.as_ref())
            .and_then(|b| b.footer_key.clone())
            .unwrap_or_else(|| defaults::COMMIT_BREAKING_FOOTER_KEY.to_string())
    }

    pub fn commit_breaking_footer_keys_normalized(&self) -> Vec<String> {
        let breaking = self.commit.as_ref().and_then(|c| c.breaking.as_ref());

        if let Some(b) = breaking {
            let mut keys = Vec::new();

            // Add footer_key if present
            if let Some(ref key) = b.footer_key
                && !key.is_empty()
            {
                keys.push(key.clone());
            }

            // Add footer_keys if present
            if let Some(ref keys_list) = b.footer_keys {
                for key in keys_list {
                    if !key.is_empty() && !keys.contains(key) {
                        keys.push(key.clone());
                    }
                }
            }

            if !keys.is_empty() {
                return keys;
            }
        }

        // Default: BREAKING CHANGE and BREAKING-CHANGE
        vec!["BREAKING CHANGE".to_string(), "BREAKING-CHANGE".to_string()]
    }

    pub fn commit_breaking_emoji(&self) -> Option<String> {
        self.commit
            .as_ref()
            .and_then(|c| c.breaking.as_ref())
            .and_then(|b| b.emoji.clone())
    }

    pub fn commit_breaking_emoji_mode(
        &self,
    ) -> crate::engine::models::policy::enforcement::EmojiMode {
        self.commit
            .as_ref()
            .and_then(|c| c.breaking.as_ref())
            .and_then(|b| b.emoji_mode)
            .unwrap_or_default()
    }

    pub fn commit_ticket_required(&self) -> bool {
        self.commit
            .as_ref()
            .and_then(|c| c.ticket.as_ref())
            .and_then(|t| t.required)
            .unwrap_or(defaults::COMMIT_TICKET_REQUIRED)
    }

    pub fn commit_ticket_pattern(&self) -> Option<String> {
        self.commit
            .as_ref()
            .and_then(|c| c.ticket.as_ref())
            .and_then(|t| t.pattern.clone())
    }

    pub fn commit_ticket_source(&self) -> crate::engine::models::policy::enforcement::TicketSource {
        self.commit
            .as_ref()
            .and_then(|c| c.ticket.as_ref())
            .and_then(|t| t.source)
            .unwrap_or(defaults::COMMIT_TICKET_SOURCE)
    }

    pub fn commit_protected_allow(&self) -> bool {
        self.commit
            .as_ref()
            .and_then(|c| c.protected.as_ref())
            .and_then(|p| p.allow)
            .unwrap_or(defaults::COMMIT_PROTECTED_ALLOW)
    }

    pub fn commit_protected_force(&self) -> bool {
        self.commit
            .as_ref()
            .and_then(|c| c.protected.as_ref())
            .and_then(|p| p.force)
            .unwrap_or(defaults::COMMIT_PROTECTED_FORCE)
    }

    pub fn commit_protected_warn(&self) -> bool {
        self.commit
            .as_ref()
            .and_then(|c| c.protected.as_ref())
            .and_then(|p| p.warn)
            .unwrap_or(defaults::COMMIT_PROTECTED_WARN)
    }

    // -------------------------------------------------------------------------
    // branch
    // -------------------------------------------------------------------------

    pub fn branch_remote(&self) -> String {
        self.branch
            .as_ref()
            .and_then(|b| b.remote.clone())
            .unwrap_or_else(|| defaults::BRANCH_REMOTE.to_string())
    }

    pub fn branch_protected_patterns(&self) -> Vec<String> {
        self.branch
            .as_ref()
            .and_then(|b| b.protected.clone())
            .unwrap_or_else(defaults::default_branch_protected_patterns)
    }

    pub fn branch_naming_pattern(&self) -> String {
        self.branch
            .as_ref()
            .and_then(|b| b.naming.as_ref())
            .and_then(|n| n.pattern.clone())
            .unwrap_or_else(|| defaults::BRANCH_NAMING_PATTERN.to_string())
    }

    pub fn branch_naming_enforce(&self) -> bool {
        defaults::BRANCH_NAMING_ENFORCE
    }

    pub fn branch_allowed_targets(&self) -> Vec<String> {
        defaults::default_branch_allowed_targets()
    }

    // -------------------------------------------------------------------------
    // pr
    // -------------------------------------------------------------------------

    pub fn pr_enabled(&self) -> bool {
        self.pr
            .as_ref()
            .and_then(|p| p.enabled)
            .unwrap_or(defaults::PR_ENABLED)
    }

    // -------------------------------------------------------------------------
    // check
    // -------------------------------------------------------------------------

    pub fn check_require_conventional(&self) -> bool {
        self.check
            .as_ref()
            .and_then(|c| c.require_conventional)
            .unwrap_or(defaults::CHECK_REQUIRE_CONVENTIONAL)
    }

    pub fn check_commits_enabled(&self) -> bool {
        self.check
            .as_ref()
            .and_then(|c| c.commits.as_ref())
            .and_then(|cc| cc.enabled)
            .unwrap_or(defaults::CHECK_COMMITS_ENABLED)
    }

    pub fn check_commits_enforce_on(
        &self,
    ) -> crate::engine::models::policy::enforcement::CommitEnforcementScope {
        self.check
            .as_ref()
            .and_then(|c| c.commits.as_ref())
            .and_then(|cc| cc.enforce_on)
            .unwrap_or(defaults::CHECK_COMMITS_ENFORCE_ON)
    }

    // -------------------------------------------------------------------------
    // push
    // -------------------------------------------------------------------------

    pub fn push_allow_protected(&self) -> bool {
        self.push
            .as_ref()
            .and_then(|p| p.allow.as_ref())
            .and_then(|a| a.protected)
            .unwrap_or(defaults::PUSH_ALLOW_PROTECTED)
    }

    pub fn push_allow_force(&self) -> bool {
        self.push
            .as_ref()
            .and_then(|p| p.allow.as_ref())
            .and_then(|a| a.force)
            .unwrap_or(defaults::PUSH_ALLOW_FORCE)
    }

    pub fn push_check_commits(&self) -> bool {
        self.push
            .as_ref()
            .and_then(|p| p.check.as_ref())
            .and_then(|c| c.commits)
            .unwrap_or(defaults::PUSH_CHECK_COMMITS)
    }

    pub fn push_check_branch_policy(&self) -> bool {
        self.push
            .as_ref()
            .and_then(|p| p.check.as_ref())
            .and_then(|c| c.branch_policy)
            .unwrap_or(defaults::PUSH_CHECK_BRANCH_POLICY)
    }

    // -------------------------------------------------------------------------
    // versioning
    // -------------------------------------------------------------------------

    pub fn versioning_tag_prefix(&self) -> String {
        self.versioning
            .as_ref()
            .and_then(|v| v.tag_prefix.clone())
            .unwrap_or_else(|| defaults::VERSIONING_TAG_PREFIX.to_string())
    }

    // -------------------------------------------------------------------------
    // changelog
    // -------------------------------------------------------------------------

    pub fn changelog_output(&self) -> String {
        self.changelog
            .as_ref()
            .and_then(|c| c.output.clone())
            .unwrap_or_else(|| defaults::CHANGELOG_OUTPUT.to_string())
    }

    pub fn changelog_format(&self) -> crate::engine::models::policy::enforcement::ChangelogFormat {
        self.changelog
            .as_ref()
            .and_then(|c| c.format)
            .unwrap_or(defaults::CHANGELOG_FORMAT)
    }

    pub fn changelog_group_by(&self) -> Vec<String> {
        self.changelog
            .as_ref()
            .and_then(|c| c.layout.as_ref())
            .and_then(|l| l.group_by.clone())
            .unwrap_or_else(defaults::default_changelog_group_by)
    }

    pub fn changelog_section_order(&self) -> Vec<String> {
        self.changelog
            .as_ref()
            .and_then(|c| c.layout.as_ref())
            .and_then(|l| l.section_order.clone())
            .unwrap_or_else(defaults::default_changelog_section_order)
    }

    pub fn changelog_scope_order(&self) -> Vec<String> {
        self.changelog
            .as_ref()
            .and_then(|c| c.layout.as_ref())
            .and_then(|l| l.scope_order.clone())
            .unwrap_or_else(defaults::default_changelog_scope_order)
    }

    pub fn changelog_show_scope(&self) -> bool {
        self.changelog
            .as_ref()
            .and_then(|c| c.layout.as_ref())
            .and_then(|l| l.show_scope)
            .unwrap_or(defaults::CHANGELOG_SHOW_SCOPE)
    }

    pub fn changelog_show_empty_sections(&self) -> Option<bool> {
        Some(
            self.changelog
                .as_ref()
                .and_then(|c| c.layout.as_ref())
                .and_then(|l| l.show_empty_sections)
                .unwrap_or(defaults::CHANGELOG_SHOW_EMPTY_SECTIONS),
        )
    }

    pub fn changelog_show_empty_scopes(&self) -> Option<bool> {
        Some(
            self.changelog
                .as_ref()
                .and_then(|c| c.layout.as_ref())
                .and_then(|l| l.show_empty_scopes)
                .unwrap_or(defaults::CHANGELOG_SHOW_EMPTY_SCOPES),
        )
    }

    pub fn changelog_misc_section(&self) -> Option<String> {
        Some(
            self.changelog
                .as_ref()
                .and_then(|c| c.layout.as_ref())
                .and_then(|l| l.misc_section.clone())
                .unwrap_or_else(|| defaults::CHANGELOG_MISC_SECTION.to_string()),
        )
    }

    pub fn changelog_header_use(&self) -> bool {
        self.changelog
            .as_ref()
            .and_then(|c| c.header.as_ref())
            .and_then(|h| h.use_header)
            .unwrap_or(true)
    }

    pub fn changelog_header_title(&self) -> String {
        self.changelog
            .as_ref()
            .and_then(|c| c.header.as_ref())
            .and_then(|h| h.title.clone())
            .unwrap_or_else(|| "Changelog".to_string())
    }

    pub fn changelog_header_description(&self) -> Option<String> {
        self.changelog
            .as_ref()
            .and_then(|c| c.header.as_ref())
            .and_then(|h| h.description.clone())
    }

    pub fn changelog_sections(
        &self,
    ) -> std::collections::BTreeMap<String, crate::engine::config::base::ChangelogSectionConfig>
    {
        self.changelog
            .as_ref()
            .and_then(|c| c.sections.clone())
            .unwrap_or_default()
    }

    pub fn changelog_unreleased_label(&self) -> String {
        self.changelog
            .as_ref()
            .and_then(|c| c.layout.as_ref())
            .and_then(|l| l.unreleased_label.clone())
            .unwrap_or_else(|| "Unreleased".to_string())
    }

    pub fn changelog_date_format(&self) -> Option<String> {
        self.changelog
            .as_ref()
            .and_then(|c| c.layout.as_ref())
            .and_then(|l| l.date_format.clone())
    }

    // -------------------------------------------------------------------------
    // release
    // -------------------------------------------------------------------------

    pub fn release_enabled(&self) -> bool {
        self.release
            .as_ref()
            .and_then(|r| r.enabled)
            .unwrap_or(defaults::RELEASE_ENABLED)
    }

    pub fn release_source_branch(&self) -> String {
        self.release
            .as_ref()
            .and_then(|r| r.source_branch.clone())
            .unwrap_or_else(|| defaults::RELEASE_SOURCE_BRANCH.to_string())
    }

    pub fn release_target_branch(&self) -> String {
        self.release
            .as_ref()
            .and_then(|r| r.target_branch.clone())
            .unwrap_or_else(|| defaults::RELEASE_TARGET_BRANCH.to_string())
    }

    pub fn release_branch_format(&self) -> String {
        self.release
            .as_ref()
            .and_then(|r| r.branch_format.clone())
            .unwrap_or_else(|| defaults::RELEASE_BRANCH_FORMAT.to_string())
    }

    pub fn release_hotfix_pattern(&self) -> String {
        self.release
            .as_ref()
            .and_then(|r| r.hotfix_pattern.clone())
            .unwrap_or_else(|| defaults::RELEASE_HOTFIX_PATTERN.to_string())
    }

    pub fn release_require_clean_worktree(&self) -> bool {
        self.release
            .as_ref()
            .and_then(|r| r.validation.as_ref())
            .and_then(|v| v.require_clean_worktree)
            .unwrap_or(defaults::RELEASE_REQUIRE_CLEAN_WORKTREE)
    }

    pub fn release_fail_if_tag_exists(&self) -> bool {
        self.release
            .as_ref()
            .and_then(|r| r.validation.as_ref())
            .and_then(|v| v.fail_if_tag_exists)
            .unwrap_or(defaults::RELEASE_FAIL_IF_TAG_EXISTS)
    }

    pub fn release_fail_if_release_branch_exists(&self) -> bool {
        self.release
            .as_ref()
            .and_then(|r| r.validation.as_ref())
            .and_then(|v| v.fail_if_release_branch_exists)
            .unwrap_or(defaults::RELEASE_FAIL_IF_RELEASE_BRANCH_EXISTS)
    }

    pub fn release_finish_tag(&self) -> bool {
        self.release
            .as_ref()
            .and_then(|r| r.finish.as_ref())
            .and_then(|f| f.tag)
            .unwrap_or(defaults::RELEASE_FINISH_TAG)
    }

    pub fn release_finish_push(&self) -> bool {
        self.release
            .as_ref()
            .and_then(|r| r.finish.as_ref())
            .and_then(|f| f.push)
            .unwrap_or(defaults::RELEASE_FINISH_PUSH)
    }

    pub fn release_finish_backmerge_branch(&self) -> String {
        self.release
            .as_ref()
            .and_then(|r| r.finish.as_ref())
            .and_then(|f| f.backmerge_branch.clone())
            .unwrap_or_else(|| defaults::RELEASE_FINISH_BACKMERGE_BRANCH.to_string())
    }

    // -------------------------------------------------------------------------
    // hooks
    // -------------------------------------------------------------------------

    pub fn hooks_pre_commit(&self) -> bool {
        self.hooks
            .as_ref()
            .and_then(|h| h.pre_commit)
            .unwrap_or(defaults::HOOKS_PRE_COMMIT)
    }

    pub fn hooks_commit_msg(&self) -> bool {
        self.hooks
            .as_ref()
            .and_then(|h| h.commit_msg)
            .unwrap_or(defaults::HOOKS_COMMIT_MSG)
    }

    pub fn hooks_pre_push(&self) -> bool {
        self.hooks
            .as_ref()
            .and_then(|h| h.pre_push)
            .unwrap_or(defaults::HOOKS_PRE_PUSH)
    }

    // -------------------------------------------------------------------------
    // ai
    // -------------------------------------------------------------------------

    pub fn ai_enabled(&self) -> bool {
        self.ai
            .as_ref()
            .and_then(|a| a.enabled)
            .unwrap_or(defaults::AI_ENABLED)
    }

    pub fn ai_provider(&self) -> AiProvider {
        self.ai
            .as_ref()
            .and_then(|a| a.provider.clone())
            .as_deref()
            .map(AiProvider::from_str)
            .unwrap_or(defaults::AI_PROVIDER)
    }

    pub fn ai_commit_enabled(&self) -> bool {
        self.ai
            .as_ref()
            .and_then(|a| a.commands.as_ref())
            .and_then(|c| c.commit)
            .unwrap_or(defaults::AI_COMMAND_COMMIT)
    }

    pub fn ai_changelog_enabled(&self) -> bool {
        self.ai
            .as_ref()
            .and_then(|a| a.commands.as_ref())
            .and_then(|c| c.changelog)
            .unwrap_or(defaults::AI_COMMAND_CHANGELOG)
    }

    pub fn ai_release_prepare_enabled(&self) -> bool {
        self.ai
            .as_ref()
            .and_then(|a| a.commands.as_ref())
            .and_then(|c| c.release_prepare)
            .unwrap_or(defaults::AI_COMMAND_RELEASE_PREPARE)
    }

    // -------------------------------------------------------------------------
    // registry
    // -------------------------------------------------------------------------

    pub fn registry_use(&self) -> Option<String> {
        self.registry
            .as_ref()
            .and_then(|r| r.use_registry.clone())
            .filter(|s| !s.trim().is_empty())
    }

    pub fn registries_map(&self) -> HashMap<String, NamedRegistryConfig> {
        self.registries
            .clone()
            .unwrap_or_else(defaults::default_registries)
    }
}

/// Deep-merge two optional CommitConfigs.
///
/// Each sub-field is merged independently so that a higher-priority layer can
/// set `subject_max_length` without blowing away the `types` that came from a
/// lower-priority layer (e.g. the registry). This is the key behaviour that
/// allows the registry to supply commit types while the repo only overrides
/// formatting settings.
fn merge_commit_config(
    high: Option<CommitConfig>,
    low: Option<CommitConfig>,
) -> Option<CommitConfig> {
    match (high, low) {
        (None, low) => low,
        (high, None) => high,
        (Some(h), Some(l)) => Some(CommitConfig {
            subject_max_length: h.subject_max_length.or(l.subject_max_length),
            use_emojis: h.use_emojis.or(l.use_emojis),
            // types: high wins if explicitly set; otherwise fall through to lower layer
            types: h.types.or(l.types),
            scopes: h.scopes.or(l.scopes),
            breaking: h.breaking.or(l.breaking),
            protected: h.protected.or(l.protected),
            ticket: h.ticket.or(l.ticket),
        }),
    }
}
