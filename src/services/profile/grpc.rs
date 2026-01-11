//! Profile service gRPC implementation
//!
//! This module provides the gRPC implementation for profile management.

use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{info, instrument};
use rand::seq::SliceRandom;

use crate::services::profile::service::ProfileServiceImpl;
use crate::services::traits as services;
use crate::services::traits::ProfileService; // Import trait to use methods

// Import generated proto types
use crate::chaser_oxide::v1::{
    profile_service_server::ProfileService as ProtoProfileService,
    create_profile_response::Response as CreateProfileResponseEnum,
    apply_profile_response::Response as ApplyProfileResponseEnum,
    get_presets_response::Response as GetPresetsResponseEnum,
    get_active_profile_response::Response as GetActiveProfileResponseEnum,
    create_custom_profile_response::Response as CreateCustomProfileResponseEnum,
    randomize_profile_response::Response as RandomizeProfileResponseEnum,
    CreateProfileRequest, CreateProfileResponse,
    ApplyProfileRequest, ApplyProfileResponse,
    GetPresetsRequest, GetPresetsResponse,
    GetActiveProfileRequest, GetActiveProfileResponse,
    CreateCustomProfileRequest, CreateCustomProfileResponse,
    RandomizeProfileRequest, RandomizeProfileResponse,
    ProfileType as ProtoProfileType,
    Profile as ProtoProfile,
    Fingerprint as ProtoFingerprint,
    HttpHeaders as ProtoHttpHeaders,
    NavigatorInfo as ProtoNavigatorInfo,
    ScreenInfo as ProtoScreenInfo,
    HardwareInfo as ProtoHardwareInfo,
    WebGlInfo as ProtoWebGlInfo,
    AudioInfo as ProtoAudioInfo,
    CanvasInfo as ProtoCanvasInfo,
    Permissions as ProtoPermissions,
    ProfileOptions as ProtoProfileOptions,
    CustomProfileOptions as ProtoCustomProfileOptions,
    ApplyProfileResult as ProtoApplyProfileResult,
    PresetProfiles as ProtoPresetProfiles,
    Error as ProtoError,
    ErrorCode,
};

/// Profile service gRPC wrapper
#[derive(Debug, Clone)]
pub struct ProfileServiceGrpc {
    inner: Arc<ProfileServiceImpl>,
}

impl ProfileServiceGrpc {
    /// Create a new profile service gRPC wrapper
    pub fn new(service: Arc<ProfileServiceImpl>) -> Self {
        Self { inner: service }
    }

    /// Convert Error to Status
    fn error_to_status(error: crate::Error) -> Status {
        let code = match &error {
            crate::Error::SessionNotFound(_) => ErrorCode::NotFound,
            crate::Error::BrowserNotFound(_) => ErrorCode::NotFound,
            crate::Error::PageNotFound(_) => ErrorCode::NotFound,
            crate::Error::ElementNotFound(_) => ErrorCode::NotFound,
            crate::Error::Configuration(_) => ErrorCode::InvalidArgument,
            _ => ErrorCode::Internal,
        };

        let proto_error = ProtoError {
            code: code.into(),
            message: error.to_string(),
            details: std::collections::HashMap::new(),
        };

        Status::unknown(proto_error.message)
    }

    /// Convert proto ProfileType (i32) to internal
    fn proto_to_profile_type(profile_type: i32) -> services::ProfileType {
        match profile_type {
            1 => services::ProfileType::Windows,   // PROFILE_TYPE_WINDOWS
            3 => services::ProfileType::Linux,     // PROFILE_TYPE_LINUX
            2 => services::ProfileType::MacOS,     // PROFILE_TYPE_MACOS
            4 => services::ProfileType::Android,   // PROFILE_TYPE_ANDROID
            5 => services::ProfileType::IOS,       // PROFILE_TYPE_IOS
            6 => services::ProfileType::Custom,    // PROFILE_TYPE_CUSTOM
            _ => services::ProfileType::Windows,   // Default (PROFILE_TYPE_UNSPECIFIED = 0)
        }
    }

