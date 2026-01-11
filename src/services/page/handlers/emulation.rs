//! 设备模拟相关的 RPC 方法处理器
//!
//! 包括：emulate_device, set_viewport, set_geolocation, bring_to_front

use std::sync::Arc;
use tonic::{Request, Response, Status};
use crate::session::SessionManager;
use crate::chaser_oxide::v1::{
    emulate_device_response::Response as EmulateDeviceResponseEnum,
    set_viewport_response::Response as SetViewportResponseEnum,
    set_geolocation_response::Response as SetGeolocationResponseEnum,
    bring_to_front_response::Response as BringToFrontResponseEnum,
    EmulateDeviceRequest, EmulateDeviceResponse,
    SetViewportRequest, SetViewportResponse,
    SetGeolocationRequest, SetGeolocationResponse,
    BringToFrontRequest, BringToFrontResponse,
    Empty,
};
use super::super::{response, scripts};

/// 设备预设配置
#[derive(Debug, Clone)]
pub struct DevicePreset {
    pub name: &'static str,
    pub width: u32,
    pub height: u32,
    pub device_scale_factor: f64,
    pub mobile: bool,
    pub user_agent: Option<&'static str>,
}

impl DevicePreset {
    pub const DESKTOP: Self = Self {
        name: "Desktop",
        width: 1920,
        height: 1080,
        device_scale_factor: 1.0,
        mobile: false,
        user_agent: None,
    };

    pub const IPHONE: Self = Self {
        name: "iPhone",
        width: 375,
        height: 667,
        device_scale_factor: 2.0,
        mobile: true,
        user_agent: Some("Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1"),
    };

    pub const IPHONE_PRO: Self = Self {
        name: "iPhone Pro",
        width: 414,
        height: 896,
        device_scale_factor: 3.0,
        mobile: true,
        user_agent: Some("Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1"),
    };

    pub const IPAD: Self = Self {
        name: "iPad",
        width: 768,
        height: 1024,
        device_scale_factor: 2.0,
        mobile: true,
        user_agent: Some("Mozilla/5.0 (iPad; CPU OS 14_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1"),
    };

    pub const IPAD_PRO: Self = Self {
        name: "iPad Pro",
        width: 1024,
        height: 1366,
        device_scale_factor: 2.0,
        mobile: true,
        user_agent: Some("Mozilla/5.0 (iPad; CPU OS 14_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1"),
    };

    pub const ANDROID_PHONE: Self = Self {
        name: "Android Phone",
        width: 360,
        height: 640,
        device_scale_factor: 2.0,
        mobile: true,
        user_agent: Some("Mozilla/5.0 (Linux; Android 10) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/86.0.4240.99 Mobile Safari/537.36"),
    };

    pub const ANDROID_TABLET: Self = Self {
        name: "Android Tablet",
        width: 800,
        height: 1280,
        device_scale_factor: 1.5,
        mobile: true,
        user_agent: Some("Mozilla/5.0 (Linux; Android 10) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/86.0.4240.99 Safari/537.36"),
    };

    /// 根据设备类型获取预设配置
    pub fn from_device_type(device_type: i32) -> Option<Self> {
        match device_type {
            1 => Some(Self::DESKTOP),
            2 => Some(Self::IPHONE),
            3 => Some(Self::IPHONE_PRO),
            4 => Some(Self::IPAD),
            5 => Some(Self::IPAD_PRO),
            6 => Some(Self::ANDROID_PHONE),
            7 => Some(Self::ANDROID_TABLET),
            _ => None,
        }
    }
}

/// 实现 PageService trait 中的设备模拟相关方法
pub struct EmulationHandlers<S> {
    pub session_manager: Arc<S>,
}

impl<S> EmulationHandlers<S>
where
    S: SessionManager + Send + Sync + 'static,
{
    /// 模拟设备
    pub async fn emulate_device(&self, request: Request<EmulateDeviceRequest>) -> Result<Response<EmulateDeviceResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                // 定义设备预设
                let (width, height, device_scale_factor, user_agent) = match req.device {
                    Some(device) => {
                        match device {
                            crate::chaser_oxide::v1::emulate_device_request::Device::DeviceType(device_type) => {
                                if let Some(preset) = DevicePreset::from_device_type(device_type) {
                                    (preset.width, preset.height, preset.device_scale_factor, preset.user_agent)
                                } else {
                                    (DevicePreset::DESKTOP.width, DevicePreset::DESKTOP.height, DevicePreset::DESKTOP.device_scale_factor, None)
                                }
                            }
                            crate::chaser_oxide::v1::emulate_device_request::Device::Viewport(viewport) => {
                                (viewport.width.max(0) as u32, viewport.height.max(0) as u32, viewport.device_scale_factor, None)
                            }
                        }
                    }
                    None => {
                        // 默认为桌面
                        (DevicePreset::DESKTOP.width, DevicePreset::DESKTOP.height, DevicePreset::DESKTOP.device_scale_factor, None)
                    }
                };

                // 设置视口
                match page.set_viewport(width, height, device_scale_factor).await {
                    Ok(_) => {
                        // 如果指定了 user agent，则设置
                        if let Some(ua) = user_agent {
                            let script = format!("({})({})", scripts::SET_USER_AGENT_SCRIPT, serde_json::json!(ua));
                            let _ = page.evaluate(&script, false).await;
                        }

                        Ok(Response::new(EmulateDeviceResponse {
                            response: Some(EmulateDeviceResponseEnum::Success(Empty {})),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }

    /// 设置视口
    pub async fn set_viewport(&self, request: Request<SetViewportRequest>) -> Result<Response<SetViewportResponse>, Status> {
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
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }

    /// 设置地理位置
    pub async fn set_geolocation(&self, request: Request<SetGeolocationRequest>) -> Result<Response<SetGeolocationResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                let geo_script = format!("({})({}, {}, {})",
                    scripts::SET_GEOLOCATION_SCRIPT,
                    req.latitude,
                    req.longitude,
                    req.accuracy
                );

                match page.evaluate(&geo_script, false).await {
                    Ok(_) => {
                        Ok(Response::new(SetGeolocationResponse {
                            response: Some(SetGeolocationResponseEnum::Success(Empty {})),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }

    /// 将页面置于前台
    pub async fn bring_to_front(&self, request: Request<BringToFrontRequest>) -> Result<Response<BringToFrontResponse>, Status> {
        let req = request.into_inner();

        match self.session_manager.get_page(&req.page_id).await {
            Ok(page) => {
                match page.evaluate(scripts::WINDOW_FOCUS_SCRIPT, false).await {
                    Ok(_) => {
                        Ok(Response::new(BringToFrontResponse {
                            response: Some(BringToFrontResponseEnum::Success(Empty {})),
                        }))
                    }
                    Err(e) => Err(response::error_to_status(e)),
                }
            }
            Err(e) => Err(response::error_to_status(e)),
        }
    }
}
