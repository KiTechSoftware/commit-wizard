use clap::Args as ClapArgs;

use crate::{
    cli::CliResult,
    core::{commit, context::Context},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Create a structured Git commit using the active Commit Wizard rules")]
pub struct Args {
    /// Allow committing with no staged changes
    #[arg(long)]
    pub allow_empty: bool,
    /// Conventional Commit type (feat, fix, etc.)
    #[arg(long = "type", short = 't')]
    pub commit_type: Option<String>,
    /// Commit Scope (Optional)
    #[arg(long, short = 's')]
    pub scope: Option<String>,
    /// Commit Short Summary (72 chars)
    #[arg(long, short = 'm')]
    pub message: Option<String>,
    /// Breaking Change Boolean
    #[arg(long, short = 'B', default_value = "false")]
    pub breaking: bool,
    /// Breaking Change Message (Optional)
    #[arg(long, short = 'd')]
    pub breaking_message: Option<String>,
    /// Commit Message Body (Optional)
    #[arg(long, short = 'b')]
    pub body: Option<String>,
    /// Commit Message Footer (can be stacked with -f "Author: Wizard" -f "Co-Author: Cat")
    #[arg(long, short = 'f')]
    pub footer: Vec<String>,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    commit::make::run(
        ctx,
        args.allow_empty,
        args.commit_type,
        args.scope,
        args.message,
        args.breaking,
        args.breaking_message,
        args.body,
        args.footer,
    )
}
