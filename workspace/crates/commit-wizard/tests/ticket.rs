//! Ticket references in `cw commit`, and the `-a` / `-A` staging flags.
//!
//! `cw` assumes no ticketing system: the project's `pattern` says what a ticket is.

mod common;

use common::{Repo, text};

fn repo_with(config: &str) -> Repo {
    let repo = Repo::bare();
    repo.write(".cwizard.toml", config);
    repo.write("a.txt", "a");
    repo.git(&["add", "a.txt"]);
    repo
}

const JIRA_STYLE: &str = "version = 1\n[commit.ticket]\npattern = \"[A-Z]+-[0-9]+\"\n";

#[test]
fn the_ticket_opens_the_subject() {
    let repo = repo_with("version = 1\n[commit.ticket]\npattern = \"Ticket-[0-9]+\"\n");
    let out = repo.cw(&[
        "commit",
        "-t",
        "feat",
        "-m",
        "add ticketing",
        "--ticket",
        "Ticket-012",
    ]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.subjects()[0], "feat: Ticket-012: add ticketing");
}

#[test]
fn the_ticket_goes_after_the_scope() {
    let repo = repo_with(JIRA_STYLE);
    let out = repo.cw(&[
        "commit", "-t", "fix", "-s", "api", "-m", "x", "--ticket", "PROJ-7",
    ]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.subjects()[0], "fix(api): PROJ-7: x");
}

#[test]
fn a_commit_written_with_a_ticket_passes_cw_check() {
    let repo =
        repo_with("version = 1\n[commit.ticket]\nrequired = true\npattern = \"Ticket-[0-9]+\"\n");
    let out = repo.cw(&["commit", "-t", "feat", "-m", "add", "--ticket", "Ticket-1"]);
    assert!(out.status.success(), "{}", text(&out));
    let check = repo.cw(&["check", "--tail", "1"]);
    assert!(check.status.success(), "{}", text(&check));
}

