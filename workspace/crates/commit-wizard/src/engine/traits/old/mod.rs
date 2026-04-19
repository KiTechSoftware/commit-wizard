use std::path::PathBuf;

use crate::application::models::{
    commit::CommitDto,
    config::{AppConfig, CheckMode},
    stage::Change,
};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct CommitPolicy {
    pub require_conventional_for_all_commits: bool,
    /// Whether to allow pushes to the main branch
    pub allow_push_to_main: bool,
    /// Whether the first commit must be conventional
    pub first_commit_must_be_conventional: bool,
    /// The check mode to use (Strict, Lenient, or Disabled)
    pub check_mode: CheckMode,
    /// The default commit count to check
    pub check_default_count: u32,
    /// Whether to enforce scope usage from the commit configuration
    pub enforce_scopes: bool,
    /// The main branch to use for the push
    pub main_branch: String,
}

#[derive(Debug, Clone)]
pub struct VersionPolicy {
    pub breaking: Vec<String>,
    pub minor: Vec<String>,
    pub patch: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Targeting {
    pub cwd: Option<PathBuf>,      // from CLI --cwd
    pub explicit: Option<PathBuf>, // from CLI --config
    pub global: bool,              // --global
}

#[derive(Debug, Clone)]
pub struct TagOptions {
    pub dry_run: bool,
    pub ci: bool,

    // inputs (some may be None; use case will prompt when !ci)
    pub set_version: Option<String>, // --set-version / --use-version
    /// Prefix behavior:
    /// - None   => unknown; use case will prompt when !ci, default "v" in CI
    /// - Some("") => explicit NO prefix
    /// - Some("v") or other => use that exact prefix
    pub prefix: Option<String>, // --prefix
    pub suffix: Option<String>,      // optional e.g. "-rc1"
    pub sign: Option<bool>,          // if None & !ci → prompt
    pub push: Option<bool>,          // if None & !ci → prompt
    pub remote: Option<String>,      // default "origin" if pushing
    pub message: Option<String>,     // default "Release <tag>"
}

pub trait ConfigResolver {
    /// Where should writes go given the targeting?
    fn resolve_write_target(&self, t: &Targeting) -> anyhow::Result<PathBuf>;

    /// If a config file exists for reading/listing, where is it?
    /// (None means “no file” → core decides behavior)
    fn resolve_read_target(&self, t: &Targeting) -> anyhow::Result<Option<PathBuf>>;
}

pub struct CommitOptions {
    pub dry_run: bool,
    pub ci: bool,
    pub allow_empty: bool,
    pub ctype: Option<String>,
    pub scope: Option<String>,
    pub breaking: bool,
    pub breaking_message: Option<String>,
    pub message: Option<String>,
    pub body: Option<String>,
    pub footer: Vec<String>,
}

pub struct GitOptions {
    pub dry_run: bool,
    pub ci: bool,
    pub allow_empty: bool,
}

#[derive(Debug, Clone)]
pub struct StageOptions {
    pub dry_run: bool,
    pub paths: Vec<String>,
    pub all: bool,
    pub exclude_staged: bool,
    pub unstage: bool,
}
pub struct InitOptions {
    pub dry_run: bool,
    pub yes: bool,
    pub config_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct FixOptions {
    pub dry_run: bool,
    pub ci: bool,
    pub auto: bool,

    // internal helpers retained for your existing flow
    pub regen: bool,
    pub cleanup: bool,
    pub editor_msg: Option<String>,
    pub editor_seq: Option<String>,

    // scanning
    pub count: Option<u32>,

    // logging
    pub verbose: u8,
}

pub struct ChangelogOptions {
    pub dry_run: bool,
    pub all: bool,
    pub from: Option<String>,
    pub to: String,
    pub output: Option<String>,
    pub append: bool,
    pub json: bool,
    pub group_by_scope: bool,
    pub next_tag: Option<String>,
    pub force: bool,
    pub include_non_cc: bool,
}

pub trait GitRepo {
    fn list_commits(&self, from: Option<&str>, to: &str) -> Result<Vec<CommitDto>>;
    fn latest_tag(&self) -> Result<Option<String>>;
    fn tag_date(&self, tag: &str) -> Result<Option<String>>;
    fn create_tag(&self, name: &str, message: &str) -> Result<()>;
    fn create_signed_tag(&self, name: &str, message: &str) -> Result<()>;
    fn push_branch(&self, remote: &str, branch: &str) -> Result<()>;
    fn push_tag(&self, remote: &str, tag: &str) -> Result<()>;
    fn stage_path(&self, path: &str) -> Result<()>;
    fn unstage_path(&self, path: &str) -> Result<()>;
    fn show_unstaged_diff(&self, path: &str) -> Result<String>;
    fn github_web_url(&self) -> Result<Option<String>>;
    fn commit(&self, message: &str, opts: &GitOptions) -> Result<()>;
    fn list_changes(&self) -> Result<Vec<Change>>;
    fn list_tags_sorted(&self) -> anyhow::Result<Vec<String>>;
    fn latest_tag_ancestor_of(&self, rev: &str) -> anyhow::Result<Option<String>> {
        let _ = rev;
        self.latest_tag()
    }
    fn stage_all(&self) -> Result<()>;
    fn unstage_all(&self) -> Result<()>;
    fn stage_paths(&self, paths: &[String]) -> Result<()>;
    fn unstage_paths(&self, paths: &[String]) -> Result<()>;
    fn diff_unstaged(&self, path: &str) -> Result<Option<String>>;
    fn current_branch(&self) -> Result<String>;
}

#[derive(Debug, Clone)]
pub enum ConfigValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<ConfigValue>),
    Object(Vec<(String, ConfigValue)>),
}

