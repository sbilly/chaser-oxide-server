//! CDP browser control implementation
//!
//! This module provides browser-level operations via CDP.

use super::client::CdpClientImpl;
use super::connection::CdpWebSocketConnection;
use super::traits::*;
use crate::Error;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// CDP browser implementation
#[derive(Debug)]
pub struct CdpBrowserImpl {
    /// Browser WebSocket endpoint (e.g., "ws://localhost:9222")
    endpoint: String,
    /// Active connections (target_id -> connection)
    connections: Arc<tokio::sync::Mutex<std::collections::HashMap<String, Arc<dyn CdpConnection>>>>,
}

impl CdpBrowserImpl {
    /// Create a new CDP browser controller
    ///
    /// # Arguments
    /// * `endpoint` - Browser WebSocket endpoint (e.g., "ws://localhost:9222")
    pub fn new<S: Into<String>>(endpoint: S) -> Self {
        let endpoint_str = endpoint.into();
        info!("Creating CDP browser controller for endpoint: {}", endpoint_str);
        Self {
            endpoint: endpoint_str,
            connections: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Connect to browser and get version info
    async fn connect_browser(&self) -> Result<reqwest::Response, Error> {
        let http_endpoint = self.endpoint.replace("ws://", "http://").replace("wss://", "https://");

        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| Error::internal(format!("Failed to create HTTP client: {}", e)))?;

        let url = format!("{}/json/version", http_endpoint);

        debug!("Fetching browser version from {}", url);

        client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::internal(format!("Failed to connect to browser: {}", e)))
    }

    /// List all targets from browser
    async fn fetch_targets(&self) -> Result<Vec<TargetInfo>, Error> {
        let http_endpoint = self.endpoint.replace("ws://", "http://").replace("wss://", "https://");

        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| Error::internal(format!("Failed to create HTTP client: {}", e)))?;

        let url = format!("{}/json", http_endpoint);

        debug!("Fetching targets from {}", url);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::internal(format!("Failed to fetch targets: {}", e)))?;

        let targets_json: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| Error::internal(format!("Failed to parse targets: {}", e)))?;

        let mut targets = Vec::new();
        for target_json in targets_json {
            if let (Some(target_id), Some(target_type), Some(url)) = (
                target_json.get("id").and_then(|v| v.as_str()),
                target_json.get("type").and_then(|v| v.as_str()),
                target_json.get("url").and_then(|v| v.as_str()),
            ) {
                targets.push(TargetInfo {
                    target_id: target_id.to_string(),
                    target_type: target_type.to_string(),
                    title: target_json
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    url: url.to_string(),
                    attached: target_json
                        .get("attached")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                });
            }
        }

        Ok(targets)
    }
}

#[async_trait]
impl CdpBrowser for CdpBrowserImpl {
    /// Create a new CDP client for a browser context
    async fn create_client(&self, target_url: &str) -> Result<Arc<dyn CdpClient>, Error> {
        info!("Creating CDP client for target: {}", target_url);

        // Connect to target WebSocket
        let connection = CdpWebSocketConnection::new(target_url).await?;

        // Store connection
        let target_id = target_url
            .rsplit('/')
            .next()
            .unwrap_or("unknown")
            .to_string();

        let mut connections = self.connections.lock().await;
        connections.insert(target_id.clone(), Arc::clone(&connection) as Arc<dyn CdpConnection>);
        drop(connections);

        // Create client
        let client = Arc::new(CdpClientImpl::new(connection));

        // Enable essential domains only (Page and Runtime are always available)
        // Other domains (Network, DOM, etc.) should be enabled by the caller as needed
        client.enable_domain("Page").await?;
        client.enable_domain("Runtime").await?;

        Ok(client)
    }

