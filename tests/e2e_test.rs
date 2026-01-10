//! End-to-end integration tests
//!
//! These tests validate complete workflows from browser launch to interaction and cleanup.

mod common;
mod mock_chrome;

use chaser_oxide::session::{SessionManager, SessionManagerImpl, BrowserOptions, PageOptions};
use common::{setup_test_browser, setup_test_page, teardown_test_browser};
use std::sync::Arc;

/// Test 1: Browser lifecycle management
#[tokio::test]
async fn test_browser_lifecycle() {
    let session_manager = Arc::new(SessionManagerImpl::mock());

    // Create browser
    let browser_id = setup_test_browser(&session_manager).await.unwrap();
    assert!(!browser_id.is_empty());

    // Verify browser exists
    let browser = session_manager.get_browser(&browser_id).await.unwrap();
    assert_eq!(browser.id(), &browser_id);
    assert!(browser.is_active());

    // Close browser
    teardown_test_browser(&session_manager, &browser_id)
        .await
        .unwrap();

    // Verify browser is closed
    let result = session_manager.get_browser(&browser_id).await;
    assert!(result.is_err());
}

/// Test 2: Page creation and retrieval
#[tokio::test]
async fn test_page_creation() {
    let session_manager = Arc::new(SessionManagerImpl::mock());
    let browser_id = setup_test_browser(&session_manager).await.unwrap();

    // Create page
    let page = session_manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .unwrap();

    assert!(!page.id().is_empty());
    assert_eq!(page.browser_id(), &browser_id);
    assert!(page.is_active());

    // Retrieve page
    let retrieved_page = session_manager.get_page(page.id()).await.unwrap();
    assert_eq!(retrieved_page.id(), page.id());

    // Cleanup
    teardown_test_browser(&session_manager, &browser_id)
        .await
        .unwrap();
}

/// Test 3: Page navigation
#[tokio::test]
async fn test_page_navigation() {
    let session_manager = Arc::new(SessionManagerImpl::mock());
    let browser_id = setup_test_browser(&session_manager).await.unwrap();

    let page = session_manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .unwrap();

    // Navigate to data URL
    let url = "data:text/html;charset=utf-8,<h1>Test</h1>";
    let nav_result = page.navigate(url, Default::default()).await;

    assert!(nav_result.is_ok());
    let result = nav_result.unwrap();
    assert!(result.url.contains("data:text/html"));

    // Cleanup
    teardown_test_browser(&session_manager, &browser_id)
        .await
        .unwrap();
}

/// Test 4: Get page content
#[tokio::test]
async fn test_get_page_content() {
    let session_manager = Arc::new(SessionManagerImpl::mock());
    let browser_id = setup_test_browser(&session_manager).await.unwrap();

    let page = session_manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .unwrap();

    // Set content
    let html = "<!DOCTYPE html><html><body><h1>Test Content</h1></body></html>";
    page.set_content(html).await.unwrap();

    // Get content
    let content = page.get_content().await.unwrap();
    assert!(content.contains("<h1>Test Content</h1>"));

    // Cleanup
    teardown_test_browser(&session_manager, &browser_id)
        .await
        .unwrap();
}

/// Test 5: JavaScript evaluation
#[tokio::test]
async fn test_javascript_evaluation() {
    let session_manager = Arc::new(SessionManagerImpl::mock());
    let browser_id = setup_test_browser(&session_manager).await.unwrap();

    let page = session_manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .unwrap();

    // Set content with script
    let html = r#"<!DOCTYPE html><html><body><div id="test">Hello</div></body></html>"#;
    page.set_content(html).await.unwrap();

    // Evaluate JavaScript
    let result = page.evaluate("document.getElementById('test').textContent", false).await;
    assert!(result.is_ok());

    let eval_result = result.unwrap();
    match eval_result {
        chaser_oxide::session::EvaluationResult::String(s) => {
            assert_eq!(s, "Hello");
        }
        _ => panic!("Expected string result"),
    }

    // Cleanup
    teardown_test_browser(&session_manager, &browser_id)
        .await
        .unwrap();
}

