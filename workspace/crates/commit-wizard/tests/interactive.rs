//! Tests that drive `cw` through a pseudo-terminal, answering real prompts.
#![cfg(unix)]

mod common;

use common::Repo;

fn staged_repo() -> Repo {
    let repo = Repo::new();
    repo.write("a.txt", "a");
    repo.git(&["add", "a.txt"]);
    repo
}

#[test]
fn commit_walks_through_every_prompt() {
    let repo = staged_repo();
    let mut cw = repo.cw_pty(&["commit"]);

    cw.expect("What type of change");
    cw.line("feat");
    cw.expect("Add a scope?");
    cw.line("y");
    cw.expect("Scope");
    cw.line("core");
    cw.expect("Write a short summary");
    cw.line("add parser");
    cw.expect("breaking changes?");
    cw.line("n");
    cw.expect("longer description?");
    cw.line("n");
    cw.expect("footer lines?");
    cw.line("n");
    cw.expect("Create this commit?");
    cw.line("y");

    assert!(cw.finish());
    assert_eq!(repo.subjects()[0], "feat(core): add parser");
}

/// Answer the prompts after the type and scope questions with "no" and accept.
fn skip_optional_prompts(cw: &mut common::Pty) {
    cw.expect("breaking changes?");
    cw.line("n");
    cw.expect("longer description?");
    cw.line("n");
    cw.expect("footer lines?");
    cw.line("n");
}

#[test]
fn commit_can_be_cancelled_at_the_final_prompt() {
    let repo = staged_repo();
    let before = repo.subjects();
    let mut cw = repo.cw_pty(&["commit"]);

    cw.expect("What type of change");
    cw.line("fix");
    cw.expect("Add a scope?");
    cw.line("n");
    cw.expect("Write a short summary");
    cw.line("never mind");
    skip_optional_prompts(&mut cw);
    cw.expect("Create this commit?");
    cw.line("n");
    cw.expect("Commit cancelled");

    cw.finish();
    assert_eq!(repo.subjects(), before);
}

#[test]
fn commit_reprompts_for_an_empty_summary() {
    let repo = staged_repo();
    let mut cw = repo.cw_pty(&["commit"]);

    cw.expect("What type of change");
    cw.line("fix");
    cw.expect("Add a scope?");
    cw.line("n");
    cw.expect("Write a short summary");
    cw.line("");
    cw.expect("Summary cannot be empty");
    cw.expect("Write a short summary");
    cw.line("handle empty input");
    skip_optional_prompts(&mut cw);
    cw.expect("Create this commit?");
    cw.line("y");

    assert!(cw.finish());
    assert_eq!(repo.subjects()[0], "fix: handle empty input");
}

#[test]
fn commit_records_breaking_change_body_and_footer() {
    let repo = staged_repo();
    let mut cw = repo.cw_pty(&["commit"]);

    cw.expect("What type of change");
    cw.line("feat");
    cw.expect("Add a scope?");
    cw.line("n");
    cw.expect("Write a short summary");
    cw.line("replace the parser");
    cw.expect("breaking changes?");
    cw.line("y");
    cw.expect("Describe the breaking change");
    cw.line("old grammar removed");
    cw.expect("longer description?");
    cw.line("y");
    cw.expect("Body");
    cw.line("Switches to the new grammar.");
    cw.expect("footer lines?");
    cw.line("n");
    cw.expect("Create this commit?");
    cw.line("y");

    assert!(cw.finish());
    let message = repo.git(&["log", "-1", "--format=%B"]);
    assert!(message.starts_with("feat"), "{message}");
    assert!(message.contains("replace the parser"), "{message}");
    assert!(
        message.contains("Switches to the new grammar."),
        "{message}"
    );
    assert!(
        message.contains("BREAKING CHANGE: old grammar removed"),
        "{message}"
    );
}

#[test]
fn add_stages_only_the_selected_files() {
    let repo = Repo::new();
    repo.write("a.txt", "a");
    repo.write("b.txt", "b");
    let mut cw = repo.cw_pty(&["add"]);

    cw.expect("Select files to stage");
    cw.send(" "); // toggle the first entry
    cw.line("");
    cw.expect("preview diffs");
    cw.line("n");

    assert!(cw.finish());
    let staged = repo.git(&["diff", "--cached", "--name-only"]);
    assert_eq!(staged, "a.txt");
}

#[test]
fn add_unstage_lists_only_staged_files() {
    let repo = Repo::new();
    repo.write("a.txt", "a");
    repo.write("b.txt", "b");
    repo.git(&["add", "a.txt"]);
    let mut cw = repo.cw_pty(&["add", "--unstage"]);

    cw.expect("a.txt");
    cw.send(" ");
    cw.line("");

    assert!(cw.finish());
    assert_eq!(repo.git(&["diff", "--cached", "--name-only"]), "");
}
