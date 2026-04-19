//! Domain models representing core concepts.
//!
//! This module contains data structures representing commits, versions, policies, Git state,
//! runtime configuration, and other domain entities. Models are typically immutable and
//! derived from configuration or user input.
//!
//! ## Submodules
//!
//! - [`git`]: Git repository and commit models
//! - [`policy`]: Configuration policies (commit rules, versioning, AI, release, etc.)
//! - [`runtime`]: Runtime state and configuration resolution
//! - [`state`]: State machines and workflow state

pub mod git;
pub mod policy;
pub mod runtime;
pub mod state;
