//! Page service implementation
//!
//! 此模块提供页面操作的 gRPC 服务实现。
//!
//! ## 架构
//!
//! 服务被组织成多个功能模块：
//! - [`conversions`][]: 类型转换（proto <-> 内部）
//! - [`response`][]: 响应构建辅助函数
//! - [`scripts`]: JavaScript 脚本常量
//! - [`handlers`]: RPC 方法实现，按功能分组
//!
//! ## 使用示例
//!
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

use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::session::SessionManager;
use crate::chaser_oxide::v1::page_service_server::PageService;

// 导入公共模块（来自父模块）
use super::handlers;

// 导入处理器
use handlers::{
    NavigationHandlers, ContentHandlers, ScriptHandlers,
    EmulationHandlers, NetworkHandlers, CookieHandlers, WaitForHandlers,
};

/// Page service implementation
///
/// 此结构体实现了 `PageService` trait，提供所有页面操作的 gRPC 方法。
///
/// # 类型参数
///
/// * `S` - SessionManager 类型，必须实现 `SessionManager` trait
#[derive(Debug, Clone)]
pub struct Service<S> {
    /// Session manager 实例
    session_manager: Arc<S>,
}

impl<S> Service<S>
where
    S: SessionManager + Send + Sync + 'static,
{
    /// 创建新的页面服务实例
    ///
    /// # 参数
    ///
    /// * `session_manager` - 会话管理器实例
    pub fn new(session_manager: Arc<S>) -> Self {
        Self { session_manager }
    }

    /// 获取导航处理器
    fn navigation(&self) -> NavigationHandlers<S> {
        NavigationHandlers {
            session_manager: Arc::clone(&self.session_manager),
        }
    }

    /// 获取内容处理器
    fn content(&self) -> ContentHandlers<S> {
        ContentHandlers {
            session_manager: Arc::clone(&self.session_manager),
        }
    }

    /// 获取脚本处理器
    fn script(&self) -> ScriptHandlers<S> {
        ScriptHandlers {
            session_manager: Arc::clone(&self.session_manager),
        }
    }

    /// 获取设备模拟处理器
    fn emulation(&self) -> EmulationHandlers<S> {
        EmulationHandlers {
            session_manager: Arc::clone(&self.session_manager),
        }
    }

    /// 获取网络处理器
    fn network(&self) -> NetworkHandlers<S> {
        NetworkHandlers {
            session_manager: Arc::clone(&self.session_manager),
        }
    }

    /// 获取 Cookie 处理器
    fn cookies(&self) -> CookieHandlers<S> {
        CookieHandlers {
            session_manager: Arc::clone(&self.session_manager),
        }
    }

    /// 获取等待处理器
    fn wait(&self) -> WaitForHandlers<S> {
        WaitForHandlers {
            session_manager: Arc::clone(&self.session_manager),
        }
    }
}

