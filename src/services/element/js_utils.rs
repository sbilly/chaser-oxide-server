//! JavaScript code generation utilities for element operations
//!
//! This module provides utilities for generating JavaScript code
//! that executes on DOM elements in the browser.

use crate::error::{Error, Result};

/// JavaScript code builder for element operations
///
/// Provides methods to generate JavaScript code snippets for common
/// DOM operations like querying, clicking, and manipulating elements.
#[derive(Debug, Clone)]
pub struct JsBuilder {
    selector_type: i32,
    selector: String,
}

impl JsBuilder {
    /// Create a new JavaScript builder for element operations
    ///
    /// # Arguments
    /// * `selector_type` - Type of selector (1=CSS, 2=XPath, 3=Text)
    /// * `selector` - The selector string
    pub fn new(selector_type: i32, selector: String) -> Self {
        Self {
            selector_type,
            selector,
        }
    }

    /// Escape a string for safe use in JavaScript
    ///
    /// Handles backslashes, single quotes, and double quotes to prevent
    /// injection attacks and syntax errors when embedding strings in JavaScript code.
    ///
    /// # Arguments
    /// * `s` - The string to escape
    ///
    /// # Returns
    /// Escaped string safe for embedding in JavaScript code
    ///
    /// # Examples
    /// ```
    /// assert_eq!(JsBuilder::escape_js_str("test's"), "test\\'s");
    /// assert_eq!(JsBuilder::escape_js_str("test\"s"), r#"test\"s"#);
    /// ```
    pub fn escape_js_str(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('\'', "\\'")
            .replace('"', r#"\""#)
    }

    /// Generate JavaScript to query an element
    ///
    /// Creates the appropriate query expression based on selector type.
    /// Supports CSS selectors (1), XPath (2), and text search (3).
    ///
    /// # Returns
    /// JavaScript expression that evaluates to the element (or null if not found)
    ///
    /// # Errors
    /// Returns error if selector_type is invalid (not 1, 2, or 3)
    pub fn element_query(&self) -> Result<String> {
        Ok(match self.selector_type {
            1 => {
                format!(
                    "document.querySelector('{}')",
                    Self::escape_js_str(&self.selector)
                )
            }
            2 => {
                format!(
                    "document.evaluate('{}', document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue",
                    Self::escape_js_str(&self.selector)
                )
            }
            3 => {
                format!(
                    "(() => {{ \
                        const walker = document.createTreeWalker(\
                            document.body, \
                            NodeFilter.SHOW_TEXT, \
                            {{ acceptNode: (node) => node.textContent.includes('{}') ? NodeFilter.FILTER_ACCEPT : NodeFilter.FILTER_REJECT }} \
                        ); \
                        let node; \
                        while (node = walker.nextNode()) {{ \
                            return node.parentElement; \
                        }} \
                        return null; \
                    }})()",
                    Self::escape_js_str(&self.selector)
                )
            }
            _ => {
                return Err(Error::internal(format!(
                    "Invalid selector type: {}",
                    self.selector_type
                )))
            }
        })
    }

    /// Build a script that executes code on an element
    ///
    /// Wraps the provided JavaScript code to execute it on the element
    /// found by the selector. The element will be available as the `el` variable.
    ///
    /// # Arguments
    /// * `js_code` - JavaScript code to execute (has access to `el` variable)
    ///
    /// # Returns
    /// Complete JavaScript script as a string wrapped in an IIFE
    ///
    /// # Examples
    /// ```no_run
    /// # use chaser_oxide::services::element::js_utils::JsBuilder;
    /// # fn test() -> Result<(), Box<dyn std::error::Error>> {
    /// let builder = JsBuilder::new(1, "button".to_string());
    /// let script = builder.execute_on_element("el.click()")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn execute_on_element(&self, js_code: &str) -> Result<String> {
        Ok(format!(
            r#"(() => {{ const el = {}; if (!el) return null; {} }})()"#,
            self.element_query()?,
            js_code
        ))
    }

    /// Build script to click on element
    ///
    /// Generates JavaScript that scrolls the element into view and triggers a click event.
    /// Uses smooth scrolling with center alignment for better visibility.
    pub fn click_script(&self) -> Result<String> {
        self.execute_on_element(
            "el.scrollIntoView({behavior: 'smooth', block: 'center'}); el.click(); 'clicked'",
        )
    }

    /// Build script to focus on element
    ///
    /// Generates JavaScript that sets keyboard focus to the element.
    pub fn focus_script(&self) -> Result<String> {
        self.execute_on_element("el.focus(); 'focused'")
    }

