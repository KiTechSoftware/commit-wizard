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

/// The part of a branch name a `{token}` in the template stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameToken {
    Type,
    Ticket,
    Slug,
}

impl NameToken {
    fn parse(text: &str) -> Option<Self> {
        match text {
            "type" => Some(Self::Type),
            "ticket" => Some(Self::Ticket),
            "slug" => Some(Self::Slug),
            _ => None,
        }
    }
}

/// One `{token}` and the literal text that follows it, up to the next token.
/// The two travel together: a part that is left out takes its separator with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameUnit {
    pub token: NameToken,
    pub separator: String,
}

/// A parsed `branch.naming.pattern`, such as `{type}/{ticket}-{slug}`.
///
/// The tokens are `{type}`, `{ticket}` and `{slug}` (the description as words joined by
/// hyphens); `{slug}` is required. The usual shapes are `{slug}`, `{type}/{slug}`,
/// `{ticket}-{slug}` and `{type}/{ticket}-{slug}`, and other separators work too.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameTemplate {
    /// Fixed text before the first token, such as `feature/` in `feature/{slug}`.
    prefix: String,
    units: Vec<NameUnit>,
}

impl NameTemplate {
    pub fn parse(template: &str) -> Result<Self> {
        let invalid = |reason: String| {
            ErrorCode::ConfigInvalid
                .error()
                .with_context("key", "branch.naming.pattern")
                .with_context("value", template)
                .with_context("reason", reason)
        };

        let mut units: Vec<NameUnit> = Vec::new();
        let mut rest = template;
        let Some(first) = rest.find('{') else {
            return Err(invalid(
                "expected a template such as {type}/{ticket}-{slug}".to_string(),
            ));
        };
        let prefix = rest[..first].to_string();
        rest = &rest[first..];
        while let Some(start) = rest.find('{') {
            let after = &rest[start + 1..];
            let Some(end) = after.find('}') else {
                return Err(invalid("a { is never closed".to_string()));
            };
            let token = NameToken::parse(&after[..end]).ok_or_else(|| {
                invalid(format!(
                    "unknown {{{}}}; use {{type}}, {{ticket}} or {{slug}}",
                    &after[..end]
                ))
            })?;
            let tail = &after[end + 1..];
            let (separator, next) = match tail.find('{') {
                Some(next) => (&tail[..next], &tail[next..]),
                None => (tail, ""),
            };
            units.push(NameUnit {
                token,
                separator: separator.to_string(),
            });
            rest = next;
        }

        if !units.iter().any(|unit| unit.token == NameToken::Slug) {
            return Err(invalid("it must contain {slug}".to_string()));
        }
        Ok(Self { prefix, units })
    }

    pub fn has(&self, token: NameToken) -> bool {
        self.units.iter().any(|unit| unit.token == token)
    }

