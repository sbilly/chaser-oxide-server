//! # 隐身引擎
//!
//! 提供浏览器指纹规避和人类行为模拟功能，用于防止自动化检测。
//!
//! ## 主要功能
//! - **指纹生成**: 生成真实的浏览器指纹，包括 User-Agent、WebGL、Canvas 等
//! - **脚本注入**: 向页面注入隐身脚本，修改浏览器属性
//! - **行为模拟**: 模拟真实用户的鼠标移动、打字、滚动等行为
//! - **配置管理**: 管理浏览器指纹配置文件
//!
//! ## 反检测特性
//! - 修改 navigator 对象属性
//! - 伪装 WebGL 和 Canvas 指纹
//! - 模拟真实的设备特征
//! - 添加人类行为的随机性和延迟
//!
//! ## 模块结构
//! - `traits`: 隐身引擎的核心 trait 定义
//! - `engine`: 隐身引擎主实现
//! - `injector`: 脚本注入器实现
//! - `behavior`: 行为模拟器实现
//! - `fingerprint`: 指纹生成器实现，包含各种预设指纹
//!
//! ## 使用示例
//! ```rust,no_run
//! use chaser_oxide::stealth::{StealthEngine, ScriptInjector, ScriptType};
//! use std::sync::Arc;
//!
//! # async fn example(injector: Arc<dyn ScriptInjector>) -> Result<(), Box<dyn std::error::Error>> {
//! // 注入隐身脚本
//! let result = injector.inject_script("page_id", ScriptType::Stealth).await?;
//! println!("Injected {} features", result.applied_features.len());
//! # Ok(())
//! # }
//! ```

pub mod traits;
pub mod engine;
pub mod injector;
pub mod behavior;
pub mod fingerprint;

#[cfg(test)]
mod tests;

pub use traits::{
    StealthEngine, ScriptInjector, BehaviorSimulator, FingerprintGenerator, ProfileManager,
    AppliedFeatures, InjectedScript, ScriptType,
    MouseMoveOptions, TypingOptions, ClickOptions, ScrollOptions,
};

pub use engine::StealthEngineImpl;
pub use injector::ScriptInjectorImpl;
pub use behavior::BehaviorSimulatorImpl;
pub use fingerprint::{FingerprintGeneratorImpl, WINDOWS_USER_AGENTS, MACOS_USER_AGENTS, LINUX_USER_AGENTS, ANDROID_USER_AGENTS, IOS_USER_AGENTS, WEBGL_VENDORS, WEBGL_RENDERERS};
