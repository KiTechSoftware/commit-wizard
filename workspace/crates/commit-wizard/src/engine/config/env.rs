/// Build a `BaseConfig` from environment variables (SRS §4).
///
/// This layer sits between CLI args and repo config in the resolution stack:
///   CLI args > ENV > repo config > registry config > global config > defaults
///
/// Only fields with a corresponding set ENV variable are populated; everything
/// else remains `None` so that lower-priority layers can still apply.
use std::env;

use crate::engine::{
    config::base::{
        BaseConfig, BranchConfig, BranchNamingConfig, CheckCommitsConfig, CheckConfig,
        CommitConfig, CommitScopesConfig, PrBranchConfig, PrConfig, PrTitleConfig, PushAllowConfig,
        PushCheckConfig, PushConfig, TicketConfig, VersioningConfig,
    },
    constants::env::*,
    models::policy::enforcement::{CommitEnforcementScope, ScopeMode, TicketSource},
};

// ---------------------------------------------------------------------------
// Primitive helpers
// ---------------------------------------------------------------------------

// These helpers use eprintln! for parse warnings rather than the app logger
// because the env module is called before the runtime and logger are available.
// The [warn] prefix matches the app's log format for grep-ability.

/// Returns `Some(value)` only if at least one of the provided fields is `Some`.
/// Removes the repetitive "if all fields are None, return None" pattern across builders.
macro_rules! some_if_any {
    ([$($field:expr),+ $(,)?] => $value:expr) => {
        if $($field.is_some())||+ {
            Some($value)
        } else {
            None
        }
    };
}

fn get_string(key: &str) -> Option<String> {
    env::var(key).ok().filter(|s| !s.is_empty())
}

fn get_bool(key: &str) -> Option<bool> {
    get_string(key).and_then(|val| parse_bool_val(&val, key))
}

fn parse_bool_val(val: &str, key: &str) -> Option<bool> {
    match val.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => {
            eprintln!(
                "[warn] CW: invalid boolean for {key}: {val:?} — expected true/false/1/0/yes/no/on/off"
            );
            None
        }
    }
}

fn get_u32(key: &str) -> Option<u32> {
    get_string(key).and_then(|val| {
        val.parse::<u32>()
            .map_err(|_| {
                eprintln!("[warn] CW: invalid integer for {key}: {val:?}");
            })
            .ok()
    })
}

fn get_list(key: &str) -> Option<Vec<String>> {
    get_string(key).map(|val| {
        val.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    })
}

// ---------------------------------------------------------------------------
// Enum parsers
// ---------------------------------------------------------------------------

fn parse_scope_mode(val: &str, key: &str) -> Option<ScopeMode> {
    match val.to_lowercase().as_str() {
        "disabled" => Some(ScopeMode::Disabled),
        "optional" => Some(ScopeMode::Optional),
        "required" => Some(ScopeMode::Required),
        _ => {
            eprintln!(
                "[warn] CW: invalid ScopeMode for {key}: {val:?} — expected disabled/optional/required"
            );
            None
        }
    }
}

fn parse_ticket_source(val: &str, key: &str) -> Option<TicketSource> {
    match val.to_lowercase().as_str() {
        "branch" => Some(TicketSource::Branch),
        "prompt" => Some(TicketSource::Prompt),
        "branch_or_prompt" => Some(TicketSource::BranchOrPrompt),
        "disabled" => Some(TicketSource::Disabled),
        _ => {
            eprintln!(
                "[warn] CW: invalid TicketSource for {key}: {val:?} — expected branch/prompt/branch_or_prompt/disabled"
            );
            None
        }
    }
}

