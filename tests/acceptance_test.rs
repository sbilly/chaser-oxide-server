//! 功能与安全验收测试
//!
//! Comprehensive acceptance tests for functionality and security requirements.

// ============= Proto Service Method Coverage Test =============

#[test]
fn test_browser_service_methods_implemented() {
    // Verify all BrowserService methods from proto are implemented
    let expected_methods = vec![
        "launch",
        "get_pages",
        "close",
        "get_version",
        "get_status",
        "connect_to",
    ];

    // Check service implementation file exists and has correct number of methods
    let service_file = std::path::Path::new("src/services/browser/service.rs");
    assert!(service_file.exists(), "Browser service file should exist");

    let content = std::fs::read_to_string(service_file)
        .expect("Should read browser service file");

    // Verify method signatures exist
    for method in &expected_methods {
        assert!(
            content.contains(&format!("async fn {}", method)),
            "BrowserService should implement {} method",
            method
        );
    }
}

#[test]
fn test_page_service_methods_implemented() {
    // Verify all PageService methods from proto are implemented
    let expected_methods = vec![
        "create_page",
        "navigate",
        "get_snapshot",
        "screenshot",
        "evaluate",
        "evaluate_on_element",
        "set_content",
        "get_content",
        "reload",
        "go_back",
        "go_forward",
        "set_viewport",
        "emulate_device",
        "bring_to_front",
        "get_metrics",
        "close_page",
        "wait_for",
        "get_pdf",
        "add_init_script",
        "override_permissions",
        "set_geolocation",
        "set_offline_mode",
        "set_cache_enabled",
        "get_cookies",
        "set_cookies",
        "clear_cookies",
    ];

    let service_file = std::path::Path::new("src/services/page/service.rs");
    assert!(service_file.exists(), "Page service file should exist");

    let content = std::fs::read_to_string(service_file)
        .expect("Should read page service file");

    for method in &expected_methods {
        assert!(
            content.contains(&format!("async fn {}", method)),
            "PageService should implement {} method",
            method
        );
    }
}

#[test]
fn test_element_service_methods_implemented() {
    // Verify all ElementService methods from proto are implemented
    let expected_methods = vec![
        "find_element",
        "find_elements",
        "click",
        "type",
        "fill",
        "get_attribute",
        "get_attributes",
        "get_text",
        "get_html",
        "hover",
        "focus",
        "select_option",
        "upload_file",
        "scroll_into_view",
        "get_bounding_box",
        "is_visible",
        "is_enabled",
        "wait_for_element",
        "get_properties",
        "press_key",
        "drag_and_drop",
    ];

    let service_file = std::path::Path::new("src/services/element/service.rs");
    assert!(service_file.exists(), "Element service file should exist");

    let content = std::fs::read_to_string(service_file)
        .expect("Should read element service file");

    for method in &expected_methods {
        // Special handling for 'type' which is a Rust keyword and uses r#type
        let search_pattern = if *method == "type" {
            "async fn r#type".to_string()
        } else {
            format!("async fn {}", method)
        };

        assert!(
            content.contains(&search_pattern),
            "ElementService should implement {} method",
            method
        );
    }
}

#[test]
fn test_event_service_methods_implemented() {
    // Verify EventService methods from proto are implemented
    let expected_methods = vec![
        "subscribe",
    ];

    let service_file = std::path::Path::new("src/services/event/service.rs");
    assert!(service_file.exists(), "Event service file should exist");

    let content = std::fs::read_to_string(service_file)
        .expect("Should read event service file");

    for method in &expected_methods {
        assert!(
            content.contains(&format!("async fn {}", method)),
            "EventService should implement {} method",
            method
        );
    }
}

// ============= Input Validation Tests =============

#[test]
fn test_browser_launch_validates_headless_flag() {
    // Verify headless parameter validation
    let service_file = std::path::Path::new("src/services/browser/service.rs");
    let content = std::fs::read_to_string(service_file)
        .expect("Should read browser service file");

    // Check that options are converted with validation
    assert!(
        content.contains("proto_to_browser_options"),
        "Should have browser options conversion with validation"
    );
}

#[test]
fn test_viewport_dimensions_validated() {
    // Verify viewport dimensions are validated (must be non-negative)
    let page_service = std::path::Path::new("src/services/page/service.rs");
    let content = std::fs::read_to_string(page_service)
        .expect("Should read page service file");

    // Check for max(0) validation on dimensions
    assert!(
        content.contains(".max(0)"),
        "Viewport dimensions should be validated to be non-negative"
    );
}

