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

/// Configuration for creating base fingerprints
pub struct FingerprintConfig<'a> {
    pub platform: &'a str,
    pub vendor: &'a str,
    pub user_agent: &'a str,
    pub locale: &'a str,
    pub screen: (u32, u32),
    pub webgl_vendor: &'a str,
    pub webgl_renderer: &'a str,
    pub hardware_concurrency: u32,
    pub device_memory: Option<u32>,
    pub accept_encoding: &'a str,
}

impl FingerprintGeneratorImpl {
    /// Create a new fingerprint generator
    pub fn new() -> Self {
        Self
    }

    /// Generate random hardware concurrency from predefined values
    fn generate_hardware_concurrency() -> u32 {
        let options = [4, 6, 8, 12, 16, 24, 32];
        options[rand::random::<usize>() % options.len()]
    }

    /// Generate random device memory from predefined values
    fn generate_device_memory() -> Option<u32> {
        let options = [4, 8, 16, 32];
        Some(options[rand::random::<usize>() % options.len()])
    }

    /// Generate random user agent from platform-specific list
    fn random_user_agent(profile_type: services::ProfileType) -> &'static str {
        let agents = match profile_type {
            services::ProfileType::Windows => WINDOWS_USER_AGENTS,
            services::ProfileType::MacOS => MACOS_USER_AGENTS,
            services::ProfileType::Linux => LINUX_USER_AGENTS,
            services::ProfileType::Android => ANDROID_USER_AGENTS,
            services::ProfileType::IOS => IOS_USER_AGENTS,
            services::ProfileType::Custom => WINDOWS_USER_AGENTS,
        };
        agents[rand::random::<usize>() % agents.len()]
    }

    /// Generate random WebGL vendor
    fn random_webgl_vendor() -> &'static str {
        WEBGL_VENDORS[rand::random::<usize>() % WEBGL_VENDORS.len()]
    }

    /// Generate random WebGL renderer
    fn random_webgl_renderer() -> &'static str {
        WEBGL_RENDERERS[rand::random::<usize>() % WEBGL_RENDERERS.len()]
    }

    /// Create base fingerprint with common options
    fn create_base_fingerprint(config: FingerprintConfig<'_>) -> services::Fingerprint {
        services::Fingerprint {
            headers: services::HeadersFingerprint {
                user_agent: config.user_agent.to_string(),
                accept_language: format!("{},en;q=0.9", config.locale),
                accept_encoding: config.accept_encoding.to_string(),
            },
            navigator: services::NavigatorFingerprint {
                platform: config.platform.to_string(),
                vendor: config.vendor.to_string(),
                hardware_concurrency: config.hardware_concurrency,
                device_memory: config.device_memory,
                language: config.locale.to_string(),
            },
            screen: services::ScreenFingerprint {
                width: config.screen.0,
                height: config.screen.1,
                color_depth: 24,
                pixel_depth: 24,
            },
            webgl: services::WebGLFingerprint {
                vendor: config.webgl_vendor.to_string(),
                renderer: config.webgl_renderer.to_string(),
            },
            options: services::ProfileOptions {
                inject_navigator: true,
                inject_screen: true,
                inject_webgl: true,
                inject_canvas: true,
                inject_audio: true,
            },
        }
    }

    /// Generate screen resolution for platform
    fn generate_screen_resolution(profile_type: services::ProfileType) -> (u32, u32) {
        // Use Vec to allow different sizes per platform
        let resolutions: Vec<(u32, u32)> = match profile_type {
            services::ProfileType::Windows => {
                vec![(1920, 1080), (2560, 1440), (3840, 2160), (1366, 768)]
            }
            services::ProfileType::MacOS => {
                vec![(2560, 1440), (2880, 1800), (3840, 2160), (5120, 2880), (1920, 1080)]
            }
            services::ProfileType::Linux => {
                vec![(1920, 1080), (2560, 1440), (3840, 2160)]
            }
            services::ProfileType::Android => {
                vec![(360, 800), (390, 844), (414, 896), (393, 851), (412, 915)]
            }
            services::ProfileType::IOS => {
                vec![(390, 844), (414, 896), (393, 851), (1024, 1366)]
            }
            services::ProfileType::Custom => {
                vec![(1920, 1080)]
            }
        };
        resolutions[rand::random::<usize>() % resolutions.len()]
    }

    /// Generate locale for platform
    #[allow(unused_variables)]
    fn generate_locale(profile_type: services::ProfileType) -> String {
        // Common locales for desktop and mobile platforms
        let locales = ["en-US", "en-GB", "de-DE", "fr-FR", "es-ES", "ja-JP", "zh-CN"];
        locales[rand::random::<usize>() % locales.len()].to_string()
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
        let screen = Self::generate_screen_resolution(services::ProfileType::Windows);
        let locale = Self::generate_locale(services::ProfileType::Windows);

        Ok(Self::create_base_fingerprint(FingerprintConfig {
            platform: "Win32",
            vendor: "Google Inc.",
            user_agent: Self::random_user_agent(services::ProfileType::Windows),
            locale: &locale,
            screen,
            webgl_vendor: Self::random_webgl_vendor(),
            webgl_renderer: Self::random_webgl_renderer(),
            hardware_concurrency: Self::generate_hardware_concurrency(),
            device_memory: Self::generate_device_memory(),
            accept_encoding: "gzip, deflate, br",
        }))
    }

    /// Generate a macOS fingerprint
    async fn generate_macos(&self) -> Result<services::Fingerprint, Error> {
        let screen = Self::generate_screen_resolution(services::ProfileType::MacOS);
        let locale = Self::generate_locale(services::ProfileType::MacOS);

        Ok(Self::create_base_fingerprint(FingerprintConfig {
            platform: "MacIntel",
            vendor: "Google Inc.",
            user_agent: Self::random_user_agent(services::ProfileType::MacOS),
            locale: &locale,
            screen,
            webgl_vendor: Self::random_webgl_vendor(),
            webgl_renderer: Self::random_webgl_renderer(),
            hardware_concurrency: Self::generate_hardware_concurrency(),
            device_memory: Self::generate_device_memory(),
            accept_encoding: "gzip, deflate, br",
        }))
    }

    /// Generate a Linux fingerprint
    async fn generate_linux(&self) -> Result<services::Fingerprint, Error> {
        let screen = Self::generate_screen_resolution(services::ProfileType::Linux);
        let locale = Self::generate_locale(services::ProfileType::Linux);

        Ok(Self::create_base_fingerprint(FingerprintConfig {
            platform: "Linux x86_64",
            vendor: "",
            user_agent: Self::random_user_agent(services::ProfileType::Linux),
            locale: &locale,
            screen,
            webgl_vendor: Self::random_webgl_vendor(),
            webgl_renderer: Self::random_webgl_renderer(),
            hardware_concurrency: Self::generate_hardware_concurrency(),
            device_memory: Self::generate_device_memory(),
            accept_encoding: "gzip, deflate",
        }))
    }

    /// Generate an Android fingerprint
    async fn generate_android(&self) -> Result<services::Fingerprint, Error> {
        let screen = Self::generate_screen_resolution(services::ProfileType::Android);
        let locale = Self::generate_locale(services::ProfileType::Android);

        Ok(Self::create_base_fingerprint(FingerprintConfig {
            platform: "Linux armv8l",
            vendor: "Google Inc.",
            user_agent: Self::random_user_agent(services::ProfileType::Android),
            locale: &locale,
            screen,
            webgl_vendor: "Qualcomm",
            webgl_renderer: "Adreno 740",
            hardware_concurrency: 8,
            device_memory: Some(8),
            accept_encoding: "gzip, deflate, br",
        }))
    }

    /// Generate an iOS fingerprint
    async fn generate_ios(&self) -> Result<services::Fingerprint, Error> {
        let screen = Self::generate_screen_resolution(services::ProfileType::IOS);
        let locale = Self::generate_locale(services::ProfileType::IOS);

        Ok(Self::create_base_fingerprint(FingerprintConfig {
            platform: "iPhone",
            vendor: "Apple Computer, Inc.",
            user_agent: Self::random_user_agent(services::ProfileType::IOS),
            locale: &locale,
            screen,
            webgl_vendor: "Apple Inc.",
            webgl_renderer: "Apple GPU",
            hardware_concurrency: 6,
            device_memory: Some(6),
            accept_encoding: "gzip, deflate, br",
        }))
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
