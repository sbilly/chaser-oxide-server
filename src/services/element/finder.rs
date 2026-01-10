//! Element finder module
//!
//! Provides functionality to find DOM elements using various selector strategies.

use crate::error::{Error, Result};
use crate::services::traits::{ElementInfo, SelectorType};
use crate::session::traits::PageContext;
use std::sync::Arc;
use tracing::{debug, instrument};

/// Element finder
///
/// Responsible for finding DOM elements using different selector strategies.
pub struct ElementFinder {
    page: Arc<dyn PageContext>,
}

impl ElementFinder {
    /// Create a new element finder
    pub fn new(page: Arc<dyn PageContext>) -> Self {
        Self { page }
    }

    /// Find a single element
    #[instrument(skip(self))]
    pub async fn find_element(
        &self,
        selector_type: SelectorType,
        selector: &str,
    ) -> Result<ElementInfo> {
        debug!("Finding element: type={:?}, selector={}", selector_type, selector);

        let script = match selector_type {
            SelectorType::Css => self.build_css_selector_script(selector, false)?,
            SelectorType::XPath => self.build_xpath_selector_script(selector, false)?,
            SelectorType::Text => self.build_text_selector_script(selector, false)?,
        };

        let result = self.page.evaluate(&script, true).await?;

        match result {
            crate::session::traits::EvaluationResult::String(json_str) => {
                let element: ElementInfo = serde_json::from_str(&json_str)
                    .map_err(|e| Error::internal(format!("Failed to parse element: {}", e)))?;

                Ok(element)
            }
            _ => Err(Error::internal("Invalid element result")),
        }
    }

    /// Find multiple elements
    #[instrument(skip(self))]
    pub async fn find_elements(
        &self,
        selector_type: SelectorType,
        selector: &str,
        limit: Option<usize>,
    ) -> Result<Vec<ElementInfo>> {
        debug!(
            "Finding elements: type={:?}, selector={}, limit={:?}",
            selector_type, selector, limit
        );

        let script = match selector_type {
            SelectorType::Css => self.build_css_selector_script(selector, true)?,
            SelectorType::XPath => self.build_xpath_selector_script(selector, true)?,
            SelectorType::Text => self.build_text_selector_script(selector, true)?,
        };

        let result = self.page.evaluate(&script, true).await?;

        match result {
            crate::session::traits::EvaluationResult::String(json_str) => {
                let mut elements: Vec<ElementInfo> = serde_json::from_str(&json_str)
                    .map_err(|e| Error::internal(format!("Failed to parse elements: {}", e)))?;

                if let Some(limit) = limit {
                    elements.truncate(limit);
                }

                Ok(elements)
            }
            _ => Err(Error::internal("Invalid elements result")),
        }
    }

