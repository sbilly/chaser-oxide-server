//! Protocol buffer utilities for gRPC services
//!
//! Provides macros and types for consistent protobuf message handling.

use crate::Error;

/// Type alias for service results
pub type ServiceResult<T> = Result<T, Error>;

/// Macro to handle service results with consistent error formatting
#[macro_export]
macro_rules! handle_service_result {
    ($expr:expr) => {
        match $expr {
            Ok(value) => ::tonic::Response::new(value),
            Err(e) => return Err(::tonic::Status::internal(e.to_string())),
        }
    };
}

/// Macro to create protobuf error responses
#[macro_export]
macro_rules! proto_error {
    ($error:expr) => {{
        use $crate::services::common::error::ToProtoError;
        $error.to_proto_error()
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::common::error::ToProtoError;

    #[test]
    fn test_proto_error_macro() {
        let error = Error::Configuration("test error".to_string());
        let proto_err = error.to_proto_error();

        assert_eq!(proto_err.message, "test error");
    }
}
