//! Stealth engine implementation
//!
//! Core engine for managing browser fingerprinting evasion and stealth capabilities.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

use crate::Error;
use super::traits::*;
use super::super::services::traits as services;

/// Stealth engine implementation
pub struct StealthEngineImpl {
    /// Script injector
    injector: Arc<dyn ScriptInjector>,
    /// Behavior simulator
    #[allow(dead_code)]
    simulator: Arc<dyn BehaviorSimulator>,
    /// Applied features tracking
    applied_features: Arc<RwLock<HashMap<String, AppliedFeatures>>>,
}

impl StealthEngineImpl {
    /// Create a new stealth engine
    pub fn new(
        injector: Arc<dyn ScriptInjector>,
        simulator: Arc<dyn BehaviorSimulator>,
    ) -> Self {
        Self {
            injector,
            simulator,
            applied_features: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl StealthEngine for StealthEngineImpl {
    /// Apply a profile to a page
    async fn apply_profile(
        &self,
        page_id: &str,
        profile: &services::Profile,
    ) -> Result<AppliedFeatures, Error> {
        let mut applied = Vec::new();

        // Set User-Agent at CDP protocol level FIRST (before any other operations)
        self.set_user_agent(page_id, &profile.fingerprint.headers.user_agent).await?;
        applied.push("user_agent".to_string());

        // Apply navigator overrides
        if profile.fingerprint.options.inject_navigator {
            self.inject_navigator(page_id, &profile.fingerprint.navigator).await?;
            applied.push("navigator".to_string());
        }

        // Apply screen overrides
        if profile.fingerprint.options.inject_screen {
            self.inject_screen(page_id, &profile.fingerprint.screen).await?;
            applied.push("screen".to_string());
        }

        // Apply WebGL protection
        if profile.fingerprint.options.inject_webgl {
            self.inject_webgl(page_id, &profile.fingerprint.webgl).await?;
            applied.push("webgl".to_string());
        }

        // Apply canvas protection
        if profile.fingerprint.options.inject_canvas {
            self.inject_canvas(page_id).await?;
            applied.push("canvas".to_string());
        }

        // Apply audio protection
        if profile.fingerprint.options.inject_audio {
            self.inject_audio(page_id).await?;
            applied.push("audio".to_string());
        }

        let features = AppliedFeatures { features: applied };

        // Track applied features
        let mut tracker = self.applied_features.write().await;
        tracker.insert(page_id.to_string(), features.clone());

        Ok(features)
    }

    /// Set User-Agent at CDP protocol level
    async fn set_user_agent(&self, page_id: &str, user_agent: &str) -> Result<(), Error> {
        self.injector.set_user_agent(page_id, user_agent).await
    }

    /// Inject navigator overrides
    async fn inject_navigator(
        &self,
        page_id: &str,
        fingerprint: &services::NavigatorFingerprint,
    ) -> Result<(), Error> {
        tracing::debug!(
            "[P5-DEBUG] Injecting navigator for page {}: platform={}, vendor={}, cores={}, memory={:?}, lang={}",
            page_id,
            fingerprint.platform,
            fingerprint.vendor,
            fingerprint.hardware_concurrency,
            fingerprint.device_memory,
            fingerprint.language
        );

        let device_memory = fingerprint.device_memory.unwrap_or(8);
        let script = format!(
            r#"(function() {{
                Object.defineProperty(navigator, 'platform', {{ get: () => '{}' }});
                Object.defineProperty(navigator, 'vendor', {{ get: () => '{}' }});
                Object.defineProperty(navigator, 'hardwareConcurrency', {{ get: () => {} }});
                Object.defineProperty(navigator, 'deviceMemory', {{ get: () => {} }});
                Object.defineProperty(navigator, 'language', {{ get: () => '{}' }});
                Object.defineProperty(navigator, 'webdriver', {{ get: () => false }});
                Object.defineProperty(navigator, 'plugins', {{ get: () => [
                    {{
                        0: {{ type: "application/x-google-chrome-pdf", suffixes: "pdf", description: "Portable Document Format" }},
                        description: "Portable Document Format",
                        filename: "internal-pdf-viewer",
                        length: 1,
                        name: "Chrome PDF Plugin"
                    }},
                    {{
                        0: {{ type: "application/pdf", suffixes: "pdf", description: "" }},
                        description: "",
                        filename: "mhjfbmdgcfjbbpaeojofohoefgiehjai",
                        length: 1,
                        name: "Chrome PDF Viewer"
                    }}
                ]}});
            }})();"#,
            fingerprint.platform, fingerprint.vendor, fingerprint.hardware_concurrency, device_memory, fingerprint.language
        );

        self.injector.inject_init_script(page_id, &script).await
    }

