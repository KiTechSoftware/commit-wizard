//! Trait definitions for external integrations (abstraction layer).
//!
//! This module defines traits that abstract away external dependencies like Git operations,
//! user interface interactions, and logging. Concrete implementations are provided by
//! the [`crate::infra`] layer.
//!
//! ## Submodules
//!
//! - `git`: Git repository and CLI operations
//! - [`logger`]: Logging trait
//! - [`prompt`]: User prompting and interaction

pub mod logger;
pub mod prompt;

pub use logger::LoggerTrait;
pub use prompt::PromptTrait;
