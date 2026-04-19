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
