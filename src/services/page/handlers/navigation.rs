//! 导航相关的 RPC 方法处理器
//!
//! 包括：create_page, navigate, reload, go_back, go_forward, close_page

use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{error, info};
use crate::session::{SessionManager, PageOptions, NavigationOptions};
use crate::chaser_oxide::v1::{
    create_page_response::Response as CreatePageResponseEnum,
    navigate_response::Response as NavigateResponseEnum,
    reload_response::Response as ReloadResponseEnum,
    go_back_response::Response as GoBackResponseEnum,
    go_forward_response::Response as GoForwardResponseEnum,
    close_page_response::Response as ClosePageResponseEnum,
    CreatePageRequest, CreatePageResponse,
    NavigateRequest, NavigateResponse,
    ReloadRequest, ReloadResponse,
    GoBackRequest, GoBackResponse,
    GoForwardRequest, GoForwardResponse,
    ClosePageRequest, ClosePageResponse,
    PageInfo as ProtoPageInfo,
    NavigationResult as ProtoNavigationResult,
    Empty,
};
use super::super::{conversions, response};

/// 实现 PageService trait 中的导航相关方法
///
/// 这些方法应该被 impl<S> PageService for Service<S> 块使用
pub struct NavigationHandlers<S> {
    pub session_manager: Arc<S>,
}

impl<S> NavigationHandlers<S>
where
    S: SessionManager + Send + Sync + 'static,
{
    /// 创建新页面
    pub async fn create_page(&self, request: Request<CreatePageRequest>) -> Result<Response<CreatePageResponse>, Status> {
        let req = request.into_inner();

        // 从请求构建 PageOptions
        let mut page_options = PageOptions::default();

        if let Some(viewport) = req.viewport {
            page_options.viewport_width = viewport.width.max(0) as u32;
            page_options.viewport_height = viewport.height.max(0) as u32;
            page_options.device_scale_factor = viewport.device_scale_factor;
        }

        match self.session_manager.create_page(&req.browser_id, page_options).await {
            Ok(page) => {
                info!(
                    page_id = %page.id(),
                    browser_id = %req.browser_id,
                    "Page created successfully"
                );

                let page_info = ProtoPageInfo {
                    page_id: page.id().to_string(),
                    browser_id: page.browser_id().to_string(),
                    url: String::new(),
                    title: String::new(),
                    is_loaded: false,
                    created_at: chrono::Utc::now().timestamp(),
                };

                // 如果提供了 URL，导航到该 URL
                if !req.url.is_empty() {
                    let page_clone = Arc::clone(&page);
                    tokio::spawn(async move {
                        let _ = page_clone.navigate(
                            &req.url,
                            NavigationOptions::default()
                        ).await;
                    });
                }

                Ok(Response::new(CreatePageResponse {
                    response: Some(CreatePageResponseEnum::PageInfo(page_info)),
                }))
            }
            Err(e) => {
                error!(
                    error = %e,
                    browser_id = %req.browser_id,
                    "Failed to create page. Please ensure Chrome is running with --remote-debugging-port=9222"
                );
                Err(response::error_to_status(e))
            }
        }
    }

    /// 导航到指定 URL
    pub async fn navigate(&self, request: Request<NavigateRequest>) -> Result<Response<NavigateResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                let options = conversions::proto_to_navigation_options(req.options.unwrap_or_default());
                match page.navigate(&req.url, options).await {
                    Ok(result) => {
                        let nav_result = conversions::navigation_result_to_proto(
                            result.url,
                            result.status_code,
                            result.is_loaded,
                        );
                        Ok(Response::new(NavigateResponse {
                            response: Some(NavigateResponseEnum::Result(nav_result)),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }

    /// 重新加载页面
    pub async fn reload(&self, request: Request<ReloadRequest>) -> Result<Response<ReloadResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                match page.reload(false).await {
                    Ok(_) => {
                        let result = ProtoNavigationResult {
                            url: String::new(),
                            status_code: 200,
                            is_loaded: true,
                        };
                        Ok(Response::new(ReloadResponse {
                            response: Some(ReloadResponseEnum::Result(result)),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }

    /// 后退
    pub async fn go_back(&self, request: Request<GoBackRequest>) -> Result<Response<GoBackResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                match page.go_back().await {
                    Ok(_) => {
                        let result = ProtoNavigationResult {
                            url: String::new(),
                            status_code: 200,
                            is_loaded: true,
                        };
                        Ok(Response::new(GoBackResponse {
                            response: Some(GoBackResponseEnum::Result(result)),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }

    /// 前进
    pub async fn go_forward(&self, request: Request<GoForwardRequest>) -> Result<Response<GoForwardResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                match page.go_forward().await {
                    Ok(_) => {
                        let result = ProtoNavigationResult {
                            url: String::new(),
                            status_code: 200,
                            is_loaded: true,
                        };
                        Ok(Response::new(GoForwardResponse {
                            response: Some(GoForwardResponseEnum::Result(result)),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }

    /// 关闭页面
    pub async fn close_page(&self, request: Request<ClosePageRequest>) -> Result<Response<ClosePageResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.close_page(&req.page_id).await {
            Ok(_) => {
                Ok(Response::new(ClosePageResponse {
                    response: Some(ClosePageResponseEnum::Success(Empty {})),
                }))
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }
}
