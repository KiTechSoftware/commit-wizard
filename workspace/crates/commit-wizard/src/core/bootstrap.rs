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

pub fn build_app_context(args: AppContextArgs) -> Result<Context> {
    let mode = RunMode::from_flags(args.ci, args.non_interactive);

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

    ctx.resolve_available_sources();
    ctx.resolve_active_config()?;

    Ok(ctx)
}
