//! AI-written commit messages: the prompt that asks for one under the project's rules, and the reading of the answer into the parts `cw commit` works with.

use cliai::GenerateRequest;
use regex::Regex;

use crate::engine::models::policy::commit::{CommitModel, ScopeRequirement};

/// The most diff text sent to the model; the rest is cut with a note.
pub const MAX_DIFF_CHARS: usize = 20_000;

/// A commit message the model proposed, split into the parts `cw commit` asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    pub commit_type: String,
    pub scope: Option<String>,
    pub summary: String,
    pub body: Option<String>,
    pub breaking: bool,
    pub breaking_message: Option<String>,
}

/// Cuts a diff down to what is sent to the model.
pub fn clip_diff(diff: &str) -> String {
    if diff.chars().count() <= MAX_DIFF_CHARS {
        return diff.to_string();
    }
    let clipped: String = diff.chars().take(MAX_DIFF_CHARS).collect();
    format!("{clipped}\n[diff truncated]")
}

/// The request that asks for a commit message for `diff` under `policy`.
pub fn commit_request(diff: &str, policy: &CommitModel) -> GenerateRequest {
    let types = policy
        .types
        .iter()
        .map(|t| t.key.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let mut rules = vec![
        "Output only the commit message, with no commentary and no code fences.".to_string(),
        "Use the Conventional Commits format: `type(scope): summary`, then optionally a blank line and a body.".to_string(),
        format!("The type must be one of: {types}."),
        format!("The summary line must be at most {} characters, in the imperative mood, with no trailing period.", policy.subject_max_length),
        "Do not add emoji.".to_string(),
    ];
    match policy.scope_requirement {
        ScopeRequirement::Disabled => rules.push("Do not use a scope.".to_string()),
        _ => {
            if let Some(scopes) = policy
                .allowed_scopes
                .as_ref()
                .filter(|_| policy.restrict_scopes_to_defined)
            {
                rules.push(format!(
                    "If you use a scope it must be one of: {}.",
                    scopes.join(", ")
                ));
            }
        }
    }
    rules.push(
        "For a breaking change, put `!` before the colon and add a `BREAKING CHANGE: <description>` line in the body.".to_string(),
    );

    GenerateRequest::new(format!(
        "Write a commit message for these staged changes:\n\n{}",
        clip_diff(diff)
    ))
    .with_persona("You write precise, concise Git commit messages.")
    .with_instructions(rules.join("\n"))
}

/// Reads the model's answer. `None` when it is not a Conventional Commit the
/// project's rules allow.
pub fn parse_suggestion(text: &str, policy: &CommitModel) -> Option<Suggestion> {
    let cleaned = text
        .lines()
        .filter(|line| !line.trim_start().starts_with("```"))
        .collect::<Vec<_>>()
        .join("\n");
    let cleaned = cleaned.trim();

    let mut lines = cleaned.lines();
    let header = lines.next()?.trim().trim_matches('`');
    let re = Regex::new(
        r"^(?P<type>[A-Za-z]+)(?:\((?P<scope>[^)]*)\))?(?P<bang>!)?:\s*(?P<summary>.+)$",
    )
    .ok()?;
    let caps = re.captures(header)?;

    let commit_type = caps["type"].to_lowercase();
    if !policy.allows_type(&commit_type) {
        return None;
    }

    let scope = caps
        .name("scope")
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| !s.is_empty())
        .filter(|_| policy.scope_requirement != ScopeRequirement::Disabled)
        .filter(|s| !policy.restrict_scopes_to_defined || policy.allows_scope(s));

    let summary = caps["summary"].trim().trim_end_matches('.').to_string();
    if summary.is_empty() {
        return None;
    }

    let mut breaking = caps.name("bang").is_some();
    let mut breaking_message = None;
    let mut body_lines = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if let Some(rest) = policy
            .breaking_footer_keys
            .iter()
            .chain(std::iter::once(&policy.breaking_footer_key))
            .find_map(|key| trimmed.strip_prefix(&format!("{key}:")))
        {
            breaking = true;
            let rest = rest.trim();
            if !rest.is_empty() {
                breaking_message = Some(rest.to_string());
            }
        } else {
            body_lines.push(line);
        }
    }
    let body = body_lines.join("\n").trim().to_string();

    Some(Suggestion {
        commit_type,
        scope,
        summary,
        body: (!body.is_empty()).then_some(body),
        breaking,
        breaking_message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> CommitModel {
        CommitModel::default()
    }

    #[test]
    fn reads_a_header_and_body() {
        let s =
            parse_suggestion("feat(api): add parser.\n\nHandles nested input.", &policy()).unwrap();
        assert_eq!(s.commit_type, "feat");
        assert_eq!(s.scope.as_deref(), Some("api"));
        assert_eq!(s.summary, "add parser");
        assert_eq!(s.body.as_deref(), Some("Handles nested input."));
        assert!(!s.breaking);
    }

    #[test]
    fn strips_code_fences() {
        let s = parse_suggestion("```\nfix: handle empty input\n```", &policy()).unwrap();
        assert_eq!(s.commit_type, "fix");
        assert_eq!(s.summary, "handle empty input");
    }

    #[test]
    fn reads_breaking_changes() {
        let s =
            parse_suggestion("feat!: drop v1\n\nBREAKING CHANGE: v1 is gone", &policy()).unwrap();
        assert!(s.breaking);
        assert_eq!(s.breaking_message.as_deref(), Some("v1 is gone"));
        assert_eq!(s.body, None);
    }

    #[test]
    fn rejects_unknown_types_and_prose() {
        assert!(parse_suggestion("wizardry: do magic", &policy()).is_none());
        assert!(parse_suggestion("Here is your commit message:", &policy()).is_none());
        assert!(parse_suggestion("", &policy()).is_none());
    }

    #[test]
    fn long_diffs_are_clipped() {
        let diff = "x".repeat(MAX_DIFF_CHARS + 50);
        assert!(clip_diff(&diff).ends_with("[diff truncated]"));
        assert_eq!(clip_diff("small"), "small");
    }

    #[test]
    fn the_request_lists_the_allowed_types() {
        let request = commit_request("diff", &policy());
        assert!(request.instructions.unwrap().contains("feat"));
    }
}
