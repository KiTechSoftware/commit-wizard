//! Ticket references in commit subjects: finding one in a branch name, checking it
//! against the project's pattern, and writing it into the subject.
//!
//! `cw` assumes no ticketing system. The `pattern` in the config says what a ticket
//! looks like (a Jira key, `#123`, `Ticket-012`, anything); without one, any text is
//! accepted as a ticket but none can be found in a branch name.

use regex::Regex;

use crate::engine::error::{ErrorCode, Result};

fn compile(pattern: &str) -> Result<Regex> {
    Regex::new(pattern).map_err(|err| {
        ErrorCode::ConfigInvalid
            .error()
            .with_context("key", "commit.ticket.pattern")
            .with_context("value", pattern)
            .with_context("reason", err.to_string())
    })
}

/// The first ticket in `branch`, if `pattern` is set and matches. A pattern with a
/// capture group yields that group; otherwise the whole match.
pub fn from_branch(pattern: Option<&str>, branch: &str) -> Result<Option<String>> {
    let Some(pattern) = pattern.filter(|p| !p.trim().is_empty()) else {
        return Ok(None);
    };
    let regex = compile(pattern)?;
    Ok(regex.captures(branch).and_then(|caps| {
        caps.get(1)
            .or_else(|| caps.get(0))
            .map(|m| m.as_str().to_string())
    }))
}

/// Whether `ticket` is acceptable: any non-empty text without a pattern, otherwise
/// text the pattern matches (the same test `cw check` applies to a message).
pub fn is_valid(pattern: Option<&str>, ticket: &str) -> Result<bool> {
    if ticket.trim().is_empty() {
        return Ok(false);
    }
    match pattern.filter(|p| !p.trim().is_empty()) {
        Some(pattern) => Ok(compile(pattern)?.is_match(ticket)),
        None => Ok(true),
    }
}

/// `summary` with the ticket written in front of it as `header_format` says, so
/// `{ticket}: ` turns "add parser" into "PROJ-12: add parser". A summary that
/// already mentions the ticket is left as it is.
pub fn with_ticket(header_format: &str, ticket: &str, summary: &str) -> Result<String> {
    if !header_format.contains("{ticket}") {
        return Err(ErrorCode::ConfigInvalid
            .error()
            .with_context("key", "commit.ticket.header_format")
            .with_context("value", header_format)
            .with_context("reason", "must contain {ticket}"));
    }
    if summary.contains(ticket) {
        return Ok(summary.to_string());
    }
    Ok(format!(
        "{}{summary}",
        header_format.replace("{ticket}", ticket)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_a_ticket_in_a_branch_name_with_the_projects_own_pattern() {
        for (pattern, branch, expected) in [
            (r"[A-Z]+-\d+", "feature/PROJ-123-login", Some("PROJ-123")),
            (r"#\d+", "fix/#42-crash", Some("#42")),
            (r"Ticket-\d+", "wip/Ticket-012", Some("Ticket-012")),
            (r"feature/(\d+)", "feature/77-thing", Some("77")),
            (r"[A-Z]+-\d+", "main", None),
        ] {
            assert_eq!(
                from_branch(Some(pattern), branch).unwrap().as_deref(),
                expected,
                "{pattern} in {branch}"
            );
        }
    }

    #[test]
    fn without_a_pattern_nothing_is_guessed_from_the_branch() {
        assert_eq!(from_branch(None, "feature/PROJ-123").unwrap(), None);
        assert_eq!(from_branch(Some("  "), "feature/PROJ-123").unwrap(), None);
    }

    #[test]
    fn a_bad_pattern_is_a_config_error() {
        assert!(from_branch(Some("("), "x").is_err());
        assert!(is_valid(Some("("), "x").is_err());
    }

    #[test]
    fn validity_follows_the_pattern_or_accepts_any_text() {
        assert!(is_valid(Some(r"PROJ-\d+"), "PROJ-1").unwrap());
        assert!(!is_valid(Some(r"PROJ-\d+"), "nope").unwrap());
        assert!(is_valid(None, "anything goes").unwrap());
        assert!(!is_valid(None, "  ").unwrap());
    }

    #[test]
    fn the_ticket_is_written_at_the_start_of_the_subject() {
        assert_eq!(
            with_ticket("{ticket}: ", "Ticket-012", "add ticketing").unwrap(),
            "Ticket-012: add ticketing"
        );
        assert_eq!(
            with_ticket("[{ticket}] ", "PROJ-1", "add parser").unwrap(),
            "[PROJ-1] add parser"
        );
        assert_eq!(
            with_ticket("{ticket}: ", "PROJ-1", "PROJ-1: already there").unwrap(),
            "PROJ-1: already there"
        );
        assert!(with_ticket("no placeholder", "PROJ-1", "x").is_err());
    }
}
