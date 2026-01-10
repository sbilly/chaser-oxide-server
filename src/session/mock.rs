//! Mock session implementation for testing
//!
//! This module provides mock implementations of session traits for development and testing.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::traits::{
    BrowserContext, BrowserOptions, PageContext, PageOptions, ElementRef,
    NavigationOptions, NavigationResult, EvaluationResult, BoundingBox,
    ScreenshotOptions, SessionManager,
};
use crate::Error;

/// Mock session manager
#[derive(Debug, Clone)]
pub struct MockSessionManager {
    browsers: Arc<RwLock<HashMap<String, Arc<MockBrowser>>>>,
    pages: Arc<RwLock<HashMap<String, Arc<dyn PageContext>>>>,
}

impl MockSessionManager {
    /// Create a new mock session manager
    pub fn new() -> Self {
        Self {
            browsers: Arc::new(RwLock::new(HashMap::new())),
            pages: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a browser
    pub async fn register_browser(&self, browser: Arc<MockBrowser>) -> String {
        let id = browser.id().to_string();
        self.browsers.write().await.insert(id.clone(), browser);
        id
    }

    /// Register a page
    pub async fn register_page(&self, page: Arc<MockPage>) -> String {
        let id = page.id().to_string();
        self.pages.write().await.insert(id.clone(), page);
        id
    }
}

impl Default for MockSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionManager for MockSessionManager {
    async fn create_browser(&self, options: BrowserOptions) -> Result<String, crate::Error> {
        let browser = Arc::new(MockBrowser::new(options));
        let id = self.register_browser(browser).await;
        Ok(id)
    }

    async fn get_browser(&self, browser_id: &str) -> Result<Arc<dyn BrowserContext>, crate::Error> {
        self.browsers
            .read()
            .await
            .get(browser_id)
            .map(|b| b.clone() as Arc<dyn BrowserContext>)
            .ok_or_else(|| crate::Error::BrowserNotFound(browser_id.to_string()))
    }

    async fn close_browser(&self, browser_id: &str) -> Result<(), crate::Error> {
        self.browsers
            .write()
            .await
            .remove(browser_id)
            .ok_or_else(|| crate::Error::BrowserNotFound(browser_id.to_string()))?;
        Ok(())
    }

    async fn list_browsers(&self) -> Result<Vec<String>, crate::Error> {
        Ok(self.browsers.read().await.keys().cloned().collect())
    }

    async fn create_page(
        &self,
        browser_id: &str,
        options: PageOptions,
    ) -> Result<Arc<dyn PageContext>, crate::Error> {
        let browser = self.get_browser(browser_id).await?;
        let page = browser.create_page(options).await?;
        let page_id = page.id().to_string();
        self.pages.write().await.insert(page_id.clone(), page.clone());
        Ok(page)
    }

    async fn get_page(&self, page_id: &str) -> Result<Arc<dyn PageContext>, crate::Error> {
        self.pages
            .read()
            .await
            .get(page_id)
            .cloned()
            .ok_or_else(|| crate::Error::PageNotFound(page_id.to_string()))
    }

    async fn close_page(&self, page_id: &str) -> Result<(), crate::Error> {
        self.pages
            .write()
            .await
            .remove(page_id)
            .ok_or_else(|| crate::Error::PageNotFound(page_id.to_string()))?;
        Ok(())
    }

    async fn cleanup(&self) -> Result<(), crate::Error> {
        // Remove inactive browsers
        let mut browsers = self.browsers.write().await;
        browsers.retain(|_, b| b.is_active());
        Ok(())
    }

    fn session_count(&self) -> usize {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.browsers.read().await.len()
            })
        })
    }
}

/// Mock browser context
#[derive(Debug)]
pub struct MockBrowser {
    id: String,
    #[allow(dead_code)]
    options: BrowserOptions,
    pages: Arc<RwLock<Vec<Arc<MockPage>>>>,
    is_active: Arc<RwLock<bool>>,
    created_at: std::time::Instant,
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

    async fn create_page(&self, options: PageOptions) -> Result<Arc<dyn PageContext>, Error> {
        let page = Arc::new(MockPage::new(
            self.id.clone(),
            options,
        ));
        self.add_page(page.clone()).await;
        Ok(page)
    }

    async fn get_pages(&self) -> Result<Vec<Arc<dyn PageContext>>, Error> {
        let pages = self.pages.read().await;
        Ok(pages.iter().map(|p| p.clone() as Arc<dyn PageContext>).collect())
    }

