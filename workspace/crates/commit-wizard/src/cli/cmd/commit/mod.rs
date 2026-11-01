//! `cw commit`: writes a Conventional Commit. Defines the arguments and hands them to the use case.

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
    /// Ticket reference for the subject, such as `PROJ-123`, `#42` or `Ticket-012`
    #[arg(long, short = 'T')]
    pub ticket: Option<String>,
    /// Stage every change first, untracked files included (like `cw add -A`)
    #[arg(long = "all", short = 'A', conflicts_with = "tracked")]
    pub all: bool,
    /// Stage changes to tracked files first, like `git commit -a`
    #[arg(long, short = 'a')]
    pub tracked: bool,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    commit::make::run(
        ctx,
        commit::make::Options {
            allow_empty: args.allow_empty,
            commit_type: args.commit_type,
            scope: args.scope,
            message: args.message,
            breaking: args.breaking,
            breaking_message: args.breaking_message,
            body: args.body,
            footer: args.footer,
            ticket: args.ticket,
            stage_all: args.all,
            stage_tracked: args.tracked,
        },
    )
}
