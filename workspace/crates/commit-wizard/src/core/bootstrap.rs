//! Application initialization and context building.
//!
//! This module handles the bootstrap process: converting CLI arguments into the application
//! [`Context`], resolving configuration, and initializing runtime state.

use std::path::PathBuf;

use crate::{
    core::context::Context,
    engine::{
        error::Result,
        models::runtime::{Runtime, mode::RunMode},
    },
};

/// All inputs needed to build an application context from CLI arguments.
pub struct AppContextArgs {
    pub verbose: u8,
    pub quiet: u8,
    pub json: bool,
    pub format: String,
    pub dry_run: bool,
    pub output_color: String,
    pub cwd: PathBuf,
    pub ci: bool,
    pub non_interactive: bool,
    pub auto_yes: bool,
    pub force: bool,
    pub config_path: Option<PathBuf>,
    pub registry: Option<String>,
    pub registry_ref: Option<String>,
    pub registry_section: Option<String>,
}

/// Whether a person is at the keyboard: both input and output are terminals.
fn on_terminal() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

/// The run mode. Prompting is the default, but only when it can work: without
/// a terminal (a script, a pipe) or with machine-readable output, cw behaves
/// as if `--non-interactive` had been given.
fn resolve_mode(ci: bool, non_interactive: bool, json: bool, terminal: bool) -> RunMode {
    let mode = RunMode::from_flags(ci, non_interactive);
    if mode == RunMode::Interactive && (json || !terminal) {
        RunMode::NonInteractive
    } else {
        mode
    }
}

pub fn build_app_context(args: AppContextArgs) -> Result<Context> {
    let mode = resolve_mode(args.ci, args.non_interactive, args.json, on_terminal());

    let output_color = if args.output_color == "auto" {
        if mode == RunMode::Interactive {
            scriba::ColorMode::Auto
        } else {
            scriba::ColorMode::Never
        }
    } else {
        scriba::ColorMode::from_str(&args.output_color)
    };

    let output_envelope = if args.json {
        scriba::EnvelopeMode::Json
    } else {
        scriba::EnvelopeMode::None
    };

    let mut runtime = Runtime::new();

    runtime
        .set_mode(mode)
        .set_cwd(args.cwd)
        .set_explicit_config_path(args.config_path)
        .set_explicit_registry(args.registry)
        .set_explicit_registry_ref(args.registry_ref)
        .set_explicit_registry_section(args.registry_section)
        .set_dry_run(args.dry_run)
        .options_mut()
        .set_auto_yes(args.auto_yes)
        .set_force(args.force)
        .set_output_envelope(output_envelope)
        .set_output_format(scriba::Format::from_str(&args.format))
        .set_output_color(output_color)
        .set_log_level(scriba::Level::from_flags(args.verbose, args.quiet));

    let mut ctx = Context::new(runtime);

    let git = ctx.git();
    if git.is_installed() && git.is_inside_work_tree() {
        ctx.set_in_git_repo(true);

        if let Some(repo_root) = git.repo_root() {
            ctx.set_repo_root(repo_root);
        }
    }

    ctx.resolve_available_sources()?;
    ctx.resolve_active_config()?;

    Ok(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompting_needs_a_terminal_and_human_readable_output() {
        assert_eq!(
            resolve_mode(false, false, false, true),
            RunMode::Interactive
        );
        assert_eq!(
            resolve_mode(false, false, false, false),
            RunMode::NonInteractive
        );
        assert_eq!(
            resolve_mode(false, false, true, true),
            RunMode::NonInteractive
        );
    }

    #[test]
    fn explicit_flags_win_over_the_terminal_check() {
        assert_eq!(resolve_mode(true, false, false, true), RunMode::Ci);
        assert_eq!(resolve_mode(true, false, false, false), RunMode::Ci);
        assert_eq!(
            resolve_mode(false, true, false, true),
            RunMode::NonInteractive
        );
    }
}
