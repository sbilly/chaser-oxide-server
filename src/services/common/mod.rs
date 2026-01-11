//! Common utilities for gRPC service implementations
//!
//! This module provides shared functions, macros, and utilities for all gRPC services
//! to reduce code duplication and ensure consistent error handling.

pub mod error;
pub mod proto;

pub use error::to_response;
pub use proto::ServiceResult;
