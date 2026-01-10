//! Fingerprint generator implementation
//!
//! Generates realistic browser fingerprints for various platforms.

use async_trait::async_trait;
use rand::Rng;

use crate::Error;
use super::super::services::traits as services;

/// User agent templates
pub const WINDOWS_USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:132.0) Gecko/20100101 Firefox/132.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0",
];

pub const MACOS_USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.2 Safari/605.1.15",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:132.0) Gecko/20100101 Firefox/132.0",
];

pub const LINUX_USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:132.0) Gecko/20100101 Firefox/132.0",
];

pub const ANDROID_USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Linux; Android 14) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36",
    "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36",
];

pub const IOS_USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (iPhone; CPU iPhone OS 18_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.2 Mobile/15E148 Safari/604.1",
    "Mozilla/5.0 (iPad; CPU OS 18_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.2 Mobile/15E148 Safari/604.1",
];

/// WebGL vendors
pub const WEBGL_VENDORS: &[&str] = &[
    "Google Inc. (NVIDIA)",
    "Google Inc. (Intel)",
    "Google Inc. (AMD)",
];

/// WebGL renderers
pub const WEBGL_RENDERERS: &[&str] = &[
    "ANGLE (NVIDIA GeForce RTX 3080 Direct3D11 vs_5_0 ps_5_0)",
    "ANGLE (NVIDIA GeForce RTX 3070 Direct3D11 vs_5_0 ps_5_0)",
    "ANGLE (Intel(R) UHD Graphics 630 Direct3D11 vs_5_0 ps_5_0)",
    "ANGLE (AMD Radeon RX 6800 Direct3D11 vs_5_0 ps_5_0)",
];

/// Fingerprint generator implementation
pub struct FingerprintGeneratorImpl;

impl Default for FingerprintGeneratorImpl {
    fn default() -> Self {
        Self
    }
}

impl FingerprintGeneratorImpl {
    /// Create a new fingerprint generator
    pub fn new() -> Self {
        Self
    }

    /// Generate random hardware concurrency
    fn generate_hardware_concurrency() -> u32 {
        let mut rng = rand::thread_rng();
        let options = [4, 6, 8, 12, 16, 24, 32];
        options[rng.gen_range(0..options.len())]
    }

    /// Generate random device memory
    fn generate_device_memory() -> Option<u32> {
        let mut rng = rand::thread_rng();
        let options = [4, 8, 16, 32];
        Some(options[rng.gen_range(0..options.len())])
    }

    /// Generate screen resolution
    fn generate_screen_resolution(profile_type: services::ProfileType) -> (u32, u32) {
        let mut rng = rand::thread_rng();

        match profile_type {
            services::ProfileType::Windows => {
                let resolutions = [(1920, 1080), (2560, 1440), (3840, 2160), (1366, 768)];
                resolutions[rng.gen_range(0..resolutions.len())]
            }
            services::ProfileType::MacOS => {
                let resolutions = [
                    (2560, 1440),
                    (2880, 1800),
                    (3840, 2160),
                    (5120, 2880),
                    (1920, 1080),
                ];
                resolutions[rng.gen_range(0..resolutions.len())]
            }
            services::ProfileType::Linux => {
                let resolutions = [(1920, 1080), (2560, 1440), (3840, 2160)];
                resolutions[rng.gen_range(0..resolutions.len())]
            }
            services::ProfileType::Android => {
                let resolutions = [
                    (360, 800),
                    (390, 844),
                    (414, 896),
                    (393, 851),
                    (412, 915),
                ];
                resolutions[rng.gen_range(0..resolutions.len())]
            }
            services::ProfileType::IOS => {
                let resolutions = [(390, 844), (414, 896), (393, 851), (1024, 1366)];
                resolutions[rng.gen_range(0..resolutions.len())]
            }
            services::ProfileType::Custom => (1920, 1080),
        }
    }

    /// Generate locale
    fn generate_locale(profile_type: services::ProfileType) -> String {
        let mut rng = rand::thread_rng();

        let locales = match profile_type {
            services::ProfileType::Windows | services::ProfileType::MacOS | services::ProfileType::Linux => {
                vec!["en-US", "en-GB", "de-DE", "fr-FR", "es-ES", "ja-JP", "zh-CN"]
            }
            services::ProfileType::Android | services::ProfileType::IOS => {
                vec!["en-US", "en-GB", "de-DE", "fr-FR", "es-ES", "ja-JP", "zh-CN"]
            }
            services::ProfileType::Custom => vec!["en-US"],
        };

        locales[rng.gen_range(0..locales.len())].to_string()
    }

    /// Generate timezone
    #[allow(dead_code)]
    fn generate_timezone(_profile_type: services::ProfileType) -> String {
        let mut rng = rand::thread_rng();

        let timezones = vec![
            "America/New_York",
            "America/Chicago",
            "America/Denver",
            "America/Los_Angeles",
            "Europe/London",
            "Europe/Paris",
            "Europe/Berlin",
            "Asia/Tokyo",
            "Asia/Shanghai",
            "Australia/Sydney",
        ];

        timezones[rng.gen_range(0..timezones.len())].to_string()
    }
}

