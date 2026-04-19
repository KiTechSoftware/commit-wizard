use std::collections::{BTreeMap, HashMap};

use crate::engine::constants::{
    AI_COMMAND_CHANGELOG, AI_COMMAND_COMMIT, AI_COMMAND_RELEASE_PREPARE, AI_ENABLED,
    BRANCH_NAMING_PATTERN, BRANCH_REMOTE, CHANGELOG_FORMAT, CHANGELOG_MISC_SECTION,
    CHANGELOG_OUTPUT, CHANGELOG_SHOW_EMPTY_SCOPES, CHANGELOG_SHOW_EMPTY_SECTIONS,
    CHANGELOG_SHOW_SCOPE, CHECK_COMMITS_ENABLED, CHECK_COMMITS_ENFORCE_ON,
    CHECK_REQUIRE_CONVENTIONAL, COMMIT_BREAKING_FOOTER_KEY, COMMIT_BREAKING_REQUIRE_FOOTER,
    COMMIT_BREAKING_REQUIRE_HEADER, COMMIT_PROTECTED_ALLOW, COMMIT_PROTECTED_FORCE,
    COMMIT_PROTECTED_WARN, COMMIT_SCOPE_MODE, COMMIT_SCOPE_RESTRICT_TO_DEFINED,
    COMMIT_SUBJECT_MAX_LENGTH, COMMIT_TICKET_REQUIRED, COMMIT_TICKET_SOURCE, COMMIT_USE_EMOJIS,
    HOOKS_COMMIT_MSG, HOOKS_PRE_COMMIT, HOOKS_PRE_PUSH, PR_ENABLED, PUSH_ALLOW_FORCE,
    PUSH_ALLOW_PROTECTED, PUSH_CHECK_BRANCH_POLICY, PUSH_CHECK_COMMITS, RELEASE_BRANCH_FORMAT,
    RELEASE_ENABLED, RELEASE_FAIL_IF_RELEASE_BRANCH_EXISTS, RELEASE_FAIL_IF_TAG_EXISTS,
    RELEASE_FINISH_BACKMERGE_BRANCH, RELEASE_FINISH_PUSH, RELEASE_FINISH_TAG,
    RELEASE_HOTFIX_PATTERN, RELEASE_REQUIRE_CLEAN_WORKTREE, RELEASE_SOURCE_BRANCH,
    RELEASE_TARGET_BRANCH, VERSIONING_TAG_PREFIX,
};
use crate::engine::{
    config::base::{
        AiCommandsConfig, AiConfig, BranchConfig, BranchNamingConfig, BreakingConfig,
        ChangelogConfig, ChangelogLayoutConfig, CheckCommitsConfig, CheckConfig, CommitConfig,
        CommitProtectedConfig, CommitScopesConfig, CommitTypeConfig, HooksConfig,
        NamedRegistryConfig, PrConfig, PushAllowConfig, PushCheckConfig, PushConfig,
        RegistryConfig, ReleaseConfig, ReleaseFinishConfig, ReleaseValidationConfig, TicketConfig,
        VersioningConfig,
    },
    constants::AI_PROVIDER,
    models::policy::enforcement::BumpLevel,
};

//
// ──────────────────────────────────────────────────────────────────────────────
// Collections / dynamic defaults
// ──────────────────────────────────────────────────────────────────────────────
//

pub fn default_commit_allowed_scopes() -> Vec<String> {
    Vec::new()
}

pub fn default_commit_types() -> BTreeMap<String, CommitTypeConfig> {
    let mut map = BTreeMap::new();

    map.insert(
        "feat".to_string(),
        CommitTypeConfig {
            emoji: Some("✨".to_string()),
            description: Some("A new feature".to_string()),
            bump: Some(BumpLevel::Minor),
            section: Some("Features".to_string()),
        },
    );

    map.insert(
        "fix".to_string(),
        CommitTypeConfig {
            emoji: Some("🐛".to_string()),
            description: Some("A bug fix".to_string()),
            bump: Some(BumpLevel::Patch),
            section: Some("Bug Fixes".to_string()),
        },
    );

    map.insert(
        "docs".to_string(),
        CommitTypeConfig {
            emoji: Some("📚".to_string()),
            description: Some("Documentation changes".to_string()),
            bump: Some(BumpLevel::Patch),
            section: Some("Documentation".to_string()),
        },
    );

    map.insert(
        "style".to_string(),
        CommitTypeConfig {
            emoji: Some("🎨".to_string()),
            description: Some("Code style changes".to_string()),
            bump: Some(BumpLevel::Patch),
            section: Some("Style".to_string()),
        },
    );

    map.insert(
        "refactor".to_string(),
        CommitTypeConfig {
            emoji: Some("♻️".to_string()),
            description: Some("Code refactoring".to_string()),
            bump: Some(BumpLevel::Patch),
            section: Some("Refactoring".to_string()),
        },
    );

    map.insert(
        "perf".to_string(),
        CommitTypeConfig {
            emoji: Some("⚡".to_string()),
            description: Some("Performance improvements".to_string()),
            bump: Some(BumpLevel::Patch),
            section: Some("Performance".to_string()),
        },
    );

    map.insert(
        "test".to_string(),
        CommitTypeConfig {
            emoji: Some("✅".to_string()),
            description: Some("Test changes".to_string()),
            bump: Some(BumpLevel::Patch),
            section: Some("Tests".to_string()),
        },
    );

    map.insert(
        "chore".to_string(),
        CommitTypeConfig {
            emoji: Some("🔧".to_string()),
            description: Some("Maintenance chores".to_string()),
            bump: Some(BumpLevel::Patch),
            section: Some("Chores".to_string()),
        },
    );

    map
}

