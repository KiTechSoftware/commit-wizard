use std::path::Path;

use crate::core::Result;

pub trait GitCliTrait {
    fn is_installed(&self) -> Result<bool>;
    fn is_inside_work_tree(&self) -> Result<bool>;
}

pub trait GitRepoTrait {
    fn repo_exists(&self) -> Result<bool>;
    fn repo_root(&self) -> Result<&Path>;
    fn repo_sync(&self) -> Result<()>;
    fn repo_dirty(&self) -> Result<bool>;
    fn repo_head(&self) -> Result<String>;
    fn repo_short_head(&self) -> Result<String>;
}

pub trait GitBranchTrait {
    fn branch_exists(&self) -> Result<bool>;
    fn branch_checkout(&self) -> Result<bool>;
    fn branch_merge_base(&self, other: &str) -> Result<String>;
    fn branch_sync(&self) -> Result<bool>;
    fn branch_head(&self) -> Result<String>;
    fn branch_current(&self) -> Result<String>;
}

pub trait GitStatusTrait {
    fn status_is_clean(&self) -> Result<bool>;
    fn status_staged_paths(&self) -> Result<Vec<&str>>;
    fn status_unstaged_paths(&self) -> Result<Vec<&str>>;
    fn status_untracked_paths(&self) -> Result<Vec<&str>>;
}
