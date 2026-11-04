//! `cw completions`: prints a shell completion script to standard output.

use clap::{Args as ClapArgs, CommandFactory};
use clap_complete::Shell;

use crate::cli::{CliResult, args::Cli};

#[derive(Debug, Clone, ClapArgs)]
#[command(
    about = "Print a shell completion script for cw",
    long_about = "Print a shell completion script for cw to standard output.\n\n\
        Examples:\n  cw completions bash > ~/.local/share/bash-completion/completions/cw\n  \
        cw completions zsh > ~/.zfunc/_cw\n  cw completions fish > ~/.config/fish/completions/cw.fish"
)]
pub struct Args {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: Shell,
}

pub fn run_standalone(args: Args) -> CliResult<()> {
    let mut cmd = Cli::command();
    clap_complete::generate(args.shell, &mut cmd, "cw", &mut std::io::stdout());
    Ok(())
}
