//! CDP layer integration tests
//!
//! This module contains integration tests for the CDP layer.
//!
//! Note: These tests require a running Chrome/Chromium instance with remote debugging enabled.
//! Start Chrome with: chrome --remote-debugging-port=9222

use super::connection::CdpWebSocketConnection;
use super::browser::CdpBrowserImpl;
use super::traits::*;
use crate::Error;

/// Test helper: Get Chrome debugging URL from environment or use default
fn get_chrome_url() -> String {
    std::env::var("CHROME_DEBUG_URL")
        .unwrap_or_else(|_| "ws://localhost:9222".to_string())
}

/// Test helper: Get test page URL
fn get_test_page_url() -> String {
    std::env::var("TEST_PAGE_URL")
        .unwrap_or_else(|_| "https://example.com".to_string())
}

/// Test helper: Check if Chrome is available
async fn is_chrome_available() -> bool {
    let url = get_chrome_url()
        .replace("ws://", "http://")
        .replace("wss://", "https://");

    if let Ok(client) = reqwest::Client::builder().build() {
        if let Ok(response) = client.get(&format!("{}/json/version", url)).send().await {
            return response.status().is_success();
        }
    }

    false
}

#[tokio::test]
async fn test_browser_get_version() {
    if !is_chrome_available().await {
        eprintln!("Skipping test: Chrome not available");
        return;
    }

    let browser = CdpBrowserImpl::new(get_chrome_url());

    let version = browser.get_version().await;

    assert!(version.is_ok(), "Failed to get browser version");

    let version = version.unwrap();
    assert!(!version.protocol_version.is_empty());
    assert!(!version.product.is_empty());
    assert!(!version.user_agent.is_empty());

    println!("Browser version: {:?}", version);
}

#[tokio::test]
async fn test_browser_get_targets() {
    if !is_chrome_available().await {
        eprintln!("Skipping test: Chrome not available");
        return;
    }

    let browser = CdpBrowserImpl::new(get_chrome_url());

    let targets = browser.get_targets().await;

    assert!(targets.is_ok(), "Failed to get targets");

    let targets = targets.unwrap();
    println!("Found {} targets", targets.len());

    // Print all targets
    for target in &targets {
        println!(
            "Target: {} ({}) - {}",
            target.target_type, target.title, target.url
        );
    }

    // Should have at least one target (the page)
    assert!(!targets.is_empty(), "No targets found");

    // Find at least one page target
    let page_targets: Vec<_> = targets
        .iter()
        .filter(|t| t.target_type == "page")
        .collect();

    assert!(!page_targets.is_empty(), "No page targets found");
}

#[tokio::test]
async fn test_websocket_connection() {
    if !is_chrome_available().await {
        eprintln!("Skipping test: Chrome not available");
        return;
    }

    let browser = CdpBrowserImpl::new(get_chrome_url());

    // Get targets
    let targets = browser.get_targets().await.unwrap();

    // Find first page target
    let page_target = targets
        .iter()
        .find(|t| t.target_type == "page")
        .expect("No page target found");

    // Get WebSocket URL for the target
    let http_url = get_chrome_url()
        .replace("ws://", "http://")
        .replace("wss://", "https://");

    let client = reqwest::Client::new();
    let response = client
        .get(&format!(
            "{}/json/list/{}",
            http_url, page_target.target_id
        ))
        .send()
        .await
        .expect("Failed to get target details");

    let target_info: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse target info");

    let ws_url = target_info
        .get("webSocketDebuggerUrl")
        .and_then(|v| v.as_str())
        .expect("No WebSocket URL in target info");

    // Connect to target
    let connection = CdpWebSocketConnection::new(ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    assert!(connection.is_active(), "Connection should be active");

    // Close connection
    connection
        .close()
        .await
        .expect("Failed to close connection");

    assert!(!connection.is_active(), "Connection should not be active after close");
}

#[tokio::test]
async fn test_cdp_send_command() {
    if !is_chrome_available().await {
        eprintln!("Skipping test: Chrome not available");
        return;
    }

    let browser = CdpBrowserImpl::new(get_chrome_url());

    // Create client
    let targets = browser.get_targets().await.unwrap();
    let page_target = targets
        .iter()
        .find(|t| t.target_type == "page")
        .expect("No page target found");

    let http_url = get_chrome_url()
        .replace("ws://", "http://")
        .replace("wss://", "https://");

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/json/list/{}", http_url, page_target.target_id))
        .send()
        .await
        .expect("Failed to get target details");

    let target_info: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse target info");

    let ws_url = target_info
        .get("webSocketDebuggerUrl")
        .and_then(|v| v.as_str())
        .expect("No WebSocket URL in target info");

    // Connect and create client
    let connection = CdpWebSocketConnection::new(ws_url)
        .await
        .expect("Failed to connect");

    let cdp_client = super::client::CdpClientImpl::new(connection);

    // Test Page.enable
    let result = cdp_client.enable_domain("Page").await;
    assert!(result.is_ok(), "Failed to enable Page domain");

    // Test Runtime.evaluate
    let eval_result = cdp_client
        .evaluate("1 + 1", false)
        .await
        .expect("Failed to evaluate JavaScript");

    match eval_result {
        EvaluationResult::Number(n) => {
            assert_eq!(n, 2.0, "Evaluation result should be 2");
        }
        _ => panic!("Expected Number result"),
    }

    // Test Page.navigate
    let nav_result = cdp_client
        .navigate(&get_test_page_url())
        .await
        .expect("Failed to navigate");

    assert_eq!(nav_result.url, get_test_page_url());

    // Wait a bit for navigation to complete
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
}

#[tokio::test]
async fn test_cdp_event_listening() {
    if !is_chrome_available().await {
        eprintln!("Skipping test: Chrome not available");
        return;
    }

    let browser = CdpBrowserImpl::new(get_chrome_url());

    // Create client
    let targets = browser.get_targets().await.unwrap();
    let page_target = targets
        .iter()
        .find(|t| t.target_type == "page")
        .expect("No page target found");

    let http_url = get_chrome_url()
        .replace("ws://", "http://")
        .replace("wss://", "https://");

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/json/list/{}", http_url, page_target.target_id))
        .send()
        .await
        .expect("Failed to get target details");

    let target_info: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse target info");

    let ws_url = target_info
        .get("webSocketDebuggerUrl")
        .and_then(|v| v.as_str())
        .expect("No WebSocket URL in target info");

    // Connect and create client
    let connection = CdpWebSocketConnection::new(ws_url)
        .await
        .expect("Failed to connect");

    let cdp_client = super::client::CdpClientImpl::new(connection.clone());

    // Subscribe to events
    let mut event_rx = cdp_client
        .subscribe_events("*")
        .await
        .expect("Failed to subscribe to events");

    // Trigger an event by navigating
    let client_clone = cdp_client.clone();
    tokio::spawn(async move {
        let _ = client_clone.navigate(&get_test_page_url()).await;
    });

    // Wait for event
    let event = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        event_rx.recv()
    )
    .await
    .expect("Timeout waiting for event")
    .expect("No event received");

    assert!(!event.method.is_empty(), "Event method should not be empty");

    println!("Successfully received event: {}", event.method);
}

