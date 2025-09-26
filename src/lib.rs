#![forbid(unsafe_code)]

// Library root: expose modules for binaries & tests
pub mod domain;
pub mod infra;
pub mod ui;

use std::path::PathBuf;

/// Minimal, crate-visible representation of CLI args used by the library.
/// Le binaire est responsable de construire cette struct à partir du type clap local.
#[derive(Debug, Clone)]
pub struct CliArgs {
    pub focus: Option<u64>,
    pub short: Option<u64>,
    pub long: Option<u64>,
    pub cycles: Option<u8>,
    pub task: Option<String>,
    pub preset: Option<PathBuf>,
}

// Re-export convenient types commonly used by binaries/tests
pub use domain::config::Config;
pub use domain::schedule::Schedule;
pub use domain::session::SessionRunner;
pub use infra::storage::Journal;