/// Test 6: Screenshot capture
#[tokio::test]
async fn test_screenshot_capture() {
    let session_manager = Arc::new(SessionManagerImpl::mock());
    let browser_id = setup_test_browser(&session_manager).await.unwrap();

    let page = session_manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .unwrap();

    // Set content
    let html = r#"<!DOCTYPE html><html><body><h1>Screenshot Test</h1></body></html>"#;
    page.set_content(html).await.unwrap();

    // Capture screenshot
    let screenshot = page.screenshot(Default::default()).await;
    assert!(screenshot.is_ok());
    let data = screenshot.unwrap();
    assert!(!data.is_empty());

    // Cleanup
    teardown_test_browser(&session_manager, &browser_id)
        .await
        .unwrap();
}

/// Test 7: Multiple pages per browser
#[tokio::test]
async fn test_multiple_pages() {
    let session_manager = Arc::new(SessionManagerImpl::mock());
    let browser_id = setup_test_browser(&session_manager).await.unwrap();

    // Create multiple pages
    let page1 = session_manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .unwrap();

    let page2 = session_manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .unwrap();

    let page3 = session_manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .unwrap();

    // Verify all pages exist
    assert_ne!(page1.id(), page2.id());
    assert_ne!(page2.id(), page3.id());
    assert_ne!(page1.id(), page3.id());

    // Get browser and list pages
    let browser = session_manager.get_browser(&browser_id).await.unwrap();
    let pages = browser.get_pages().await.unwrap();
    assert_eq!(pages.len(), 3);

    // Cleanup
    teardown_test_browser(&session_manager, &browser_id)
        .await
        .unwrap();
}

/// Test 8: Page reload
#[tokio::test]
async fn test_page_reload() {
    let session_manager = Arc::new(SessionManagerImpl::mock());
    let browser_id = setup_test_browser(&session_manager).await.unwrap();

    let page = session_manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .unwrap();

    // Set content
    let html = r#"<!DOCTYPE html><html><body><h1>Original</h1></body></html>"#;
    page.set_content(html).await.unwrap();

    // Reload
    let reload_result = page.reload(false).await;
    assert!(reload_result.is_ok());

    // Verify page still active
    assert!(page.is_active());

    // Cleanup
    teardown_test_browser(&session_manager, &browser_id)
        .await
        .unwrap();
}

/// Test 9: Multiple browsers
#[tokio::test]
async fn test_multiple_browsers() {
    let session_manager = Arc::new(SessionManagerImpl::mock());

    // Create multiple browsers
    let browser1_id = setup_test_browser(&session_manager).await.unwrap();
    let browser2_id = setup_test_browser(&session_manager).await.unwrap();
    let browser3_id = setup_test_browser(&session_manager).await.unwrap();

    // Verify all browsers exist
    assert_ne!(browser1_id, browser2_id);
    assert_ne!(browser2_id, browser3_id);

    // List browsers
    let browsers = session_manager.list_browsers().await.unwrap();
    assert_eq!(browsers.len(), 3);

    // Verify session count
    assert_eq!(session_manager.session_count(), 3);

    // Cleanup
    teardown_test_browser(&session_manager, &browser1_id).await.unwrap();
    teardown_test_browser(&session_manager, &browser2_id).await.unwrap();
    teardown_test_browser(&session_manager, &browser3_id).await.unwrap();
}

/// Test 10: Session cleanup
#[tokio::test]
async fn test_session_cleanup() {
    let session_manager = Arc::new(SessionManagerImpl::mock());

    // Create browser and page
    let browser_id = setup_test_browser(&session_manager).await.unwrap();
    let _page = session_manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .unwrap();

    // Close browser (marks as inactive)
    session_manager.close_browser(&browser_id).await.unwrap();

    // Run cleanup
    session_manager.cleanup().await.unwrap();

    // Verify session count is 0 after cleanup
    assert_eq!(session_manager.session_count(), 0);
}

