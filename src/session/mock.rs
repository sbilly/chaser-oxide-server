//! Mock implementations for testing
//!
//! This module provides mock implementations of session management traits for testing purposes.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::cdp::traits::CdpClient;
use crate::session::traits::{
    BrowserContext, BrowserContextInfo, BrowserOptions, ElementRef, PageContext, PageOptions,
    NavigationOptions, NavigationResult, EvaluationResult, BoundingBox, ScreenshotOptions,
};

/// Mock session manager
#[derive(Debug)]
pub struct MockSessionManager {
    browsers: Arc<RwLock<HashMap<String, Arc<MockBrowser>>>>,
    pages: Arc<RwLock<HashMap<String, Arc<MockPage>>>>,
}

impl MockSessionManager {
    /// Create a new mock session manager
    pub fn new() -> Self {
        Self {
            browsers: Arc::new(RwLock::new(HashMap::new())),
            pages: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a browser to this manager
    pub async fn add_browser(&self, browser: Arc<MockBrowser>) {
        let id = browser.id().to_string();
        self.browsers.write().await.insert(id, browser);
    }

    /// Get browser count
    pub async fn browser_count(&self) -> usize {
        self.browsers.read().await.len()
    }
}

impl Default for MockSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::session::traits::SessionManager for MockSessionManager {
    async fn create_browser(&self, options: BrowserOptions) -> Result<String, crate::Error> {
        let browser = Arc::new(MockBrowser::new(options));
        let id = browser.id().to_string();
        self.add_browser(browser).await;
        Ok(id)
    }

    async fn get_browser(&self, browser_id: &str) -> Result<Arc<dyn BrowserContext>, crate::Error> {
        let browsers = self.browsers.read().await;
        browsers
            .get(browser_id)
            .map(|b| b.clone() as Arc<dyn BrowserContext>)
            .ok_or_else(|| crate::Error::browser_not_found(browser_id))
    }

    async fn close_browser(&self, browser_id: &str) -> Result<(), crate::Error> {
        let browser = self.get_browser(browser_id).await?;
        browser.close().await?;
        self.browsers.write().await.remove(browser_id);
        Ok(())
    }

    async fn list_browsers(&self) -> Result<Vec<String>, crate::Error> {
        let browsers = self.browsers.read().await;
        Ok(browsers.keys().cloned().collect())
    }

    async fn create_page(
        &self,
        browser_id: &str,
        options: PageOptions,
    ) -> Result<Arc<dyn PageContext>, crate::Error> {
        let browser = self.get_browser(browser_id).await?;
        browser.create_page(options).await
    }

    async fn get_page(&self, page_id: &str) -> Result<Arc<dyn PageContext>, crate::Error> {
        let pages = self.pages.read().await;
        pages
            .get(page_id)
            .map(|p| p.clone() as Arc<dyn PageContext>)
            .ok_or_else(|| crate::Error::page_not_found(page_id))
    }

    async fn close_page(&self, page_id: &str) -> Result<(), crate::Error> {
        let page = self.get_page(page_id).await?;
        page.close().await?;
        self.pages.write().await.remove(page_id);
        Ok(())
    }

    async fn cleanup(&self) -> Result<(), crate::Error> {
        Ok(())
    }

    fn session_count(&self) -> usize {
        self.browsers
            .try_read()
            .map(|b| b.len())
            .unwrap_or(0)
    }
}

/// Mock browser context
#[derive(Debug)]
pub struct MockBrowser {
    id: String,
    options: BrowserOptions,
    pages: Arc<RwLock<Vec<Arc<MockPage>>>>,
    is_active: Arc<RwLock<bool>>,
    created_at: std::time::Instant,
    incognito_contexts: Arc<RwLock<HashMap<String, BrowserContextInfo>>>,
}

impl MockBrowser {
    /// Create a new mock browser
    pub fn new(options: BrowserOptions) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            options,
            pages: Arc::new(RwLock::new(Vec::new())),
            is_active: Arc::new(RwLock::new(true)),
            created_at: std::time::Instant::now(),
            incognito_contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a page to this browser
    pub async fn add_page(&self, page: Arc<MockPage>) {
        self.pages.write().await.push(page);
    }

    /// Get uptime in milliseconds
    pub fn uptime_ms(&self) -> u64 {
        self.created_at.elapsed().as_millis() as u64
    }

    /// Get page count
    pub async fn page_count(&self) -> usize {
        self.pages.read().await.len()
    }
}

#[async_trait]
impl BrowserContext for MockBrowser {
    fn id(&self) -> &str {
        &self.id
    }

    async fn create_page(&self, options: PageOptions) -> Result<Arc<dyn PageContext>, crate::Error> {
        let page = Arc::new(MockPage::new(
            self.id.clone(),
            options,
        ));
        self.add_page(page.clone()).await;
        Ok(page)
    }

    async fn get_pages(&self) -> Result<Vec<Arc<dyn PageContext>>, crate::Error> {
        let pages = self.pages.read().await;
        Ok(pages.iter().map(|p| p.clone() as Arc<dyn PageContext>).collect())
    }

    async fn close(&self) -> Result<(), crate::Error> {
        *self.is_active.write().await = false;
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

    async fn create_incognito_context(&self) -> Result<BrowserContextInfo, crate::Error> {
        let context_id = Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().timestamp();

        let context_info = BrowserContextInfo {
            context_id: context_id.clone(),
            browser_id: self.id.clone(),
            is_incognito: true,
            created_at,
        };

        self.incognito_contexts
            .write()
            .await
            .insert(context_id.clone(), context_info.clone());

        Ok(context_info)
    }

    async fn close_incognito_context(&self, context_id: &str) -> Result<(), crate::Error> {
        let removed = self.incognito_contexts.write().await.remove(context_id);

        if removed.is_some() {
            Ok(())
        } else {
            Err(crate::Error::browser_not_found(&format!(
                "Incognito context {} not found",
                context_id
            )))
        }
    }

    async fn get_contexts(&self) -> Result<Vec<BrowserContextInfo>, crate::Error> {
        let contexts = self.incognito_contexts.read().await.values().cloned().collect();
        Ok(contexts)
    }
}

/// Mock page context
#[derive(Debug)]
pub struct MockPage {
    id: String,
    browser_id: String,
    #[allow(dead_code)]
    options: PageOptions,
    url: Arc<RwLock<String>>,
    title: Arc<RwLock<String>>,
    content: Arc<RwLock<String>>,
    is_active: Arc<RwLock<bool>>,
    viewport: Arc<RwLock<(u32, u32, f64)>>,
    cdp_client: Arc<dyn crate::cdp::traits::CdpClient>,
}

impl MockPage {
    /// Create a new mock page
    pub fn new(browser_id: String, options: PageOptions) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            browser_id,
            options,
            url: Arc::new(RwLock::new("about:blank".to_string())),
            title: Arc::new(RwLock::new(String::new())),
            content: Arc::new(RwLock::new(String::new())),
            is_active: Arc::new(RwLock::new(true)),
            viewport: Arc::new(RwLock::new((1920, 1080, 1.0))),
            cdp_client: Arc::new(crate::cdp::mock::MockCdpClient::new()),
        }
    }

    /// Set URL (for testing)
    pub async fn set_url(&self, url: String) {
        *self.url.write().await = url;
    }

    /// Set title (for testing)
    pub async fn set_title(&self, title: String) {
        *self.title.write().await = title;
    }

    /// Set content (for testing)
    pub async fn set_content_internal(&self, content: String) {
        *self.content.write().await = content;
    }
}

#[async_trait]
impl PageContext for MockPage {
    fn id(&self) -> &str {
        &self.id
    }