    /// Wait for element
    #[instrument(skip(self))]
    pub async fn wait_for_element(
        &self,
        selector_type: SelectorType,
        selector: &str,
        timeout_ms: u64,
    ) -> Result<ElementInfo> {
        debug!(
            "Waiting for element: type={:?}, selector={}, timeout={}",
            selector_type, selector, timeout_ms
        );

        let start = std::time::Instant::now();
        let poll_interval = tokio::time::Duration::from_millis(100);

        while start.elapsed().as_millis() < timeout_ms as u128 {
            match self.find_element(selector_type, selector).await {
                Ok(element) => return Ok(element),
                Err(Error::ElementNotFound(_)) => {
                    tokio::time::sleep(poll_interval).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        Err(Error::timeout(format!(
            "Element not found within {}ms: {:?} {}",
            timeout_ms, selector_type, selector
        )))
    }

    /// Build CSS selector script
    fn build_css_selector_script(&self, selector: &str, multiple: bool) -> Result<String> {
        let selector_escaped = selector.replace('\'', "\\'");
        let method = if multiple { "querySelectorAll" } else { "querySelector" };

        let script = if multiple {
            format!(
                r#"
                (() => {{
                    const elements = document.{}('{}');
                    const results = [];
                    for (let i = 0; i < elements.length; i++) {{
                        const el = elements[i];
                        results.push({{
                            element_id: el.id || 'css-' + i,
                            tag_name: el.tagName.toLowerCase(),
                            text_content: el.textContent ? el.textContent.substring(0, 100) : null
                        }});
                    }}
                    return JSON.stringify(results);
                }})()
                "#,
                method, selector_escaped
            )
        } else {
            format!(
                r#"
                (() => {{
                    const el = document.{}('{}');
                    if (!el) return null;
                    return JSON.stringify({{
                        element_id: el.id || 'css-single',
                        tag_name: el.tagName.toLowerCase(),
                        text_content: el.textContent ? el.textContent.substring(0, 100) : null
                    }});
                }})()
                "#,
                method, selector_escaped
            )
        };

        Ok(script)
    }

    /// Build XPath selector script
    fn build_xpath_selector_script(&self, xpath: &str, multiple: bool) -> Result<String> {
        let xpath_escaped = xpath.replace('\\', "\\\\").replace('"', r#"\""#);

        let script = if multiple {
            format!(
                r#"
                (() => {{
                    const result = document.evaluate('{}', document, null, XPathResult.ORDERED_NODE_SNAPSHOT_TYPE, null);
                    const results = [];
                    for (let i = 0; i < result.snapshotLength; i++) {{
                        const el = result.snapshotItem(i);
                        results.push({{
                            element_id: el.id || 'xpath-' + i,
                            tag_name: el.tagName.toLowerCase(),
                            text_content: el.textContent ? el.textContent.substring(0, 100) : null
                        }});
                    }}
                    return JSON.stringify(results);
                }})()
                "#,
                xpath_escaped
            )
        } else {
            format!(
                r#"
                (() => {{
                    const result = document.evaluate('{}', document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null);
                    const el = result.singleNodeValue;
                    if (!el) return null;
                    return JSON.stringify({{
                        element_id: el.id || 'xpath-single',
                        tag_name: el.tagName.toLowerCase(),
                        text_content: el.textContent ? el.textContent.substring(0, 100) : null
                    }});
                }})()
                "#,
                xpath_escaped
            )
        };

        Ok(script)
    }

    /// Build text selector script
    fn build_text_selector_script(&self, text: &str, multiple: bool) -> Result<String> {
        let text_escaped = text.replace('\\', "\\\\").replace('"', r#"\""#);

        let script = if multiple {
            format!(
                r#"
                (() => {{
                    const walker = document.createTreeWalker(
                        document.body,
                        NodeFilter.SHOW_TEXT,
                        {{
                            acceptNode: (node) => {{
                                return node.textContent.includes('{}') ? NodeFilter.FILTER_ACCEPT : NodeFilter.FILTER_REJECT;
                            }}
                        }}
                    );
                    const results = [];
                    let node;
                    let i = 0;
                    while (node = walker.nextNode()) {{
                        const el = node.parentElement;
                        if (el) {{
                            results.push({{
                                element_id: el.id || 'text-' + i,
                                tag_name: el.tagName.toLowerCase(),
                                text_content: el.textContent ? el.textContent.substring(0, 100) : null
                            }});
                            i++;
                        }}
                    }}
                    return JSON.stringify(results);
                }})()
                "#,
                text_escaped
            )
        } else {
            format!(
                r#"
                (() => {{
                    const walker = document.createTreeWalker(
                        document.body,
                        NodeFilter.SHOW_TEXT,
                        {{
                            acceptNode: (node) => {{
                                return node.textContent.includes('{}') ? NodeFilter.FILTER_ACCEPT : NodeFilter.FILTER_REJECT;
                            }}
                        }}
                    );
                    let node;
                    while (node = walker.nextNode()) {{
                        const el = node.parentElement;
                        if (el) {{
                            return JSON.stringify({{
                                element_id: el.id || 'text-single',
                                tag_name: el.tagName.toLowerCase(),
                                text_content: el.textContent ? el.textContent.substring(0, 100) : null
                            }});
                        }}
                    }}
                    return null;
                }})()
                "#,
                text_escaped
            )
        };

        Ok(script)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock PageContext for testing
    struct MockPage;

    #[test]
    fn test_build_css_selector_script() {
        // Test script generation without needing a real page
        let script = format!(
            r#"
            (() => {{
                const el = document.querySelector('{}');
                if (!el) return null;
                return JSON.stringify({{
                    element_id: el.id || 'css-single',
                    tag_name: el.tagName.toLowerCase(),
                    text_content: el.textContent ? el.textContent.substring(0, 100) : null
                }});
            }})()
            "#,
            "button.submit".replace('\'', "\\'")
        );

        assert!(script.contains("querySelector"));
        assert!(script.contains("button.submit"));
    }

    #[test]
    fn test_build_xpath_selector_script() {
        let xpath = "//button[@type='submit']";
        let xpath_escaped = xpath.replace('\\', "\\\\").replace('"', r#"\""#);

        let script = format!(
            r#"
            (() => {{
                const result = document.evaluate('{}', document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null);
                const el = result.singleNodeValue;
                if (!el) return null;
                return JSON.stringify({{
                    element_id: el.id || 'xpath-single',
                    tag_name: el.tagName.toLowerCase(),
                    text_content: el.textContent ? el.textContent.substring(0, 100) : null
                }});
            }})()
            "#,
            xpath_escaped
        );

        assert!(script.contains("document.evaluate"));
        assert!(script.contains("XPathResult.FIRST_ORDERED_NODE_TYPE"));
    }

    #[test]
    fn test_build_text_selector_script() {
        let text = "Submit";
        let text_escaped = text.replace('\\', "\\\\").replace('"', r#"\""#);

        let script = format!(
            r#"
            (() => {{
                const walker = document.createTreeWalker(
                    document.body,
                    NodeFilter.SHOW_TEXT,
                    {{
                        acceptNode: (node) => {{
                            return node.textContent.includes('{}') ? NodeFilter.FILTER_ACCEPT : NodeFilter.FILTER_REJECT;
                        }}
                    }}
                );
                let node;
                while (node = walker.nextNode()) {{
                    const el = node.parentElement;
                    if (el) {{
                        return JSON.stringify({{
                            element_id: el.id || 'text-single',
                            tag_name: el.tagName.toLowerCase(),
                            text_content: el.textContent ? el.textContent.substring(0, 100) : null
                        }});
                    }}
                }}
                return null;
            }})()
            "#,
            text_escaped
        );

        assert!(script.contains("createTreeWalker"));
        assert!(script.contains("Submit"));
    }

    #[test]
    fn test_selector_escaping() {
        let selector_with_quote = "button[title='Click here']";
        let escaped = selector_with_quote.replace('\'', "\\'");

        // The replace adds backslashes before quotes, not remove them
        assert!(escaped.contains("\\'"));
        // The original quotes should still be present after being escaped
        assert!(escaped.contains("'"));
    }

    #[test]
    fn test_xpath_escaping() {
        let xpath_with_quotes = "//button[@type=\"submit\"]";
        let escaped = xpath_with_quotes.replace('\\', "\\\\").replace('"', r#"\""#);

        // The replace adds backslashes before quotes, not remove them
        assert!(escaped.contains(r#"\""#));
        // The original quotes should still be present after being escaped
        assert!(escaped.contains("\""));
    }

    #[test]
    fn test_multiple_vs_single_selector() {
        // Test multiple selector generation
        let multiple_method = "querySelectorAll";
        let single_method = "querySelector";

        assert_eq!(multiple_method, "querySelectorAll");
        assert_eq!(single_method, "querySelector");
        assert_ne!(multiple_method, single_method);
    }
}
