//! ProfileService gRPC implementation
//!
//! gRPC service for browser profile management.

use std::sync::Arc;
use async_trait::async_trait;
use tonic::{Status};

use crate::Error;
use super::super::traits as services;
use super::super::super::stealth::traits as stealth;
use super::super::super::stealth::traits::FingerprintGenerator;
use super::super::super::stealth::fingerprint::FingerprintGeneratorImpl;

/// ProfileService gRPC implementation
pub struct ProfileServiceImpl {
    /// Profile manager
    profile_manager: Arc<dyn stealth::ProfileManager>,
    /// Stealth engine
    stealth_engine: Arc<dyn stealth::StealthEngine>,
    /// Session manager
    #[allow(dead_code)]
    session_manager: Arc<dyn crate::session::traits::SessionManager>,
}

impl std::fmt::Debug for ProfileServiceImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProfileServiceImpl")
            .field("profile_manager", &"Arc<dyn ProfileManager>")
            .field("stealth_engine", &"Arc<dyn StealthEngine>")
            .field("session_manager", &"Arc<dyn SessionManager>")
            .finish()
    }
}

impl ProfileServiceImpl {
    /// Create a new ProfileService
    pub fn new(
        profile_manager: Arc<dyn stealth::ProfileManager>,
        stealth_engine: Arc<dyn stealth::StealthEngine>,
        session_manager: Arc<dyn crate::session::traits::SessionManager>,
    ) -> Self {
        Self {
            profile_manager,
            stealth_engine,
            session_manager,
        }
    }

    /// Convert Error to Status
    #[allow(dead_code)]
    fn error_to_status(error: Error) -> Status {
        match error {
            Error::SessionNotFound(msg) => Status::not_found(msg),
            Error::BrowserNotFound(msg) => Status::not_found(msg),
            Error::PageNotFound(msg) => Status::not_found(msg),
            Error::ElementNotFound(msg) => Status::not_found(msg),
            Error::Configuration(msg) => Status::invalid_argument(msg),
            Error::Cdp(msg) => Status::internal(msg),
            _ => Status::internal(error.to_string()),
        }
    }
}

#[async_trait]
impl services::ProfileService for ProfileServiceImpl {
    /// Create a profile
    async fn create_profile(
        &self,
        profile_type: services::ProfileType,
    ) -> Result<services::Profile, Error> {
        self.profile_manager.create_profile(profile_type).await
    }

    /// Apply profile to page
    async fn apply_profile(
        &self,
        page_id: &str,
        profile_id: &str,
    ) -> Result<services::AppliedFeatures, Error> {
        // Get profile
        let profile = self.profile_manager.get_profile(profile_id).await?;

        // Apply via stealth engine
        let stealth_features = self
            .stealth_engine
            .apply_profile(page_id, &profile)
            .await?;

        // Convert to services::AppliedFeatures
        let features = services::AppliedFeatures {
            features: stealth_features.features,
        };

        Ok(features)
    }

    /// Get presets
    async fn get_presets(&self) -> Result<Vec<services::ProfilePreset>, Error> {
        self.profile_manager.get_presets().await
    }

    /// Get active profile
    async fn get_active_profile(
        &self,
        _page_id: &str,
    ) -> Result<Option<services::Profile>, Error> {
        // For now, return None
        // In a real implementation, we would track active profiles per page
        Ok(None)
    }

    /// Create custom profile
    async fn create_custom_profile(
        &self,
        options: services::CustomProfileOptions,
    ) -> Result<services::Profile, Error> {
        let generator = FingerprintGeneratorImpl::new();
        let fingerprint = generator.generate_custom(&options.options).await?;

        let profile = services::Profile {
            profile_id: uuid::Uuid::new_v4().to_string(),
            profile_type: services::ProfileType::Custom,
            fingerprint,
        };

        Ok(profile)
    }

    /// Randomize profile
    async fn randomize_profile(&self, profile_id: &str) -> Result<services::Profile, Error> {
        let generator = FingerprintGeneratorImpl::new();

        // Get existing profile
        let profile = self.profile_manager.get_profile(profile_id).await?;

        // Generate new fingerprint based on profile type
        use crate::services::traits::ProfileType;
        let new_fingerprint = match profile.profile_type {
            ProfileType::Windows => generator.generate_windows().await?,
            ProfileType::MacOS => generator.generate_macos().await?,
            ProfileType::Linux => generator.generate_linux().await?,
            ProfileType::Android => generator.generate_android().await?,
            ProfileType::IOS => generator.generate_ios().await?,
            ProfileType::Custom => {
                let default_options = services::CustomOptions {
                    user_agent: None,
                    platform: None,
                    viewport: None,
                };
                generator.generate_custom(&default_options).await?
            },
        };

        // Create new profile
        let new_profile = services::Profile {
            profile_id: uuid::Uuid::new_v4().to_string(),
            profile_type: profile.profile_type,
            fingerprint: new_fingerprint,
        };

        Ok(new_profile)
    }
}