#[async_trait]
impl super::traits::FingerprintGenerator for FingerprintGeneratorImpl {
    /// Generate a Windows fingerprint
    async fn generate_windows(&self) -> Result<services::Fingerprint, Error> {
        let mut rng = rand::thread_rng();
        let (width, height) = Self::generate_screen_resolution(services::ProfileType::Windows);

        Ok(services::Fingerprint {
            headers: services::HeadersFingerprint {
                user_agent: WINDOWS_USER_AGENTS[rng.gen_range(0..WINDOWS_USER_AGENTS.len())].to_string(),
                accept_language: "en-US,en;q=0.9".to_string(),
                accept_encoding: "gzip, deflate, br".to_string(),
            },
            navigator: services::NavigatorFingerprint {
                platform: "Win32".to_string(),
                vendor: "Google Inc.".to_string(),
                hardware_concurrency: Self::generate_hardware_concurrency(),
                device_memory: Self::generate_device_memory(),
                language: Self::generate_locale(services::ProfileType::Windows),
            },
            screen: services::ScreenFingerprint {
                width,
                height,
                color_depth: 24,
                pixel_depth: 24,
            },
            webgl: services::WebGLFingerprint {
                vendor: WEBGL_VENDORS[rng.gen_range(0..WEBGL_VENDORS.len())].to_string(),
                renderer: WEBGL_RENDERERS[rng.gen_range(0..WEBGL_RENDERERS.len())].to_string(),
            },
            options: services::ProfileOptions {
                inject_navigator: true,
                inject_screen: true,
                inject_webgl: true,
                inject_canvas: true,
                inject_audio: true,
            },
        })
    }

    /// Generate a macOS fingerprint
    async fn generate_macos(&self) -> Result<services::Fingerprint, Error> {
        let mut rng = rand::thread_rng();
        let (width, height) = Self::generate_screen_resolution(services::ProfileType::MacOS);

        Ok(services::Fingerprint {
            headers: services::HeadersFingerprint {
                user_agent: MACOS_USER_AGENTS[rng.gen_range(0..MACOS_USER_AGENTS.len())].to_string(),
                accept_language: "en-US,en;q=0.9".to_string(),
                accept_encoding: "gzip, deflate, br".to_string(),
            },
            navigator: services::NavigatorFingerprint {
                platform: "MacIntel".to_string(),
                vendor: "Google Inc.".to_string(),
                hardware_concurrency: Self::generate_hardware_concurrency(),
                device_memory: Self::generate_device_memory(),
                language: Self::generate_locale(services::ProfileType::MacOS),
            },
            screen: services::ScreenFingerprint {
                width,
                height,
                color_depth: 24,
                pixel_depth: 24,
            },
            webgl: services::WebGLFingerprint {
                vendor: WEBGL_VENDORS[rng.gen_range(0..WEBGL_VENDORS.len())].to_string(),
                renderer: WEBGL_RENDERERS[rng.gen_range(0..WEBGL_RENDERERS.len())].to_string(),
            },
            options: services::ProfileOptions {
                inject_navigator: true,
                inject_screen: true,
                inject_webgl: true,
                inject_canvas: true,
                inject_audio: true,
            },
        })
    }

    /// Generate a Linux fingerprint
    async fn generate_linux(&self) -> Result<services::Fingerprint, Error> {
        let mut rng = rand::thread_rng();
        let (width, height) = Self::generate_screen_resolution(services::ProfileType::Linux);

        Ok(services::Fingerprint {
            headers: services::HeadersFingerprint {
                user_agent: LINUX_USER_AGENTS[rng.gen_range(0..LINUX_USER_AGENTS.len())].to_string(),
                accept_language: "en-US,en;q=0.9".to_string(),
                accept_encoding: "gzip, deflate".to_string(),
            },
            navigator: services::NavigatorFingerprint {
                platform: "Linux x86_64".to_string(),
                vendor: "".to_string(),
                hardware_concurrency: Self::generate_hardware_concurrency(),
                device_memory: Self::generate_device_memory(),
                language: Self::generate_locale(services::ProfileType::Linux),
            },
            screen: services::ScreenFingerprint {
                width,
                height,
                color_depth: 24,
                pixel_depth: 24,
            },
            webgl: services::WebGLFingerprint {
                vendor: WEBGL_VENDORS[rng.gen_range(0..WEBGL_VENDORS.len())].to_string(),
                renderer: WEBGL_RENDERERS[rng.gen_range(0..WEBGL_RENDERERS.len())].to_string(),
            },
            options: services::ProfileOptions {
                inject_navigator: true,
                inject_screen: true,
                inject_webgl: true,
                inject_canvas: true,
                inject_audio: true,
            },
        })
    }

