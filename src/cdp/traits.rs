//! CDP (Chrome DevTools Protocol) layer traits
//!
//! This module defines the abstract interfaces for CDP communication.

use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

/// CDP event representation
#[derive(Debug, Clone)]
pub struct CdpEvent {
    /// Event method (e.g., "Page.loadEventFired")
    pub method: String,
    /// Event parameters
    pub params: Value,
    /// Session ID (for multi-session targets)
    pub session_id: Option<String>,
}

/// CDP response representation
#[derive(Debug, Clone)]
pub struct CdpResponse {
    /// Response ID (matches request ID)
    pub id: u64,
    /// Response result
    pub result: Option<Value>,
    /// Error if any
    pub error: Option<CdpError>,
}

/// CDP error representation
#[derive(Debug, Clone)]
pub struct CdpError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Additional error data
    pub data: Option<Value>,
}

/// CDP connection trait
///
/// Represents a WebSocket connection to a Chrome DevTools Protocol target.
#[async_trait]
pub trait CdpConnection: Send + Sync + std::fmt::Debug {
    /// Send a CDP command and wait for response
    async fn send_command(
        &self,
        method: &str,
        params: Value,
    ) -> Result<CdpResponse, crate::Error>;

    /// Subscribe to CDP events
    async fn listen_events(&self) -> Result<tokio::sync::mpsc::Receiver<CdpEvent>, crate::Error>;

    /// Close the connection
    async fn close(&self) -> Result<(), crate::Error>;

    /// Check if connection is active
    fn is_active(&self) -> bool;
}

/// CDP client trait
///
/// High-level CDP client that provides typed methods for common CDP operations.
#[async_trait]
pub trait CdpClient: Send + Sync + std::fmt::Debug {
    /// Get the underlying connection
    fn connection(&self) -> Arc<dyn CdpConnection>;

    /// Navigate to a URL
    async fn navigate(&self, url: &str) -> Result<NavigationResult, crate::Error>;

    /// Evaluate JavaScript in the page
    async fn evaluate(&self, script: &str, await_promise: bool) -> Result<EvaluationResult, crate::Error>;

    /// Capture a screenshot
    async fn screenshot(&self, format: ScreenshotFormat) -> Result<Vec<u8>, crate::Error>;

    /// Get page content
    async fn get_content(&self) -> Result<String, crate::Error>;

    /// Set page content
    async fn set_content(&self, html: &str) -> Result<(), crate::Error>;

    /// Reload the page
    async fn reload(&self, ignore_cache: bool) -> Result<(), crate::Error>;

    /// Enable a domain
    async fn enable_domain(&self, domain: &str) -> Result<(), crate::Error>;

    /// Call a raw CDP method (returns JSON Value)
    async fn call_method(&self, method: &str, params: Value) -> Result<Value, crate::Error>;

    /// Subscribe to events (returns a receiver)
    async fn subscribe_events(&self, event_type: &str) -> Result<tokio::sync::mpsc::Receiver<CdpEvent>, crate::Error>;
}

/// Navigation result
#[derive(Debug, Clone)]
pub struct NavigationResult {
    /// Navigation ID
    pub navigation_id: Option<String>,
    /// URL after navigation
    pub url: String,
    /// HTTP status code
    pub status_code: u16,
}

/// JavaScript evaluation result
#[derive(Debug, Clone)]
pub enum EvaluationResult {
    /// String value
    String(String),
    /// Number value
    Number(f64),
    /// Boolean value
    Bool(bool),
    /// Null value
    Null,
    /// Object/Array (as JSON)
    Object(Value),
}

/// Screenshot format
#[derive(Debug, Clone, Copy)]
pub enum ScreenshotFormat {
    /// PNG format
    Png,
    /// JPEG format
    Jpeg(u8), // quality 0-100
    /// WebP format
    WebP(u8), // quality 0-100
}

/// CDP browser trait
///
/// Controls browser-level operations via CDP.
#[async_trait]
pub trait CdpBrowser: Send + Sync + std::fmt::Debug {
    /// Create a new CDP client for a browser context
    async fn create_client(&self, target_url: &str) -> Result<Arc<dyn CdpClient>, crate::Error>;

    /// Close the browser
    async fn close(&self) -> Result<(), crate::Error>;

    /// Get browser version
    async fn get_version(&self) -> Result<BrowserVersion, crate::Error>;

    /// List all targets (pages, workers, etc.)
    async fn get_targets(&self) -> Result<Vec<TargetInfo>, crate::Error>;

    /// Create a new browser target (page) using CDP Target.createTarget
    ///
    /// Returns the WebSocket URL of the newly created target.
    async fn create_target(&self, url: &str) -> Result<String, crate::Error>;
}

/// Browser version information
#[derive(Debug, Clone)]
pub struct BrowserVersion {
    /// Protocol version
    pub protocol_version: String,
    /// Product name
    pub product: String,
    /// User agent
    pub user_agent: String,
    /// JavaScript engine version
    pub js_version: String,
}

/// Target information (page, worker, etc.)
#[derive(Debug, Clone)]
pub struct TargetInfo {
    /// Target ID
    pub target_id: String,
    /// Target type
    pub target_type: String,
    /// Target title
    pub title: String,
    /// Target URL
    pub url: String,
    /// Whether target can be attached to
    pub attached: bool,
}
