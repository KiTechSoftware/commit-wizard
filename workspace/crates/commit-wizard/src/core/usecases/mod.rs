//! One module per command family. Each use case reads the `Context`, calls `engine` for the rules and `infra` for Git and the terminal, and prints a result.

pub mod branch;
pub mod changelog;
pub mod commit;
pub mod configs;
pub mod doctor;
pub mod usage;
pub mod version;
