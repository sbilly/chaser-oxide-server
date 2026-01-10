//! Unified error types for Chaser-Oxide

use std::net;
use thiserror::Error;

/// Unified Result type
pub type Result<T> = std::result::Result<T, Error>;

/// Unified error type for Chaser-Oxide
#[derive(Error, Debug)]
pub enum Error {
    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Network errors
    #[error("Network error: {0}")]
    Net(#[from] net::AddrParseError),

    /// WebSocket errors
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// CDP protocol errors
    #[error("CDP error: {0}")]
    Cdp(String),

    /// gRPC errors
    #[error("gRPC error: {0}")]
    Grpc(#[from] Box<tonic::Status>),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Session not found
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    /// Browser not found
    #[error("Browser not found: {0}")]
    BrowserNotFound(String),

    /// Page not found
    #[error("Page not found: {0}")]
    PageNotFound(String),

    /// Element not found
    #[error("Element not found: {0}")]
    ElementNotFound(String),

    /// Timeout
    #[error("Operation timeout: {0}")]
    Timeout(String),

    /// Navigation failed
    #[error("Navigation failed: {0}")]
    NavigationFailed(String),

    /// Script execution failed
    #[error("Script execution failed: {0}")]
    ScriptExecutionFailed(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl Error {
    /// Create a new WebSocket error
    pub fn websocket<S: Into<String>>(msg: S) -> Self {
        Error::WebSocket(msg.into())
    }

    /// Create a new CDP error
    pub fn cdp<S: Into<String>>(msg: S) -> Self {
        Error::Cdp(msg.into())
    }

    /// Create a new session not found error
    pub fn session_not_found<S: Into<String>>(id: S) -> Self {
        Error::SessionNotFound(id.into())
    }

    /// Create a new browser not found error
    pub fn browser_not_found<S: Into<String>>(id: S) -> Self {
        Error::BrowserNotFound(id.into())
    }

    /// Create a new page not found error
    pub fn page_not_found<S: Into<String>>(id: S) -> Self {
        Error::PageNotFound(id.into())
    }

    /// Create a new element not found error
    pub fn element_not_found<S: Into<String>>(id: S) -> Self {
        Error::ElementNotFound(id.into())
    }

    /// Create a new timeout error
    pub fn timeout<S: Into<String>>(msg: S) -> Self {
        Error::Timeout(msg.into())
    }

    /// Create a new navigation failed error
    pub fn navigation_failed<S: Into<String>>(msg: S) -> Self {
        Error::NavigationFailed(msg.into())
    }

    /// Create a new script execution failed error
    pub fn script_execution_failed<S: Into<String>>(msg: S) -> Self {
        Error::ScriptExecutionFailed(msg.into())
    }

    /// Create a new configuration error
    pub fn configuration<S: Into<String>>(msg: S) -> Self {
        Error::Configuration(msg.into())
    }

    /// Create a new internal error
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Error::Internal(msg.into())
    }
}

/// Convert Error to gRPC Status
impl From<Error> for tonic::Status {
    fn from(err: Error) -> Self {
        match err {
            Error::SessionNotFound(_) | Error::BrowserNotFound(_) | Error::PageNotFound(_) => {
                tonic::Status::not_found(err.to_string())
            }
            Error::Timeout(_) => tonic::Status::deadline_exceeded(err.to_string()),
            Error::NavigationFailed(_) | Error::ScriptExecutionFailed(_) => {
                tonic::Status::aborted(err.to_string())
            }
            Error::Configuration(_) => tonic::Status::invalid_argument(err.to_string()),
            Error::Grpc(status) => *status,
            _ => tonic::Status::internal(err.to_string()),
        }
    }
}
