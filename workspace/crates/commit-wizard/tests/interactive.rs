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

#[test]
fn commit_asks_for_a_required_ticket_and_writes_it_into_the_subject() {
    let repo = Repo::bare();
    repo.write(
        ".cwizard.toml",
        "version = 1\n[commit.ticket]\nrequired = true\nsource = \"prompt\"\npattern = \"Ticket-[0-9]+\"\n",
    );
    repo.write("a.txt", "a");
    repo.git(&["add", "a.txt"]);
    let mut cw = repo.cw_pty(&["commit"]);

    cw.expect("What type of change");
    cw.line("feat");
    cw.expect("Add a scope?");
    cw.line("n");
    cw.expect("Ticket");
    cw.line("not a ticket");
    cw.expect("does not match");
    cw.line("Ticket-012");
    cw.expect("Write a short summary");
    cw.line("add ticketing");
    skip_optional_prompts(&mut cw);
    cw.expect("Create this commit?");
    cw.line("y");

    assert!(cw.finish());
    assert_eq!(repo.subjects()[0], "feat: Ticket-012: add ticketing");
}

// --- cw branch ------------------------------------------------------------------

fn repo_with_branches() -> Repo {
    let repo = staged_repo();
    repo.git(&["commit", "-q", "-m", "chore: base"]);
    repo.git(&["push", "-q", "-u", "origin", "main"]);
    for name in ["alpha", "beta"] {
        repo.git(&["switch", "-q", "-c", &format!("feature/{name}")]);
        repo.commit(&format!("feat: {name}"));
        repo.git(&["push", "-q", "-u", "origin", &format!("feature/{name}")]);
    }
    repo.git(&["switch", "-q", "main"]);
    repo
}

#[test]
fn branch_offers_a_list_to_switch_to() {
    let repo = repo_with_branches();
    let mut cw = repo.cw_pty(&["branch"]);
    cw.expect("Switch to which branch?");
    cw.line("feature/beta");
    assert!(cw.finish());
    assert_eq!(
        repo.git(&["branch", "--show-current"]).trim(),
        "feature/beta"
    );
}

#[test]
fn branch_new_asks_for_the_kind_and_the_description() {
    let repo = repo_with_branches();
    let mut cw = repo.cw_pty(&["branch", "new"]);
    cw.expect("What kind of branch is it?");
    cw.line("bugfix");
    cw.expect("What are you working on?");
    cw.line("Fix the crash");
    cw.expect("Create branch bugfix/fix-the-crash?");
    cw.line("y");
    assert!(cw.finish());
    assert_eq!(
        repo.git(&["branch", "--show-current"]).trim(),
        "bugfix/fix-the-crash"
    );
}

#[test]
fn branch_prune_asks_before_deleting_and_can_be_declined() {
    let repo = repo_with_branches();
    repo.git(&[
        "merge",
        "-q",
        "--no-ff",
        "feature/alpha",
        "-m",
        "Merge alpha",
    ]);
    repo.git(&["push", "-q", "origin", "main"]);
    repo.git(&["push", "-q", "origin", "--delete", "feature/alpha"]);

    let mut declined = repo.cw_pty(&["branch", "prune"]);
    declined.expect("Delete 1 branch?");
    declined.line("n");
    assert!(declined.finish());
    assert!(
        repo.git(&["branch", "--list", "feature/alpha"])
            .contains("feature/alpha")
    );

    let mut accepted = repo.cw_pty(&["branch", "prune"]);
    accepted.expect("Delete 1 branch?");
    accepted.line("y");
    assert!(accepted.finish());
    assert!(
        !repo
            .git(&["branch", "--list", "feature/alpha"])
            .contains("feature/alpha")
    );
}

#[test]
fn branch_new_only_asks_about_the_parts_the_template_has() {
    let repo = repo_with_branches();
    repo.write(
        ".cwizard.toml",
        "version = 1\n[branch.naming]\npattern = \"{slug}\"\n",
    );
    let mut cw = repo.cw_pty(&["branch", "new"]);
    cw.expect("What are you working on?");
    cw.line("Plain name");
    cw.expect("Create branch plain-name?");
    cw.line("y");
    assert!(cw.finish());
    assert_eq!(repo.git(&["branch", "--show-current"]).trim(), "plain-name");
}

#[test]
fn branch_new_keeps_asking_for_what_the_config_requires() {
    let repo = repo_with_branches();
    repo.write(
        ".cwizard.toml",
        "version = 1\n[branch.naming]\nrequire_type = true\nrequire_ticket = true\n",
    );
    let mut cw = repo.cw_pty(&["branch", "new"]);
    cw.expect("What kind of branch is it?");
    cw.line("feature");
    cw.expect("Ticket");
    cw.line("");
    cw.expect("A ticket is required");
    cw.line("PROJ-4");
    cw.expect("What are you working on?");
    cw.line("");
    cw.expect("A description is required");
    cw.line("Add login");
    cw.expect("Create branch feature/PROJ-4-add-login?");
    cw.line("y");
    assert!(cw.finish());
    assert_eq!(
        repo.git(&["branch", "--show-current"]).trim(),
        "feature/PROJ-4-add-login"
    );
}