fn parse_enforcement_scope(val: &str, key: &str) -> Option<CommitEnforcementScope> {
    match val.to_lowercase().as_str() {
        "all_branches" | "all" => Some(CommitEnforcementScope::AllBranches),
        "protected_branches" | "protected" => Some(CommitEnforcementScope::ProtectedBranches),
        "none" => Some(CommitEnforcementScope::None),
        _ => {
            eprintln!(
                "[warn] CW: invalid CommitEnforcementScope for {key}: {val:?} — expected all_branches/protected_branches/none"
            );
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Build a `BaseConfig` from environment variables. Returns `None` when no
/// relevant `CW_*` variables are set (so lower-priority layers still apply).
///
/// Meta-flag `CW_ALLOW_ENV_OVERRIDE=false` disables this layer entirely.
pub fn build_env_config() -> Option<BaseConfig> {
    // Honour global opt-out flag.
    if let Some(val) = get_string(ENV_ALLOW_ENV_OVERRIDE)
        && !parse_bool_val(&val, ENV_ALLOW_ENV_OVERRIDE).unwrap_or(true)
    {
        return None;
    }

    let commit = build_commit();
    let branch = build_branch();
    let pr = build_pr();
    let check = build_check();
    let push = build_push();
    let versioning = build_versioning();

    if commit.is_none()
        && branch.is_none()
        && pr.is_none()
        && check.is_none()
        && push.is_none()
        && versioning.is_none()
    {
        return None;
    }

    Some(BaseConfig {
        commit,
        branch,
        pr,
        check,
        push,
        versioning,
        changelog: None,
        release: None,
        ai: None,
        hooks: None,
        registry: None,
        registries: None,
    })
}

// ---------------------------------------------------------------------------
// Section builders
// ---------------------------------------------------------------------------

fn build_commit() -> Option<CommitConfig> {
    let subject_max_length = get_u32(ENV_COMMIT_SUBJECT_MAX_LENGTH);
    let scopes = {
        let mode = get_string(ENV_COMMIT_SCOPES_MODE)
            .and_then(|v| parse_scope_mode(&v, ENV_COMMIT_SCOPES_MODE));
        let restrict_to_defined = get_bool(ENV_COMMIT_SCOPES_RESTRICT_TO_DEFINED);
        some_if_any!([mode, restrict_to_defined] =>
            CommitScopesConfig { mode, restrict_to_defined, definitions: None })
    };
    let ticket = {
        let required = get_bool(ENV_COMMIT_TICKET_REQUIRED);
        let pattern = get_string(ENV_COMMIT_TICKET_PATTERN);
        let source = get_string(ENV_COMMIT_TICKET_SOURCE)
            .and_then(|v| parse_ticket_source(&v, ENV_COMMIT_TICKET_SOURCE));
        some_if_any!([required, pattern, source] =>
            TicketConfig { required, pattern, source, header_format: None })
    };
    some_if_any!([subject_max_length, scopes, ticket] =>
    CommitConfig {
        subject_max_length,
        use_emojis: None,
        types: None,
        scopes,
        breaking: None,
        protected: None,
        ticket,
    })
}

fn build_branch() -> Option<BranchConfig> {
    let remote = get_string(ENV_BRANCH_REMOTE);
    let protected = get_list(ENV_BRANCH_PROTECTED);
    let naming = get_string(ENV_BRANCH_NAMING_PATTERN).map(|pattern| BranchNamingConfig {
        pattern: Some(pattern),
    });
    some_if_any!([remote, protected, naming] => BranchConfig { remote, protected, naming })
}

fn build_pr() -> Option<PrConfig> {
    let title = {
        let require_conventional = get_bool(ENV_PR_TITLE_REQUIRE_CONVENTIONAL);
        let require_ticket = get_bool(ENV_PR_TITLE_REQUIRE_TICKET);
        let scope_mode = get_string(ENV_PR_TITLE_SCOPE_MODE)
            .and_then(|v| parse_scope_mode(&v, ENV_PR_TITLE_SCOPE_MODE));
        some_if_any!([require_conventional, require_ticket, scope_mode] =>
            PrTitleConfig { require_conventional, require_ticket, scope_mode })
    };
    let branch = {
        let source_pattern = get_string(ENV_PR_BRANCH_SOURCE_PATTERN);
        let target_allowed = get_list(ENV_PR_BRANCH_TARGET_ALLOWED);
        some_if_any!([source_pattern, target_allowed] =>
        PrBranchConfig {
            check_source: None,
            check_target: None,
            source_pattern,
            target_allowed,
        })
    };
    some_if_any!([title, branch] => PrConfig { enabled: None, title, branch })
}

fn build_check() -> Option<CheckConfig> {
    let require_conventional = get_bool(ENV_CHECK_REQUIRE_CONVENTIONAL);
    let commits = {
        let enabled = get_bool(ENV_CHECK_COMMITS_ENABLED);
        let enforce_on = get_string(ENV_CHECK_COMMITS_ENFORCE_ON)
            .and_then(|v| parse_enforcement_scope(&v, ENV_CHECK_COMMITS_ENFORCE_ON));
        some_if_any!([enabled, enforce_on] => CheckCommitsConfig { enabled, enforce_on })
    };
    some_if_any!([require_conventional, commits] => CheckConfig { require_conventional, commits })
}

fn build_push() -> Option<PushConfig> {
    let allow = {
        let protected = get_bool(ENV_PUSH_ALLOW_PROTECTED);
        let force = get_bool(ENV_PUSH_ALLOW_FORCE);
        some_if_any!([protected, force] => PushAllowConfig { protected, force })
    };
    let check = {
        let commits = get_bool(ENV_PUSH_CHECK_COMMITS);
        let branch_policy = get_bool(ENV_PUSH_CHECK_BRANCH_POLICY);
        some_if_any!([commits, branch_policy] => PushCheckConfig { commits, branch_policy })
    };
    some_if_any!([allow, check] => PushConfig { allow, check })
}

fn build_versioning() -> Option<VersioningConfig> {
    get_string(ENV_VERSIONING_TAG_PREFIX).map(|tag_prefix| VersioningConfig {
        tag_prefix: Some(tag_prefix),
    })
}

// ---------------------------------------------------------------------------
// Registry selection helpers (used by registry resolver)
// ---------------------------------------------------------------------------

/// Registry parameters sourced from ENV variables (SRS §6).
pub struct EnvRegistryParams {
    pub url: Option<String>,
    pub r#ref: Option<String>,
    pub section: Option<String>,
}

pub fn get_env_registry_params() -> EnvRegistryParams {
    EnvRegistryParams {
        url: get_string(ENV_REGISTRY_URL),
        r#ref: get_string(ENV_REGISTRY_REF),
        section: get_string(ENV_REGISTRY_SECTION),
    }
}
