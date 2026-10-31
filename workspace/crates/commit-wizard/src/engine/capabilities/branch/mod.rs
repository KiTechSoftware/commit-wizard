//! Branch names, and which local branches `cw branch prune` may delete.
//!
//! Pure functions over data: Git itself is `infra`'s business.

use crate::engine::{
    capabilities::commit::push::is_protected_branch,
    error::{ErrorCode, Result},
    models::git::{BranchInfo, BranchState},
};

/// The kinds of branch the guided `cw branch new` offers. Any word is accepted with `--type`.
pub const BRANCH_KINDS: [&str; 7] = [
    "feature", "bugfix", "hotfix", "chore", "docs", "refactor", "release",
];

const MAX_SLUG: usize = 50;

/// Lower-case words joined by hyphens: "Add the Login page!" becomes "add-the-login-page".
pub fn slugify(text: &str) -> String {
    let mut slug = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            slug.extend(ch.to_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_matches('-');
    let cut: String = slug.chars().take(MAX_SLUG).collect();
    cut.trim_end_matches('-').to_string()
}

/// `<kind>/<ticket>-<slug>`, leaving out the parts that are not given.
///
/// The ticket is kept as written (only whitespace becomes a hyphen), since it is
/// the project's own identifier.
pub fn branch_name(kind: Option<&str>, ticket: Option<&str>, description: &str) -> Result<String> {
    let slug = slugify(description);
    let ticket = ticket
        .map(|t| t.split_whitespace().collect::<Vec<_>>().join("-"))
        .filter(|t| !t.is_empty());

    let tail = match (&ticket, slug.is_empty()) {
        (Some(ticket), false) => format!("{ticket}-{slug}"),
        (Some(ticket), true) => ticket.clone(),
        (None, false) => slug,
        (None, true) => {
            return Err(ErrorCode::InvalidInput
                .error()
                .with_context("field", "name")
                .with_context(
                    "reason",
                    "a branch needs a name or a ticket (pass NAME or --ticket)",
                ));
        }
    };

    let kind = kind.map(slugify).filter(|k| !k.is_empty());
    Ok(match kind {
        Some(kind) => format!("{kind}/{tail}"),
        None => tail,
    })
}

/// How `cw branch list` words a branch's standing against its upstream.
pub fn state_label(state: &BranchState) -> String {
    match state {
        BranchState::Local => "local only".to_string(),
        BranchState::UpToDate => "up to date".to_string(),
        BranchState::Ahead(n) => format!("ahead {n}"),
        BranchState::Behind(n) => format!("behind {n}"),
        BranchState::Diverged { ahead, behind } => format!("ahead {ahead}, behind {behind}"),
        BranchState::Gone => "remote deleted".to_string(),
    }
}

/// Why a branch is not on the prune list, or what will happen to it if it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PruneVerdict {
    /// Its remote branch is gone and it is merged: safe to delete.
    Delete,
    /// Its remote branch is gone but it has commits the checked-out history lacks.
    /// Deleted only with `--force`, because the work may exist nowhere else.
    Unmerged,
}

#[derive(Debug, Clone)]
pub struct PrunePlan {
    pub candidates: Vec<(BranchInfo, PruneVerdict)>,
    /// Branches whose remote is gone but that are left alone, and why.
    pub kept: Vec<(BranchInfo, &'static str)>,
}

/// Sorts the local branches whose remote branch has gone into those to delete, those that
/// need `--force`, and those never touched (the current branch and protected ones).
pub fn plan_prune(branches: &[BranchInfo], protected: &[String]) -> PrunePlan {
    let mut plan = PrunePlan {
        candidates: Vec::new(),
        kept: Vec::new(),
    };
    for branch in branches.iter().filter(|b| b.state == BranchState::Gone) {
        if branch.current {
            plan.kept.push((branch.clone(), "it is checked out"));
        } else if is_protected_branch(&branch.name, protected) {
            plan.kept.push((branch.clone(), "it is protected"));
        } else if branch.merged {
            plan.candidates.push((branch.clone(), PruneVerdict::Delete));
        } else {
            plan.candidates
                .push((branch.clone(), PruneVerdict::Unmerged));
        }
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(name: &str, state: BranchState, current: bool, merged: bool) -> BranchInfo {
        BranchInfo {
            name: name.to_string(),
            current,
            upstream: Some(format!("origin/{name}")),
            state,
            merged,
            hash: "abc1234".to_string(),
            subject: "x".to_string(),
            updated: 0,
        }
    }

    #[test]
    fn slugs_are_lowercase_words_joined_by_hyphens() {
        assert_eq!(slugify("Add the Login page!"), "add-the-login-page");
        assert_eq!(slugify("  --weird__spacing--  "), "weird-spacing");
        assert_eq!(slugify("!!!"), "");
        assert!(slugify(&"word ".repeat(40)).len() <= 50);
        assert!(!slugify(&"word ".repeat(40)).ends_with('-'));
    }

    #[test]
    fn names_combine_kind_ticket_and_description() {
        assert_eq!(
            branch_name(Some("feature"), Some("PROJ-123"), "Add login").unwrap(),
            "feature/PROJ-123-add-login"
        );
        assert_eq!(
            branch_name(Some("fix"), Some("#42"), "crash").unwrap(),
            "fix/#42-crash"
        );
        assert_eq!(branch_name(None, None, "Add login").unwrap(), "add-login");
        assert_eq!(
            branch_name(Some("hotfix"), Some("Ticket 9"), "").unwrap(),
            "hotfix/Ticket-9"
        );
        assert!(branch_name(Some("feature"), None, "  ").is_err());
    }

    #[test]
    fn only_merged_unprotected_gone_branches_are_safe_to_delete() {
        let protected = vec!["main".to_string(), "release/*".to_string()];
        let branches = vec![
            info("done", BranchState::Gone, false, true),
            info("squashed", BranchState::Gone, false, false),
            info("here", BranchState::Gone, true, true),
            info("release/1.0", BranchState::Gone, false, true),
            info("live", BranchState::UpToDate, false, true),
            info("local-only", BranchState::Local, false, true),
        ];
        let plan = plan_prune(&branches, &protected);

        let names = |v: &PruneVerdict| -> Vec<&str> {
            plan.candidates
                .iter()
                .filter(|(_, verdict)| verdict == v)
                .map(|(b, _)| b.name.as_str())
                .collect()
        };
        assert_eq!(names(&PruneVerdict::Delete), ["done"]);
        assert_eq!(names(&PruneVerdict::Unmerged), ["squashed"]);
        let kept: Vec<_> = plan
            .kept
            .iter()
            .map(|(b, why)| (b.name.as_str(), *why))
            .collect();
        assert_eq!(
            kept,
            [
                ("here", "it is checked out"),
                ("release/1.0", "it is protected")
            ]
        );
    }
}
