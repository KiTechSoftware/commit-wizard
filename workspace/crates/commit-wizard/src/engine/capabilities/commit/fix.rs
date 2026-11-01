//! Composing corrected commit messages for `cw fix`.
//!
//! Pure functions: given an offending message and the answers a person gave
//! (or flags supplied), produce the message that should replace it.

use regex::Regex;

use crate::engine::models::policy::commit::CommitModel;

/// `type(scope)!: summary`, with the type's emoji in front when the policy uses them.
pub fn build_header(
    commit_type: &str,
    scope: Option<&str>,
    summary: &str,
    breaking: bool,
    emoji: Option<&str>,
    use_emojis: bool,
) -> String {
    let type_prefix = if let (true, Some(e)) = (use_emojis, emoji) {
        format!("{e} {commit_type}")
    } else {
        commit_type.to_string()
    };

    match (scope, breaking) {
        (Some(scope), true) => format!("{type_prefix}({scope})!: {summary}"),
        (Some(scope), false) => format!("{type_prefix}({scope}): {summary}"),
        (None, true) => format!("{type_prefix}!: {summary}"),
        (None, false) => format!("{type_prefix}: {summary}"),
    }
}

/// What a person decided a commit should say.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reword {
    pub commit_type: String,
    pub scope: Option<String>,
    pub subject: String,
    pub breaking: bool,
    /// Text for the `BREAKING CHANGE:` footer, if one should be added.
    pub breaking_note: Option<String>,
}

/// Everything after the first line of a message (its body and footers), with
/// the blank line that separates it, or nothing if there is none.
pub fn body_of(message: &str) -> &str {
    match message.split_once('\n') {
        Some((_, rest)) => rest.trim_end(),
        None => "",
    }
}

/// The corrected message: a new header, the original body and footers kept,
/// and a breaking-change footer added when asked for and not already there.
pub fn reworded_message(original: &str, reword: &Reword, policy: &CommitModel) -> String {
    let emoji = policy
        .find_type(&reword.commit_type)
        .and_then(|t| t.emoji.as_deref());
    let header = build_header(
        &reword.commit_type,
        reword.scope.as_deref(),
        &reword.subject,
        reword.breaking,
        emoji,
        policy.use_emojis,
    );

    let mut message = header;
    let body = body_of(original);
    if !body.trim().is_empty() {
        message.push('\n');
        message.push_str(body);
    }

    let footer_key = &policy.breaking_footer_key;
    if let Some(note) = reword.breaking_note.as_deref().map(str::trim)
        && !note.is_empty()
        && !message.contains(&format!("{footer_key}:"))
    {
        message.push_str(&format!("\n\n{footer_key}: {note}"));
    }
    message
}

/// The original message with its header line replaced, body and footers kept.
pub fn with_header(original: &str, header: &str) -> String {
    let body = body_of(original);
    if body.trim().is_empty() {
        header.trim().to_string()
    } else {
        format!("{}\n{body}", header.trim())
    }
}

/// The summary without any `type:` / `type(scope):` prefix or trailing period,
/// as a starting point for the new subject.
pub fn suggested_subject(summary: &str) -> String {
    let re = Regex::new(r"^\S+?(\([^)]*\))?!?:\s*(.+)$").expect("valid pattern");
    let text = re
        .captures(summary.trim())
        .and_then(|caps| caps.get(2))
        .map(|m| m.as_str())
        .unwrap_or_else(|| summary.trim());
    text.trim().trim_end_matches('.').to_string()
}