    /// Convert internal Profile to proto
    fn profile_to_proto(profile: services::Profile) -> ProtoProfile {
        let fingerprint_clone = profile.fingerprint.clone();
        ProtoProfile {
            profile_id: profile.profile_id,
            r#type: match profile.profile_type {
                services::ProfileType::Windows => ProtoProfileType::Windows,
                services::ProfileType::MacOS => ProtoProfileType::Macos,
                services::ProfileType::Linux => ProtoProfileType::Linux,
                services::ProfileType::Android => ProtoProfileType::Android,
                services::ProfileType::IOS => ProtoProfileType::Ios,
                services::ProfileType::Custom => ProtoProfileType::Custom,
            } as i32,
            fingerprint: Some(Self::fingerprint_to_proto(profile.fingerprint)),
            options: Some(Self::profile_options_to_proto(fingerprint_clone.options)),
            created_at: 0, // Not tracked in internal struct yet
        }
    }

    /// Convert internal Fingerprint to proto
    fn fingerprint_to_proto(fingerprint: services::Fingerprint) -> ProtoFingerprint {
        let language = fingerprint.navigator.language.clone();
        ProtoFingerprint {
            headers: Some(ProtoHttpHeaders {
                user_agent: fingerprint.headers.user_agent,
                accept_language: fingerprint.headers.accept_language,
                accept_encoding: fingerprint.headers.accept_encoding,
                sec_ch_ua: String::new(),      // Not in internal struct
                sec_ch_ua_platform: String::new(),
                sec_ch_ua_mobile: String::new(),
                sec_ch_ua_arch: String::new(),
            }),
            navigator: Some(ProtoNavigatorInfo {
                platform: fingerprint.navigator.platform,
                vendor: fingerprint.navigator.vendor,
                product: String::new(),      // Not in internal struct
                app_version: String::new(),  // Not in internal struct
                app_name: String::new(),     // Not in internal struct
                app_code_name: String::new(), // Not in internal struct
                hardware_concurrency: fingerprint.navigator.hardware_concurrency > 0, // Convert u32 to bool
                device_memory: fingerprint.navigator.device_memory.unwrap_or(8) as i32, // Option<u32> to i32
                language: language.clone(),
                languages: vec![language.clone()], // Simplified
                do_not_track: false, // Not in internal struct
                cookie_enabled: String::new(), // Not in internal struct
                pdf_viewer_enabled: false,     // Not in internal struct
                webdriver: false,              // Should be false for stealth
            }),
            screen: Some(ProtoScreenInfo {
                width: fingerprint.screen.width as i32,
                height: fingerprint.screen.height as i32,
                avail_width: fingerprint.screen.width as i32,  // Same as width
                avail_height: fingerprint.screen.height as i32, // Same as height
                color_depth: fingerprint.screen.color_depth as i32,
                pixel_depth: fingerprint.screen.pixel_depth as i32,
                top: 0,
                left: 0,
                device_pixel_ratio: 1.0,
                orientation: true, // Default
            }),
            hardware: Some(ProtoHardwareInfo {
                cpu_cores: fingerprint.navigator.hardware_concurrency as i32,
                device_memory: fingerprint.navigator.device_memory.unwrap_or(8) as i32,
                gpu_vendor: fingerprint.webgl.vendor.clone(),
                gpu_renderer: fingerprint.webgl.renderer.clone(),
            }),
            webgl: Some(ProtoWebGlInfo {
                vendor: fingerprint.webgl.vendor,
                renderer: fingerprint.webgl.renderer,
                version: String::new(), // Not in internal struct
                shading_language_version: String::new(), // Not in internal struct
                max_texture_size: 0,    // Not in internal struct
                extensions: vec![],
                webgl2: false,          // Not in internal struct
            }),
            audio: Some(ProtoAudioInfo {
                context_count: 0,       // Not in internal struct
                audio_context: String::new(), // Not in internal struct
            }),
            canvas: Some(ProtoCanvasInfo {
                canvas_fingerprint: false, // Not in internal struct
                canvas_renderer: String::new(), // Not in internal struct
            }),
            timezone: String::new(),  // Not in internal struct
            locale: language.clone(), // Use navigator language
            languages: vec![language], // Simplified
            permissions: Some(ProtoPermissions {
                geolocation: false,
                notifications: false,
                camera: false,
                microphone: false,
                midi: false,
                bluetooth: false,
                usb: false,
            }),
        }
    }

    /// Convert internal ProfileOptions to proto
    fn profile_options_to_proto(options: services::ProfileOptions) -> ProtoProfileOptions {
        ProtoProfileOptions {
            inject_navigator: options.inject_navigator,
            inject_screen: options.inject_screen,
            inject_webgl: options.inject_webgl,
            inject_canvas: options.inject_canvas,
            inject_audio: options.inject_audio,
            neutralize_utility_world: false, // Not in internal struct
            use_isolated_world: false,       // Not in internal struct
            randomize_metrics: false,        // Not in internal struct
            prevent_detection: false,        // Not in internal struct
        }
    }

    /// Convert proto CustomProfileOptions to internal
    fn proto_to_custom_options(opts: ProtoCustomProfileOptions, template: i32) -> services::CustomProfileOptions {
        services::CustomProfileOptions {
            profile_name: String::new(), // Not provided in proto
            template: Self::proto_to_profile_type(template),
            options: services::CustomOptions {
                user_agent: if opts.user_agent.is_empty() { None } else { Some(opts.user_agent) },
                platform: if opts.platform.is_empty() { None } else { Some(opts.platform) },
                viewport: if opts.screen_width == 0 && opts.screen_height == 0 {
                    None
                } else {
                    Some(services::Viewport {
                        width: opts.screen_width as u32,
                        height: opts.screen_height as u32,
                        device_scale_factor: opts.device_pixel_ratio,
                    })
                },
            },
            profile_options: services::ProfileOptions {
                inject_navigator: true,
                inject_screen: true,
                inject_webgl: true,
                inject_canvas: true,
                inject_audio: true,
            },
        }
    }
}

