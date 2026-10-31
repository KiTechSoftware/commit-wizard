//! End-to-end tests for `cw bump` and `cw tag` against throwaway git repos.

use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

struct Repo {
    dir: TempDir,
    home: TempDir,
}

impl Repo {
    fn new() -> Self {
        let repo = Self {
            dir: TempDir::new().unwrap(),
            home: TempDir::new().unwrap(),
        };
        repo.git(&["init", "-q", "-b", "main"]);
        repo.git(&["config", "user.email", "test@example.com"]);
        repo.git(&["config", "user.name", "test"]);
        repo.git(&["config", "commit.gpgsign", "false"]);
        repo.git(&["config", "tag.gpgsign", "false"]);
        // A generated config keeps the tests offline (no registry fetch).
        let init = repo.cw(&["init"]);
        assert!(init.status.success(), "init failed: {}", text(&init));
        repo.commit("chore: init");
        repo
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn git(&self, args: &[&str]) -> String {
        let out = Command::new("git")
            .args(args)
            .current_dir(self.path())
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {}", text(&out));
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn commit(&self, message: &str) {
        self.git(&["commit", "-q", "--allow-empty", "-m", message]);
    }

    fn commit_with_footer(&self, message: &str, footer: &str) {
        self.git(&["commit", "-q", "--allow-empty", "-m", message, "-m", footer]);
    }

    fn cw(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_cw"))
            .args(args)
            .current_dir(self.path())
            .env("HOME", self.home.path())
            .env("XDG_CONFIG_HOME", self.home.path())
            .env("NO_COLOR", "1")
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap()
    }

    /// Run `cw bump --dry-run` and return the `next_tag` it reports.
    fn next_tag(&self) -> String {
        let out = self.cw(&["bump", "--dry-run"]);
        assert!(out.status.success(), "bump failed: {}", text(&out));
        field(&text(&out), "next_tag")
    }

    fn tag(&self, args: &[&str]) -> Output {
        let mut full = vec!["tag", "--yes"];
        full.extend_from_slice(args);
        self.cw(&full)
    }

    fn tags(&self) -> Vec<String> {
        self.git(&["tag", "--list"])
            .lines()
            .map(str::to_string)
            .collect()
    }
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn field(output: &str, key: &str) -> String {
    output
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key}: ")))
        .unwrap_or_else(|| panic!("no `{key}` in output:\n{output}"))
        .trim()
        .to_string()
}

fn released(repo: &Repo, version: &str) {
    let out = repo.tag(&["--set-version", version]);
    assert!(out.status.success(), "tag {version}: {}", text(&out));
}

#[test]
fn bump_levels() {
    let repo = Repo::new();
    released(&repo, "1.2.3");

    repo.commit("fix: patch change");
    assert_eq!(repo.next_tag(), "v1.2.4");

    repo.commit("feat: minor change");
    assert_eq!(repo.next_tag(), "v1.3.0");

    repo.commit("feat!: major change");
    assert_eq!(repo.next_tag(), "v2.0.0");
}

#[test]
fn breaking_change_footer_bumps_major() {
    let repo = Repo::new();
    released(&repo, "1.2.3");
    repo.commit_with_footer("feat: drop x", "BREAKING CHANGE: x is gone");
    assert_eq!(repo.next_tag(), "v2.0.0");
}

#[test]
fn tag_creates_the_computed_version() {
    let repo = Repo::new();
    released(&repo, "0.1.0");
    repo.commit("feat: add thing");

    let out = repo.tag(&[]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(repo.tags().contains(&"v0.2.0".to_string()));
}

#[test]
fn dry_run_tag_creates_nothing() {
    let repo = Repo::new();
    released(&repo, "0.1.0");
    repo.commit("fix: small");

    let out = repo.tag(&["--dry-run"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("Would create tag: v0.1.1"));
    assert_eq!(repo.tags(), vec!["v0.1.0".to_string()]);
}

#[test]
fn bump_uses_highest_tag_when_tags_share_a_commit() {
    let repo = Repo::new();
    // Created back to back, so they share a timestamp and a commit.
    released(&repo, "0.1.0");
    released(&repo, "1.0.0");
    released(&repo, "1.1.0");
    repo.commit("fix: after tags");
    assert_eq!(repo.next_tag(), "v1.1.1");
}

#[test]
fn bump_ignores_tags_from_other_branches() {
    let repo = Repo::new();
    released(&repo, "1.0.0");

    repo.git(&["checkout", "-q", "-b", "other"]);
    repo.commit("feat: elsewhere");
    released(&repo, "9.0.0");

    repo.git(&["checkout", "-q", "main"]);
    repo.commit("fix: on main");
    assert_eq!(repo.next_tag(), "v1.0.1");
}

#[test]
fn suffix_creates_numbered_release_candidates() {
    let repo = Repo::new();
    released(&repo, "1.0.0");
    repo.commit("feat: next");

    for expected in ["v1.1.0-rc.1", "v1.1.0-rc.2"] {
        let out = repo.tag(&["--suffix", "rc"]);
        assert!(out.status.success(), "{}", text(&out));
        assert!(
            repo.tags().contains(&expected.to_string()),
            "{:?}",
            repo.tags()
        );
        repo.commit("fix: rc feedback");
    }
}

#[test]
fn set_version_accepts_prereleases_and_rejects_malformed_ones() {
    let repo = Repo::new();
    assert!(repo.tag(&["--set-version", "2.0.0-rc.1"]).status.success());
    assert!(repo.tags().contains(&"v2.0.0-rc.1".to_string()));

    let bad = repo.tag(&["--set-version", "2.0.0rc.1"]);
    assert!(!bad.status.success());
    assert!(!repo.tags().contains(&"v2.0.0rc.1".to_string()));
}

#[test]
fn prerelease_is_promoted_by_bump_and_tag() {
    let repo = Repo::new();
    released(&repo, "1.0.0");
    repo.commit("feat: next");
    assert!(repo.tag(&["--suffix", "rc"]).status.success());

    // No new commits: the rc is promoted to its release.
    assert_eq!(repo.next_tag(), "v1.1.0");
    let out = repo.tag(&[]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(repo.tags().contains(&"v1.1.0".to_string()));

    // The next cycle starts from the final release.
    repo.commit("fix: later");
    assert_eq!(repo.next_tag(), "v1.1.1");
}
