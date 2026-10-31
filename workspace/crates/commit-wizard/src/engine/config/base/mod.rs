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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global: Option<GlobalConfig>,
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
            global: None,
        }
    }

    /// Merge two configs. `self` has higher priority — its sections win over `lower` for each
    /// top-level section, falling back to `lower` only when `self`'s section is `None`.
    /// Merge two layers, with `self` taking precedence over `lower`.
    ///
    /// The merge is recursive: tables are combined key by key and the higher
    /// layer wins only for the leaves it actually sets. A local config can
    /// therefore change one commit type, one scope policy value or one
    /// versioning field and keep everything else from the layer below (for
    /// example a registry). Arrays are replaced, not concatenated.
    pub fn merge(self, lower: BaseConfig) -> BaseConfig {
        match deep_merge_configs(&self, &lower) {
            Some(merged) => merged,
            None => self.shallow_merge(lower),
        }
    }

    /// Per-table merge used if the recursive merge cannot round-trip.
    fn shallow_merge(self, lower: BaseConfig) -> BaseConfig {
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
            registry: merge_registry_config(self.registry, lower.registry),
            registries: self.registries.or(lower.registries),
            global: self.global.or(lower.global),
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

    /// Whether the changelog links entries and releases to the repository (default: yes).
    pub fn changelog_links_enabled(&self) -> bool {
        self.changelog
            .as_ref()
            .and_then(|c| c.links.as_ref())
            .and_then(|l| l.enabled)
            .unwrap_or(true)
    }

    /// Repository web address to link to, instead of the one from the `origin` remote.
    pub fn changelog_links_url(&self) -> Option<String> {
        self.changelog
            .as_ref()
            .and_then(|c| c.links.as_ref())
            .and_then(|l| l.url.clone())
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

    /// Whether the global config sits under this project's config as its
    /// lowest layer (`[global] inherit = true`). Default: no.
    pub fn global_inherit(&self) -> bool {
        self.global
            .as_ref()
            .and_then(|g| g.inherit)
            .unwrap_or(false)
    }

    /// Whether a failure to load the active registry must stop the command.
    pub fn registry_required(&self) -> bool {
        self.registry
            .as_ref()
            .and_then(|r| r.required)
            .unwrap_or(false)
    }

    /// Whether registries are kept in (and served from) the local cache.
    pub fn registry_cache(&self) -> bool {
        self.registry.as_ref().and_then(|r| r.cache).unwrap_or(true)
    }

    pub fn registries_map(&self) -> HashMap<String, NamedRegistryConfig> {
        self.registries
            .clone()
            .unwrap_or_else(defaults::default_registries)
    }
}

/// Recursively merge `high` over `low` through their TOML representation.
/// Returns `None` if either side cannot be converted, so callers can fall back.
fn deep_merge_configs(high: &BaseConfig, low: &BaseConfig) -> Option<BaseConfig> {
    let mut merged = toml::Value::try_from(low).ok()?;
    deep_merge_values(&mut merged, toml::Value::try_from(high).ok()?);
    let mut config: BaseConfig = merged.try_into().ok()?;
    // `use` has its own rule: an empty value must not mask a lower layer.
    config.registry = merge_registry_config(high.registry.clone(), low.registry.clone());
    Some(config)
}

fn deep_merge_values(low: &mut toml::Value, high: toml::Value) {
    match (low, high) {
        (toml::Value::Table(low), toml::Value::Table(high)) => {
            for (key, value) in high {
                match low.get_mut(&key) {
                    Some(existing) => deep_merge_values(existing, value),
                    None => {
                        low.insert(key, value);
                    }
                }
            }
        }
        (low, high) => *low = high,
    }
}

/// Merge two optional `[registry]` tables field by field.
///
/// A higher-priority layer that sets only some fields (say `use`) must not
/// discard the others (`required`, `cache`, `section`) from a lower layer, so
/// a repo config can pick a registry while a global config makes it required.
fn merge_registry_config(
    high: Option<RegistryConfig>,
    low: Option<RegistryConfig>,
) -> Option<RegistryConfig> {
    match (high, low) {
        (None, low) => low,
        (high, None) => high,
        (Some(h), Some(l)) => Some(RegistryConfig {
            // An empty `use` means "not chosen", so it must not mask a lower layer.
            use_registry: h
                .use_registry
                .filter(|name| !name.trim().is_empty())
                .or(l.use_registry),
            section: h.section.or(l.section),
            required: h.required.or(l.required),
            cache: h.cache.or(l.cache),
        }),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn with_registry(registry: RegistryConfig) -> BaseConfig {
        BaseConfig {
            registry: Some(registry),
            ..BaseConfig::empty()
        }
    }

    fn registry(
        use_registry: Option<&str>,
        required: Option<bool>,
        cache: Option<bool>,
    ) -> RegistryConfig {
        RegistryConfig {
            use_registry: use_registry.map(str::to_string),
            section: None,
            required,
            cache,
        }
    }

    #[test]
    fn registry_fields_merge_instead_of_replacing_the_table() {
        let global = with_registry(registry(None, Some(true), Some(false)));
        let repo = with_registry(registry(Some("team"), None, None));

        let merged = repo.merge(global);

        assert_eq!(merged.registry_use().as_deref(), Some("team"));
        assert!(merged.registry_required(), "global required must survive");
        assert!(
            !merged.registry_cache(),
            "global cache = false must survive"
        );
    }

    #[test]
    fn higher_layer_wins_per_field() {
        let global = with_registry(registry(Some("global"), Some(true), Some(true)));
        let repo = with_registry(registry(Some("repo"), Some(false), None));

        let merged = repo.merge(global);

        assert_eq!(merged.registry_use().as_deref(), Some("repo"));
        assert!(!merged.registry_required());
        assert!(merged.registry_cache());
    }

    #[test]
    fn empty_use_does_not_mask_a_lower_layer() {
        let global = with_registry(registry(Some("global"), None, None));
        let repo = with_registry(registry(Some(""), None, None));

        assert_eq!(repo.merge(global).registry_use().as_deref(), Some("global"));
    }

    #[test]
    fn registry_defaults_when_no_layer_sets_them() {
        let merged = BaseConfig::empty().merge(BaseConfig::empty());
        assert!(!merged.registry_required());
        assert!(merged.registry_cache());
    }

    fn parse(toml: &str) -> BaseConfig {
        toml::from_str(toml).expect("valid config")
    }

    #[test]
    fn commit_types_merge_per_type_and_per_field() {
        let registry = parse(
            r#"
[commit.types.fix]
description = "A bug fix"
bump = "Patch"
section = "Bug Fixes"

[commit.types.wip]
description = "Work in progress"
bump = "Patch"
"#,
        );
        let local = parse(
            r#"
[commit.types.fix]
bump = "Minor"

[commit.types.wip]
enabled = false

[commit.types.docs]
description = "Documentation"
"#,
        );

        let types = local.merge(registry).commit_types();

        let fix = &types["fix"];
        assert_eq!(
            fix.description.as_deref(),
            Some("A bug fix"),
            "kept from below"
        );
        assert_eq!(fix.section.as_deref(), Some("Bug Fixes"), "kept from below");
        assert!(
            matches!(
                fix.bump,
                Some(crate::engine::models::policy::enforcement::BumpLevel::Minor)
            ),
            "overridden locally"
        );
        assert_eq!(types["wip"].enabled, Some(false));
        assert_eq!(types["docs"].description.as_deref(), Some("Documentation"));
        assert_eq!(types.len(), 3);
    }

    #[test]
    fn other_tables_merge_per_field() {
        let registry = parse("[push.allow]\nprotected = true\nforce = false\n");
        let local = parse("[push.allow]\nforce = true\n");

        let allow = local.merge(registry).push.unwrap().allow.unwrap();

        assert_eq!(allow.protected, Some(true), "kept from below");
        assert_eq!(allow.force, Some(true), "overridden locally");
    }

    #[test]
    fn arrays_are_replaced_not_concatenated() {
        let registry = parse("[branch]\nprotected = [\"main\", \"release/*\"]\n");
        let local = parse("[branch]\nprotected = [\"trunk\"]\n");

        let merged = local.merge(registry);

        assert_eq!(
            merged.branch.unwrap().protected,
            Some(vec!["trunk".to_string()])
        );
    }

    #[test]
    fn merging_is_associative_across_three_layers() {
        let low = parse("[commit]\nsubject_max_length = 100\nuse_emojis = false\n");
        let mid = parse("[commit]\nuse_emojis = true\n");
        let high = parse("[commit]\nsubject_max_length = 50\n");

        let merged = high.merge(mid.merge(low));

        assert_eq!(merged.commit_subject_max_length(), 50);
        assert!(merged.commit_use_emojis());
    }
}
