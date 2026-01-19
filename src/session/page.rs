//! Page context implementation
//!
//! Manages page lifecycle and operations.

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::cdp;
use crate::cdp::traits::CdpClient;
use crate::session::traits::{
    EvaluationResult, LoadState, NavigationOptions, NavigationResult,
    PageContext, ScreenshotOptions, ScreenshotFormat as SessionScreenshotFormat,
};
use crate::Error;

/// Convert CDP evaluation result to session evaluation result
impl From<cdp::traits::EvaluationResult> for EvaluationResult {
    fn from(value: cdp::traits::EvaluationResult) -> Self {
        match value {
            cdp::traits::EvaluationResult::String(s) => Self::String(s),
            cdp::traits::EvaluationResult::Number(n) => Self::Number(n),
            cdp::traits::EvaluationResult::Bool(b) => Self::Bool(b),
            cdp::traits::EvaluationResult::Null => Self::Null,
            cdp::traits::EvaluationResult::Object(v) => Self::Object(v),
        }
    }
}

/// Page context implementation
#[derive(Debug)]
pub struct PageContextImpl {
    id: String,
    browser_id: String,
    options: crate::session::traits::PageOptions,
    cdp_client: Arc<dyn CdpClient>,
    is_active: Arc<tokio::sync::RwLock<bool>>,
}

impl PageContextImpl {
    /// Create a new page context
    pub fn new(
        browser_id: String,
        options: crate::session::traits::PageOptions,
        cdp_client: Arc<dyn CdpClient>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            browser_id,
            options,
            cdp_client,
            is_active: Arc::new(tokio::sync::RwLock::new(true)),
        }
    }

    /// Ensure page is active, return error if not
    async fn ensure_active(&self) -> Result<(), Error> {
        let active = *self.is_active.read().await;
        if active {
            Ok(())
        } else {
            Err(Error::page_not_found(&self.id))
        }
    }

    /// Convert screenshot options
    fn convert_screenshot_format(format: SessionScreenshotFormat) -> cdp::traits::ScreenshotFormat {
        match format {
            SessionScreenshotFormat::Png => cdp::traits::ScreenshotFormat::Png,
            SessionScreenshotFormat::Jpeg => cdp::traits::ScreenshotFormat::Jpeg(100),
            SessionScreenshotFormat::WebP => cdp::traits::ScreenshotFormat::WebP(100),
        }
    }

    /// Navigate browser history (back or forward)
    async fn navigate_history(&self, direction: &str) -> Result<(), Error> {
        let javascript_url = format!("javascript:history.{}()", direction);
        self.cdp_client
            .call_method("Page.navigate", serde_json::json!({ "url": javascript_url }))
            .await?;
        Ok(())
    }
}

#[async_trait]
impl PageContext for PageContextImpl {
    fn id(&self) -> &str {
        &self.id
    }

    fn browser_id(&self) -> &str {
        &self.browser_id
    }

    async fn navigate(&self, url: &str, options: NavigationOptions) -> Result<NavigationResult, Error> {
        self.ensure_active().await?;

        let nav_result = self.cdp_client.navigate(url, options.timeout).await?;

        // Wait for load state if specified
        let delay_ms = match options.wait_until {
            LoadState::Load => 100,
            LoadState::DOMContentLoaded => 50,
            LoadState::NetworkIdle | LoadState::NetworkAlmostIdle => 500,
        };
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

        Ok(NavigationResult {
            url: nav_result.url,
            status_code: 200,
            is_loaded: true,
        })
    }

    async fn get_content(&self) -> Result<String, Error> {
        self.ensure_active().await?;
        self.cdp_client.get_content().await
    }

    async fn set_content(&self, html: &str) -> Result<(), Error> {
        self.ensure_active().await?;
        self.cdp_client.set_content(html).await
    }

    async fn reload(&self, ignore_cache: bool) -> Result<(), Error> {
        self.ensure_active().await?;
        self.cdp_client.reload(ignore_cache).await
    }

    async fn go_back(&self) -> Result<(), Error> {
        self.ensure_active().await?;
        self.navigate_history("back").await
    }

    async fn go_forward(&self) -> Result<(), Error> {
        self.ensure_active().await?;
        self.navigate_history("forward").await
    }

    async fn evaluate(&self, script: &str, await_promise: bool) -> Result<EvaluationResult, Error> {
        self.ensure_active().await?;

        let result = self.cdp_client.evaluate(script, await_promise).await?;
        tracing::debug!("PageContext::evaluate: CDP returned {:?}", result);

        let session_result: EvaluationResult = result.into();
        tracing::debug!("PageContext::evaluate: returning {:?}", session_result);
        Ok(session_result)
    }

