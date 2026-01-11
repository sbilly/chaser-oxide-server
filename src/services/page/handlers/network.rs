//! 网络相关的 RPC 方法处理器
//!
//! 包括：set_offline_mode, set_cache_enabled, override_permissions

use std::sync::Arc;
use tonic::{Request, Response, Status};
use crate::session::SessionManager;
use crate::chaser_oxide::v1::{
    set_offline_mode_response::Response as SetOfflineModeResponseEnum,
    set_cache_enabled_response::Response as SetCacheEnabledResponseEnum,
    override_permissions_response::Response as OverridePermissionsResponseEnum,
    SetOfflineModeRequest, SetOfflineModeResponse,
    SetCacheEnabledRequest, SetCacheEnabledResponse,
    OverridePermissionsRequest, OverridePermissionsResponse,
    Empty,
};
use super::super::{response, scripts};

/// 实现 PageService trait 中的网络相关方法
pub struct NetworkHandlers<S> {
    pub session_manager: Arc<S>,
}

impl<S> NetworkHandlers<S>
where
    S: SessionManager + Send + Sync + 'static,
{
    /// 设置离线模式
    pub async fn set_offline_mode(&self, request: Request<SetOfflineModeRequest>) -> Result<Response<SetOfflineModeResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                let offline_script = format!("({})({})", scripts::SET_OFFLINE_MODE_SCRIPT, req.offline);

                match page.evaluate(&offline_script, false).await {
                    Ok(_) => {
                        Ok(Response::new(SetOfflineModeResponse {
                            response: Some(SetOfflineModeResponseEnum::Success(Empty {})),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }

    /// 设置缓存启用状态
    pub async fn set_cache_enabled(&self, request: Request<SetCacheEnabledRequest>) -> Result<Response<SetCacheEnabledResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                let cache_script = format!("({})({})", scripts::SET_CACHE_ENABLED_SCRIPT, !req.enabled);

                match page.evaluate(&cache_script, false).await {
                    Ok(_) => {
                        Ok(Response::new(SetCacheEnabledResponse {
                            response: Some(SetCacheEnabledResponseEnum::Success(Empty {})),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }

    /// 覆盖权限
    pub async fn override_permissions(&self, request: Request<OverridePermissionsRequest>) -> Result<Response<OverridePermissionsResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                let permissions_json = serde_json::to_string(&req.permissions).unwrap_or_default();
                let permissions_script = format!("({})({})", scripts::OVERRIDE_PERMISSIONS_SCRIPT, permissions_json);

                match page.evaluate(&permissions_script, false).await {
                    Ok(_) => {
                        Ok(Response::new(OverridePermissionsResponse {
                            response: Some(OverridePermissionsResponseEnum::Success(Empty {})),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }
}
