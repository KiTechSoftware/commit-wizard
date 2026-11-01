//! `cw check` must fail the build when commits break the rules.

mod common;

use common::{Repo, text};

fn history(messages: &[&str]) -> Repo {
    let repo = Repo::new();
    for message in messages {
        repo.write(
            &format!("{}.txt", message.len() + repo.subjects().len()),
            message,
        );
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", message]);
    }
    repo
}

#[test]
fn a_clean_history_exits_zero() {
    let repo = history(&["feat: one", "fix: two"]);
    let out = repo.cw(&["check"]);
    assert!(out.status.success(), "{}", text(&out));
}

#[test]
fn an_invalid_commit_makes_the_command_fail() {
    let repo = history(&["feat: one", "not conventional"]);
    let out = repo.cw(&["check"]);

    assert!(!out.status.success(), "{}", text(&out));
    let output = text(&out);
    // The report still explains what is wrong.
    assert!(output.contains("not conventional"), "{output}");
    assert!(output.contains("1 of 3 commit(s) are invalid"), "{output}");
}

#[test]
fn it_fails_the_same_way_in_ci_and_non_interactive_modes() {
    let repo = history(&["wip"]);
    for flag in ["--ci", "--non-interactive"] {
        assert!(!repo.cw(&[flag, "check"]).status.success(), "{flag}");
    }
}

#[test]
fn json_output_is_still_printed_when_it_fails() {
    let repo = history(&["wip"]);
    let out = repo.cw(&["--json", "check"]);

    assert!(!out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("report is not JSON ({e}): {stdout}"));
    assert!(json.to_string().contains("invalid_count"), "{json}");
}

#[test]
fn checking_only_recent_valid_commits_passes() {
    let repo = history(&["wip", "feat: good"]);
    // The bad commit is older than the newest one, which is all that is checked.
    let out = repo.cw(&["check", "--tail", "1"]);
    assert!(out.status.success(), "{}", text(&out));
}

#[test]
fn an_empty_range_is_not_a_failure() {
    let repo = history(&["feat: one"]);
    let head = repo.git(&["rev-parse", "HEAD"]);
    let out = repo.cw(&["check", "--from", &head]);
    assert!(out.status.success(), "{}", text(&out));
}
