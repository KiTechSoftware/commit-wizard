use crate::engine::{
    capabilities::commit::check::ValidatedCommit,
    models::policy::{Policy, check::CommitCheckEnforcement},
};

#[derive(Debug, Clone)]
pub struct PushReport {
    pub current_branch: String,
    pub protected_branch: bool,
    pub commits: Vec<ValidatedCommit>,
    pub total_checked: usize,
    pub invalid_count: usize,
    pub blocked: bool,
    pub block_reasons: Vec<String>,
}

pub fn evaluate_push(
    policy: &Policy,
    current_branch: &str,
    commits: Vec<ValidatedCommit>,
) -> PushReport {
    let protected_branch = is_protected_branch(current_branch, &policy.branch.protected_patterns);
    let total_checked = commits.len();
    let invalid_count = commits.iter().filter(|c| !c.valid).count();

    let mut blocked = false;
    let mut block_reasons = Vec::<String>::new();

    // Branch protection
    if policy.push.check_branch_policy && protected_branch && !policy.push.allow_protected {
        blocked = true;
        block_reasons.push(format!(
            "push to protected branch '{}' is blocked by policy",
            current_branch
        ));
    }

    // Commit validation enforcement
    if policy.push.check_commits
        && policy.check.commits_enabled
        && policy.check.require_conventional
        && should_enforce_commit_check(policy, protected_branch)
        && invalid_count > 0
    {
        blocked = true;
        block_reasons.push(format!(
            "{} commit(s) do not satisfy conventional commit policy",
            invalid_count
        ));
    }

    PushReport {
        current_branch: current_branch.to_string(),
        protected_branch,
        commits,
        total_checked,
        invalid_count,
        blocked,
        block_reasons,
    }
}

fn should_enforce_commit_check(policy: &Policy, protected_branch: bool) -> bool {
    match policy.check.enforcement {
        CommitCheckEnforcement::AllBranches => true,
        CommitCheckEnforcement::ProtectedBranches => protected_branch,
        CommitCheckEnforcement::None => false,
    }
}

fn is_protected_branch(branch: &str, patterns: &[String]) -> bool {
    patterns
        .iter()
        .any(|pattern| branch_matches_pattern(branch, pattern))
}

fn branch_matches_pattern(branch: &str, pattern: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix("/*") {
        return branch == prefix || branch.starts_with(&format!("{prefix}/"));
    }

    branch == pattern
}
