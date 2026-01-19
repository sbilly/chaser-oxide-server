//! Chaser-Oxide: Rust-based browser automation microservice

// Core modules
pub mod error;
pub mod config;

// Functional modules
pub mod cdp;
pub mod process;
pub mod session;
pub mod services;
pub mod stealth;

// Re-exports
pub use error::{Error, Result};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Generated protobuf modules
#[allow(clippy::large_enum_variant)]
pub mod chaser_oxide {
    pub mod v1 {
        tonic::include_proto!("chaser.oxide.v1");
    }
}
