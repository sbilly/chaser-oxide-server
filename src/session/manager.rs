//! Session manager implementation
//!
//! Manages all browser, page, and element sessions with thread-safe operations.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::cdp::traits::CdpBrowser;
use crate::session::traits::{
    BrowserContext, BrowserOptions, PageContext, PageOptions, SessionManager,
};
use crate::Error;

/// Session manager implementation
pub struct SessionManagerImpl {
    browsers: Arc<RwLock<HashMap<String, Arc<dyn BrowserContext>>>>,
    cdp_browser_factory: Arc<dyn Fn() -> Result<Arc<dyn CdpBrowser>, Error> + Send + Sync>,
}

impl SessionManagerImpl {
    /// Create a new session manager
    pub fn new<F>(factory: F) -> Self
    where
        F: Fn() -> Result<Arc<dyn CdpBrowser>, Error> + Send + Sync + 'static,
    {
        Self {
            browsers: Arc::new(RwLock::new(HashMap::new())),
            cdp_browser_factory: Arc::new(factory),
        }
    }

    /// Create a session manager with a mock CDP browser for testing
    pub fn mock() -> Self {
        Self::new(|| Ok(Arc::new(crate::cdp::mock::MockCdpBrowser::new())))
    }
}

#[cfg(test)]
impl Default for SessionManagerImpl {
    fn default() -> Self {
        Self::mock()
    }
}