    async fn screenshot(&self, options: ScreenshotOptions) -> Result<Vec<u8>, Error> {
        self.ensure_active().await?;

        let format = Self::convert_screenshot_format(options.format);
        self.cdp_client.screenshot(format).await
    }

    async fn set_viewport(&self, width: u32, height: u32, device_scale_factor: f64) -> Result<(), Error> {
        self.ensure_active().await?;

        let _ = self.cdp_client
            .call_method(
                "Emulation.setDeviceMetricsOverride",
                serde_json::json!({
                    "width": width,
                    "height": height,
                    "deviceScaleFactor": device_scale_factor,
                    "mobile": self.options.is_mobile,
                }),
            )
            .await?;

        Ok(())
    }

    async fn close(&self) -> Result<(), Error> {
        tracing::info!("PageContext::close: Closing page {}", self.id);

        // Check if page is still active before attempting close
        let active = *self.is_active.read().await;
        if !active {
            tracing::warn!("PageContext::close: Page {} is already inactive", self.id);
            return Ok(());
        }

        // Try to close the page via CDP - Page.close command will close the page in the browser
        tracing::debug!("PageContext::close: Sending Page.close CDP command for page {}", self.id);
        let close_result = self
            .cdp_client
            .call_method("Page.close", serde_json::json!({}))
            .await;

        match &close_result {
            Ok(_) => {
                tracing::info!("PageContext::close: Page.close CDP command succeeded for page {}", self.id);
            }
            Err(e) => {
                tracing::warn!("PageContext::close: Page.close CDP command failed for page {}: {}", self.id, e);
                tracing::warn!("PageContext::close: The page may not be closed in the browser");
            }
        }

        // Mark as inactive regardless of CDP result
        // This ensures the page context is removed from browser's management
        tracing::debug!("PageContext::close: Marking page {} as inactive", self.id);
        *self.is_active.write().await = false;

        tracing::info!("PageContext::close: Page {} close completed", self.id);
        Ok(())
    }

    fn is_active(&self) -> bool {
        // Use try_read to avoid blocking in sync context
        self.is_active
            .try_read()
            .ok()
            .map(|active| *active)
            .unwrap_or(false)
    }

    fn get_cdp_client(&self) -> Arc<dyn crate::cdp::traits::CdpClient> {
        self.cdp_client.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_page_creation() {
        let cdp_client = Arc::new(crate::cdp::mock::MockCdpClient::new());
        let page = PageContextImpl::new(
            "test-browser".to_string(),
            crate::session::traits::PageOptions::default(),
            cdp_client,
        );

        assert!(page.is_active());
        assert_eq!(page.browser_id(), "test-browser");
    }

    #[tokio::test]
    async fn test_page_navigate() {
        let cdp_client = Arc::new(crate::cdp::mock::MockCdpClient::new());
        let page = PageContextImpl::new(
            "test-browser".to_string(),
            crate::session::traits::PageOptions::default(),
            cdp_client,
        );

        let result = page
            .navigate(
                "https://example.com",
                NavigationOptions {
                    timeout: 30000,
                    wait_until: LoadState::Load,
                },
            )
            .await
            .unwrap();

        assert_eq!(result.url, "https://example.com");
    }

    #[tokio::test]
    async fn test_page_evaluate() {
        let cdp_client = Arc::new(crate::cdp::mock::MockCdpClient::new());
        let page = PageContextImpl::new(
            "test-browser".to_string(),
            crate::session::traits::PageOptions::default(),
            cdp_client,
        );

        let result = page.evaluate("document.title", false).await.unwrap();
        matches!(result, EvaluationResult::String(_));
    }

    #[tokio::test]
    async fn test_page_screenshot() {
        let cdp_client = Arc::new(crate::cdp::mock::MockCdpClient::new());
        let page = PageContextImpl::new(
            "test-browser".to_string(),
            crate::session::traits::PageOptions::default(),
            cdp_client,
        );

        let screenshot = page
            .screenshot(ScreenshotOptions {
                format: crate::session::traits::ScreenshotFormat::Png,
                quality: None,
                full_page: false,
                clip: None,
            })
            .await
            .unwrap();

        assert!(!screenshot.is_empty());
    }

    #[tokio::test]
    async fn test_page_close() {
        let cdp_client = Arc::new(crate::cdp::mock::MockCdpClient::new());
        let page = PageContextImpl::new(
            "test-browser".to_string(),
            crate::session::traits::PageOptions::default(),
            cdp_client,
        );

        assert!(page.is_active());
        page.close().await.unwrap();
        assert!(!page.is_active());
    }
}
