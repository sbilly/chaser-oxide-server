//! # 事件服务
//!
//! 提供页面事件流功能，允许客户端订阅和接收浏览器事件。
//!
//! ## 主要功能
//! - **事件订阅**: 订阅特定页面的所有事件
//! - **事件分发**: 将浏览器事件实时推送给订阅者
//! - **事件过滤**: 按类型过滤事件（控制台、网络等）
//! - **双向流**: 支持双向 gRPC 流，实时传递事件
//!
//! ## 事件类型
//! - **Console**: 控制台日志、警告、错误
//! - **Network**: 网络请求和响应
//! - **Page**: 页面生命周期事件
//! - **DOM**: DOM 变化事件
//!
//! ## 模块结构
//! - `dispatcher`: 事件分发器，管理事件订阅和广播
//! - `service`: gRPC 服务实现
//!
//! ## RPC 方法
//! - `SubscribeEvents`: 订阅页面事件流（双向流）
//!
//! ## 使用示例
//! ```rust,no_run
//! use chaser_oxide::services::{EventDispatcher, EventGrpcService};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let dispatcher = EventDispatcher::new(1000);
//! # let service = EventGrpcService::new(dispatcher);
//! // 通过 gRPC 客户端建立双向流
//! // let stream = client.subscribe_events(request).await?.into_inner();
//! // while let Some(event) = stream.next().await {
//! //     println!("Received event: {:?}", event);
//! // }
//! # Ok(())
//! # }
//! ```

pub mod dispatcher;
pub mod service;

#[cfg(test)]
mod tests;

pub use dispatcher::EventDispatcher;
pub use service::EventGrpcService;