    /// Generate an Android fingerprint
    async fn generate_android(&self) -> Result<services::Fingerprint, Error> {
        let mut rng = rand::thread_rng();
        let (width, height) = Self::generate_screen_resolution(services::ProfileType::Android);

        Ok(services::Fingerprint {
            headers: services::HeadersFingerprint {
                user_agent: ANDROID_USER_AGENTS[rng.gen_range(0..ANDROID_USER_AGENTS.len())].to_string(),
                accept_language: "en-US,en;q=0.9".to_string(),
                accept_encoding: "gzip, deflate, br".to_string(),
            },
            navigator: services::NavigatorFingerprint {
                platform: "Linux armv8l".to_string(),
                vendor: "Google Inc.".to_string(),
                hardware_concurrency: 8,
                device_memory: Some(8),
                language: Self::generate_locale(services::ProfileType::Android),
            },
            screen: services::ScreenFingerprint {
                width,
                height,
                color_depth: 24,
                pixel_depth: 24,
            },
            webgl: services::WebGLFingerprint {
                vendor: "Qualcomm".to_string(),
                renderer: "Adreno 740".to_string(),
            },
            options: services::ProfileOptions {
                inject_navigator: true,
                inject_screen: true,
                inject_webgl: true,
                inject_canvas: true,
                inject_audio: true,
            },
        })
    }

    /// Generate an iOS fingerprint
    async fn generate_ios(&self) -> Result<services::Fingerprint, Error> {
        let mut rng = rand::thread_rng();
        let (width, height) = Self::generate_screen_resolution(services::ProfileType::IOS);

        Ok(services::Fingerprint {
            headers: services::HeadersFingerprint {
                user_agent: IOS_USER_AGENTS[rng.gen_range(0..IOS_USER_AGENTS.len())].to_string(),
                accept_language: "en-US,en;q=0.9".to_string(),
                accept_encoding: "gzip, deflate, br".to_string(),
            },
            navigator: services::NavigatorFingerprint {
                platform: "iPhone".to_string(),
                vendor: "Apple Computer, Inc.".to_string(),
                hardware_concurrency: 6,
                device_memory: Some(6),
                language: Self::generate_locale(services::ProfileType::IOS),
            },
            screen: services::ScreenFingerprint {
                width,
                height,
                color_depth: 24,
                pixel_depth: 24,
            },
            webgl: services::WebGLFingerprint {
                vendor: "Apple Inc.".to_string(),
                renderer: "Apple GPU".to_string(),
            },
            options: services::ProfileOptions {
                inject_navigator: true,
                inject_screen: true,
                inject_webgl: true,
                inject_canvas: true,
                inject_audio: true,
            },
        })
    }

    /// Generate a custom fingerprint
    async fn generate_custom(
        &self,
        options: &services::CustomOptions,
    ) -> Result<services::Fingerprint, Error> {
        let (width, height) = options
            .viewport
            .as_ref()
            .map(|v| (v.width, v.height))
            .unwrap_or((1920, 1080));

        Ok(services::Fingerprint {
            headers: services::HeadersFingerprint {
                user_agent: options
                    .user_agent
                    .clone()
                    .unwrap_or_else(|| WINDOWS_USER_AGENTS[0].to_string()),
                accept_language: "en-US,en;q=0.9".to_string(),
                accept_encoding: "gzip, deflate, br".to_string(),
            },
            navigator: services::NavigatorFingerprint {
                platform: options
                    .platform
                    .clone()
                    .unwrap_or_else(|| "Win32".to_string()),
                vendor: "Google Inc.".to_string(),
                hardware_concurrency: 8,
                device_memory: Some(8),
                language: "en-US".to_string(),
            },
            screen: services::ScreenFingerprint {
                width,
                height,
                color_depth: 24,
                pixel_depth: 24,
            },
            webgl: services::WebGLFingerprint {
                vendor: "Google Inc. (NVIDIA)".to_string(),
                renderer: "ANGLE (NVIDIA GeForce RTX 3080 Direct3D11 vs_5_0 ps_5_0)".to_string(),
            },
            options: services::ProfileOptions {
                inject_navigator: true,
                inject_screen: true,
                inject_webgl: true,
                inject_canvas: true,
                inject_audio: true,
            },
        })
    }

    /// Randomize an existing fingerprint
    async fn randomize(
        &self,
        fingerprint: &services::Fingerprint,
    ) -> Result<services::Fingerprint, Error> {
        let mut rng = rand::thread_rng();

        // Randomize hardware concurrency
        let new_concurrency = Self::generate_hardware_concurrency();

        // Randomize device memory
        let new_memory = Self::generate_device_memory();

        // Small randomization of screen dimensions
        let width_variation = rng.gen_range(-5..5);
        let height_variation = rng.gen_range(-5..5);

        let mut new_fingerprint = fingerprint.clone();
        new_fingerprint.navigator.hardware_concurrency = new_concurrency;
        new_fingerprint.navigator.device_memory = new_memory;
        new_fingerprint.screen.width = (fingerprint.screen.width as i32 + width_variation).max(1024) as u32;
        new_fingerprint.screen.height = (fingerprint.screen.height as i32 + height_variation).max(768) as u32;

        Ok(new_fingerprint)
    }
}