    /// The branch name for the parts given. A part that is missing is left out together
    /// with its separator, so `{type}/{ticket}-{slug}` gives `fix/login` with no ticket
    /// and `PROJ-1-login` with no type.
    pub fn render(
        &self,
        kind: Option<&str>,
        ticket: Option<&str>,
        description: &str,
    ) -> Result<String> {
        let kind = kind.map(slugify).filter(|k| !k.is_empty());
        let ticket = ticket
            .map(|t| t.split_whitespace().collect::<Vec<_>>().join("-"))
            .filter(|t| !t.is_empty());
        let slug = Some(slugify(description)).filter(|s| !s.is_empty());

        if ticket.is_none() && slug.is_none() {
            return Err(ErrorCode::InvalidInput
                .error()
                .with_context("field", "description")
                .with_context(
                    "reason",
                    "a branch needs a description or a ticket (pass --description or --ticket)",
                ));
        }

        let mut name = self.prefix.clone();
        for unit in &self.units {
            let value = match unit.token {
                NameToken::Type => kind.as_deref(),
                NameToken::Ticket => ticket.as_deref(),
                NameToken::Slug => slug.as_deref(),
            };
            if let Some(value) = value {
                name.push_str(value);
                name.push_str(&unit.separator);
            }
        }
        // What follows the last part present is a dangling separator ("PROJ-1-").
        let name = name.trim_end_matches(['-', '/', '_', '.']);
        Ok(name.to_string())
    }
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
///
/// With `include_merged`, a branch that still follows an existing remote branch is also
/// picked when it is merged. A branch following nothing never is: one just created at
/// the current commit looks merged too.
pub fn plan_prune(
    branches: &[BranchInfo],
    protected: &[String],
    include_merged: bool,
) -> PrunePlan {
    let mut plan = PrunePlan {
        candidates: Vec::new(),
        kept: Vec::new(),
    };
    let picked = |b: &&BranchInfo| {
        b.state == BranchState::Gone
            || (include_merged && b.merged && b.state != BranchState::Local)
    };
    for branch in branches.iter().filter(picked) {
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

/// Remote-tracking branches (`origin/topic`) that are merged and safe to delete from the
/// remote, and those left alone with the reason. Protected branches and the branch the
/// checked-out one follows are never picked.
pub fn plan_prune_remote(
    remotes: &[BranchInfo],
    locals: &[BranchInfo],
    protected: &[String],
) -> PrunePlan {
    let mut plan = PrunePlan {
        candidates: Vec::new(),
        kept: Vec::new(),
    };
    let followed = locals
        .iter()
        .find(|b| b.current)
        .and_then(|b| b.upstream.as_deref());
    for branch in remotes.iter().filter(|b| b.merged) {
        let short = branch
            .name
            .split_once('/')
            .map_or(&*branch.name, |(_, s)| s);
        if followed == Some(branch.name.as_str()) {
            plan.kept
                .push((branch.clone(), "the checked-out branch follows it"));
        } else if is_protected_branch(short, protected) {
            plan.kept.push((branch.clone(), "it is protected"));
        } else {
            plan.candidates.push((branch.clone(), PruneVerdict::Delete));
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

    fn render(template: &str, kind: Option<&str>, ticket: Option<&str>, text: &str) -> String {
        NameTemplate::parse(template)
            .unwrap()
            .render(kind, ticket, text)
            .unwrap()
    }

    #[test]
    fn the_four_usual_templates() {
        let (k, t, d) = (Some("feature"), Some("PROJ-123"), "Add login");
        assert_eq!(render("{slug}", k, t, d), "add-login");
        assert_eq!(render("{type}/{slug}", k, t, d), "feature/add-login");
        assert_eq!(render("{ticket}-{slug}", k, t, d), "PROJ-123-add-login");
        assert_eq!(
            render("{type}/{ticket}-{slug}", k, t, d),
            "feature/PROJ-123-add-login"
        );
    }

    #[test]
    fn a_part_that_is_missing_takes_its_separator_with_it() {
        let template = "{type}/{ticket}-{slug}";
        assert_eq!(render(template, Some("fix"), None, "crash"), "fix/crash");
        assert_eq!(render(template, None, Some("#42"), "crash"), "#42-crash");
        assert_eq!(render(template, None, None, "crash"), "crash");
        assert_eq!(
            render(template, Some("bugfix"), Some("Ticket 9"), "crash"),
            "bugfix/Ticket-9-crash"
        );
        // A ticket alone names the branch; so does a description alone.
        assert_eq!(
            render(template, Some("bugfix"), Some("#42"), ""),
            "bugfix/#42"
        );
        assert_eq!(render(template, None, Some("Ticket 9"), ""), "Ticket-9");
    }

    #[test]
    fn a_branch_needs_a_description_or_a_ticket() {
        let template = NameTemplate::parse("{type}/{ticket}-{slug}").unwrap();
        assert!(template.render(Some("feature"), None, "  ").is_err());
        assert!(
            template
                .render(Some("feature"), Some("PROJ-1"), "  ")
                .is_ok()
        );
        assert!(template.render(Some("feature"), None, "x").is_ok());
    }

    #[test]
    fn fixed_text_before_the_first_part_is_kept() {
        assert_eq!(
            render("feature/{ticket}-{slug}", None, Some("A-1"), "x y"),
            "feature/A-1-x-y"
        );
        assert_eq!(
            render("feature/{slug}", Some("fix"), None, "x"),
            "feature/x"
        );
    }

    #[test]
    fn other_separators_work() {
        assert_eq!(
            render("{type}_{ticket}_{slug}", Some("fix"), Some("A-1"), "x y"),
            "fix_A-1_x-y"
        );
    }

    #[test]
    fn bad_templates_say_what_is_wrong() {
        for (template, expected) in [
            ("feature/{issue}", "unknown {issue}"),
            ("{type}/{ticket}", "{slug}"),
            ("^(feature|bugfix)/.*$", "template"),
            ("{slug", "never closed"),
            ("{type}{ticket}", "{slug}"),
        ] {
            let err = NameTemplate::parse(template).unwrap_err();
            let reason = err
                .context
                .iter()
                .find(|(key, _)| key == "reason")
                .map(|(_, value)| value.as_str())
                .unwrap_or_default();
            assert!(reason.contains(expected), "{template}: {reason}");
        }
    }

    #[test]
    fn the_template_knows_which_parts_it_uses() {
        let template = NameTemplate::parse("{type}/{slug}").unwrap();
        assert!(template.has(NameToken::Type) && !template.has(NameToken::Ticket));
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
        let plan = plan_prune(&branches, &protected, false);

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
