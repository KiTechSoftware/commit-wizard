//
// ──────────────────────────────────────────────────────────────────────────────
// General
// ──────────────────────────────────────────────────────────────────────────────
//

use crate::engine::models::policy::enforcement::{
    AiProvider, BumpLevel, ChangelogFormat, CommitEnforcementScope, ScopeMode, TicketSource,
};

pub const CONFIG_VERSION: u32 = 1;

pub const DEFAULT_COMMIT_BUMP: BumpLevel = BumpLevel::Patch;
//
// ──────────────────────────────────────────────────────────────────────────────
// Commit
// ──────────────────────────────────────────────────────────────────────────────
//

pub const COMMIT_SUBJECT_MAX_LENGTH: u32 = 72;
pub const COMMIT_USE_EMOJIS: bool = false;

pub const COMMIT_SCOPE_MODE: ScopeMode = ScopeMode::Optional;
pub const COMMIT_SCOPE_RESTRICT_TO_DEFINED: bool = false;

pub const COMMIT_BREAKING_REQUIRE_HEADER: bool = false;
pub const COMMIT_BREAKING_REQUIRE_FOOTER: bool = false;
pub const COMMIT_BREAKING_FOOTER_KEY: &str = "BREAKING CHANGE";

pub const COMMIT_TICKET_ENABLED: bool = false;
pub const COMMIT_TICKET_REQUIRED: bool = false;
pub const COMMIT_TICKET_SOURCE: TicketSource = TicketSource::Disabled;

pub const COMMIT_PROTECTED_ALLOW: bool = true;
pub const COMMIT_PROTECTED_FORCE: bool = false;
pub const COMMIT_PROTECTED_WARN: bool = true;

//
// ──────────────────────────────────────────────────────────────────────────────
// Branch
// ──────────────────────────────────────────────────────────────────────────────
//

pub const BRANCH_REMOTE: &str = "origin";
pub const BRANCH_NAMING_PATTERN: &str = "feature/{issue}";
pub const BRANCH_NAMING_ENFORCE: bool = false;

//
// ──────────────────────────────────────────────────────────────────────────────
// PR
// ──────────────────────────────────────────────────────────────────────────────
//

pub const PR_ENABLED: bool = true;

//
// ──────────────────────────────────────────────────────────────────────────────
// Check
// ──────────────────────────────────────────────────────────────────────────────
//

pub const CHECK_REQUIRE_CONVENTIONAL: bool = true;
pub const CHECK_COMMITS_ENABLED: bool = true;
pub const CHECK_COMMITS_ENFORCE_ON: CommitEnforcementScope =
    CommitEnforcementScope::ProtectedBranches;

//
// ──────────────────────────────────────────────────────────────────────────────
// Push
// ──────────────────────────────────────────────────────────────────────────────
//

pub const PUSH_ALLOW_PROTECTED: bool = false;
pub const PUSH_ALLOW_FORCE: bool = false;

pub const PUSH_CHECK_COMMITS: bool = true;
pub const PUSH_CHECK_BRANCH_POLICY: bool = true;

//
// ──────────────────────────────────────────────────────────────────────────────
// Versioning
// ──────────────────────────────────────────────────────────────────────────────
//

pub const VERSIONING_TAG_PREFIX: &str = "v";

//
// ──────────────────────────────────────────────────────────────────────────────
// Changelog
// ──────────────────────────────────────────────────────────────────────────────
//

pub const CHANGELOG_OUTPUT: &str = "CHANGELOG.md";
pub const CHANGELOG_FORMAT: ChangelogFormat = ChangelogFormat::Markdown;
pub const CHANGELOG_SHOW_SCOPE: bool = true;
pub const CHANGELOG_SHOW_EMPTY_SECTIONS: bool = false;
pub const CHANGELOG_SHOW_EMPTY_SCOPES: bool = false;
pub const CHANGELOG_MISC_SECTION: &str = "Miscellaneous";

//
// ──────────────────────────────────────────────────────────────────────────────
// Release
// ──────────────────────────────────────────────────────────────────────────────
//

pub const RELEASE_ENABLED: bool = false;
pub const RELEASE_SOURCE_BRANCH: &str = "main";
pub const RELEASE_TARGET_BRANCH: &str = "main";
pub const RELEASE_BRANCH_FORMAT: &str = "release/{version}";
pub const RELEASE_HOTFIX_PATTERN: &str = "hotfix/*";

pub const RELEASE_REQUIRE_CLEAN_WORKTREE: bool = true;
pub const RELEASE_FAIL_IF_TAG_EXISTS: bool = true;
pub const RELEASE_FAIL_IF_RELEASE_BRANCH_EXISTS: bool = true;

pub const RELEASE_FINISH_TAG: bool = true;
pub const RELEASE_FINISH_PUSH: bool = true;
pub const RELEASE_FINISH_BACKMERGE_BRANCH: &str = "main";

//
// ──────────────────────────────────────────────────────────────────────────────
// Hooks
// ──────────────────────────────────────────────────────────────────────────────
//

pub const HOOKS_PRE_COMMIT: bool = false;
pub const HOOKS_COMMIT_MSG: bool = false;
pub const HOOKS_PRE_PUSH: bool = false;

//
// ──────────────────────────────────────────────────────────────────────────────
// AI
// ──────────────────────────────────────────────────────────────────────────────
//

pub const AI_ENABLED: bool = false;
pub const AI_PROVIDER: AiProvider = AiProvider::Copilot;

pub const AI_COMMAND_COMMIT: bool = false;
pub const AI_COMMAND_CHANGELOG: bool = false;
pub const AI_COMMAND_RELEASE_PREPARE: bool = false;

//
// ──────────────────────────────────────────────────────────────────────────────
// Registry
// ──────────────────────────────────────────────────────────────────────────────
//

pub const REGISTRY_DEFAULT_REF: &str = "main";
