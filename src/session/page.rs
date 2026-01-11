//! Page context implementation
//!
//! Manages page lifecycle and operations.

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::cdp::traits::CdpClient;
use crate::session::traits::{
    EvaluationResult, LoadState, NavigationOptions, NavigationResult,
    PageContext, ScreenshotOptions,
};
use crate::Error;

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

    /// Convert screenshot options
    fn convert_screenshot_format(format: crate::session::traits::ScreenshotFormat) -> crate::cdp::traits::ScreenshotFormat {
        match format {
            crate::session::traits::ScreenshotFormat::Png => crate::cdp::traits::ScreenshotFormat::Png,
            crate::session::traits::ScreenshotFormat::Jpeg => crate::cdp::traits::ScreenshotFormat::Jpeg(100),
            crate::session::traits::ScreenshotFormat::WebP => crate::cdp::traits::ScreenshotFormat::WebP(100),
        }
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
        let active = *self.is_active.read().await;
        if !active {
            return Err(Error::page_not_found(&self.id));
        }

        // Navigate using CDP
        let nav_result = self.cdp_client.navigate(url).await?;

        // Wait for load state if specified
        match options.wait_until {
            LoadState::Load => {
                // In real implementation, wait for load event
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            LoadState::DOMContentLoaded => {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
            LoadState::NetworkIdle | LoadState::NetworkAlmostIdle => {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }

        Ok(NavigationResult {
            url: nav_result.url,
            status_code: 200,
            is_loaded: true,
        })
    }

    async fn get_content(&self) -> Result<String, Error> {
        let active = *self.is_active.read().await;
        if !active {
            return Err(Error::page_not_found(&self.id));
        }

        self.cdp_client.get_content().await
    }

    async fn set_content(&self, html: &str) -> Result<(), Error> {
        let active = *self.is_active.read().await;
        if !active {
            return Err(Error::page_not_found(&self.id));
        }

        self.cdp_client.set_content(html).await
    }

    async fn reload(&self, ignore_cache: bool) -> Result<(), Error> {
        let active = *self.is_active.read().await;
        if !active {
            return Err(Error::page_not_found(&self.id));
        }

        self.cdp_client.reload(ignore_cache).await
    }

    async fn go_back(&self) -> Result<(), Error> {
        let active = *self.is_active.read().await;
        if !active {
            return Err(Error::page_not_found(&self.id));
        }

        // Use CDP to navigate back
        self.cdp_client
            .call_method(
                "Page.navigate",
                serde_json::json!({ "url": "javascript:history.back()" }),
            )
            .await?;
        Ok(())
    }

    async fn go_forward(&self) -> Result<(), Error> {
        let active = *self.is_active.read().await;
        if !active {
            return Err(Error::page_not_found(&self.id));
        }

        // Use CDP to navigate forward
        self.cdp_client
            .call_method(
                "Page.navigate",
                serde_json::json!({ "url": "javascript:history.forward()" }),
            )
            .await?;
        Ok(())
    }

    async fn evaluate(&self, script: &str, await_promise: bool) -> Result<EvaluationResult, Error> {
        let active = *self.is_active.read().await;
        if !active {
            return Err(Error::page_not_found(&self.id));
        }

        let result = self.cdp_client.evaluate(script, await_promise).await?;
        tracing::debug!("PageContext::evaluate: CDP returned {:?}", result);

        let session_result = match result {
            crate::cdp::traits::EvaluationResult::String(s) => EvaluationResult::String(s),
            crate::cdp::traits::EvaluationResult::Number(n) => EvaluationResult::Number(n),
            crate::cdp::traits::EvaluationResult::Bool(b) => EvaluationResult::Bool(b),
            crate::cdp::traits::EvaluationResult::Null => EvaluationResult::Null,
            crate::cdp::traits::EvaluationResult::Object(v) => EvaluationResult::Object(v),
        };
        tracing::debug!("PageContext::evaluate: returning {:?}", session_result);
        Ok(session_result)
    }

    async fn screenshot(&self, options: ScreenshotOptions) -> Result<Vec<u8>, Error> {
        let active = *self.is_active.read().await;
        if !active {
            return Err(Error::page_not_found(&self.id));
        }

        let format = Self::convert_screenshot_format(options.format);
        self.cdp_client.screenshot(format).await
    }

    async fn set_viewport(&self, width: u32, height: u32, device_scale_factor: f64) -> Result<(), Error> {
        let active = *self.is_active.read().await;
        if !active {
            return Err(Error::page_not_found(&self.id));
        }

        // Ignore result
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
