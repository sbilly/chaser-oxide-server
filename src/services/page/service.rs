//! Page service implementation
//!
//! This module provides the gRPC implementation for page operations.

use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::session::{SessionManager, PageOptions, NavigationOptions, ScreenshotOptions, LoadState, ScreenshotFormat, ClipRegion, EvaluationResult as SessionEvaluationResult};
use crate::services::traits::PageInfo;
use crate::Error;

// Import generated proto types
use crate::chaser_oxide::v1::{
    page_service_server::PageService,
    create_page_response::Response as CreatePageResponseEnum,
    navigate_response::Response as NavigateResponseEnum,
    get_snapshot_response::Response as GetSnapshotResponseEnum,
    screenshot_response::Response as ScreenshotResponseEnum,
    evaluate_response::Response as EvaluateResponseEnum,
    evaluate_on_element_response::Response as EvaluateOnElementResponseEnum,
    set_content_response::Response as SetContentResponseEnum,
    get_content_response::Response as GetContentResponseEnum,
    reload_response::Response as ReloadResponseEnum,
    go_back_response::Response as GoBackResponseEnum,
    go_forward_response::Response as GoForwardResponseEnum,
    set_viewport_response::Response as SetViewportResponseEnum,
    emulate_device_response::Response as EmulateDeviceResponseEnum,
    bring_to_front_response::Response as BringToFrontResponseEnum,
    get_metrics_response::Response as GetMetricsResponseEnum,
    close_page_response::Response as ClosePageResponseEnum,
    wait_for_response::Response as WaitForResponseEnum,
    get_pdf_response::Response as GetPdfResponseEnum,
    add_init_script_response::Response as AddInitScriptResponseEnum,
    override_permissions_response::Response as OverridePermissionsResponseEnum,
    set_geolocation_response::Response as SetGeolocationResponseEnum,
    set_offline_mode_response::Response as SetOfflineModeResponseEnum,
    set_cache_enabled_response::Response as SetCacheEnabledResponseEnum,
    get_cookies_response::Response as GetCookiesResponseEnum,
    set_cookies_response::Response as SetCookiesResponseEnum,
    clear_cookies_response::Response as ClearCookiesResponseEnum,
    CreatePageRequest, CreatePageResponse,
    NavigateRequest, NavigateResponse,
    GetSnapshotRequest, GetSnapshotResponse,
    ScreenshotRequest, ScreenshotResponse,
    EvaluateRequest, EvaluateResponse,
    EvaluateOnElementRequest, EvaluateOnElementResponse,
    SetContentRequest, SetContentResponse,
    GetContentRequest, GetContentResponse,
    ReloadRequest, ReloadResponse,
    GoBackRequest, GoBackResponse,
    GoForwardRequest, GoForwardResponse,
    SetViewportRequest, SetViewportResponse,
    EmulateDeviceRequest, EmulateDeviceResponse,
    BringToFrontRequest, BringToFrontResponse,
    GetMetricsRequest, GetMetricsResponse,
    ClosePageRequest, ClosePageResponse,
    WaitForRequest, WaitForResponse,
    GetPdfRequest, GetPdfResponse,
    AddInitScriptRequest, AddInitScriptResponse,
    OverridePermissionsRequest, OverridePermissionsResponse,
    SetGeolocationRequest, SetGeolocationResponse,
    SetOfflineModeRequest, SetOfflineModeResponse,
    SetCacheEnabledRequest, SetCacheEnabledResponse,
    GetCookiesRequest, GetCookiesResponse,
    SetCookiesRequest, SetCookiesResponse,
    ClearCookiesRequest, ClearCookiesResponse,
    PageInfo as ProtoPageInfo,
    NavigationResult as ProtoNavigationResult,
    NavigationOptions as ProtoNavigationOptions,
    navigation_options,
    ScreenshotOptions as ProtoScreenshotOptions,
    screenshot_options,
    EvaluationResult as ProtoEvaluationResult,
    evaluation_result,
    PageSnapshot as ProtoPageSnapshot,
    NodeInfo,
    ScreenshotResult,
    PageContent,
    Metrics,
    Cookie,
    Cookies,
    Empty,
    Error as ProtoError,
    ErrorCode,
    Value,
};

/// Page service implementation
#[derive(Debug, Clone)]
pub struct Service<S> {
    session_manager: Arc<S>,
}

