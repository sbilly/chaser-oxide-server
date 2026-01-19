//! CDP client implementation
//!
//! Provides a high-level CDP client with typed methods for common operations.

use super::traits::*;
use super::types::*;
use crate::Error;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::sync::Arc;
use tracing::{debug, info};

/// Page load polling interval (milliseconds)
const PAGE_LOAD_POLL_INTERVAL_MS: u64 = 100;

/// CDP client implementation
#[derive(Debug, Clone)]
pub struct CdpClientImpl {
    /// Underlying CDP connection
    connection: Arc<dyn CdpConnection>,
}

impl CdpClientImpl {
    /// Create a new CDP client
    ///
    /// # Arguments
    /// * `connection` - CDP connection instance
    pub fn new(connection: Arc<dyn CdpConnection>) -> Self {
        info!("Creating CDP client");
        Self { connection }
    }

    /// Serialize CDP parameters to JSON Value
    fn serialize_params<T: serde::Serialize>(params: &T) -> Result<serde_json::Value, Error> {
        serde_json::to_value(params)
            .map_err(|e| Error::cdp(format!("Serialization error: {}", e)))
    }

    /// Parse remote object value to evaluation result
    fn parse_remote_object(obj: &RemoteObject) -> Result<EvaluationResult, Error> {
        match obj.r#type.as_str() {
            "string" => Ok(EvaluationResult::String(
                obj.value
                    .as_ref()
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            )),
            "number" => Ok(EvaluationResult::Number(
                obj.value.as_ref().and_then(|v| v.as_f64()).unwrap_or(0.0),
            )),
            "boolean" => Ok(EvaluationResult::Bool(
                obj.value.as_ref().and_then(|v| v.as_bool()).unwrap_or(false),
            )),
            "undefined" | "null" => Ok(EvaluationResult::Null),
            "object" | "function" | "bigint" | "symbol" => {
                Ok(EvaluationResult::Object(obj.value.clone().unwrap_or(serde_json::Value::Null)))
            }
            _ => Ok(EvaluationResult::Null),
        }
    }

    /// Wait for page load to complete by polling document.readyState
    async fn wait_for_page_load(&self, timeout_ms: u64) -> Result<(), Error> {
        let max_attempts = timeout_ms / PAGE_LOAD_POLL_INTERVAL_MS;

        for attempt in 0..max_attempts {
            tokio::time::sleep(tokio::time::Duration::from_millis(PAGE_LOAD_POLL_INTERVAL_MS)).await;

            match self.evaluate("document.readyState", false).await {
                Ok(EvaluationResult::String(state)) if state == "complete" => {
                    info!("Page loaded successfully on attempt {}", attempt + 1);
                    return Ok(());
                }
                Ok(EvaluationResult::String(state)) => {
                    debug!("Document ready state on attempt {}: {}", attempt + 1, state);
                }
                Err(e) => {
                    debug!("Error checking ready state on attempt {}: {}", attempt + 1, e);
                }
                _ => {}
            }
        }

        info!("Page load polling timeout - continuing anyway");
        Ok(())
    }
}

#[async_trait]
impl CdpClient for CdpClientImpl {
    /// Get the underlying connection
    fn connection(&self) -> Arc<dyn CdpConnection> {
        Arc::clone(&self.connection)
    }

    /// Navigate to a URL
    async fn navigate(&self, url: &str, timeout_ms: u64) -> Result<NavigationResult, Error> {
        info!("Navigating to {}", url);

        let params = NavigateParams {
            url: url.to_string(),
            referrer: None,
            transition_type: None,
        };

        let result = self
            .call_method("Page.navigate", Self::serialize_params(&params)?)
            .await?;

        // Wait for page load to complete
        let _ = self.wait_for_page_load(timeout_ms).await;

        Ok(NavigationResult {
            navigation_id: result
                .get("navigationId")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            url: result
                .get("frame")
                .and_then(|f| f.get("url"))
                .and_then(|u| u.as_str())
                .unwrap_or(url)
                .to_string(),
            status_code: 200,
        })
    }

    /// Evaluate JavaScript in the page
    async fn evaluate(&self, script: &str, await_promise: bool) -> Result<EvaluationResult, Error> {
        debug!("Evaluating script: {}", script);

        let params = EvaluateParams {
            expression: script.to_string(),
            await_promise: Some(await_promise),
            return_by_value: Some(true),
            context_id: None,
        };

        let result = self
            .call_method("Runtime.evaluate", Self::serialize_params(&params)?)
            .await?;

        // Check for exception
        if let Some(exception) = result.get("exceptionDetails") {
            return Err(Error::script_execution_failed(
                exception
                    .get("exception")
                    .and_then(|e| e.get("description"))
                    .and_then(|d| d.as_str())
                    .unwrap_or("Unknown error"),
            ));
        }

        // Parse result - CDP response structure: {"result": {"result": {...}}}
        let eval_response: EvaluateResponse = serde_json::from_value(result)
            .map_err(|e| Error::cdp(format!("Failed to parse EvaluateResponse: {}", e)))?;

        Self::parse_remote_object(&eval_response.result)
    }

