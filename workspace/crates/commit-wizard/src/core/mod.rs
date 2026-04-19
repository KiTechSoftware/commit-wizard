//! Core application logic and context management.
//!
//! This module provides the foundational abstractions for the application, including
//! the [`Context`] which holds application state, error handling, and use case definitions.
//!
//! ## Submodules
//!
//! - [`bootstrap`]: Application initialization and context building
//! - [`context`]: Core [`Context`] type and state management
//! - [`error`]: Error types and conversion utilities
//! - [`usecases`]: Re-exported use cases (operations) available in the application
//!
//! The flow is typically:
//! 1. [`bootstrap::build_app_context()`] creates the [`Context`]
//! 2. Use cases are executed via context methods
//! 3. Errors are reported using [`report_error()`]

pub mod bootstrap;
pub mod context;
pub mod error;
pub mod usecases;

pub use context::Context;
pub use error::CoreResult;
pub use error::exit_code;
pub use error::report_error;

pub use usecases::*;