    /// Inject screen overrides
    async fn inject_screen(
        &self,
        page_id: &str,
        fingerprint: &services::ScreenFingerprint,
    ) -> Result<(), Error> {
        let avail_height = fingerprint.height - 40;
        let script = format!(
            r#"(function() {{
                Object.defineProperty(screen, 'width', {{ get: () => {} }});
                Object.defineProperty(screen, 'height', {{ get: () => {} }});
                Object.defineProperty(screen, 'colorDepth', {{ get: () => {} }});
                Object.defineProperty(screen, 'pixelDepth', {{ get: () => {} }});
                Object.defineProperty(screen, 'availWidth', {{ get: () => {} }});
                Object.defineProperty(screen, 'availHeight', {{ get: () => {} }});
                Object.defineProperty(window, 'devicePixelRatio', {{ get: () => 1.0 }});
            }})();"#,
            fingerprint.width, fingerprint.height, fingerprint.color_depth,
            fingerprint.pixel_depth, fingerprint.width, avail_height
        );

        self.injector.inject_init_script(page_id, &script).await
    }

    /// Inject WebGL protection
    async fn inject_webgl(
        &self,
        page_id: &str,
        fingerprint: &services::WebGLFingerprint,
    ) -> Result<(), Error> {
        let script = format!(
            r#"(function() {{
                const getParameter = WebGLRenderingContext.prototype.getParameter;
                WebGLRenderingContext.prototype.getParameter = function(parameter) {{
                    if (parameter === 37445) return '{}';
                    if (parameter === 37446) return '{}';
                    return getParameter.call(this, parameter);
                }};

                const getSupportedExtensions = WebGLRenderingContext.prototype.getSupportedExtensions;
                WebGLRenderingContext.prototype.getSupportedExtensions = function() {{
                    return getSupportedExtensions.call(this).sort(() => Math.random() - 0.5);
                }};
            }})();"#,
            fingerprint.vendor, fingerprint.renderer
        );

        self.injector.inject_init_script(page_id, &script).await
    }

    /// Inject canvas protection
    async fn inject_canvas(&self, page_id: &str) -> Result<(), Error> {
        let script = r#"(function() {
            const addNoise = (data) => {
                for (let i = 0; i < data.length; i += 4) {
                    data[i] += Math.random() * 0.1;
                    data[i + 1] += Math.random() * 0.1;
                    data[i + 2] += Math.random() * 0.1;
                }
            };

            const originalToDataURL = HTMLCanvasElement.prototype.toDataURL;
            HTMLCanvasElement.prototype.toDataURL = function(type) {
                const context = this.getContext('2d');
                if (context) {
                    const imageData = context.getImageData(0, 0, this.width, this.height);
                    addNoise(imageData.data);
                    context.putImageData(imageData, 0, 0);
                }
                return originalToDataURL.apply(this, arguments);
            };

            const originalGetImageData = CanvasRenderingContext2D.prototype.getImageData;
            CanvasRenderingContext2D.prototype.getImageData = function() {
                const imageData = originalGetImageData.apply(this, arguments);
                addNoise(imageData.data);
                return imageData;
            };
        })();"#;

        self.injector.inject_init_script(page_id, script).await
    }

    /// Inject audio protection
    async fn inject_audio(&self, page_id: &str) -> Result<(), Error> {
        let script = r#"(function() {
            const originalGetChannelData = AudioBuffer.prototype.getChannelData;
            AudioBuffer.prototype.getChannelData = function() {
                const data = originalGetChannelData.apply(this, arguments);
                for (let i = 0; i < data.length; i++) {
                    data[i] += Math.random() * 0.0001;
                }
                return data;
            };
        })();"#;

        self.injector.inject_init_script(page_id, script).await
    }

    /// Get applied features
    async fn get_applied_features(&self, page_id: &str) -> Result<AppliedFeatures, Error> {
        let tracker = self.applied_features.read().await;
        tracker
            .get(page_id)
            .cloned()
            .ok_or_else(|| Error::PageNotFound(format!("No features applied for page: {}", page_id)))
    }

    /// Remove all injections
    async fn remove_all(&self, page_id: &str) -> Result<(), Error> {
        self.injector.clear_all(page_id).await?;

        let mut tracker = self.applied_features.write().await;
        tracker.remove(page_id);

        Ok(())
    }
}