/// 实现 PageService trait
///
/// 此宏实现将所有的 RPC 方法委托给相应的处理器
#[tonic::async_trait]
impl<S> PageService for Service<S>
where
    S: SessionManager + Send + Sync + 'static,
{
    async fn create_page(&self, request: Request<crate::chaser_oxide::v1::CreatePageRequest>) -> Result<Response<crate::chaser_oxide::v1::CreatePageResponse>, Status> {
        self.navigation().create_page(request).await
    }

    async fn navigate(&self, request: Request<crate::chaser_oxide::v1::NavigateRequest>) -> Result<Response<crate::chaser_oxide::v1::NavigateResponse>, Status> {
        self.navigation().navigate(request).await
    }

    async fn get_snapshot(&self, request: Request<crate::chaser_oxide::v1::GetSnapshotRequest>) -> Result<Response<crate::chaser_oxide::v1::GetSnapshotResponse>, Status> {
        self.content().get_snapshot(request).await
    }

    async fn screenshot(&self, request: Request<crate::chaser_oxide::v1::ScreenshotRequest>) -> Result<Response<crate::chaser_oxide::v1::ScreenshotResponse>, Status> {
        self.content().screenshot(request).await
    }

    async fn evaluate(&self, request: Request<crate::chaser_oxide::v1::EvaluateRequest>) -> Result<Response<crate::chaser_oxide::v1::EvaluateResponse>, Status> {
        self.script().evaluate(request).await
    }

    async fn evaluate_on_element(&self, request: Request<crate::chaser_oxide::v1::EvaluateOnElementRequest>) -> Result<Response<crate::chaser_oxide::v1::EvaluateOnElementResponse>, Status> {
        self.script().evaluate_on_element(request).await
    }

    async fn set_content(&self, request: Request<crate::chaser_oxide::v1::SetContentRequest>) -> Result<Response<crate::chaser_oxide::v1::SetContentResponse>, Status> {
        self.content().set_content(request).await
    }

    async fn get_content(&self, request: Request<crate::chaser_oxide::v1::GetContentRequest>) -> Result<Response<crate::chaser_oxide::v1::GetContentResponse>, Status> {
        self.content().get_content(request).await
    }

    async fn reload(&self, request: Request<crate::chaser_oxide::v1::ReloadRequest>) -> Result<Response<crate::chaser_oxide::v1::ReloadResponse>, Status> {
        self.navigation().reload(request).await
    }

    async fn go_back(&self, request: Request<crate::chaser_oxide::v1::GoBackRequest>) -> Result<Response<crate::chaser_oxide::v1::GoBackResponse>, Status> {
        self.navigation().go_back(request).await
    }

    async fn go_forward(&self, request: Request<crate::chaser_oxide::v1::GoForwardRequest>) -> Result<Response<crate::chaser_oxide::v1::GoForwardResponse>, Status> {
        self.navigation().go_forward(request).await
    }

    async fn set_viewport(&self, request: Request<crate::chaser_oxide::v1::SetViewportRequest>) -> Result<Response<crate::chaser_oxide::v1::SetViewportResponse>, Status> {
        self.emulation().set_viewport(request).await
    }

    async fn emulate_device(&self, request: Request<crate::chaser_oxide::v1::EmulateDeviceRequest>) -> Result<Response<crate::chaser_oxide::v1::EmulateDeviceResponse>, Status> {
        self.emulation().emulate_device(request).await
    }

    async fn bring_to_front(&self, request: Request<crate::chaser_oxide::v1::BringToFrontRequest>) -> Result<Response<crate::chaser_oxide::v1::BringToFrontResponse>, Status> {
        self.emulation().bring_to_front(request).await
    }

    async fn get_metrics(&self, request: Request<crate::chaser_oxide::v1::GetMetricsRequest>) -> Result<Response<crate::chaser_oxide::v1::GetMetricsResponse>, Status> {
        self.script().get_metrics(request).await
    }

    async fn close_page(&self, request: Request<crate::chaser_oxide::v1::ClosePageRequest>) -> Result<Response<crate::chaser_oxide::v1::ClosePageResponse>, Status> {
        self.navigation().close_page(request).await
    }

    async fn wait_for(&self, request: Request<crate::chaser_oxide::v1::WaitForRequest>) -> Result<Response<crate::chaser_oxide::v1::WaitForResponse>, Status> {
        self.wait().wait_for(request).await
    }

    async fn get_pdf(&self, request: Request<crate::chaser_oxide::v1::GetPdfRequest>) -> Result<Response<crate::chaser_oxide::v1::GetPdfResponse>, Status> {
        self.content().get_pdf(request).await
    }

    async fn add_init_script(&self, request: Request<crate::chaser_oxide::v1::AddInitScriptRequest>) -> Result<Response<crate::chaser_oxide::v1::AddInitScriptResponse>, Status> {
        self.script().add_init_script(request).await
    }

    async fn get_cookies(&self, request: Request<crate::chaser_oxide::v1::GetCookiesRequest>) -> Result<Response<crate::chaser_oxide::v1::GetCookiesResponse>, Status> {
        self.cookies().get_cookies(request).await
    }

    async fn set_cookies(&self, request: Request<crate::chaser_oxide::v1::SetCookiesRequest>) -> Result<Response<crate::chaser_oxide::v1::SetCookiesResponse>, Status> {
        self.cookies().set_cookies(request).await
    }

    async fn clear_cookies(&self, request: Request<crate::chaser_oxide::v1::ClearCookiesRequest>) -> Result<Response<crate::chaser_oxide::v1::ClearCookiesResponse>, Status> {
        self.cookies().clear_cookies(request).await
    }

    async fn override_permissions(&self, request: Request<crate::chaser_oxide::v1::OverridePermissionsRequest>) -> Result<Response<crate::chaser_oxide::v1::OverridePermissionsResponse>, Status> {
        self.network().override_permissions(request).await
    }

    async fn set_geolocation(&self, request: Request<crate::chaser_oxide::v1::SetGeolocationRequest>) -> Result<Response<crate::chaser_oxide::v1::SetGeolocationResponse>, Status> {
        self.emulation().set_geolocation(request).await
    }

    async fn set_offline_mode(&self, request: Request<crate::chaser_oxide::v1::SetOfflineModeRequest>) -> Result<Response<crate::chaser_oxide::v1::SetOfflineModeResponse>, Status> {
        self.network().set_offline_mode(request).await
    }

    async fn set_cache_enabled(&self, request: Request<crate::chaser_oxide::v1::SetCacheEnabledRequest>) -> Result<Response<crate::chaser_oxide::v1::SetCacheEnabledResponse>, Status> {
        self.network().set_cache_enabled(request).await
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::mock::MockSessionManager;

    // 测试：服务创建
    #[tokio::test]
    async fn test_page_service_creation() {
        let session_manager = Arc::new(MockSessionManager::new());
        let service = Service::new(session_manager);
        assert!(true, "PageService created successfully");
    }

    // 测试：处理器访问
    #[tokio::test]
    async fn test_handler_access() {
        let session_manager = Arc::new(MockSessionManager::new());
        let service = Service::new(session_manager);

        // 验证所有处理器都可以访问
        let _nav = service.navigation();
        let _content = service.content();
        let _script = service.script();
        let _emulation = service.emulation();
        let _network = service.network();
        let _cookies = service.cookies();
        let _wait = service.wait();

        assert!(true);
    }
}