#[tokio::test]
async fn test_cdp_screenshot() {
    if !is_chrome_available().await {
        eprintln!("Skipping test: Chrome not available");
        return;
    }

    let browser = CdpBrowserImpl::new(get_chrome_url());

    // Create client
    let targets = browser.get_targets().await.unwrap();
    let page_target = targets
        .iter()
        .find(|t| t.target_type == "page")
        .expect("No page target found");

    let http_url = get_chrome_url()
        .replace("ws://", "http://")
        .replace("wss://", "https://");

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/json/list/{}", http_url, page_target.target_id))
        .send()
        .await
        .expect("Failed to get target details");

    let target_info: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse target info");

    let ws_url = target_info
        .get("webSocketDebuggerUrl")
        .and_then(|v| v.as_str())
        .expect("No WebSocket URL in target info");

    // Connect and create client
    let connection = CdpWebSocketConnection::new(ws_url)
        .await
        .expect("Failed to connect");

    let cdp_client = super::client::CdpClientImpl::new(connection);

    // Navigate to test page
    let _ = cdp_client.navigate(&get_test_page_url()).await;

    // Wait for page to load
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Take screenshot
    let screenshot = cdp_client
        .screenshot(ScreenshotFormat::Png)
        .await
        .expect("Failed to capture screenshot");

    // Verify screenshot data
    assert!(!screenshot.is_empty(), "Screenshot should not be empty");
    assert!(screenshot.len() > 100, "Screenshot should contain data");

    // Verify PNG signature
    assert_eq!(&screenshot[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);

    println!("Screenshot captured successfully: {} bytes", screenshot.len());
}

#[tokio::test]
async fn test_cdp_get_content() {
    if !is_chrome_available().await {
        eprintln!("Skipping test: Chrome not available");
        return;
    }

    let browser = CdpBrowserImpl::new(get_chrome_url());

    // Create client
    let targets = browser.get_targets().await.unwrap();
    let page_target = targets
        .iter()
        .find(|t| t.target_type == "page")
        .expect("No page target found");

    let http_url = get_chrome_url()
        .replace("ws://", "http://")
        .replace("wss://", "https://");

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/json/list/{}", http_url, page_target.target_id))
        .send()
        .await
        .expect("Failed to get target details");

    let target_info: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse target info");

    let ws_url = target_info
        .get("webSocketDebuggerUrl")
        .and_then(|v| v.as_str())
        .expect("No WebSocket URL in target info");

    // Connect and create client
    let connection = CdpWebSocketConnection::new(ws_url)
        .await
        .expect("Failed to connect");

    let cdp_client = super::client::CdpClientImpl::new(connection);

    // Navigate to test page
    let _ = cdp_client.navigate(&get_test_page_url()).await;

    // Wait for page to load
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Get page content
    let content = cdp_client
        .get_content()
        .await
        .expect("Failed to get page content");

    assert!(!content.is_empty(), "Page content should not be empty");
    assert!(content.contains("<html"), "Content should contain HTML");

    println!("Page content retrieved successfully: {} bytes", content.len());
}

// Unit tests for type conversions
#[cfg(test)]
mod unit_tests {
    use super::super::types::*;

    #[test]
    fn test_evaluate_params_serialization() {
        let params = EvaluateParams {
            expression: "1 + 1".to_string(),
            await_promise: Some(false),
            return_by_value: Some(true),
            context_id: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("expression"));
        assert!(json.contains("awaitPromise"));
        assert!(json.contains("returnByValue"));
    }

    #[test]
    fn test_navigate_params_serialization() {
        let params = NavigateParams {
            url: "https://example.com".to_string(),
            referrer: None,
            transition_type: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"url\":\"https://example.com\""));
    }

    #[test]
    fn test_clip_serialization() {
        let clip = Clip {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
            scale: Some(1.0),
        };

        let json = serde_json::to_string(&clip).unwrap();
        assert!(json.contains("\"x\":0"));
        assert!(json.contains("\"width\":800"));
        assert!(json.contains("\"scale\":1"));
    }
}
