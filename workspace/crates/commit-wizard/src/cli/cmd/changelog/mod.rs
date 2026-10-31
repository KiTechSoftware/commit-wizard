//! `cw changelog`: generates a changelog. Defines the arguments and hands them to the use case.

use clap::{Args as ClapArgs, ValueEnum};

use crate::{
    cli::CliResult,
    core::{context::Context, usecases::changelog},
    engine::models::policy::changelog::ChangelogOutputFormat,
};

/// File format for the generated changelog.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FileFormat {
    Markdown,
    Json,
}

impl From<FileFormat> for ChangelogOutputFormat {
    fn from(value: FileFormat) -> Self {
        match value {
            FileFormat::Markdown => Self::Markdown,
            FileFormat::Json => Self::Json,
        }
    }
}

#[derive(Debug, Clone, ClapArgs)]
#[command(
    about = "Generate or update a changelog from Conventional Commit history",
    long_about = "Generate or update a changelog from Conventional Commit history.\n\n\
        By default the changes since the latest version tag are written as an \"Unreleased\" \
        section (or under --next-tag) above the existing releases. Use --all to rebuild every \
        release from the tag history, or --from/--to for a single custom range. Global --force \
        recreates the file instead of updating it, and --dry-run previews without writing."
)]
pub struct Args {
    /// Generate every release found in the tag history
    #[arg(long)]
    pub all: bool,

    /// Start of a custom range (exclusive); produces a single section
    #[arg(long)]
    pub from: Option<String>,

    /// End of the range
    #[arg(long, default_value = "HEAD")]
    pub to: String,

    /// File to write (default: changelog.output from config); `-` prints to stdout
    #[arg(long, value_name = "PATH")]
    pub output: Option<String>,

    /// File format (default: changelog.format from config)
    #[arg(long, value_enum, value_name = "FORMAT")]
    pub output_format: Option<FileFormat>,

    /// Group entries by scope under each section
    #[arg(long)]
    pub group_by_scope: bool,

    /// Label the pending changes with this version instead of "Unreleased"
    #[arg(long, value_name = "VERSION")]
    pub next_tag: Option<String>,

    /// Also list commits that are not Conventional Commits, under the misc section
    #[arg(long)]
    pub include_non_cc: bool,

    /// Give prerelease tags (such as v1.0.0-rc.1) their own sections
    #[arg(long)]
    pub prereleases: bool,
}

pub async fn run(ctx: &Context, args: Args) -> CliResult<()> {
    changelog::create::run(
        ctx,
        changelog::create::Options {
            all: args.all,
            from: args.from,
            to: args.to,
            output: args.output,
            format: args.output_format.map(Into::into),
            group_by_scope: args.group_by_scope,
            next_tag: args.next_tag,
            include_non_cc: args.include_non_cc,
            prereleases: args.prereleases,
        },
    )
}