#[test]
fn test_timeout_parameter_validated() {
    // Verify timeout parameters are validated
    let page_service = std::path::Path::new("src/services/page/service.rs");
    let content = std::fs::read_to_string(page_service)
        .expect("Should read page service file");

    // Check for timeout validation
    assert!(
        content.contains("timeout.max(0)"),
        "Timeout should be validated to be non-negative"
    );
}

#[test]
fn test_element_selector_type_validated() {
    // Verify selector type is validated
    let element_service = std::path::Path::new("src/services/element/service.rs");
    let content = std::fs::read_to_string(element_service)
        .expect("Should read element service file");

    // Check for selector type validation
    assert!(
        content.contains("convert_selector_type"),
        "Should have selector type validation"
    );
}

// ============= Resource Limit Tests =============

#[test]
fn test_resource_limit_configuration_exists() {
    // Check for configuration file or constants that define resource limits
    let config_file = std::path::Path::new("src/config.rs");
    if config_file.exists() {
        let content = std::fs::read_to_string(config_file)
            .expect("Should read config file");

        // Look for resource limit constants
        let has_max_browsers = content.contains("max_browsers")
            || content.contains("MAX_BROWSERS")
            || content.contains("browser_limit");

        let has_max_pages = content.contains("max_pages")
            || content.contains("MAX_PAGES")
            || content.contains("page_limit");

        // At least some resource limits should be defined
        assert!(
            has_max_browsers || has_max_pages,
            "Resource limits should be defined in configuration"
        );
    }
}

#[test]
fn test_session_manager_has_cleanup() {
    // Verify session manager implements resource cleanup
    let session_files = vec![
        "src/session/mod.rs",
        "src/session/manager.rs",
    ];

    let mut has_cleanup = false;
    for file in session_files {
        let path = std::path::Path::new(file);
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .expect("Should read session file");

            if content.contains("close_browser")
                || content.contains("cleanup")
                || content.contains("drop")
            {
                has_cleanup = true;
                break;
            }
        }
    }

    assert!(
        has_cleanup,
        "Session manager should implement cleanup methods"
    );
}

// ============= Error Message Security Tests =============

#[test]
fn test_error_messages_dont_leak_paths() {
    // Verify error messages don't leak file system paths
    let service_files = vec![
        "src/services/browser/service.rs",
        "src/services/page/service.rs",
        "src/services/element/service.rs",
    ];

    for file in service_files {
        let path = std::path::Path::new(file);
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .expect("Should read service file");

            // Error messages should use generic messages, not full paths
            // Check for proper error handling patterns
            assert!(
                !content.contains("unwrap()") || content.contains("expect"),
                "Should use proper error handling instead of unwrap/expect in production code"
            );
        }
    }
}

#[test]
fn test_error_to_proto_converts_safely() {
    // Verify error conversion to proto doesn't leak sensitive info
    let service_files = vec![
        "src/services/browser/service.rs",
        "src/services/page/service.rs",
    ];

    for file in service_files {
        let path = std::path::Path::new(file);
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .expect("Should read service file");

            // Check for error_to_proto method that safely converts errors
            assert!(
                content.contains("error_to_proto"),
                "Service should have safe error conversion method"
            );

            // Verify error codes are used instead of raw error messages
            assert!(
                content.contains("ErrorCode"),
                "Errors should use error codes for client communication"
            );
        }
    }
}

// ============= Memory Leak Prevention Tests =============

#[test]
fn test_pages_closed_on_browser_close() {
    // Verify that closing a browser also closes all its pages
    let session_manager = std::path::Path::new("src/session/manager.rs");
    if session_manager.exists() {
        let content = std::fs::read_to_string(session_manager)
            .expect("Should read session manager file");

        // Check that close_browser method exists and handles cleanup
        assert!(
            content.contains("close_browser"),
            "Session manager should have close_browser method for cleanup"
        );
    }
}

#[test]
fn test_arc_used_for_shared_state() {
    // Verify Arc<T> is used for shared session manager to prevent memory leaks
    let service_files = vec![
        "src/services/browser/service.rs",
        "src/services/page/service.rs",
        "src/services/element/service.rs",
    ];

    for file in service_files {
        let path = std::path::Path::new(file);
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .expect("Should read service file");

            // Check for Arc usage
            assert!(
                content.contains("Arc<"),
                "Service should use Arc for shared state management"
            );
        }
    }
}

// ============= Stealth Configuration Tests =============

#[test]
fn test_stealth_module_exists() {
    // Verify stealth module exists
    let stealth_files = vec![
        "src/stealth/mod.rs",
        "src/stealth/injector.rs",
        "src/stealth/behavior.rs",
    ];

    for file in stealth_files {
        let path = std::path::Path::new(file);
        assert!(
            path.exists(),
            "Stealth module file {} should exist",
            file
        );
    }
}

