//! # Commit Wizard
//!
//! A lightweight Rust library for **Conventional Commits**, semantic versioning, and changelog automation.
//!
//! This crate provides the core functionality for validating commits, managing versions, generating
//! changelogs, and managing Git tags according to semantic versioning principles. It can be used
//! programmatically via the library interface or through the `cw` CLI binary.
//!
//! ## Modules
//!
//! - [`cli`]: Command-line interface implementation (public commands and arguments)
//! - [`core`]: Core application logic including bootstrap, context management, and use cases
//! - [`engine`]: Business logic layer with traits, models, capabilities, and configuration
//! - [`infra`]: Infrastructure layer for Git operations, UI interactions, and file system access
//!
//! ## Example Usage
//!
//! ```ignore
//! use commit_wizard::core::Context;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create application context
//!     let context = Context::new().await?;
//!
//!     // Use context to access capabilities and operations
//!     Ok(())
//! }
//! ```
//!
//! ## Conventional Commits
//!
//! This library enforces [Conventional Commits](https://www.conventionalcommits.org/) format:
//!
//! ```text
//! <type>(<scope>): <description>
//!
//! <body>
//!
//! <footer>
//! ```
//!
//! Supported types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`, `build`
//!
//! ## Semantic Versioning
//!
//! Version bumping follows [Semantic Versioning](https://semver.org/) principles:
//! - `MAJOR`: Breaking changes (`feat!` or `BREAKING CHANGE`)
//! - `MINOR`: New features (`feat`)
//! - `PATCH`: Bug fixes (`fix`)

pub mod cli;
pub mod core;
pub mod engine;
pub mod infra;

/// A macro to create a vector of strings from a list of string literals.
#[macro_export]
macro_rules! strings {
    ($($s:expr),* $(,)?) => {
        vec![$($s.to_string()),*]
    };
}

#[macro_export]
macro_rules! string_vec_map {
    (
        $(
            $key:expr => [$($val:expr),* $(,)?]
        ),* $(,)?
    ) => {{
        let mut map = std::collections::BTreeMap::new();
        $(
            let vec = vec![$($val.to_string()),*];
            map.insert($key.to_string(), vec);
        )*
        map
    }};
}

#[cfg(test)]
mod tests {

    #[test]
    fn strings_macro_works_as_expected() {
        let result = strings!["hello", "world", "123"];
        let expected = vec!["hello".to_string(), "world".to_string(), "123".to_string()];
        assert_eq!(result, expected);
    }

    #[test]
    fn string_vec_map_macro_works_as_expected() {
        let result = string_vec_map! {
            "a" => ["1", "2"],
            "b" => ["3", "4"]
        };
        let mut expected = std::collections::BTreeMap::new();
        expected.insert("a".to_string(), vec!["1".to_string(), "2".to_string()]);
        expected.insert("b".to_string(), vec!["3".to_string(), "4".to_string()]);
        assert_eq!(result, expected);
    }
}