#[async_trait]
impl SessionManager for SessionManagerImpl {
    async fn create_browser(&self, options: BrowserOptions) -> Result<String, Error> {
        // Create CDP browser
        let cdp_browser = (self.cdp_browser_factory)()?;

        // Create browser context
        let browser = Arc::new(crate::session::browser::BrowserContextImpl::new(
            options.clone(),
            cdp_browser,
        ));

        // Store browser
        let browser_id = browser.id().to_string();
        self.browsers
            .write()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))?
            .insert(browser_id.clone(), browser);

        Ok(browser_id)
    }

    async fn get_browser(&self, browser_id: &str) -> Result<Arc<dyn BrowserContext>, Error> {
        self.browsers
            .read()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))?
            .get(browser_id)
            .cloned()
            .ok_or_else(|| Error::browser_not_found(browser_id))
    }

    async fn close_browser(&self, browser_id: &str) -> Result<(), Error> {
        // Get browser
        let browser = self.get_browser(browser_id).await?;

        // Close browser
        browser.close().await?;

        // Remove from map
        self.browsers
            .write()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))?
            .remove(browser_id);

        Ok(())
    }

    async fn list_browsers(&self) -> Result<Vec<String>, Error> {
        let browsers = self
            .browsers
            .read()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;
        Ok(browsers.keys().cloned().collect())
    }

    async fn create_page(
        &self,
        browser_id: &str,
        options: PageOptions,
    ) -> Result<Arc<dyn PageContext>, Error> {
        let browser = self.get_browser(browser_id).await?;
        browser.create_page(options).await
    }

    async fn get_page(&self, page_id: &str) -> Result<Arc<dyn PageContext>, Error> {
        // Search through all browsers to find the page
        // Collect browser Arcs first to avoid holding lock across await
        let browser_refs: Vec<Arc<dyn BrowserContext>> = self
            .browsers
            .read()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))?
            .values()
            .cloned()
            .collect();
        // Lock guard dropped here

        for browser in browser_refs {
            if let Ok(pages) = browser.get_pages().await {
                for page in pages {
                    if page.id() == page_id {
                        return Ok(page);
                    }
                }
            }
        }

        Err(Error::page_not_found(page_id))
    }

    async fn close_page(&self, page_id: &str) -> Result<(), Error> {
        let page = self.get_page(page_id).await?;
        page.close().await
    }

    async fn cleanup(&self) -> Result<(), Error> {
        // Close all inactive browsers
        let mut to_remove = Vec::new();

        {
            let browsers = self
                .browsers
                .read()
                .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;

            for (id, browser) in browsers.iter() {
                if !browser.is_active() {
                    to_remove.push(id.clone());
                }
            }
        }

        // Remove inactive browsers
        if !to_remove.is_empty() {
            let mut browsers = self
                .browsers
                .write()
                .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;

            for id in to_remove {
                browsers.remove(&id);
            }
        }

        Ok(())
    }

    fn session_count(&self) -> usize {
        self.browsers
            .read()
            .map(|b| b.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_manager_creation() {
        let manager = SessionManagerImpl::mock();
        assert_eq!(manager.session_count(), 0);
    }

    #[tokio::test]
    async fn test_create_browser() {
        let manager = SessionManagerImpl::mock();
        let browser_id = manager.create_browser(BrowserOptions::default()).await.unwrap();

        assert!(!browser_id.is_empty());
        assert_eq!(manager.session_count(), 1);

        // Get browser
        let browser = manager.get_browser(&browser_id).await.unwrap();
        assert_eq!(browser.id(), &browser_id);
    }

    #[tokio::test]
    async fn test_list_browsers() {
        let manager = SessionManagerImpl::mock();

        manager.create_browser(BrowserOptions::default()).await.unwrap();
        manager.create_browser(BrowserOptions::default()).await.unwrap();

        let browsers = manager.list_browsers().await.unwrap();
        assert_eq!(browsers.len(), 2);
    }

    #[tokio::test]
    async fn test_close_browser() {
        let manager = SessionManagerImpl::mock();
        let browser_id = manager.create_browser(BrowserOptions::default()).await.unwrap();

        assert_eq!(manager.session_count(), 1);

        manager.close_browser(&browser_id).await.unwrap();
        assert_eq!(manager.session_count(), 0);

        // Should not be able to get browser after close
        let result = manager.get_browser(&browser_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_page() {
        let manager = SessionManagerImpl::mock();
        let browser_id = manager.create_browser(BrowserOptions::default()).await.unwrap();

        let page = manager
            .create_page(&browser_id, PageOptions::default())
            .await
            .unwrap();

        assert!(!page.id().is_empty());
        assert_eq!(page.browser_id(), &browser_id);
    }

    #[tokio::test]
    async fn test_get_page() {
        let manager = SessionManagerImpl::mock();
        let browser_id = manager.create_browser(BrowserOptions::default()).await.unwrap();

        let page = manager
            .create_page(&browser_id, PageOptions::default())
            .await
            .unwrap();

        let retrieved_page = manager.get_page(page.id()).await.unwrap();
        assert_eq!(retrieved_page.id(), page.id());
    }

    #[tokio::test]
    async fn test_close_page() {
        let manager = SessionManagerImpl::mock();
        let browser_id = manager.create_browser(BrowserOptions::default()).await.unwrap();

        let page = manager
            .create_page(&browser_id, PageOptions::default())
            .await
            .unwrap();

        manager.close_page(page.id()).await.unwrap();

        // Page should be inactive
        let retrieved_page = manager.get_page(page.id()).await.unwrap();
        assert!(!retrieved_page.is_active());
    }

    #[tokio::test]
    async fn test_cleanup() {
        let manager = SessionManagerImpl::mock();
        let browser_id = manager.create_browser(BrowserOptions::default()).await.unwrap();

        // Close browser
        manager.close_browser(&browser_id).await.unwrap();

        // Cleanup should remove the inactive browser
        manager.cleanup().await.unwrap();

        // Should not affect active count
        assert_eq!(manager.session_count(), 0);
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let manager = Arc::new(SessionManagerImpl::mock());
        let mut handles = Vec::new();

        // Create multiple browsers concurrently
        for _ in 0..10 {
            let manager_clone = manager.clone();
            handles.push(tokio::spawn(async move {
                manager_clone
                    .create_browser(BrowserOptions::default())
                    .await
            }));
        }

        // Wait for all to complete
        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        assert_eq!(manager.session_count(), 10);
    }

    #[tokio::test]
    async fn test_browser_not_found() {
        let manager = SessionManagerImpl::mock();
        let result = manager.get_browser("non-existent").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::BrowserNotFound(_)));
    }

    #[tokio::test]
    async fn test_page_not_found() {
        let manager = SessionManagerImpl::mock();
        let result = manager.get_page("non-existent").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::PageNotFound(_)));
    }
}
