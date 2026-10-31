use clap::Args as ClapArgs;

use crate::{
    cli::CliResult,
    core::{context::Context, usecases::commit::fix},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(
    about = "Correct commit messages that break the project's rules",
    long_about = "Correct commit messages that break the project's rules.\n\n\
        Only messages change: each commit keeps its files, author and dates, and the commits \
        from the first fixed one up to HEAD get new hashes. The old tip is kept under \
        refs/cw/fix-backup/ so the change can be undone.\n\n\
        By default only commits that are not on any remote are checked. Commits that are \
        already published are refused unless the global --force is given. On a terminal you \
        are walked through each offending commit; in scripts give the corrected first line \
        with --reword, and use --dry-run to list what would change."
)]
pub struct Args {
    /// Check the commits after this one (a tag, branch or hash) instead of the unpublished ones
    #[arg(long, value_name = "REF")]
    pub from: Option<String>,

    /// Only look at the newest N commits (0 = all)
    #[arg(long, alias = "count", value_name = "N")]
    pub tail: Option<u32>,

    /// Corrected first line for one commit, as HASH=HEADER (repeatable)
    #[arg(long, value_name = "HASH=HEADER")]
    pub reword: Vec<String>,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    fix::run(
        ctx,
        fix::Options {
            from: args.from,
            tail: args.tail,
            rewords: args.reword,
        },
    )
}
