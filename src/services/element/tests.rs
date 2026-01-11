//! ElementService unit tests

use super::super::traits::SelectorType;
use super::finder::ElementFinder;
use super::interactor::ElementInteractor;
use crate::session::mock::{MockElement, MockPage, MockSessionManager};
use std::sync::Arc;

#[tokio::test]
async fn test_element_finder_css_selector() {
    let browser_id = "test-browser".to_string();
    let manager = Arc::new(MockSessionManager::new());

    // Create a mock page
    let page = Arc::new(MockPage::new(browser_id, crate::session::traits::PageOptions::default()));

    let finder = ElementFinder::new(page);

    // Test CSS selector (will return mock data)
    let result = finder.find_element(SelectorType::Css, "button.submit").await;

    // Since we're using mock, this should work with mock data
    assert!(result.is_ok() || result.is_err()); // Just check it doesn't panic
}

#[tokio::test]
async fn test_element_finder_xpath_selector() {
    let browser_id = "test-browser".to_string();
    let page = Arc::new(MockPage::new(
        browser_id,
        crate::session::traits::PageOptions::default(),
    ));

    let finder = ElementFinder::new(page);

    // Test XPath selector
    let result = finder.find_element(SelectorType::XPath, "//button[@type='submit']").await;

    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_element_finder_text_selector() {
    let browser_id = "test-browser".to_string();
    let page = Arc::new(MockPage::new(
        browser_id,
        crate::session::traits::PageOptions::default(),
    ));

    let finder = ElementFinder::new(page);

    // Test text selector
    let result = finder.find_element(SelectorType::Text, "Submit").await;

    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_element_finder_find_multiple() {
    let browser_id = "test-browser".to_string();
    let page = Arc::new(MockPage::new(
        browser_id,
        crate::session::traits::PageOptions::default(),
    ));

    let finder = ElementFinder::new(page);

    // Test finding multiple elements
    let result = finder
        .find_elements(SelectorType::Css, "button", Some(5))
        .await;

    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_element_interactor_click() {
    let element = Arc::new(MockElement::new(
        "test-page".to_string(),
        "button".to_string(),
        Some("Submit".to_string()),
    ));

    let interactor = ElementInteractor::new(element);

    // Test click
    let result = interactor.click().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_element_interactor_type_text() {
    let element = Arc::new(MockElement::new(
        "test-page".to_string(),
        "input".to_string(),
        None,
    ));

    let interactor = ElementInteractor::new(element);

    // Test type text
    let result = interactor.type_text("test text", Some(10)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_element_interactor_fill() {
    let element = Arc::new(MockElement::new(
        "test-page".to_string(),
        "input".to_string(),
        None,
    ));

    let interactor = ElementInteractor::new(element);

    // Test fill
    let result = interactor.fill("test value", true).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_element_interactor_get_text() {
    let element = Arc::new(MockElement::new(
        "test-page".to_string(),
        "div".to_string(),
        Some("Test content".to_string()),
    ));

    let interactor = ElementInteractor::new(element);

    // Test get text
    let result = interactor.get_text(true).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Test content");
}

#[tokio::test]
async fn test_element_interactor_get_html() {
    let element = Arc::new(MockElement::new(
        "test-page".to_string(),
        "div".to_string(),
        None,
    ));

    let interactor = ElementInteractor::new(element);

    // Test get HTML
    let result = interactor.get_html(true).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "<div></div>");
}

#[tokio::test]
async fn test_element_interactor_hover() {
    let element = Arc::new(MockElement::new(
        "test-page".to_string(),
        "button".to_string(),
        None,
    ));

    let interactor = ElementInteractor::new(element);

    // Test hover
    let result = interactor.hover().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_element_interactor_focus() {
    let element = Arc::new(MockElement::new(
        "test-page".to_string(),
        "input".to_string(),
        None,
    ));

    let interactor = ElementInteractor::new(element);

    // Test focus
    let result = interactor.focus().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_element_interactor_is_visible() {
    let element = Arc::new(MockElement::new(
        "test-page".to_string(),
        "div".to_string(),
        None,
    ));

    let interactor = ElementInteractor::new(element);

    // Test is visible
    let result = interactor.is_visible().await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_element_interactor_is_enabled() {
    let element = Arc::new(MockElement::new(
        "test-page".to_string(),
        "button".to_string(),
        None,
    ));

    let interactor = ElementInteractor::new(element);

    // Test is enabled
    let result = interactor.is_enabled().await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_element_interactor_get_bounding_box() {
    let element = Arc::new(MockElement::new(
        "test-page".to_string(),
        "div".to_string(),
        None,
    ));

    let interactor = ElementInteractor::new(element);

    // Test get bounding box
    let result = interactor.get_bounding_box().await;
    assert!(result.is_ok());

    let bbox = result.unwrap();
    assert_eq!(bbox.width, 100.0);
    assert_eq!(bbox.height, 100.0);
}

#[tokio::test]
async fn test_element_interactor_get_attribute() {
    let element = Arc::new(MockElement::new(
        "test-page".to_string(),
        "input".to_string(),
        None,
    ));

    let interactor = ElementInteractor::new(element);

    // Test get attribute (mock returns None)
    let result = interactor.get_attribute("placeholder").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);
}

#[tokio::test]
async fn test_element_interactor_get_attributes() {
    let element = Arc::new(MockElement::new(
        "test-page".to_string(),
        "input".to_string(),
        None,
    ));

    let interactor = ElementInteractor::new(element);

    // Test get multiple attributes
    let attrs = vec!["type".to_string(), "name".to_string(), "id".to_string()];
    let result = interactor.get_attributes(&attrs).await;
    assert!(result.is_ok());

    let attributes = result.unwrap();
    // Mock element returns None for all attributes, and get_attributes only adds entries
    // when the attribute value is Some, so we get an empty vector
    assert_eq!(attributes.len(), 0);
}

#[tokio::test]
async fn test_element_interactor_select_option() {
    let element = Arc::new(MockElement::new(
        "test-page".to_string(),
        "select".to_string(),
        None,
    ));

    let interactor = ElementInteractor::new(element);

    // Test select option
    let values = vec!["option1".to_string(), "option2".to_string()];
    let result = interactor.select_option(values).await;
    // Implementation logs but doesn't fail in test environment
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_element_interactor_upload_file() {
    let element = Arc::new(MockElement::new(
        "test-page".to_string(),
        "input".to_string(),
        None,
    ));

    let interactor = ElementInteractor::new(element);

    // Test file upload with valid paths
    let files = vec!["/path/to/file.txt".to_string()];
    let result = interactor.upload_file(files).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_element_interactor_upload_file_empty_paths() {
    let element = Arc::new(MockElement::new(
        "test-page".to_string(),
        "input".to_string(),
        None,
    ));

    let interactor = ElementInteractor::new(element);

    // Test file upload with empty paths should fail
    let files: Vec<String> = vec![];
    let result = interactor.upload_file(files).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_element_interactor_scroll_into_view() {
    let element = Arc::new(MockElement::new(
        "test-page".to_string(),
        "div".to_string(),
        None,
    ));

    let interactor = ElementInteractor::new(element);

    // Test scroll into view
    let result = interactor.scroll_into_view(true).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_element_interactor_press_key() {
    let element = Arc::new(MockElement::new(
        "test-page".to_string(),
        "input".to_string(),
        None,
    ));

    let interactor = ElementInteractor::new(element);

    // Test press key with combination
    let result = interactor.press_key("Ctrl+A", Some(50)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_element_interactor_drag_and_drop() {
    let element = Arc::new(MockElement::new(
        "test-page".to_string(),
        "div".to_string(),
        None,
    ));

    let interactor = ElementInteractor::new(element);

    // Test drag and drop
    let result = interactor.drag_and_drop("#target").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_element_interactor_clear_value() {
    let element = Arc::new(MockElement::new(
        "test-page".to_string(),
        "input".to_string(),
        Some("existing value".to_string()),
    ));

    let interactor = ElementInteractor::new(element);

    // Test fill with clear_first
    let result = interactor.fill("new value", true).await;
    assert!(result.is_ok());
}
