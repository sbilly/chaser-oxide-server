//! Stealth engine traits
//!
//! This module defines the abstract interfaces for browser fingerprinting evasion
//! and human behavior simulation.

use async_trait::async_trait;
use super::super::services::traits as services;

// ============================================================================
// Stealth Engine
// ============================================================================

/// Stealth engine trait
///
/// Core engine for managing browser fingerprinting evasion.
#[async_trait]
pub trait StealthEngine: Send + Sync {
    /// Apply a profile to a page
    async fn apply_profile(
        &self,
        page_id: &str,
        profile: &services::Profile,
    ) -> Result<AppliedFeatures, crate::Error>;

    /// Set User-Agent at CDP protocol level
    async fn set_user_agent(&self, page_id: &str, user_agent: &str) -> Result<(), crate::Error>;

    /// Inject navigator overrides
    async fn inject_navigator(
        &self,
        page_id: &str,
        fingerprint: &services::NavigatorFingerprint,
    ) -> Result<(), crate::Error>;

    /// Inject screen overrides
    async fn inject_screen(
        &self,
        page_id: &str,
        fingerprint: &services::ScreenFingerprint,
    ) -> Result<(), crate::Error>;

    /// Inject WebGL protection
    async fn inject_webgl(
        &self,
        page_id: &str,
        fingerprint: &services::WebGLFingerprint,
    ) -> Result<(), crate::Error>;

    /// Inject canvas protection
    async fn inject_canvas(&self, page_id: &str) -> Result<(), crate::Error>;

    /// Inject audio protection
    async fn inject_audio(&self, page_id: &str) -> Result<(), crate::Error>;

    /// Get applied features
    async fn get_applied_features(&self, page_id: &str) -> Result<AppliedFeatures, crate::Error>;

    /// Remove all injections
    async fn remove_all(&self, page_id: &str) -> Result<(), crate::Error>;
}

/// Applied features
#[derive(Debug, Clone)]
pub struct AppliedFeatures {
    pub features: Vec<String>,
}

// ============================================================================
// Script Injector
// ============================================================================

/// Script injector trait
///
/// Handles injection of JavaScript code for fingerprint modification.
#[async_trait]
pub trait ScriptInjector: Send + Sync {
    /// Inject JavaScript before page load
    async fn inject_init_script(&self, page_id: &str, script: &str) -> Result<(), crate::Error>;

    /// Evaluate JavaScript in the page
    async fn evaluate(&self, page_id: &str, script: &str) -> Result<String, crate::Error>;

    /// Inject CSS
    async fn inject_style(&self, page_id: &str, css: &str) -> Result<(), crate::Error>;

    /// Set User-Agent at CDP protocol level
    async fn set_user_agent(&self, page_id: &str, user_agent: &str) -> Result<(), crate::Error>;

    /// Get all injected scripts
    async fn get_injected_scripts(&self, page_id: &str) -> Result<Vec<InjectedScript>, crate::Error>;

    /// Remove injected script
    async fn remove_script(&self, page_id: &str, script_id: &str) -> Result<(), crate::Error>;

    /// Clear all injected scripts
    async fn clear_all(&self, page_id: &str) -> Result<(), crate::Error>;
}

/// Injected script information
#[derive(Debug, Clone)]
pub struct InjectedScript {
    pub script_id: String,
    pub script_type: ScriptType,
    pub content: String,
}

/// Script type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScriptType {
    InitScript,
    EvaluatedScript,
    Style,
}

// ============================================================================
// Behavior Simulator
// ============================================================================

/// Behavior simulator trait
///
/// Simulates human-like behavior patterns to evade detection.
#[async_trait]
pub trait BehaviorSimulator: Send + Sync {
    /// Simulate mouse movement using Bezier curves
    async fn simulate_mouse_move(
        &self,
        page_id: &str,
        start: (f64, f64),
        end: (f64, f64),
        options: MouseMoveOptions,
    ) -> Result<(), crate::Error>;

    /// Simulate human-like typing
    async fn simulate_typing(
        &self,
        page_id: &str,
        element_id: &str,
        text: &str,
        options: TypingOptions,
    ) -> Result<(), crate::Error>;

    /// Simulate human-like clicking
    async fn simulate_click(
        &self,
        page_id: &str,
        element_id: &str,
        options: ClickOptions,
    ) -> Result<(), crate::Error>;

    /// Simulate scroll behavior
    async fn simulate_scroll(
        &self,
        page_id: &str,
        target_y: f64,
        options: ScrollOptions,
    ) -> Result<(), crate::Error>;

