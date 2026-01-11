//! 类型转换函数
//!
//! 此模块提供 proto 类型与内部类型之间的双向转换。

use crate::session::{
    NavigationOptions, ScreenshotOptions, LoadState, ScreenshotFormat, ClipRegion,
    EvaluationResult as SessionEvaluationResult,
};
use crate::services::traits::EvaluationResult;
use crate::chaser_oxide::v1::{
    NavigationOptions as ProtoNavigationOptions,
    navigation_options,
    ScreenshotOptions as ProtoScreenshotOptions,
    screenshot_options,
    EvaluationResult as ProtoEvaluationResult,
    evaluation_result,
    PageInfo as ProtoPageInfo,
    NavigationResult as ProtoNavigationResult,
    ScreenshotResult,
};

// ============================================================================
// Proto -> Internal 转换
// ============================================================================

/// 将 proto NavigationOptions 转换为内部 NavigationOptions
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

/// 将 proto ScreenshotOptions 转换为内部 ScreenshotOptions
pub fn proto_to_screenshot_options(opts: ProtoScreenshotOptions) -> ScreenshotOptions {
    let format = match opts.format() {
        screenshot_options::Format::Unspecified => ScreenshotFormat::Png,
        screenshot_options::Format::Png => ScreenshotFormat::Png,
        screenshot_options::Format::Jpeg => ScreenshotFormat::Jpeg,
        screenshot_options::Format::Webp => ScreenshotFormat::WebP,
    };

    // 从 proto Rectangle 转换为内部 ClipRegion
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

// ============================================================================
// Internal -> Proto 转换
// ============================================================================

/// 将内部 PageInfo 转换为 proto PageInfo
#[allow(dead_code)]
pub fn page_info_to_proto(info: crate::services::traits::PageInfo) -> ProtoPageInfo {
    ProtoPageInfo {
        page_id: info.page_id,
        browser_id: String::new(),
        url: info.url,
        title: info.title,
        is_loaded: true,
        created_at: chrono::Utc::now().timestamp(),
    }
}

/// 将内部 EvaluationResult 转换为 proto EvaluationResult
pub fn evaluation_result_to_proto(result: EvaluationResult) -> ProtoEvaluationResult {
    tracing::debug!("evaluation_result_to_proto: received {:?}", result);

    let proto_result = match result {
        EvaluationResult::String(s) => {
            tracing::debug!("evaluation_result_to_proto: String variant with value='{}'", s);
            ProtoEvaluationResult {
                response: Some(evaluation_result::Response::StringValue(s)),
                r#type: "string".to_string(),
                class_name: String::new(),
            }
        }
        EvaluationResult::Number(n) => {
            tracing::debug!("evaluation_result_to_proto: Number variant with value={}", n);
            ProtoEvaluationResult {
                response: Some(evaluation_result::Response::DoubleValue(n)),
                r#type: "number".to_string(),
                class_name: String::new(),
            }
        }
        EvaluationResult::Bool(b) => {
            tracing::debug!("evaluation_result_to_proto: Bool variant with value={}", b);
            ProtoEvaluationResult {
                response: Some(evaluation_result::Response::BoolValue(b)),
                r#type: "boolean".to_string(),
                class_name: String::new(),
            }
        }
        EvaluationResult::Null => {
            tracing::debug!("evaluation_result_to_proto: Null variant");
            ProtoEvaluationResult {
                response: Some(evaluation_result::Response::NullValue(crate::chaser_oxide::v1::Value {})),
                r#type: "null".to_string(),
                class_name: String::new(),
            }
        }
        EvaluationResult::Object(v) => {
            tracing::debug!("evaluation_result_to_proto: Object variant with value={:?}", v);
            // 将 JSON 对象转换为字符串表示
            ProtoEvaluationResult {
                response: Some(evaluation_result::Response::StringValue(v.to_string())),
                r#type: "object".to_string(),
                class_name: String::new(),
            }
        }
    };

    tracing::debug!(
        "evaluation_result_to_proto: returning type='{}', response={:?}",
        proto_result.r#type,
        proto_result.response
    );

    proto_result
}

/// 将会话层的 EvaluationResult 转换为服务层的 EvaluationResult
pub fn evaluation_result_from_session(result: SessionEvaluationResult) -> EvaluationResult {
    match result {
        SessionEvaluationResult::String(s) => EvaluationResult::String(s),
        SessionEvaluationResult::Number(n) => EvaluationResult::Number(n),
        SessionEvaluationResult::Bool(b) => EvaluationResult::Bool(b),
        SessionEvaluationResult::Null => EvaluationResult::Null,
        SessionEvaluationResult::Object(v) => EvaluationResult::Object(v),
    }
}

/// 将截图数据转换为 ScreenshotResult
pub fn screenshot_to_proto(data: Vec<u8>, format: ScreenshotFormat) -> ScreenshotResult {
    ScreenshotResult {
        data,
        format: match format {
            ScreenshotFormat::Png => "png".to_string(),
            ScreenshotFormat::Jpeg => "jpeg".to_string(),
            ScreenshotFormat::WebP => "webp".to_string(),
        },
        width: 1920,  // TODO: 从实际截图获取
        height: 1080, // TODO: 从实际截图获取
    }
}

/// 将导航结果转换为 proto NavigationResult
pub fn navigation_result_to_proto(
    url: String,
    status_code: u16,
    is_loaded: bool,
) -> ProtoNavigationResult {
    ProtoNavigationResult {
        url,
        status_code: status_code as i32,
        is_loaded,
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::LoadState;
    use crate::chaser_oxide::v1::navigation_options::LoadState as ProtoLoadState;

    #[test]
    fn test_proto_to_navigation_options_default() {
        let proto_opts = ProtoNavigationOptions {
            timeout: 30000,
            wait_until: ProtoLoadState::NetworkIdle as i32,
            ..Default::default()
        };

        let opts = proto_to_navigation_options(proto_opts);
        assert_eq!(opts.timeout, 30000);
        assert!(matches!(opts.wait_until, LoadState::NetworkIdle));
    }

    #[test]
    fn test_proto_to_navigation_options_zero_timeout() {
        let proto_opts = ProtoNavigationOptions {
            timeout: 0,
            wait_until: ProtoLoadState::Unspecified as i32,
            ..Default::default()
        };

        let opts = proto_to_navigation_options(proto_opts);
        assert_eq!(opts.timeout, 0);
        assert!(matches!(opts.wait_until, LoadState::Load)); // Default
    }

    #[test]
    fn test_proto_to_navigation_options_all_load_states() {
        // Test Load state
        let opts1 = proto_to_navigation_options(ProtoNavigationOptions {
            timeout: 10000,
            wait_until: ProtoLoadState::Load as i32,
            ..Default::default()
        });
        assert!(matches!(opts1.wait_until, LoadState::Load));

        // Test DomContentLoaded state
        let opts2 = proto_to_navigation_options(ProtoNavigationOptions {
            timeout: 10000,
            wait_until: ProtoLoadState::DomContentLoaded as i32,
            ..Default::default()
        });
        assert!(matches!(opts2.wait_until, LoadState::DOMContentLoaded));

        // Test NetworkAlmostIdle state
        let opts3 = proto_to_navigation_options(ProtoNavigationOptions {
            timeout: 10000,
            wait_until: ProtoLoadState::NetworkAlmostIdle as i32,
            ..Default::default()
        });
        assert!(matches!(opts3.wait_until, LoadState::NetworkAlmostIdle));
    }

    #[test]
    fn test_proto_to_screenshot_options_png() {
        let proto_opts = ProtoScreenshotOptions {
            format: screenshot_options::Format::Png as i32,
            quality: 0,
            full_page: false,
            ..Default::default()
        };

        let opts = proto_to_screenshot_options(proto_opts);
        assert!(matches!(opts.format, ScreenshotFormat::Png));
        assert_eq!(opts.quality, None);
        assert_eq!(opts.full_page, false);
    }

    #[test]
    fn test_proto_to_screenshot_options_jpeg() {
        let proto_opts = ProtoScreenshotOptions {
            format: screenshot_options::Format::Jpeg as i32,
            quality: 90,
            full_page: true,
            ..Default::default()
        };

        let opts = proto_to_screenshot_options(proto_opts);
        assert!(matches!(opts.format, ScreenshotFormat::Jpeg));
        assert_eq!(opts.quality, Some(90));
        assert_eq!(opts.full_page, true);
    }

    #[test]
    fn test_proto_to_screenshot_options_webp() {
        let proto_opts = ProtoScreenshotOptions {
            format: screenshot_options::Format::Webp as i32,
            quality: 80,
            full_page: false,
            ..Default::default()
        };

        let opts = proto_to_screenshot_options(proto_opts);
        assert!(matches!(opts.format, ScreenshotFormat::WebP));
        assert_eq!(opts.quality, Some(80));
    }

    #[test]
    fn test_proto_to_screenshot_options_unspecified() {
        let proto_opts = ProtoScreenshotOptions {
            format: screenshot_options::Format::Unspecified as i32,
            quality: 0,
            full_page: false,
            ..Default::default()
        };

        let opts = proto_to_screenshot_options(proto_opts);
        assert!(matches!(opts.format, ScreenshotFormat::Png)); // Default
    }

    #[test]
    fn test_evaluation_result_string_conversion() {
        let result = EvaluationResult::String("test".to_string());
        let proto = evaluation_result_to_proto(result);
        assert!(matches!(
            proto.response,
            Some(evaluation_result::Response::StringValue(_))
        ));
        assert_eq!(proto.r#type, "string");
    }

    #[test]
    fn test_evaluation_result_number_conversion() {
        let result = EvaluationResult::Number(42.5);
        let proto = evaluation_result_to_proto(result);
        assert!(matches!(
            proto.response,
            Some(evaluation_result::Response::DoubleValue(42.5))
        ));
        assert_eq!(proto.r#type, "number");
    }

    #[test]
    fn test_evaluation_result_bool_conversion() {
        let result = EvaluationResult::Bool(true);
        let proto = evaluation_result_to_proto(result);
        assert!(matches!(
            proto.response,
            Some(evaluation_result::Response::BoolValue(true))
        ));
        assert_eq!(proto.r#type, "boolean");
    }

    #[test]
    fn test_evaluation_result_null_conversion() {
        let result = EvaluationResult::Null;
        let proto = evaluation_result_to_proto(result);
        assert!(matches!(
            proto.response,
            Some(evaluation_result::Response::NullValue(_))
        ));
        assert_eq!(proto.r#type, "null");
    }

    #[test]
    fn test_evaluation_result_object_conversion() {
        let json_value = serde_json::json!({"key": "value"});
        let result = EvaluationResult::Object(json_value);
        let proto = evaluation_result_to_proto(result);
        assert!(matches!(
            proto.response,
            Some(evaluation_result::Response::StringValue(_))
        ));
        assert_eq!(proto.r#type, "object");
    }

    #[test]
    fn test_evaluation_result_from_session() {
        let session_result = SessionEvaluationResult::String("test".to_string());
        let service_result = evaluation_result_from_session(session_result);
        assert!(matches!(service_result, EvaluationResult::String(_)));
    }

    #[test]
    fn test_screenshot_to_proto() {
        let data = vec![1, 2, 3, 4, 5];
        let result = screenshot_to_proto(data, ScreenshotFormat::Png);
        assert_eq!(result.format, "png");
        assert_eq!(result.data.len(), 5);
    }

    #[test]
    fn test_navigation_result_to_proto() {
        let result = navigation_result_to_proto("https://example.com".to_string(), 200, true);
        assert_eq!(result.url, "https://example.com");
        assert_eq!(result.status_code, 200);
        assert_eq!(result.is_loaded, true);
    }
}
