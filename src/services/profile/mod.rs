//! # 配置服务
//!
//! 提供浏览器指纹配置管理和隐身能力，用于防止自动化检测。
//!
//! ## 主要功能
//! - **配置管理**: 创建、查询、删除浏览器指纹配置
//! - **预设配置**: 提供常见设备的预设配置（Windows、macOS、Android 等）
//! - **自定义配置**: 支持自定义指纹参数
//! - **应用配置**: 将配置应用到浏览器实例
//!
//! ## 配置类型
//! - **Preset**: 预设配置（如 Chrome on Windows）
//! - **Custom**: 自定义配置
//! - **Random**: 随机生成配置
//!
//! ## 指纹组件
//! - **Headers**: HTTP 头部（User-Agent、Accept 等）
//! - **Navigator**: Navigator 对象属性
//! - **Screen**: 屏幕分辨率和颜色深度
//! - **WebGL**: WebGL 渲染器信息
//! - **Canvas**: Canvas 指纹
//!
//! ## 模块结构
//! - `service`: 配置服务实现
//! - `profile`: 配置管理器实现
//! - `grpc`: gRPC 服务包装
//!
//! ## RPC 方法
//! - `CreateProfile`: 创建新配置
//! - `GetProfile`: 获取配置详情
//! - `ListProfiles`: 列出所有配置
//! - `DeleteProfile`: 删除配置
//! - `ApplyProfile`: 应用配置到浏览器
//!
//! ## 使用示例
//! ```rust,no_run
//! use chaser_oxide::services::ProfileServiceGrpc;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let service = ProfileServiceGrpc::new(/* profile_service_impl */);
//! // 通过 gRPC 客户端调用
//! // let request = CreateProfileRequest {
//! //     profile_type: ProfileType::ChromeWindows as i32,
//! //     ..Default::default()
//! // };
//! // let response = client.create_profile(request).await?;
//! # Ok(())
//! # }
//! ```

pub mod service;
pub mod manager;
pub mod grpc;

pub use service::ProfileServiceImpl;
pub use manager::ProfileManagerImpl;
pub use grpc::ProfileServiceGrpc;
