//! Profile manager implementation
//!
//! Manages browser profiles and their fingerprints.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use uuid::Uuid;

use crate::Error;
use super::super::traits as services;
use super::super::super::stealth::traits as stealth;

/// Profile manager implementation
pub struct ProfileManagerImpl {
    /// Fingerprint generator
    generator: Arc<dyn stealth::FingerprintGenerator>,
    /// Profile storage
    profiles: Arc<RwLock<HashMap<String, services::Profile>>>,
}

impl ProfileManagerImpl {
    /// Create a new profile manager
    pub fn new(generator: Arc<dyn stealth::FingerprintGenerator>) -> Self {
        Self {
            generator,
            profiles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate profile ID
    fn generate_id() -> String {
        Uuid::new_v4().to_string()
    }
}

#[async_trait]
impl stealth::ProfileManager for ProfileManagerImpl {
    /// Create a new profile
    async fn create_profile(
        &self,
        profile_type: services::ProfileType,
    ) -> Result<services::Profile, Error> {
        let fingerprint = match profile_type {
            services::ProfileType::Windows => self.generator.generate_windows().await?,
            services::ProfileType::MacOS => self.generator.generate_macos().await?,
            services::ProfileType::Linux => self.generator.generate_linux().await?,
            services::ProfileType::Android => self.generator.generate_android().await?,
            services::ProfileType::IOS => self.generator.generate_ios().await?,
            services::ProfileType::Custom => {
                return Err(Error::Configuration(
                    "Use create_custom_profile for custom profiles".to_string(),
                ))
            }
        };

        let profile = services::Profile {
            profile_id: Self::generate_id(),
            profile_type,
            fingerprint,
        };

        // Store profile
        let mut profiles = self.profiles.write().await;
        profiles.insert(profile.profile_id.clone(), profile.clone());

        Ok(profile)
    }

    /// Get a profile by ID
    async fn get_profile(&self, profile_id: &str) -> Result<services::Profile, Error> {
        let profiles = self.profiles.read().await;
        profiles
            .get(profile_id)
            .cloned()
            .ok_or_else(|| Error::SessionNotFound(format!("Profile not found: {}", profile_id)))
    }

    /// List all profiles
    async fn list_profiles(&self) -> Result<Vec<services::Profile>, Error> {
        let profiles = self.profiles.read().await;
        Ok(profiles.values().cloned().collect())
    }

    /// Delete a profile
    async fn delete_profile(&self, profile_id: &str) -> Result<(), Error> {
        let mut profiles = self.profiles.write().await;
        profiles
            .remove(profile_id)
            .ok_or_else(|| Error::SessionNotFound(format!("Profile not found: {}", profile_id)))?;
        Ok(())
    }

    /// Update a profile
    async fn update_profile(
        &self,
        profile_id: &str,
        fingerprint: services::Fingerprint,
    ) -> Result<(), Error> {
        let mut profiles = self.profiles.write().await;
        let profile = profiles
            .get_mut(profile_id)
            .ok_or_else(|| Error::SessionNotFound(format!("Profile not found: {}", profile_id)))?;

        profile.fingerprint = fingerprint;
        Ok(())
    }

    /// Get preset profiles
    async fn get_presets(&self) -> Result<Vec<services::ProfilePreset>, Error> {
        Ok(vec![
            services::ProfilePreset {
                name: "Windows Chrome".to_string(),
                profile_type: services::ProfileType::Windows,
                description: "Windows 10 with Chrome browser".to_string(),
            },
            services::ProfilePreset {
                name: "macOS Safari".to_string(),
                profile_type: services::ProfileType::MacOS,
                description: "macOS with Safari browser".to_string(),
            },
            services::ProfilePreset {
                name: "Linux Firefox".to_string(),
                profile_type: services::ProfileType::Linux,
                description: "Linux with Firefox browser".to_string(),
            },
            services::ProfilePreset {
                name: "Android Chrome".to_string(),
                profile_type: services::ProfileType::Android,
                description: "Android mobile with Chrome".to_string(),
            },
            services::ProfilePreset {
                name: "iOS Safari".to_string(),
                profile_type: services::ProfileType::IOS,
                description: "iOS mobile with Safari".to_string(),
            },
        ])
    }
}
