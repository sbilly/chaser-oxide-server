//! Browser service implementation
//!
//! This module provides the gRPC implementation for browser lifecycle management.

use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{error, info};

use crate::session::{SessionManager, BrowserOptions};
use crate::services::traits::{BrowserInfo, BrowserVersion, BrowserStatus, PageInfo};
use crate::Error;

// Import generated proto types
use crate::chaser_oxide::v1::{
    browser_service_server::BrowserService,
    launch_response::Response as LaunchResponseEnum,
    get_pages_response::Response as GetPagesResponseEnum,
    close_response::Response as CloseResponseEnum,
    get_version_response::Response as GetVersionResponseEnum,
    get_status_response::Response as GetStatusResponseEnum,
    connect_response::Response as ConnectResponseEnum,
    LaunchRequest, LaunchResponse,
    GetPagesRequest, GetPagesResponse, GetPagesResult,
    CloseRequest, CloseResponse,
    GetVersionRequest, GetVersionResponse,
    GetStatusRequest, GetStatusResponse,
    ConnectRequest, ConnectResponse,
    BrowserOptions as ProtoBrowserOptions,
    PageInfo as ProtoPageInfo,
    BrowserInfo as ProtoBrowserInfo,
    VersionInfo,
    BrowserStatus as ProtoBrowserStatus,
    Empty,
    Error as ProtoError,
    ErrorCode,
};

/// Browser service implementation
#[derive(Debug, Clone)]
pub struct Service<S> {
    session_manager: Arc<S>,
}

