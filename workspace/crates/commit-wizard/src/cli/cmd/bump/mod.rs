use clap::Args as ClapArgs;

use crate::{
    cli::CliResult,
    core::{context::Context, version},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(
    about = "Calculate the next semantic version based on the active Commit Wizard rules and selected commit history"
)]
pub struct Args {
    /// Beginning of commit range
    #[arg(long)]
    pub from: Option<String>,

    /// End of commit range
    #[arg(long, default_value = "HEAD")]
    pub to: String,

    /// Validate the last N commits (0 = all)
    #[arg(long, alias = "count", value_name = "N")]
    pub tail: Option<u32>,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    version::bump::run(ctx, args.from, args.to, args.tail)
}
