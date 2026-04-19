use clap::Args as ClapArgs;

use crate::{
    cli::CliResult,
    core::{commit, context::Context},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Validate commit history against the active Commit Wizard rules")]
pub struct Args {
    /// Validate last N commits (0 = all)
    #[arg(long, alias = "count", value_name = "N")]
    pub tail: Option<u32>,

    /// Beginning of commit range
    #[arg(long)]
    pub from: Option<String>,

    /// End of commit range
    #[arg(long)]
    pub to: Option<String>,

    /// Full commit hash
    #[arg(long, alias = "full-hash", short = 'H')]
    pub full_commit_hash: bool,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    commit::check::run(ctx, args.tail, args.from, args.to, args.full_commit_hash)
}
