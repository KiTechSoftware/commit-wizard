//! `cw changelog` with no options on a terminal asks what to generate.
#![cfg(unix)]

mod common;

use common::Repo;

fn history() -> Repo {
    let repo = Repo::new();
    repo.commit("feat: first feature");
    repo.git(&["tag", "v0.1.0"]);
    repo.commit("fix(core): a fix");
    repo.git(&["tag", "v0.2.0"]);
    repo.commit("feat(cli): colored output");
    repo
}

fn changelog(repo: &Repo) -> String {
    std::fs::read_to_string(repo.path().join("CHANGELOG.md")).expect("CHANGELOG.md was written")
}

fn headings(markdown: &str) -> Vec<&str> {
    markdown.lines().filter(|l| l.starts_with("## ")).collect()
}

#[test]
fn since_the_last_tag_with_a_version_label() {
    let repo = history();
    let mut cw = repo.cw_pty(&["changelog"]);

    cw.expect("What would you like to generate?");
    cw.line("Changes since");
    cw.expect("Version label");
    cw.line("0.3.0");
    cw.expect("Group entries by scope?");
    cw.line("n");
    cw.expect("not Conventional Commits?");
    cw.line("n");

    assert!(cw.finish());
    let file = changelog(&repo);
    assert_eq!(headings(&file).len(), 1, "{file}");
    assert!(headings(&file)[0].starts_with("## v0.3.0"), "{file}");
    assert!(
        file.contains("colored output") && !file.contains("a fix"),
        "{file}"
    );
}

#[test]
fn an_empty_label_means_unreleased() {
    let repo = history();
    let mut cw = repo.cw_pty(&["changelog"]);

    cw.expect("What would you like to generate?");
    cw.line("Changes since");
    cw.expect("Version label");
    cw.line("");
    cw.expect("Group entries by scope?");
    cw.line("n");
    cw.expect("not Conventional Commits?");
    cw.line("n");

    assert!(cw.finish());
    assert_eq!(headings(&changelog(&repo)), ["## Unreleased"]);
}

#[test]
fn full_history_skips_the_version_label() {
    let repo = history();
    let mut cw = repo.cw_pty(&["changelog"]);

    cw.expect("What would you like to generate?");
    cw.line("Full history");
    cw.expect("Group entries by scope?");
    cw.line("y");
    cw.expect("not Conventional Commits?");
    cw.line("n");

    assert!(cw.finish());
    let file = changelog(&repo);
    assert_eq!(headings(&file).len(), 3, "{file}");
    assert!(
        file.contains("#### cli") && file.contains("#### core"),
        "grouped by scope: {file}"
    );
}

#[test]
fn a_custom_range_asks_for_both_ends() {
    let repo = history();
    let mut cw = repo.cw_pty(&["changelog"]);

    cw.expect("What would you like to generate?");
    cw.line("Custom range");
    cw.expect("From (tag or commit)");
    cw.line("v0.1.0");
    cw.expect("To (tag or commit)");
    cw.line("v0.2.0");
    cw.expect("Version label");
    cw.line("");
    cw.expect("Group entries by scope?");
    cw.line("n");
    cw.expect("not Conventional Commits?");
    cw.line("n");

    assert!(cw.finish());
    let file = changelog(&repo);
    assert_eq!(headings(&file).len(), 1, "{file}");
    assert!(
        headings(&file)[0].starts_with("## v0.1.0..v0.2.0"),
        "{file}"
    );
    assert!(
        file.contains("a fix") && !file.contains("first feature"),
        "{file}"
    );
}

#[test]
fn a_custom_range_needs_a_start() {
    let repo = history();
    let mut cw = repo.cw_pty(&["changelog"]);

    cw.expect("What would you like to generate?");
    cw.line("Custom range");
    cw.expect("From (tag or commit)");
    cw.line("");

    assert!(!cw.finish());
    assert!(!repo.path().join("CHANGELOG.md").exists());
}

#[test]
fn any_flag_skips_the_questions() {
    let repo = history();
    // If a prompt appeared the process would wait for input and time out.
    let cw = repo.cw_pty(&["changelog", "--all"]);

    assert!(cw.finish());
    assert_eq!(headings(&changelog(&repo)).len(), 3);
}
