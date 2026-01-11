//! Session management traits
//!
//! This module defines the abstract interfaces for managing browser, page, and element sessions.

use async_trait::async_trait;
use std::sync::Arc;

/// Browser options for launching a browser
#[derive(Debug, Clone)]
pub struct BrowserOptions {
    /// Headless mode (no GUI)
    pub headless: bool,
    /// Window width
    pub window_width: u32,
    /// Window height
    pub window_height: u32,
    /// User agent string
    pub user_agent: Option<String>,
    /// Proxy server
    pub proxy: Option<String>,
    /// Additional arguments to pass to Chrome
    pub args: Vec<String>,
    /// Chrome executable path
    pub executable_path: Option<String>,
    /// CDP endpoint (e.g., "ws://localhost:9222" or from CHASER_CDP_ENDPOINT env var)
    pub cdp_endpoint: Option<String>,
}

impl Default for BrowserOptions {
    fn default() -> Self {
        Self {
            headless: true,
            window_width: 1920,
            window_height: 1080,
            user_agent: None,
            proxy: None,
            args: vec![],
            executable_path: None,
            cdp_endpoint: None,
        }
    }
}

/// Page options for creating a new page
#[derive(Debug, Clone)]
pub struct PageOptions {
    /// Default URL
    pub default_url: Option<String>,
    /// Viewport width
    pub viewport_width: u32,
    /// Viewport height
    pub viewport_height: u32,
    /// Device scale factor
    pub device_scale_factor: f64,
    /// Mobile emulation
    pub is_mobile: bool,
}

impl Default for PageOptions {
    fn default() -> Self {
        Self {
            default_url: Some("about:blank".to_string()),
            viewport_width: 1920,
            viewport_height: 1080,
            device_scale_factor: 1.0,
            is_mobile: false,
        }
    }
}

/// Screenshot options
#[derive(Debug, Clone)]
pub struct ScreenshotOptions {
    /// Screenshot format
    pub format: ScreenshotFormat,
    /// Quality for JPEG/WebP (0-100)
    pub quality: Option<u8>,
    /// Capture full page
    pub full_page: bool,
    /// Clip region
    pub clip: Option<ClipRegion>,
}

impl Default for ScreenshotOptions {
    fn default() -> Self {
        Self {
            format: ScreenshotFormat::Png,
            quality: None,
            full_page: false,
            clip: None,
        }
    }
}

/// Screenshot format
#[derive(Debug, Clone, Copy, Default)]
pub enum ScreenshotFormat {
    #[default]
    Png,
    Jpeg,
    WebP,
}

/// Clip region for screenshots
#[derive(Debug, Clone)]
pub struct ClipRegion {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub scale: f64,
}

/// Navigation options
#[derive(Debug, Clone)]
pub struct NavigationOptions {
    /// Timeout in milliseconds
    pub timeout: u64,
    /// Wait until condition
    pub wait_until: LoadState,
}

impl Default for NavigationOptions {
    fn default() -> Self {
        Self {
            timeout: 30000,
            wait_until: LoadState::Load,
        }
    }
}

/// Page load state
#[derive(Debug, Clone, Copy)]
pub enum LoadState {
    Load,
    DOMContentLoaded,
    NetworkIdle,
    NetworkAlmostIdle,
}

/// Browser context trait
///
/// Represents a running browser instance.
#[async_trait]
pub trait BrowserContext: Send + Sync + std::fmt::Debug {
    /// Get browser ID
    fn id(&self) -> &str;

    /// Create a new page
    async fn create_page(&self, options: PageOptions) -> Result<Arc<dyn PageContext>, crate::Error>;

    /// Get all pages
    async fn get_pages(&self) -> Result<Vec<Arc<dyn PageContext>>, crate::Error>;

    /// Close the browser
    async fn close(&self) -> Result<(), crate::Error>;

    /// Check if browser is active
    fn is_active(&self) -> bool;
}

/// Page context trait
///
/// Represents a page/tab in a browser.
#[async_trait]
pub trait PageContext: Send + Sync + std::fmt::Debug {
    /// Get page ID
    fn id(&self) -> &str;

    /// Get parent browser ID
    fn browser_id(&self) -> &str;

