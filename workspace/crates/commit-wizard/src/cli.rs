use clap::{Args, Parser, Subcommand};

/// 🧙‍♂️ A lightweight conventional commits assistant.
#[derive(Parser, Debug)]
#[command(
    name = "commit-wizard",
    version,
    author,
    about = "🧙‍♂️ A lightweight conventional commits assistant."
)]
pub struct Cli {
    /// Global flags (apply to all subcommands)
    #[command(flatten)]
    pub global: GlobalOpts,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Args, Debug, Clone)]
pub struct GlobalOpts {
    /// Don't make changes; print what would happen
    #[arg(long, global = true)]
    pub dry_run: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start the guided commit flow
    Commit {
        /// Allow committing with no staged changes
        #[arg(long)]
        allow_empty: bool,
    },
}