impl From<&str> for ConfigValue {
    fn from(s: &str) -> Self {
        ConfigValue::String(s.to_owned())
    }
}

pub trait ConfigStore: Send + Sync {
    /// Load the effective config (adapter should apply defaults/patches so it’s usable).
    fn load(&self) -> Result<AppConfig>;

    /// Save a full config (adapter decides where; typically project file or global).
    fn save(&self, cfg: &AppConfig, target: Option<PathBuf>) -> Result<PathBuf>;

    /// Transactional edit: load -> mutate -> save -> return new config.
    fn edit<F>(&self, f: F, target: Option<PathBuf>) -> Result<(PathBuf, AppConfig)>
    where
        F: FnOnce(&mut AppConfig);

    /// Set a dotted key to a value (format-agnostic). Adapters map to their native doc model.
    fn set_key(&self, dotted: &str, value: ConfigValue, target: Option<PathBuf>)
    -> Result<PathBuf>;

    /// Unset a dotted key. No-op if absent. Returns the path actually written.
    fn unset_key(&self, dotted: &str, target: Option<PathBuf>) -> Result<PathBuf>;
}

pub trait FileStore: Send + Sync {
    fn read(&self, path: &str) -> Result<String>;
    fn write(&self, path: &str, content: &str) -> Result<()>;
    fn exists(&self, path: &str) -> Result<bool>;
}

pub trait Prompt {
    fn confirm(&self, prompt: &str, default: bool) -> Result<bool>;
    fn input(&self, prompt: &str, default: Option<&str>) -> Result<String>;
    fn input_optional(&self, prompt: &str) -> Result<Option<String>>;
    fn select(&self, prompt: &str, items: &[String], default: usize) -> Result<Option<usize>>;
    fn multiselect(&self, prompt: &str, items: &[String], defaults: &[bool]) -> Result<Vec<usize>>;
}
// pub trait Clock { fn today(&self) -> chrono::NaiveDate; }

pub trait EditorPort: Send + Sync {
    fn edit(&self, initial: &str) -> Result<String>;
}

pub trait Renderer: Send + Sync {
    fn info(&self, msg: &str);
    fn render_diff(&self, patch: &str);
}

/// Clock (time source)
pub trait Clock: Send + Sync {
    fn now_rfc3339(&self) -> String;
}

/// Filesystem (simple, text-based I/O used by fix.rs)
pub trait FsPort: Send + Sync {
    fn write_str(&self, path: &str, contents: &str) -> Result<()>;
    fn read_to_string(&self, path: &str) -> Result<String>;
    fn remove_file_if_exists(&self, path: &str) -> Result<()>;
    fn exists(&self, path: &str) -> Result<bool>;
    fn read_lines(&self, path: &str) -> Result<Vec<String>>;
    fn temp_dir(&self) -> PathBuf;
}

/// Env (env vars + current exe)
pub trait EnvPort: Send + Sync {
    fn current_exe(&self) -> Result<PathBuf>;
    fn var(&self, key: &str) -> Result<String>;
}

/// Process runner (for the two git commands needed by fix.rs)
pub trait ProcPort: Send + Sync {
    fn git_rebase_interactive_root(
        &self,
        rebase_fixes_file: &str,
        seq_editor_cmd: &str,
        msg_editor_cmd: &str,
    ) -> anyhow::Result<bool>;

    fn git_rev_list_all(&self) -> anyhow::Result<Vec<String>>;

    fn git_parent_of(&self, commit: &str) -> anyhow::Result<Option<String>>;

    fn git_rebase_interactive_from(
        &self,
        base: &str,
        rebase_fixes_file: &str,
        seq_editor_cmd: &str,
        msg_editor_cmd: &str,
    ) -> anyhow::Result<bool>;
}