    /// Add random delay
    async fn random_delay(&self, min_ms: u64, max_ms: u64) -> Result<(), crate::Error>;
}

/// Mouse movement options
#[derive(Debug, Clone)]
pub struct MouseMoveOptions {
    /// Movement duration in milliseconds
    pub duration_ms: u64,
    /// Bezier curve control points deviation
    pub deviation: f64,
    /// Number of intermediate points
    pub points: u32,
}

impl Default for MouseMoveOptions {
    fn default() -> Self {
        Self {
            duration_ms: 500,
            deviation: 50.0,
            points: 20,
        }
    }
}

/// Typing options
#[derive(Debug, Clone)]
pub struct TypingOptions {
    /// Mean typing delay in milliseconds
    pub mean_delay_ms: u64,
    /// Standard deviation for typing delay
    pub std_dev_ms: u64,
    /// Probability of making a typo
    pub typo_probability: f64,
    /// Probability of using backspace
    pub backspace_probability: f64,
}

impl Default for TypingOptions {
    fn default() -> Self {
        Self {
            mean_delay_ms: 100,
            std_dev_ms: 50,
            typo_probability: 0.02,
            backspace_probability: 0.01,
        }
    }
}

/// Click options
#[derive(Debug, Clone)]
pub struct ClickOptions {
    /// Delay before click in milliseconds
    pub delay_before_ms: u64,
    /// Mouse movement duration
    pub movement_duration_ms: u64,
    /// Hold duration in milliseconds
    pub hold_duration_ms: u64,
}

impl Default for ClickOptions {
    fn default() -> Self {
        Self {
            delay_before_ms: 100,
            movement_duration_ms: 300,
            hold_duration_ms: 50,
        }
    }
}

/// Scroll options
#[derive(Debug, Clone)]
pub struct ScrollOptions {
    /// Scroll duration in milliseconds
    pub duration_ms: u64,
    /// Number of scroll steps
    pub steps: u32,
    /// Add random acceleration
    pub acceleration: bool,
}

impl Default for ScrollOptions {
    fn default() -> Self {
        Self {
            duration_ms: 1000,
            steps: 10,
            acceleration: true,
        }
    }
}

// ============================================================================
// Fingerprint Generator
// ============================================================================

/// Fingerprint generator trait
///
/// Generates realistic browser fingerprints.
#[async_trait]
pub trait FingerprintGenerator: Send + Sync {
    /// Generate a Windows fingerprint
    async fn generate_windows(&self) -> Result<services::Fingerprint, crate::Error>;

    /// Generate a macOS fingerprint
    async fn generate_macos(&self) -> Result<services::Fingerprint, crate::Error>;

    /// Generate a Linux fingerprint
    async fn generate_linux(&self) -> Result<services::Fingerprint, crate::Error>;

    /// Generate an Android fingerprint
    async fn generate_android(&self) -> Result<services::Fingerprint, crate::Error>;

    /// Generate an iOS fingerprint
    async fn generate_ios(&self) -> Result<services::Fingerprint, crate::Error>;

    /// Generate a custom fingerprint
    async fn generate_custom(
        &self,
        options: &services::CustomOptions,
    ) -> Result<services::Fingerprint, crate::Error>;

    /// Randomize an existing fingerprint
    async fn randomize(
        &self,
        fingerprint: &services::Fingerprint,
    ) -> Result<services::Fingerprint, crate::Error>;
}

// ============================================================================
// Profile Manager
// ============================================================================

/// Profile manager trait
///
/// Manages browser profiles and their fingerprints.
#[async_trait]
pub trait ProfileManager: Send + Sync {
    /// Create a new profile
    async fn create_profile(
        &self,
        profile_type: services::ProfileType,
    ) -> Result<services::Profile, crate::Error>;

    /// Get a profile by ID
    async fn get_profile(&self, profile_id: &str) -> Result<services::Profile, crate::Error>;

    /// List all profiles
    async fn list_profiles(&self) -> Result<Vec<services::Profile>, crate::Error>;

    /// Delete a profile
    async fn delete_profile(&self, profile_id: &str) -> Result<(), crate::Error>;

    /// Update a profile
    async fn update_profile(
        &self,
        profile_id: &str,
        fingerprint: services::Fingerprint,
    ) -> Result<(), crate::Error>;

    /// Get preset profiles
    async fn get_presets(&self) -> Result<Vec<services::ProfilePreset>, crate::Error>;
}
