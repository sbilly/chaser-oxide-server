//! CDP client implementation
//!
//! This module provides a high-level CDP client with typed methods for common operations.

use super::traits::*;
use super::types::*;
use crate::Error;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::sync::Arc;
use tracing::{debug, info};

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

    /// Parse remote object value to evaluation result
    fn parse_remote_object(obj: &crate::cdp::types::RemoteObject) -> Result<EvaluationResult, Error> {
        let result = match obj.r#type.as_str() {
            "string" => {
                let value = obj.value
                    .as_ref()
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                debug!("parse_remote_object: string type, value='{}'", value);
                Ok(EvaluationResult::String(value))
            },
            "number" => {
                let value = obj.value
                    .as_ref()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                debug!("parse_remote_object: number type, value={}", value);
                Ok(EvaluationResult::Number(value))
            },
            "boolean" => {
                let value = obj.value
                    .as_ref()
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                debug!("parse_remote_object: boolean type, value={}", value);
                Ok(EvaluationResult::Bool(value))
            },
            "undefined" | "null" => {
                debug!("parse_remote_object: null/undefined type");
                Ok(EvaluationResult::Null)
            },
            "object" | "function" | "bigint" | "symbol" => {
                let value = obj.value.clone().unwrap_or(serde_json::Value::Null);
                debug!("parse_remote_object: object type, value={:?}", value);
                Ok(EvaluationResult::Object(value))
            },
            _ => {
                debug!("parse_remote_object: unknown type '{}', returning Null", obj.r#type);
                Ok(EvaluationResult::Null)
            },
        };
        debug!("parse_remote_object: returning {:?}", result);
        result
    }
}

#[async_trait]
impl CdpClient for CdpClientImpl {
    /// Get the underlying connection
    fn connection(&self) -> Arc<dyn CdpConnection> {
        Arc::clone(&self.connection)
    }

    /// Navigate to a URL
    async fn navigate(&self, url: &str) -> Result<NavigationResult, Error> {
        info!("Navigating to {}", url);

        let params = NavigateParams {
            url: url.to_string(),
            referrer: None,
            transition_type: None,
        };

        let result = self
            .call_method(
                "Page.navigate",
                serde_json::to_value(params).map_err(|e| Error::cdp(format!("Serialization error: {}", e)))?,
            )
            .await?;

        // Wait for page load by polling document.readyState
        // This is more reliable than event-based approach due to race conditions
        let max_attempts = 50; // 5 seconds (50 * 100ms)
        let mut page_loaded = false;

        for attempt in 0..max_attempts {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            match self.evaluate("document.readyState", false).await {
                Ok(EvaluationResult::String(state)) if state == "complete" => {
                    info!("Page loaded successfully on attempt {}", attempt + 1);
                    page_loaded = true;
                    break;
                }
                Ok(EvaluationResult::String(state)) => {
                    debug!("Document ready state on attempt {}: {}", attempt + 1, state);
                }
                Ok(_) => {
                    debug!("Unexpected document.readyState type on attempt {}", attempt + 1);
                }
                Err(e) => {
                    // Page might not be ready yet, continue polling
                    debug!("Error checking ready state on attempt {}: {}", attempt + 1, e);
                }
            }
        }

        if !page_loaded {
            info!("Page load polling timeout - continuing anyway");
        }

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
            status_code: 200, // Default to OK
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
            .call_method(
                "Runtime.evaluate",
                serde_json::to_value(params).map_err(|e| Error::cdp(format!("Serialization error: {}", e)))?,
            )
            .await?;

        // Check for exception
        if let Some(exception) = result.get("exceptionDetails") {
            return Err(Error::script_execution_failed(
                exception.get("exception")
                    .and_then(|e| e.get("description"))
                    .and_then(|d| d.as_str())
                    .unwrap_or("Unknown error")
                    .to_string()
            ));
        }

        // Parse result - CDP response structure: {"result": {"result": {...}}}
        let eval_response: crate::cdp::types::EvaluateResponse = serde_json::from_value(result)
            .map_err(|e| Error::cdp(format!("Failed to parse EvaluateResponse: {}", e)))?;
        let remote_obj = eval_response.result;
        debug!("evaluate: parsed RemoteObject: type='{}', value={:?}", remote_obj.r#type, remote_obj.value);

        let eval_result = Self::parse_remote_object(&remote_obj)?;
        debug!("evaluate: parse_remote_object returned {:?}", eval_result);
        Ok(eval_result)
    }

    /// Capture a screenshot
    async fn screenshot(&self, format: ScreenshotFormat) -> Result<Vec<u8>, Error> {
        info!("Capturing screenshot");

        let (format_str, quality) = match format {
            ScreenshotFormat::Png => ("png".to_string(), None),
            ScreenshotFormat::Jpeg(q) => ("jpeg".to_string(), Some(q)),
            ScreenshotFormat::WebP(q) => ("webp".to_string(), Some(q)),
        };

        let mut params = serde_json::json!({
            "format": format_str,
        });

        if let Some(q) = quality {
            params["quality"] = serde_json::json!(q);
        }

        let result = self.call_method("Page.captureScreenshot", params).await?;

        let data = result
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::cdp("No data in screenshot result"))?;

        // Decode base64
        BASE64
            .decode(data)
            .map_err(|e| Error::cdp(format!("Failed to decode screenshot: {}", e)))
    }

    /// Get page content
    async fn get_content(&self) -> Result<String, Error> {
        debug!("Getting page content");

        let script = "document.documentElement.outerHTML";

        match self.evaluate(script, false).await? {
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

        let params = serde_json::json!({
            "ignoreCache": ignore_cache,
        });

        // Ignore result for reload command
        let _ = self.call_method("Page.reload", params).await?;

        Ok(())
    }

    /// Enable a domain
    async fn enable_domain(&self, domain: &str) -> Result<(), Error> {
        info!("Enabling domain: {}", domain);

        let method = format!("{}.enable", domain);

        // Ignore result for enable command
        let _ = self.call_method(&method, serde_json::json!({})).await?;

        Ok(())
    }

    /// Call a raw CDP method
    async fn call_method(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, Error> {
        debug!("Calling CDP method: {}", method);

        let response = self.connection().send_command(method, params).await?;

        response.result.ok_or_else(|| Error::cdp("No result in response"))
    }

    /// Subscribe to events
    async fn subscribe_events(&self, event_type: &str) -> Result<tokio::sync::mpsc::Receiver<CdpEvent>, Error> {
        info!("Subscribing to events: {}", event_type);

        let event_receiver = self.connection.listen_events().await?;

        // Filter events by type
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let filter_event_type = event_type.to_string();

        tokio::spawn(async move {
            let mut event_receiver = event_receiver;
            while let Some(event) = event_receiver.recv().await {
                if (event.method == filter_event_type || filter_event_type == "*")
                    && tx.send(event).await.is_err()
                {
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
