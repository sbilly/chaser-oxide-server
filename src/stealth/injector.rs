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
}

#[async_trait]
impl ScriptInjector for ScriptInjectorImpl {
    /// Inject JavaScript before page load
    async fn inject_init_script(&self, page_id: &str, script: &str) -> Result<(), Error> {
        let script_id = self.generate_script_id();

        tracing::debug!("[P5-DEBUG] Injecting script for page {}: {} bytes", page_id, script.len());

        // Get the page's CDP client from session manager
        let page = self.session_manager.get_page(page_id).await?;
        let cdp_client = page.get_cdp_client();

        // Use Page.addScriptToEvaluateOnNewDocument for future navigations
        let params = serde_json::json!({
            "source": script
        });

        let add_script_result = cdp_client
            .call_method("Page.addScriptToEvaluateOnNewDocument", params)
            .await;

        match &add_script_result {
            Ok(result) => {
                if let Some(identifier) = result.get("identifier") {
                    tracing::debug!("[P5-DEBUG] Page.addScriptToEvaluateOnNewDocument SUCCESS, identifier: {}", identifier);
                } else {
                    tracing::debug!("[P5-DEBUG] Page.addScriptToEvaluateOnNewDocument SUCCESS (no identifier in response): {:?}", result);
                }
            }
            Err(e) => {
                tracing::error!("[P5-DEBUG] Page.addScriptToEvaluateOnNewDocument FAILED: {}", e);
                return Err(Error::internal(format!("Page.addScriptToEvaluateOnNewDocument failed: {}", e)));
            }
        }

        // Also evaluate immediately for the current page (if already loaded)
        // This ensures the script takes effect on the current page too
        let eval_params = serde_json::json!({
            "expression": script,
            "awaitPromise": true
        });

        let eval_result = cdp_client
            .call_method("Runtime.evaluate", eval_params)
            .await;

        match &eval_result {
            Ok(result) => {
                tracing::debug!("[P5-DEBUG] Runtime.evaluate SUCCESS: {:?}", result);
            }
            Err(e) => {
                tracing::warn!("[P5-DEBUG] Runtime.evaluate failed (non-critical): {}", e);
                // Don't fail on Runtime.evaluate error, the addScriptToEvaluateOnNewDocument is more important
            }
        }

        // Track injected script
        let injected = InjectedScript {
            script_id: script_id.clone(),
            script_type: ScriptType::InitScript,
            content: script.to_string(),
        };

        let mut tracker = self.injected_scripts.write().await;
        tracker
            .entry(page_id.to_string())
            .or_insert_with(Vec::new)
            .push(injected);

        Ok(())
    }

    /// Evaluate JavaScript in the page
    async fn evaluate(&self, page_id: &str, script: &str) -> Result<String, Error> {
        // Get the page's CDP client from session manager
        let page = self.session_manager.get_page(page_id).await?;
        let cdp_client = page.get_cdp_client();

        let params = serde_json::json!({
            "expression": script,
            "awaitPromise": true,
            "returnByValue": true
        });

        let result = cdp_client
            .call_method("Runtime.evaluate", params)
            .await?;

        // Extract result value
        if let Some(result_obj) = result.get("result") {
            if let Some(value) = result_obj.get("value") {
                return Ok(value.to_string());
            }
        }

        Ok(String::new())
    }

    /// Inject CSS
    async fn inject_style(&self, page_id: &str, css: &str) -> Result<(), Error> {
        let script = format!(
            r#"(function() {{
                const style = document.createElement('style');
                style.textContent = '{}';
                document.head.appendChild(style);
            }})();"#,
            css.replace('\\', "\\\\").replace('\'', "\\'")
        );

        self.evaluate(page_id, &script).await?;

        // Track injected style
        let injected = InjectedScript {
            script_id: self.generate_script_id(),
            script_type: ScriptType::Style,
            content: css.to_string(),
        };

        let mut tracker = self.injected_scripts.write().await;
        tracker
            .entry(page_id.to_string())
            .or_insert_with(Vec::new)
            .push(injected);

        Ok(())
    }

    /// Set User-Agent at CDP protocol level
    async fn set_user_agent(&self, page_id: &str, user_agent: &str) -> Result<(), Error> {
        // Get the page's CDP client from session manager
        let page = self.session_manager.get_page(page_id).await?;
        let cdp_client = page.get_cdp_client();

        // Enable Network domain first
        cdp_client.enable_domain("Network").await?;

        // Set User-Agent override using Network.setUserAgentOverride
        let params = serde_json::json!({
            "userAgent": user_agent
        });

        cdp_client
            .call_method("Network.setUserAgentOverride", params)
            .await?;

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
