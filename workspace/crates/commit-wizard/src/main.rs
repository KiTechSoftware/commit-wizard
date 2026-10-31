//! The `cw` executable: runs [`cli::run`](commit_wizard::cli::run) and exits with its status.

fn main() -> std::process::ExitCode {
    commit_wizard::cli::run()
}