    /// Build script to hover over element
    ///
    /// Generates JavaScript that dispatches a mouseover event on the element.
    /// Creates a proper MouseEvent with bubbling and cancelable properties.
    pub fn hover_script(&self) -> Result<String> {
        self.execute_on_element(
            r#"const event = new MouseEvent('mouseover', {bubbles: true, cancelable: true, view: window});
               el.dispatchEvent(event); 'hovered'"#,
        )
    }

    /// Build script to get element text content
    ///
    /// Returns the textContent, falling back to innerText if needed.
    pub fn get_text_script(&self) -> Result<String> {
        self.execute_on_element("el.textContent || el.innerText || ''")
    }

    /// Build script to get element HTML
    ///
    /// # Arguments
    /// * `outer` - If true, get outerHTML (includes element tag); otherwise get innerHTML
    pub fn get_html_script(&self, outer: bool) -> Result<String> {
        let prop = if outer { "outerHTML" } else { "innerHTML" };
        self.execute_on_element(&format!("el.{} || ''", prop))
    }

    /// Build script to get element attribute
    ///
    /// # Arguments
    /// * `attr_name` - Name of the attribute to retrieve
    ///
    /// # Returns
    /// Attribute value or empty string if not present
    pub fn get_attribute_script(&self, attr_name: &str) -> Result<String> {
        self.execute_on_element(&format!(
            "el.getAttribute('{}') || ''",
            Self::escape_js_str(attr_name)
        ))
    }

