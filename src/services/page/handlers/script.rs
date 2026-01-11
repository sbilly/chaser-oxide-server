//! 脚本执行相关的 RPC 方法处理器
//!
//! 包括：evaluate, evaluate_on_element, add_init_script, get_metrics

use std::sync::Arc;
use tonic::{Request, Response, Status};
use crate::session::{SessionManager, EvaluationResult as SessionEvaluationResult};
use crate::chaser_oxide::v1::{
    evaluate_response::Response as EvaluateResponseEnum,
    evaluate_on_element_response::Response as EvaluateOnElementResponseEnum,
    add_init_script_response::Response as AddInitScriptResponseEnum,
    get_metrics_response::Response as GetMetricsResponseEnum,
    EvaluateRequest, EvaluateResponse,
    EvaluateOnElementRequest, EvaluateOnElementResponse,
    AddInitScriptRequest, AddInitScriptResponse,
    GetMetricsRequest, GetMetricsResponse,
    Metrics,
    Empty,
};
use super::super::{conversions, response};

/// 实现 PageService trait 中的脚本执行相关方法
pub struct ScriptHandlers<S> {
    pub session_manager: Arc<S>,
}

impl<S> ScriptHandlers<S>
where
    S: SessionManager + Send + Sync + 'static,
{
    /// 在页面中执行 JavaScript 代码
    pub async fn evaluate(&self, request: Request<EvaluateRequest>) -> Result<Response<EvaluateResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                match page.evaluate(&req.expression, req.await_promise).await {
                    Ok(result) => {
                        let converted = conversions::evaluation_result_from_session(result);
                        Ok(Response::new(EvaluateResponse {
                            response: Some(EvaluateResponseEnum::Result(
                                conversions::evaluation_result_to_proto(converted)
                            )),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }

    /// 在元素上执行 JavaScript 代码
    pub async fn evaluate_on_element(&self, request: Request<EvaluateOnElementRequest>) -> Result<Response<EvaluateOnElementResponse>, Status> {
        let req = request.into_inner();

        let element_ref = req.element.ok_or_else(|| Status::invalid_argument("element is required"))?;

        match self.session_manager.get_page(&element_ref.page_id).await {
            Ok(page) => {
                // 构建在元素上执行的脚本
                let script = format!(
                    r#"
                    (() => {{
                        const element = document.querySelector(`[data-element-id="{}"]`);
                        if (!element) {{ return null; }}

                        return {};
                    }})()
                    "#,
                    element_ref.element_id,
                    req.expression
                );

                match page.evaluate(&script, req.await_promise).await {
                    Ok(result) => {
                        let converted = conversions::evaluation_result_from_session(result);
                        Ok(Response::new(EvaluateOnElementResponse {
                            response: Some(EvaluateOnElementResponseEnum::Result(
                                conversions::evaluation_result_to_proto(converted)
                            )),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }

    /// 添加初始化脚本
    pub async fn add_init_script(&self, request: Request<AddInitScriptRequest>) -> Result<Response<AddInitScriptResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                // 添加脚本以在新文档上评估
                // 目前只是立即执行它
                match page.evaluate(&req.script, false).await {
                    Ok(_) => {
                        Ok(Response::new(AddInitScriptResponse {
                            response: Some(AddInitScriptResponseEnum::Success(Empty {})),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }

    /// 获取性能指标
    pub async fn get_metrics(&self, request: Request<GetMetricsRequest>) -> Result<Response<GetMetricsResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                match page.evaluate(super::super::scripts::GET_METRICS_SCRIPT, false).await {
                    Ok(SessionEvaluationResult::Object(obj)) => {
                        let metrics = Metrics {
                            timestamp: obj.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            layout_duration: obj.get("layout_duration").and_then(|v| v.as_i64()).unwrap_or(0),
                            recalculate_style_duration: obj.get("recalculate_style_duration").and_then(|v| v.as_i64()).unwrap_or(0),
                            documents: obj.get("documents").and_then(|v| v.as_i64()).unwrap_or(1) as i32,
                            frames: obj.get("frames").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                            js_event_listeners: obj.get("js_event_listeners").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                            layouts: vec![],
                            style_recalcs: vec![],
                        };

                        Ok(Response::new(GetMetricsResponse {
                            response: Some(GetMetricsResponseEnum::Metrics(metrics)),
                        }))
                    }
                    Ok(_) => {
                        // 如果评估返回非对象，返回默认指标
                        let metrics = Metrics {
                            timestamp: chrono::Utc::now().to_rfc3339(),
                            layout_duration: 0,
                            recalculate_style_duration: 0,
                            documents: 1,
                            frames: 0,
                            js_event_listeners: 0,
                            layouts: vec![],
                            style_recalcs: vec![],
                        };

                        Ok(Response::new(GetMetricsResponse {
                            response: Some(GetMetricsResponseEnum::Metrics(metrics)),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }
}