pub fn default_branch_protected_patterns() -> Vec<String> {
    vec![
        "main".to_string(),
        "master".to_string(),
        "release/*".to_string(),
    ]
}

pub fn default_branch_allowed_targets() -> Vec<String> {
    Vec::new()
}

pub fn default_changelog_group_by() -> Vec<String> {
    vec!["type".to_string()]
}

pub fn default_changelog_section_order() -> Vec<String> {
    vec![
        "feat".to_string(),
        "fix".to_string(),
        "docs".to_string(),
        "style".to_string(),
        "refactor".to_string(),
        "perf".to_string(),
        "test".to_string(),
        "chore".to_string(),
    ]
}

pub fn default_changelog_scope_order() -> Vec<String> {
    Vec::new()
}

pub fn default_registries() -> HashMap<String, NamedRegistryConfig> {
    HashMap::new()
}

//
// ──────────────────────────────────────────────────────────────────────────────
// Structured config defaults
// These are useful for BaseConfig accessors and template generation
// ──────────────────────────────────────────────────────────────────────────────
//

pub fn default_commit_scopes_config() -> CommitScopesConfig {
    CommitScopesConfig {
        mode: Some(COMMIT_SCOPE_MODE),
        restrict_to_defined: Some(COMMIT_SCOPE_RESTRICT_TO_DEFINED),
        definitions: None,
    }
}

pub fn default_commit_breaking_config() -> BreakingConfig {
    BreakingConfig {
        require_header: Some(COMMIT_BREAKING_REQUIRE_HEADER),
        require_footer: Some(COMMIT_BREAKING_REQUIRE_FOOTER),
        footer_key: Some(COMMIT_BREAKING_FOOTER_KEY.to_string()),
        footer_keys: None,
        emoji: None,
        emoji_mode: None,
    }
}

pub fn default_commit_ticket_config() -> TicketConfig {
    TicketConfig {
        required: Some(COMMIT_TICKET_REQUIRED),
        pattern: None,
        source: Some(COMMIT_TICKET_SOURCE),
        header_format: None,
    }
}

pub fn default_commit_protected_config() -> CommitProtectedConfig {
    CommitProtectedConfig {
        allow: Some(COMMIT_PROTECTED_ALLOW),
        force: Some(COMMIT_PROTECTED_FORCE),
        warn: Some(COMMIT_PROTECTED_WARN),
    }
}

pub fn default_commit_config() -> CommitConfig {
    CommitConfig {
        subject_max_length: Some(COMMIT_SUBJECT_MAX_LENGTH),
        use_emojis: Some(COMMIT_USE_EMOJIS),
        types: Some(default_commit_types()),
        scopes: Some(default_commit_scopes_config()),
        breaking: Some(default_commit_breaking_config()),
        protected: Some(default_commit_protected_config()),
        ticket: Some(default_commit_ticket_config()),
    }
}

pub fn default_branch_naming_config() -> BranchNamingConfig {
    BranchNamingConfig {
        pattern: Some(BRANCH_NAMING_PATTERN.to_string()),
    }
}

pub fn default_branch_config() -> BranchConfig {
    BranchConfig {
        remote: Some(BRANCH_REMOTE.to_string()),
        protected: Some(default_branch_protected_patterns()),
        naming: Some(default_branch_naming_config()),
    }
}

pub fn default_pr_config() -> PrConfig {
    PrConfig {
        enabled: Some(PR_ENABLED),
        title: None,
        branch: None,
    }
}

pub fn default_check_commits_config() -> CheckCommitsConfig {
    CheckCommitsConfig {
        enabled: Some(CHECK_COMMITS_ENABLED),
        enforce_on: Some(CHECK_COMMITS_ENFORCE_ON),
    }
}

pub fn default_check_config() -> CheckConfig {
    CheckConfig {
        require_conventional: Some(CHECK_REQUIRE_CONVENTIONAL),
        commits: Some(default_check_commits_config()),
    }
}

pub fn default_push_allow_config() -> PushAllowConfig {
    PushAllowConfig {
        protected: Some(PUSH_ALLOW_PROTECTED),
        force: Some(PUSH_ALLOW_FORCE),
    }
}

pub fn default_push_check_config() -> PushCheckConfig {
    PushCheckConfig {
        commits: Some(PUSH_CHECK_COMMITS),
        branch_policy: Some(PUSH_CHECK_BRANCH_POLICY),
    }
}

