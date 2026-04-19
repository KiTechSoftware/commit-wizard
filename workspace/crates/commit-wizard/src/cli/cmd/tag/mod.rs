use clap::Args as ClapArgs;

use crate::{
    cli::CliResult,
    core::{context::Context, version::tag},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(
    about = "Create a release tag for the current repository using the active Commit Wizard rules"
)]
pub struct Args {
    /// Explicit version (e.g., 1.2.3 or v1.2.3)
    #[arg(long = "set-version", alias = "use-version", value_name = "SEMVER")]
    pub set_version: Option<String>,

    /// Tag prefix (e.g., v; use \"\" for none)
    #[arg(long)]
    pub prefix: Option<String>,

    /// Tag suffix (e.g., -rc1)
    #[arg(long)]
    pub suffix: Option<String>,

    /// Create GPG-signed tag
    #[arg(long)]
    pub sign: bool,

    /// Push tag to remote
    #[arg(long)]
    pub push: bool,

    /// Branch to create tag from (default: HEAD)
    #[arg(long)]
    pub branch: Option<String>,

    /// Remote for push (default: origin)
    #[arg(long)]
    pub remote: Option<String>,

    /// Tag message/body
    #[arg(long)]
    pub message: Option<String>,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    tag::run(
        ctx,
        args.set_version,
        args.prefix,
        args.suffix,
        args.sign,
        args.push,
        args.branch,
        args.remote,
        args.message,
    )
}
