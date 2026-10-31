use clap::{Args as ClapArgs, Subcommand, ValueEnum};
use std::fmt;

use crate::{
    cli::CliResult,
    core::{
        configs::{
            self,
            wizard::{self, InitScope},
        },
        context::Context,
    },
    engine::config::ConfigFormat,
};

#[derive(Debug, Clone, ClapArgs)]
#[command(about = "Initialize Commit Wizard configuration assets")]
pub struct Args {
    #[arg(long, conflicts_with_all = ["full", "profile"])]
    pub minimal: bool,

    #[arg(long, conflicts_with_all = ["minimal", "profile"])]
    pub full: bool,

    /// Profile to start from (default: standard). Skips the profile question.
    #[arg(long, value_enum)]
    pub profile: Option<InitProfile>,

    /// Set multiple keys: --set policy.require_conventional_for_all_commits=true
    #[arg(long = "set", value_parser = clap::builder::NonEmptyStringValueParser::new())]
    pub sets: Vec<String>,

    /// Format of the config file to create (ignored when --config names a file)
    #[arg(long, global = true, value_enum)]
    pub file_format: Option<FileFormat>,

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
    /// The profile a flag asked for, if any.
    pub fn chosen(args: &Args) -> Option<Self> {
        if args.minimal {
            Some(Self::Minimal)
        } else if args.full {
            Some(Self::Full)
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
    let profile_flag = InitProfile::chosen(&args).map(|profile| profile.to_string());
    let format_flag = args.file_format.map(ConfigFormat::from);

    // Guided: a person at a terminal is asked how the project works. Flags
    // answer their own question, and scripts, --yes, --ci and --non-interactive
    // skip the wizard entirely.
    let scope = match &args.subcommand {
        None => Some(InitScope::Project { hidden: true }),
        Some(InitSubcommand::Project { hidden }) => Some(InitScope::Project { hidden: *hidden }),
        Some(InitSubcommand::Config { global: true }) => Some(InitScope::Global),
        _ => None,
    };
    let plan = match scope {
        Some(scope) if ctx.is_interactive() && !ctx.auto_yes() => {
            match wizard::guide(ctx, scope, profile_flag.clone(), format_flag)? {
                Some(plan) => Some(plan),
                None => return Ok(()),
            }
        }
        _ => None,
    };

    let profile = plan
        .as_ref()
        .map(|plan| plan.profile.clone())
        .or(profile_flag)
        .unwrap_or_else(|| InitProfile::default().to_string());
    let file_format = plan
        .as_ref()
        .map(|plan| plan.file_format)
        .or(format_flag)
        .unwrap_or_default();
    let (answers, overwrite) = match plan {
        Some(plan) => (Some(plan.answers), plan.overwrite),
        None => (None, false),
    };

    match args.subcommand {
        Some(InitSubcommand::Project { hidden }) => {
            configs::init_project_config(ctx, hidden, profile, file_format, answers, overwrite)
        }
        Some(InitSubcommand::Config { global }) => {
            configs::init_config(ctx, global, profile, file_format, answers, overwrite)
        }
        Some(InitSubcommand::Rules { global }) => {
            configs::init_rules(ctx, global, profile, file_format)
        }
        Some(InitSubcommand::Registry {
            path,
            with_rules,
            section,
        }) => configs::init_registry(ctx, path, with_rules, section, profile),
        None => configs::init_project_config(ctx, true, profile, file_format, answers, overwrite),
    }
}
