//! Service layer traits
//!
//! This module defines the abstract interfaces for gRPC services.

use async_trait::async_trait;
use super::super::session::traits as session;

// ============================================================================
// Browser Service
// ============================================================================

/// Browser service trait
///
/// Provides browser lifecycle management operations.
#[async_trait]
pub trait BrowserService: Send + Sync {
    /// Launch a new browser instance
    async fn launch(&self, options: session::BrowserOptions) -> Result<BrowserInfo, crate::Error>;

    /// Close a browser instance
    async fn close(&self, browser_id: &str) -> Result<(), crate::Error>;

    /// Connect to an existing browser
    async fn connect(&self, endpoint: &str) -> Result<String, crate::Error>;

    /// Get browser version
    async fn get_version(&self, browser_id: &str) -> Result<BrowserVersion, crate::Error>;

    /// Get browser status
    async fn get_status(&self, browser_id: &str) -> Result<BrowserStatus, crate::Error>;

    /// Get all pages in a browser
    async fn get_pages(&self, browser_id: &str) -> Result<Vec<PageInfo>, crate::Error>;
}

/// Browser information
#[derive(Debug, Clone)]
pub struct BrowserInfo {
    pub browser_id: String,
    pub user_agent: String,
    pub cdp_endpoint: String,
}

/// Browser version information
#[derive(Debug, Clone)]
pub struct BrowserVersion {
    pub protocol_version: String,
    pub product: String,
    pub revision: String,
    pub user_agent: String,
    pub js_version: String,
}

/// Browser status
#[derive(Debug, Clone)]
pub struct BrowserStatus {
    pub is_active: bool,
    pub page_count: usize,
    pub uptime_ms: u64,
}

/// Page information
#[derive(Debug, Clone)]
pub struct PageInfo {
    pub page_id: String,
    pub url: String,
    pub title: String,
}

// ============================================================================
// Page Service
// ============================================================================

/// Page service trait
///
/// Provides page-level operations.
#[async_trait]
pub trait PageService: Send + Sync {
    /// Create a new page
    async fn create_page(
        &self,
        browser_id: &str,
        url: Option<String>,
    ) -> Result<PageInfo, crate::Error>;

    /// Navigate to URL
    async fn navigate(
        &self,
        page_id: &str,
        url: &str,
        options: session::NavigationOptions,
    ) -> Result<NavigationResult, crate::Error>;

    /// Get page snapshot
    async fn get_snapshot(&self, page_id: &str) -> Result<PageSnapshot, crate::Error>;

    /// Take screenshot
    async fn screenshot(
        &self,
        page_id: &str,
        options: session::ScreenshotOptions,
    ) -> Result<ScreenshotData, crate::Error>;

    /// Evaluate JavaScript
    async fn evaluate(
        &self,
        page_id: &str,
        expression: &str,
        await_promise: bool,
    ) -> Result<EvaluationResult, crate::Error>;

    /// Set page content
    async fn set_content(&self, page_id: &str, html: &str) -> Result<(), crate::Error>;

    /// Get page content
    async fn get_content(&self, page_id: &str) -> Result<String, crate::Error>;

    /// Reload page
    async fn reload(&self, page_id: &str, ignore_cache: bool) -> Result<(), crate::Error>;

    /// Go back
    async fn go_back(&self, page_id: &str) -> Result<(), crate::Error>;

    /// Go forward
    async fn go_forward(&self, page_id: &str) -> Result<(), crate::Error>;

    /// Set viewport
    async fn set_viewport(
        &self,
        page_id: &str,
        width: u32,
        height: u32,
        device_scale_factor: f64,
    ) -> Result<(), crate::Error>;

    /// Close page
    async fn close_page(&self, page_id: &str) -> Result<(), crate::Error>;

    /// Wait for condition
    async fn wait_for(
        &self,
        page_id: &str,
        condition: WaitCondition,
        timeout: u64,
    ) -> Result<(), crate::Error>;
}

/// Page snapshot
#[derive(Debug, Clone)]
pub struct PageSnapshot {
    pub title: String,
    pub url: String,
    pub elements: Vec<ElementInfo>,
}

