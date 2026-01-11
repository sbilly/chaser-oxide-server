//! Chaser-Oxide: Rust-based browser automation microservice
//!
//! This library provides a gRPC server for browser automation using Chrome DevTools Protocol.

pub mod error;
pub mod config;

pub mod cdp;
pub mod session;
pub mod services;
pub mod stealth;

// Re-exports
pub use error::{Error, Result};

// Generated protobuf modules
#[allow(clippy::large_enum_variant)]
pub mod chaser_oxide {
    pub mod v1 {
        tonic::include_proto!("chaser.oxide.v1");
    }
}

/// Chaser-Oxide library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
