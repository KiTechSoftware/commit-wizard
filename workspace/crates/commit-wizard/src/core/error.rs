//! Turns an engine error into what the user sees: the report printed on stderr and the process exit status (see `engine::error` for the code ranges).

use std::process::ExitCode;

use crate::engine::Error;

pub type CoreResult<T> = Result<T, Error>;

pub fn report_error(err: &Error) {
    eprintln!("{}: {}", err.code.id(), err.message);
    for (key, value) in &err.context {
        eprintln!("  {key}: {value}");
    }
}

pub fn exit_code(err: &Error) -> ExitCode {
    ExitCode::from(err.exit_code() as u8)
}