/// Screenshot data
#[derive(Debug, Clone)]
pub struct ScreenshotData {
    pub data: Vec<u8>,
    pub format: String,
}

/// Navigation result
#[derive(Debug, Clone)]
pub struct NavigationResult {
    pub url: String,
    pub status_code: u16,
    pub is_loaded: bool,
}

/// Evaluation result
#[derive(Debug, Clone)]
pub enum EvaluationResult {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
    Object(serde_json::Value),
}

/// Wait condition
#[derive(Debug, Clone)]
pub enum WaitCondition {
    Selector(String),
    Navigation,
    LoadState(session::LoadState),
}

// ============================================================================
// Element Service
// ============================================================================

/// Element service trait
///
/// Provides element interaction operations.
#[async_trait]
pub trait ElementService: Send + Sync {
    /// Find element
    async fn find_element(
        &self,
        page_id: &str,
        selector_type: SelectorType,
        selector: &str,
    ) -> Result<ElementInfo, crate::Error>;

    /// Find elements
    async fn find_elements(
        &self,
        page_id: &str,
        selector_type: SelectorType,
        selector: &str,
    ) -> Result<Vec<ElementInfo>, crate::Error>;

    /// Click element
    async fn click(&self, element_id: &str) -> Result<(), crate::Error>;

    /// Type text
    async fn type_text(&self, element_id: &str, text: &str) -> Result<(), crate::Error>;

    /// Fill element
    async fn fill(&self, element_id: &str, value: &str) -> Result<(), crate::Error>;

    /// Get attribute
    async fn get_attribute(&self, element_id: &str, name: &str) -> Result<Option<String>, crate::Error>;

    /// Get text
    async fn get_text(&self, element_id: &str) -> Result<String, crate::Error>;

    /// Get HTML
    async fn get_html(&self, element_id: &str) -> Result<String, crate::Error>;

    /// Hover
    async fn hover(&self, element_id: &str) -> Result<(), crate::Error>;

    /// Focus
    async fn focus(&self, element_id: &str) -> Result<(), crate::Error>;

    /// Select option
    async fn select_option(&self, element_id: &str, values: Vec<String>) -> Result<(), crate::Error>;

    /// Is visible
    async fn is_visible(&self, element_id: &str) -> Result<bool, crate::Error>;

    /// Is enabled
    async fn is_enabled(&self, element_id: &str) -> Result<bool, crate::Error>;

    /// Get bounding box
    async fn get_bounding_box(&self, element_id: &str) -> Result<BoundingBox, crate::Error>;
}

/// Selector type
#[derive(Debug, Clone, Copy)]
pub enum SelectorType {
    Css,
    XPath,
    Text,
}

/// Element information
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ElementInfo {
    pub element_id: String,
    pub tag_name: String,
    pub text_content: Option<String>,
}

/// Bounding box
#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

// ============================================================================
// Profile Service
// ============================================================================

/// Profile service trait
///
/// Provides browser fingerprinting and stealth capabilities.
#[async_trait]
pub trait ProfileService: Send + Sync {
    /// Create a profile
    async fn create_profile(&self, profile_type: ProfileType) -> Result<Profile, crate::Error>;

    /// Apply profile to page
    async fn apply_profile(&self, page_id: &str, profile_id: &str) -> Result<AppliedFeatures, crate::Error>;

    /// Get presets
    async fn get_presets(&self) -> Result<Vec<ProfilePreset>, crate::Error>;

    /// Get active profile
    async fn get_active_profile(&self, page_id: &str) -> Result<Option<Profile>, crate::Error>;

    /// Create custom profile
    async fn create_custom_profile(
        &self,
        options: CustomProfileOptions,
    ) -> Result<Profile, crate::Error>;

    /// Randomize profile
    async fn randomize_profile(&self, profile_id: &str) -> Result<Profile, crate::Error>;
}

/// Profile type
#[derive(Debug, Clone, Copy)]
pub enum ProfileType {
    Windows,
    Linux,
    MacOS,
    Android,
    IOS,
    Custom,
}