impl<S> Service<S>
where
    S: SessionManager + Send + Sync + 'static,
{
    /// Create a new page service
    pub fn new(session_manager: Arc<S>) -> Self {
        Self { session_manager }
    }

    /// Convert Error to ProtoError
    fn error_to_proto(error: Error) -> ProtoError {
        let code = match &error {
            Error::PageNotFound(_) => ErrorCode::PageClosed,
            Error::Timeout(_) => ErrorCode::Timeout,
            Error::NavigationFailed(_) => ErrorCode::NavigationFailed,
            Error::ScriptExecutionFailed(_) => ErrorCode::EvaluationFailed,
            Error::Configuration(_) => ErrorCode::InvalidArgument,
            _ => ErrorCode::Internal,
        };

        ProtoError {
            code: code.into(),
            message: error.to_string(),
            details: std::collections::HashMap::new(),
        }
    }

    /// Convert proto NavigationOptions to internal
    pub fn proto_to_navigation_options(opts: ProtoNavigationOptions) -> NavigationOptions {
        let wait_until = match opts.wait_until() {
            navigation_options::LoadState::Unspecified => LoadState::Load,
            navigation_options::LoadState::Load => LoadState::Load,
            navigation_options::LoadState::DomContentLoaded => LoadState::DOMContentLoaded,
            navigation_options::LoadState::NetworkIdle => LoadState::NetworkIdle,
            navigation_options::LoadState::NetworkAlmostIdle => LoadState::NetworkAlmostIdle,
        };

        NavigationOptions {
            timeout: opts.timeout.max(0) as u64,
            wait_until,
        }
    }

    /// Convert proto ScreenshotOptions to internal
    fn proto_to_screenshot_options(opts: ProtoScreenshotOptions) -> ScreenshotOptions {
        let format = match opts.format() {
            screenshot_options::Format::Unspecified => ScreenshotFormat::Png,
            screenshot_options::Format::Png => ScreenshotFormat::Png,
            screenshot_options::Format::Jpeg => ScreenshotFormat::Jpeg,
            screenshot_options::Format::Webp => ScreenshotFormat::WebP,
        };

        // Convert clip region from proto Rectangle to internal ClipRegion
        let clip = opts.clip.map(|rect| ClipRegion {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            scale: 1.0,
        });

        ScreenshotOptions {
            format,
            quality: if opts.quality > 0 { Some(opts.quality as u8) } else { None },
            full_page: opts.full_page,
            clip,
        }
    }

    /// Convert session EvaluationResult to services EvaluationResult
    fn convert_evaluation_result(result: SessionEvaluationResult) -> crate::services::traits::EvaluationResult {
        match result {
            SessionEvaluationResult::String(s) => crate::services::traits::EvaluationResult::String(s),
            SessionEvaluationResult::Number(n) => crate::services::traits::EvaluationResult::Number(n),
            SessionEvaluationResult::Bool(b) => crate::services::traits::EvaluationResult::Bool(b),
            SessionEvaluationResult::Null => crate::services::traits::EvaluationResult::Null,
            SessionEvaluationResult::Object(v) => crate::services::traits::EvaluationResult::Object(v),
        }
    }

    /// Convert internal EvaluationResult to proto
    pub fn evaluation_result_to_proto(result: crate::services::traits::EvaluationResult) -> ProtoEvaluationResult {
        tracing::debug!("evaluation_result_to_proto: received {:?}", result);
        let proto_result = match result {
            crate::services::traits::EvaluationResult::String(s) => {
                tracing::debug!("evaluation_result_to_proto: String variant with value='{}'", s);
                ProtoEvaluationResult {
                    response: Some(evaluation_result::Response::StringValue(s)),
                    r#type: "string".to_string(),
                    class_name: String::new(),
                }
            },
            crate::services::traits::EvaluationResult::Number(n) => {
                tracing::debug!("evaluation_result_to_proto: Number variant with value={}", n);
                ProtoEvaluationResult {
                    response: Some(evaluation_result::Response::DoubleValue(n)),
                    r#type: "number".to_string(),
                    class_name: String::new(),
                }
            },
            crate::services::traits::EvaluationResult::Bool(b) => {
                tracing::debug!("evaluation_result_to_proto: Bool variant with value={}", b);
                ProtoEvaluationResult {
                    response: Some(evaluation_result::Response::BoolValue(b)),
                    r#type: "boolean".to_string(),
                    class_name: String::new(),
                }
            },
            crate::services::traits::EvaluationResult::Null => {
                tracing::debug!("evaluation_result_to_proto: Null variant");
                ProtoEvaluationResult {
                    response: Some(evaluation_result::Response::NullValue(Value {})),
                    r#type: "null".to_string(),
                    class_name: String::new(),
                }
            },
            crate::services::traits::EvaluationResult::Object(v) => {
                tracing::debug!("evaluation_result_to_proto: Object variant with value={:?}", v);
                // Convert JSON object to string representation
                ProtoEvaluationResult {
                    response: Some(evaluation_result::Response::StringValue(v.to_string())),
                    r#type: "object".to_string(),
                    class_name: String::new(),
                }
            }
        };
        tracing::debug!("evaluation_result_to_proto: returning type='{}', response={:?}",
            proto_result.r#type, proto_result.response);
        proto_result
    }

    /// Convert internal PageInfo to proto
    #[allow(dead_code)]
    fn page_info_to_proto(info: PageInfo) -> ProtoPageInfo {
        ProtoPageInfo {
            page_id: info.page_id,
            browser_id: String::new(),
            url: info.url,
            title: info.title,
            is_loaded: true,
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}

#[tonic::async_trait]
impl<S> PageService for Service<S>
where
    S: SessionManager + Send + Sync + 'static,
{
    async fn create_page(&self, request: Request<CreatePageRequest>) -> Result<Response<CreatePageResponse>, Status> {
        let req = request.into_inner();

        // Build PageOptions from request
        let mut page_options = PageOptions::default();

        if let Some(viewport) = req.viewport {
            page_options.viewport_width = viewport.width.max(0) as u32;
            page_options.viewport_height = viewport.height.max(0) as u32;
            page_options.device_scale_factor = viewport.device_scale_factor;
        }

        match self.session_manager.create_page(&req.browser_id, page_options).await {
            Ok(page) => {
                let page_info = ProtoPageInfo {
                    page_id: page.id().to_string(),
                    browser_id: page.browser_id().to_string(),
                    url: String::new(),
                    title: String::new(),
                    is_loaded: false,
                    created_at: chrono::Utc::now().timestamp(),
                };

                // If URL provided, navigate to it
                if !req.url.is_empty() {
                    let page_clone = Arc::clone(&page);
                    tokio::spawn(async move {
                        let _ = page_clone.navigate(
                            &req.url,
                            crate::session::NavigationOptions::default()
                        ).await;
                    });
                }

                Ok(Response::new(CreatePageResponse {
                    response: Some(CreatePageResponseEnum::PageInfo(page_info)),
                }))
            }
            Err(e) => {
                Ok(Response::new(CreatePageResponse {
                    response: Some(CreatePageResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn navigate(&self, request: Request<NavigateRequest>) -> Result<Response<NavigateResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                let options = Self::proto_to_navigation_options(req.options.unwrap_or_default());
                match page.navigate(&req.url, options).await {
                    Ok(result) => {
                        let nav_result = ProtoNavigationResult {
                            url: result.url,
                            status_code: result.status_code as i32,
                            is_loaded: result.is_loaded,
                        };
                        Ok(Response::new(NavigateResponse {
                            response: Some(NavigateResponseEnum::Result(nav_result)),
                        }))
                    }
                    Err(e) => {
                        Ok(Response::new(NavigateResponse {
                            response: Some(NavigateResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(NavigateResponse {
                    response: Some(NavigateResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn get_snapshot(&self, request: Request<GetSnapshotRequest>) -> Result<Response<GetSnapshotResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                // Get page content and build snapshot
                let title = match page.evaluate("document.title", false).await {
                    Ok(SessionEvaluationResult::String(t)) => t,
                    _ => String::new(),
                };

                let url = match page.evaluate("window.location.href", false).await {
                    Ok(SessionEvaluationResult::String(u)) => u,
                    _ => String::new(),
                };

                // Get basic accessible tree using JavaScript
                let snapshot_script = r#"
                    (() => {
                        function buildTree(node, depth = 0) {
                            if (depth > 100) return null; // Prevent infinite recursion

                            const nodeInfo = {
                                node_id: node.id || Math.random().toString(36).substr(2, 9),
                                role: node.getAttribute?.('role') || node.tagName?.toLowerCase() || '',
                                name: node.getAttribute?.('aria-label') || node.textContent?.substr(0, 100) || '',
                                description: node.getAttribute?.('aria-describedby') || '',
                                tag_name: node.tagName?.toLowerCase() || '',
                                attributes: [],
                                children: [],
                                is_visible: node.offsetParent !== null,
                                is_interactive: ['button', 'a', 'input', 'select', 'textarea'].includes(node.tagName?.toLowerCase() || '')
                            };

                            // Get important attributes
                            if (node.id) nodeInfo.attributes.push(`id=${node.id}`);
                            if (node.className) nodeInfo.attributes.push(`class=${node.className}`);
                            if (node.type) nodeInfo.attributes.push(`type=${node.type}`);

                            // Process children
                            if (node.children) {
                                for (let child of node.children) {
                                    const childInfo = buildTree(child, depth + 1);
                                    if (childInfo) {
                                        nodeInfo.children.push(childInfo.node_id);
                                    }
                                }
                            }

                            return nodeInfo;
                        }

                        return JSON.stringify(buildTree(document.body));
                    })()
                "#;

                let snapshot = match page.evaluate(snapshot_script, false).await {
                    Ok(SessionEvaluationResult::String(json)) => {
                        serde_json::from_str::<serde_json::Value>(&json).ok()
                    }
                    _ => None,
                };

                let nodes = if let Some(_obj) = snapshot {
                    // Convert to NodeInfo (simplified version)
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
            Err(e) => {
                Ok(Response::new(GetSnapshotResponse {
                    response: Some(GetSnapshotResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn screenshot(&self, request: Request<ScreenshotRequest>) -> Result<Response<ScreenshotResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                let options = Self::proto_to_screenshot_options(req.options.unwrap_or_default());
                match page.screenshot(options).await {
                    Ok(data) => {
                        let result = ScreenshotResult {
                            data: bytes::Bytes::from(data).to_vec(),
                            format: "png".to_string(),
                            width: 1920,
                            height: 1080,
                        };
                        Ok(Response::new(ScreenshotResponse {
                            response: Some(ScreenshotResponseEnum::Result(result)),
                        }))
                    }
                    Err(e) => {
                        Ok(Response::new(ScreenshotResponse {
                            response: Some(ScreenshotResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(ScreenshotResponse {
                    response: Some(ScreenshotResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn evaluate(&self, request: Request<EvaluateRequest>) -> Result<Response<EvaluateResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                match page.evaluate(&req.expression, req.await_promise).await {
                    Ok(result) => {
                        let converted = Self::convert_evaluation_result(result);
                        Ok(Response::new(EvaluateResponse {
                            response: Some(EvaluateResponseEnum::Result(
                                Self::evaluation_result_to_proto(converted)
                            )),
                        }))
                    }
                    Err(e) => {
                        Ok(Response::new(EvaluateResponse {
                            response: Some(EvaluateResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(EvaluateResponse {
                    response: Some(EvaluateResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn evaluate_on_element(&self, request: Request<EvaluateOnElementRequest>) -> Result<Response<EvaluateOnElementResponse>, Status> {
        let req = request.into_inner();

        let element_ref = req.element.ok_or_else(|| Status::invalid_argument("element is required"))?;

        match self.session_manager.get_page(&element_ref.page_id).await {
            Ok(page) => {
                // Build script to evaluate on element
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
                        let converted = Self::convert_evaluation_result(result);
                        Ok(Response::new(EvaluateOnElementResponse {
                            response: Some(EvaluateOnElementResponseEnum::Result(
                                Self::evaluation_result_to_proto(converted)
                            )),
                        }))
                    }
                    Err(e) => {
                        Ok(Response::new(EvaluateOnElementResponse {
                            response: Some(EvaluateOnElementResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(EvaluateOnElementResponse {
                    response: Some(EvaluateOnElementResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn set_content(&self, request: Request<SetContentRequest>) -> Result<Response<SetContentResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                match page.set_content(&req.html).await {
                    Ok(_) => {
                        Ok(Response::new(SetContentResponse {
                            response: Some(SetContentResponseEnum::Success(Empty {})),
                        }))
                    }
                    Err(e) => {
                        Ok(Response::new(SetContentResponse {
                            response: Some(SetContentResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(SetContentResponse {
                    response: Some(SetContentResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn get_content(&self, request: Request<GetContentRequest>) -> Result<Response<GetContentResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                match page.get_content().await {
                    Ok(html) => {
                        let content = PageContent {
                            html: html.clone(),
                            text: html, // Placeholder - actual implementation would extract text
                            scripts: vec![],
                            stylesheets: vec![],
                        };
                        Ok(Response::new(GetContentResponse {
                            response: Some(GetContentResponseEnum::Content(content)),
                        }))
                    }
                    Err(e) => {
                        Ok(Response::new(GetContentResponse {
                            response: Some(GetContentResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(GetContentResponse {
                    response: Some(GetContentResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn reload(&self, request: Request<ReloadRequest>) -> Result<Response<ReloadResponse>, Status> {
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
                    Err(e) => {
                        Ok(Response::new(ReloadResponse {
                            response: Some(ReloadResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(ReloadResponse {
                    response: Some(ReloadResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn go_back(&self, request: Request<GoBackRequest>) -> Result<Response<GoBackResponse>, Status> {
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
                    Err(e) => {
                        Ok(Response::new(GoBackResponse {
                            response: Some(GoBackResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(GoBackResponse {
                    response: Some(GoBackResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn go_forward(&self, request: Request<GoForwardRequest>) -> Result<Response<GoForwardResponse>, Status> {
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
                    Err(e) => {
                        Ok(Response::new(GoForwardResponse {
                            response: Some(GoForwardResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(GoForwardResponse {
                    response: Some(GoForwardResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn set_viewport(&self, request: Request<SetViewportRequest>) -> Result<Response<SetViewportResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                let viewport = req.viewport.unwrap_or_default();
                match page.set_viewport(
                    viewport.width.max(0) as u32,
                    viewport.height.max(0) as u32,
                    viewport.device_scale_factor,
                ).await {
                    Ok(_) => {
                        Ok(Response::new(SetViewportResponse {
                            response: Some(SetViewportResponseEnum::Success(Empty {})),
                        }))
                    }
                    Err(e) => {
                        Ok(Response::new(SetViewportResponse {
                            response: Some(SetViewportResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(SetViewportResponse {
                    response: Some(SetViewportResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn emulate_device(&self, request: Request<EmulateDeviceRequest>) -> Result<Response<EmulateDeviceResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                // Define device presets
                let (width, height, device_scale_factor, _mobile, user_agent) = match req.device {
                    Some(device) => {
                        match device {
                            crate::chaser_oxide::v1::emulate_device_request::Device::DeviceType(device_type) => {
                                match device_type {
                                    1 => { // Desktop
                                        (1920, 1080, 1.0, false, None)
                                    }
                                    2 => { // Iphone
                                        (375, 667, 2.0, true, Some("Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1"))
                                    }
                                    3 => { // IphonePro
                                        (414, 896, 3.0, true, Some("Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1"))
                                    }
                                    4 => { // Ipad
                                        (768, 1024, 2.0, true, Some("Mozilla/5.0 (iPad; CPU OS 14_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1"))
                                    }
                                    5 => { // IpadPro
                                        (1024, 1366, 2.0, true, Some("Mozilla/5.0 (iPad; CPU OS 14_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1"))
                                    }
                                    6 => { // AndroidPhone
                                        (360, 640, 2.0, true, Some("Mozilla/5.0 (Linux; Android 10) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/86.0.4240.99 Mobile Safari/537.36"))
                                    }
                                    7 => { // AndroidTablet
                                        (800, 1280, 1.5, true, Some("Mozilla/5.0 (Linux; Android 10) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/86.0.4240.99 Safari/537.36"))
                                    }
                                    _ => (1920, 1080, 1.0, false, None)
                                }
                            }
                            crate::chaser_oxide::v1::emulate_device_request::Device::Viewport(viewport) => {
                                (viewport.width.max(0) as u32, viewport.height.max(0) as u32, viewport.device_scale_factor, false, None)
                            }
                        }
                    }
                    None => {
                        // Default to desktop
                        (1920, 1080, 1.0, false, None)
                    }
                };

                // Set viewport
                match page.set_viewport(width, height, device_scale_factor).await {
                    Ok(_) => {
                        // Set user agent if specified
                        if let Some(ua) = user_agent {
                            let _ = page.evaluate(
                                &format!("Object.defineProperty(navigator, 'userAgent', {{get: () => '{}'}})", ua),
                                false
                            ).await;
                        }

                        Ok(Response::new(EmulateDeviceResponse {
                            response: Some(EmulateDeviceResponseEnum::Success(Empty {})),
                        }))
                    }
                    Err(e) => {
                        Ok(Response::new(EmulateDeviceResponse {
                            response: Some(EmulateDeviceResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(EmulateDeviceResponse {
                    response: Some(EmulateDeviceResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn bring_to_front(&self, request: Request<BringToFrontRequest>) -> Result<Response<BringToFrontResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                // Use Page.bringToFront CDP command
                match page.evaluate("window.focus()", false).await {
                    Ok(_) => {
                        Ok(Response::new(BringToFrontResponse {
                            response: Some(BringToFrontResponseEnum::Success(Empty {})),
                        }))
                    }
                    Err(e) => {
                        Ok(Response::new(BringToFrontResponse {
                            response: Some(BringToFrontResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(BringToFrontResponse {
                    response: Some(BringToFrontResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn get_metrics(&self, request: Request<GetMetricsRequest>) -> Result<Response<GetMetricsResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                // Get performance metrics using JavaScript
                let metrics_script = r#"
                    (() => {
                        if (!window.performance || !window.performance.timing) {
                            return null;
                        }

                        const timing = window.performance.timing;
                        const navigationStart = timing.navigationStart;

                        return {
                            timestamp: Date.now().toString(),
                            layout_duration: timing.domContentLoadedEventEnd - timing.domContentLoadedEventStart,
                            recalculate_style_duration: timing.domComplete - timing.domLoading,
                            documents: document.querySelectorAll('document').length || 1,
                            frames: window.frames.length,
                            js_event_listeners: window.performance.getEntriesByType('resource').length,
                            layouts: [],
                            style_recalcs: []
                        };
                    })()
                "#;

                match page.evaluate(metrics_script, false).await {
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
                        // Return default metrics if evaluation returns non-object
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
                    Err(e) => {
                        Ok(Response::new(GetMetricsResponse {
                            response: Some(GetMetricsResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(GetMetricsResponse {
                    response: Some(GetMetricsResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn close_page(&self, request: Request<ClosePageRequest>) -> Result<Response<ClosePageResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.close_page(&req.page_id).await {
            Ok(_) => {
                Ok(Response::new(ClosePageResponse {
                    response: Some(ClosePageResponseEnum::Success(Empty {})),
                }))
            }
            Err(e) => {
                Ok(Response::new(ClosePageResponse {
                    response: Some(ClosePageResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn wait_for(&self, request: Request<WaitForRequest>) -> Result<Response<WaitForResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                // Wait condition based on request
                let wait_result = match req.wait_condition {
                    Some(condition) => {
                        match condition {
                            crate::chaser_oxide::v1::wait_for_request::WaitCondition::Selector(selector) => {
                                // Wait for selector to appear
                                let script = format!(
                                    r#"
                                    (() => {{
                                        return new Promise((resolve) => {{
                                            const check = () => {{
                                                const element = document.querySelector('{}');
                                                if (element) {{
                                                    resolve(true);
                                                }} else {{
                                                    setTimeout(check, 100);
                                                }}
                                            }};
                                            check();
                                        }});
                                    }})()
                                    "#,
                                    selector
                                );

                                page.evaluate(&script, false).await
                            }
                            crate::chaser_oxide::v1::wait_for_request::WaitCondition::Timeout(timeout_ms) => {
                                // Just wait for specified timeout
                                tokio::time::sleep(tokio::time::Duration::from_millis(timeout_ms as u64)).await;
                                Ok(SessionEvaluationResult::Bool(true))
                            }
                            crate::chaser_oxide::v1::wait_for_request::WaitCondition::NavigationUrl(url) => {
                                // Wait for navigation to specific URL
                                let script = format!(
                                    r#"
                                    (() => {{
                                        return new Promise((resolve) => {{
                                            const check = () => {{
                                                if (window.location.href === '{}') {{
                                                    resolve(true);
                                                }} else {{
                                                    setTimeout(check, 100);
                                                }}
                                            }};
                                            check();
                                        }});
                                    }})()
                                    "#,
                                    url
                                );

                                page.evaluate(&script, false).await
                            }
                        }
                    }
                    None => {
                        // Default: wait a short time
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
                    Err(e) => {
                        Ok(Response::new(WaitForResponse {
                            response: Some(WaitForResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(WaitForResponse {
                    response: Some(WaitForResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn get_pdf(&self, request: Request<GetPdfRequest>) -> Result<Response<GetPdfResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(_page) => {
                // Note: Full CDP Page.printToPDF implementation requires:
                // 1. CDP connection support for Page.printToPDF command
                // 2. PDF options handling (landscape, display_header_footer, etc.)
                // 3. Base64-encoded PDF data decoding
                //
                // For now, return a not-yet-implemented error
                let error = Self::error_to_proto(Error::internal(
                    "PDF generation via CDP Page.printToPDF is not yet implemented. \
                     Please add CDP support for Page.printToPDF command to enable this feature."
                ));

                Ok(Response::new(GetPdfResponse {
                    response: Some(GetPdfResponseEnum::Error(error)),
                }))
            }
            Err(e) => {
                Ok(Response::new(GetPdfResponse {
                    response: Some(GetPdfResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn add_init_script(&self, request: Request<AddInitScriptRequest>) -> Result<Response<AddInitScriptResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                // Add script to evaluate on new documents
                // For now, just evaluate it immediately
                match page.evaluate(&req.script, false).await {
                    Ok(_) => {
                        Ok(Response::new(AddInitScriptResponse {
                            response: Some(AddInitScriptResponseEnum::Success(Empty {})),
                        }))
                    }
                    Err(e) => {
                        Ok(Response::new(AddInitScriptResponse {
                            response: Some(AddInitScriptResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(AddInitScriptResponse {
                    response: Some(AddInitScriptResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn get_cookies(&self, request: Request<GetCookiesRequest>) -> Result<Response<GetCookiesResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                // Use Network.getCookies CDP command
                let script = r#"
                    (() => {
                        return document.cookie.split(';').map(cookie => {
                            const [name, value] = cookie.trim().split('=');
                            return {
                                name: name || '',
                                value: value || '',
                                domain: window.location.hostname,
                                path: '/',
                                expires: 0,
                                size: cookie.length,
                                http_only: false,
                                secure: window.location.protocol === 'https:',
                                session: true,
                                same_site: 'Lax'
                            };
                        }).filter(c => c.name);
                    })()
                "#;

                match page.evaluate(script, false).await {
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
                    Err(e) => {
                        Ok(Response::new(GetCookiesResponse {
                            response: Some(GetCookiesResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(GetCookiesResponse {
                    response: Some(GetCookiesResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn set_cookies(&self, request: Request<SetCookiesRequest>) -> Result<Response<SetCookiesResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                // Set each cookie using document.cookie
                for cookie in req.cookies {
                    let cookie_string = format!("{}={}; Domain={}; Path={}",
                        cookie.name, cookie.value, cookie.domain, cookie.path);

                    let script = format!("document.cookie = {}", serde_json::json!(cookie_string));

                    match page.evaluate(&script, false).await {
                        Ok(_) => continue,
                        Err(e) => {
                            return Ok(Response::new(SetCookiesResponse {
                                response: Some(SetCookiesResponseEnum::Error(
                                    Self::error_to_proto(e)
                                )),
                            }))
                        }
                    }
                }

                Ok(Response::new(SetCookiesResponse {
                    response: Some(SetCookiesResponseEnum::Success(Empty {})),
                }))
            }
            Err(e) => {
                Ok(Response::new(SetCookiesResponse {
                    response: Some(SetCookiesResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn clear_cookies(&self, request: Request<ClearCookiesRequest>) -> Result<Response<ClearCookiesResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                // Clear all cookies by setting them to expire in the past
                let script = r#"
                    (() => {
                        const cookies = document.cookie.split(';');
                        cookies.forEach(cookie => {
                            const [name] = cookie.trim().split('=');
                            if (name) {
                                document.cookie = name + '=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';
                            }
                        });
                        return true;
                    })()
                "#;

                match page.evaluate(script, false).await {
                    Ok(_) => {
                        Ok(Response::new(ClearCookiesResponse {
                            response: Some(ClearCookiesResponseEnum::Success(Empty {})),
                        }))
                    }
                    Err(e) => {
                        Ok(Response::new(ClearCookiesResponse {
                            response: Some(ClearCookiesResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(ClearCookiesResponse {
                    response: Some(ClearCookiesResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn override_permissions(&self, request: Request<OverridePermissionsRequest>) -> Result<Response<OverridePermissionsResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                // Use Browser.grantPermissions CDP command
                // For now, set navigator.permissions using JavaScript
                let permissions_script = format!(r#"
                    (() => {{
                        const permissions = {};
                        // Override permissions API
                        const originalQuery = navigator.permissions.query;
                        navigator.permissions.query = (name) => {{
                            return Promise.resolve({{ state: 'granted' }});
                        }};
                        return true;
                    }})()
                "#, serde_json::to_string(&req.permissions).unwrap_or_default());

                match page.evaluate(&permissions_script, false).await {
                    Ok(_) => {
                        Ok(Response::new(OverridePermissionsResponse {
                            response: Some(OverridePermissionsResponseEnum::Success(Empty {})),
                        }))
                    }
                    Err(e) => {
                        Ok(Response::new(OverridePermissionsResponse {
                            response: Some(OverridePermissionsResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(OverridePermissionsResponse {
                    response: Some(OverridePermissionsResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn set_geolocation(&self, request: Request<SetGeolocationRequest>) -> Result<Response<SetGeolocationResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                // Use Emulation.setGeolocationOverride CDP command
                let geo_script = format!(r#"
                    (() => {{
                        // Override navigator.geolocation
                        const mockGeolocation = {{
                            getCurrentPosition: (success) => {{
                                success({{
                                    coords: {{
                                        latitude: {},
                                        longitude: {},
                                        accuracy: {},
                                        altitude: null,
                                        altitudeAccuracy: null,
                                        heading: null,
                                        speed: null
                                    }},
                                    timestamp: Date.now()
                                }});
                            }},
                            watchPosition: (success) => {{
                                success({{
                                    coords: {{
                                        latitude: {},
                                        longitude: {},
                                        accuracy: {},
                                        altitude: null,
                                        altitudeAccuracy: null,
                                        heading: null,
                                        speed: null
                                    }},
                                    timestamp: Date.now()
                                }});
                                return 0;
                            }},
                            clearWatch: () => {{}}
                        }};

                        Object.defineProperty(navigator, 'geolocation', {{
                            get: () => mockGeolocation
                        }});

                        return true;
                    }})()
                "#, req.latitude, req.longitude, req.accuracy, req.latitude, req.longitude, req.accuracy);

                match page.evaluate(&geo_script, false).await {
                    Ok(_) => {
                        Ok(Response::new(SetGeolocationResponse {
                            response: Some(SetGeolocationResponseEnum::Success(Empty {})),
                        }))
                    }
                    Err(e) => {
                        Ok(Response::new(SetGeolocationResponse {
                            response: Some(SetGeolocationResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(SetGeolocationResponse {
                    response: Some(SetGeolocationResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn set_offline_mode(&self, request: Request<SetOfflineModeRequest>) -> Result<Response<SetOfflineModeResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                // Use Network.emulateNetworkConditions CDP command
                let offline_script = format!(r#"
                    (() => {{
                        // Override navigator.onLine
                        Object.defineProperty(navigator, 'onLine', {{
                            get: () => {}
                        }});

                        // Override window.navigator.connection
                        if (navigator.connection) {{
                            Object.defineProperty(navigator.connection, 'effectiveType', {{
                                get: () => 'slow-2g'
                            }});
                        }}

                        return true;
                    }})()
                "#, req.offline);

                match page.evaluate(&offline_script, false).await {
                    Ok(_) => {
                        Ok(Response::new(SetOfflineModeResponse {
                            response: Some(SetOfflineModeResponseEnum::Success(Empty {})),
                        }))
                    }
                    Err(e) => {
                        Ok(Response::new(SetOfflineModeResponse {
                            response: Some(SetOfflineModeResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(SetOfflineModeResponse {
                    response: Some(SetOfflineModeResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn set_cache_enabled(&self, request: Request<SetCacheEnabledRequest>) -> Result<Response<SetCacheEnabledResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                // Use Network.setCacheDisabled CDP command
                let cache_script = format!(r#"
                    (() => {{
                        // Override fetch API to control caching
                        const originalFetch = window.fetch;
                        window.fetch = function(...args) {{
                            const options = args[1] || {{}};
                            if (!{}) {{
                                options.cache = 'no-store';
                            }}
                            return originalFetch.apply(this, [args[0], options]);
                        }};
                        return true;
                    }})()
                "#, !req.enabled);

                match page.evaluate(&cache_script, false).await {
                    Ok(_) => {
                        Ok(Response::new(SetCacheEnabledResponse {
                            response: Some(SetCacheEnabledResponseEnum::Success(Empty {})),
                        }))
                    }
                    Err(e) => {
                        Ok(Response::new(SetCacheEnabledResponse {
                            response: Some(SetCacheEnabledResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(SetCacheEnabledResponse {
                    response: Some(SetCacheEnabledResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::mock::MockSessionManager;
    use tonic::Request;

    // Test 1: Service creation
    #[tokio::test]
    async fn test_page_service_creation() {
        let session_manager = Arc::new(MockSessionManager::new());
        let service = Service::new(session_manager);
        assert!(true, "Service created successfully");
    }

    // Test 2: Proto navigation options conversion
    #[tokio::test]
    async fn test_proto_conversion() {
        let proto_opts = ProtoNavigationOptions {
            timeout: 30000,
            wait_until: navigation_options::LoadState::NetworkIdle as i32,
            ..Default::default()
        };

        let opts = Service::<MockSessionManager>::proto_to_navigation_options(proto_opts);
        assert_eq!(opts.timeout, 30000);
        assert_eq!(matches!(opts.wait_until, LoadState::NetworkIdle), true);
    }

    // Test 3: Evaluation result string conversion
    #[tokio::test]
    async fn test_evaluation_result_string_conversion() {
        let result = crate::services::traits::EvaluationResult::String("test".to_string());
        let proto = Service::<MockSessionManager>::evaluation_result_to_proto(result);
        assert!(matches!(proto.response, Some(evaluation_result::Response::StringValue(_))));
        assert_eq!(proto.r#type, "string");
    }

    // Test 4: Evaluation result number conversion
    #[tokio::test]
    async fn test_evaluation_result_number_conversion() {
        let result = crate::services::traits::EvaluationResult::Number(42.5);
        let proto = Service::<MockSessionManager>::evaluation_result_to_proto(result);
        assert!(matches!(proto.response, Some(evaluation_result::Response::DoubleValue(42.5))));
        assert_eq!(proto.r#type, "number");
    }

    // Test 5: Evaluation result bool conversion
    #[tokio::test]
    async fn test_evaluation_result_bool_conversion() {
        let result = crate::services::traits::EvaluationResult::Bool(true);
        let proto = Service::<MockSessionManager>::evaluation_result_to_proto(result);
        assert!(matches!(proto.response, Some(evaluation_result::Response::BoolValue(true))));
        assert_eq!(proto.r#type, "boolean");
    }

    // Test 6: Evaluation result null conversion
    #[tokio::test]
    async fn test_evaluation_result_null_conversion() {
        let result = crate::services::traits::EvaluationResult::Null;
        let proto = Service::<MockSessionManager>::evaluation_result_to_proto(result);
        assert!(matches!(proto.response, Some(evaluation_result::Response::NullValue(_))));
        assert_eq!(proto.r#type, "null");
    }

    // Test 7: Screenshot options conversion PNG
    #[tokio::test]
    async fn test_screenshot_options_png_conversion() {
        let proto_opts = ProtoScreenshotOptions {
            format: screenshot_options::Format::Png as i32,
            quality: 0,
            full_page: false,
            ..Default::default()
        };

        let opts = Service::<MockSessionManager>::proto_to_screenshot_options(proto_opts);
        assert_eq!(matches!(opts.format, ScreenshotFormat::Png), true);
        assert_eq!(opts.quality, None);
        assert_eq!(opts.full_page, false);
    }

    // Test 8: Screenshot options conversion JPEG
    #[tokio::test]
    async fn test_screenshot_options_jpeg_conversion() {
        let proto_opts = ProtoScreenshotOptions {
            format: screenshot_options::Format::Jpeg as i32,
            quality: 90,
            full_page: true,
            ..Default::default()
        };

        let opts = Service::<MockSessionManager>::proto_to_screenshot_options(proto_opts);
        assert_eq!(matches!(opts.format, ScreenshotFormat::Jpeg), true);
        assert_eq!(opts.quality, Some(90));
        assert_eq!(opts.full_page, true);
    }

    // Test 9: Screenshot options conversion WebP
    #[tokio::test]
    async fn test_screenshot_options_webp_conversion() {
        let proto_opts = ProtoScreenshotOptions {
            format: screenshot_options::Format::Webp as i32,
            quality: 80,
            full_page: false,
            ..Default::default()
        };

        let opts = Service::<MockSessionManager>::proto_to_screenshot_options(proto_opts);
        assert_eq!(matches!(opts.format, ScreenshotFormat::WebP), true);
        assert_eq!(opts.quality, Some(80));
    }

    // Test 10: Error conversion - PageNotFound
    #[tokio::test]
    async fn test_error_conversion_page_not_found() {
        let error = Error::page_not_found("test-page");
        let proto_error = Service::<MockSessionManager>::error_to_proto(error);
        assert_eq!(proto_error.code, ErrorCode::PageClosed as i32);
    }

    // Test 11: Error conversion - Timeout
    #[tokio::test]
    async fn test_error_conversion_timeout() {
        let error = Error::timeout("Operation timed out");
        let proto_error = Service::<MockSessionManager>::error_to_proto(error);
        assert_eq!(proto_error.code, ErrorCode::Timeout as i32);
    }

    // Test 12: Error conversion - NavigationFailed
    #[tokio::test]
    async fn test_error_conversion_navigation_failed() {
        let error = Error::navigation_failed("Navigation failed");
        let proto_error = Service::<MockSessionManager>::error_to_proto(error);
        assert_eq!(proto_error.code, ErrorCode::NavigationFailed as i32);
    }

    // Test 13: Error conversion - ScriptExecutionFailed
    #[tokio::test]
    async fn test_error_conversion_script_execution_failed() {
        let error = Error::script_execution_failed("Script error");
        let proto_error = Service::<MockSessionManager>::error_to_proto(error);
        assert_eq!(proto_error.code, ErrorCode::EvaluationFailed as i32);
    }

    // Test 14: Error conversion - Internal
    #[tokio::test]
    async fn test_error_conversion_internal() {
        let error = Error::internal("Internal error");
        let proto_error = Service::<MockSessionManager>::error_to_proto(error);
        assert_eq!(proto_error.code, ErrorCode::Internal as i32);
    }

    // Test 15: Navigation options load states
    #[tokio::test]
    async fn test_navigation_options_load_states() {
        // Test Load state
        let opts1 = ProtoNavigationOptions {
            timeout: 10000,
            wait_until: navigation_options::LoadState::Load as i32,
            ..Default::default()
        };
        let converted1 = Service::<MockSessionManager>::proto_to_navigation_options(opts1);
        assert_eq!(matches!(converted1.wait_until, LoadState::Load), true);

        // Test DomContentLoaded state
        let opts2 = ProtoNavigationOptions {
            timeout: 10000,
            wait_until: navigation_options::LoadState::DomContentLoaded as i32,
            ..Default::default()
        };
        let converted2 = Service::<MockSessionManager>::proto_to_navigation_options(opts2);
        assert_eq!(matches!(converted2.wait_until, LoadState::DOMContentLoaded), true);

        // Test NetworkAlmostIdle state
        let opts3 = ProtoNavigationOptions {
            timeout: 10000,
            wait_until: navigation_options::LoadState::NetworkAlmostIdle as i32,
            ..Default::default()
        };
        let converted3 = Service::<MockSessionManager>::proto_to_navigation_options(opts3);
        assert_eq!(matches!(converted3.wait_until, LoadState::NetworkAlmostIdle), true);
    }

    // Test 16: Page info to proto conversion
    #[tokio::test]
    async fn test_page_info_to_proto_conversion() {
        let info = PageInfo {
            page_id: "page-123".to_string(),
            url: "https://example.com".to_string(),
            title: "Example Page".to_string(),
        };

        let proto = Service::<MockSessionManager>::page_info_to_proto(info);
        assert_eq!(proto.page_id, "page-123");
        assert_eq!(proto.url, "https://example.com");
        assert_eq!(proto.title, "Example Page");
        assert_eq!(proto.is_loaded, true);
    }

    // Test 17: Evaluation result object conversion
    #[tokio::test]
    async fn test_evaluation_result_object_conversion() {
        let json_value = serde_json::json!({"key": "value"});
        let result = crate::services::traits::EvaluationResult::Object(json_value);
        let proto = Service::<MockSessionManager>::evaluation_result_to_proto(result);
        assert!(matches!(proto.response, Some(evaluation_result::Response::StringValue(_))));
        assert_eq!(proto.r#type, "object");
    }

    // Test 18: Proto options with zero timeout
    #[tokio::test]
    async fn test_proto_options_zero_timeout() {
        let proto_opts = ProtoNavigationOptions {
            timeout: 0,
            wait_until: navigation_options::LoadState::Unspecified as i32,
            ..Default::default()
        };

        let opts = Service::<MockSessionManager>::proto_to_navigation_options(proto_opts);
        assert_eq!(opts.timeout, 0);
        assert_eq!(matches!(opts.wait_until, LoadState::Load), true); // Default
    }

    // Test 19: Screenshot options unspecified format defaults to PNG
    #[tokio::test]
    async fn test_screenshot_options_unspecified_format() {
        let proto_opts = ProtoScreenshotOptions {
            format: screenshot_options::Format::Unspecified as i32,
            quality: 0,
            full_page: false,
            ..Default::default()
        };

        let opts = Service::<MockSessionManager>::proto_to_screenshot_options(proto_opts);
        assert_eq!(matches!(opts.format, ScreenshotFormat::Png), true); // Default
    }
}