    /// Navigate to URL
    async fn navigate(&self, url: &str, options: NavigationOptions) -> Result<NavigationResult, crate::Error>;

    /// Get page content
    async fn get_content(&self) -> Result<String, crate::Error>;

    /// Set page content
    async fn set_content(&self, html: &str) -> Result<(), crate::Error>;

    /// Reload page
    async fn reload(&self, ignore_cache: bool) -> Result<(), crate::Error>;

    /// Go back in history
    async fn go_back(&self) -> Result<(), crate::Error>;

    /// Go forward in history
    async fn go_forward(&self) -> Result<(), crate::Error>;

    /// Evaluate JavaScript
    async fn evaluate(&self, script: &str, await_promise: bool) -> Result<EvaluationResult, crate::Error>;

    /// Capture screenshot
    async fn screenshot(&self, options: ScreenshotOptions) -> Result<Vec<u8>, crate::Error>;

    /// Set viewport size
    async fn set_viewport(&self, width: u32, height: u32, device_scale_factor: f64) -> Result<(), crate::Error>;

    /// Close the page
    async fn close(&self) -> Result<(), crate::Error>;

    /// Check if page is active
    fn is_active(&self) -> bool;

    /// Get the CDP client for this page
    fn get_cdp_client(&self) -> Arc<dyn crate::cdp::traits::CdpClient>;
}

/// Element reference trait
///
/// Represents a DOM element in a page.
#[async_trait]
pub trait ElementRef: Send + Sync {
    /// Get element ID
    fn id(&self) -> &str;

    /// Get parent page ID
    fn page_id(&self) -> &str;

    /// Get element text
    async fn get_text(&self) -> Result<String, crate::Error>;

    /// Get element HTML
    async fn get_html(&self) -> Result<String, crate::Error>;

    /// Get element attribute
    async fn get_attribute(&self, name: &str) -> Result<Option<String>, crate::Error>;

    /// Click element
    async fn click(&self) -> Result<(), crate::Error>;

    /// Type text into element
    async fn type_text(&self, text: &str) -> Result<(), crate::Error>;

    /// Focus element
    async fn focus(&self) -> Result<(), crate::Error>;

    /// Hover over element
    async fn hover(&self) -> Result<(), crate::Error>;

    /// Scroll element into view
    async fn scroll_into_view(&self) -> Result<(), crate::Error>;

    /// Check if element is visible
    async fn is_visible(&self) -> Result<bool, crate::Error>;

    /// Check if element is enabled
    async fn is_enabled(&self) -> Result<bool, crate::Error>;

    /// Get element bounding box
    async fn get_bounding_box(&self) -> Result<BoundingBox, crate::Error>;
}

/// Element bounding box
#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Navigation result
#[derive(Debug, Clone)]
pub struct NavigationResult {
    pub url: String,
    pub status_code: u16,
    pub is_loaded: bool,
}

/// JavaScript evaluation result
#[derive(Debug, Clone)]
pub enum EvaluationResult {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
    Object(serde_json::Value),
}

/// Session manager trait
///
/// Manages all browser, page, and element sessions.
#[async_trait]
pub trait SessionManager: Send + Sync {
    /// Create a new browser
    async fn create_browser(&self, options: BrowserOptions) -> Result<String, crate::Error>;

    /// Get a browser by ID
    async fn get_browser(&self, browser_id: &str) -> Result<Arc<dyn BrowserContext>, crate::Error>;

    /// Close a browser
    async fn close_browser(&self, browser_id: &str) -> Result<(), crate::Error>;

    /// List all browsers
    async fn list_browsers(&self) -> Result<Vec<String>, crate::Error>;

    /// Create a page in a browser
    async fn create_page(
        &self,
        browser_id: &str,
        options: PageOptions,
    ) -> Result<Arc<dyn PageContext>, crate::Error>;

    /// Get a page by ID
    async fn get_page(&self, page_id: &str) -> Result<Arc<dyn PageContext>, crate::Error>;

    /// Close a page
    async fn close_page(&self, page_id: &str) -> Result<(), crate::Error>;

    /// Clean up closed sessions
    async fn cleanup(&self) -> Result<(), crate::Error>;

    /// Get session count
    fn session_count(&self) -> usize;
}
