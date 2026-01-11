//! Unified error conversion utilities for gRPC services
//!
//! Provides functions and traits for converting domain errors to tonic responses.

use crate::error::Error;

/// Convert Result to tonic Response with error handling
#[allow(clippy::result_large_err)]
pub fn to_response<T>(
    result: Result<T, Error>,
) -> Result<tonic::Response<T>, tonic::Status> {
    result.map(tonic::Response::new)
        .map_err(|e| tonic::Status::internal(e.to_string()))
}

/// Trait for converting errors to protobuf error format
pub trait ToProtoError {
    /// Convert error to protobuf error representation
    fn to_proto_error(&self) -> ProtoError;
}

/// Protobuf error representation
#[derive(Debug, Clone)]
pub struct ProtoError {
    pub message: String,
}

impl ToProtoError for Error {
    fn to_proto_error(&self) -> ProtoError {
        ProtoError {
            message: self.to_string(),
        }
    }
}