    fn browser_id(&self) -> &str {
        &self.browser_id
    }

    async fn navigate(&self, url: &str, _options: NavigationOptions) -> Result<NavigationResult, crate::Error> {
        *self.url.write().await = url.to_string();
        Ok(NavigationResult {
            url: url.to_string(),
            status_code: 200,
            is_loaded: true,
        })
    }

    async fn get_content(&self) -> Result<String, crate::Error> {
        Ok(self.content.read().await.clone())
    }

    async fn set_content(&self, html: &str) -> Result<(), crate::Error> {
        *self.content.write().await = html.to_string();
        Ok(())
    }

    async fn reload(&self, _ignore_cache: bool) -> Result<(), crate::Error> {
        Ok(())
    }

    async fn go_back(&self) -> Result<(), crate::Error> {
        Ok(())
    }

    async fn go_forward(&self) -> Result<(), crate::Error> {
        Ok(())
    }

    async fn evaluate(&self, script: &str, _await_promise: bool) -> Result<EvaluationResult, crate::Error> {
        // Simple mock implementation
        if script.contains("return") {
            Ok(EvaluationResult::String("mock result".to_string()))
        } else {
            Ok(EvaluationResult::Null)
        }
    }

    async fn screenshot(&self, _options: ScreenshotOptions) -> Result<Vec<u8>, crate::Error> {
        Ok(vec![1, 2, 3, 4]) // Mock screenshot data
    }