/// Test 11: Page close and cleanup
#[tokio::test]
async fn test_page_close() {
    let session_manager = Arc::new(SessionManagerImpl::mock());
    let browser_id = setup_test_browser(&session_manager).await.unwrap();

    // Create page
    let page = session_manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .unwrap();

    let page_id = page.id().to_string();

    // Close page
    page.close().await.unwrap();

    // Verify page is inactive
    let retrieved_page = session_manager.get_page(&page_id).await.unwrap();
    assert!(!retrieved_page.is_active());

    // Cleanup
    teardown_test_browser(&session_manager, &browser_id)
        .await
        .unwrap();
}

/// Test 12: Viewport manipulation
#[tokio::test]
async fn test_viewport_manipulation() {
    let session_manager = Arc::new(SessionManagerImpl::mock());
    let browser_id = setup_test_browser(&session_manager).await.unwrap();

    let page = session_manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .unwrap();

    // Set viewport
    let viewport_result = page.set_viewport(800, 600, 1.0).await;
    assert!(viewport_result.is_ok());

    // Cleanup
    teardown_test_browser(&session_manager, &browser_id)
        .await
        .unwrap();
}

/// Test 13: Concurrent operations
#[tokio::test]
async fn test_concurrent_operations() {
    let session_manager = Arc::new(SessionManagerImpl::mock());
    let browser_id = setup_test_browser(&session_manager).await.unwrap();

    // Create multiple pages concurrently
    let mut handles = Vec::new();

    for _ in 0..10 {
        let sm = session_manager.clone();
        let bid = browser_id.clone();

        handles.push(tokio::spawn(async move {
            sm.create_page(&bid, PageOptions::default()).await
        }));
    }

    // Wait for all to complete
    let mut page_ids = Vec::new();
    for handle in handles {
        let page = handle.await.unwrap().unwrap();
        page_ids.push(page.id().to_string());
    }

    assert_eq!(page_ids.len(), 10);

    // Verify all unique
    let unique_ids: std::collections::HashSet<_> = page_ids.iter().collect();
    assert_eq!(unique_ids.len(), 10);

    // Cleanup
    teardown_test_browser(&session_manager, &browser_id)
        .await
        .unwrap();
}

/// Test 14: Error handling - browser not found
#[tokio::test]
async fn test_error_browser_not_found() {
    let session_manager = Arc::new(SessionManagerImpl::mock());

    let result = session_manager.get_browser("non-existent-browser").await;
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("not found"));
    }
}

/// Test 15: Error handling - page not found
#[tokio::test]
async fn test_error_page_not_found() {
    let session_manager = Arc::new(SessionManagerImpl::mock());

    let result = session_manager.get_page("non-existent-page").await;
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("not found"));
    }
}

/// Test 16: Complete workflow
#[tokio::test]
async fn test_complete_workflow() {
    let session_manager = Arc::new(SessionManagerImpl::mock());

    // Step 1: Create browser
    let browser_id = session_manager
        .create_browser(BrowserOptions::default())
        .await
        .unwrap();
    assert!(!browser_id.is_empty());

    // Step 2: Create page
    let page = session_manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .unwrap();
    assert!(page.is_active());

    // Step 3: Set content
    let html = r#"<!DOCTYPE html><html><body><h1 id="title">Complete Workflow</h1></body></html>"#;
    page.set_content(html).await.unwrap();

    // Step 4: Evaluate script
    let result: Result<chaser_oxide::session::EvaluationResult, chaser_oxide::Error> =
        page.evaluate("document.getElementById('title').textContent", false).await;
    assert!(result.is_ok());

    // Step 5: Screenshot
    let screenshot: Result<Vec<u8>, chaser_oxide::Error> =
        page.screenshot(chaser_oxide::session::ScreenshotOptions::default()).await;
    assert!(screenshot.is_ok());

    // Step 6: Close page
    page.close().await.unwrap();

    // Step 7: Close browser
    session_manager.close_browser(&browser_id).await.unwrap();

    // Step 8: Verify cleanup
    let result: Result<std::sync::Arc<dyn chaser_oxide::session::BrowserContext>, chaser_oxide::Error> =
        session_manager.get_browser(&browser_id).await;
    assert!(result.is_err());
}
