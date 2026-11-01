//! `cw branch`: lists, switches, creates and prunes branches. Defines the arguments and hands them to the use case.

use clap::{Args as ClapArgs, Subcommand};

use crate::{
    cli::CliResult,
    core::{branch, context::Context},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "List, switch, create and prune branches")]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: Option<BranchSubcommand>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum BranchSubcommand {
    /// List branches with what they follow and where they stand
    List {
        /// Show remote-tracking branches instead of local ones
        #[arg(long, short = 'r', conflicts_with = "all")]
        remote: bool,
        /// Show local and remote-tracking branches
        #[arg(long, short = 'a')]
        all: bool,
    },
    /// Switch to a branch (pick from a list on a terminal)
    Switch {
        /// Branch to switch to; a branch that exists only on a remote is checked out to follow it
        name: Option<String>,
    },
    /// Create a branch named from a type, ticket and description, and switch to it
    New {
        /// The description, as a shorthand for --description; a full branch name if it contains a /
        name: Option<String>,
        /// What the branch is for ("add login page"), turned into words joined by hyphens
        #[arg(long, short = 'd', conflicts_with = "name")]
        description: Option<String>,
        /// Kind of branch, used as the prefix (feature, bugfix, hotfix, ...)
        #[arg(long = "type", short = 't')]
        kind: Option<String>,
        /// Ticket reference to put in the name, such as PROJ-123, #42 or Ticket-012
        #[arg(long, short = 'T')]
        ticket: Option<String>,
        /// Commit, branch or tag to start from (default: the current commit)
        #[arg(long)]
        from: Option<String>,
        /// Create the branch but stay on the current one
        #[arg(long)]
        no_switch: bool,
    },
    /// Delete local branches whose remote branch is gone
    Prune {
        /// Also delete branches with commits that are not merged anywhere else
        /// (a squash-merged branch looks like this)
        #[arg(long)]
        force: bool,
        /// Do not fetch first; use what the last fetch found
        #[arg(long)]
        no_fetch: bool,
        /// Also delete local branches that are merged, even if their remote branch
        /// still exists
        #[arg(long)]
        merged: bool,
        /// Also delete merged branches from the remote (`git push --delete`)
        #[arg(long)]
        remote: bool,
    },
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    match args.subcommand {
        Some(BranchSubcommand::List { remote, all }) => branch::list::run(ctx, remote, all),
        Some(BranchSubcommand::Switch { name }) => branch::switch::run(ctx, name),
        Some(BranchSubcommand::New {
            name,
            description,
            kind,
            ticket,
            from,
            no_switch,
        }) => branch::create::run(
            ctx,
            branch::create::Options {
                name,
                description,
                kind,
                ticket,
                from,
                no_switch,
            },
        ),
        Some(BranchSubcommand::Prune {
            force,
            no_fetch,
            merged,
            remote,
        }) => branch::prune::run(
            ctx,
            branch::prune::Options {
                force,
                no_fetch,
                merged,
                remote,
            },
        ),
        // Bare `cw branch`: choose a branch on a terminal, otherwise list them.
        None if ctx.is_interactive() => branch::switch::run(ctx, None),
        None => branch::list::run(ctx, false, false),
    }
}
