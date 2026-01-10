//! Browser context implementation
//!
//! Manages browser lifecycle and page creation.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::cdp::traits::CdpBrowser;
use crate::session::traits::{BrowserContext, BrowserOptions, PageContext, PageOptions};
use crate::Error;

/// Browser context implementation
#[derive(Debug)]
pub struct BrowserContextImpl {
    id: String,
    options: BrowserOptions,
    cdp_browser: Arc<dyn CdpBrowser>,
    pages: Arc<RwLock<HashMap<String, Arc<dyn PageContext>>>>,
    is_active: Arc<RwLock<bool>>,
}

impl BrowserContextImpl {
    /// Create a new browser context
    pub fn new(options: BrowserOptions, cdp_browser: Arc<dyn CdpBrowser>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            options,
            cdp_browser,
            pages: Arc::new(RwLock::new(HashMap::new())),
            is_active: Arc::new(RwLock::new(true)),
        }
    }

    /// Get browser options
    pub fn options(&self) -> &BrowserOptions {
        &self.options
    }
}

#[async_trait]
impl BrowserContext for BrowserContextImpl {
    fn id(&self) -> &str {
        &self.id
    }

    async fn create_page(&self, options: PageOptions) -> Result<Arc<dyn PageContext>, Error> {
        // Check if browser is active
        let active = *self
            .is_active
            .read()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;
        if !active {
            return Err(Error::browser_not_found(&self.id));
        }

        // Determine the URL for the new page
        let default_url = options.default_url.as_ref().map(|s| s.as_str()).unwrap_or("about:blank");

        // Create a new target and get its WebSocket URL using CDP Target.createTarget
        let ws_url = self.cdp_browser.create_target(default_url).await?;

        // Create CDP client with the WebSocket URL
        let cdp_client = self.cdp_browser.create_client(&ws_url).await?;

        // Set User-Agent at CDP level if provided in browser options
        // This must be done BEFORE any navigation to ensure correct UA is used
        if let Some(user_agent) = &self.options.user_agent {
            if !user_agent.is_empty() {
                // Enable Network domain first
                cdp_client.enable_domain("Network").await?;

                // Set User-Agent override using Network.setUserAgentOverride
                let params = serde_json::json!({
                    "userAgent": user_agent
                });

                cdp_client.call_method("Network.setUserAgentOverride", params).await?;

                tracing::debug!("User-Agent set at page creation: {}", user_agent);
            }
        }

        // Extract target_id from ws_url for use as page key
        let target_id = ws_url
            .rsplit('/')
            .next()
            .unwrap_or("unknown");

        // Create page context
        let page = Arc::new(crate::session::page::PageContextImpl::new(
            self.id.clone(),
            options,
            cdp_client,
        ));

        // Store page using target_id as the key
        self.pages
            .write()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))?
            .insert(target_id.to_string(), page.clone());

        Ok(page)
    }

    async fn get_pages(&self) -> Result<Vec<Arc<dyn PageContext>>, Error> {
        let active = *self
            .is_active
            .read()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;
        if !active {
            return Err(Error::browser_not_found(&self.id));
        }

        let pages = self
            .pages
            .read()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;
        Ok(pages.values().cloned().collect())
    }

    async fn close(&self) -> Result<(), Error> {
        // Close all pages - collect pages first to avoid holding lock across await
        let pages_to_close: Vec<Arc<dyn PageContext>> = self
            .pages
            .write()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))?
            .drain()
            .map(|(_, page)| page)
            .collect();
        // Lock guard dropped here

        for page in pages_to_close {
            let _ = page.close().await;
        }

        // Mark as inactive
        *self
            .is_active
            .write()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))? = false;

        Ok(())
    }

    fn is_active(&self) -> bool {
        self.is_active
            .read()
            .map(|active| *active)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_browser_creation() {
        let options = BrowserOptions::default();
        let cdp_browser = Arc::new(crate::cdp::mock::MockCdpBrowser::new());
        let browser = BrowserContextImpl::new(options, cdp_browser);

        assert!(browser.is_active());
        assert!(!browser.id().is_empty());
    }

    #[tokio::test]
    async fn test_browser_create_page() {
        let options = BrowserOptions::default();
        let cdp_browser = Arc::new(crate::cdp::mock::MockCdpBrowser::new());
        let browser = BrowserContextImpl::new(options, cdp_browser);

        let page_options = PageOptions::default();
        let page = browser.create_page(page_options).await.unwrap();

        assert_eq!(page.browser_id(), browser.id());
        assert!(page.is_active());
    }

    #[tokio::test]
    async fn test_browser_get_pages() {
        let options = BrowserOptions::default();
        let cdp_browser = Arc::new(crate::cdp::mock::MockCdpBrowser::new());
        let browser = BrowserContextImpl::new(options, cdp_browser);

        // Create multiple pages
        browser.create_page(PageOptions::default()).await.unwrap();
        browser.create_page(PageOptions::default()).await.unwrap();

        let pages = browser.get_pages().await.unwrap();
        assert_eq!(pages.len(), 2);
    }

    #[tokio::test]
    async fn test_browser_close() {
        let options = BrowserOptions::default();
        let cdp_browser = Arc::new(crate::cdp::mock::MockCdpBrowser::new());
        let browser = BrowserContextImpl::new(options, cdp_browser);

        // Create a page
        browser.create_page(PageOptions::default()).await.unwrap();

        // Close browser
        browser.close().await.unwrap();
        assert!(!browser.is_active());

        // Should not be able to create pages after close
        let result = browser.create_page(PageOptions::default()).await;
        assert!(result.is_err());
    }
}
