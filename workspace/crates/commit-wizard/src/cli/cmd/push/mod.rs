use clap::Args as ClapArgs;

use crate::{
    cli::CliResult,
    core::{commit, context::Context},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Push local Git changes while applying Commit Wizard push-time policy checks")]
pub struct Args {
    /// Start commit (e.g. tag or hash). If omitted, compare from repo start (or upstream in a later enhancement).
    #[arg(long)]
    pub from: Option<String>,
    /// End commit (default: HEAD)
    #[arg(long, default_value = "HEAD")]
    pub to: String,
    /// Push destination (default: origin)
    #[arg(long, default_value = "origin")]
    pub remote: String,
    /// Branch to push (default: current branch)
    #[arg(long)]
    pub branch: Option<String>,
    /// Permit pushing with no new commits
    #[arg(long)]
    pub allow_empty: bool,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    commit::push::run(
        ctx,
        args.from,
        args.to,
        args.remote,
        args.branch,
        args.allow_empty,
    )
}
