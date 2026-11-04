//! `cw hooks`: installs, removes and reports the Git hooks that run `cw`.

use clap::{Args as ClapArgs, Subcommand, ValueEnum};

use crate::{
    cli::CliResult,
    core::{context::Context, hooks},
};

/// The Git hooks `cw` can manage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum HookName {
    /// Validate each commit message as it is written
    CommitMsg,
    /// Validate the commits about to be pushed
    PrePush,
}

impl HookName {
    pub fn file_name(self) -> &'static str {
        match self {
            Self::CommitMsg => "commit-msg",
            Self::PrePush => "pre-push",
        }
    }
}

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Install, remove and inspect the Git hooks that run Commit Wizard")]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Action {
    /// Install hooks (default: the ones enabled in [hooks], else commit-msg and pre-push)
    Install {
        /// Hooks to install; repeat for several
        #[arg(long = "hook", value_enum)]
        hooks: Vec<HookName>,
    },
    /// Remove hooks that cw installed (foreign hooks are never touched)
    Uninstall {
        /// Hooks to remove (default: all cw-managed hooks)
        #[arg(long = "hook", value_enum)]
        hooks: Vec<HookName>,
    },
    /// Show which hooks are installed
    Status,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    match args.action {
        Action::Install { hooks } => hooks::install(ctx, hooks),
        Action::Uninstall { hooks } => hooks::uninstall(ctx, hooks),
        Action::Status => hooks::status(ctx),
    }
}