impl<S> Service<S>
where
    S: SessionManager + Send + Sync + 'static,
{
    /// Create a new browser service
    pub fn new(session_manager: Arc<S>) -> Self {
        Self { session_manager }
    }

    /// Convert Error to ProtoError
    fn error_to_proto(error: Error) -> ProtoError {
        let code = match &error {
            Error::BrowserNotFound(_) => ErrorCode::NotFound,
            Error::Configuration(_) => ErrorCode::InvalidArgument,
            Error::Timeout(_) => ErrorCode::Timeout,
            _ => ErrorCode::Internal,
        };

        ProtoError {
            code: code.into(),
            message: error.to_string(),
            details: std::collections::HashMap::new(),
        }
    }

    /// Convert proto BrowserOptions to internal BrowserOptions
    pub fn proto_to_browser_options(opts: ProtoBrowserOptions) -> BrowserOptions {
        BrowserOptions {
            headless: opts.headless,
            window_width: opts.window_width.max(0) as u32,
            window_height: opts.window_height.max(0) as u32,
            user_agent: if opts.user_agent.is_empty() { None } else { Some(opts.user_agent) },
            proxy: if opts.proxy_server.is_empty() { None } else { Some(opts.proxy_server) },
            args: opts.args,
            executable_path: if opts.executable_path.is_empty() { None } else { Some(opts.executable_path) },
            // Read CDP endpoint from environment variable if set
            cdp_endpoint: std::env::var("CHASER_CDP_ENDPOINT").ok(),
        }
    }

    /// Convert internal BrowserInfo to proto
    fn browser_info_to_proto(info: BrowserInfo) -> ProtoBrowserInfo {
        ProtoBrowserInfo {
            browser_id: info.browser_id,
            executable_path: String::new(), // Will be filled by actual implementation
            pid: 0, // Will be filled by actual implementation
            headless: true, // Will be filled by actual implementation
            user_agent: info.user_agent,
            launched_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Convert internal PageInfo to proto
    fn page_info_to_proto(info: PageInfo) -> ProtoPageInfo {
        ProtoPageInfo {
            page_id: info.page_id,
            browser_id: String::new(), // Will be filled by actual implementation
            url: info.url,
            title: info.title,
            is_loaded: true,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Convert internal BrowserVersion to proto
    fn browser_version_to_proto(version: BrowserVersion) -> VersionInfo {
        VersionInfo {
            protocol_version: version.protocol_version,
            product: version.product,
            revision: version.revision,
            user_agent: version.user_agent,
            javascript_version: version.js_version,
        }
    }

    /// Convert internal BrowserStatus to proto
    fn browser_status_to_proto(status: BrowserStatus) -> ProtoBrowserStatus {
        ProtoBrowserStatus {
            browser_id: String::new(), // Will be filled by actual implementation
            is_running: status.is_active,
            page_count: status.page_count as i32,
            uptime_seconds: (status.uptime_ms / 1000) as i64,
            memory_usage_bytes: 0, // Will be filled by actual implementation
            active_pages: vec![],
        }
    }
}

#[tonic::async_trait]
impl<S> BrowserService for Service<S>
where
    S: SessionManager + Send + Sync + 'static,
{
    async fn launch(&self, request: Request<LaunchRequest>) -> Result<Response<LaunchResponse>, Status> {
        let req = request.into_inner();
        let options = Self::proto_to_browser_options(req.options.unwrap_or_default());

        match self.session_manager.create_browser(options).await {
            Ok(browser_id) => {
                info!(browser_id = %browser_id, "Browser launched successfully");

                let browser = self.session_manager.get_browser(&browser_id).await
                    .map_err(|e| Status::from(Error::internal(e.to_string())))?;

                let info = BrowserInfo {
                    browser_id: browser.id().to_string(),
                    user_agent: String::new(), // Will be filled by actual implementation
                    cdp_endpoint: String::new(), // Will be filled by actual implementation
                };

                Ok(Response::new(LaunchResponse {
                    response: Some(LaunchResponseEnum::BrowserInfo(
                        Self::browser_info_to_proto(info)
                    )),
                }))
            }
            Err(e) => {
                error!(
                    error = %e,
                    "Failed to launch browser. Please ensure Chrome is running with --remote-debugging-port=9222"
                );

                Ok(Response::new(LaunchResponse {
                    response: Some(LaunchResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn get_pages(&self, request: Request<GetPagesRequest>) -> Result<Response<GetPagesResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_browser(&req.browser_id).await {
            Ok(browser) => {
                match browser.get_pages().await {
                    Ok(pages) => {
                        let proto_pages: Vec<ProtoPageInfo> = pages.into_iter()
                            .map(|p| PageInfo {
                                page_id: p.id().to_string(),
                                url: String::new(), // Will be filled by actual implementation
                                title: String::new(), // Will be filled by actual implementation
                            })
                            .map(Self::page_info_to_proto)
                            .collect();

                        Ok(Response::new(GetPagesResponse {
                            response: Some(GetPagesResponseEnum::Pages(
                                GetPagesResult { pages: proto_pages }
                            )),
                        }))
                    }
                    Err(e) => {
                        Ok(Response::new(GetPagesResponse {
                            response: Some(GetPagesResponseEnum::Error(
                                Self::error_to_proto(e)
                            )),
                        }))
                    }
                }
            }
            Err(e) => {
                Ok(Response::new(GetPagesResponse {
                    response: Some(GetPagesResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn close(&self, request: Request<CloseRequest>) -> Result<Response<CloseResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.close_browser(&req.browser_id).await {
            Ok(_) => {
                Ok(Response::new(CloseResponse {
                    response: Some(CloseResponseEnum::Success(Empty {})),
                }))
            }
            Err(e) => {
                Ok(Response::new(CloseResponse {
                    response: Some(CloseResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn get_version(&self, _request: Request<GetVersionRequest>) -> Result<Response<GetVersionResponse>, Status> {
        // This is a placeholder - actual implementation would query the browser
        let version = BrowserVersion {
            protocol_version: "1.3".to_string(),
            product: "Chrome/120.0.6099.109".to_string(),
            revision: "d46a2e6e16f6a1b7d0c4e8b5c8f0a1e2d3c4b5a6".to_string(),
            user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            js_version: "12.0.0".to_string(),
        };

        Ok(Response::new(GetVersionResponse {
            response: Some(GetVersionResponseEnum::VersionInfo(
                Self::browser_version_to_proto(version)
            )),
        }))
    }

    async fn get_status(&self, request: Request<GetStatusRequest>) -> Result<Response<GetStatusResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_browser(&req.browser_id).await {
            Ok(_browser) => {
                // This is a placeholder - actual implementation would get real status
                let status = BrowserStatus {
                    is_active: true,
                    page_count: 1,
                    uptime_ms: 60000,
                };

                Ok(Response::new(GetStatusResponse {
                    response: Some(GetStatusResponseEnum::Status(
                        Self::browser_status_to_proto(status)
                    )),
                }))
            }
            Err(e) => {
                Ok(Response::new(GetStatusResponse {
                    response: Some(GetStatusResponseEnum::Error(
                        Self::error_to_proto(e)
                    )),
                }))
            }
        }
    }

    async fn connect_to(&self, _request: Request<ConnectRequest>) -> Result<Response<ConnectResponse>, Status> {
        // This is a placeholder - actual implementation would connect to existing browser
        let error = Error::internal("Connect not implemented yet");
        Ok(Response::new(ConnectResponse {
            response: Some(ConnectResponseEnum::Error(
                Self::error_to_proto(error)
            )),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::mock::MockSessionManager;

    #[tokio::test]
    async fn test_browser_service_creation() {
        let session_manager = Arc::new(MockSessionManager::new());
        let service = Service::new(session_manager);
        assert!(true, "Service created successfully");
    }

    #[tokio::test]
    async fn test_proto_conversion() {
        let proto_opts = ProtoBrowserOptions {
            headless: true,
            window_width: 1920,
            window_height: 1080,
            user_agent: "test-agent".to_string(),
            proxy_server: "http://proxy:8080".to_string(),
            ..Default::default()
        };

        let opts = Service::<MockSessionManager>::proto_to_browser_options(proto_opts);
        assert_eq!(opts.headless, true);
        assert_eq!(opts.window_width, 1920);
        assert_eq!(opts.window_height, 1080);
        assert_eq!(opts.user_agent, Some("test-agent".to_string()));
        assert_eq!(opts.proxy, Some("http://proxy:8080".to_string()));
    }
}
