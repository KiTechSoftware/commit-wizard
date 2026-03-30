pub mod adapters;
pub mod application;
pub mod cli;
pub mod domain;
pub mod ports;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command};

use adapters::git::{cmd::CmdGit, noop::NoopGit};
use adapters::prompt::tty::TtyPrompt;
use application::usecases::CreateCommit;
use ports::git::CommitOptions;

fn run_commit(allow_empty: bool, dry_run: bool) -> Result<()> {
    let opts = CommitOptions { allow_empty };

    // pick adapter based on global dry-run
    let git: Box<dyn ports::git::GitPort> = if dry_run {
        Box::new(NoopGit::default())
    } else {
        Box::new(CmdGit::default())
    };

    let uc = CreateCommit::new(TtyPrompt::default(), git);
    uc.run(&opts)
}

pub fn run_cli() -> Result<()> {
    let cli = Cli::parse();
    let dry_run = cli.global.dry_run;

    match cli.command {
        Command::Commit { allow_empty } => run_commit(allow_empty, dry_run)?,
    }

    Ok(())
}
