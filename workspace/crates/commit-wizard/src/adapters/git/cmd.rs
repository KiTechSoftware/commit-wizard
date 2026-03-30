use anyhow::{Context, Result, anyhow};
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

use crate::ports::git::{CommitOptions, GitPort};

/// Shells out to the `git` CLI to perform commits.
#[derive(Default)]
pub struct CmdGit;

impl CmdGit {
    fn run_status(args: &[&str]) -> Result<std::process::ExitStatus> {
        Command::new("git")
            .args(args)
            .status()
            .with_context(|| format!("failed to launch `git {}`", args.join(" ")))
    }

    fn run_capture(args: &[&str]) -> Result<std::process::Output> {
        Command::new("git")
            .args(args)
            .output()
            .with_context(|| format!("failed to launch `git {}`", args.join(" ")))
    }
}

impl GitPort for CmdGit {
    fn commit(&self, message: &str, opts: &CommitOptions) -> Result<()> {
        // Are we in a repo?
        if !Self::run_status(&["rev-parse", "--git-dir"])?.success() {
            return Err(anyhow!(
                "Not inside a git repository (run `git init` first)."
            ));
        }

        // Skip staged check only if allow_empty is true
        if !opts.allow_empty {
            // exit=0 => no staged changes; exit=1 => there are staged changes
            if Self::run_status(&["diff", "--cached", "--quiet"])?.success() {
                return Err(anyhow!(
                    "No staged changes. Stage files with `git add -A` (or pass --allow-empty)."
                ));
            }
        }

        // Write message to a temp file
        let mut tf = NamedTempFile::new().context("failed to create temp file")?;
        tf.write_all(message.as_bytes())
            .context("write commit message")?;
        let msg_path = tf.path().to_string_lossy().to_string();

        // Build git args
        let mut args = vec!["commit", "-F", &msg_path];
        if opts.allow_empty {
            args.push("--allow-empty");
        }

        let out = Self::run_capture(&args)?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(anyhow!("git commit failed: {}", stderr.trim()));
        }
        let stdout = String::from_utf8_lossy(&out.stdout);
        if !stdout.trim().is_empty() {
            println!("{stdout}");
        }
        Ok(())
    }
}
