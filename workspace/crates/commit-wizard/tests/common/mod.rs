//! Shared helpers: drive `cw` against throwaway git repos.
#![allow(dead_code)]

use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

pub struct Repo {
    dir: TempDir,
    home: TempDir,
    remote: TempDir,
    gnupg: Option<TempDir>,
}

impl Repo {
    pub fn new() -> Self {
        let repo = Self {
            dir: TempDir::new().unwrap(),
            home: TempDir::new().unwrap(),
            remote: TempDir::new().unwrap(),
            gnupg: None,
        };
        repo.git(&["init", "-q", "-b", "main"]);
        repo.git(&["config", "user.email", "test@example.com"]);
        repo.git(&["config", "user.name", "test"]);
        repo.git(&["config", "commit.gpgsign", "false"]);
        repo.git(&["config", "tag.gpgsign", "false"]);
        Command::new("git")
            .args(["init", "-q", "--bare"])
            .arg(repo.remote.path())
            .status()
            .unwrap();
        repo.git(&[
            "remote",
            "add",
            "origin",
            repo.remote.path().to_str().unwrap(),
        ]);
        // A generated config keeps the tests offline (no registry fetch).
        let init = repo.cw(&["init"]);
        assert!(init.status.success(), "init failed: {}", text(&init));
        repo.commit("chore: init");
        repo
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn git(&self, args: &[&str]) -> String {
        let out = Command::new("git")
            .args(args)
            .current_dir(self.path())
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {}", text(&out));
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    pub fn commit(&self, message: &str) {
        self.git(&["commit", "-q", "--allow-empty", "-m", message]);
    }

    pub fn commit_with_footer(&self, message: &str, footer: &str) {
        self.git(&["commit", "-q", "--allow-empty", "-m", message, "-m", footer]);
    }

    pub fn cw(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_cw"))
            .args(args)
            .current_dir(self.path())
            .env("HOME", self.home.path())
            .env("XDG_CONFIG_HOME", self.home.path())
            .env("NO_COLOR", "1")
            .envs(self.gnupg.iter().map(|g| ("GNUPGHOME", g.path())))
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap()
    }

    /// Run `cw bump --dry-run` and return the `next_tag` it reports.
    pub fn next_tag(&self) -> String {
        let out = self.cw(&["bump", "--dry-run"]);
        assert!(out.status.success(), "bump failed: {}", text(&out));
        field(&text(&out), "next_tag")
    }

    pub fn tag(&self, args: &[&str]) -> Output {
        let mut full = vec!["tag", "--yes"];
        full.extend_from_slice(args);
        self.cw(&full)
    }

    pub fn tags(&self) -> Vec<String> {
        self.git(&["tag", "--list"])
            .lines()
            .map(str::to_string)
            .collect()
    }
}

pub fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

pub fn field(output: &str, key: &str) -> String {
    output
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key}: ")))
        .unwrap_or_else(|| panic!("no `{key}` in output:\n{output}"))
        .trim()
        .to_string()
}

impl Repo {
    /// Branches present on the remote.
    pub fn remote_branches(&self) -> Vec<String> {
        self.remote_refs("refs/heads")
    }

    /// Tags present on the remote.
    pub fn remote_tags(&self) -> Vec<String> {
        self.remote_refs("refs/tags")
    }

    fn remote_refs(&self, prefix: &str) -> Vec<String> {
        let out = Command::new("git")
            .args(["for-each-ref", "--format=%(refname:short)", prefix])
            .current_dir(self.remote.path())
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::to_string)
            .collect()
    }

    /// Create a passphrase-less GPG key in a temporary keyring and make git
    /// sign tags with it. Returns false when gpg is unavailable.
    pub fn enable_signing(&mut self) -> bool {
        // Short path: gpg-agent sockets cannot live under long temp dirs.
        let Ok(gnupg) = tempfile::Builder::new().prefix("cwg").tempdir_in("/tmp") else {
            return false;
        };
        let made = Command::new("gpg")
            .env("GNUPGHOME", gnupg.path())
            .args([
                "--batch",
                "--pinentry-mode",
                "loopback",
                "--passphrase",
                "",
                "--quick-gen-key",
                "cw-test@example.com",
                "default",
                "default",
                "never",
            ])
            .output();
        if !matches!(made, Ok(ref out) if out.status.success()) {
            return false;
        }
        self.gnupg = Some(gnupg);
        self.git(&["config", "user.signingkey", "cw-test@example.com"]);
        true
    }

    /// Verify a tag's signature with the temporary keyring.
    pub fn tag_is_signed(&self, tag: &str) -> bool {
        Command::new("git")
            .args(["tag", "-v", tag])
            .current_dir(self.path())
            .envs(self.gnupg.iter().map(|g| ("GNUPGHOME", g.path())))
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }
}

pub fn released(repo: &Repo, version: &str) {
    let out = repo.tag(&["--set-version", version]);
    assert!(out.status.success(), "tag {version}: {}", text(&out));
}
