//! # 元素服务
//!
//! 提供页面元素查找和交互功能，支持多种选择器和操作方式。
//!
//! ## 主要功能
//! - **元素查找**: 支持 CSS 选择器、XPath、文本等多种查找方式
//! - **元素交互**: 点击、输入、悬停、滚动等操作
//! - **元素信息**: 获取元素的属性、文本、边界框等信息
//! - **批量操作**: 支持批量查找和操作多个元素
//!
//! ## 选择器类型
//! - `CSS`: CSS 选择器（如 `.class`, `#id`, `[attr=value]`）
//! - `XPath`: XPath 表达式
//! - `Text`: 按文本内容查找
//! - `AriaLabel`: 按 ARIA 标签查找
//!
//! ## 模块结构
//! - `finder`: 元素查找器实现
//! - `interactor`: 元素交互器实现
//! - `service`: gRPC 服务实现
//!
//! ## RPC 方法
//! - `FindElement`: 查找单个元素
//! - `FindElements`: 查找多个元素
//! - `ClickElement`: 点击元素
//! - `TypeElement`: 在元素中输入文本
//! - `GetElementInfo`: 获取元素信息
//!
//! ## 使用示例
//! ```rust,no_run
//! use chaser_oxide::services::ElementGrpcService;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let service = ElementGrpcService::new(/* session_manager */);
//! // 通过 gRPC 客户端调用
//! // let request = FindElementRequest {
//! //     selector: "button.submit".to_string(),
//! //     selector_type: SelectorType::Css as i32,
//! //     ..Default::default()
//! // };
//! // let response = client.find_element(request).await?;
//! # Ok(())
//! # }
//! ```

pub mod finder;
pub mod interactor;
pub mod service;

#[cfg(test)]
mod tests;

pub use service::ElementGrpcService;
