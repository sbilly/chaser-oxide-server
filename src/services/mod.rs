//! # 服务层
//!
//! 提供所有 gRPC 服务的实现，将底层功能暴露为网络接口。
//!
//! ## 主要服务
//! - **BrowserService**: 浏览器生命周期管理（创建、关闭、列表）
//! - **PageService**: 页面操作（导航、截图、脚本执行）
//! - **ElementService**: 元素查找和交互
//! - **EventService**: 事件流（控制台、网络事件）
//! - **ProfileService**: 浏览器指纹配置管理
//!
//! ## 架构设计
//! 所有服务都实现了对应的 trait，并通过 gRPC 暴露给客户端。
//! 服务层通过依赖注入使用会话管理器和隐身引擎等核心组件。
//!
//! ## 模块结构
//! - `traits`: 服务层的核心 trait 定义
//! - `common`: gRPC 服务公共工具和宏
//! - `browser`: 浏览器服务实现
//! - `page`: 页面服务实现
//! - `element`: 元素服务实现
//! - `event`: 事件服务实现
//! - `profile`: 配置服务实现
//!
//! ## 使用示例
//! 服务通过 gRPC 客户端调用，以下是各服务的主要方法：
//! - `BrowserService::CreateBrowser`: 创建新浏览器实例
//! - `PageService::Navigate`: 导航到指定 URL
//! - `ElementService::FindElement`: 查找页面元素
//! - `EventService::SubscribeEvents`: 订阅页面事件
//! - `ProfileService::ApplyProfile`: 应用指纹配置


pub mod traits;
pub mod common;
pub mod browser;
pub mod page;
pub mod profile;
pub mod element;
pub mod event;

pub use traits::{
    BrowserService, PageService, ElementService, ProfileService, EventService,
    BrowserInfo, BrowserVersion, BrowserStatus, PageInfo,
    PageSnapshot, ScreenshotData, NavigationResult, EvaluationResult, WaitCondition,
    SelectorType, ElementInfo, BoundingBox,
    ProfileType, Profile, ProfilePreset, CustomProfileOptions,
    ProfileOptions, CustomOptions, Viewport, AppliedFeatures,
    Fingerprint, HeadersFingerprint, NavigatorFingerprint, ScreenFingerprint, WebGLFingerprint,
    EventType, Event, PageEvent, ConsoleEvent, NetworkEvent, ConsoleLevel,
};

// Export gRPC service implementations
pub use browser::BrowserServer as BrowserServiceGrpc;
pub use page::Service as PageServiceGrpc;
pub use profile::ProfileServiceImpl;
// pub use profile::ProfileServiceGrpc;  // Temporarily disabled
pub use element::ElementGrpcService;
pub use event::{EventDispatcher, EventGrpcService};
