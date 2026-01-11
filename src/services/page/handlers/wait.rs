//! 等待条件相关的 RPC 方法处理器
//!
//! 包括：wait_for

use std::sync::Arc;
use tonic::{Request, Response, Status};
use crate::session::{SessionManager, EvaluationResult as SessionEvaluationResult};
use crate::chaser_oxide::v1::{
    wait_for_response::Response as WaitForResponseEnum,
    WaitForRequest, WaitForResponse,
    Empty,
};
use super::super::{response, scripts};

/// 实现 PageService trait 中的等待条件相关方法
pub struct WaitForHandlers<S> {
    pub session_manager: Arc<S>,
}

impl<S> WaitForHandlers<S>
where
    S: SessionManager + Send + Sync + 'static,
{
    /// 等待条件满足
    pub async fn wait_for(&self, request: Request<WaitForRequest>) -> Result<Response<WaitForResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                // 根据请求等待条件
                let wait_result = match req.wait_condition {
                    Some(condition) => {
                        match condition {
                            crate::chaser_oxide::v1::wait_for_request::WaitCondition::Selector(selector) => {
                                // 等待选择器出现
                                let script = format!("{}('{}')", scripts::WAIT_FOR_SELECTOR_SCRIPT, selector);
                                page.evaluate(&script, false).await
                            }
                            crate::chaser_oxide::v1::wait_for_request::WaitCondition::Timeout(timeout_ms) => {
                                // 等待指定的超时时间
                                tokio::time::sleep(tokio::time::Duration::from_millis(timeout_ms as u64)).await;
                                Ok(SessionEvaluationResult::Bool(true))
                            }
                            crate::chaser_oxide::v1::wait_for_request::WaitCondition::NavigationUrl(url) => {
                                // 等待导航到指定 URL
                                let script = format!("{}('{}')", scripts::WAIT_FOR_URL_SCRIPT, url);
                                page.evaluate(&script, false).await
                            }
                        }
                    }
                    None => {
                        // 默认：等待短时间
                        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                        Ok(SessionEvaluationResult::Bool(true))
                    }
                };

                match wait_result {
                    Ok(_) => {
                        Ok(Response::new(WaitForResponse {
                            response: Some(WaitForResponseEnum::Success(Empty {})),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }
}