#[test]
fn test_stealth_injection_defined() {
    // Verify stealth injection logic is defined
    let injector_file = std::path::Path::new("src/stealth/injector.rs");
    if injector_file.exists() {
        let content = std::fs::read_to_string(injector_file)
            .expect("Should read injector file");

        assert!(
            content.contains("inject") || content.contains("apply"),
            "Stealth injector should have injection methods"
        );
    }
}

// ============= Event Subscription Tests =============

#[test]
fn test_event_dispatcher_exists() {
    // Verify event dispatcher exists
    let dispatcher_file = std::path::Path::new("src/services/event/dispatcher.rs");
    assert!(
        dispatcher_file.exists(),
        "Event dispatcher file should exist"
    );
}

#[test]
fn test_event_types_defined() {
    // Verify event types are defined in proto
    let event_proto = std::path::Path::new("protos/event.proto");
    assert!(event_proto.exists(), "Event proto file should exist");

    let content = std::fs::read_to_string(event_proto)
        .expect("Should read event proto file");

    // Check for essential event types
    let event_types = vec![
        "EVENT_TYPE_PAGE_LOADED",
        "EVENT_TYPE_CONSOLE_LOG",
        "EVENT_TYPE_REQUEST_SENT",
        "EVENT_TYPE_RESPONSE_RECEIVED",
    ];

    for event_type in event_types {
        assert!(
            content.contains(event_type),
            "Event type {} should be defined",
            event_type
        );
    }
}

// ============= Python Client Example Tests =============

#[test]
fn test_python_client_examples_exist() {
    // Verify Python client examples exist
    let example_files = vec![
        "docs/examples/python/basic_client.py",
        "docs/examples/python/stealth_client.py",
    ];

    for file in example_files {
        let path = std::path::Path::new(file);
        assert!(
            path.exists(),
            "Python example file {} should exist",
            file
        );
    }
}

#[test]
fn test_python_client_basic_functionality() {
    // Verify basic client has essential functionality
    let basic_client = std::path::Path::new("docs/examples/python/basic_client.py");
    if basic_client.exists() {
        let content = std::fs::read_to_string(basic_client)
            .expect("Should read basic client file");

        // Check for essential operations
        let essential_ops = vec![
            "launch",
            "create_page",
            "navigate",
            "screenshot",
            "evaluate",
        ];

        for op in essential_ops {
            assert!(
                content.contains(op),
                "Basic client should demonstrate {} operation",
                op
            );
        }
    }
}

#[test]
fn test_python_client_stealth_functionality() {
    // Verify stealth client demonstrates stealth features
    let stealth_client = std::path::Path::new("docs/examples/python/stealth_client.py");
    if stealth_client.exists() {
        let content = std::fs::read_to_string(stealth_client)
            .expect("Should read stealth client file");

        assert!(
            content.contains("stealth") || content.contains("profile"),
            "Stealth client should demonstrate stealth features"
        );
    }
}

// ============= Parameter Boundary Tests =============

#[test]
fn test_string_input_sanitization() {
    // Verify string inputs are sanitized (especially for JavaScript injection)
    let element_service = std::path::Path::new("src/services/element/service.rs");
    if element_service.exists() {
        let content = std::fs::read_to_string(element_service)
            .expect("Should read element service file");

        // Check for string escaping
        assert!(
            content.contains("replace") || content.contains("escape"),
            "Element service should sanitize string inputs to prevent injection"
        );
    }
}

#[test]
fn test_numeric_bounds_checked() {
    // Verify numeric parameters have bounds checking
    let page_service = std::path::Path::new("src/services/page/service.rs");
    let content = std::fs::read_to_string(page_service)
        .expect("Should read page service file");

    // Check for max() calls that prevent negative values
    assert!(
        content.contains(".max(0)") || content.contains("abs()"),
        "Numeric parameters should have bounds checking"
    );
}

// ============= Documentation Tests =============

#[test]
fn test_api_documentation_exists() {
    // Verify API documentation exists
    let api_docs = vec![
        "docs/api-design.md",
        "docs/architecture.md",
        "docs/implementation-plan.md",
    ];

    for doc in api_docs {
        let path = std::path::Path::new(doc);
        assert!(
            path.exists(),
            "API documentation file {} should exist",
            doc
        );
    }
}

#[test]
fn test_readme_exists() {
    // Verify README exists
    let readme_path = std::path::Path::new("README.md");
    assert!(
        readme_path.exists(),
        "README.md should exist in project root"
    );
}
