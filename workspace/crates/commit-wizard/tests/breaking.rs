//! `commit.breaking.require_footer` decides whether a BREAKING CHANGE
//! description is mandatory (on) or optional (off, written only if given).

mod common;

use common::{Repo, text};

fn repo(require_footer: bool) -> Repo {
    let repo = Repo::new();
    let path = repo.path().join(".cwizard.toml");
    let config = std::fs::read_to_string(&path).unwrap().replace(
        "require_footer = false",
        &format!("require_footer = {require_footer}"),
    );
    std::fs::write(&path, config).unwrap();
    repo.write("x.txt", "x");
    repo.git(&["add", "x.txt"]);
    repo
}

fn last_message(repo: &Repo) -> String {
    repo.git(&["log", "-1", "--format=%B"])
}

#[test]
fn optional_footer_is_written_when_given() {
    let repo = repo(false);
    let out = repo.cw(&[
        "commit",
        "-t",
        "feat",
        "-m",
        "drop x",
        "--breaking",
        "-d",
        "x is gone",
    ]);
    assert!(out.status.success(), "{}", text(&out));
    let message = last_message(&repo);
    assert!(message.starts_with("feat!: drop x"), "{message}");
    assert!(message.contains("BREAKING CHANGE: x is gone"), "{message}");
}

#[test]
fn optional_footer_may_be_omitted() {
    let repo = repo(false);
    let out = repo.cw(&["commit", "-t", "feat", "-m", "drop x", "--breaking"]);
    assert!(out.status.success(), "{}", text(&out));
    let message = last_message(&repo);
    assert!(message.starts_with("feat!: drop x"), "{message}");
    assert!(!message.contains("BREAKING CHANGE"), "{message}");
}

#[test]
fn required_footer_is_written_when_given() {
    let repo = repo(true);
    let out = repo.cw(&[
        "commit",
        "-t",
        "feat",
        "-m",
        "drop x",
        "--breaking",
        "-d",
        "x is gone",
    ]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(last_message(&repo).contains("BREAKING CHANGE: x is gone"));
}

#[test]
fn required_footer_cannot_be_omitted() {
    let repo = repo(true);
    let before = repo.subjects();
    let out = repo.cw(&["commit", "-t", "feat", "-m", "drop x", "--breaking"]);
    assert!(!out.status.success(), "{}", text(&out));
    assert_eq!(repo.subjects(), before);
}

#[cfg(unix)]
mod prompts {
    use super::*;

    fn answer_breaking(repo: &Repo, description: &str, continues: bool) -> bool {
        let mut cw = repo.cw_pty(&["commit"]);
        cw.expect("What type of change");
        cw.line("feat");
        cw.expect("Add a scope?");
        cw.line("n");
        cw.expect("Write a short summary");
        cw.line("drop x");
        cw.expect("breaking changes?");
        cw.line("y");
        cw.expect("Describe the breaking change");
        cw.line(description);
        if !continues {
            cw.expect("breaking change description cannot be empty");
            return cw.finish();
        }
        cw.expect("longer description?");
        cw.line("n");
        cw.expect("footer lines?");
        cw.line("n");
        cw.expect("Create this commit?");
        cw.line("y");
        cw.finish()
    }

    #[test]
    fn empty_description_is_accepted_when_optional() {
        let repo = repo(false);
        assert!(answer_breaking(&repo, "", true));
        let message = last_message(&repo);
        assert!(message.starts_with("feat!: drop x"), "{message}");
        assert!(!message.contains("BREAKING CHANGE"), "{message}");
    }

    #[test]
    fn empty_description_is_rejected_when_required() {
        let repo = repo(true);
        let before = repo.subjects();
        assert!(!answer_breaking(&repo, "", false));
        assert_eq!(repo.subjects(), before);
    }
}
