//! CDP (Chrome DevTools Protocol) type definitions
//!
//! This module defines the core data structures for CDP communication.

use serde::{Deserialize, Serialize};

/// CDP JSON-RPC request
#[derive(Debug, Clone, Serialize)]
pub struct CdpRequest {
    /// Request ID
    pub id: u64,
    /// Method name (e.g., "Page.navigate")
    pub method: String,
    /// Method parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    /// Session ID for multi-session targets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// CDP JSON-RPC notification (event)
#[derive(Debug, Clone, Deserialize)]
pub struct CdpNotification {
    /// Event method (e.g., "Page.loadEventFired")
    pub method: String,
    /// Event parameters
    #[serde(default)]
    pub params: serde_json::Value,
    /// Session ID for multi-session targets
    #[serde(default)]
    pub session_id: Option<String>,
}

/// CDP JSON-RPC response
#[derive(Debug, Clone, Deserialize)]
pub struct CdpRpcResponse {
    /// Response ID (matches request ID)
    pub id: u64,
    /// Response result
    #[serde(default)]
    pub result: serde_json::Value,
    /// Error if any
    #[serde(default)]
    pub error: Option<CdpErrorDetail>,
}

/// CDP error detail
#[derive(Debug, Clone, Deserialize)]
pub struct CdpErrorDetail {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Additional error data
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

/// CDP message (union of request, response, and notification)
#[derive(Debug, Clone)]
pub enum CdpMessage {
    /// Request (client -> server)
    Request(CdpRequest),
    /// Response (server -> client)
    Response(CdpRpcResponse),
    /// Notification/Event (server -> client)
    Notification(CdpNotification),
}

/// Page navigation parameters
#[derive(Debug, Clone, Serialize)]
pub struct NavigateParams {
    /// URL to navigate to
    pub url: String,
    /// Referrer URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
    /// Transition type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition_type: Option<String>,
}

/// JavaScript evaluation parameters
#[derive(Debug, Clone, Serialize)]
pub struct EvaluateParams {
    /// JavaScript expression to evaluate
    pub expression: String,
    /// Whether to await promise
    #[serde(skip_serializing_if = "Option::is_none", rename = "awaitPromise")]
    pub await_promise: Option<bool>,
    /// Whether to return as value
    #[serde(skip_serializing_if = "Option::is_none", rename = "returnByValue")]
    pub return_by_value: Option<bool>,
    /// Execution context ID
    #[serde(skip_serializing_if = "Option::is_none", rename = "contextId")]
    pub context_id: Option<i64>,
}

/// Screenshot parameters
#[derive(Debug, Clone, Serialize)]
pub struct ScreenshotParams {
    /// Image format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// JPEG quality (0-100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<u8>,
    /// Clip to viewport
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clip: Option<Clip>,
}

/// Clip region for screenshot
#[derive(Debug, Clone, Serialize)]
pub struct Clip {
    /// X offset
    pub x: f64,
    /// Y offset
    pub y: f64,
    /// Width
    pub width: f64,
    /// Height
    pub height: f64,
    /// Page scale factor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
}

/// Remote object (result of JavaScript evaluation)
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RemoteObject {
    /// Object type
    #[serde(default)]
    pub r#type: String,
    /// Object subtype
    #[serde(default)]
    pub subtype: Option<String>,
    /// Object value
    #[serde(default)]
    pub value: Option<serde_json::Value>,
    /// Object description
    #[serde(default)]
    pub description: Option<String>,
    /// Unserializable value
    #[serde(rename = "unserializableValue", default)]
    pub unserializable_value: Option<String>,
}

/// Exception details
#[derive(Debug, Clone, Deserialize)]
pub struct ExceptionDetails {
    /// Exception ID
    pub exception_id: i32,
    /// Exception text
    pub text: Option<String>,
    /// Line number
    pub line_number: i32,
    /// Column number
    pub column_number: i32,
    /// Exception object
    #[serde(default)]
    pub exception: Option<RemoteObject>,
}

/// JavaScript evaluation response
#[derive(Debug, Clone, Deserialize)]
pub struct EvaluateResponse {
    /// Evaluation result
    #[serde(default)]
    pub result: RemoteObject,
    /// Exception details if evaluation failed
    #[serde(default)]
    pub exception_details: Option<ExceptionDetails>,
}

/// Document node
#[derive(Debug, Clone, Deserialize)]
pub struct Node {
    /// Node ID
    pub node_id: i32,
    /// Backend ID
    #[serde(default)]
    pub backend_node_id: i32,
    /// Node type
    #[serde(default)]
    pub node_type: i32,
    /// Node name
    #[serde(default)]
    pub node_name: String,
    /// Local name
    #[serde(default)]
    pub local_name: String,
    /// Node value
    #[serde(default)]
    pub node_value: String,
    /// Child node count
    #[serde(default)]
    pub child_node_count: i32,
    /// Children
    #[serde(default)]
    pub children: Option<Vec<Node>>,
    /// Attributes
    #[serde(default)]
    pub attributes: Option<Vec<String>>,
}

/// Get document response
#[derive(Debug, Clone, Deserialize)]
pub struct GetDocumentResponse {
    /// Root node
    pub root: Node,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdp_request_serialization() {
        let request = CdpRequest {
            id: 1,
            method: "Page.navigate".to_string(),
            params: Some(serde_json::json!({ "url": "https://example.com" })),
            session_id: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"method\":\"Page.navigate\""));
    }

    #[test]
    fn test_cdp_request_without_params() {
        let request = CdpRequest {
            id: 2,
            method: "Page.enable".to_string(),
            params: None,
            session_id: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        // params should not be serialized when None
        assert!(!json.contains("\"params\""));
    }
}
