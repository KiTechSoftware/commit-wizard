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
    /// A git repo with one commit and a generated cw config.
    pub fn new() -> Self {
        let repo = Self::bare();
        // A generated config keeps the tests offline (no registry fetch).
        let init = repo.cw(&["init"]);
        assert!(init.status.success(), "init failed: {}", text(&init));
        repo.git(&["add", "-A"]);
        repo.commit("chore: init");
        repo
    }

    /// A git repo with a remote but no cw config and no commits.
    pub fn bare() -> Self {
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

/// An interactive `cw` process attached to a pseudo-terminal.
#[cfg(unix)]
pub struct Pty {
    session: expectrl::session::OsSession,
}

#[cfg(unix)]
impl Pty {
    /// Wait until `text` appears in the output.
    pub fn expect(&mut self, text: &str) {
        use expectrl::Expect;
        if let Err(err) = self.session.expect(text) {
            let mut buf = [0u8; 8192];
            let seen = self.session.try_read(&mut buf).unwrap_or(0);
            panic!(
                "waiting for {text:?}: {err}\nunmatched output: {:?}",
                String::from_utf8_lossy(&buf[..seen])
            );
        }
    }

    /// Type text without pressing Enter.
    pub fn send(&mut self, text: &str) {
        use expectrl::Expect;
        self.session.send(text).unwrap();
    }

    /// Type text and press Enter.
    pub fn line(&mut self, text: &str) {
        use expectrl::Expect;
        self.session.send_line(text).unwrap();
    }

    /// Wait (bounded) for the process to exit and report whether it succeeded.
    /// A process that is still running after the deadline is killed.
    pub fn finish(mut self) -> bool {
        use expectrl::{Expect, process::unix::WaitStatus};
        // Drain the pty so the process can finish writing and exit.
        let _ = self.session.expect(expectrl::Eof);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            match self.session.get_process().status() {
                Ok(WaitStatus::Exited(_, code)) => return code == 0,
                Ok(WaitStatus::StillAlive) if std::time::Instant::now() < deadline => {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                _ => return false,
            }
        }
    }
}

#[cfg(unix)]
impl Drop for Pty {
    fn drop(&mut self) {
        // Never leave a prompt-blocked `cw` behind when a test fails early.
        let _ = self.session.get_process_mut().exit(true);
    }
}

#[cfg(unix)]
impl Repo {
    /// Start `cw` attached to a pty so it behaves interactively.
    pub fn cw_pty(&self, args: &[&str]) -> Pty {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_cw"));
        cmd.args(args)
            .current_dir(self.path())
            .env("HOME", self.home.path())
            .env("XDG_CONFIG_HOME", self.home.path())
            .env("NO_COLOR", "1")
            .env("TERM", "xterm")
            .envs(self.gnupg.iter().map(|g| ("GNUPGHOME", g.path())));
        let mut session = expectrl::Session::spawn(cmd).unwrap();
        session.set_expect_timeout(Some(std::time::Duration::from_secs(10)));
        Pty { session }
    }

    /// Subject lines of all commits, newest first.
    pub fn subjects(&self) -> Vec<String> {
        self.git(&["log", "--format=%s"])
            .lines()
            .map(str::to_string)
            .collect()
    }

    /// Like `subjects`, but empty (not a failure) when there are no commits.
    pub fn subjects_or_empty(&self) -> Vec<String> {
        let out = Command::new("git")
            .args(["log", "--format=%s"])
            .current_dir(self.path())
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::to_string)
            .collect()
    }

    /// Write the global (per-user) config file for this repo's isolated HOME.
    pub fn write_global_config(&self, contents: &str) {
        let dir = self.home.path().join("cwizard");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("config.toml"), contents).unwrap();
    }

    /// Write a global file (`config.yaml`, `rules.json`, ...) for this repo's isolated HOME.
    pub fn write_global_file(&self, name: &str, contents: &str) {
        let dir = self.home.path().join("cwizard");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(name), contents).unwrap();
    }

    pub fn home_path(&self) -> &Path {
        self.home.path()
    }

    pub fn write(&self, name: &str, contents: &str) {
        std::fs::write(self.path().join(name), contents).unwrap();
    }
}
