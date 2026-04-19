use std::path::PathBuf;

use crate::core::usage;
use crate::{cli::CliResult, core::context::Context};
use clap::{ArgAction, Args as ClapArgs};
use std::collections::HashSet;

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Stage changes in the current Git repository for commit preparation")]
pub struct Args {
    /// One or more paths (files or directories) to stage/unstage
    /// e.g. `cw add src/ README.md templates/`
    #[arg(value_name = "PATH", num_args = 0.., trailing_var_arg = true)]
    pub paths: Vec<PathBuf>,

    /// Additional explicit paths; may be repeated: `--path a --path b`
    #[arg(long = "path", value_name = "PATH", num_args = 1.., action = ArgAction::Append)]
    pub more_paths: Option<Vec<PathBuf>>,

    /// Add all files (equivalent to `git add -A`)
    #[arg(long, short = 'A')]
    pub all: bool,

    /// Exclude already staged files from the selection list (interactive)
    #[arg(long)]
    pub exclude_staged: bool,

    /// Unstage instead of staging (affects paths and --all fast-paths; narrows interactive to staged)
    #[arg(long)]
    pub unstage: bool,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    let mut paths = args.paths;

    if let Some(mut more) = args.more_paths {
        paths.append(&mut more);
    }

    let mut seen = HashSet::new();
    paths.retain(|p| seen.insert(p.clone()));

    usage::stage::run(ctx, paths, args.all, args.exclude_staged, args.unstage)
}
