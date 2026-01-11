//! 内容操作相关的 RPC 方法处理器
//!
//! 包括：screenshot, get_snapshot, get_content, set_content, get_pdf

use std::sync::Arc;
use tonic::{Request, Response, Status};
use crate::session::{SessionManager, EvaluationResult as SessionEvaluationResult};
use crate::chaser_oxide::v1::{
    screenshot_response::Response as ScreenshotResponseEnum,
    get_snapshot_response::Response as GetSnapshotResponseEnum,
    get_content_response::Response as GetContentResponseEnum,
    set_content_response::Response as SetContentResponseEnum,
    get_pdf_response::Response as GetPdfResponseEnum,
    ScreenshotRequest, ScreenshotResponse,
    GetSnapshotRequest, GetSnapshotResponse,
    GetContentRequest, GetContentResponse,
    SetContentRequest, SetContentResponse,
    GetPdfRequest, GetPdfResponse,
    PageSnapshot as ProtoPageSnapshot,
    NodeInfo,
    PageContent,
};
use super::super::{conversions, response, scripts};

/// 实现 PageService trait 中的内容操作相关方法
pub struct ContentHandlers<S> {
    pub session_manager: Arc<S>,
}

impl<S> ContentHandlers<S>
where
    S: SessionManager + Send + Sync + 'static,
{
    /// 截取页面截图
    pub async fn screenshot(&self, request: Request<ScreenshotRequest>) -> Result<Response<ScreenshotResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                let options = conversions::proto_to_screenshot_options(req.options.unwrap_or_default());
                let format = options.format;

                match page.screenshot(options).await {
                    Ok(data) => {
                        let result = conversions::screenshot_to_proto(data, format);
                        Ok(Response::new(ScreenshotResponse {
                            response: Some(ScreenshotResponseEnum::Result(result)),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }

    /// 获取页面快照
    pub async fn get_snapshot(&self, request: Request<GetSnapshotRequest>) -> Result<Response<GetSnapshotResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                // 获取页面标题
                let title = match page.evaluate(scripts::GET_TITLE_SCRIPT, false).await {
                    Ok(SessionEvaluationResult::String(t)) => t,
                    _ => String::new(),
                };

                // 获取页面 URL
                let url = match page.evaluate(scripts::GET_URL_SCRIPT, false).await {
                    Ok(SessionEvaluationResult::String(u)) => u,
                    _ => String::new(),
                };

                // 获取可访问性树
                let snapshot = match page.evaluate(scripts::SNAPSHOT_SCRIPT, false).await {
                    Ok(SessionEvaluationResult::String(json)) => {
                        serde_json::from_str::<serde_json::Value>(&json).ok()
                    }
                    _ => None,
                };

                let nodes = if snapshot.is_some() {
                    // 转换为 NodeInfo（简化版本）
                    vec![NodeInfo {
                        node_id: "root".to_string(),
                        role: "document".to_string(),
                        name: title.clone(),
                        description: String::new(),
                        tag_name: "body".to_string(),
                        attributes: vec![],
                        children: vec![],
                        is_visible: true,
                        is_interactive: false,
                    }]
                } else {
                    vec![]
                };

                let page_snapshot = ProtoPageSnapshot {
                    title,
                    url,
                    nodes,
                };

                Ok(Response::new(GetSnapshotResponse {
                    response: Some(GetSnapshotResponseEnum::Snapshot(page_snapshot)),
                }))
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }

    /// 获取页面内容
    pub async fn get_content(&self, request: Request<GetContentRequest>) -> Result<Response<GetContentResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                match page.get_content().await {
                    Ok(html) => {
                        let content = PageContent {
                            html: html.clone(),
                            text: html, // 占位符 - 实际实现应提取纯文本
                            scripts: vec![],
                            stylesheets: vec![],
                        };
                        Ok(Response::new(GetContentResponse {
                            response: Some(GetContentResponseEnum::Content(content)),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }

    /// 设置页面内容
    pub async fn set_content(&self, request: Request<SetContentRequest>) -> Result<Response<SetContentResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                match page.set_content(&req.html).await {
                    Ok(_) => {
                        Ok(Response::new(SetContentResponse {
                            response: Some(SetContentResponseEnum::Success(crate::chaser_oxide::v1::Empty {})),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }

    /// 获取 PDF
    pub async fn get_pdf(&self, request: Request<GetPdfRequest>) -> Result<Response<GetPdfResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(_page) => {
                // 注意：完整的 CDP Page.printToPDF 实现需要：
                // 1. CDP 连接支持 Page.printToPDF 命令
                // 2. PDF 选项处理（landscape, display_header_footer 等）
                // 3. Base64 编码的 PDF 数据解码
                //
                // 目前返回未实现错误
                let error = response::error_to_proto(crate::Error::internal(
                    "PDF generation via CDP Page.printToPDF is not yet implemented. \
                     Please add CDP support for Page.printToPDF command to enable this feature."
                ));

                Ok(Response::new(GetPdfResponse {
                    response: Some(GetPdfResponseEnum::Error(error)),
                }))
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }
}