#[tonic::async_trait]
impl ProtoProfileService for ProfileServiceGrpc {
    #[instrument(skip(self))]
    async fn create_profile(
        &self,
        request: Request<CreateProfileRequest>,
    ) -> Result<Response<CreateProfileResponse>, Status> {
        info!("CreateProfile request received");

        let req = request.into_inner();

        // Convert proto type to internal type
        let profile_type = Self::proto_to_profile_type(req.r#type);

        match self.inner.create_profile(profile_type).await {
            Ok(profile) => {
                let response = CreateProfileResponse {
                    response: Some(CreateProfileResponseEnum::Profile(Self::profile_to_proto(profile))),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                let response = CreateProfileResponse {
                    response: Some(CreateProfileResponseEnum::Error(ProtoError {
                        code: ErrorCode::Internal.into(),
                        message: e.to_string(),
                        details: std::collections::HashMap::new(),
                    })),
                };
                Ok(Response::new(response))
            }
        }
    }

    #[instrument(skip(self))]
    async fn apply_profile(
        &self,
        request: Request<ApplyProfileRequest>,
    ) -> Result<Response<ApplyProfileResponse>, Status> {
        info!("ApplyProfile request received");

        let req = request.into_inner();

        match self.inner.apply_profile(&req.page_id, &req.profile_id).await {
            Ok(features) => {
                let result = ProtoApplyProfileResult {
                    page_id: req.page_id,
                    profile_id: req.profile_id,
                    success: true,
                    applied_features: features.features,
                };
                let response = ApplyProfileResponse {
                    response: Some(ApplyProfileResponseEnum::Result(result)),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                let error = Self::error_to_status(e);
                let response = ApplyProfileResponse {
                    response: Some(ApplyProfileResponseEnum::Error(ProtoError {
                        code: ErrorCode::Internal.into(),
                        message: error.message().to_string(),
                        details: std::collections::HashMap::new(),
                    })),
                };
                Ok(Response::new(response))
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_presets(
        &self,
        _request: Request<GetPresetsRequest>,
    ) -> Result<Response<GetPresetsResponse>, Status> {
        info!("GetPresets request received");

        match self.inner.get_presets().await {
            Ok(presets) => {
                // Convert ProfilePreset to full Profile objects by generating fingerprints
                let profiles_proto: Vec<ProtoProfile> = futures::future::join_all(presets.into_iter().map(|preset| {
                    let inner = self.inner.clone();
                    async move {
                        // Create a profile from the preset type
                        match inner.create_profile(preset.profile_type).await {
                            Ok(mut profile) => {
                                // Update profile_id to indicate it's a preset
                                profile.profile_id = format!("preset-{}", preset.name.to_lowercase().replace(' ', "-"));
                                Ok::<ProtoProfile, tonic::Status>(Self::profile_to_proto(profile))
                            }
                            Err(_) => {
                                // Fallback: return minimal profile if generation fails
                                Ok::<ProtoProfile, tonic::Status>(ProtoProfile {
                                    profile_id: format!("preset-{}", preset.name.to_lowercase().replace(' ', "-")),
                                    r#type: match preset.profile_type {
                                        services::ProfileType::Windows => ProtoProfileType::Windows,
                                        services::ProfileType::MacOS => ProtoProfileType::Macos,
                                        services::ProfileType::Linux => ProtoProfileType::Linux,
                                        services::ProfileType::Android => ProtoProfileType::Android,
                                        services::ProfileType::IOS => ProtoProfileType::Ios,
                                        services::ProfileType::Custom => ProtoProfileType::Custom,
                                    } as i32,
                                    fingerprint: None,
                                    options: None,
                                    created_at: 0,
                                })
                            }
                        }
                    }
                }))
                .await
                .into_iter()
                .collect::<Result<Vec<_>, tonic::Status>>()?;

                let response = GetPresetsResponse {
                    response: Some(GetPresetsResponseEnum::Presets(ProtoPresetProfiles {
                        presets: profiles_proto,
                    })),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                let error = Self::error_to_status(e);
                let response = GetPresetsResponse {
                    response: Some(GetPresetsResponseEnum::Error(ProtoError {
                        code: ErrorCode::Internal.into(),
                        message: error.message().to_string(),
                        details: std::collections::HashMap::new(),
                    })),
                };
                Ok(Response::new(response))
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_active_profile(
        &self,
        request: Request<GetActiveProfileRequest>,
    ) -> Result<Response<GetActiveProfileResponse>, Status> {
        info!("GetActiveProfile request received");

        let req = request.into_inner();

        match self.inner.get_active_profile(&req.page_id).await {
            Ok(Some(profile)) => {
                let response = GetActiveProfileResponse {
                    response: Some(GetActiveProfileResponseEnum::Profile(Self::profile_to_proto(profile))),
                };
                Ok(Response::new(response))
            }
            Ok(None) => {
                let response = GetActiveProfileResponse {
                    response: Some(GetActiveProfileResponseEnum::Error(ProtoError {
                        code: ErrorCode::NotFound.into(),
                        message: "No active profile found".to_string(),
                        details: std::collections::HashMap::new(),
                    })),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                let error = Self::error_to_status(e);
                let response = GetActiveProfileResponse {
                    response: Some(GetActiveProfileResponseEnum::Error(ProtoError {
                        code: ErrorCode::Internal.into(),
                        message: error.message().to_string(),
                        details: std::collections::HashMap::new(),
                    })),
                };
                Ok(Response::new(response))
            }
        }
    }

    #[instrument(skip(self))]
    async fn create_custom_profile(
        &self,
        request: Request<CreateCustomProfileRequest>,
    ) -> Result<Response<CreateCustomProfileResponse>, Status> {
        info!("CreateCustomProfile request received");

        let req = request.into_inner();

        // Convert proto options to internal options
        let opts = req.options.unwrap_or_default();
        let custom_options = Self::proto_to_custom_options(opts, req.template);

        match self.inner.create_custom_profile(custom_options).await {
            Ok(profile) => {
                let response = CreateCustomProfileResponse {
                    response: Some(CreateCustomProfileResponseEnum::Profile(Self::profile_to_proto(profile))),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                let error = Self::error_to_status(e);
                let response = CreateCustomProfileResponse {
                    response: Some(CreateCustomProfileResponseEnum::Error(ProtoError {
                        code: ErrorCode::Internal.into(),
                        message: error.message().to_string(),
                        details: std::collections::HashMap::new(),
                    })),
                };
                Ok(Response::new(response))
            }
        }
    }

    #[instrument(skip(self))]
    async fn randomize_profile(
        &self,
        request: Request<RandomizeProfileRequest>,
    ) -> Result<Response<RandomizeProfileResponse>, Status> {
        info!("RandomizeProfile request received");

        let req = request.into_inner();

        // Get randomization options (default to all false if not provided)
        let options = req.options.unwrap_or_default();

        // Create a profile of the specified type
        let profile_type = Self::proto_to_profile_type(req.r#type);

        match self.inner.create_profile(profile_type).await {
            Ok(mut profile) => {
                // Apply randomization based on options
                if options.randomize_screen {
                    // Randomize screen resolution within typical ranges for the profile type
                    let resolutions = match profile_type {
                        services::ProfileType::Windows => vec![(1920, 1080), (2560, 1440), (3840, 2160), (1366, 768)],
                        services::ProfileType::MacOS => vec![(2560, 1440), (2880, 1800), (1920, 1080), (1680, 1050)],
                        services::ProfileType::Linux => vec![(1920, 1080), (2560, 1440), (3840, 2160), (1366, 768)],
                        services::ProfileType::Android => vec![(360, 640), (412, 915), (1080, 2400), (1440, 3200)],
                        services::ProfileType::IOS => vec![(375, 667), (414, 896), (390, 844), (428, 926)],
                        services::ProfileType::Custom => vec![(1920, 1080)],
                    };

                    if let Some(random_res) = resolutions.choose(&mut rand::thread_rng()) {
                        profile.fingerprint.screen.width = random_res.0;
                        profile.fingerprint.screen.height = random_res.1;
                    }
                }

                if options.randomize_webgl {
                    // Randomize WebGL parameters
                    let vendors = ["Intel Inc.", "NVIDIA Corporation", "AMD", "Qualcomm"];
                    let renderers = [
                        "Intel Iris OpenGL Engine",
                        "NVIDIA GeForce GTX 1060",
                        "AMD Radeon RX 580",
                        "Intel UHD Graphics 620",
                    ];

                    if let Some(random_vendor) = vendors.choose(&mut rand::thread_rng()) {
                        profile.fingerprint.webgl.vendor = random_vendor.to_string();
                    }
                    if let Some(random_renderer) = renderers.choose(&mut rand::thread_rng()) {
                        profile.fingerprint.webgl.renderer = random_renderer.to_string();
                    }
                }

                if options.randomize_language {
                    // Randomize language variants
                    let languages = match profile_type {
                        services::ProfileType::Windows => vec!["en-US", "en-GB", "de-DE", "fr-FR", "es-ES"],
                        services::ProfileType::MacOS => vec!["en-US", "en-GB", "ja-JP", "zh-CN", "fr-FR"],
                        services::ProfileType::Linux => vec!["en-US", "de-DE", "ru-RU", "pt-BR", "en-GB"],
                        services::ProfileType::Android => vec!["en-US", "zh-CN", "es-ES", "hi-IN", "pt-BR"],
                        services::ProfileType::IOS => vec!["en-US", "zh-CN", "ja-JP", "fr-FR", "de-DE"],
                        services::ProfileType::Custom => vec!["en-US"],
                    };

                    if let Some(random_lang) = languages.choose(&mut rand::thread_rng()) {
                        profile.fingerprint.navigator.language = random_lang.to_string();
                    }
                }

                // Note: Timezone randomization would require additional data structures
                // that are not currently in the Fingerprint struct
                if options.randomize_timezone {
                    // Placeholder for future implementation
                    tracing::warn!("Timezone randomization requested but not yet implemented");
                }

                let response = RandomizeProfileResponse {
                    response: Some(RandomizeProfileResponseEnum::Profile(Self::profile_to_proto(profile))),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                let error = Self::error_to_status(e);
                let response = RandomizeProfileResponse {
                    response: Some(RandomizeProfileResponseEnum::Error(ProtoError {
                        code: ErrorCode::Internal.into(),
                        message: error.message().to_string(),
                        details: std::collections::HashMap::new(),
                    })),
                };
                Ok(Response::new(response))
            }
        }
    }
}
