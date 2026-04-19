pub mod args;
pub mod cmd;

use std::process::ExitCode;
pub type CliResult<T> = Result<T, Error>;

use crate::{
    core::{bootstrap::build_app_context, exit_code, report_error},
    engine::{Error, ErrorCode},
};
use clap::Parser;
use tracing::Level;
use tracing_subscriber::{EnvFilter, fmt};

pub fn run() -> ExitCode {
    match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime.block_on(async_run()),
        Err(err) => {
            let err = ErrorCode::ProcessFailure
                .error()
                .with_context_str("runtime", err);

            report_error(&err);
            exit_code(&err)
        }
    }
}

async fn async_run() -> ExitCode {
    let cli = args::Cli::parse();

    let default_level = match (cli.global.verbose, cli.global.quiet) {
        (_, q) if q >= 2 => Level::ERROR,
        (_, 1) => Level::WARN,
        (1, _) => Level::INFO,
        (2, _) => Level::DEBUG,
        (v, _) if v >= 3 => Level::TRACE,
        _ => Level::WARN,
    };
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_level.as_str()));
    fmt().with_env_filter(filter).with_writer(std::io::stderr).init();

    let ctx = match build_app_context(cli.global.into()) {
        Ok(ctx) => ctx,
        Err(err) => {
            let err = ErrorCode::ProcessFailure
                .error()
                .with_context_str("app_context", err);
            report_error(&err);
            return exit_code(&err);
        }
    };

    let result = match cli.command {
        args::Command::Add(args) => cmd::add::run(&ctx, args).await,
        args::Command::Bump(args) => cmd::bump::run(&ctx, args).await,
        args::Command::Check(args) => cmd::check::run(&ctx, args).await,
        args::Command::Commit(args) => cmd::commit::run(&ctx, args).await,
        args::Command::Config(args) => cmd::config::run(&ctx, args).await,
        args::Command::Doctor(args) => cmd::doctor::run(&ctx, args).await,
        args::Command::Init(args) => cmd::init::run(&ctx, args).await,
        args::Command::Push(args) => cmd::push::run(&ctx, args).await,
        args::Command::Tag(args) => cmd::tag::run(&ctx, args).await,
    };

    if let Err(err) = result {
        return exit_code(&err);
    }

    ExitCode::SUCCESS
}