#[test]
fn any_text_is_a_ticket_when_the_project_sets_no_pattern() {
    let repo = repo_with("version = 1\n");
    let out = repo.cw(&["commit", "-t", "fix", "-m", "x", "--ticket", "ACME 9"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.subjects()[0], "fix: ACME 9: x");
}

#[test]
fn header_format_decides_how_the_ticket_is_written() {
    let repo = repo_with("version = 1\n[commit.ticket]\nheader_format = \"[{ticket}] \"\n");
    let out = repo.cw(&["commit", "-t", "fix", "-m", "x", "--ticket", "#5"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.subjects()[0], "fix: [#5] x");
}

#[test]
fn a_header_format_without_the_placeholder_is_an_error() {
    let repo = repo_with("version = 1\n[commit.ticket]\nheader_format = \"no placeholder\"\n");
    let out = repo.cw(&["commit", "-t", "fix", "-m", "x", "--ticket", "A-1"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("{ticket}"), "{}", text(&out));
}

#[test]
fn a_ticket_that_does_not_match_the_pattern_is_refused() {
    let repo = repo_with(JIRA_STYLE);
    let before = repo.subjects_or_empty();
    let out = repo.cw(&["commit", "-t", "fix", "-m", "x", "--ticket", "nope"]);
    assert!(!out.status.success());
    assert!(
        text(&out).contains("commit.ticket.pattern"),
        "{}",
        text(&out)
    );
    assert_eq!(repo.subjects_or_empty(), before);
}

#[test]
fn a_required_ticket_missing_without_a_terminal_names_the_flag() {
    let repo = repo_with("version = 1\n[commit.ticket]\nrequired = true\n");
    let out = repo.cw(&["commit", "-t", "fix", "-m", "x"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("--ticket"), "{}", text(&out));
}

#[test]
fn an_optional_ticket_may_be_left_out() {
    let repo = repo_with(JIRA_STYLE);
    let out = repo.cw(&["commit", "-t", "fix", "-m", "x"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.subjects()[0], "fix: x");
}

#[test]
fn the_branch_name_supplies_the_ticket() {
    let repo = repo_with(
        "version = 1\n[commit.ticket]\nrequired = true\nsource = \"branch\"\npattern = \"[A-Z]+-[0-9]+\"\n",
    );
    repo.git(&["checkout", "-q", "-b", "feature/PROJ-77-login"]);
    let out = repo.cw(&["commit", "-t", "feat", "-m", "login"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.subjects()[0], "feat: PROJ-77: login");
}

#[test]
fn a_branch_without_a_ticket_fails_when_one_is_required() {
    let repo = repo_with(
        "version = 1\n[commit.ticket]\nrequired = true\nsource = \"branch\"\npattern = \"[A-Z]+-[0-9]+\"\n",
    );
    let out = repo.cw(&["commit", "-t", "feat", "-m", "login"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("--ticket"), "{}", text(&out));
}

#[test]
fn the_flag_overrides_the_branch() {
    let repo = repo_with(
        "version = 1\n[commit.ticket]\nsource = \"branch\"\npattern = \"[A-Z]+-[0-9]+\"\n",
    );
    repo.git(&["checkout", "-q", "-b", "feature/PROJ-1-x"]);
    let out = repo.cw(&["commit", "-t", "fix", "-m", "x", "--ticket", "OTHER-2"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.subjects()[0], "fix: OTHER-2: x");
}

#[test]
fn the_ticket_counts_towards_the_subject_length() {
    let repo = repo_with("version = 1\n[commit]\nsubject_max_length = 20\n");
    let out = repo.cw(&[
        "commit",
        "-t",
        "fix",
        "-m",
        "twelve chars",
        "--ticket",
        "LONG-12345",
    ]);
    assert!(!out.status.success());
    assert!(text(&out).contains("max length"), "{}", text(&out));
}

#[test]
fn dry_run_shows_the_ticket_without_committing() {
    let repo = repo_with(JIRA_STYLE);
    let out = repo.cw(&[
        "commit",
        "--dry-run",
        "-t",
        "fix",
        "-m",
        "x",
        "--ticket",
        "PROJ-3",
    ]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("fix: PROJ-3: x"), "{}", text(&out));
    assert!(repo.subjects_or_empty().is_empty());
}

#[test]
fn ticket_source_can_be_set_with_cw_config() {
    let repo = Repo::bare();
    repo.write(".cwizard.toml", "version = 1\n");
    let out = repo.cw(&["config", "set", "commit.ticket.source", "branch_or_prompt"]);
    assert!(out.status.success(), "{}", text(&out));
    let bad = repo.cw(&["config", "set", "commit.ticket.source", "carrier-pigeon"]);
    assert!(!bad.status.success());
}

// --- -a and -A -----------------------------------------------------------------

fn tracked_and_untracked() -> Repo {
    let repo = Repo::bare();
    repo.write("tracked.txt", "one");
    repo.git(&["add", "-A"]);
    repo.commit("chore: start");
    repo.write("tracked.txt", "two");
    repo.write("untracked.txt", "new");
    repo
}

#[test]
fn commit_dash_a_stages_tracked_changes_only() {
    let repo = tracked_and_untracked();
    let out = repo.cw(&["commit", "-a", "-t", "fix", "-m", "tracked"]);
    assert!(out.status.success(), "{}", text(&out));
    let shown = repo.git(&["show", "--stat", "--format=", "HEAD"]);
    assert!(shown.contains("tracked.txt"), "{shown}");
    assert!(!shown.contains("untracked.txt"), "{shown}");
    assert!(
        repo.git(&["status", "--short"])
            .contains("?? untracked.txt")
    );
}

#[test]
fn commit_dash_a_includes_deletions_of_tracked_files() {
    let repo = tracked_and_untracked();
    std::fs::remove_file(repo.path().join("tracked.txt")).unwrap();
    let out = repo.cw(&["commit", "-a", "-t", "fix", "-m", "remove"]);
    assert!(out.status.success(), "{}", text(&out));
    let shown = repo.git(&["show", "--stat", "--format=", "HEAD"]);
    assert!(shown.contains("tracked.txt"), "{shown}");
}

#[test]
fn commit_capital_a_stages_everything() {
    let repo = tracked_and_untracked();
    let out = repo.cw(&["commit", "-A", "-t", "fix", "-m", "all"]);
    assert!(out.status.success(), "{}", text(&out));
    let shown = repo.git(&["show", "--stat", "--format=", "HEAD"]);
    assert!(
        shown.contains("tracked.txt") && shown.contains("untracked.txt"),
        "{shown}"
    );
}

#[test]
fn the_two_staging_flags_cannot_be_combined() {
    let repo = tracked_and_untracked();
    let out = repo.cw(&["commit", "-a", "-A", "-t", "fix", "-m", "x"]);
    assert!(!out.status.success());
}

#[test]
fn a_dry_run_does_not_stage_anything() {
    let repo = tracked_and_untracked();
    let out = repo.cw(&["commit", "-A", "--dry-run", "-t", "fix", "-m", "x"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(
        repo.git(&["diff", "--cached", "--name-only"])
            .trim()
            .is_empty()
    );
}

#[test]
fn add_dash_a_stages_tracked_changes_only() {
    let repo = tracked_and_untracked();
    let out = repo.cw(&["add", "-a"]);
    assert!(out.status.success(), "{}", text(&out));
    let staged = repo.git(&["diff", "--cached", "--name-only"]);
    assert_eq!(staged.trim(), "tracked.txt");
}

#[test]
fn add_dash_a_cannot_be_combined_with_paths_all_or_unstage() {
    let repo = tracked_and_untracked();
    assert!(!repo.cw(&["add", "-a", "--all"]).status.success());
    assert!(!repo.cw(&["add", "-a", "--unstage"]).status.success());
    assert!(!repo.cw(&["add", "-a", "tracked.txt"]).status.success());
}
