//! Executable commands for the CLI.
//!
//! This module contains the implementation of all CLI subcommands:
//! - `add`: Stage files for commit
//! - `bump`: Bump semantic version
//! - `check`: Validate commits against Conventional Commits
//! - `commit`: Create a structured commit interactively
//! - `config`: Manage configuration
//! - `doctor`: Diagnose issues
//! - `init`: Initialize project configuration
//! - `push`: Push commits and tags
//! - `tag`: Create and manage version tags

pub mod add;
pub mod bump;
pub mod check;
pub mod commit;
pub mod config;
pub mod doctor;
pub mod init;
pub mod push;
pub mod tag;