    /// Capture a screenshot
    async fn screenshot(&self, format: ScreenshotFormat) -> Result<Vec<u8>, Error> {
        info!("Capturing screenshot");

        let (format_str, quality) = match format {
            ScreenshotFormat::Png => ("png", None),
            ScreenshotFormat::Jpeg(q) => ("jpeg", Some(q)),
            ScreenshotFormat::WebP(q) => ("webp", Some(q)),
        };

        let mut params = serde_json::json!({ "format": format_str });
        if let Some(q) = quality {
            params["quality"] = serde_json::json!(q);
        }

        let result = self.call_method("Page.captureScreenshot", params).await?;
        let data = result
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::cdp("No data in screenshot result"))?;

        BASE64
            .decode(data)
            .map_err(|e| Error::cdp(format!("Failed to decode screenshot: {}", e)))
    }

    /// Get page content
    async fn get_content(&self) -> Result<String, Error> {
        debug!("Getting page content");

        match self.evaluate("document.documentElement.outerHTML", false).await? {
            EvaluationResult::String(html) => Ok(html),
            _ => Ok(String::new()),
        }
    }

    /// Set page content
    async fn set_content(&self, html: &str) -> Result<(), Error> {
        debug!("Setting page content");

        let script = format!("document.documentElement.outerHTML = {}", serde_json::json!(html));
        self.evaluate(&script, false).await?;
        Ok(())
    }

    /// Reload the page
    async fn reload(&self, ignore_cache: bool) -> Result<(), Error> {
        info!("Reloading page (ignore_cache: {})", ignore_cache);

        let _ = self
            .call_method("Page.reload", serde_json::json!({ "ignoreCache": ignore_cache }))
            .await?;

        Ok(())
    }

    /// Enable a domain
    async fn enable_domain(&self, domain: &str) -> Result<(), Error> {
        info!("Enabling domain: {}", domain);

        let _ = self
            .call_method(&format!("{}.enable", domain), serde_json::json!({}))
            .await?;

        Ok(())
    }

    /// Call a raw CDP method
    async fn call_method(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, Error> {
        debug!("Calling CDP method: {}", method);

        self.connection()
            .send_command(method, params)
            .await?
            .result
            .ok_or_else(|| Error::cdp("No result in response"))
    }

    /// Subscribe to events
    async fn subscribe_events(&self, event_type: &str) -> Result<tokio::sync::mpsc::Receiver<CdpEvent>, Error> {
        info!("Subscribing to events: {}", event_type);

        let event_receiver = self.connection.listen_events().await?;
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let event_filter = event_type.to_string();

        tokio::spawn(async move {
            let mut receiver = event_receiver;
            while let Some(event) = receiver.recv().await {
                let matches = event_filter == "*" || event.method == event_filter;
                if matches && tx.send(event).await.is_err() {
                    break;
                }
            }
        });

        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_remote_object_string() {
        let obj = crate::cdp::types::RemoteObject {
            r#type: "string".to_string(),
            subtype: None,
            value: Some(serde_json::json!("test")),
            description: None,
            unserializable_value: None,
        };

        let result = CdpClientImpl::parse_remote_object(&obj).unwrap();
        assert!(matches!(result, EvaluationResult::String(s) if s == "test"));
    }

    #[test]
    fn test_parse_remote_object_number() {
        let obj = crate::cdp::types::RemoteObject {
            r#type: "number".to_string(),
            subtype: None,
            value: Some(serde_json::json!(42.5)),
            description: None,
            unserializable_value: None,
        };

        let result = CdpClientImpl::parse_remote_object(&obj).unwrap();
        assert!(matches!(result, EvaluationResult::Number(n) if n == 42.5));
    }

    #[test]
    fn test_parse_remote_object_bool() {
        let obj = crate::cdp::types::RemoteObject {
            r#type: "boolean".to_string(),
            subtype: None,
            value: Some(serde_json::json!(true)),
            description: None,
            unserializable_value: None,
        };

        let result = CdpClientImpl::parse_remote_object(&obj).unwrap();
        assert!(matches!(result, EvaluationResult::Bool(true)));
    }

    #[test]
    fn test_parse_remote_object_null() {
        let obj = crate::cdp::types::RemoteObject {
            r#type: "undefined".to_string(),
            subtype: None,
            value: None,
            description: None,
            unserializable_value: None,
        };

        let result = CdpClientImpl::parse_remote_object(&obj).unwrap();
        assert!(matches!(result, EvaluationResult::Null));
    }
}
