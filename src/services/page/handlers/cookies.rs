//! Cookie 管理相关的 RPC 方法处理器
//!
//! 包括：get_cookies, set_cookies, clear_cookies

use std::sync::Arc;
use tonic::{Request, Response, Status};
use crate::session::{SessionManager, EvaluationResult as SessionEvaluationResult};
use crate::chaser_oxide::v1::{
    get_cookies_response::Response as GetCookiesResponseEnum,
    set_cookies_response::Response as SetCookiesResponseEnum,
    clear_cookies_response::Response as ClearCookiesResponseEnum,
    GetCookiesRequest, GetCookiesResponse,
    SetCookiesRequest, SetCookiesResponse,
    ClearCookiesRequest, ClearCookiesResponse,
    Cookie,
    Cookies,
    Empty,
};
use super::super::{response, scripts};

/// 实现 PageService trait 中的 Cookie 管理相关方法
pub struct CookieHandlers<S> {
    pub session_manager: Arc<S>,
}

impl<S> CookieHandlers<S>
where
    S: SessionManager + Send + Sync + 'static,
{
    /// 获取 Cookies
    pub async fn get_cookies(&self, request: Request<GetCookiesRequest>) -> Result<Response<GetCookiesResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                match page.evaluate(scripts::GET_COOKIES_SCRIPT, false).await {
                    Ok(SessionEvaluationResult::Object(obj)) => {
                        if let Some(arr) = obj.as_array() {
                            let cookies: Vec<Cookie> = arr.iter().filter_map(|c| {
                                c.as_object().map(|obj| Cookie {
                                    name: obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                    value: obj.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                    domain: obj.get("domain").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                    path: obj.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                    expires: obj.get("expires").and_then(|v| v.as_i64()).unwrap_or(0),
                                    size: obj.get("size").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                    http_only: obj.get("http_only").and_then(|v| v.as_bool()).unwrap_or(false),
                                    secure: obj.get("secure").and_then(|v| v.as_bool()).unwrap_or(false),
                                    session: obj.get("session").and_then(|v| v.as_bool()).unwrap_or(false),
                                    same_site: obj.get("same_site").and_then(|v| v.as_str()).unwrap_or("Lax").to_string(),
                                })
                            }).collect();

                            Ok(Response::new(GetCookiesResponse {
                                response: Some(GetCookiesResponseEnum::Cookies(Cookies { cookies })),
                            }))
                        } else {
                            Ok(Response::new(GetCookiesResponse {
                                response: Some(GetCookiesResponseEnum::Cookies(Cookies { cookies: vec![] })),
                            }))
                        }
                    }
                    Ok(_) => {
                        Ok(Response::new(GetCookiesResponse {
                            response: Some(GetCookiesResponseEnum::Cookies(Cookies { cookies: vec![] })),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }

    /// 设置 Cookies
    pub async fn set_cookies(&self, request: Request<SetCookiesRequest>) -> Result<Response<SetCookiesResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                // 使用 document.cookie 设置每个 cookie
                for cookie in req.cookies {
                    let cookie_string = format!("{}={}; Domain={}; Path={}",
                        cookie.name, cookie.value, cookie.domain, cookie.path);

                    let script = format!("document.cookie = {}", serde_json::json!(cookie_string));

                    match page.evaluate(&script, false).await {
                        Ok(_) => continue,
                        Err(e) => {
                            return Err(response::error_to_status(e));
                        }
                    }
                }

                Ok(Response::new(SetCookiesResponse {
                    response: Some(SetCookiesResponseEnum::Success(Empty {})),
                }))
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }

    /// 清除 Cookies
    pub async fn clear_cookies(&self, request: Request<ClearCookiesRequest>) -> Result<Response<ClearCookiesResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                match page.evaluate(scripts::CLEAR_COOKIES_SCRIPT, false).await {
                    Ok(_) => {
                        Ok(Response::new(ClearCookiesResponse {
                            response: Some(ClearCookiesResponseEnum::Success(Empty {})),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }
}
