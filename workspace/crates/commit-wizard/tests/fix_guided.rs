//! `cw fix` on a terminal walks through each offending commit.
#![cfg(unix)]

mod common;

use common::{Pty, Repo};

fn commit_file(repo: &Repo, name: &str, message: &str) {
    repo.write(name, name);
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", message]);
}

/// feat: good | added a button | feat: fine | wip stuff
fn history() -> Repo {
    let repo = Repo::new();
    commit_file(&repo, "a.txt", "feat: good");
    commit_file(&repo, "b.txt", "added a button");
    commit_file(&repo, "c.txt", "feat: fine");
    commit_file(&repo, "d.txt", "wip stuff");
    repo
}

fn subjects(repo: &Repo) -> Vec<String> {
    repo.git(&["log", "--reverse", "--format=%s"])
        .lines()
        .map(str::to_string)
        .collect()
}

const ACTION: &str = "What would you like to do with this commit?";
const TYPE: &str = "Which type is it?";
const SCOPE: &str = "Scope";
const SUBJECT: &str = "Short subject";
const BREAKING: &str = "Is this a breaking change?";
const CONFIRM: &str = "get new hashes";

/// Reword a commit, taking the suggested type and the proposed subject.
fn reword_with_defaults(cw: &mut Pty) {
    cw.expect(ACTION);
    cw.line(""); // Reword it
    cw.expect(TYPE);
    cw.line("");
    cw.expect(SCOPE);
    cw.line("");
    cw.expect(SUBJECT);
    cw.line("");
    cw.expect(BREAKING);
    cw.line("n");
}

#[test]
fn rewording_every_offender_with_the_suggestions() {
    let repo = history();
    let mut cw = repo.cw_pty(&["fix"]);

    // "added a button" -> suggested feat, subject "added a button".
    reword_with_defaults(&mut cw);
    // "wip stuff" has no suggestion, so the first type is offered.
    reword_with_defaults(&mut cw);
    cw.expect(CONFIRM);
    cw.line("y");

    assert!(cw.finish());
    let log = subjects(&repo);
    assert_eq!(log[2], "feat: added a button", "{log:?}");
    assert_eq!(log[4], "chore: wip stuff", "{log:?}");
}

#[test]
fn the_type_the_wording_hints_at_is_offered_first() {
    let repo = Repo::new();
    commit_file(&repo, "a.txt", "Fixed the parser");
    let mut cw = repo.cw_pty(&["fix"]);

    cw.expect(ACTION);
    cw.line("");
    cw.expect(TYPE);
    cw.expect("fix (suggested)");
    cw.line("");
    cw.expect(SCOPE);
    cw.line("");
    cw.expect(SUBJECT);
    cw.line("");
    cw.expect(BREAKING);
    cw.line("n");
    cw.expect(CONFIRM);
    cw.line("y");

    assert!(cw.finish());
    assert_eq!(subjects(&repo).last().unwrap(), "fix: Fixed the parser");
}

#[test]
fn a_commit_can_be_skipped() {
    let repo = history();
    let mut cw = repo.cw_pty(&["fix"]);

    cw.expect(ACTION);
    cw.line("Skip it");
    reword_with_defaults(&mut cw);
    cw.expect(CONFIRM);
    cw.line("y");

    assert!(cw.finish());
    let log = subjects(&repo);
    assert_eq!(
        log[2], "added a button",
        "the skipped commit must stay: {log:?}"
    );
    assert_eq!(log[4], "chore: wip stuff", "{log:?}");
}

#[test]
fn stopping_keeps_what_was_decided_so_far() {
    let repo = history();
    let mut cw = repo.cw_pty(&["fix"]);

    reword_with_defaults(&mut cw);
    cw.expect(ACTION);
    cw.line("Stop here");
    cw.expect(CONFIRM);
    cw.line("y");

    assert!(cw.finish());
    let log = subjects(&repo);
    assert_eq!(log[2], "feat: added a button", "{log:?}");
    assert_eq!(log[4], "wip stuff", "{log:?}");
}