/// A commit type the wording hints at, if the policy has it ("Fixed the bug" -> fix).
pub fn suggested_type(summary: &str, policy: &CommitModel) -> Option<String> {
    let first = summary
        .trim()
        .trim_start_matches(|c: char| !c.is_alphanumeric())
        .split(|c: char| !c.is_alphanumeric())
        .next()?
        .to_ascii_lowercase();

    let guess = match first.as_str() {
        "fix" | "fixed" | "fixes" | "bugfix" | "bug" | "hotfix" => "fix",
        "feat" | "feature" | "add" | "added" | "adds" | "implement" | "implemented" => "feat",
        "doc" | "docs" | "document" | "documented" | "readme" => "docs",
        "refactor" | "refactored" | "cleanup" | "clean" | "rename" | "renamed" => "refactor",
        "test" | "tests" | "testing" => "test",
        "perf" | "optimize" | "optimise" | "speed" => "perf",
        "style" | "format" | "formatting" | "lint" => "style",
        "build" | "deps" | "bump" | "upgrade" => "build",
        "ci" => "ci",
        "chore" | "update" | "updated" | "misc" => "chore",
        other => other,
    };
    policy.allows_type(guess).then(|| guess.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> CommitModel {
        CommitModel::default()
    }

    fn reword(commit_type: &str, scope: Option<&str>, subject: &str) -> Reword {
        Reword {
            commit_type: commit_type.to_string(),
            scope: scope.map(str::to_string),
            subject: subject.to_string(),
            breaking: false,
            breaking_note: None,
        }
    }

    #[test]
    fn headers_have_scope_breaking_marker_and_optional_emoji() {
        assert_eq!(
            build_header("feat", None, "x", false, None, false),
            "feat: x"
        );
        assert_eq!(
            build_header("feat", Some("api"), "x", true, None, false),
            "feat(api)!: x"
        );
        assert_eq!(
            build_header("fix", None, "x", false, Some("🐛"), true),
            "🐛 fix: x"
        );
        assert_eq!(
            build_header("fix", None, "x", false, Some("🐛"), false),
            "fix: x"
        );
    }

    #[test]
    fn the_body_and_footers_survive_a_reword() {
        let original =
            "Fixed it\n\nLonger explanation\nover two lines\n\nSigned-off-by: A <a@b.c>\n";
        let message = reworded_message(
            original,
            &reword("fix", Some("core"), "handle empty input"),
            &policy(),
        );

        assert_eq!(
            message,
            "fix(core): handle empty input\n\nLonger explanation\nover two lines\n\nSigned-off-by: A <a@b.c>"
        );
    }

    #[test]
    fn a_message_with_no_body_stays_one_line() {
        let message = reworded_message("oops", &reword("chore", None, "tidy"), &policy());
        assert_eq!(message, "chore: tidy");
    }

    #[test]
    fn a_breaking_note_adds_a_footer_once() {
        let mut breaking = reword("feat", None, "drop v1");
        breaking.breaking = true;
        breaking.breaking_note = Some("the v1 api is gone".to_string());

        let plain = reworded_message("dropped v1", &breaking, &policy());
        assert_eq!(
            plain,
            "feat!: drop v1\n\nBREAKING CHANGE: the v1 api is gone"
        );

        // A footer already present is not duplicated.
        let had = reworded_message("x\n\nBREAKING CHANGE: already said", &breaking, &policy());
        assert_eq!(had.matches("BREAKING CHANGE:").count(), 1, "{had}");
    }

    #[test]
    fn replacing_just_the_header_keeps_the_rest() {
        assert_eq!(with_header("old\n\nbody", "fix: new"), "fix: new\n\nbody");
        assert_eq!(with_header("old", " fix: new "), "fix: new");
    }

    #[test]
    fn subjects_lose_a_bad_prefix_and_trailing_period() {
        assert_eq!(
            suggested_subject("fixx(core): handle empty input."),
            "handle empty input"
        );
        assert_eq!(suggested_subject("Fixed the parser."), "Fixed the parser");
        assert_eq!(suggested_subject("  tidy up  "), "tidy up");
    }

    #[test]
    fn types_are_guessed_from_the_wording_only_when_the_policy_has_them() {
        use crate::engine::models::policy::{commit::CommitTypeModel, enforcement::BumpLevel};
        let mut policy = policy();
        policy.types.push(CommitTypeModel {
            key: "docs".to_string(),
            emoji: None,
            description: None,
            bump: BumpLevel::Patch,
            section: "Documentation".to_string(),
        });
        // `refactor` is not in this policy, so it is never suggested.
        assert_eq!(suggested_type("Refactored the parser", &policy), None);
        assert_eq!(
            suggested_type("Fixed the parser", &policy).as_deref(),
            Some("fix")
        );
        assert_eq!(
            suggested_type("Add a button", &policy).as_deref(),
            Some("feat")
        );
        assert_eq!(
            suggested_type("docs tweak", &policy).as_deref(),
            Some("docs")
        );
        assert_eq!(suggested_type("wip", &policy), None);
        assert_eq!(suggested_type("", &policy), None);
    }
}