pub fn default_push_config() -> PushConfig {
    PushConfig {
        allow: Some(default_push_allow_config()),
        check: Some(default_push_check_config()),
    }
}

pub fn default_versioning_config() -> VersioningConfig {
    VersioningConfig {
        tag_prefix: Some(VERSIONING_TAG_PREFIX.to_string()),
    }
}

pub fn default_changelog_layout_config() -> ChangelogLayoutConfig {
    ChangelogLayoutConfig {
        group_by: Some(default_changelog_group_by()),
        section_order: Some(default_changelog_section_order()),
        scope_order: Some(default_changelog_scope_order()),
        show_scope: Some(CHANGELOG_SHOW_SCOPE),
        show_empty_sections: Some(CHANGELOG_SHOW_EMPTY_SECTIONS),
        show_empty_scopes: Some(CHANGELOG_SHOW_EMPTY_SCOPES),
        misc_section: Some(CHANGELOG_MISC_SECTION.to_string()),
        unreleased_label: None,
        date_format: None,
    }
}

pub fn default_changelog_config() -> ChangelogConfig {
    ChangelogConfig {
        output: Some(CHANGELOG_OUTPUT.to_string()),
        format: Some(CHANGELOG_FORMAT),
        header: None,
        layout: Some(default_changelog_layout_config()),
        sections: None,
    }
}

pub fn default_release_validation_config() -> ReleaseValidationConfig {
    ReleaseValidationConfig {
        require_clean_worktree: Some(RELEASE_REQUIRE_CLEAN_WORKTREE),
        fail_if_tag_exists: Some(RELEASE_FAIL_IF_TAG_EXISTS),
        fail_if_release_branch_exists: Some(RELEASE_FAIL_IF_RELEASE_BRANCH_EXISTS),
    }
}

pub fn default_release_finish_config() -> ReleaseFinishConfig {
    ReleaseFinishConfig {
        tag: Some(RELEASE_FINISH_TAG),
        push: Some(RELEASE_FINISH_PUSH),
        backmerge_branch: Some(RELEASE_FINISH_BACKMERGE_BRANCH.to_string()),
    }
}

pub fn default_release_config() -> ReleaseConfig {
    ReleaseConfig {
        enabled: Some(RELEASE_ENABLED),
        source_branch: Some(RELEASE_SOURCE_BRANCH.to_string()),
        target_branch: Some(RELEASE_TARGET_BRANCH.to_string()),
        branch_format: Some(RELEASE_BRANCH_FORMAT.to_string()),
        hotfix_pattern: Some(RELEASE_HOTFIX_PATTERN.to_string()),
        validation: Some(default_release_validation_config()),
        finish: Some(default_release_finish_config()),
    }
}

pub fn default_hooks_config() -> HooksConfig {
    HooksConfig {
        pre_commit: Some(HOOKS_PRE_COMMIT),
        commit_msg: Some(HOOKS_COMMIT_MSG),
        pre_push: Some(HOOKS_PRE_PUSH),
    }
}

pub fn default_ai_commands_config() -> AiCommandsConfig {
    AiCommandsConfig {
        commit: Some(AI_COMMAND_COMMIT),
        changelog: Some(AI_COMMAND_CHANGELOG),
        release_prepare: Some(AI_COMMAND_RELEASE_PREPARE),
    }
}

pub fn default_ai_config() -> AiConfig {
    AiConfig {
        enabled: Some(AI_ENABLED),
        provider: Some(AI_PROVIDER.to_string()),
        commands: Some(default_ai_commands_config()),
    }
}

pub fn default_registry_config() -> RegistryConfig {
    RegistryConfig {
        use_registry: Some(String::new()),
        section: None,
    }
}

//
// ──────────────────────────────────────────────────────────────────────────────
// BaseConfig presets
// ──────────────────────────────────────────────────────────────────────────────
//

pub fn minimal_base_config() -> crate::engine::config::BaseConfig {
    crate::engine::config::BaseConfig::empty()
}

pub fn standard_base_config() -> crate::engine::config::BaseConfig {
    crate::engine::config::BaseConfig {
        commit: Some(default_commit_config()),
        branch: Some(default_branch_config()),
        pr: Some(default_pr_config()),
        check: Some(default_check_config()),
        push: Some(default_push_config()),
        versioning: Some(default_versioning_config()),
        changelog: Some(default_changelog_config()),
        release: Some(default_release_config()),
        hooks: Some(default_hooks_config()),
        ai: Some(default_ai_config()),
        registry: None,
        registries: None,
    }
}

pub fn full_base_config() -> crate::engine::config::BaseConfig {
    crate::engine::config::BaseConfig {
        commit: Some(default_commit_config()),
        branch: Some(default_branch_config()),
        pr: Some(default_pr_config()),
        check: Some(default_check_config()),
        push: Some(default_push_config()),
        versioning: Some(default_versioning_config()),
        changelog: Some(default_changelog_config()),
        release: Some(default_release_config()),
        hooks: Some(default_hooks_config()),
        ai: Some(default_ai_config()),
        registry: Some(default_registry_config()),
        registries: Some(default_registries()),
    }
}
