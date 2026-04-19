use std::{
    collections::{BTreeMap, HashMap},
    io::Write as _,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use git2::{
    DiffFormat, DiffOptions, IndexAddOption, ObjectType, Oid, Repository, Sort, Status,
    StatusEntry, StatusOptions,
};

use crate::engine::{
    Error, ErrorCode,
    models::git::{Change, CommitSummary, FileStatus},
};

/// A tag boundary with all commits that fall in the range `(prev_tag, tag]`.
/// The `tag` field is either a version string or `"Unreleased"` for commits
/// that have not yet been tagged.
#[derive(Debug, Clone)]
pub struct TaggedCommits {
    pub tag: String,
    pub commits: Vec<CommitSummary>,
}

#[derive(Debug, Clone)]
pub struct Git {
    cwd: PathBuf,
}

impl Git {
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        Self { cwd: cwd.into() }
    }

    pub fn is_installed(&self) -> bool {
        self.run(["--version"])
            .is_some_and(|output| output.status.success())
    }

    pub fn is_inside_work_tree(&self) -> bool {
        Repository::discover(&self.cwd).is_ok()
    }

    pub fn repo_root(&self) -> Option<PathBuf> {
        Repository::discover(&self.cwd).ok().and_then(|repo| {
            // workdir() returns None for bare repos; for all others it returns
            // the working tree root (may have a trailing slash, so canonicalize).
            repo.workdir().and_then(|p| std::fs::canonicalize(p).ok())
        })
    }

    pub fn commit(&self, message: &str, allow_empty: bool) -> Result<(), Error> {
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.cwd);
        cmd.arg("commit");

        if allow_empty {
            cmd.arg("--allow-empty");
        }

        cmd.arg("-F").arg("-");

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|err| {
                ErrorCode::ProcessFailure
                    .error()
                    .with_context("command", "git commit -F -")
                    .with_context("cwd", self.cwd.display().to_string())
                    .with_context("error", err.to_string())
            })?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(message.as_bytes()).map_err(|err| {
                ErrorCode::ProcessFailure
                    .error()
                    .with_context("command", "git commit -F -")
                    .with_context("cwd", self.cwd.display().to_string())
                    .with_context("error", err.to_string())
            })?;
        }

        let status = child.wait().map_err(|err| {
            ErrorCode::ProcessFailure
                .error()
                .with_context("command", "git commit -F -")
                .with_context("cwd", self.cwd.display().to_string())
                .with_context("error", err.to_string())
        })?;

        if status.success() {
            Ok(())
        } else {
            Err(ErrorCode::ProcessFailure
                .error()
                .with_context("command", "git commit -F -")
                .with_context("cwd", self.cwd.display().to_string())
                .with_context("status", status.to_string()))
        }
    }

    pub fn list_commits(&self, from: Option<&str>, to: &str) -> Result<Vec<CommitSummary>, Error> {
        let repo = self.repo()?;
        let to_oid = self.revparse_oid(&repo, to)?;
        let from_oid = match from {
            Some(value) => Some(self.revparse_oid(&repo, value)?),
            None => None,
        };

        let mut revwalk = repo
            .revwalk()
            .map_err(|err| self.git2_error("revwalk", err))?;
        revwalk
            .push(to_oid)
            .map_err(|err| self.git2_error("revwalk.push", err))?;

        if let Some(oid) = from_oid {
            revwalk
                .hide(oid)
                .map_err(|err| self.git2_error("revwalk.hide", err))?;
        }

        revwalk
            .set_sorting(Sort::TOPOLOGICAL | Sort::TIME)
            .map_err(|err| self.git2_error("revwalk.set_sorting", err))?;

        let mut out = Vec::new();
        for oid in revwalk {
            let oid = oid.map_err(|err| self.git2_error("revwalk.next", err))?;
            let commit = repo
                .find_commit(oid)
                .map_err(|err| self.git2_error("find_commit", err))?;

            out.push(CommitSummary {
                hash: commit.id().to_string(),
                summary: commit.summary().unwrap_or("").to_string(),
                full_message: commit.message().map(|m| m.to_string()),
            });
        }

        Ok(out)
    }

    pub fn latest_tag(&self) -> Result<Option<String>, Error> {
        let repo = self.repo()?;
        let tags = repo
            .tag_names(None)
            .map_err(|err| self.git2_error("tag_names", err))?;

        let mut latest: Option<(String, git2::Time)> = None;

        for tag_name in tags.iter().flatten() {
            let tag_ref = format!("refs/tags/{tag_name}");
            let Ok(reference) = repo.find_reference(&tag_ref) else {
                continue;
            };
            let Ok(obj) = reference.peel(ObjectType::Commit) else {
                continue;
            };
            let Some(commit) = obj.as_commit() else {
                continue;
            };

            let time = commit.time();

            match &latest {
                Some((_, prev)) if time.seconds() <= prev.seconds() => {}
                _ => latest = Some((tag_name.to_string(), time)),
            }
        }

        Ok(latest.map(|(name, _)| name))
    }

    pub fn tag_date(&self, tag: &str) -> Result<Option<String>, Error> {
        let repo = self.repo()?;
        let reference = match repo.find_reference(&format!("refs/tags/{tag}")) {
            Ok(reference) => reference,
            Err(_) => return Ok(None),
        };

        let obj = match reference.peel(ObjectType::Commit) {
            Ok(obj) => obj,
            Err(_) => return Ok(None),
        };

        let Some(commit) = obj.as_commit() else {
            return Ok(None);
        };

        let ts = commit.time().seconds();
        let Some(dt) = chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0) else {
            return Ok(None);
        };

        Ok(Some(dt.format("%Y-%m-%d").to_string()))
    }

    pub fn create_tag(&self, name: &str, message: &str) -> Result<(), Error> {
        let repo = self.repo()?;
        let obj = repo
            .revparse_single("HEAD")
            .map_err(|err| self.git2_error("revparse_single HEAD", err))?;
        let sig = repo
            .signature()
            .map_err(|err| self.git2_error("signature", err))?;

        repo.tag(name, &obj, &sig, message, false)
            .map_err(|err| self.git2_error("tag", err))?;

        Ok(())
    }

    pub fn create_signed_tag(&self, name: &str, message: &str) -> Result<(), Error> {
        let output = self
            .run_dynamic(["tag", "-s", "-m", message, name])
            .ok_or_else(|| {
                ErrorCode::ProcessFailure
                    .error()
                    .with_context("command", "git tag -s")
                    .with_context("cwd", self.cwd.display().to_string())
            })?;

        self.ensure_success("git tag -s", output)
    }

    pub fn create_branch(&self, name: &str, from: Option<&str>) -> Result<(), Error> {
        let repo = self.repo()?;
        let from_ref = from.unwrap_or("HEAD");
        let target_oid = self.revparse_oid(&repo, from_ref)?;
        let commit = repo
            .find_commit(target_oid)
            .map_err(|err| self.git2_error("find_commit", err))?;
        repo.branch(name, &commit, false)
            .map_err(|err| self.git2_error("branch", err))?;
        Ok(())
    }

    pub fn checkout_branch(&self, name: &str) -> Result<(), Error> {
        let output = self.run_dynamic(["checkout", name]).ok_or_else(|| {
            ErrorCode::ProcessFailure
                .error()
                .with_context("command", "git checkout <branch>")
                .with_context("cwd", self.cwd.display().to_string())
        })?;
        self.ensure_success("git checkout <branch>", output)
    }

    pub fn list_branches(&self) -> Result<Vec<String>, Error> {
        let repo = self.repo()?;
        let mut names = Vec::new();
        for branch in repo
            .branches(Some(git2::BranchType::Local))
            .map_err(|err| self.git2_error("branches", err))?
        {
            let (b, _) = branch.map_err(|err| self.git2_error("branch_iter", err))?;
            if let Some(name) = b
                .name()
                .map_err(|err| self.git2_error("branch.name", err))?
            {
                names.push(name.to_string());
            }
        }
        Ok(names)
    }

    pub fn rename_branch(&self, old_name: &str, new_name: &str) -> Result<(), Error> {
        let output = self
            .run_dynamic(["branch", "-m", old_name, new_name])
            .ok_or_else(|| {
                ErrorCode::ProcessFailure
                    .error()
                    .with_context("command", "git branch -m <old> <new>")
                    .with_context("cwd", self.cwd.display().to_string())
            })?;
        self.ensure_success("git branch -m <old> <new>", output)
    }

    pub fn delete_branch(&self, name: &str, force: bool) -> Result<(), Error> {
        let flag = if force { "-D" } else { "-d" };
        let output = self.run_dynamic(["branch", flag, name]).ok_or_else(|| {
            ErrorCode::ProcessFailure
                .error()
                .with_context("command", format!("git branch {} <name>", flag))
                .with_context("cwd", self.cwd.display().to_string())
        })?;
        self.ensure_success(&format!("git branch {} <name>", flag), output)
    }

    pub fn fetch(&self, remote: &str) -> Result<(), Error> {
        let output = self.run_dynamic(["fetch", remote]).ok_or_else(|| {
            ErrorCode::ProcessFailure
                .error()
                .with_context("command", "git fetch <remote>")
                .with_context("cwd", self.cwd.display().to_string())
        })?;
        self.ensure_success("git fetch <remote>", output)
    }

    pub fn rebase(&self, onto: &str) -> Result<(), Error> {
        let output = self.run_dynamic(["rebase", onto]).ok_or_else(|| {
            ErrorCode::ProcessFailure
                .error()
                .with_context("command", "git rebase <onto>")
                .with_context("cwd", self.cwd.display().to_string())
        })?;
        self.ensure_success("git rebase <onto>", output)
    }

    pub fn merge(&self, branch: &str) -> Result<(), Error> {
        let output = self.run_dynamic(["merge", branch]).ok_or_else(|| {
            ErrorCode::ProcessFailure
                .error()
                .with_context("command", "git merge <branch>")
                .with_context("cwd", self.cwd.display().to_string())
        })?;
        self.ensure_success("git merge <branch>", output)
    }

    pub fn prune_remote_tracking(&self, remote: &str) -> Result<(), Error> {
        let output = self
            .run_dynamic(["remote", "prune", remote])
            .ok_or_else(|| {
                ErrorCode::ProcessFailure
                    .error()
                    .with_context("command", "git remote prune <remote>")
                    .with_context("cwd", self.cwd.display().to_string())
            })?;
        self.ensure_success("git remote prune <remote>", output)
    }

    pub fn is_branch_merged(&self, name: &str) -> Result<bool, Error> {
        let output = self.run_dynamic(["branch", "--merged"]).ok_or_else(|| {
            ErrorCode::ProcessFailure
                .error()
                .with_context("command", "git branch --merged")
                .with_context("cwd", self.cwd.display().to_string())
        })?;
        if !output.status.success() {
            return Ok(false);
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .any(|line| line.trim().trim_start_matches('*').trim() == name))
    }

    pub fn push_branch(&self, remote: &str, branch: &str) -> Result<(), Error> {
        let output = self.run_dynamic(["push", remote, branch]).ok_or_else(|| {
            ErrorCode::ProcessFailure
                .error()
                .with_context("command", "git push <remote> <branch>")
                .with_context("cwd", self.cwd.display().to_string())
        })?;

        self.ensure_success("git push <remote> <branch>", output)
    }

    pub fn push_tag(&self, remote: &str, tag: &str) -> Result<(), Error> {
        let output = self.run_dynamic(["push", remote, tag]).ok_or_else(|| {
            ErrorCode::ProcessFailure
                .error()
                .with_context("command", "git push <remote> <tag>")
                .with_context("cwd", self.cwd.display().to_string())
        })?;

        self.ensure_success("git push <remote> <tag>", output)
    }

    pub fn stage_path(&self, path: &str) -> Result<(), Error> {
        self.stage_paths(&[path.to_string()])
    }

    pub fn unstage_path(&self, path: &str) -> Result<(), Error> {
        self.unstage_paths(&[path.to_string()])
    }

    pub fn list_changes(&self) -> Result<Vec<Change>, Error> {
        let repo = self.repo()?;
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .include_ignored(false)
            .include_unmodified(false)
            .renames_head_to_index(true)
            .renames_index_to_workdir(true)
            .renames_from_rewrites(true)
            .recurse_untracked_dirs(true)
            .show(git2::StatusShow::IndexAndWorkdir);

        let statuses = repo
            .statuses(Some(&mut opts))
            .map_err(|err| self.git2_error("statuses", err))?;

        let mut seen: BTreeMap<String, Change> = BTreeMap::new();

        for entry in statuses.iter() {
            let st = entry.status();
            let staged = st.is_index_new()
                || st.is_index_modified()
                || st.is_index_deleted()
                || st.is_index_renamed()
                || st.is_index_typechange();

            let status = Self::map_status(&entry);
            let path = match &status {
                FileStatus::Renamed { new, .. } => new.clone(),
                _ => Self::best_new_path(&entry)
                    .unwrap_or_else(|| entry.path().unwrap_or_default().to_string()),
            };

            seen.entry(path.clone()).or_insert(Change {
                path,
                staged,
                status,
            });
        }

        Ok(seen.into_values().collect())
    }

    pub fn list_tags_sorted(&self) -> Result<Vec<String>, Error> {
        let output = self
            .run(["tag", "--list", "--sort=creatordate"])
            .ok_or_else(|| {
                ErrorCode::ProcessFailure
                    .error()
                    .with_context("command", "git tag --list --sort=creatordate")
                    .with_context("cwd", self.cwd.display().to_string())
            })?;

        if !output.status.success() {
            return Err(ErrorCode::ProcessFailure
                .error()
                .with_context("command", "git tag --list --sort=creatordate")
                .with_context("cwd", self.cwd.display().to_string())
                .with_context("status", output.status.to_string())
                .with_context(
                    "stderr",
                    String::from_utf8_lossy(&output.stderr).trim().to_string(),
                ));
        }

        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }

    pub fn latest_tag_ancestor_of(&self, rev: &str) -> Result<Option<String>, Error> {
        let output = self
            .run_dynamic(["describe", "--tags", "--abbrev=0", rev])
            .ok_or_else(|| {
                ErrorCode::ProcessFailure
                    .error()
                    .with_context("command", "git describe --tags --abbrev=0 <rev>")
                    .with_context("cwd", self.cwd.display().to_string())
            })?;

        if !output.status.success() {
            return Ok(None);
        }

        let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(if tag.is_empty() { None } else { Some(tag) })
    }

    /// Returns commits grouped by tag boundary, ordered newest-first.
    ///
    /// Each entry contains the tag name (or `"Unreleased"` for the leading
    /// segment) and the commits that fall in that range.  The ranges are
    /// computed as half-open intervals `(prev_tag, tag]` so every commit
    /// appears in exactly one entry.
    ///
    /// `to` is the tip of the walk (usually `"HEAD"`).
    pub fn commits_by_tag(&self, to: &str) -> Result<Vec<TaggedCommits>, Error> {
        let tags = self.list_tags_sorted().unwrap_or_default();
        let mut result: Vec<TaggedCommits> = Vec::new();

        // Unreleased: commits since last tag up to `to`
        let unreleased = self.list_commits(tags.last().map(|s| s.as_str()), to)?;
        if !unreleased.is_empty() {
            result.push(TaggedCommits {
                tag: "Unreleased".to_string(),
                commits: unreleased,
            });
        }

        // One entry per tag (newest first), each covering (prev_tag, this_tag]
        for (idx, tag) in tags.iter().enumerate().rev() {
            let prev = if idx > 0 {
                Some(tags[idx - 1].as_str())
            } else {
                None
            };
            let commits = self.list_commits(prev, tag)?;
            if !commits.is_empty() {
                result.push(TaggedCommits {
                    tag: tag.clone(),
                    commits,
                });
            }
        }

        Ok(result)
    }

    pub fn stage_all(&self) -> Result<(), Error> {
        let repo = self.repo()?;
        let mut index = repo.index().map_err(|err| self.git2_error("index", err))?;
        index
            .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .map_err(|err| self.git2_error("index.add_all", err))?;
        index
            .write()
            .map_err(|err| self.git2_error("index.write", err))?;
        Ok(())
    }

    pub fn unstage_all(&self) -> Result<(), Error> {
        let repo = self.repo()?;
        let head_tree = repo.head().ok().and_then(|h| h.peel_to_tree().ok());
        let mut index = repo.index().map_err(|err| self.git2_error("index", err))?;

        if let Some(tree) = head_tree {
            index
                .read_tree(&tree)
                .map_err(|err| self.git2_error("index.read_tree", err))?;
        } else {
            index
                .clear()
                .map_err(|err| self.git2_error("index.clear", err))?;
        }

        index
            .write()
            .map_err(|err| self.git2_error("index.write", err))?;

        Ok(())
    }

    pub fn stage_paths(&self, paths: &[String]) -> Result<(), Error> {
        if paths.is_empty() {
            return Ok(());
        }

        let repo = self.repo()?;
        let mut index = repo.index().map_err(|err| self.git2_error("index", err))?;

        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .include_ignored(false)
            .include_unmodified(false)
            .renames_head_to_index(true)
            .renames_index_to_workdir(true)
            .renames_from_rewrites(true)
            .recurse_untracked_dirs(true)
            .show(git2::StatusShow::IndexAndWorkdir);

        let statuses = repo
            .statuses(Some(&mut opts))
            .map_err(|err| self.git2_error("statuses", err))?;

        let mut info: HashMap<String, (Status, Option<(String, String)>)> = HashMap::new();

        for entry in statuses.iter() {
            let st = entry.status();
            let path = Self::best_new_path(&entry)
                .or_else(|| entry.path().map(|s| s.to_string()))
                .unwrap_or_default();

            let renamed = if st.intersects(Status::INDEX_RENAMED | Status::WT_RENAMED) {
                Self::renamed_paths(&entry)
            } else {
                None
            };

            info.insert(path, (st, renamed));
        }

        for p in paths {
            let path = Path::new(p);

            if path.is_dir() {
                index
                    .add_all([p.as_str()].iter(), IndexAddOption::DEFAULT, None)
                    .map_err(|err| self.git2_error("index.add_all(dir)", err))?;
                continue;
            }

            if path.exists() {
                index
                    .add_path(path)
                    .map_err(|err| self.git2_error("index.add_path", err))?;
                continue;
            }

            match info.get(p) {
                Some((st, renamed)) => {
                    if st.intersects(Status::WT_DELETED | Status::INDEX_DELETED) {
                        index
                            .remove_path(path)
                            .map_err(|err| self.git2_error("index.remove_path", err))?;
                        continue;
                    }

                    if let Some((old, new)) = renamed {
                        let new_path = Path::new(new);

                        if new_path.is_dir() {
                            index
                                .add_all([new.as_str()].iter(), IndexAddOption::DEFAULT, None)
                                .map_err(|err| self.git2_error("index.add_all(rename dir)", err))?;
                        } else if new_path.exists() {
                            index.add_path(new_path).map_err(|err| {
                                self.git2_error("index.add_path(rename new)", err)
                            })?;
                        } else {
                            index
                                .add_all([new.as_str()].iter(), IndexAddOption::DEFAULT, None)
                                .map_err(|err| {
                                    self.git2_error("index.add_all(rename fallback)", err)
                                })?;
                        }

                        index
                            .remove_path(Path::new(old))
                            .map_err(|err| self.git2_error("index.remove_path(rename old)", err))?;
                        continue;
                    }

                    index
                        .add_all([p.as_str()].iter(), IndexAddOption::DEFAULT, None)
                        .map_err(|err| self.git2_error("index.add_all(fallback)", err))?;
                }
                None => {
                    index
                        .add_all([p.as_str()].iter(), IndexAddOption::DEFAULT, None)
                        .map_err(|err| self.git2_error("index.add_all(missing)", err))?;
                }
            }
        }

        index
            .write()
            .map_err(|err| self.git2_error("index.write", err))?;

        Ok(())
    }

    pub fn unstage_paths(&self, paths: &[String]) -> Result<(), Error> {
        if paths.is_empty() {
            return Ok(());
        }

        let repo = self.repo()?;
        let head_exists = repo.head().is_ok();

        if head_exists {
            let mut chunk: Vec<&str> = Vec::new();
            const CHUNK_SIZE: usize = 200;

            for path in paths {
                chunk.push(path.as_str());

                if chunk.len() >= CHUNK_SIZE {
                    self.git_reset_head_paths(&chunk)?;
                    chunk.clear();
                }
            }

            if !chunk.is_empty() {
                self.git_reset_head_paths(&chunk)?;
            }

            Ok(())
        } else {
            let mut index = repo.index().map_err(|err| self.git2_error("index", err))?;

            for path in paths {
                index
                    .remove_all([path.as_str()].iter(), None)
                    .map_err(|err| {
                        ErrorCode::ProcessFailure
                            .error()
                            .with_context("command", "git rm --cached / index.remove_all")
                            .with_context("cwd", self.cwd.display().to_string())
                            .with_context("path", path.clone())
                            .with_context("error", err.to_string())
                    })?;
            }

            index
                .write()
                .map_err(|err| self.git2_error("index.write", err))?;

            Ok(())
        }
    }

    pub fn diff_unstaged(&self, path: &str) -> Result<Option<String>, Error> {
        let repo = self.repo()?;
        let mut opts = DiffOptions::new();
        opts.pathspec(path);

        let diff = repo
            .diff_index_to_workdir(None, Some(&mut opts))
            .map_err(|err| self.git2_error("diff_index_to_workdir", err))?;

        if diff.deltas().len() == 0 {
            return Ok(None);
        }

        let mut buf = String::new();
        diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
            let origin = line.origin();
            let text = std::str::from_utf8(line.content()).unwrap_or("");

            match origin {
                '+' | '-' | ' ' => {
                    buf.push(origin);
                    buf.push_str(text);
                }
                _ => buf.push_str(text),
            }

            true
        })
        .map_err(|err| self.git2_error("diff.print", err))?;

        Ok(Some(buf))
    }

    pub fn show_unstaged_diff(&self, path: &str) -> Result<String, Error> {
        Ok(self.diff_unstaged(path)?.unwrap_or_default())
    }

    pub fn github_web_url(&self) -> Result<Option<String>, Error> {
        let repo = self.repo()?;
        let remotes = repo
            .remotes()
            .map_err(|err| self.git2_error("remotes", err))?;

        let remote = remotes.iter().flatten().find(|r| *r == "origin");
        let Some(remote) = remote else {
            return Ok(None);
        };

        let remote_obj = repo
            .find_remote(remote)
            .map_err(|err| self.git2_error("find_remote", err))?;
        let Some(url) = remote_obj.url() else {
            return Ok(None);
        };

        let https = if url.starts_with("git@github.com:") {
            url.replacen("git@github.com:", "https://github.com/", 1)
                .trim_end_matches(".git")
                .to_string()
        } else if url.starts_with("https://github.com/") {
            url.trim_end_matches(".git").to_string()
        } else {
            return Ok(None);
        };

        Ok(Some(https))
    }

    pub fn current_branch(&self) -> Result<String, Error> {
        let repo = self.repo()?;
        let head = repo.head().map_err(|err| {
            ErrorCode::ProcessFailure
                .error()
                .with_context("command", "git symbolic-ref --short HEAD")
                .with_context("cwd", self.cwd.display().to_string())
                .with_context("error", err.to_string())
                .with_context("reason", "failed to read HEAD")
        })?;

        if head.is_branch() {
            head.shorthand().map(|s| s.to_string()).ok_or_else(|| {
                ErrorCode::ProcessFailure
                    .error()
                    .with_context("command", "git symbolic-ref --short HEAD")
                    .with_context("cwd", self.cwd.display().to_string())
                    .with_context("reason", "unable to determine branch name from HEAD")
            })
        } else {
            let oid = head
                .target()
                .map(|id| id.to_string())
                .unwrap_or_else(|| "<unknown>".to_string());

            Err(ErrorCode::ProcessFailure
                .error()
                .with_context("command", "git symbolic-ref --short HEAD")
                .with_context("cwd", self.cwd.display().to_string())
                .with_context("reason", "detached HEAD")
                .with_context("head", oid))
        }
    }

    fn repo(&self) -> Result<Repository, Error> {
        Repository::discover(&self.cwd).map_err(|err| {
            ErrorCode::ProcessFailure
                .error()
                .with_context("command", "git rev-parse --git-dir")
                .with_context("cwd", self.cwd.display().to_string())
                .with_context("error", err.to_string())
                .with_context("reason", "failed to discover git repository")
        })
    }

    fn revparse_oid(&self, repo: &Repository, rev: &str) -> Result<Oid, Error> {
        repo.revparse_single(rev)
            .map_err(|err| {
                ErrorCode::ProcessFailure
                    .error()
                    .with_context("command", "git rev-parse <rev>")
                    .with_context("cwd", self.cwd.display().to_string())
                    .with_context("rev", rev.to_string())
                    .with_context("error", err.to_string())
            })
            .and_then(|obj| {
                // Peel annotated tags down to the underlying commit object
                obj.peel(ObjectType::Commit).map(|c| c.id()).map_err(|err| {
                    ErrorCode::ProcessFailure
                        .error()
                        .with_context("command", "git rev-parse <rev>")
                        .with_context("cwd", self.cwd.display().to_string())
                        .with_context("rev", rev.to_string())
                        .with_context("error", err.to_string())
                })
            })
    }

    fn git_reset_head_paths(&self, paths: &[&str]) -> Result<(), Error> {
        let mut args = vec!["reset", "-q", "HEAD", "--"];
        args.extend(paths.iter().copied());

        let output = Command::new("git")
            .current_dir(&self.cwd)
            .args(&args)
            .output()
            .map_err(|err| {
                ErrorCode::ProcessFailure
                    .error()
                    .with_context("command", format!("git {}", args.join(" ")))
                    .with_context("cwd", self.cwd.display().to_string())
                    .with_context("error", err.to_string())
            })?;

        if output.status.success() {
            Ok(())
        } else {
            Err(ErrorCode::ProcessFailure
                .error()
                .with_context("command", format!("git {}", args.join(" ")))
                .with_context("cwd", self.cwd.display().to_string())
                .with_context("status", output.status.to_string())
                .with_context(
                    "stderr",
                    String::from_utf8_lossy(&output.stderr).trim().to_string(),
                ))
        }
    }

    fn git2_error(&self, operation: &str, err: git2::Error) -> Error {
        ErrorCode::ProcessFailure
            .error()
            .with_context("operation", operation.to_string())
            .with_context("cwd", self.cwd.display().to_string())
            .with_context("error", err.to_string())
    }

    fn ensure_success(&self, command: &str, output: Output) -> Result<(), Error> {
        if output.status.success() {
            return Ok(());
        }

        Err(ErrorCode::ProcessFailure
            .error()
            .with_context("command", command.to_string())
            .with_context("cwd", self.cwd.display().to_string())
            .with_context("status", output.status.to_string())
            .with_context(
                "stderr",
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ))
    }

    fn is_staged_bits(status: Status) -> bool {
        status.intersects(
            Status::INDEX_NEW
                | Status::INDEX_MODIFIED
                | Status::INDEX_DELETED
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE,
        )
    }

    fn renamed_paths(entry: &StatusEntry<'_>) -> Option<(String, String)> {
        if let Some(delta) = entry.head_to_index() {
            let old = delta.old_file().path()?.to_string_lossy().into_owned();
            let new = delta.new_file().path()?.to_string_lossy().into_owned();
            return Some((old, new));
        }

        if let Some(delta) = entry.index_to_workdir() {
            let old = delta.old_file().path()?.to_string_lossy().into_owned();
            let new = delta.new_file().path()?.to_string_lossy().into_owned();
            return Some((old, new));
        }

        None
    }

    fn best_new_path(entry: &StatusEntry<'_>) -> Option<String> {
        if let Some(delta) = entry.head_to_index()
            && let Some(path) = delta.new_file().path()
        {
            return Some(path.to_string_lossy().into_owned());
        }

        if let Some(delta) = entry.index_to_workdir()
            && let Some(path) = delta.new_file().path()
        {
            return Some(path.to_string_lossy().into_owned());
        }

        entry.path().map(|p| p.to_string())
    }

    fn map_status(entry: &StatusEntry<'_>) -> FileStatus {
        let status = entry.status();

        if status.contains(Status::IGNORED) {
            return FileStatus::Unknown;
        }

        let staged = Self::is_staged_bits(status);

        if status.contains(Status::WT_NEW) && !staged {
            return FileStatus::Untracked;
        }

        if status.intersects(Status::INDEX_RENAMED | Status::WT_RENAMED) {
            if let Some((old, new)) = Self::renamed_paths(entry) {
                return FileStatus::Renamed { old, new };
            }
            return FileStatus::Unknown;
        }

        if status.intersects(Status::INDEX_DELETED | Status::WT_DELETED) {
            return FileStatus::Deleted;
        }

        if status.intersects(Status::INDEX_TYPECHANGE | Status::WT_TYPECHANGE) {
            return FileStatus::TypeChange;
        }

        if status.intersects(Status::INDEX_MODIFIED | Status::WT_MODIFIED) {
            return FileStatus::Modified;
        }

        if status.intersects(Status::INDEX_NEW | Status::WT_NEW) {
            return FileStatus::Added;
        }

        FileStatus::Unknown
    }

    fn run<const N: usize>(&self, args: [&str; N]) -> Option<Output> {
        Command::new("git")
            .current_dir(&self.cwd)
            .args(args)
            .output()
            .ok()
    }

    fn run_dynamic<const N: usize>(&self, args: [&str; N]) -> Option<Output> {
        Command::new("git")
            .current_dir(&self.cwd)
            .args(args)
            .output()
            .ok()
    }
}
