//! # 页面服务
//!
//! 提供 gRPC 服务用于页面操作，包括导航、截图、脚本执行等。
//!
//! ## 主要功能
//! - **页面导航**: 导航到指定 URL 并等待加载完成
//! - **页面截图**: 支持多种格式（PNG、JPEG）和裁剪区域
//! - **脚本执行**: 在页面上下文中执行 JavaScript 代码
//! - **页面快照**: 获取页面的可访问性快照
//! - **等待条件**: 等待特定条件满足（如元素出现、URL 变化）
//!
//! ## RPC 方法
//! - `CreatePage`: 在浏览器中创建新页面
//! - `Navigate`: 导航到指定 URL
//! - `Screenshot`: 截取页面截图
//! - `Execute`: 执行 JavaScript 代码
//! - `Snapshot`: 获取页面快照
//! - `Wait`: 等待条件满足
//! - `ClosePage`: 关闭页面
//!
//! ## 使用示例
//! ```rust,no_run
//! use chaser_oxide::services::PageServiceGrpc;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let service = PageServiceGrpc::new(/* session_manager */);
//! // 通过 gRPC 客户端调用
//! // let response = client.navigate(request).await?;
//! # Ok(())
//! # }
//! ```

pub mod service;

#[cfg(test)]
mod tests;

pub use service::Service;
