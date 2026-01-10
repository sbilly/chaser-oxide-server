//! Mock Chrome DevTools Protocol server
//!
//! This module provides a mock Chrome server for testing without requiring a real Chrome instance.

use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};

/// Mock Chrome server
pub struct MockChromeServer {
    addr: String,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl MockChromeServer {
    /// Start a new mock Chrome server
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let ws_addr = format!("ws://{}", addr);

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        tokio::spawn(async move {
            let mut connection_id = 0;

            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, peer_addr)) => {
                                tracing::info!("Mock Chrome: Connection from {}", peer_addr);
                                tokio::spawn(Self::handle_connection(stream, connection_id));
                                connection_id += 1;
                            }
                            Err(e) => {
                                tracing::error!("Mock Chrome: Accept error: {}", e);
                                break;
                            }
                        }
                    }
                    _ = &mut shutdown_rx => {
                        tracing::info!("Mock Chrome: Shutdown signal received");
                        break;
                    }
                }
            }
        });

        Ok(Self {
            addr: ws_addr,
            shutdown_tx: Some(shutdown_tx),
        })
    }

    /// Handle a WebSocket connection
    async fn handle_connection(stream: TcpStream, connection_id: u32) {
        match accept_async(stream).await {
            Ok(ws_stream) => {
                let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                // Handle incoming messages
                while let Some(result) = ws_receiver.next().await {
                    match result {
                        Ok(Message::Text(text)) => {
                            if let Ok(req) = serde_json::from_str::<Value>(&text) {
                                let response = Self::create_cdp_response(&req);
                                if let Ok(resp_text) = serde_json::to_string(&response) {
                                    if ws_sender.send(Message::Text(resp_text)).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                        Ok(Message::Close(_)) => {
                            tracing::debug!("Mock Chrome: Connection {} closed", connection_id);
                            break;
                        }
                        Err(e) => {
                            tracing::error!("Mock Chrome: WebSocket error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                tracing::error!("Mock Chrome: WebSocket handshake error: {}", e);
            }
        }
    }

    /// Create a CDP response for a request
    fn create_cdp_response(req: &Value) -> Value {
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("unknown");

        let id = req.get("id").and_then(|i| i.as_i64()).unwrap_or(0);

        match method {
            "Page.enable" => json!({
                "id": id,
                "result": {}
            }),
            "Runtime.enable" => json!({
                "id": id,
                "result": {}
            }),
            "Network.enable" => json!({
                "id": id,
                "result": {}
            }),
            "DOM.enable" => json!({
                "id": id,
                "result": {}
            }),
            "Page.navigate" => json!({
                "id": id,
                "result": {
                    "frameId": "test-frame",
                    "loaderId": "test-loader"
                }
            }),
            "Runtime.evaluate" => {
                let result = req.get("params")
                    .and_then(|p| p.get("expression"))
                    .and_then(|e| e.as_str());

                if let Some(expr) = result {
                    if expr.contains("document.querySelector") {
                        json!({
                            "id": id,
                            "result": {
                                "result": {
                                    "type": "object",
                                    "objectId": "test-element-id"
                                }
                            }
                        })
                    } else {
                        json!({
                            "id": id,
                            "result": {
                                "result": {
                                    "type": "string",
                                    "value": "test-result"
                                }
                            }
                        })
                    }
                } else {
                    json!({
                        "id": id,
                        "error": {
                            "code": -32000,
                            "message": "Invalid expression"
                        }
                    })
                }
            }
            "DOM.querySelector" => json!({
                "id": id,
                "result": {
                    "nodeId": 1
                }
            }),
            "DOM.describeNode" => json!({
                "id": id,
                "result": {
                    "node": {
                        "nodeId": 1,
                        "nodeName": "DIV",
                        "nodeType": 1,
                        "attributes": []
                    }
                }
            }),
            "Page.captureScreenshot" => json!({
                "id": id,
                "result": {
                    "data": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
                }
            }),
            _ => json!({
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method not implemented: {}", method)
                }
            }),
        }
    }

    /// Get the WebSocket endpoint URL
    pub fn ws_endpoint(&self) -> &str {
        &self.addr
    }

    /// Get the HTTP endpoint for version info
    pub fn http_endpoint(&self) -> String {
        self.addr.replace("ws://", "http://")
    }

    /// Get version info (for testing)
    pub fn get_version_info() -> Value {
        json!({
            "Browser": "Chrome/120.0.6099.109",
            "Protocol-Version": "1.3",
            "User-Agent": "Mozilla/5.0 (Test)",
            "WebKit-Version": "537.36"
        })
    }

    /// Get target list (for testing)
    pub fn get_targets() -> Value {
        json!([
            {
                "id": "test-target-1",
                "type": "page",
                "title": "Test Page",
                "url": "about:blank",
                "attached": false
            }
        ])
    }
}

impl Drop for MockChromeServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_chrome_startup() {
        let server = MockChromeServer::start().await.unwrap();
        assert!(server.ws_endpoint().starts_with("ws://127.0.0.1:"));
    }

    #[tokio::test]
    async fn test_version_response() {
        let version = MockChromeServer::get_version_info();
        assert_eq!(version["Browser"], "Chrome/120.0.6099.109");
    }

    #[tokio::test]
    async fn test_targets_response() {
        let targets = MockChromeServer::get_targets();
        assert!(targets.as_array().unwrap().len() > 0);
    }
}
