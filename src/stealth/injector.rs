//! Script injector implementation
//!
//! Handles injection of JavaScript code for fingerprint modification.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use uuid::Uuid;

use crate::Error;
use super::traits::*;

/// Script injector implementation
pub struct ScriptInjectorImpl {
    /// Session manager for getting page CDP clients
    session_manager: Arc<dyn crate::session::traits::SessionManager>,
    /// Injected scripts tracking
    injected_scripts: Arc<RwLock<HashMap<String, Vec<InjectedScript>>>>,
}

impl ScriptInjectorImpl {
    /// Create a new script injector
    pub fn new(session_manager: Arc<dyn crate::session::traits::SessionManager>) -> Self {
        Self {
            session_manager,
            injected_scripts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate script ID
    fn generate_script_id(&self) -> String {
        Uuid::new_v4().to_string()
    }

    /// Track injected script
    async fn track_script(&self, page_id: &str, script_id: String, script_type: ScriptType, content: String) {
        let injected = InjectedScript {
            script_id,
            script_type,
            content,
        };

        let mut tracker = self.injected_scripts.write().await;
        tracker
            .entry(page_id.to_string())
            .or_insert_with(Vec::new)
            .push(injected);
    }

    /// Get CDP client for page
    async fn get_cdp_client(&self, page_id: &str) -> Result<Arc<dyn crate::cdp::CdpClient>, Error> {
        let page = self.session_manager.get_page(page_id).await?;
        Ok(page.get_cdp_client())
    }
}

#[async_trait]
impl ScriptInjector for ScriptInjectorImpl {
    /// Inject JavaScript before page load
    async fn inject_init_script(&self, page_id: &str, script: &str) -> Result<(), Error> {
        let script_id = self.generate_script_id();
        tracing::debug!("[P5-DEBUG] Injecting script for page {}: {} bytes", page_id, script.len());

        let cdp_client = self.get_cdp_client(page_id).await?;

        // Add script to evaluate on new document
        let params = serde_json::json!({ "source": script });
        match cdp_client.call_method("Page.addScriptToEvaluateOnNewDocument", params).await {
            Ok(result) => {
                if let Some(identifier) = result.get("identifier") {
                    tracing::debug!("[P5-DEBUG] Script added with identifier: {}", identifier);
                }
            }
            Err(e) => {
                tracing::error!("[P5-DEBUG] Failed to add script: {}", e);
                return Err(Error::internal(format!("Page.addScriptToEvaluateOnNewDocument failed: {}", e)));
            }
        }

        // Evaluate immediately for current page
        let eval_params = serde_json::json!({
            "expression": script,
            "awaitPromise": true
        });

        if let Err(e) = cdp_client.call_method("Runtime.evaluate", eval_params).await {
            tracing::warn!("[P5-DEBUG] Runtime.evaluate failed (non-critical): {}", e);
        }

        self.track_script(page_id, script_id, ScriptType::InitScript, script.to_string()).await;
        Ok(())
    }

    /// Evaluate JavaScript in the page
    async fn evaluate(&self, page_id: &str, script: &str) -> Result<String, Error> {
        let cdp_client = self.get_cdp_client(page_id).await?;

        let params = serde_json::json!({
            "expression": script,
            "awaitPromise": true,
            "returnByValue": true
        });

        let result = cdp_client.call_method("Runtime.evaluate", params).await?;

        result
            .get("result")
            .and_then(|r| r.get("value"))
            .map(|v| v.to_string())
            .ok_or_else(|| Error::ScriptExecutionFailed("No result value".to_string()))
    }

    /// Inject CSS
    async fn inject_style(&self, page_id: &str, css: &str) -> Result<(), Error> {
        let escaped_css = css.replace('\\', "\\\\").replace('\'', "\\'");
        let script = format!(
            r#"(function() {{
                const style = document.createElement('style');
                style.textContent = '{}';
                document.head.appendChild(style);
            }})();"#,
            escaped_css
        );

        self.evaluate(page_id, &script).await?;
        self.track_script(page_id, self.generate_script_id(), ScriptType::Style, css.to_string()).await;
        Ok(())
    }

    /// Set User-Agent at CDP protocol level
    async fn set_user_agent(&self, page_id: &str, user_agent: &str) -> Result<(), Error> {
        let cdp_client = self.get_cdp_client(page_id).await?;
        cdp_client.enable_domain("Network").await?;

        let params = serde_json::json!({ "userAgent": user_agent });
        cdp_client.call_method("Network.setUserAgentOverride", params).await?;
        Ok(())
    }

    /// Get all injected scripts
    async fn get_injected_scripts(&self, page_id: &str) -> Result<Vec<InjectedScript>, Error> {
        let tracker = self.injected_scripts.read().await;
        tracker
            .get(page_id)
            .cloned()
            .ok_or_else(|| Error::PageNotFound(format!("No scripts injected for page: {}", page_id)))
    }

    /// Remove injected script
    async fn remove_script(&self, page_id: &str, script_id: &str) -> Result<(), Error> {
        let mut tracker = self.injected_scripts.write().await;

        if let Some(scripts) = tracker.get_mut(page_id) {
            scripts.retain(|s| s.script_id != script_id);
        }

        Ok(())
    }

    /// Clear all injected scripts
    async fn clear_all(&self, page_id: &str) -> Result<(), Error> {
        let mut tracker = self.injected_scripts.write().await;
        tracker.remove(page_id);

        Ok(())
    }
}
