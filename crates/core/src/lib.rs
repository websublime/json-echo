#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

mod config;
mod database;
mod errors;
mod filesystem;

/// Version of the crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the version of the crate
#[must_use]
pub fn version() -> &'static str {
    VERSION
}

pub use config::{Config, ConfigManager, ConfigResponse, ConfigRoute, ConfigRouteResponse};
pub use database::{Database, Model};
pub use errors::{Error, FileSystemError, FileSystemResult};
pub use filesystem::{FileSystemManager, PathUtils};
