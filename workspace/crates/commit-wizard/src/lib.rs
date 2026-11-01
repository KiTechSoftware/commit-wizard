#![doc = include_str!("../README.md")]
//!
//! ---
//!
//! ## This is a tool, not a library
//!
//! The code is published as a library crate only so that the `cw` binary, its tests and
//! docs.rs can share it. Everything except [`cli::run`] is hidden from these docs and carries
//! **no stability guarantee**: it may change in any release, patch releases included. If you
//! want to build on `cw`, depend on its command line and its configuration files, not on this
//! API.
//!
//! ## Contributing to the code
//!
//! The crate is layered from the outside in: `cli` (argument parsing and dispatch), `core`
//! (the use case behind each command), `engine` (models, configuration and rules, with no I/O)
//! and `infra` (Git, terminal and file-system adapters).
//! [ARCHITECTURE.md](https://github.com/KiTechSoftware/commit-wizard/blob/main/ARCHITECTURE.md)
//! walks through how a command flows through them and how to add one, and
//! `cargo doc --document-private-items --open` shows the hidden modules.
//!
//! ## What `cw` enforces
//!
//! Commits follow the Conventional Commits layout:
//!
//! ```text
//! <type>(<scope>): <description>
//!
//! <body>
//!
//! <footer>
//! ```
//!
//! The allowed types and scopes come from your configuration (the standard profile ships
//! `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test` and `chore`; a shared registry or
//! your own config can add more). Version bumps follow [Semantic Versioning](https://semver.org/):
//!
//! - **major**: a breaking change (`feat!:` or a `BREAKING CHANGE:` footer)
//! - **minor**: a new feature (`feat`)
//! - **patch**: a fix (`fix`) and other patch-level types

#![warn(missing_docs)]

pub mod cli;

// Implementation detail: hidden from the API docs and not covered by any stability promise.
#[doc(hidden)]
pub mod core;
#[doc(hidden)]
pub mod engine;
#[doc(hidden)]
pub mod infra;

/// A macro to create a vector of strings from a list of string literals.
#[doc(hidden)]
#[macro_export]
macro_rules! strings {
    ($($s:expr),* $(,)?) => {
        vec![$($s.to_string()),*]
    };
}

#[doc(hidden)]
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
