//! When cw asks questions: on a terminal by default, never in scripts, and
//! with defaults only when asked (`--yes`, `--ci`).

mod common;

use common::{Repo, text};

fn staged() -> Repo {
    let repo = Repo::new();
    repo.write("x.txt", "x");
    repo.git(&["add", "x.txt"]);
    repo
}

// --- no terminal (a script, a pipe) ---------------------------------------

#[test]
fn commit_without_a_terminal_asks_for_flags_instead_of_prompting() {
    let repo = staged();
    let before = repo.subjects();

    let out = repo.cw(&["commit"]);

    assert!(!out.status.success());
    assert!(
        text(&out).contains("type is required in non-interactive mode"),
        "{}",
        text(&out)
    );
    assert_eq!(repo.subjects(), before);
}

#[test]
fn add_without_a_terminal_needs_paths() {
    let repo = Repo::new();
    repo.write("y.txt", "y");

    let out = repo.cw(&["add"]);

    assert!(!out.status.success());
    assert!(
        text(&out).contains("Interactive input required"),
        "{}",
        text(&out)
    );
}

#[test]
fn flags_are_enough_without_a_terminal() {
    let repo = staged();
    let out = repo.cw(&["commit", "-t", "feat", "-m", "works in scripts"]);
    assert!(out.status.success(), "{}", text(&out));
}

#[test]
fn yes_does_not_make_up_missing_answers() {
    // --yes accepts defaults; the commit type has none, so it must be given.
    let repo = staged();
    let out = repo.cw(&["commit", "--yes"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("type is required"), "{}", text(&out));
}

// --- on a terminal --------------------------------------------------------

#[cfg(unix)]
mod on_a_terminal {
    use super::*;

    fn history() -> Repo {
        let repo = Repo::new();
        repo.commit("feat: first");
        repo.git(&["tag", "v0.1.0"]);
        repo.commit("fix: second");
        repo
    }

    #[test]
    fn non_interactive_never_prompts_even_with_a_terminal() {
        let repo = staged();
        let before = repo.subjects();
        let cw = repo.cw_pty(&["commit", "--non-interactive"]);

        // It must finish by itself (a prompt would wait forever) and fail.
        assert!(!cw.finish());
        assert_eq!(repo.subjects(), before);
    }

    #[test]
    fn yes_with_no_default_fails_instead_of_waiting() {
        let repo = staged();
        let cw = repo.cw_pty(&["commit", "--yes"]);
        assert!(!cw.finish());
    }

    #[test]
    fn yes_skips_the_changelog_menu() {
        let repo = history();
        let cw = repo.cw_pty(&["changelog", "--yes"]);

        assert!(cw.finish());
        let file = std::fs::read_to_string(repo.path().join("CHANGELOG.md")).unwrap();
        assert!(
            file.contains("## Unreleased") && file.contains("second"),
            "{file}"
        );
    }

    #[test]
    fn json_output_never_prompts() {
        let repo = history();
        let cw = repo.cw_pty(&["--json", "changelog"]);
        assert!(cw.finish());
        assert!(repo.path().join("CHANGELOG.md").exists());
    }

    #[test]
    fn ci_mode_runs_without_the_menu() {
        let repo = history();
        let cw = repo.cw_pty(&["--ci", "changelog"]);
        assert!(cw.finish());
        assert!(repo.path().join("CHANGELOG.md").exists());
    }

    #[test]
    fn a_plain_terminal_session_still_prompts() {
        let repo = staged();
        let mut cw = repo.cw_pty(&["commit"]);
        cw.expect("What type of change");
        cw.line("feat");
        cw.expect("Add a scope?");
        cw.line("n");
        cw.expect("Write a short summary");
        cw.line("guided as before");
        cw.expect("breaking changes?");
        cw.line("n");
        cw.expect("longer description?");
        cw.line("n");
        cw.expect("footer lines?");
        cw.line("n");
        cw.expect("Create this commit?");
        cw.line("y");

        assert!(cw.finish());
        assert_eq!(repo.subjects()[0], "feat: guided as before");
    }
}
