//! Browser context implementation
//!
//! Manages browser lifecycle and page creation.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::cdp::traits::CdpBrowser;
use crate::session::traits::{BrowserContext, BrowserContextInfo, BrowserOptions, PageContext, PageOptions};
use crate::Error;

/// Summary of browser close operation results
#[derive(Debug, Default)]
struct CloseSummary {
    succeeded: usize,
    failed: Vec<(String, Error)>,
}

impl CloseSummary {
    fn new() -> Self {
        Self {
            succeeded: 0,
            failed: Vec::new(),
        }
    }

    fn record_success(&mut self) {
        self.succeeded += 1;
    }

    fn record_failure(&mut self, page_id: String, error: Error) {
        self.failed.push((page_id, error));
    }

    fn log_summary(&self, _browser_id: &str) {
        if !self.failed.is_empty() {
            tracing::warn!(
                "BrowserContext::close: {} pages failed to close:",
                self.failed.len()
            );
            for (page_id, error) in &self.failed {
                tracing::warn!("  - Page {}: {}", page_id, error);
            }
        }
        tracing::info!(
            "BrowserContext::close: Page close summary: {} succeeded, {} failed",
            self.succeeded,
            self.failed.len()
        );
    }
}

/// Browser context implementation
#[derive(Debug)]
pub struct BrowserContextImpl {
    id: String,
    options: BrowserOptions,
    cdp_browser: Arc<dyn CdpBrowser>,
    pages: Arc<RwLock<HashMap<String, Arc<dyn PageContext>>>>,
    is_active: Arc<RwLock<bool>>,
    incognito_contexts: Arc<RwLock<HashMap<String, BrowserContextInfo>>>,
}

// Lock error helper
fn lock_error<E: std::error::Error>(e: E) -> Error {
    Error::internal(format!("Lock error: {}", e))
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
            incognito_contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new browser context with a specific ID
    ///
    /// This is used when the browser ID is already known (e.g., from ProcessManager)
    /// to ensure ID consistency across the system.
    pub fn with_id(id: String, options: BrowserOptions, cdp_browser: Arc<dyn CdpBrowser>) -> Self {
        Self {
            id,
            options,
            cdp_browser,
            pages: Arc::new(RwLock::new(HashMap::new())),
            is_active: Arc::new(RwLock::new(true)),
            incognito_contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get browser options
    pub fn options(&self) -> &BrowserOptions {
        &self.options
    }

    /// Ensure browser is active, returning error if not
    fn ensure_active(&self) -> Result<(), Error> {
        let active = *self.is_active.read().map_err(lock_error)?;
        if active {
            Ok(())
        } else {
            Err(Error::browser_not_found(&self.id))
        }
    }

    /// Read pages lock with unified error handling
    fn read_pages(
        &self,
    ) -> Result<std::sync::RwLockReadGuard<'_, HashMap<String, Arc<dyn PageContext>>>, Error> {
        self.pages.read().map_err(lock_error)
    }

    /// Write pages lock with unified error handling
    fn write_pages(
        &self,
    ) -> Result<std::sync::RwLockWriteGuard<'_, HashMap<String, Arc<dyn PageContext>>>, Error> {
        self.pages.write().map_err(lock_error)
    }

    /// Set user agent for a CDP client
    async fn set_user_agent(&self, cdp_client: Arc<dyn crate::cdp::traits::CdpClient>) -> Result<(), Error> {
        let Some(user_agent) = &self.options.user_agent else {
            return Ok(());
        };
        if user_agent.is_empty() {
            return Ok(());
        }

        cdp_client.enable_domain("Network").await?;

        let params = serde_json::json!({ "userAgent": user_agent });
        cdp_client.call_method("Network.setUserAgentOverride", params).await?;

        tracing::debug!("User-Agent set at page creation: {}", user_agent);
        Ok(())
    }
}

#[async_trait]
impl BrowserContext for BrowserContextImpl {
    fn id(&self) -> &str {
        &self.id
    }

    async fn create_page(&self, options: PageOptions) -> Result<Arc<dyn PageContext>, Error> {
        self.ensure_active()?;

        let default_url = options.default_url.as_deref().unwrap_or("about:blank");
        let ws_url = self.cdp_browser.create_target(default_url).await?;
        let cdp_client = self.cdp_browser.create_client(&ws_url).await?;

        self.set_user_agent(cdp_client.clone()).await?;

        let target_id = ws_url.rsplit('/').next().unwrap_or("unknown");
        let page = Arc::new(crate::session::page::PageContextImpl::new(
            self.id.clone(),
            options,
            cdp_client,
        ));

        self.write_pages()?.insert(target_id.to_string(), page.clone());
        Ok(page)
    }