/// Browser profile
#[derive(Debug, Clone)]
pub struct Profile {
    pub profile_id: String,
    pub profile_type: ProfileType,
    pub fingerprint: Fingerprint,
}

/// Fingerprint data
#[derive(Debug, Clone)]
pub struct Fingerprint {
    pub headers: HeadersFingerprint,
    pub navigator: NavigatorFingerprint,
    pub screen: ScreenFingerprint,
    pub webgl: WebGLFingerprint,
    pub options: ProfileOptions,
}

/// Headers fingerprint
#[derive(Debug, Clone)]
pub struct HeadersFingerprint {
    pub user_agent: String,
    pub accept_language: String,
    pub accept_encoding: String,
}

/// Navigator fingerprint
#[derive(Debug, Clone)]
pub struct NavigatorFingerprint {
    pub platform: String,
    pub vendor: String,
    pub hardware_concurrency: u32,
    pub device_memory: Option<u32>,
    pub language: String,
}

/// Screen fingerprint
#[derive(Debug, Clone)]
pub struct ScreenFingerprint {
    pub width: u32,
    pub height: u32,
    pub color_depth: u32,
    pub pixel_depth: u32,
}

/// WebGL fingerprint
#[derive(Debug, Clone)]
pub struct WebGLFingerprint {
    pub vendor: String,
    pub renderer: String,
}

/// Applied features
#[derive(Debug, Clone)]
pub struct AppliedFeatures {
    pub features: Vec<String>,
}

/// Profile preset
#[derive(Debug, Clone)]
pub struct ProfilePreset {
    pub name: String,
    pub profile_type: ProfileType,
    pub description: String,
}

/// Custom profile options
#[derive(Debug, Clone)]
pub struct CustomProfileOptions {
    pub profile_name: String,
    pub template: ProfileType,
    pub options: CustomOptions,
    pub profile_options: ProfileOptions,
}

/// Custom options
#[derive(Debug, Clone)]
pub struct CustomOptions {
    pub user_agent: Option<String>,
    pub platform: Option<String>,
    pub viewport: Option<Viewport>,
}

/// Viewport
#[derive(Debug, Clone)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    pub device_scale_factor: f64,
}

/// Profile options
#[derive(Debug, Clone)]
pub struct ProfileOptions {
    pub inject_navigator: bool,
    pub inject_screen: bool,
    pub inject_webgl: bool,
    pub inject_canvas: bool,
    pub inject_audio: bool,
}

// ============================================================================
// Event Service
// ============================================================================

/// Event service trait
///
/// Provides event streaming capabilities.
#[async_trait]
pub trait EventService: Send + Sync {
    /// Subscribe to events
    async fn subscribe(
        &self,
        page_id: &str,
        event_types: Vec<EventType>,
    ) -> Result<tokio::sync::mpsc::Receiver<Event>, crate::Error>;

    /// Unsubscribe from events
    async fn unsubscribe(&self, page_id: &str) -> Result<(), crate::Error>;
}

/// Event type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    PageLoaded,
    PageNavigated,
    PageClosed,
    ConsoleLog,
    ConsoleError,
    RequestSent,
    ResponseReceived,
    JsException,
    DialogOpened,
}

/// Event
#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub timestamp: i64,
    pub page_event: Option<PageEvent>,
    pub console_event: Option<ConsoleEvent>,
    pub network_event: Option<NetworkEvent>,
}

/// Page event
#[derive(Debug, Clone)]
pub struct PageEvent {
    pub url: String,
    pub title: Option<String>,
}

/// Console event
#[derive(Debug, Clone)]
pub struct ConsoleEvent {
    pub level: ConsoleLevel,
    pub args: Vec<String>,
}

/// Console level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleLevel {
    Log,
    Debug,
    Info,
    Warn,
    Error,
}

/// Network event
#[derive(Debug, Clone)]
pub struct NetworkEvent {
    pub url: String,
    pub method: String,
    pub status_code: u16,
}
