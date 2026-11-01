//! Branch operations behind `cw branch`: listing with upstream status, switching,
//! creating, deleting, and refreshing what the remotes have.

use git2::{BranchType, Oid, Repository};

use super::Git;
use crate::engine::{
    Error, ErrorCode,
    models::git::{BranchInfo, BranchState},
};

impl Git {
    /// Local branches, most recently committed to first.
    pub fn branch_infos(&self) -> Result<Vec<BranchInfo>, Error> {
        let repo = self.repo()?;
        let head = repo.head().ok().and_then(|head| head.target());
        let config = repo.config().ok();
        let mut out = Vec::new();

        for entry in repo
            .branches(Some(BranchType::Local))
            .map_err(|err| self.git2_error("branches", err))?
        {
            let (branch, _) = entry.map_err(|err| self.git2_error("branch_iter", err))?;
            let Some(name) = branch
                .name()
                .map_err(|err| self.git2_error("branch.name", err))?
                .map(str::to_string)
            else {
                continue;
            };
            let Ok(tip) = branch.get().peel_to_commit() else {
                continue;
            };

            // What the branch follows, from its own config (so a remote branch that
            // has since been deleted is still known).
            let follows = config.as_ref().and_then(|config| {
                let remote = config.get_string(&format!("branch.{name}.remote")).ok()?;
                let merge = config.get_string(&format!("branch.{name}.merge")).ok()?;
                let short = merge.strip_prefix("refs/heads/")?.to_string();
                Some((remote, short))
            });
            let (upstream, state) = match follows {
                None => (None, BranchState::Local),
                Some((remote, short)) => {
                    let label = if remote == "." {
                        short.clone()
                    } else {
                        format!("{remote}/{short}")
                    };
                    let reference = if remote == "." {
                        format!("refs/heads/{short}")
                    } else {
                        format!("refs/remotes/{remote}/{short}")
                    };
                    let state = match repo
                        .find_reference(&reference)
                        .ok()
                        .and_then(|r| r.peel_to_commit().ok())
                    {
                        None => BranchState::Gone,
                        Some(upstream) => match repo.graph_ahead_behind(tip.id(), upstream.id()) {
                            Ok((0, 0)) => BranchState::UpToDate,
                            Ok((ahead, 0)) => BranchState::Ahead(ahead),
                            Ok((0, behind)) => BranchState::Behind(behind),
                            Ok((ahead, behind)) => BranchState::Diverged { ahead, behind },
                            Err(_) => BranchState::UpToDate,
                        },
                    };
                    (Some(label), state)
                }
            };

            out.push(describe(
                &repo,
                name,
                branch.is_head(),
                upstream,
                state,
                tip.id(),
                head,
            ));
        }

        out.sort_by(|a, b| b.updated.cmp(&a.updated).then(a.name.cmp(&b.name)));
        Ok(out)
    }

    /// Remote-tracking branches (`origin/main`), as of the last fetch.
    pub fn remote_branch_infos(&self) -> Result<Vec<BranchInfo>, Error> {
        let repo = self.repo()?;
        let head = repo.head().ok().and_then(|head| head.target());
        let mut out = Vec::new();

        for entry in repo
            .branches(Some(BranchType::Remote))
            .map_err(|err| self.git2_error("branches", err))?
        {
            let (branch, _) = entry.map_err(|err| self.git2_error("branch_iter", err))?;
            let Some(name) = branch
                .name()
                .map_err(|err| self.git2_error("branch.name", err))?
                .map(str::to_string)
            else {
                continue;
            };
            if name.ends_with("/HEAD") {
                continue;
            }
            let Ok(tip) = branch.get().peel_to_commit() else {
                continue;
            };
            out.push(describe(
                &repo,
                name,
                false,
                None,
                BranchState::Local,
                tip.id(),
                head,
            ));
        }

        out.sort_by(|a, b| b.updated.cmp(&a.updated).then(a.name.cmp(&b.name)));
        Ok(out)
    }

    /// Refreshes the remote-tracking branches and drops those deleted on the remote
    /// (`git fetch --prune`).
    pub fn fetch_prune(&self) -> Result<(), Error> {
        let output = self.run_dynamic(["fetch", "--prune"]).ok_or_else(|| {
            ErrorCode::ProcessFailure
                .error()
                .with_context("command", "git fetch --prune")
                .with_context("cwd", self.cwd.display().to_string())
        })?;
        self.ensure_success("git fetch --prune", output)
    }

    /// Deletes `branch` on `remote` (`git push <remote> --delete <branch>`).
    pub fn delete_remote_branch(&self, remote: &str, branch: &str) -> Result<(), Error> {
        let output = self
            .run_dynamic(["push", remote, "--delete", branch])
            .ok_or_else(|| {
                ErrorCode::ProcessFailure
                    .error()
                    .with_context("command", "git push <remote> --delete <branch>")
                    .with_context("cwd", self.cwd.display().to_string())
            })?;
        self.ensure_success("git push <remote> --delete <branch>", output)
    }

    /// Whether `name` is acceptable as a branch name (`git check-ref-format --branch`).
    pub fn is_valid_branch_name(&self, name: &str) -> bool {
        self.run_dynamic(["check-ref-format", "--branch", name])
            .is_some_and(|output| output.status.success())
    }

    pub fn branch_exists(&self, name: &str) -> bool {
        self.repo()
            .map(|repo| repo.find_branch(name, BranchType::Local).is_ok())
            .unwrap_or(false)
    }

    /// Checks out `name`; a branch that exists only on one remote is created to follow it.
    pub fn switch_branch(&self, name: &str) -> Result<(), Error> {
        let output = self.run_dynamic(["switch", name]).ok_or_else(|| {
            ErrorCode::ProcessFailure
                .error()
                .with_context("command", "git switch")
                .with_context("cwd", self.cwd.display().to_string())
        })?;
        self.ensure_success("git switch", output)
    }

    /// Creates `name` at `from` (or the current commit) and checks it out.
    pub fn create_and_switch_branch(&self, name: &str, from: Option<&str>) -> Result<(), Error> {
        let output = match from {
            Some(from) => self.run_dynamic(["switch", "-c", name, from]),
            None => self.run_dynamic(["switch", "-c", name]),
        }
        .ok_or_else(|| {
            ErrorCode::ProcessFailure
                .error()
                .with_context("command", "git switch -c")
                .with_context("cwd", self.cwd.display().to_string())
        })?;
        self.ensure_success("git switch -c", output)
    }
}

fn describe(
    repo: &Repository,
    name: String,
    current: bool,
    upstream: Option<String>,
    state: BranchState,
    tip: Oid,
    head: Option<Oid>,
) -> BranchInfo {
    let commit = repo.find_commit(tip).ok();
    let merged = head
        .is_some_and(|head| head == tip || repo.graph_descendant_of(head, tip).unwrap_or(false));
    BranchInfo {
        name,
        current,
        upstream,
        state,
        merged,
        hash: tip.to_string().chars().take(7).collect(),
        subject: commit
            .as_ref()
            .and_then(|c| c.summary())
            .unwrap_or_default()
            .to_string(),
        updated: commit.map(|c| c.time().seconds()).unwrap_or_default(),
    }
}