    async fn set_viewport(&self, width: u32, height: u32, device_scale_factor: f64) -> Result<(), crate::Error> {
        *self.viewport.write().await = (width, height, device_scale_factor);
        Ok(())
    }

    async fn close(&self) -> Result<(), crate::Error> {
        *self.is_active.write().await = false;
        Ok(())
    }

    fn is_active(&self) -> bool {
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

/// Mock element reference
#[derive(Debug)]
pub struct MockElement {
    id: String,
    page_id: String,
    text: Arc<RwLock<String>>,
    is_visible: Arc<RwLock<bool>>,
    is_enabled: Arc<RwLock<bool>>,
}

impl MockElement {
    /// Create a new mock element
    pub fn new(page_id: String, text: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            page_id,
            text: Arc::new(RwLock::new(text)),
            is_visible: Arc::new(RwLock::new(true)),
            is_enabled: Arc::new(RwLock::new(true)),
        }
    }
}

#[async_trait]
impl ElementRef for MockElement {
    fn id(&self) -> &str {
        &self.id
    }

    fn page_id(&self) -> &str {
        &self.page_id
    }

    async fn get_text(&self) -> Result<String, crate::Error> {
        Ok(self.text.read().await.clone())
    }

    async fn get_html(&self) -> Result<String, crate::Error> {
        let text = self.text.read().await.clone();
        Ok(format!("<div>{}</div>", text))
    }

    async fn get_attribute(&self, _name: &str) -> Result<Option<String>, crate::Error> {
        Ok(None)
    }

    async fn click(&self) -> Result<(), crate::Error> {
        Ok(())
    }

    async fn type_text(&self, _text: &str) -> Result<(), crate::Error> {
        Ok(())
    }

    async fn focus(&self) -> Result<(), crate::Error> {
        Ok(())
    }

    async fn hover(&self) -> Result<(), crate::Error> {
        Ok(())
    }

    async fn scroll_into_view(&self) -> Result<(), crate::Error> {
        Ok(())
    }

    async fn is_visible(&self) -> Result<bool, crate::Error> {
        Ok(*self.is_visible.read().await)
    }

    async fn is_enabled(&self) -> Result<bool, crate::Error> {
        Ok(*self.is_enabled.read().await)
    }

    async fn get_bounding_box(&self) -> Result<BoundingBox, crate::Error> {
        Ok(BoundingBox {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::traits::SessionManager;

    #[tokio::test]
    async fn test_mock_session_manager() {
        let manager = MockSessionManager::new();

        // Create browser
        let browser_id = manager
            .create_browser(BrowserOptions::default())
            .await
            .unwrap();
        assert_eq!(manager.browser_count().await, 1);

        // Get browser
        let browser = manager.get_browser(&browser_id).await.unwrap();
        assert_eq!(browser.id(), browser_id);

        // Create page
        let page = browser
            .create_page(PageOptions::default())
            .await
            .unwrap();
        assert_eq!(page.browser_id(), browser_id);

        // Close browser
        manager.close_browser(&browser_id).await.unwrap();
        assert_eq!(manager.browser_count().await, 0);
    }

    #[tokio::test]
    async fn test_mock_browser() {
        let browser = MockBrowser::new(BrowserOptions::default());

        // Create page
        let page = browser.create_page(PageOptions::default()).await.unwrap();
        assert_eq!(page.browser_id(), browser.id());

        // Get pages
        let pages = browser.get_pages().await.unwrap();
        assert_eq!(pages.len(), 1);

        // Close
        browser.close().await.unwrap();
        assert!(!browser.is_active());
    }

    #[tokio::test]
    async fn test_mock_page() {
        let page = MockPage::new("test-browser".to_string(), PageOptions::default());

        // Navigate
        let result = page
            .navigate("https://example.com", NavigationOptions::default())
            .await
            .unwrap();
        assert_eq!(result.url, "https://example.com");

        // Content
        page.set_content("Hello").await.unwrap();
        let content = page.get_content().await.unwrap();
        assert_eq!(content, "Hello");

        // Close
        page.close().await.unwrap();
        assert!(!page.is_active());
    }

    #[tokio::test]
    async fn test_incognito_context() {
        let browser = MockBrowser::new(BrowserOptions::default());

        // Create incognito context
        let context = browser.create_incognito_context().await.unwrap();
        assert!(context.is_incognito);
        assert_eq!(context.browser_id, browser.id());

        // List contexts
        let contexts = browser.get_contexts().await.unwrap();
        assert_eq!(contexts.len(), 1);

        // Close context
        browser.close_incognito_context(&context.context_id).await.unwrap();
        let contexts = browser.get_contexts().await.unwrap();
        assert_eq!(contexts.len(), 0);
    }
}
