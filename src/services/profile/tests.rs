//! Profile service tests

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::super::super::traits::*;

    #[tokio::test]
    async fn test_profile_presets() {
        // Test preset profiles
        let presets = vec![
            ProfilePreset {
                name: "Windows Chrome".to_string(),
                profile_type: ProfileType::Windows,
                description: "Windows 10 with Chrome browser".to_string(),
            },
            ProfilePreset {
                name: "macOS Safari".to_string(),
                profile_type: ProfileType::MacOS,
                description: "macOS with Safari browser".to_string(),
            },
        ];

        assert_eq!(presets.len(), 2);
        assert_eq!(presets[0].profile_type, ProfileType::Windows);
        assert_eq!(presets[1].profile_type, ProfileType::MacOS);
    }

    #[test]
    fn test_profile_type_variants() {
        // Test all profile type variants
        let types = vec![
            ProfileType::Windows,
            ProfileType::Linux,
            ProfileType::MacOS,
            ProfileType::Android,
            ProfileType::IOS,
            ProfileType::Custom,
        ];

        assert_eq!(types.len(), 6);
    }

    #[test]
    fn test_fingerprint_structure() {
        let fingerprint = Fingerprint {
            headers: HeadersFingerprint {
                user_agent: "Mozilla/5.0".to_string(),
                accept_language: "en-US".to_string(),
                accept_encoding: "gzip".to_string(),
            },
            navigator: NavigatorFingerprint {
                platform: "Win32".to_string(),
                vendor: "Google Inc.".to_string(),
                hardware_concurrency: 8,
                device_memory: Some(8),
                language: "en-US".to_string(),
            },
            screen: ScreenFingerprint {
                width: 1920,
                height: 1080,
                color_depth: 24,
                pixel_depth: 24,
            },
            webgl: WebGLFingerprint {
                vendor: "Google Inc. (NVIDIA)".to_string(),
                renderer: "ANGLE (NVIDIA GeForce)".to_string(),
            },
            options: ProfileOptions {
                inject_navigator: true,
                inject_screen: true,
                inject_webgl: true,
                inject_canvas: true,
                inject_audio: true,
            },
        };

        assert_eq!(fingerprint.navigator.hardware_concurrency, 8);
        assert_eq!(fingerprint.screen.width, 1920);
        assert!(fingerprint.options.inject_navigator);
    }

    #[test]
    fn test_profile_options() {
        let options = ProfileOptions {
            inject_navigator: true,
            inject_screen: true,
            inject_webgl: true,
            inject_canvas: true,
            inject_audio: true,
        };

        assert!(options.inject_navigator);
        assert!(options.inject_screen);
        assert!(options.inject_webgl);
        assert!(options.inject_canvas);
        assert!(options.inject_audio);
    }
}
