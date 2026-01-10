//! # Chrome DevTools Protocol (CDP) 层
//!
//! 提供 Chrome/Chromium 浏览器的 WebSocket 通信接口，基于 Chrome DevTools Protocol 实现浏览器自动化。
//!
//! ## 主要功能
//! - **WebSocket 连接管理**: 建立和维护与浏览器的 CDP WebSocket 连接
//! - **协议通信**: 发送 CDP 命令并接收响应
//! - **事件订阅**: 监听浏览器事件（如页面加载、控制台日志等）
//! - **导航控制**: 页面导航、加载状态监控
//! - **脚本执行**: 在页面上下文中执行 JavaScript
//! - **截图功能**: 支持多种格式的页面截图
//!
//! ## 模块结构
//! - `traits`: CDP 操作的核心 trait 定义
//! - `types`: CDP 协议相关的数据类型
//! - `connection`: WebSocket 连接实现
//! - `client`: CDP 客户端实现
//! - `browser`: 浏览器级别的操作
//! - `mock`: 用于测试的 Mock 实现
//!
//! ## 使用示例
//! ```rust,no_run
//! use chaser_oxide::cdp::{CdpBrowserImpl, CdpClient};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // 创建浏览器实例
//! let browser = CdpBrowserImpl::new("ws://localhost:9222".to_string());
//! let client = browser.create_client().await?;
//!
//! // 导航到页面
//! let result = client.navigate("https://example.com").await?;
//! println!("Navigated to: {}", result.url);
//! # Ok(())
//! # }
//! ```

pub mod traits;
pub mod types;
pub mod connection;
pub mod client;
pub mod browser;
pub mod mock;

#[cfg(test)]
pub mod tests;

pub use traits::{
    CdpConnection, CdpClient, CdpBrowser, CdpEvent, CdpResponse, CdpError,
    NavigationResult, EvaluationResult, ScreenshotFormat,
    BrowserVersion, TargetInfo,
};

// Re-export implementation structs
pub use connection::CdpWebSocketConnection;
pub use client::CdpClientImpl;
pub use browser::CdpBrowserImpl;

// Re-export mock for development/testing
pub use mock::{MockCdpClient, MockCdpBrowser};
