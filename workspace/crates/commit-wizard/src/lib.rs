//! # Commit Wizard
//!
//! **A spellbook for your commits.** `commit-wizard` installs a command-line tool, `cw`, for
//! [Conventional Commits](https://www.conventionalcommits.org/), semantic versioning and
//! changelog automation.
//!
//! ```text
//! cargo install commit-wizard --locked
//! cw init        # guided setup
//! cw commit      # write a Conventional Commit, step by step
//! cw check       # validate history; exits non-zero if any commit breaks the rules
//! cw tag         # work out the next version and tag it
//! cw changelog   # write the changelog from your tags
//! ```
//!
//! The user manual, with every command and the full configuration schema, is at
//! <https://commit-wizard.kitechsoftware.com>.
//!
//! ## This is a tool, not a library
//!
//! The code is published as a library crate only so that the `cw` binary, its tests and
//! docs.rs can share it. The modules below are an implementation detail and carry **no
//! stability guarantee**: they may change in any release, patch releases included. If you want
//! to build on `cw`, depend on its command line and its configuration files, not on this API.
//!
//! ## How the code is organised
//!
//! For contributors, the crate is layered from the outside in:
//!
//! - [`cli`]: argument parsing and command dispatch (`cw <command>`)
//! - [`core`]: the use cases behind each command, and the context they run in
//! - [`engine`]: models, configuration, validation rules and other logic with no I/O
//! - [`infra`]: Git, terminal and file-system adapters
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