    /// Close the browser
    async fn close(&self) -> Result<(), Error> {
        info!("CdpBrowser::close: Closing browser at endpoint {}", self.endpoint);

        let mut connections = self.connections.lock().await;
        let connection_count = connections.len();

        if connection_count == 0 {
            info!("CdpBrowser::close: No active connections to close");
            return Ok(());
        }

        info!("CdpBrowser::close: Closing {} active CDP connections", connection_count);

        let mut success_count = 0;
        let mut failed_targets = Vec::new();

        // Close all connections
        for (target_id, connection) in connections.iter() {
            debug!("CdpBrowser::close: Closing connection to target: {}", target_id);
            match connection.close().await {
                Ok(_) => {
                    success_count += 1;
                    debug!("CdpBrowser::close: Successfully closed connection to {}", target_id);
                }
                Err(e) => {
                    let tid = target_id.clone();
                    warn!("CdpBrowser::close: Failed to close connection to {}: {}", tid, e);
                    failed_targets.push((tid, e));
                }
            }
        }

        connections.clear();

        if !failed_targets.is_empty() {
            warn!("CdpBrowser::close: {} connections failed to close:", failed_targets.len());
            for (target_id, error) in &failed_targets {
                warn!("  - Target {}: {}", target_id, error);
            }
        }

        info!("CdpBrowser::close: Connection close summary: {} succeeded, {} failed",
            success_count, failed_targets.len());

        Ok(())
    }

    /// Get browser version
    async fn get_version(&self) -> Result<BrowserVersion, Error> {
        info!("Getting browser version");

        let response = self.connect_browser().await?;

        let version_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::internal(format!("Failed to parse version: {}", e)))?;

        Ok(BrowserVersion {
            protocol_version: version_json
                .get("Protocol-Version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            product: version_json
                .get("Browser")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            user_agent: version_json
                .get("User-Agent")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            js_version: version_json
                .get("WebKit-Version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
        })
    }

    /// List all targets (pages, workers, etc.)
    async fn get_targets(&self) -> Result<Vec<TargetInfo>, Error> {
        info!("Getting browser targets");

        self.fetch_targets().await
    }

    /// Create a new browser target (page) using Chrome HTTP API
    ///
    /// Uses the /json/new endpoint which creates a new page and returns its WebSocket URL directly.
    /// This is simpler than using Target.createTarget CDP command.
    async fn create_target(&self, url: &str) -> Result<String, Error> {
        info!("Creating new target with URL: {}", url);

        // Convert WebSocket endpoint to HTTP endpoint
        let http_endpoint = self.endpoint.replace("ws://", "http://").replace("wss://", "https://");

        // Use /json/new endpoint to create a new page
        let new_url = format!("{}/json/new?{}", http_endpoint, url);

        let http_client = reqwest::Client::builder()
            .build()
            .map_err(|e| Error::internal(format!("Failed to create HTTP client: {}", e)))?;

        debug!("Creating new page via HTTP API: {}", new_url);

        let response = http_client
            .put(&new_url)
            .send()
            .await
            .map_err(|e| {
                Error::internal(format!(
r#"Failed to connect to Chrome CDP endpoint at {}.
Please start Chrome with:
  macOS: /Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome --remote-debugging-port=9222 --user-data-dir=/tmp/chrome-debug
  Linux: google-chrome --remote-debugging-port=9222 --user-data-dir=/tmp/chrome-debug
  Windows: chrome.exe --remote-debugging-port=9222 --user-data-dir=C:\chrome-debug
Original error: {}"#,
                    self.endpoint, e
                ))
            })?;

        // Read response as text first for debugging
        let response_text = response
            .text()
            .await
            .map_err(|e| Error::internal(format!("Failed to read response: {}", e)))?;

        debug!("Response from /json/new: {}", response_text);

        // Parse the JSON response
        let target_json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| Error::internal(format!("Failed to parse new target response: {} (response was: {})", e, response_text)))?;

        let ws_url = target_json
            .get("webSocketDebuggerUrl")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::internal("No webSocketDebuggerUrl in new target response"))?;

        debug!("Created new target with WebSocket URL: {}", ws_url);

        Ok(ws_url.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_creation() {
        let browser = CdpBrowserImpl::new("ws://localhost:9222");
        assert_eq!(browser.endpoint, "ws://localhost:9222");
    }

    #[test]
    fn test_endpoint_conversion() {
        let browser = CdpBrowserImpl::new("wss://remote.example.com:9222");
        assert_eq!(browser.endpoint, "wss://remote.example.com:9222");
    }
}
