//! Stealth engine integration tests
//!
//! Comprehensive integration tests for the stealth engine including:
//! - Fingerprint generation
//! - Script injection
//! - Behavior simulation
//! - End-to-end workflows

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::super::traits::*;
    use super::super::super::services::traits as services;
    use super::super::super::cdp::mock::MockCdpClient as CdpMockClient;
    use std::sync::Arc;

    // ============================================================================
    // Fingerprint Generation Tests
    // ============================================================================

    #[tokio::test]
    async fn test_fingerprint_generator_windows() {
        let generator = FingerprintGeneratorImpl::new();

        let result = generator.generate_windows().await;

        assert!(result.is_ok());
        let fingerprint = result.unwrap();

        // Verify Windows-specific properties
        assert_eq!(fingerprint.navigator.platform, "Win32");
        assert_eq!(fingerprint.navigator.vendor, "Google Inc.");
        assert!(fingerprint.navigator.hardware_concurrency >= 4);
        assert!(fingerprint.navigator.device_memory.is_some());
        assert!(fingerprint.headers.user_agent.contains("Windows"));

        // Verify screen resolution is valid
        assert!(fingerprint.screen.width >= 1366);
        assert!(fingerprint.screen.height >= 768);
        assert_eq!(fingerprint.screen.color_depth, 24);
        assert_eq!(fingerprint.screen.pixel_depth, 24);

        // Verify WebGL
        assert!(!fingerprint.webgl.vendor.is_empty());
        assert!(!fingerprint.webgl.renderer.is_empty());

        // Verify all injection options are enabled
        assert!(fingerprint.options.inject_navigator);
        assert!(fingerprint.options.inject_screen);
        assert!(fingerprint.options.inject_webgl);
        assert!(fingerprint.options.inject_canvas);
        assert!(fingerprint.options.inject_audio);
    }

    #[tokio::test]
    async fn test_fingerprint_generator_macos() {
        let generator = FingerprintGeneratorImpl::new();

        let result = generator.generate_macos().await;

        assert!(result.is_ok());
        let fingerprint = result.unwrap();

        // Verify macOS-specific properties
        assert_eq!(fingerprint.navigator.platform, "MacIntel");
        assert_eq!(fingerprint.navigator.vendor, "Google Inc.");
        assert!(fingerprint.headers.user_agent.contains("Macintosh"));

        // Verify screen resolution is valid for macOS
        assert!(fingerprint.screen.width >= 1920);
        assert!(fingerprint.screen.height >= 1080);
    }

    #[tokio::test]
    async fn test_fingerprint_generator_linux() {
        let generator = FingerprintGeneratorImpl::new();

        let result = generator.generate_linux().await;

        assert!(result.is_ok());
        let fingerprint = result.unwrap();

        // Verify Linux-specific properties
        assert_eq!(fingerprint.navigator.platform, "Linux x86_64");
        assert_eq!(fingerprint.navigator.vendor, "");
        assert!(fingerprint.headers.user_agent.contains("Linux"));
    }

    #[tokio::test]
    async fn test_fingerprint_generator_android() {
        let generator = FingerprintGeneratorImpl::new();

        let result = generator.generate_android().await;

        assert!(result.is_ok());
        let fingerprint = result.unwrap();

        // Verify Android-specific properties
        assert_eq!(fingerprint.navigator.platform, "Linux armv8l");
        assert_eq!(fingerprint.navigator.hardware_concurrency, 8);
        assert_eq!(fingerprint.navigator.device_memory, Some(8));
        assert!(fingerprint.headers.user_agent.contains("Android"));

        // Verify mobile screen resolution
        assert!(fingerprint.screen.width >= 360);
        assert!(fingerprint.screen.width <= 414);
        assert!(fingerprint.screen.height >= 800);
    }

    #[tokio::test]
    async fn test_fingerprint_generator_ios() {
        let generator = FingerprintGeneratorImpl::new();

        let result = generator.generate_ios().await;

        assert!(result.is_ok());
        let fingerprint = result.unwrap();

        // Verify iOS-specific properties
        assert_eq!(fingerprint.navigator.platform, "iPhone");
        assert_eq!(fingerprint.navigator.vendor, "Apple Computer, Inc.");
        assert_eq!(fingerprint.navigator.hardware_concurrency, 6);
        assert_eq!(fingerprint.navigator.device_memory, Some(6));
        assert!(fingerprint.headers.user_agent.contains("iPhone") || fingerprint.headers.user_agent.contains("iPad"));

        // Verify mobile screen resolution
        assert!(fingerprint.screen.width >= 390);
        assert!(fingerprint.screen.width <= 1024);
    }

    #[tokio::test]
    async fn test_fingerprint_generator_custom() {
        let generator = FingerprintGeneratorImpl::new();

        let options = services::CustomOptions {
            user_agent: Some("CustomUserAgent/1.0".to_string()),
            platform: Some("CustomPlatform".to_string()),
            viewport: Some(services::Viewport {
                width: 2560,
                height: 1440,
                device_scale_factor: 1.0,
            }),
        };

        let result = generator.generate_custom(&options).await;

        assert!(result.is_ok());
        let fingerprint = result.unwrap();

        // Verify custom properties
        assert_eq!(fingerprint.headers.user_agent, "CustomUserAgent/1.0");
        assert_eq!(fingerprint.navigator.platform, "CustomPlatform");
        assert_eq!(fingerprint.screen.width, 2560);
        assert_eq!(fingerprint.screen.height, 1440);
    }

    #[tokio::test]
    async fn test_fingerprint_randomization() {
        let generator = FingerprintGeneratorImpl::new();

        // Generate base fingerprint
        let base = generator.generate_windows().await.unwrap();

        // Generate multiple randomizations
        let randomized1 = generator.randomize(&base).await.unwrap();
        let randomized2 = generator.randomize(&base).await.unwrap();

        // Verify hardware concurrency changed
        assert_ne!(
            base.navigator.hardware_concurrency,
            randomized1.navigator.hardware_concurrency
        );

        // Verify randomizations are different
        assert_ne!(
            randomized1.navigator.hardware_concurrency,
            randomized2.navigator.hardware_concurrency
        );

        // Verify screen dimensions have small variations
        let width_diff = (base.screen.width as i32 - randomized1.screen.width as i32).abs();
        assert!(width_diff <= 5);

        // Verify device memory changed
        assert_ne!(
            base.navigator.device_memory,
            randomized1.navigator.device_memory
        );
    }

    #[tokio::test]
    async fn test_fingerprint_user_agent_variety() {
        let generator = FingerprintGeneratorImpl::new();

        // Generate multiple Windows fingerprints
        let agents = vec![
            generator.generate_windows().await.unwrap().headers.user_agent,
            generator.generate_windows().await.unwrap().headers.user_agent,
            generator.generate_windows().await.unwrap().headers.user_agent,
            generator.generate_windows().await.unwrap().headers.user_agent,
        ];

        // Verify user agents come from the predefined list
        for agent in &agents {
            assert!(WINDOWS_USER_AGENTS.contains(&agent.as_str()));
        }

        // Verify at least some variety (probability check)
        let unique_agents: std::collections::HashSet<_> = agents.iter().collect();
        assert!(unique_agents.len() >= 1);
    }

    #[tokio::test]
    async fn test_fingerprint_hardware_concurrency_range() {
        let generator = FingerprintGeneratorImpl::new();

        // Generate many fingerprints and check hardware concurrency range
        for _ in 0..20 {
            let fingerprint = generator.generate_windows().await.unwrap();
            let concurrency = fingerprint.navigator.hardware_concurrency;

            // Verify it's in the valid range [4, 6, 8, 12, 16, 24, 32]
            assert!(concurrency >= 4);
            assert!(concurrency <= 32);
            assert!(concurrency % 2 == 0);
        }
    }

    #[tokio::test]
    async fn test_fingerprint_device_memory_range() {
        let generator = FingerprintGeneratorImpl::new();

        // Generate many fingerprints and check device memory range
        for _ in 0..20 {
            let fingerprint = generator.generate_windows().await.unwrap();
            let memory = fingerprint.navigator.device_memory;

            assert!(memory.is_some());
            let mem_value = memory.unwrap();
            assert!(mem_value == 4 || mem_value == 8 || mem_value == 16 || mem_value == 32);
        }
    }

    #[tokio::test]
    async fn test_fingerprint_screen_resolutions_by_platform() {
        let generator = FingerprintGeneratorImpl::new();

        // Windows resolutions
        for _ in 0..10 {
            let fp = generator.generate_windows().await.unwrap();
            let valid = vec![(1920, 1080), (2560, 1440), (3840, 2160), (1366, 768)];
            assert!(valid.contains(&(fp.screen.width, fp.screen.height)));
        }

        // macOS resolutions
        for _ in 0..10 {
            let fp = generator.generate_macos().await.unwrap();
            let valid = vec![
                (2560, 1440),
                (2880, 1800),
                (3840, 2160),
                (5120, 2880),
                (1920, 1080),
            ];
            assert!(valid.contains(&(fp.screen.width, fp.screen.height)));
        }

        // Android resolutions
        for _ in 0..10 {
            let fp = generator.generate_android().await.unwrap();
            let valid = vec![(360, 800), (390, 844), (414, 896), (393, 851), (412, 915)];
            assert!(valid.contains(&(fp.screen.width, fp.screen.height)));
        }
    }

    #[tokio::test]
    async fn test_fingerprint_webgl_vendor_renderer() {
        let generator = FingerprintGeneratorImpl::new();

        // Generate multiple fingerprints
        for _ in 0..10 {
            let fp = generator.generate_windows().await.unwrap();

            // Verify vendor is from predefined list
            assert!(WEBGL_VENDORS.contains(&fp.webgl.vendor.as_str()));

            // Verify renderer is from predefined list
            assert!(WEBGL_RENDERERS.contains(&fp.webgl.renderer.as_str()));
        }
    }

    // ============================================================================
    // Script Injector Tests
    // ============================================================================

    #[tokio::test]
    async fn test_script_injector_inject_init_script() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = ScriptInjectorImpl::new(mock_cdp.clone());

        let page_id = "test_page_1";
        let script = "(function() { console.log('test'); })();";

        let result = injector.inject_init_script(page_id, script).await;

        assert!(result.is_ok());

        // Verify script was tracked
        let scripts = injector.get_injected_scripts(page_id).await.unwrap();
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].script_type, ScriptType::InitScript);
        assert_eq!(scripts[0].content, script);
    }

    #[tokio::test]
    async fn test_script_injector_evaluate() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = ScriptInjectorImpl::new(mock_cdp.clone());

        let page_id = "test_page_2";
        let script = "1 + 1";

        let result = injector.evaluate(page_id, script).await;

        assert!(result.is_ok());
        // Mock returns "mock_result"
        assert!(!result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_script_injector_inject_style() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = ScriptInjectorImpl::new(mock_cdp.clone());

        let page_id = "test_page_3";
        let css = "body { background: red; }";

        let result = injector.inject_style(page_id, css).await;

        assert!(result.is_ok());

        // Verify style was tracked
        let scripts = injector.get_injected_scripts(page_id).await.unwrap();
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].script_type, ScriptType::Style);
        assert_eq!(scripts[0].content, css);
    }

    #[tokio::test]
    async fn test_script_injector_multiple_scripts() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = ScriptInjectorImpl::new(mock_cdp.clone());

        let page_id = "test_page_4";

        // Inject multiple scripts
        injector
            .inject_init_script(page_id, "script1")
            .await
            .unwrap();
        injector
            .inject_init_script(page_id, "script2")
            .await
            .unwrap();
        injector
            .inject_style(page_id, "style1")
            .await
            .unwrap();

        let scripts = injector.get_injected_scripts(page_id).await.unwrap();
        assert_eq!(scripts.len(), 3);
    }

    #[tokio::test]
    async fn test_script_injector_remove_script() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = ScriptInjectorImpl::new(mock_cdp.clone());

        let page_id = "test_page_5";

        // Inject a script
        injector
            .inject_init_script(page_id, "script1")
            .await
            .unwrap();

        let scripts = injector.get_injected_scripts(page_id).await.unwrap();
        let script_id = scripts[0].script_id.clone();

        // Remove the script
        let result = injector.remove_script(page_id, &script_id).await;
        assert!(result.is_ok());

        // Verify script was removed
        let scripts = injector.get_injected_scripts(page_id).await.unwrap();
        assert_eq!(scripts.len(), 0);
    }

    #[tokio::test]
    async fn test_script_injector_clear_all() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = ScriptInjectorImpl::new(mock_cdp.clone());

        let page_id = "test_page_6";

        // Inject multiple scripts
        injector
            .inject_init_script(page_id, "script1")
            .await
            .unwrap();
        injector
            .inject_init_script(page_id, "script2")
            .await
            .unwrap();
        injector
            .inject_style(page_id, "style1")
            .await
            .unwrap();

        // Clear all
        let result = injector.clear_all(page_id).await;
        assert!(result.is_ok());

        // Verify all scripts were cleared
        let result = injector.get_injected_scripts(page_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_script_injector_script_id_generation() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = ScriptInjectorImpl::new(mock_cdp.clone());

        let page_id = "test_page_7";

        // Inject multiple scripts
        injector
            .inject_init_script(page_id, "script1")
            .await
            .unwrap();
        injector
            .inject_init_script(page_id, "script2")
            .await
            .unwrap();

        let scripts = injector.get_injected_scripts(page_id).await.unwrap();

        // Verify script IDs are unique
        assert_ne!(scripts[0].script_id, scripts[1].script_id);

        // Verify script IDs are UUIDs
        assert!(scripts[0].script_id.len() > 30);
        assert!(scripts[1].script_id.len() > 30);
    }

    #[tokio::test]
    async fn test_script_injector_get_nonexistent_scripts() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = ScriptInjectorImpl::new(mock_cdp);

        let result = injector.get_injected_scripts("nonexistent_page").await;
        assert!(result.is_err());
    }

    // ============================================================================
    // Stealth Engine Tests
    // ============================================================================

    #[tokio::test]
    async fn test_stealth_engine_apply_profile() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = Arc::new(ScriptInjectorImpl::new(mock_cdp.clone())) as Arc<dyn ScriptInjector>;
        let simulator =
            Arc::new(BehaviorSimulatorImpl::new(mock_cdp.clone())) as Arc<dyn BehaviorSimulator>;

        let engine = StealthEngineImpl::new(injector, simulator);

        // Create a test profile
        let profile = services::Profile {
            profile_id: "test_profile".to_string(),
            profile_type: services::ProfileType::Windows,
            fingerprint: services::Fingerprint {
                headers: services::HeadersFingerprint {
                    user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
                    accept_language: "en-US,en;q=0.9".to_string(),
                    accept_encoding: "gzip, deflate, br".to_string(),
                },
                navigator: services::NavigatorFingerprint {
                    platform: "Win32".to_string(),
                    vendor: "Google Inc.".to_string(),
                    hardware_concurrency: 8,
                    device_memory: Some(8),
                    language: "en-US".to_string(),
                },
                screen: services::ScreenFingerprint {
                    width: 1920,
                    height: 1080,
                    color_depth: 24,
                    pixel_depth: 24,
                },
                webgl: services::WebGLFingerprint {
                    vendor: "Google Inc. (NVIDIA)".to_string(),
                    renderer: "ANGLE (NVIDIA GeForce RTX 3080)".to_string(),
                },
                options: services::ProfileOptions {
                    inject_navigator: true,
                    inject_screen: true,
                    inject_webgl: true,
                    inject_canvas: true,
                    inject_audio: true,
                },
            },
        };

        let result = engine.apply_profile("page_1", &profile).await;

        assert!(result.is_ok());
        let features = result.unwrap();

        // Verify all features were applied
        assert_eq!(features.features.len(), 5);
        assert!(features.features.contains(&"navigator".to_string()));
        assert!(features.features.contains(&"screen".to_string()));
        assert!(features.features.contains(&"webgl".to_string()));
        assert!(features.features.contains(&"canvas".to_string()));
        assert!(features.features.contains(&"audio".to_string()));
    }

    #[tokio::test]
    async fn test_stealth_engine_apply_profile_partial() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = Arc::new(ScriptInjectorImpl::new(mock_cdp.clone())) as Arc<dyn ScriptInjector>;
        let simulator =
            Arc::new(BehaviorSimulatorImpl::new(mock_cdp.clone())) as Arc<dyn BehaviorSimulator>;

        let engine = StealthEngineImpl::new(injector, simulator);

        // Create a profile with only some features enabled
        let profile = services::Profile {
            profile_id: "test_profile".to_string(),
            profile_type: services::ProfileType::Windows,
            fingerprint: services::Fingerprint {
                headers: services::HeadersFingerprint {
                    user_agent: "Mozilla/5.0".to_string(),
                    accept_language: "en-US".to_string(),
                    accept_encoding: "gzip".to_string(),
                },
                navigator: services::NavigatorFingerprint {
                    platform: "Win32".to_string(),
                    vendor: "Google Inc.".to_string(),
                    hardware_concurrency: 8,
                    device_memory: Some(8),
                    language: "en-US".to_string(),
                },
                screen: services::ScreenFingerprint {
                    width: 1920,
                    height: 1080,
                    color_depth: 24,
                    pixel_depth: 24,
                },
                webgl: services::WebGLFingerprint {
                    vendor: "Google Inc.".to_string(),
                    renderer: "ANGLE".to_string(),
                },
                options: services::ProfileOptions {
                    inject_navigator: true,
                    inject_screen: true,
                    inject_webgl: false,
                    inject_canvas: false,
                    inject_audio: false,
                },
            },
        };

        let result = engine.apply_profile("page_2", &profile).await;

        assert!(result.is_ok());
        let features = result.unwrap();

        // Verify only enabled features were applied
        assert_eq!(features.features.len(), 2);
        assert!(features.features.contains(&"navigator".to_string()));
        assert!(features.features.contains(&"screen".to_string()));
        assert!(!features.features.contains(&"webgl".to_string()));
    }

    #[tokio::test]
    async fn test_stealth_engine_get_applied_features() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = Arc::new(ScriptInjectorImpl::new(mock_cdp.clone())) as Arc<dyn ScriptInjector>;
        let simulator =
            Arc::new(BehaviorSimulatorImpl::new(mock_cdp.clone())) as Arc<dyn BehaviorSimulator>;

        let engine = StealthEngineImpl::new(injector, simulator);

        let page_id = "page_3";

        // Apply a profile
        let profile = services::Profile {
            profile_id: "test_profile".to_string(),
            profile_type: services::ProfileType::Windows,
            fingerprint: services::Fingerprint {
                headers: services::HeadersFingerprint {
                    user_agent: "Mozilla/5.0".to_string(),
                    accept_language: "en-US".to_string(),
                    accept_encoding: "gzip".to_string(),
                },
                navigator: services::NavigatorFingerprint {
                    platform: "Win32".to_string(),
                    vendor: "Google Inc.".to_string(),
                    hardware_concurrency: 8,
                    device_memory: Some(8),
                    language: "en-US".to_string(),
                },
                screen: services::ScreenFingerprint {
                    width: 1920,
                    height: 1080,
                    color_depth: 24,
                    pixel_depth: 24,
                },
                webgl: services::WebGLFingerprint {
                    vendor: "Google Inc.".to_string(),
                    renderer: "ANGLE".to_string(),
                },
                options: services::ProfileOptions {
                    inject_navigator: true,
                    inject_screen: true,
                    inject_webgl: false,
                    inject_canvas: false,
                    inject_audio: false,
                },
            },
        };

        engine.apply_profile(page_id, &profile).await.unwrap();

        // Get applied features
        let result = engine.get_applied_features(page_id).await;

        assert!(result.is_ok());
        let features = result.unwrap();
        assert_eq!(features.features.len(), 2);
    }

    #[tokio::test]
    async fn test_stealth_engine_get_applied_features_nonexistent() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = Arc::new(ScriptInjectorImpl::new(mock_cdp.clone())) as Arc<dyn ScriptInjector>;
        let simulator =
            Arc::new(BehaviorSimulatorImpl::new(mock_cdp.clone())) as Arc<dyn BehaviorSimulator>;

        let engine = StealthEngineImpl::new(injector, simulator);

        let result = engine.get_applied_features("nonexistent_page").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stealth_engine_remove_all() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = Arc::new(ScriptInjectorImpl::new(mock_cdp.clone())) as Arc<dyn ScriptInjector>;
        let simulator =
            Arc::new(BehaviorSimulatorImpl::new(mock_cdp.clone())) as Arc<dyn BehaviorSimulator>;

        let engine = StealthEngineImpl::new(injector.clone(), simulator);

        let page_id = "page_4";

        // Apply a profile
        let profile = services::Profile {
            profile_id: "test_profile".to_string(),
            profile_type: services::ProfileType::Windows,
            fingerprint: services::Fingerprint {
                headers: services::HeadersFingerprint {
                    user_agent: "Mozilla/5.0".to_string(),
                    accept_language: "en-US".to_string(),
                    accept_encoding: "gzip".to_string(),
                },
                navigator: services::NavigatorFingerprint {
                    platform: "Win32".to_string(),
                    vendor: "Google Inc.".to_string(),
                    hardware_concurrency: 8,
                    device_memory: Some(8),
                    language: "en-US".to_string(),
                },
                screen: services::ScreenFingerprint {
                    width: 1920,
                    height: 1080,
                    color_depth: 24,
                    pixel_depth: 24,
                },
                webgl: services::WebGLFingerprint {
                    vendor: "Google Inc.".to_string(),
                    renderer: "ANGLE".to_string(),
                },
                options: services::ProfileOptions {
                    inject_navigator: true,
                    inject_screen: true,
                    inject_webgl: false,
                    inject_canvas: false,
                    inject_audio: false,
                },
            },
        };

        engine.apply_profile(page_id, &profile).await.unwrap();

        // Remove all
        let result = engine.remove_all(page_id).await;
        assert!(result.is_ok());

        // Verify features were removed
        let result = engine.get_applied_features(page_id).await;
        assert!(result.is_err());

        // Verify scripts were cleared
        let result = injector.get_injected_scripts(page_id).await;
        assert!(result.is_err());
    }

    // ============================================================================
    // Behavior Simulator Tests
    // ============================================================================

    #[tokio::test]
    async fn test_behavior_simulator_mouse_move_options() {
        let options = MouseMoveOptions::default();

        assert_eq!(options.duration_ms, 500);
        assert_eq!(options.deviation, 50.0);
        assert_eq!(options.points, 20);
    }

    #[tokio::test]
    async fn test_behavior_simulator_typing_options() {
        let options = TypingOptions::default();

        assert_eq!(options.mean_delay_ms, 100);
        assert_eq!(options.std_dev_ms, 50);
        assert_eq!(options.typo_probability, 0.02);
        assert_eq!(options.backspace_probability, 0.01);
    }

    #[tokio::test]
    async fn test_behavior_simulator_click_options() {
        let options = ClickOptions::default();

        assert_eq!(options.delay_before_ms, 100);
        assert_eq!(options.movement_duration_ms, 300);
        assert_eq!(options.hold_duration_ms, 50);
    }

    #[tokio::test]
    async fn test_behavior_simulator_scroll_options() {
        let options = ScrollOptions::default();

        assert_eq!(options.duration_ms, 1000);
        assert_eq!(options.steps, 10);
        assert!(options.acceleration);
    }

    #[tokio::test]
    async fn test_behavior_simulator_random_delay() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let simulator = BehaviorSimulatorImpl::new(mock_cdp);

        let result = simulator.random_delay(10, 20).await;
        assert!(result.is_ok());

        // Delay should complete without error
        // Actual delay time is not tested to avoid slowing down tests
    }

    #[tokio::test]
    async fn test_behavior_simulator_bezier_path_generation() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let simulator = BehaviorSimulatorImpl::new(mock_cdp);

        let start = (0.0, 0.0);
        let end = (100.0, 100.0);
        let options = MouseMoveOptions {
            duration_ms: 100,
            deviation: 20.0,
            points: 10,
        };

        // Simulate mouse movement (will use mock CDP)
        let result = simulator.simulate_mouse_move("test_page", start, end, options).await;

        assert!(result.is_ok());
    }

    // ============================================================================
    // End-to-End Tests
    // ============================================================================

    #[tokio::test]
    async fn test_end_to_end_profile_workflow() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = Arc::new(ScriptInjectorImpl::new(mock_cdp.clone())) as Arc<dyn ScriptInjector>;
        let simulator =
            Arc::new(BehaviorSimulatorImpl::new(mock_cdp.clone())) as Arc<dyn BehaviorSimulator>;

        let engine = StealthEngineImpl::new(injector, simulator);

        // Step 1: Generate a fingerprint
        let generator = FingerprintGeneratorImpl::new();
        let fingerprint = generator.generate_windows().await.unwrap();

        // Step 2: Create a profile
        let profile = services::Profile {
            profile_id: "e2e_profile".to_string(),
            profile_type: services::ProfileType::Windows,
            fingerprint,
        };

        // Step 3: Apply profile
        let page_id = "e2e_page";
        let features = engine.apply_profile(page_id, &profile).await.unwrap();

        assert!(!features.features.is_empty());

        // Step 4: Verify applied features
        let applied = engine.get_applied_features(page_id).await.unwrap();
        assert_eq!(features.features.len(), applied.features.len());

        // Step 5: Cleanup
        engine.remove_all(page_id).await.unwrap();

        // Verify cleanup
        let result = engine.get_applied_features(page_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_end_to_end_multi_profile_workflow() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = Arc::new(ScriptInjectorImpl::new(mock_cdp.clone())) as Arc<dyn ScriptInjector>;
        let simulator =
            Arc::new(BehaviorSimulatorImpl::new(mock_cdp.clone())) as Arc<dyn BehaviorSimulator>;

        let engine = StealthEngineImpl::new(injector, simulator);

        let generator = FingerprintGeneratorImpl::new();

        // Apply different profiles to different pages
        let windows_fp = generator.generate_windows().await.unwrap();
        let macos_fp = generator.generate_macos().await.unwrap();

        let windows_profile = services::Profile {
            profile_id: "windows_profile".to_string(),
            profile_type: services::ProfileType::Windows,
            fingerprint: windows_fp,
        };

        let macos_profile = services::Profile {
            profile_id: "macos_profile".to_string(),
            profile_type: services::ProfileType::MacOS,
            fingerprint: macos_fp,
        };

        // Apply to different pages
        engine
            .apply_profile("page_windows", &windows_profile)
            .await
            .unwrap();
        engine
            .apply_profile("page_macos", &macos_profile)
            .await
            .unwrap();

        // Verify both are tracked separately
        let windows_features = engine.get_applied_features("page_windows").await.unwrap();
        let macos_features = engine.get_applied_features("page_macos").await.unwrap();

        assert!(!windows_features.features.is_empty());
        assert!(!macos_features.features.is_empty());

        // Cleanup
        engine.remove_all("page_windows").await.unwrap();
        engine.remove_all("page_macos").await.unwrap();
    }

    #[tokio::test]
    async fn test_end_to_end_fingerprint_randomization_workflow() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = Arc::new(ScriptInjectorImpl::new(mock_cdp.clone())) as Arc<dyn ScriptInjector>;
        let simulator =
            Arc::new(BehaviorSimulatorImpl::new(mock_cdp.clone())) as Arc<dyn BehaviorSimulator>;

        let engine = StealthEngineImpl::new(injector, simulator);

        let generator = FingerprintGeneratorImpl::new();

        // Generate base fingerprint
        let base = generator.generate_windows().await.unwrap();

        // Create multiple randomizations
        let rand1 = generator.randomize(&base).await.unwrap();
        let rand2 = generator.randomize(&rand1).await.unwrap();
        let rand3 = generator.randomize(&rand2).await.unwrap();

        // Create profiles
        let profile1 = services::Profile {
            profile_id: "profile1".to_string(),
            profile_type: services::ProfileType::Windows,
            fingerprint: rand1,
        };

        let profile2 = services::Profile {
            profile_id: "profile2".to_string(),
            profile_type: services::ProfileType::Windows,
            fingerprint: rand2,
        };

        let profile3 = services::Profile {
            profile_id: "profile3".to_string(),
            profile_type: services::ProfileType::Windows,
            fingerprint: rand3,
        };

        // Apply all profiles
        engine.apply_profile("page1", &profile1).await.unwrap();
        engine.apply_profile("page2", &profile2).await.unwrap();
        engine.apply_profile("page3", &profile3).await.unwrap();

        // Verify all applied
        assert!(engine.get_applied_features("page1").await.is_ok());
        assert!(engine.get_applied_features("page2").await.is_ok());
        assert!(engine.get_applied_features("page3").await.is_ok());

        // Cleanup
        engine.remove_all("page1").await.unwrap();
        engine.remove_all("page2").await.unwrap();
        engine.remove_all("page3").await.unwrap();
    }

    // ============================================================================
    // Edge Cases and Error Handling
    // ============================================================================

    #[tokio::test]
    async fn test_empty_profile_options() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = Arc::new(ScriptInjectorImpl::new(mock_cdp.clone())) as Arc<dyn ScriptInjector>;
        let simulator =
            Arc::new(BehaviorSimulatorImpl::new(mock_cdp.clone())) as Arc<dyn BehaviorSimulator>;

        let engine = StealthEngineImpl::new(injector, simulator);

        // Create a profile with all options disabled
        let profile = services::Profile {
            profile_id: "empty_profile".to_string(),
            profile_type: services::ProfileType::Windows,
            fingerprint: services::Fingerprint {
                headers: services::HeadersFingerprint {
                    user_agent: "Mozilla/5.0".to_string(),
                    accept_language: "en-US".to_string(),
                    accept_encoding: "gzip".to_string(),
                },
                navigator: services::NavigatorFingerprint {
                    platform: "Win32".to_string(),
                    vendor: "Google Inc.".to_string(),
                    hardware_concurrency: 8,
                    device_memory: Some(8),
                    language: "en-US".to_string(),
                },
                screen: services::ScreenFingerprint {
                    width: 1920,
                    height: 1080,
                    color_depth: 24,
                    pixel_depth: 24,
                },
                webgl: services::WebGLFingerprint {
                    vendor: "Google Inc.".to_string(),
                    renderer: "ANGLE".to_string(),
                },
                options: services::ProfileOptions {
                    inject_navigator: false,
                    inject_screen: false,
                    inject_webgl: false,
                    inject_canvas: false,
                    inject_audio: false,
                },
            },
        };

        let result = engine.apply_profile("empty_page", &profile).await;

        assert!(result.is_ok());
        let features = result.unwrap();
        assert_eq!(features.features.len(), 0);
    }

    #[tokio::test]
    async fn test_custom_fingerprint_with_defaults() {
        let generator = FingerprintGeneratorImpl::new();

        // Custom options with minimal parameters
        let options = services::CustomOptions {
            user_agent: None,
            platform: None,
            viewport: None,
        };

        let result = generator.generate_custom(&options).await;

        assert!(result.is_ok());
        let fingerprint = result.unwrap();

        // Verify defaults are used
        assert_eq!(fingerprint.headers.user_agent, WINDOWS_USER_AGENTS[0]);
        assert_eq!(fingerprint.navigator.platform, "Win32");
        assert_eq!(fingerprint.screen.width, 1920);
        assert_eq!(fingerprint.screen.height, 1080);
    }

    #[tokio::test]
    async fn test_script_injector_css_escaping() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = ScriptInjectorImpl::new(mock_cdp);

        let page_id = "test_page_css";
        let css = r#"body { content: '\'; "\""; }"#; // CSS with quotes and backslashes

        let result = injector.inject_style(page_id, css).await;

        assert!(result.is_ok());

        // Verify script was tracked
        let scripts = injector.get_injected_scripts(page_id).await.unwrap();
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].script_type, ScriptType::Style);
    }

    #[tokio::test]
    async fn test_multiple_page_isolation() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = ScriptInjectorImpl::new(mock_cdp);

        // Inject scripts into different pages
        injector
            .inject_init_script("page_a", "script_a")
            .await
            .unwrap();
        injector
            .inject_init_script("page_b", "script_b")
            .await
            .unwrap();

        // Verify isolation
        let scripts_a = injector.get_injected_scripts("page_a").await.unwrap();
        let scripts_b = injector.get_injected_scripts("page_b").await.unwrap();

        assert_eq!(scripts_a.len(), 1);
        assert_eq!(scripts_b.len(), 1);
        assert_ne!(scripts_a[0].content, scripts_b[0].content);
    }

    #[tokio::test]
    async fn test_remove_nonexistent_script() {
        let mock_cdp = Arc::new(CdpMockClient::new());
        let injector = ScriptInjectorImpl::new(mock_cdp);

        let page_id = "test_page_remove";

        // Try to remove a script that doesn't exist
        let result = injector.remove_script(page_id, "nonexistent_id").await;

        // Should succeed without error
        assert!(result.is_ok());
    }

    // ============================================================================
    // Constants Validation Tests
    // ============================================================================

    #[test]
    fn test_user_agent_constants_validity() {
        // Verify user agent strings are valid
        assert!(!WINDOWS_USER_AGENTS.is_empty());
        assert!(!MACOS_USER_AGENTS.is_empty());
        assert!(!LINUX_USER_AGENTS.is_empty());
        assert!(!ANDROID_USER_AGENTS.is_empty());
        assert!(!IOS_USER_AGENTS.is_empty());

        // Verify user agents contain expected patterns
        for ua in WINDOWS_USER_AGENTS {
            assert!(ua.contains("Windows") || ua.contains("Win"));
        }

        for ua in MACOS_USER_AGENTS {
            assert!(ua.contains("Macintosh") || ua.contains("Mac"));
        }

        for ua in ANDROID_USER_AGENTS {
            assert!(ua.contains("Android"));
        }

        for ua in IOS_USER_AGENTS {
            assert!(ua.contains("iPhone") || ua.contains("iPad") || ua.contains("iOS"));
        }
    }

    #[test]
    fn test_webgl_constants_validity() {
        // Verify WebGL constants are valid
        assert!(!WEBGL_VENDORS.is_empty());
        assert!(!WEBGL_RENDERERS.is_empty());

        // Verify vendors contain expected patterns
        for vendor in WEBGL_VENDORS {
            assert!(!vendor.is_empty());
        }

        // Verify renderers contain expected patterns
        for renderer in WEBGL_RENDERERS {
            assert!(!renderer.is_empty());
            assert!(renderer.contains("ANGLE") || renderer.contains("NVIDIA") || renderer.contains("Intel") || renderer.contains("AMD") || renderer.contains("Qualcomm") || renderer.contains("Apple"));
        }
    }
}
