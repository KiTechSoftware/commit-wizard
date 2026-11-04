//! `cw release`: opens and finishes a release branch.

use clap::{Args as ClapArgs, Subcommand};

use crate::{
    cli::CliResult,
    core::{context::Context, release},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(
    about = "Open and finish a release branch using the [release] settings",
    long_about = "Open and finish a release branch using the [release] settings.\n\n\
        `start` creates the release branch from the source branch. `finish` merges it into \
        the target branch, tags the release, pushes, and merges back. Requires \
        `[release] enabled = true`. Use --dry-run to preview."
)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Action {
    /// Create and switch to the release branch
    Start {
        /// Release version (default: calculated from the commits since the last tag)
        #[arg(long = "set-version", value_name = "SEMVER")]
        set_version: Option<String>,
    },
    /// Merge the release branch, tag it, push, and merge back
    Finish {
        /// Release version (default: taken from the current release branch name)
        #[arg(long = "set-version", value_name = "SEMVER")]
        set_version: Option<String>,
        /// Remote to push to
        #[arg(long, default_value = "origin")]
        remote: String,
    },
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    match args.action {
        Action::Start { set_version } => release::start(ctx, set_version),
        Action::Finish {
            set_version,
            remote,
        } => release::finish(ctx, set_version, remote),
    }
}
