use clap::{Args as ClapArgs, Subcommand};

use crate::{
    cli::CliResult,
    core::{context::Context, doctor},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Inspect and diagnose the local Commit Wizard environment")]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: Option<DoctorSubcommand>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum DoctorSubcommand {
    /// Attempt safe local repairs for issues detected by doctor
    Fix,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    match args.subcommand {
        Some(DoctorSubcommand::Fix) => doctor::fix(ctx),
        None => doctor::run(ctx),
    }
}