    async fn close(&self) -> Result<(), Error> {
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

    async fn navigate(&self, url: &str, _options: NavigationOptions) -> Result<NavigationResult, Error> {
        *self.url.write().await = url.to_string();
        Ok(NavigationResult {
            url: url.to_string(),
            status_code: 200,
            is_loaded: true,
        })
    }

    async fn get_content(&self) -> Result<String, Error> {
        Ok(self.content.read().await.clone())
    }

    async fn set_content(&self, html: &str) -> Result<(), Error> {
        *self.content.write().await = html.to_string();
        Ok(())
    }

    async fn reload(&self, _ignore_cache: bool) -> Result<(), Error> {
        Ok(())
    }

    async fn go_back(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn go_forward(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn evaluate(&self, script: &str, _await_promise: bool) -> Result<EvaluationResult, Error> {
        // Simple mock: handle basic cases for testing
        if script == "document.title" {
            Ok(EvaluationResult::String("Test Page".to_string()))
        } else if script.contains("+") {
            // Simple arithmetic evaluation
            let parts: Vec<&str> = script.split('+').collect();
            if parts.len() == 2 {
                let a: f64 = parts[0].trim().parse().unwrap_or(0.0);
                let b: f64 = parts[1].trim().parse().unwrap_or(0.0);
                Ok(EvaluationResult::Number(a + b))
            } else {
                Ok(EvaluationResult::String(script.to_string()))
            }
        } else {
            Ok(EvaluationResult::String(script.to_string()))
        }
    }

    async fn screenshot(&self, _options: ScreenshotOptions) -> Result<Vec<u8>, Error> {
        // Return a minimal 1x1 PNG
        Ok(vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, // IHDR length
            0x49, 0x48, 0x44, 0x52, // IHDR
            0x00, 0x00, 0x00, 0x01, // Width: 1
            0x00, 0x00, 0x00, 0x01, // Height: 1
            0x08, 0x02, 0x00, 0x00, 0x00, // Bit depth: 8, Color type: 2 (RGB)
            0x90, 0x77, 0x53, 0xDE, // CRC
        ])
    }

    async fn set_viewport(&self, width: u32, height: u32, device_scale_factor: f64) -> Result<(), Error> {
        *self.viewport.write().await = (width, height, device_scale_factor);
        Ok(())
    }

    async fn close(&self) -> Result<(), Error> {
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

    fn get_cdp_client(&self) -> Arc<dyn crate::cdp::traits::CdpClient> {
        self.cdp_client.clone()
    }
}

/// Mock element reference
#[derive(Debug)]
pub struct MockElement {
    id: String,
    page_id: String,
    tag_name: String,
    text_content: Option<String>,
}

impl MockElement {
    /// Create a new mock element
    pub fn new(page_id: String, tag_name: String, text_content: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            page_id,
            tag_name,
            text_content,
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

    async fn get_text(&self) -> Result<String, Error> {
        Ok(self.text_content.clone().unwrap_or_default())
    }

    async fn get_html(&self) -> Result<String, Error> {
        Ok(format!("<{}></{}>", self.tag_name, self.tag_name))
    }

    async fn get_attribute(&self, _name: &str) -> Result<Option<String>, Error> {
        Ok(None)
    }

    async fn click(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn type_text(&self, _text: &str) -> Result<(), Error> {
        Ok(())
    }

    async fn focus(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn hover(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn scroll_into_view(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn is_visible(&self) -> Result<bool, Error> {
        Ok(true)
    }

    async fn is_enabled(&self) -> Result<bool, Error> {
        Ok(true)
    }

    async fn get_bounding_box(&self) -> Result<BoundingBox, Error> {
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

    #[tokio::test]
    async fn test_mock_browser() {
        let options = BrowserOptions::default();
        let browser = MockBrowser::new(options);

        assert!(!browser.id().is_empty());
        assert!(browser.is_active());

        let pages = browser.get_pages().await.unwrap();
        assert_eq!(pages.len(), 0);

        browser.close().await.unwrap();
        assert!(!browser.is_active());
    }

    #[tokio::test]
    async fn test_mock_page() {
        let browser_id = "test-browser".to_string();
        let options = PageOptions::default();
        let page = MockPage::new(browser_id, options);

        assert!(!page.id().is_empty());
        assert_eq!(page.browser_id(), "test-browser");
        assert!(page.is_active());

        // Test navigation
        let result = page.navigate("https://example.com", NavigationOptions {
            timeout: 30000,
            wait_until: super::super::traits::LoadState::Load,
        }).await.unwrap();
        assert_eq!(result.url, "https://example.com");

        // Test content
        page.set_content("<html><body>Test</body></html>").await.unwrap();
        let content = page.get_content().await.unwrap();
        assert!(content.contains("Test"));

        // Test close
        page.close().await.unwrap();
        assert!(!page.is_active());
    }

    #[tokio::test]
    async fn test_mock_element() {
        let page_id = "test-page".to_string();
        let element = MockElement::new(page_id, "div".to_string(), Some("Test text".to_string()));

        assert!(!element.id().is_empty());
        assert_eq!(element.page_id(), "test-page");

        let text = element.get_text().await.unwrap();
        assert_eq!(text, "Test text");

        let html = element.get_html().await.unwrap();
        assert_eq!(html, "<div></div>");

        let bbox = element.get_bounding_box().await.unwrap();
        assert_eq!(bbox.width, 100.0);
    }
}
