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
//! ## 架构
//!
//! 服务被组织成多个功能模块：
//! - [`service`][]: 主服务实现，委托给专门的处理器
//! - [`handlers`][]: RPC 方法处理器，按功能分组
//! - [`conversions`][]: 类型转换（proto <-> 内部）
//! - [`response`][]: 响应构建辅助函数
//! - [`scripts`]: JavaScript 脚本常量
//!
//! ## 使用示例
//! ```rust,no_run
//! use chaser_oxide::services::page::Service;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let session_manager = Arc::new(/* ... */);
//! let service = Service::new(session_manager);
//! // 服务现在可以通过 gRPC 使用
//! # Ok(())
//! # }
//! ```

// 公共模块
pub mod service;
pub mod conversions;
pub mod response;
pub mod scripts;
pub mod handlers;

// 测试模块
#[cfg(test)]
mod tests;

// 重新导出主要类型
pub use service::Service;
