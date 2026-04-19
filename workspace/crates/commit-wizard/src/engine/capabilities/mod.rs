//! High-level capabilities and orchestration.
//!
//! This module provides capability interfaces that combine models, traits, and business logic
//! to implement complete workflows. Capabilities are the primary interface for use cases.
//!
//! ## Submodules
//!
//! - [`commit`]: Commit message validation and creation
//! - [`config`]: Configuration management capabilities
//! - [`usage`]: Usage tracking and reporting
//! - [`versioning`]: Version bumping and tag management

pub mod commit;
pub mod config;
pub mod usage;
pub mod versioning;
