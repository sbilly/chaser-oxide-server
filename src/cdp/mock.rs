//! Mock CDP implementation for testing
//!
//! Provides mock implementations of CDP traits for development and testing.

use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::cdp::traits::*;
use crate::Error;

/// Minimal PNG header (1x1 transparent pixel)
const PNG_HEADER: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xDE,
];

/// Minimal JPEG header
const JPEG_HEADER: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0];

/// Minimal WebP header
const WEBP_HEADER: &[u8] = &[0x52, 0x49, 0x46, 0x46];

/// Mock CDP connection
#[derive(Debug)]
pub struct MockCdpConnection {
    is_active: Arc<AtomicBool>,
    next_id: AtomicU64,
}

impl MockCdpConnection {
    /// Create a new mock CDP connection
    pub fn new() -> Self {
        Self {
            is_active: Arc::new(AtomicBool::new(true)),
            next_id: AtomicU64::new(1),
        }
    }
}

impl Default for MockCdpConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CdpConnection for MockCdpConnection {
    async fn send_command(&self, method: &str, _params: serde_json::Value) -> Result<CdpResponse, Error> {
        if !self.is_active.load(Ordering::Relaxed) {
            return Err(Error::cdp("Connection is closed"));
        }

        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        let result = match method {
            "Page.navigate" => Some(serde_json::json!({
                "frameId": uuid::Uuid::new_v4().to_string(),
                "loaderId": uuid::Uuid::new_v4().to_string(),
            })),
            "Runtime.evaluate" => Some(serde_json::json!({
                "result": { "type": "string", "value": "mock result" }
            })),
            "Page.captureScreenshot" => Some(serde_json::json!({
                "data": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
            })),
            "DOM.getOuterHtml" => Some(serde_json::json!({
                "outerHtml": "<html><body>Mock HTML</body></html>"
            })),
            "Target.createBrowserContext" => Some(serde_json::json!({
                "browserContextId": uuid::Uuid::new_v4().to_string(),
            })),
            "Target.disposeBrowserContext" => Some(serde_json::json!({})),
            _ => Some(serde_json::json!({})),
        };

        Ok(CdpResponse {
            id,
            result,
            error: None,
        })
    }

    async fn listen_events(&self) -> Result<tokio::sync::mpsc::Receiver<CdpEvent>, Error> {
        if !self.is_active.load(Ordering::Relaxed) {
            return Err(Error::cdp("Connection is closed"));
        }

        // Return empty channel for mock
        Ok(tokio::sync::mpsc::channel(100).1)
    }

    async fn close(&self) -> Result<(), Error> {
        self.is_active.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn is_active(&self) -> bool {
        self.is_active.load(Ordering::Relaxed)
    }
}

/// Mock CDP client
#[derive(Debug)]
pub struct MockCdpClient {
    connection: Arc<MockCdpConnection>,
    url: Arc<Mutex<Option<String>>>,
    content: Arc<Mutex<String>>,
}

impl MockCdpClient {
    /// Create a new mock CDP client
    pub fn new() -> Self {
        Self {
            connection: Arc::new(MockCdpConnection::new()),
            url: Arc::new(Mutex::new(None)),
            content: Arc::new(Mutex::new(String::new())),
        }
    }

    /// Evaluate simple arithmetic expression
    fn eval_arithmetic(script: &str) -> EvaluationResult {
        let parts: Vec<&str> = script.split('+').collect();
        if parts.len() == 2 {
            let a = parts[0].trim().parse().unwrap_or(0.0);
            let b = parts[1].trim().parse().unwrap_or(0.0);
            EvaluationResult::Number(a + b)
        } else {
            EvaluationResult::String(script.to_string())
        }
    }
}

impl Default for MockCdpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CdpClient for MockCdpClient {
    fn connection(&self) -> Arc<dyn CdpConnection> {
        Arc::clone(&self.connection) as Arc<dyn CdpConnection>
    }

    async fn navigate(&self, url: &str, _timeout_ms: u64) -> Result<NavigationResult, Error> {
        *self.url.lock().await = Some(url.to_string());
        Ok(NavigationResult {
            navigation_id: Some(uuid::Uuid::new_v4().to_string()),
            url: url.to_string(),
            status_code: 200,
        })
    }

    async fn evaluate(&self, script: &str, _await_promise: bool) -> Result<EvaluationResult, Error> {
        Ok(match script {
            s if s.contains("document.title") => EvaluationResult::String("Test Page".to_string()),
            s if s.contains("window.location.href") => {
                EvaluationResult::String(self.url.lock().await.clone().unwrap_or_default())
            }
            s if s.contains('+') => Self::eval_arithmetic(s),
            _ => EvaluationResult::String("mock result".to_string()),
        })
    }

    async fn screenshot(&self, format: ScreenshotFormat) -> Result<Vec<u8>, Error> {
        Ok(match format {
            ScreenshotFormat::Png => PNG_HEADER.to_vec(),
            ScreenshotFormat::Jpeg(_) => JPEG_HEADER.to_vec(),
            ScreenshotFormat::WebP(_) => WEBP_HEADER.to_vec(),
        })
    }

    async fn get_content(&self) -> Result<String, Error> {
        Ok(self.content.lock().await.clone())
    }

    async fn set_content(&self, html: &str) -> Result<(), Error> {
        *self.content.lock().await = html.to_string();
        Ok(())
    }

    async fn reload(&self, _ignore_cache: bool) -> Result<(), Error> {
        Ok(())
    }

