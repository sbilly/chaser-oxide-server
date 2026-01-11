//! # 浏览器服务
//!
//! 提供 gRPC 服务用于浏览器生命周期管理，包括浏览器的创建、查询和关闭。
//!
//! ## 主要功能
//! - **创建浏览器**: 使用指定选项创建新的浏览器实例
//! - **查询浏览器**: 获取浏览器信息和状态
//! - **关闭浏览器**: 清理浏览器资源并关闭
//! - **浏览器列表**: 列出所有活跃的浏览器实例
//!
//! ## RPC 方法
//! - `CreateBrowser`: 创建新浏览器
//! - `GetBrowser`: 获取浏览器详情
//! - `CloseBrowser`: 关闭指定浏览器
//! - `ListBrowsers`: 列出所有浏览器
//!
//! ## 使用示例
//! ```rust,no_run
//! use chaser_oxide::services::BrowserServiceGrpc;
//! use chaser_oxide::session::BrowserOptions;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let service = BrowserServiceGrpc::new(/* session_manager */);
//! // 通过 gRPC 客户端调用
//! // let response = client.create_browser(request).await?;
//! # Ok(())
//! # }
//! ```

pub mod service;

#[cfg(test)]
mod tests;

pub use service::Service as BrowserServer;
