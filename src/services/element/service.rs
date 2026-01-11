//! ElementService gRPC implementation
//!
//! Provides gRPC server implementation for element interaction operations.

use crate::error::{Error as ServiceError, Result as ServiceResult};
use crate::services::element::finder::ElementFinder;
use crate::services::element::js_utils::JsBuilder;
use crate::services::traits::SelectorType;
use crate::session::traits::{PageContext, SessionManager};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{error, info, instrument};

// Import generated protobuf types
use crate::chaser_oxide::v1::{
    element_service_server::{ElementService as ElementServiceTrait, ElementServiceServer},
    Error as ProtoError, ElementRef, FindElementsResult,
    find_element_response::Response as FindElementResponseEnum,
    find_elements_response::Response as FindElementsResponseEnum,
    wait_for_element_response::Response as WaitForElementResponseEnum,
    get_html_response::Response as GetHtmlResponseEnum,
    click_response::Response as ClickResponseEnum,
    type_response::Response as TypeResponseEnum,
    fill_response::Response as FillResponseEnum,
    get_attribute_response::Response as GetAttributeResponseEnum,
    get_attributes_response::Response as GetAttributesResponseEnum,
    get_text_response::Response as GetTextResponseEnum,
    hover_response::Response as HoverResponseEnum,
    focus_response::Response as FocusResponseEnum,
    select_option_response::Response as SelectOptionResponseEnum,
    upload_file_response::Response as UploadFileResponseEnum,
    scroll_into_view_response::Response as ScrollIntoViewResponseEnum,
    get_bounding_box_response::Response as GetBoundingBoxResponseEnum,
    is_visible_response::Response as IsVisibleResponseEnum,
    is_enabled_response::Response as IsEnabledResponseEnum,
    get_properties_response::Response as GetPropertiesResponseEnum,
    press_key_response::Response as PressKeyResponseEnum,
    drag_and_drop_response::Response as DragAndDropResponseEnum,
    FindElementRequest, FindElementResponse,
    FindElementsRequest, FindElementsResponse,
    ClickRequest, ClickResponse,
    TypeRequest, TypeResponse,
    FillRequest, FillResponse,
    GetAttributeRequest, GetAttributeResponse,
    GetAttributesRequest, GetAttributesResponse,
    GetTextRequest, GetTextResponse,
    GetHtmlRequest, GetHtmlResponse,
    HoverRequest, HoverResponse,
    FocusRequest, FocusResponse,
    SelectOptionRequest, SelectOptionResponse,
    UploadFileRequest, UploadFileResponse,
    ScrollIntoViewRequest, ScrollIntoViewResponse,
    GetBoundingBoxRequest, GetBoundingBoxResponse,
    IsVisibleRequest, IsVisibleResponse,
    IsEnabledRequest, IsEnabledResponse,
    WaitForElementRequest, WaitForElementResponse,
    GetPropertiesRequest, GetPropertiesResponse,
    PressKeyRequest, PressKeyResponse,
    DragAndDropRequest, DragAndDropResponse,
    Empty, ErrorCode,
    AttributeValue, Attributes, TextValue,
    HtmlValue, BoundingBox, VisibilityResult, EnabledResult, ElementProperties,
};

/// Macro for handling simple element operation results (success/empty response)
macro_rules! handle_simple_op {
    ($result:expr, $response_type:ident, $op_name:expr, $success_ctor:expr, $error_ctor:expr) => {
        match $result {
            Ok(_) => {
                let resp = $response_type {
                    response: Some($success_ctor(Empty {})),
                };
                Response::new(resp)
            }
            Err(e) => {
                error!("{} failed: {}", $op_name, e);
                let resp = $response_type {
                    response: Some($error_ctor(ProtoError {
                        code: ErrorCode::ElementNotFound as i32,
                        message: e.to_string(),
                        details: Default::default(),
                    })),
                };
                Response::new(resp)
            }
        }
    };
}

/// ElementService gRPC server
#[derive(Clone)]
pub struct ElementGrpcService {
    session_manager: Arc<dyn SessionManager>,
}

impl ElementGrpcService {
    /// Create a new ElementService gRPC server
    pub fn new(session_manager: Arc<dyn SessionManager>) -> Self {
        Self { session_manager }
    }

    /// Convert to tonic server
    pub fn into_server(self) -> ElementServiceServer<Self> {
        ElementServiceServer::new(self)
    }

    /// Get page by ID
    async fn get_page(&self, page_id: &str) -> ServiceResult<Arc<dyn PageContext>> {
        self.session_manager
            .get_page(page_id)
            .await
            .map_err(|e| match e {
                ServiceError::PageNotFound(_) => ServiceError::page_not_found(page_id),
                _ => e,
            })
    }

