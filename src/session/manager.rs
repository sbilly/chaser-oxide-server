//! Session manager implementation
//!
//! Manages all browser, page, and element sessions with thread-safe operations.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::cdp::browser::CdpBrowserImpl;
use crate::cdp::traits::CdpBrowser;
use crate::process::ProcessManager;
use crate::session::traits::{
    BrowserContext, BrowserOptions, PageContext, PageOptions, SessionManager,
};
use crate::Error;

/// Lock guard type alias for read operations on browsers map
type BrowsersReadGuard<'a> = std::sync::RwLockReadGuard<'a, HashMap<String, Arc<dyn BrowserContext>>>;

/// Lock guard type alias for write operations on browsers map
type BrowsersWriteGuard<'a> = std::sync::RwLockWriteGuard<'a, HashMap<String, Arc<dyn BrowserContext>>>;

/// Session manager implementation
pub struct SessionManagerImpl {
    browsers: Arc<RwLock<HashMap<String, Arc<dyn BrowserContext>>>>,
    cdp_browser_factory: Arc<dyn Fn() -> Result<Arc<dyn CdpBrowser>, Error> + Send + Sync>,
    process_manager: Option<Arc<ProcessManager>>,
}

impl SessionManagerImpl {
    /// Acquire read lock on browsers map with consistent error handling
    fn read_browsers(&self) -> Result<BrowsersReadGuard<'_>, Error> {
        self.browsers
            .read()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))
    }

    /// Acquire write lock on browsers map with consistent error handling
    fn write_browsers(&self) -> Result<BrowsersWriteGuard<'_>, Error> {
        self.browsers
            .write()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))
    }
    /// Create a new session manager
    pub fn new<F>(factory: F) -> Self
    where
        F: Fn() -> Result<Arc<dyn CdpBrowser>, Error> + Send + Sync + 'static,
    {
        Self {
            browsers: Arc::new(RwLock::new(HashMap::new())),
            cdp_browser_factory: Arc::new(factory),
            process_manager: None,
        }
    }

    /// Create a new session manager with ProcessManager support
    ///
    /// This allows the session manager to automatically launch and manage
    /// Chrome browser processes.
    ///
    /// # Arguments
    ///
    /// * `factory` - CDP browser factory function
    /// * `process_manager` - Process manager for launching browsers
    pub fn with_process_manager<F>(
        factory: F,
        process_manager: Arc<ProcessManager>,
    ) -> Self
    where
        F: Fn() -> Result<Arc<dyn CdpBrowser>, Error> + Send + Sync + 'static,
    {
        Self {
            browsers: Arc::new(RwLock::new(HashMap::new())),
            cdp_browser_factory: Arc::new(factory),
            process_manager: Some(process_manager),
        }
    }

    /// Create a session manager with a mock CDP browser for testing
    pub fn mock() -> Self {
        Self::new(|| Ok(Arc::new(crate::cdp::mock::MockCdpBrowser::new())))
    }

    /// Store a browser context in the manager
    fn insert_browser(&self, browser_id: String, browser: Arc<dyn BrowserContext>) -> Result<(), Error> {
        self.write_browsers()?.insert(browser_id, browser);
        Ok(())
    }

    /// Remove a browser from the manager by ID
    fn remove_browser(&self, browser_id: &str) -> Result<(), Error> {
        self.write_browsers()?.remove(browser_id);
        Ok(())
    }

    /// Collect IDs of all inactive browsers
    fn collect_inactive_browser_ids(&self) -> Result<Vec<String>, Error> {
        let browsers = self.read_browsers()?;
        Ok(browsers
            .iter()
            .filter(|(_, b)| !b.is_active())
            .map(|(id, _)| id.clone())
            .collect())
    }

    /// Find a page across all browsers by its ID
    async fn find_page(&self, page_id: &str) -> Result<Arc<dyn PageContext>, Error> {
        // Collect browser Arcs first to avoid holding lock across await
        let browsers: Vec<Arc<dyn BrowserContext>> = self
            .read_browsers()?
            .values()
            .cloned()
            .collect();

        for browser in browsers {
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

    /// Create a CDP browser, either via process manager or factory
    async fn create_cdp_browser(&self) -> Result<(String, Arc<dyn CdpBrowser>), Error> {
        if let Some(pm) = &self.process_manager {
            let (id, cdp_endpoint) = pm.launch_browser().await?;
            let cdp = Arc::new(CdpBrowserImpl::new(cdp_endpoint)) as Arc<dyn CdpBrowser>;
            Ok((id, cdp))
        } else {
            let cdp = (self.cdp_browser_factory)()?;
            let browser_id = uuid::Uuid::new_v4().to_string();
            Ok((browser_id, cdp))
        }
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
        let (browser_id, cdp_browser) = self.create_cdp_browser().await?;

        let browser = Arc::new(crate::session::browser::BrowserContextImpl::with_id(
            browser_id.clone(),
            options,
            cdp_browser,
        ));

        self.insert_browser(browser_id.clone(), browser)?;
        Ok(browser_id)
    }

    async fn get_browser(&self, browser_id: &str) -> Result<Arc<dyn BrowserContext>, Error> {
        self.read_browsers()?
            .get(browser_id)
            .cloned()
            .ok_or_else(|| Error::browser_not_found(browser_id))
    }

    async fn close_browser(&self, browser_id: &str) -> Result<(), Error> {
        let browser = self.get_browser(browser_id).await?;
        browser.close().await?;

        if let Some(pm) = &self.process_manager {
            pm.terminate_browser(browser_id).await?;
        }

        self.remove_browser(browser_id)
    }

    async fn list_browsers(&self) -> Result<Vec<String>, Error> {
        Ok(self.read_browsers()?.keys().cloned().collect())
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
        self.find_page(page_id).await
    }

    async fn close_page(&self, page_id: &str) -> Result<(), Error> {
        let page = self.get_page(page_id).await?;
        page.close().await
    }

    async fn cleanup(&self) -> Result<(), Error> {
        let to_remove = self.collect_inactive_browser_ids()?;

        if !to_remove.is_empty() {
            let mut browsers = self.write_browsers()?;
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
