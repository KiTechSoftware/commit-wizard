use clap::{Args as ClapArgs, Subcommand, ValueEnum};
use std::fmt;

use crate::{
    cli::CliResult,
    core::{configs, context::Context},
    engine::config::ConfigFormat,
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Initialize Commit Wizard configuration assets")]
pub struct Args {
    #[arg(long, conflicts_with_all = ["full", "profile"])]
    pub minimal: bool,

    #[arg(long, conflicts_with_all = ["minimal", "profile"])]
    pub full: bool,

    #[arg(long, default_value_t = InitProfile::Standard, value_enum)]
    pub profile: InitProfile,

    /// Set multiple keys: --set policy.require_conventional_for_all_commits=true
    #[arg(long = "set", value_parser = clap::builder::NonEmptyStringValueParser::new())]
    pub sets: Vec<String>,

    /// Format of the config file to create (ignored when --config names a file)
    #[arg(long, global = true, value_enum, default_value_t = FileFormat::Toml)]
    pub file_format: FileFormat,

    #[command(subcommand)]
    pub subcommand: Option<InitSubcommand>,
}

/// File format for generated config files.
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum FileFormat {
    #[default]
    Toml,
    Yaml,
    Json,
}

impl From<FileFormat> for ConfigFormat {
    fn from(value: FileFormat) -> Self {
        match value {
            FileFormat::Toml => Self::Toml,
            FileFormat::Yaml => Self::Yaml,
            FileFormat::Json => Self::Json,
        }
    }
}

impl fmt::Display for FileFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(ConfigFormat::from(*self).name())
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum InitProfile {
    Minimal,
    #[default]
    Standard,
    Full,
}

impl InitProfile {
    pub fn resolved(args: &Args) -> Self {
        if args.minimal {
            Self::Minimal
        } else if args.full {
            Self::Full
        } else {
            args.profile
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::Standard => "standard",
            Self::Full => "full",
        }
    }
}

impl fmt::Display for InitProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str((*self).as_str())
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum InitSubcommand {
    Project {
        #[arg(long)]
        hidden: bool,
    },
    Config {
        #[arg(long, short)]
        global: bool,
    },
    Rules {
        #[arg(long, short)]
        global: bool,
    },
    Registry {
        path: std::path::PathBuf,
        #[arg(long = "rules")]
        with_rules: bool,
        #[arg(long)]
        section: Vec<String>,
    },
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    let profile = InitProfile::resolved(&args).to_string();
    let file_format = ConfigFormat::from(args.file_format);

    match args.subcommand {
        Some(InitSubcommand::Project { hidden }) => {
            configs::init_project_config(ctx, hidden, profile, file_format)
        }
        Some(InitSubcommand::Config { global }) => {
            configs::init_config(ctx, global, profile, file_format)
        }
        Some(InitSubcommand::Rules { global }) => {
            configs::init_rules(ctx, global, profile, file_format)
        }
        Some(InitSubcommand::Registry {
            path,
            with_rules,
            section,
        }) => configs::init_registry(ctx, path, with_rules, section, profile),
        None => configs::init_project_config(ctx, true, profile, file_format),
    }
}
