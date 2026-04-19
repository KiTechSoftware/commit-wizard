//! Business logic layer with traits, models, and capabilities.
//!
//! This module encapsulates the core business rules for handling commits, versioning,
//! configuration, and related concerns. It abstracts away infrastructure details and
//! provides high-level capabilities for the application.
//!
//! ## Submodules
//!
//! - [`traits`]: Trait definitions for Git, UI, logging, and prompts (abstraction layer)
//! - [`models`]: Data models for commits, versions, policies, configurations, and runtime state
//! - [`capabilities`]: High-level capabilities built on top of models and traits
//! - [`config`]: Configuration file parsing and resolution
//! - [`constants`]: Application constants and default values
//! - [`fs`]: File system operations and abstractions
//! - [`error`]: Engine-specific error types
//!
//! ## Architecture
//!
//! This layer follows a hexagonal architecture pattern:
//! - **Traits** define boundaries (Git, UI, Logger)
//! - **Models** represent domain concepts
//! - **Capabilities** orchestrate operations using traits and models
//! - Infrastructure implementations in [`crate::infra`] satisfy the trait contracts

pub mod capabilities;
pub mod config;
pub mod constants;
pub mod error;
pub mod fs;
pub mod models;
pub mod traits;

pub use error::*;
pub use traits::*;