#[test]
fn declining_the_final_question_rewrites_nothing() {
    let repo = history();
    let before = repo.git(&["rev-parse", "HEAD"]);
    let mut cw = repo.cw_pty(&["fix"]);

    reword_with_defaults(&mut cw);
    reword_with_defaults(&mut cw);
    cw.expect(CONFIRM);
    cw.line("n");

    assert!(cw.finish(), "declining is not an error");
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), before);
    assert!(repo.git(&["for-each-ref", "refs/cw/fix-backup"]).is_empty());
}

#[test]
fn skipping_everything_changes_nothing() {
    let repo = history();
    let before = repo.git(&["rev-parse", "HEAD"]);
    let mut cw = repo.cw_pty(&["fix"]);

    cw.expect(ACTION);
    cw.line("Skip it");
    cw.expect(ACTION);
    cw.line("Skip it");

    assert!(cw.finish());
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), before);
}

#[test]
fn a_breaking_change_gets_a_marker_and_a_footer() {
    let repo = Repo::new();
    commit_file(&repo, "a.txt", "dropped the old api");
    let mut cw = repo.cw_pty(&["fix"]);

    cw.expect(ACTION);
    cw.line("");
    cw.expect(TYPE);
    cw.line("feat");
    cw.expect(SCOPE);
    cw.line("");
    cw.expect(SUBJECT);
    cw.line("");
    cw.expect(BREAKING);
    cw.line("y");
    cw.expect("Describe the breaking change");
    cw.line("the v1 api is gone");
    cw.expect(CONFIRM);
    cw.line("y");

    assert!(cw.finish());
    let message = repo.git(&["log", "-1", "--format=%B"]);
    assert!(
        message.starts_with("feat!: dropped the old api"),
        "{message}"
    );
    assert!(
        message.contains("BREAKING CHANGE: the v1 api is gone"),
        "{message}"
    );
}

#[test]
fn a_message_that_is_still_wrong_is_asked_about_again() {
    // Scopes are required, so leaving the scope empty is not good enough.
    let repo = Repo::new();
    let path = repo.path().join(".cwizard.toml");
    let config = std::fs::read_to_string(&path)
        .unwrap()
        .replace("mode = \"optional\"", "mode = \"required\"");
    std::fs::write(&path, config).unwrap();
    repo.git(&["add", "-A"]);
    // The init commit needs a scope too, or it would also break the new rule.
    repo.git(&["commit", "-q", "--amend", "-m", "chore(init): init"]);
    commit_file(&repo, "a.txt", "added a button");
    let mut cw = repo.cw_pty(&["fix"]);

    cw.expect(ACTION);
    cw.line("");
    cw.expect(TYPE);
    cw.line("");
    cw.expect(SCOPE);
    cw.line(""); // no scope
    cw.expect(SUBJECT);
    cw.line("");
    cw.expect(BREAKING);
    cw.line("n");
    cw.expect("still breaks the rules");
    cw.expect("Scope is required");
    // Asked again from the type onwards.
    cw.expect(TYPE);
    cw.line("");
    cw.expect(SCOPE);
    cw.line("ui");
    cw.expect(SUBJECT);
    cw.line("");
    cw.expect(BREAKING);
    cw.line("n");
    cw.expect(CONFIRM);
    cw.line("y");

    assert!(cw.finish());
    assert_eq!(subjects(&repo).last().unwrap(), "feat(ui): added a button");
}

#[test]
fn yes_and_non_interactive_never_prompt() {
    for flag in ["--yes", "--non-interactive"] {
        let repo = history();
        let before = repo.git(&["rev-parse", "HEAD"]);
        let cw = repo.cw_pty(&["fix", flag]);

        // There is nothing to default to, so it stops instead of waiting.
        assert!(!cw.finish(), "{flag}");
        assert_eq!(repo.git(&["rev-parse", "HEAD"]), before, "{flag}");
    }
}
