//! Plain data the engine receives from Git: commit summaries and changed files.

#[derive(Debug, Clone)]
pub struct CommitSummary {
    pub hash: String,
    /// First line of the commit message
    pub summary: String,
    /// Full commit message including body and footer (None if not fetched)
    pub full_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Renamed { old: String, new: String },
    TypeChange,
    Untracked,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Change {
    pub path: String,
    pub staged: bool,
    pub status: FileStatus,
}

/// How a local branch stands against the remote branch it follows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchState {
    /// The branch follows nothing.
    Local,
    /// Level with its upstream.
    UpToDate,
    Ahead(usize),
    Behind(usize),
    Diverged {
        ahead: usize,
        behind: usize,
    },
    /// It follows a remote branch that no longer exists (deleted after a merge, say).
    Gone,
}

/// A local or remote-tracking branch, as `cw branch list` shows it.
#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub current: bool,
    pub upstream: Option<String>,
    pub state: BranchState,
    /// Whether every commit on the branch is already in the checked-out history.
    pub merged: bool,
    pub hash: String,
    pub subject: String,
    /// Committer time of the tip, in Unix seconds.
    pub updated: i64,
}
