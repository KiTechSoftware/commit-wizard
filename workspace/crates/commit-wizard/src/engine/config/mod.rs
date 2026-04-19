//! Configuration file parsing and resolution.
//!
//! This module handles loading, parsing, and resolving `.cwizard.toml` configuration files.
//! It supports base configuration, environment variable overrides, registry integration,
//! and custom rules.
//!
//! ## Submodules
//!
//! - [`base`]: Base configuration structure
//! - [`schema`]: Configuration schema definitions
//! - [`resolver`]: Configuration resolution logic
//! - [`rules`]: Custom commit rules
//! - [`registry`]: Registry configuration
//! - [`env`]: Environment variable handling

pub mod base;
pub mod env;
pub mod registry;
pub mod resolver;
pub mod rules;
pub mod schema;

pub use base::BaseConfig;
pub use rules::RulesConfig;
pub use schema::*;
