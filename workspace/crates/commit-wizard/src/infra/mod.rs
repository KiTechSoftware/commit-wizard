//! Infrastructure layer providing implementations for external integrations.
//!
//! This module contains concrete implementations of traits defined in [`crate::engine::traits`],
//! including Git operations, user interface interactions, and other infrastructure concerns.
//!
//! ## Submodules
//!
//! - [`git`]: Git repository operations (implements `GitTrait`)
//! - [`ui`]: User interface interactions (implements UI traits for prompts and output)
//!
//! This layer is kept separate from business logic, allowing for easy testing
//! and mocking of external dependencies.

pub mod git;
pub mod ui;
