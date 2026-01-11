//! # 会话管理层
//!
//! 管理浏览器上下文、页面和元素的生命周期，提供高级抽象来简化浏览器自动化操作。
//!
//! ## 主要功能
//! - **浏览器管理**: 创建和管理多个浏览器实例
//! - **页面管理**: 在浏览器中创建、切换和关闭页面
//! - **元素操作**: 查找和交互页面元素
//! - **会话清理**: 自动清理过期和空闲的会话资源
//! - **并发安全**: 所有操作都是线程安全的，支持并发访问
//!
//! ## 核心概念
//! - **BrowserContext**: 浏览器上下文，可以包含多个页面
//! - **PageContext**: 页面上下文，提供页面级别的操作
//! - **ElementRef**: 元素引用，用于页面元素交互
//!
//! ## 模块结构
//! - `traits`: 会话管理的核心 trait 定义
//! - `manager`: 会话管理器实现
//! - `browser`: 浏览器上下文实现
//! - `page`: 页面上下文实现
//! - `element`: 元素引用实现
//! - `mock`: 用于测试的 Mock 实现
//!
//! ## 使用示例
//! ```rust,no_run
//! use chaser_oxide::session::{SessionManager, BrowserOptions, PageOptions};
//! use std::sync::Arc;
//!
//! # async fn example(manager: Arc<dyn SessionManager>) -> Result<(), Box<dyn std::error::Error>> {
//! // 创建浏览器
//! let browser = manager.create_browser(BrowserOptions::default()).await?;
//!
//! // 创建页面
//! let page = manager.create_page(browser.id(), PageOptions::default()).await?;
//!
//! // 导航到 URL
//! let result = manager.navigate_page(page.id(), "https://example.com", None).await?;
//! println!("Page loaded: {}", result.url);
//! # Ok(())
//! # }
//! ```

pub mod traits;
pub mod manager;
pub mod browser;
pub mod page;
pub mod element;
pub mod mock;

#[cfg(test)]
pub mod tests;

pub use traits::{
    SessionManager, BrowserContext, PageContext, ElementRef,
    BrowserOptions, PageOptions, ScreenshotOptions, NavigationOptions,
    LoadState, ScreenshotFormat, ClipRegion,
    NavigationResult, EvaluationResult, BoundingBox,
};

// Re-export implementation structs
pub use manager::SessionManagerImpl;
pub use browser::BrowserContextImpl;
pub use page::PageContextImpl;
pub use element::ElementRefImpl;

// Re-export mock implementations for testing
#[cfg(test)]
pub use mock::{MockSessionManager, MockBrowser, MockPage, MockElement};