    async fn enable_domain(&self, _domain: &str) -> Result<(), Error> {
        Ok(())
    }

    async fn call_method(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, Error> {
        let response = self.connection.send_command(method, params).await?;

        if let Some(error) = response.error {
            return Err(Error::cdp(format!("{:?}", error)));
        }

        response.result.ok_or_else(|| Error::cdp("No result in response"))
    }

    async fn subscribe_events(&self, _event_type: &str) -> Result<tokio::sync::mpsc::Receiver<CdpEvent>, Error> {
        // Return empty channel for mock
        Ok(tokio::sync::mpsc::channel(100).1)
    }
}

/// Mock CDP browser
#[derive(Debug)]
pub struct MockCdpBrowser {
    is_active: AtomicBool,
    contexts: Arc<Mutex<Vec<BrowserContextInfo>>>,
}

impl MockCdpBrowser {
    /// Create a new mock CDP browser
    pub fn new() -> Self {
        Self {
            is_active: AtomicBool::new(true),
            contexts: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Default for MockCdpBrowser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CdpBrowser for MockCdpBrowser {
    async fn create_client(&self, _target_url: &str) -> Result<Arc<dyn CdpClient>, Error> {
        if !self.is_active.load(Ordering::Relaxed) {
            return Err(Error::cdp("Browser is closed"));
        }
        Ok(Arc::new(MockCdpClient::new()))
    }

    async fn close(&self) -> Result<(), Error> {
        self.is_active.store(false, Ordering::Relaxed);
        Ok(())
    }

    async fn get_version(&self) -> Result<BrowserVersion, Error> {
        Ok(BrowserVersion {
            protocol_version: "1.3".to_string(),
            product: "Chrome/120.0.0.0".to_string(),
            user_agent: "Mock Chrome/120.0.0.0".to_string(),
            js_version: "12.0.0.0".to_string(),
        })
    }

    async fn get_targets(&self) -> Result<Vec<TargetInfo>, Error> {
        Ok(vec![])
    }

    async fn create_target(&self, url: &str) -> Result<String, Error> {
        if !self.is_active.load(Ordering::Relaxed) {
            return Err(Error::cdp("Browser is closed"));
        }

        let ws_url = format!("ws://localhost:9222/devtools/page/{}", uuid::Uuid::new_v4());
        tracing::debug!("Mock: Created target for URL {} => {}", url, ws_url);
        Ok(ws_url)
    }

    async fn create_browser_context(
        &self,
        proxy_server: Option<String>,
    ) -> Result<BrowserContextInfo, Error> {
        if !self.is_active.load(Ordering::Relaxed) {
            return Err(Error::cdp("Browser is closed"));
        }

        use std::time::SystemTime;

        let context_id = uuid::Uuid::new_v4().to_string();
        let created_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let context_info = BrowserContextInfo {
            context_id: context_id.clone(),
            browser_endpoint: "ws://localhost:9222".to_string(),
            is_incognito: true,
            created_at,
        };

        // Store context
        self.contexts.lock().await.push(context_info.clone());

        tracing::debug!("Mock: Created browser context with proxy: {:?} => {}", proxy_server, context_id);
        Ok(context_info)
    }

    async fn dispose_browser_context(&self, context_id: &str) -> Result<(), Error> {
        if !self.is_active.load(Ordering::Relaxed) {
            return Err(Error::cdp("Browser is closed"));
        }

        let mut contexts = self.contexts.lock().await;
        contexts.retain(|ctx| ctx.context_id != context_id);

        tracing::debug!("Mock: Disposed browser context: {}", context_id);
        Ok(())
    }

    async fn get_browser_contexts(&self) -> Result<Vec<BrowserContextInfo>, Error> {
        if !self.is_active.load(Ordering::Relaxed) {
            return Err(Error::cdp("Browser is closed"));
        }

        Ok(self.contexts.lock().await.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_connection() {
        let conn = MockCdpConnection::new();
        assert!(conn.is_active());

        let response = conn
            .send_command("Runtime.evaluate", serde_json::json!({}))
            .await
            .unwrap();
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_mock_client() {
        let client = MockCdpClient::new();

        let result = client
            .navigate("https://example.com", 30000)
            .await
            .unwrap();
        assert_eq!(result.url, "https://example.com");

        let eval_result = client.evaluate("document.title", false).await.unwrap();
        matches!(eval_result, EvaluationResult::String(_));
    }

    #[tokio::test]
    async fn test_mock_browser() {
        let browser = MockCdpBrowser::new();

        let client = browser.create_client("ws://localhost:9222").await.unwrap();
        assert!(Arc::strong_count(&client.connection()) >= 1);

        let version = browser.get_version().await.unwrap();
        assert_eq!(version.product, "Chrome/120.0.0.0");
    }

    #[tokio::test]
    async fn test_mock_browser_context() {
        let browser = MockCdpBrowser::new();

        // Test create browser context
        let context = browser.create_browser_context(Some("http://proxy.example.com:8080".to_string())).await.unwrap();
        assert!(context.is_incognito);
        assert!(context.created_at > 0);

        // Test get browser contexts
        let contexts = browser.get_browser_contexts().await.unwrap();
        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0].context_id, context.context_id);

        // Test dispose browser context
        browser.dispose_browser_context(&context.context_id).await.unwrap();
        let contexts = browser.get_browser_contexts().await.unwrap();
        assert_eq!(contexts.len(), 0);
    }
}