    /// Build script to get element bounding box
    ///
    /// Returns the element's position and dimensions as a JSON object.
    pub fn get_bounding_box_script(&self) -> Result<String> {
        self.execute_on_element(
            r#"JSON.stringify({
                x: el.getBoundingClientRect().x,
                y: el.getBoundingClientRect().y,
                width: el.getBoundingClientRect().width,
                height: el.getBoundingClientRect().height
            })"#,
        )
    }

    /// Build script to check element visibility
    ///
    /// Performs comprehensive visibility checking including:
    /// - CSS display property
    /// - CSS visibility property
    /// - CSS opacity property
    /// - Element dimensions (zero width/height)
    /// - Viewport position (above/below viewport)
    ///
    /// # Returns
    /// JSON object with `visible` (boolean) and `reason` (string explaining why invisible)
    pub fn is_visible_script(&self) -> Result<String> {
        self.execute_on_element(
            r#"const style = window.getComputedStyle(el);
              const rect = el.getBoundingClientRect();

              let reason = 'visible';
              let visible = true;

              if (style.display === 'none') {
                  visible = false;
                  reason = 'display: none';
              } else if (style.visibility === 'hidden') {
                  visible = false;
                  reason = 'visibility: hidden';
              } else if (style.opacity === '0' || style.opacity === '0.0') {
                  visible = false;
                  reason = 'opacity: 0';
              } else if (rect.width === 0 || rect.height === 0) {
                  visible = false;
                  reason = 'zero size';
              } else if (rect.top < 0 && rect.bottom < 0) {
                  visible = false;
                  reason = 'above viewport';
              } else if (rect.top > window.innerHeight && rect.bottom > window.innerHeight) {
                  visible = false;
                  reason = 'below viewport';
              }

              JSON.stringify({visible, reason})"#,
        )
    }

    /// Build script to check if element is enabled
    ///
    /// Checks element enabled state including:
    /// - HTML disabled attribute
    /// - HTML readonly attribute
    /// - Parent fieldset disabled state (inherited)
    ///
    /// # Returns
    /// JSON object with `enabled` (boolean) and `reason` (string explaining why disabled)
    pub fn is_enabled_script(&self) -> Result<String> {
        self.execute_on_element(
            r#"let enabled = true;
              let reason = 'enabled';

              if (el.disabled) {
                  enabled = false;
                  reason = 'disabled attribute';
              } else if (el.readOnly) {
                  enabled = false;
                  reason = 'readonly attribute';
              } else {
                  let parent = el.parentElement;
                  while (parent) {
                      if (parent.tagName === 'FIELDSET' && parent.disabled) {
                          enabled = false;
                          reason = 'parent fieldset disabled';
                          break;
                      }
                      parent = parent.parentElement;
                  }
              }

              JSON.stringify({enabled, reason})"#,
        )
    }

    /// Build script to type text into element
    ///
    /// Sets the element's value and triggers input/change events.
    ///
    /// # Arguments
    /// * `text` - Text value to set
    pub fn type_text_script(&self, text: &str) -> Result<String> {
        self.execute_on_element(&format!(
            r#"el.focus(); el.value = '{}';
               el.dispatchEvent(new Event('input', {{bubbles: true}}));
               el.dispatchEvent(new Event('change', {{bubbles: true}})); 'typed'"#,
            Self::escape_js_str(text)
        ))
    }

    /// Build script to fill element with value
    ///
    /// Optionally clears existing value before setting new value.
    /// Triggers input and change events for proper form handling.
    ///
    /// # Arguments
    /// * `value` - Value to set
    /// * `clear_first` - If true, attempt to clear existing value first
    pub fn fill_script(&self, value: &str, clear_first: bool) -> Result<String> {
        let clear = if clear_first {
            "if (el.clear) el.clear();"
        } else {
            ""
        };

        self.execute_on_element(&format!(
            r#"el.focus(); {} el.value = '{}';
               el.dispatchEvent(new Event('input', {{bubbles: true}}));
               el.dispatchEvent(new Event('change', {{bubbles: true}})); 'filled'"#,
            clear,
            Self::escape_js_str(value)
        ))
    }

    /// Build script to select option in dropdown
    ///
    /// Sets the value property of a select element and triggers change event.
    ///
    /// # Arguments
    /// * `value` - Option value to select
    pub fn select_option_script(&self, value: &str) -> Result<String> {
        self.execute_on_element(&format!(
            r#"el.value = '{}'; el.dispatchEvent(new Event('change', {{bubbles: true}})); 'selected'"#,
            Self::escape_js_str(value)
        ))
    }

    /// Build script to scroll element into view
    ///
    /// Uses smooth scrolling behavior.
    ///
    /// # Arguments
    /// * `align_to_top` - If true, align to top of viewport; otherwise align to bottom
    pub fn scroll_into_view_script(&self, align_to_top: bool) -> Result<String> {
        let block = if align_to_top { "start" } else { "end" };
        self.execute_on_element(&format!(
            "el.scrollIntoView({{behavior: 'smooth', block: '{}'}}); 'scrolled'",
            block
        ))
    }

    /// Build script to press key on element
    ///
    /// Dispatches keyboard events (keydown, keypress, keyup) for the specified key.
    ///
    /// # Arguments
    /// * `key` - Key to press (e.g., "Enter", "Escape", "Ctrl+A")
    pub fn press_key_script(&self, key: &str) -> Result<String> {
        self.execute_on_element(&format!(
            r#"el.dispatchEvent(new KeyboardEvent('keydown', {{key: '{}', bubbles: true}}));
               el.dispatchEvent(new KeyboardEvent('keypress', {{key: '{}', bubbles: true}}));
               el.dispatchEvent(new KeyboardEvent('keyup', {{key: '{}', bubbles: true}})); 'key_pressed'"#,
            Self::escape_js_str(key),
            Self::escape_js_str(key),
            Self::escape_js_str(key)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_js_str() {
        assert_eq!(JsBuilder::escape_js_str("test"), "test");
        assert_eq!(JsBuilder::escape_js_str("test's"), "test\\'s");
        assert_eq!(JsBuilder::escape_js_str("test\"s"), r#"test\"s"#);
        assert_eq!(JsBuilder::escape_js_str("test\\s"), "test\\\\s");
    }

    #[test]
    fn test_css_query() {
        let builder = JsBuilder::new(1, "button.submit".to_string());
        let query = builder.element_query().unwrap();
        assert!(query.contains("querySelector"));
        assert!(query.contains("button.submit"));
    }

    #[test]
    fn test_xpath_query() {
        let builder = JsBuilder::new(2, "//button[@type='submit']".to_string());
        let query = builder.element_query().unwrap();
        assert!(query.contains("document.evaluate"));
        assert!(query.contains("XPathResult"));
    }

    #[test]
    fn test_text_query() {
        let builder = JsBuilder::new(3, "Submit".to_string());
        let query = builder.element_query().unwrap();
        assert!(query.contains("createTreeWalker"));
        assert!(query.contains("Submit"));
    }

    #[test]
    fn test_click_script() {
        let builder = JsBuilder::new(1, "button".to_string());
        let script = builder.click_script().unwrap();
        assert!(script.contains("scrollIntoView"));
        assert!(script.contains("click()"));
    }

    #[test]
    fn test_type_text_script() {
        let builder = JsBuilder::new(1, "input".to_string());
        let script = builder.type_text_script("hello").unwrap();
        assert!(script.contains("el.focus()"));
        assert!(script.contains("el.value = 'hello'"));
        assert!(script.contains("dispatchEvent"));
    }

    #[test]
    fn test_invalid_selector_type() {
        let builder = JsBuilder::new(99, "test".to_string());
        assert!(builder.element_query().is_err());
    }
}
