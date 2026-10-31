use clap::{Args as ClapArgs, Subcommand};

use crate::{
    cli::CliResult,
    core::{configs, context::Context},
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Inspect and manage Commit Wizard configuration")]
pub struct Args {
    /// Edit the global config instead of project config
    #[arg(short = 'g', long = "global", conflicts_with = "config")]
    pub global: bool,
    #[command(subcommand)]
    pub subcommand: ConfigSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ConfigSubcommand {
    /// Show the config file path that CW will target
    Path,
    /// Show the active configuration
    Show {
        // Show effective configuration (merging global and local)
        // #[arg(long)]
        // effective: bool,
    },
    /// Get a config value
    Get {
        /// Config key to retrieve
        key: String,
    },
    /// Set a config value
    Set {
        /// Config key to set
        key: String,
        /// Value to set
        value: String,
    },
    /// Unset a config value
    Unset {
        /// Config key to unset
        key: String,
    },
    /// List settings: those in the config file, or with --effective the
    /// resolved configuration and where each value came from
    List {
        /// Show the resolved configuration across all layers (registry,
        /// project, env, cli) with the source of every value
        #[arg(long)]
        effective: bool,
        /// Only list keys starting with this prefix, e.g. commit.types
        prefix: Option<String>,
    },
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    let global = args.global;
    match args.subcommand {
        ConfigSubcommand::Path => configs::config_path(ctx, global),
        ConfigSubcommand::List { effective, prefix } => {
            configs::config_list(ctx, global, effective, prefix)
        }
        ConfigSubcommand::Show {} => configs::config_show(ctx, global),
        ConfigSubcommand::Get { key } => configs::config_get(ctx, &key, global),
        ConfigSubcommand::Set { key, value, .. } => configs::config_set(ctx, &key, &value, global),
        ConfigSubcommand::Unset { key } => configs::config_unset(ctx, &key, global),
    }
}