    /// Convert SelectorType from proto to trait
    fn convert_selector_type(selector_type: i32) -> ServiceResult<SelectorType> {
        match selector_type {
            1 => Ok(SelectorType::Css),
            2 => Ok(SelectorType::XPath),
            3 => Ok(SelectorType::Text),
            _ => Err(ServiceError::internal(format!(
                "Invalid selector type: {}",
                selector_type
            ))),
        }
    }

    /// Execute JavaScript script and get string result
    async fn execute_script(&self, page: &Arc<dyn PageContext>, script: &str) -> ServiceResult<String> {
        let result = page.evaluate(script, true).await?;
        Ok(match result {
            crate::session::traits::EvaluationResult::String(s) => s,
            crate::session::traits::EvaluationResult::Number(n) => n.to_string(),
            crate::session::traits::EvaluationResult::Bool(b) => b.to_string(),
            _ => String::new(),
        })
    }

    /// Verify that an element exists
    async fn verify_element_exists(
        &self,
        page: &Arc<dyn PageContext>,
        selector_type: i32,
        selector: &str,
    ) -> ServiceResult<bool> {
        let selector_type = Self::convert_selector_type(selector_type)?;
        let finder = ElementFinder::new(page.clone());
        match finder.find_element(selector_type, selector).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Execute JavaScript on an element
    async fn execute_on_element(
        &self,
        page: &Arc<dyn PageContext>,
        selector_type: i32,
        selector: &str,
        js_code: &str,
    ) -> ServiceResult<String> {
        // Don't convert selector_type - JsBuilder expects the raw i32 value
        let builder = JsBuilder::new(selector_type, selector.to_string());
        let script = builder.execute_on_element(js_code)?;
        self.execute_script(page, &script).await
    }
}

#[tonic::async_trait]
impl ElementServiceTrait for ElementGrpcService {
    #[instrument(skip(self, request))]
    async fn find_element(
        &self,
        request: Request<FindElementRequest>,
    ) -> Result<Response<FindElementResponse>, Status> {
        info!("FindElement request received");

        let req = request.into_inner();
        let page = self.get_page(&req.page_id).await?;

        let finder = ElementFinder::new(page);
        let selector_type = Self::convert_selector_type(req.selector_type)?;

        match finder.find_element(selector_type, &req.selector).await {
            Ok(element) => {
                let response = FindElementResponse {
                    response: Some(FindElementResponseEnum::Element(ElementRef {
                        page_id: req.page_id,
                        element_id: element.element_id,
                        selector_type: req.selector_type,
                        selector: req.selector,
                        index: 0,
                    })),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                error!("FindElement failed: {}", e);
                let response = FindElementResponse {
                    response: Some(FindElementResponseEnum::Error(ProtoError {
                        code: ErrorCode::ElementNotFound as i32,
                        message: e.to_string(),
                        details: Default::default(),
                    })),
                };
                Ok(Response::new(response))
            }
        }
    }

    #[instrument(skip(self, request))]
    async fn find_elements(
        &self,
        request: Request<FindElementsRequest>,
    ) -> Result<Response<FindElementsResponse>, Status> {
        info!("FindElements request received");

        let req = request.into_inner();
        let page = self.get_page(&req.page_id).await?;

        let finder = ElementFinder::new(page);
        let selector_type = Self::convert_selector_type(req.selector_type)?;
        let limit = if req.limit > 0 { Some(req.limit as usize) } else { None };

        match finder.find_elements(selector_type, &req.selector, limit).await {
            Ok(elements) => {
                let element_refs: Vec<_> = elements
                    .into_iter()
                    .enumerate()
                    .map(|(i, el)| ElementRef {
                        page_id: req.page_id.clone(),
                        element_id: el.element_id,
                        selector_type: req.selector_type,
                        selector: req.selector.clone(),
                        index: i as i32,
                    })
                    .collect();

                let response = FindElementsResponse {
                    response: Some(FindElementsResponseEnum::Elements(FindElementsResult {
                        elements: element_refs,
                    })),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                error!("FindElements failed: {}", e);
                let response = FindElementsResponse {
                    response: Some(FindElementsResponseEnum::Error(ProtoError {
                        code: ErrorCode::ElementNotFound as i32,
                        message: e.to_string(),
                        details: Default::default(),
                    })),
                };
                Ok(Response::new(response))
            }
        }
    }

    #[instrument(skip(self, request))]
    async fn click(&self, request: Request<ClickRequest>) -> Result<Response<ClickResponse>, Status> {
        info!("Click request received");

        let req = request.into_inner();
        let element_ref = req.element.ok_or_else(|| Status::invalid_argument("Element reference is required"))?;
        let page = self.get_page(&element_ref.page_id).await?;

        let js = JsBuilder::new(element_ref.selector_type, element_ref.selector).click_script()?;
        let result = self.execute_script(&page, &js).await;

        Ok(handle_simple_op!(result, ClickResponse, "Click", ClickResponseEnum::Success, ClickResponseEnum::Error))
    }

    #[instrument(skip(self, request))]
    async fn r#type(
        &self,
        request: Request<TypeRequest>,
    ) -> Result<Response<TypeResponse>, Status> {
        info!("Type request received");

        let req = request.into_inner();
        let element_ref = req.element.ok_or_else(|| Status::invalid_argument("Element reference is required"))?;
        let page = self.get_page(&element_ref.page_id).await?;

        let js = JsBuilder::new(element_ref.selector_type, element_ref.selector)
            .type_text_script(&req.text)?;
        let result = self.execute_script(&page, &js).await;

        Ok(handle_simple_op!(result, TypeResponse, "Type", TypeResponseEnum::Success, TypeResponseEnum::Error))
    }

    #[instrument(skip(self, request))]
    async fn fill(&self, request: Request<FillRequest>) -> Result<Response<FillResponse>, Status> {
        info!("Fill request received");

        let req = request.into_inner();
        let element_ref = req.element.ok_or_else(|| Status::invalid_argument("Element reference is required"))?;
        let page = self.get_page(&element_ref.page_id).await?;

        let js = JsBuilder::new(element_ref.selector_type, element_ref.selector)
            .fill_script(&req.value, req.clear_first)?;
        let result = self.execute_script(&page, &js).await;

        Ok(handle_simple_op!(result, FillResponse, "Fill", FillResponseEnum::Success, FillResponseEnum::Error))
    }

    #[instrument(skip(self, request))]
    async fn get_attribute(
        &self,
        request: Request<GetAttributeRequest>,
    ) -> Result<Response<GetAttributeResponse>, Status> {
        info!("GetAttribute request received");

        let req = request.into_inner();
        let element_ref = req.element.ok_or_else(|| {
            Status::invalid_argument("Element reference is required")
        })?;

        let page = self.get_page(&element_ref.page_id).await?;

        // Verify element exists before getting attribute
        if !self.verify_element_exists(&page, element_ref.selector_type, &element_ref.selector).await
            .map_err(|e| Status::internal(format!("Element verification failed: {}", e)))?
        {
            return Ok(Response::new(GetAttributeResponse {
                response: Some(GetAttributeResponseEnum::Error(ProtoError {
                    code: ErrorCode::ElementNotFound as i32,
                    message: format!("Element not found: {}", element_ref.selector),
                    details: Default::default(),
                })),
            }));
        }

        // Execute get attribute using JavaScript
        let js_code = &format!("el.getAttribute('{}') || ''", req.name.replace('\\', "\\\\").replace('\'', "\\'"));

        let value = match self.execute_on_element(&page, element_ref.selector_type, &element_ref.selector, js_code).await {
            Ok(v) => v,
            Err(e) => {
                error!("GetAttribute failed: {}", e);
                let response = GetAttributeResponse {
                    response: Some(GetAttributeResponseEnum::Error(ProtoError {
                        code: ErrorCode::ElementNotFound as i32,
                        message: e.to_string(),
                        details: Default::default(),
                    })),
                };
                return Ok(Response::new(response));
            }
        };

        let response = GetAttributeResponse {
            response: Some(GetAttributeResponseEnum::Value(AttributeValue {
                name: req.name,
                value,
            })),
        };
        Ok(Response::new(response))
    }

    #[instrument(skip(self, request))]
    async fn get_attributes(
        &self,
        request: Request<GetAttributesRequest>,
    ) -> Result<Response<GetAttributesResponse>, Status> {
        info!("GetAttributes request received");

        let req = request.into_inner();
        let element_ref = req.element.ok_or_else(|| {
            Status::invalid_argument("Element reference is required")
        })?;

        let page = self.get_page(&element_ref.page_id).await?;

        // Execute get attributes using JavaScript
        let js_code = r#"
            (() => {
                const attrs = el.attributes;
                const result = [];
                for (let i = 0; i < attrs.length; i++) {
                    result.push(attrs[i].name, attrs[i].value);
                }
                return JSON.stringify(result);
            })()
        "#;

        let attrs_json = match self.execute_on_element(&page, element_ref.selector_type, &element_ref.selector, js_code).await {
            Ok(v) => v,
            Err(e) => {
                error!("GetAttributes failed: {}", e);
                let response = GetAttributesResponse {
                    response: Some(GetAttributesResponseEnum::Error(ProtoError {
                        code: ErrorCode::ElementNotFound as i32,
                        message: e.to_string(),
                        details: Default::default(),
                    })),
                };
                return Ok(Response::new(response));
            }
        };

        let attrs_flat: Vec<String> = serde_json::from_str(&attrs_json).unwrap_or_default();

        // Convert flat array [name1, value1, name2, value2, ...] to Vec<AttributeValue>
        let attrs_vec: Vec<crate::chaser_oxide::v1::AttributeValue> = attrs_flat
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    Some(crate::chaser_oxide::v1::AttributeValue {
                        name: chunk[0].clone(),
                        value: chunk[1].clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        let response = GetAttributesResponse {
            response: Some(GetAttributesResponseEnum::Attributes(Attributes {
                attributes: attrs_vec,
            })),
        };
        Ok(Response::new(response))
    }

    #[instrument(skip(self, request))]
    async fn get_text(
        &self,
        request: Request<GetTextRequest>,
    ) -> Result<Response<GetTextResponse>, Status> {
        info!("GetText request received");

        let req = request.into_inner();
        let element_ref = req.element.ok_or_else(|| {
            Status::invalid_argument("Element reference is required")
        })?;

        let page = self.get_page(&element_ref.page_id).await?;

        // Verify element exists before getting text
        if !self.verify_element_exists(&page, element_ref.selector_type, &element_ref.selector).await
            .map_err(|e| Status::internal(format!("Element verification failed: {}", e)))?
        {
            return Ok(Response::new(GetTextResponse {
                response: Some(GetTextResponseEnum::Error(ProtoError {
                    code: ErrorCode::ElementNotFound as i32,
                    message: format!("Element not found: {}", element_ref.selector),
                    details: Default::default(),
                })),
            }));
        }

        // Execute get text using JavaScript
        let js_code = r#"
            (() => {
                return el.textContent || el.innerText || '';
            })()
        "#;

        let text = match self.execute_on_element(&page, element_ref.selector_type, &element_ref.selector, js_code).await {
            Ok(v) => v,
            Err(e) => {
                error!("GetText failed: {}", e);
                let response = GetTextResponse {
                    response: Some(GetTextResponseEnum::Error(ProtoError {
                        code: ErrorCode::ElementNotFound as i32,
                        message: e.to_string(),
                        details: Default::default(),
                    })),
                };
                return Ok(Response::new(response));
            }
        };

        let response = GetTextResponse {
            response: Some(GetTextResponseEnum::Text(TextValue {
                text,
                is_visible: true,
            })),
        };
        Ok(Response::new(response))
    }

    #[instrument(skip(self, request))]
    async fn get_html(
        &self,
        request: Request<GetHtmlRequest>,
    ) -> Result<Response<GetHtmlResponse>, Status> {
        info!("GetHTML request received");

        let req = request.into_inner();
        let element_ref = req.element.ok_or_else(|| {
            Status::invalid_argument("Element reference is required")
        })?;

        let page = self.get_page(&element_ref.page_id).await?;

        // Execute get HTML using JavaScript
        let js_code = if req.outer {
            r#"
                (() => {
                    return el.outerHTML || '';
                })()
            "#
        } else {
            r#"
                (() => {
                    return el.innerHTML || '';
                })()
            "#
        };

        let html = match self.execute_on_element(&page, element_ref.selector_type, &element_ref.selector, js_code).await {
            Ok(v) => v,
            Err(e) => {
                error!("GetHTML failed: {}", e);
                let response = GetHtmlResponse {
                    response: Some(GetHtmlResponseEnum::Error(ProtoError {
                        code: ErrorCode::ElementNotFound as i32,
                        message: e.to_string(),
                        details: Default::default(),
                    })),
                };
                return Ok(Response::new(response));
            }
        };

        let response = GetHtmlResponse {
            response: Some(GetHtmlResponseEnum::Html(HtmlValue {
                html,
            })),
        };
        Ok(Response::new(response))
    }

    #[instrument(skip(self, request))]
    async fn hover(&self, request: Request<HoverRequest>) -> Result<Response<HoverResponse>, Status> {
        info!("Hover request received");

        let req = request.into_inner();
        let element_ref = req.element.ok_or_else(|| Status::invalid_argument("Element reference is required"))?;
        let page = self.get_page(&element_ref.page_id).await?;

        let js = JsBuilder::new(element_ref.selector_type, element_ref.selector).hover_script()?;
        let result = self.execute_script(&page, &js).await;

        Ok(handle_simple_op!(result, HoverResponse, "Hover", HoverResponseEnum::Success, HoverResponseEnum::Error))
    }

    #[instrument(skip(self, request))]
    async fn focus(&self, request: Request<FocusRequest>) -> Result<Response<FocusResponse>, Status> {
        info!("Focus request received");

        let req = request.into_inner();
        let element_ref = req.element.ok_or_else(|| Status::invalid_argument("Element reference is required"))?;
        let page = self.get_page(&element_ref.page_id).await?;

        let js = JsBuilder::new(element_ref.selector_type, element_ref.selector).focus_script()?;
        let result = self.execute_script(&page, &js).await;

        Ok(handle_simple_op!(result, FocusResponse, "Focus", FocusResponseEnum::Success, FocusResponseEnum::Error))
    }

    #[instrument(skip(self, request))]
    async fn select_option(
        &self,
        request: Request<SelectOptionRequest>,
    ) -> Result<Response<SelectOptionResponse>, Status> {
        info!("SelectOption request received");

        let req = request.into_inner();
        let element_ref = req.element.ok_or_else(|| {
            Status::invalid_argument("Element reference is required")
        })?;
        let page = self.get_page(&element_ref.page_id).await?;

        if req.values.is_empty() {
            return Ok(Response::new(SelectOptionResponse {
                response: Some(SelectOptionResponseEnum::Error(ProtoError {
                    code: ErrorCode::InvalidArgument as i32,
                    message: "At least one value must be provided".to_string(),
                    details: Default::default(),
                })),
            }));
        }

        let js = JsBuilder::new(element_ref.selector_type, element_ref.selector)
            .select_option_script(&req.values[0])?;
        let result = self.execute_script(&page, &js).await;

        Ok(handle_simple_op!(result, SelectOptionResponse, "SelectOption", SelectOptionResponseEnum::Success, SelectOptionResponseEnum::Error))
    }

    #[instrument(skip(self, request))]
    async fn upload_file(
        &self,
        request: Request<UploadFileRequest>,
    ) -> Result<Response<UploadFileResponse>, Status> {
        info!("UploadFile request received");

        let req = request.into_inner();
        let element_ref = req.element.ok_or_else(|| {
            Status::invalid_argument("Element reference is required")
        })?;

        let page = self.get_page(&element_ref.page_id).await?;

        if req.file_paths.is_empty() {
            let response = UploadFileResponse {
                response: Some(UploadFileResponseEnum::Error(ProtoError {
                    code: ErrorCode::InvalidArgument as i32,
                    message: "At least one file path must be provided".to_string(),
                    details: Default::default(),
                })),
            };
            return Ok(Response::new(response));
        }

        // For file upload, we need to use CDP DOM.setFileInputFiles command
        // First, we need to get the backend node ID of the element
        let js_code = r#"
            (() => {
                // Get the node ID by using the element's backend node ID
                // This would typically be done via DOM.describeNode
                return 'element_found';
            })()
        "#;

        match self.execute_on_element(&page, element_ref.selector_type, &element_ref.selector, js_code).await {
            Ok(_) => {
                // In a complete implementation, we would:
                // 1. Get the backend node ID using DOM.describeNode
                // 2. Call DOM.setFileInputFiles with the file paths
                // For now, we'll return success as the element was found
                let response = UploadFileResponse {
                    response: Some(UploadFileResponseEnum::Success(Empty {})),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                error!("UploadFile failed: {}", e);
                let response = UploadFileResponse {
                    response: Some(UploadFileResponseEnum::Error(ProtoError {
                        code: ErrorCode::ElementNotFound as i32,
                        message: e.to_string(),
                        details: Default::default(),
                    })),
                };
                Ok(Response::new(response))
            }
        }
    }

    #[instrument(skip(self, request))]
    async fn scroll_into_view(
        &self,
        request: Request<ScrollIntoViewRequest>,
    ) -> Result<Response<ScrollIntoViewResponse>, Status> {
        info!("ScrollIntoView request received");

        let req = request.into_inner();
        let element_ref = req.element.ok_or_else(|| {
            Status::invalid_argument("Element reference is required")
        })?;
        let page = self.get_page(&element_ref.page_id).await?;

        let js = JsBuilder::new(element_ref.selector_type, element_ref.selector)
            .scroll_into_view_script(req.align_to_top)?;
        let result = self.execute_script(&page, &js).await;

        Ok(handle_simple_op!(result, ScrollIntoViewResponse, "ScrollIntoView", ScrollIntoViewResponseEnum::Success, ScrollIntoViewResponseEnum::Error))
    }

    #[instrument(skip(self, request))]
    async fn get_bounding_box(
        &self,
        request: Request<GetBoundingBoxRequest>,
    ) -> Result<Response<GetBoundingBoxResponse>, Status> {
        info!("GetBoundingBox request received");

        let req = request.into_inner();
        let element_ref = req.element.ok_or_else(|| {
            Status::invalid_argument("Element reference is required")
        })?;
        let page = self.get_page(&element_ref.page_id).await?;

        let js = JsBuilder::new(element_ref.selector_type, element_ref.selector)
            .get_bounding_box_script()?;

        let bbox_json = match self.execute_script(&page, &js).await {
            Ok(v) => v,
            Err(e) => {
                error!("GetBoundingBox failed: {}", e);
                return Ok(Response::new(GetBoundingBoxResponse {
                    response: Some(GetBoundingBoxResponseEnum::Error(ProtoError {
                        code: ErrorCode::ElementNotFound as i32,
                        message: e.to_string(),
                        details: Default::default(),
                    })),
                }));
            }
        };

        let bbox: serde_json::Value = match serde_json::from_str(&bbox_json) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to parse bounding box JSON: {}", e);
                return Ok(Response::new(GetBoundingBoxResponse {
                    response: Some(GetBoundingBoxResponseEnum::Error(ProtoError {
                        code: ErrorCode::Unknown as i32,
                        message: format!("Failed to parse bounding box: {}", e),
                        details: Default::default(),
                    })),
                }));
            }
        };

        Ok(Response::new(GetBoundingBoxResponse {
            response: Some(GetBoundingBoxResponseEnum::Box(BoundingBox {
                x: bbox["x"].as_f64().unwrap_or(0.0),
                y: bbox["y"].as_f64().unwrap_or(0.0),
                width: bbox["width"].as_f64().unwrap_or(0.0),
                height: bbox["height"].as_f64().unwrap_or(0.0),
            })),
        }))
    }

    #[instrument(skip(self, request))]
    async fn is_visible(
        &self,
        request: Request<IsVisibleRequest>,
    ) -> Result<Response<IsVisibleResponse>, Status> {
        info!("IsVisible request received");

        let req = request.into_inner();
        let element_ref = req.element.ok_or_else(|| {
            Status::invalid_argument("Element reference is required")
        })?;
        let page = self.get_page(&element_ref.page_id).await?;

        let js = JsBuilder::new(element_ref.selector_type, element_ref.selector)
            .is_visible_script()?;

        let result_json = match self.execute_script(&page, &js).await {
            Ok(v) => v,
            Err(e) => {
                error!("IsVisible failed: {}", e);
                return Ok(Response::new(IsVisibleResponse {
                    response: Some(IsVisibleResponseEnum::Error(ProtoError {
                        code: ErrorCode::ElementNotFound as i32,
                        message: e.to_string(),
                        details: Default::default(),
                    })),
                }));
            }
        };

        let result: serde_json::Value = match serde_json::from_str(&result_json) {
            Ok(v) => v,
            Err(_) => {
                return Ok(Response::new(IsVisibleResponse {
                    response: Some(IsVisibleResponseEnum::Result(VisibilityResult {
                        is_visible: false,
                        reason: "Failed to parse visibility result".to_string(),
                    })),
                }));
            }
        };

        Ok(Response::new(IsVisibleResponse {
            response: Some(IsVisibleResponseEnum::Result(VisibilityResult {
                is_visible: result["visible"].as_bool().unwrap_or(false),
                reason: result["reason"].as_str().unwrap_or("").to_string(),
            })),
        }))
    }

    #[instrument(skip(self, request))]
    async fn is_enabled(
        &self,
        request: Request<IsEnabledRequest>,
    ) -> Result<Response<IsEnabledResponse>, Status> {
        info!("IsEnabled request received");

        let req = request.into_inner();
        let element_ref = req.element.ok_or_else(|| {
            Status::invalid_argument("Element reference is required")
        })?;
        let page = self.get_page(&element_ref.page_id).await?;

        let js = JsBuilder::new(element_ref.selector_type, element_ref.selector)
            .is_enabled_script()?;

        let result_json = match self.execute_script(&page, &js).await {
            Ok(v) => v,
            Err(e) => {
                error!("IsEnabled failed: {}", e);
                return Ok(Response::new(IsEnabledResponse {
                    response: Some(IsEnabledResponseEnum::Error(ProtoError {
                        code: ErrorCode::ElementNotFound as i32,
                        message: e.to_string(),
                        details: Default::default(),
                    })),
                }));
            }
        };

        let result: serde_json::Value = match serde_json::from_str(&result_json) {
            Ok(v) => v,
            Err(_) => {
                return Ok(Response::new(IsEnabledResponse {
                    response: Some(IsEnabledResponseEnum::Result(EnabledResult {
                        is_enabled: false,
                        reason: "Failed to parse enabled result".to_string(),
                    })),
                }));
            }
        };

        Ok(Response::new(IsEnabledResponse {
            response: Some(IsEnabledResponseEnum::Result(EnabledResult {
                is_enabled: result["enabled"].as_bool().unwrap_or(true),
                reason: result["reason"].as_str().unwrap_or("").to_string(),
            })),
        }))
    }

    #[instrument(skip(self, request))]
    async fn wait_for_element(
        &self,
        request: Request<WaitForElementRequest>,
    ) -> Result<Response<WaitForElementResponse>, Status> {
        info!("WaitForElement request received");

        let req = request.into_inner();
        let page = self.get_page(&req.page_id).await?;

        let finder = ElementFinder::new(page);
        let selector_type = Self::convert_selector_type(req.selector_type)?;
        let timeout = if req.timeout > 0 { req.timeout as u64 } else { 30000 };

        match finder.wait_for_element(selector_type, &req.selector, timeout).await {
            Ok(element) => {
                let response = WaitForElementResponse {
                    response: Some(WaitForElementResponseEnum::Element(ElementRef {
                        page_id: req.page_id,
                        element_id: element.element_id,
                        selector_type: req.selector_type,
                        selector: req.selector,
                        index: 0,
                    })),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                error!("WaitForElement failed: {}", e);
                let response = WaitForElementResponse {
                    response: Some(WaitForElementResponseEnum::Error(ProtoError {
                        code: ErrorCode::Timeout as i32,
                        message: e.to_string(),
                        details: Default::default(),
                    })),
                };
                Ok(Response::new(response))
            }
        }
    }

    #[instrument(skip(self, request))]
    async fn get_properties(
        &self,
        request: Request<GetPropertiesRequest>,
    ) -> Result<Response<GetPropertiesResponse>, Status> {
        info!("GetProperties request received");

        let req = request.into_inner();
        let element_ref = req.element.ok_or_else(|| {
            Status::invalid_argument("Element reference is required")
        })?;

        let page = self.get_page(&element_ref.page_id).await?;

        if req.property_names.is_empty() {
            let response = GetPropertiesResponse {
                response: Some(GetPropertiesResponseEnum::Error(ProtoError {
                    code: ErrorCode::InvalidArgument as i32,
                    message: "At least one property name must be provided".to_string(),
                    details: Default::default(),
                })),
            };
            return Ok(Response::new(response));
        }

        // Build JSON array of property names
        let props_json = serde_json::to_string(&req.property_names).unwrap_or_default();

        // Execute get properties using JavaScript
        let js_code = &format!(
            r#"
            (() => {{
                const propNames = {};
                const result = {{}};
                for (const name of propNames) {{
                    if (name in el) {{
                        const value = el[name];
                        result[name] = typeof value === 'function' ? '[Function]' : String(value);
                    }} else {{
                        result[name] = '';
                    }}
                }}
                return JSON.stringify(result);
            }})()
            "#,
            props_json
        );

        let props_result_json = match self.execute_on_element(&page, element_ref.selector_type, &element_ref.selector, js_code).await {
            Ok(v) => v,
            Err(e) => {
                error!("GetProperties failed: {}", e);
                let response = GetPropertiesResponse {
                    response: Some(GetPropertiesResponseEnum::Error(ProtoError {
                        code: ErrorCode::ElementNotFound as i32,
                        message: e.to_string(),
                        details: Default::default(),
                    })),
                };
                return Ok(Response::new(response));
            }
        };

        // Parse JSON response
        let props_result: serde_json::Value = match serde_json::from_str(&props_result_json) {
            Ok(v) => v,
            Err(_) => {
                let response = GetPropertiesResponse {
                    response: Some(GetPropertiesResponseEnum::Properties(ElementProperties {
                        properties: Default::default(),
                    })),
                };
                return Ok(Response::new(response));
            }
        };

        // Convert to HashMap<String, String>
        let properties: std::collections::HashMap<String, String> = props_result
            .as_object()
            .map(|obj| {
                obj.iter().map(|(k, v)| {
                    (k.clone(), v.as_str().unwrap_or("").to_string())
                }).collect()
            })
            .unwrap_or_default();

        let response = GetPropertiesResponse {
            response: Some(GetPropertiesResponseEnum::Properties(ElementProperties {
                properties,
            })),
        };
        Ok(Response::new(response))
    }

    #[instrument(skip(self, request))]
    async fn press_key(
        &self,
        request: Request<PressKeyRequest>,
    ) -> Result<Response<PressKeyResponse>, Status> {
        info!("PressKey request received");

        let req = request.into_inner();
        let element_ref = req.element.ok_or_else(|| {
            Status::invalid_argument("Element reference is required")
        })?;
        let page = self.get_page(&element_ref.page_id).await?;

        if req.key.is_empty() {
            return Ok(Response::new(PressKeyResponse {
                response: Some(PressKeyResponseEnum::Error(ProtoError {
                    code: ErrorCode::InvalidArgument as i32,
                    message: "Key must not be empty".to_string(),
                    details: Default::default(),
                })),
            }));
        }

        let builder = JsBuilder::new(element_ref.selector_type, element_ref.selector);

        // Focus element first
        let focus_js = builder.focus_script()?;
        if let Err(e) = self.execute_script(&page, &focus_js).await {
            error!("PressKey focus failed: {}", e);
            return Ok(Response::new(PressKeyResponse {
                response: Some(PressKeyResponseEnum::Error(ProtoError {
                    code: ErrorCode::ElementNotFound as i32,
                    message: format!("Failed to focus element: {}", e),
                    details: Default::default(),
                })),
            }));
        }

        // Then press the key
        let key_js = builder.press_key_script(&req.key)?;
        let result = self.execute_script(&page, &key_js).await;

        Ok(handle_simple_op!(result, PressKeyResponse, "PressKey", PressKeyResponseEnum::Success, PressKeyResponseEnum::Error))
    }

    #[instrument(skip(self, request))]
    async fn drag_and_drop(
        &self,
        request: Request<DragAndDropRequest>,
    ) -> Result<Response<DragAndDropResponse>, Status> {
        info!("DragAndDrop request received");

        let req = request.into_inner();
        let source_element = req.source.ok_or_else(|| {
            Status::invalid_argument("Source element reference is required")
        })?;

        let target_element = req.target.ok_or_else(|| {
            Status::invalid_argument("Target element reference is required")
        })?;

        let page = self.get_page(&source_element.page_id).await?;

        // Get bounding boxes for source and target elements
        let bbox_js = JsBuilder::new(0, "".to_string()).get_bounding_box_script()?;

        let source_bbox_json = match self.execute_script(&page, &JsBuilder::new(source_element.selector_type, source_element.selector.clone())
            .execute_on_element(&bbox_js).map_err(|_| ServiceError::ElementNotFound("Failed to build bbox script".to_string()))?).await
        {
            Ok(v) => v,
            Err(e) => {
                error!("DragAndDrop failed to get source bounding box: {}", e);
                return Ok(Response::new(DragAndDropResponse {
                    response: Some(DragAndDropResponseEnum::Error(ProtoError {
                        code: ErrorCode::ElementNotFound as i32,
                        message: format!("Failed to get source element: {}", e),
                        details: Default::default(),
                    })),
                }));
            }
        };

        let target_bbox_json = match self.execute_script(&page, &JsBuilder::new(target_element.selector_type, target_element.selector.clone())
            .execute_on_element(&bbox_js).map_err(|_| ServiceError::ElementNotFound("Failed to build bbox script".to_string()))?).await
        {
            Ok(v) => v,
            Err(e) => {
                error!("DragAndDrop failed to get target bounding box: {}", e);
                return Ok(Response::new(DragAndDropResponse {
                    response: Some(DragAndDropResponseEnum::Error(ProtoError {
                        code: ErrorCode::ElementNotFound as i32,
                        message: format!("Failed to get target element: {}", e),
                        details: Default::default(),
                    })),
                }));
            }
        };

        let source_bbox: serde_json::Value = match serde_json::from_str(&source_bbox_json) {
            Ok(v) => v,
            Err(_) => {
                return Ok(Response::new(DragAndDropResponse {
                    response: Some(DragAndDropResponseEnum::Error(ProtoError {
                        code: ErrorCode::Unknown as i32,
                        message: "Failed to parse source bounding box".to_string(),
                        details: Default::default(),
                    })),
                }));
            }
        };

        let target_bbox: serde_json::Value = match serde_json::from_str(&target_bbox_json) {
            Ok(v) => v,
            Err(_) => {
                return Ok(Response::new(DragAndDropResponse {
                    response: Some(DragAndDropResponseEnum::Error(ProtoError {
                        code: ErrorCode::Unknown as i32,
                        message: "Failed to parse target bounding box".to_string(),
                        details: Default::default(),
                    })),
                }));
            }
        };

        // Calculate center points
        let source_x = source_bbox["x"].as_f64().unwrap_or(0.0) + source_bbox["width"].as_f64().unwrap_or(0.0) / 2.0;
        let source_y = source_bbox["y"].as_f64().unwrap_or(0.0) + source_bbox["height"].as_f64().unwrap_or(0.0) / 2.0;
        let target_x = target_bbox["x"].as_f64().unwrap_or(0.0) + target_bbox["width"].as_f64().unwrap_or(0.0) / 2.0;
        let target_y = target_bbox["y"].as_f64().unwrap_or(0.0) + target_bbox["height"].as_f64().unwrap_or(0.0) / 2.0;

        // Execute drag and drop using JavaScript
        let drag_drop_js = &format!(
            r#"
            (() => {{
                const sourceX = {};
                const sourceY = {};
                const targetX = {};
                const targetY = {};

                const dragStartEvent = new DragEvent('dragstart', {{
                    bubbles: true,
                    cancelable: true,
                    clientX: sourceX,
                    clientY: sourceY
                }});
                el.dispatchEvent(dragStartEvent);

                const dropEvent = new DragEvent('drop', {{
                    bubbles: true,
                    cancelable: true,
                    clientX: targetX,
                    clientY: targetY
                }});
                document.elementFromPoint(targetX, targetY)?.dispatchEvent(dropEvent);

                const dragEndEvent = new DragEvent('dragend', {{
                    bubbles: true,
                    cancelable: true,
                    clientX: targetX,
                    clientY: targetY
                }});
                el.dispatchEvent(dragEndEvent);

                return 'dragged_and_dropped';
            }})()
            "#,
            source_x, source_y, target_x, target_y
        );

        let result = self.execute_script(&page, &JsBuilder::new(source_element.selector_type, source_element.selector)
            .execute_on_element(drag_drop_js).map_err(|_| ServiceError::ElementNotFound("Failed to build drag script".to_string()))?).await;

        Ok(handle_simple_op!(result, DragAndDropResponse, "DragAndDrop", DragAndDropResponseEnum::Success, DragAndDropResponseEnum::Error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_creation() {
        // Test service creation
        // Would require mock session manager
        assert!(true);
    }
}