    async fn get_pages(&self) -> Result<Vec<Arc<dyn PageContext>>, Error> {
        self.ensure_active()?;
        Ok(self.read_pages()?.values().cloned().collect())
    }

    async fn close(&self) -> Result<(), Error> {
        tracing::info!("BrowserContext::close: Closing browser {}", self.id);

        let pages_to_close: Vec<Arc<dyn PageContext>> = self
            .write_pages()?
            .drain()
            .map(|(_, page)| page)
            .collect();

        let page_count = pages_to_close.len();
        tracing::info!(
            "BrowserContext::close: Closing {} pages for browser {}",
            page_count,
            self.id
        );

        let mut summary = CloseSummary::new();

        for page in &pages_to_close {
            tracing::debug!(
                "BrowserContext::close: Closing page {} in browser {}",
                page.id(),
                self.id
            );
            match page.close().await {
                Ok(_) => {
                    summary.record_success();
                    tracing::debug!("BrowserContext::close: Successfully closed page {}", page.id());
                }
                Err(e) => {
                    let page_id = page.id().to_string();
                    tracing::warn!(
                        "BrowserContext::close: Failed to close page {}: {}",
                        page_id,
                        e
                    );
                    summary.record_failure(page_id, e);
                }
            }
        }

        summary.log_summary(&self.id);

        if let Err(e) = self.cdp_browser.close().await {
            tracing::warn!(
                "BrowserContext::close: Failed to close CDP browser connections: {}",
                e
            );
        } else {
            tracing::info!("BrowserContext::close: CDP browser connections closed successfully");
        }

        *self.is_active.write().map_err(lock_error)? = false;
        tracing::info!("BrowserContext::close: Browser {} close completed", self.id);
        Ok(())
    }

    fn is_active(&self) -> bool {
        self.is_active.read().map(|active| *active).unwrap_or(false)
    }

    async fn create_incognito_context(&self) -> Result<BrowserContextInfo, Error> {
        self.ensure_active()?;

        // Generate a unique context ID
        let context_id = Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().timestamp();

        // Create context info
        let context_info = BrowserContextInfo {
            context_id: context_id.clone(),
            browser_id: self.id.clone(),
            is_incognito: true,
            created_at,
        };

        // Store the context
        self.incognito_contexts
            .write()
            .map_err(lock_error)?
            .insert(context_id.clone(), context_info.clone());

        tracing::info!(
            "Created incognito context {} for browser {}",
            context_id,
            self.id
        );

        Ok(context_info)
    }

    async fn close_incognito_context(&self, context_id: &str) -> Result<(), Error> {
        self.ensure_active()?;

        let removed = self
            .incognito_contexts
            .write()
            .map_err(lock_error)?
            .remove(context_id);

        if removed.is_some() {
            tracing::info!(
                "Closed incognito context {} for browser {}",
                context_id,
                self.id
            );
            Ok(())
        } else {
            Err(Error::browser_not_found(&format!(
                "Incognito context {} not found",
                context_id
            )))
        }
    }

    async fn get_contexts(&self) -> Result<Vec<BrowserContextInfo>, Error> {
        self.ensure_active()?;

        let contexts = self
            .incognito_contexts
            .read()
            .map_err(lock_error)?
            .values()
            .cloned()
            .collect();

        Ok(contexts)
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

    #[tokio::test]
    async fn test_incognito_context() {
        let options = BrowserOptions::default();
        let cdp_browser = Arc::new(crate::cdp::mock::MockCdpBrowser::new());
        let browser = BrowserContextImpl::new(options, cdp_browser);

        // Create incognito context
        let context = browser.create_incognito_context().await.unwrap();
        assert!(context.is_incognito);
        assert_eq!(context.browser_id, browser.id());

        // List contexts
        let contexts = browser.get_contexts().await.unwrap();
        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0].context_id, context.context_id);

        // Close context
        browser.close_incognito_context(&context.context_id).await.unwrap();
        let contexts = browser.get_contexts().await.unwrap();
        assert_eq!(contexts.len(), 0);
    }
}
