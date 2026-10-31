//! Executable commands for the CLI.
//!
//! This module contains the implementation of all CLI subcommands:
//! - `add`: Stage files for commit
//! - `branch`: List, switch, create and prune branches
//! - `bump`: Bump semantic version
//! - `changelog`: Generate a changelog from commit history
//! - `check`: Validate commits against Conventional Commits
//! - `commit`: Create a structured commit interactively
//! - `config`: Manage configuration
//! - `doctor`: Diagnose issues
//! - `fix`: Correct commit messages that break the rules
//! - `init`: Initialize project configuration
//! - `push`: Push commits and tags
//! - `tag`: Create and manage version tags

pub mod add;
pub mod branch;
pub mod bump;
pub mod changelog;
pub mod check;
pub mod commit;
pub mod config;
pub mod doctor;
pub mod fix;
pub mod init;
pub mod push;
pub mod tag;
