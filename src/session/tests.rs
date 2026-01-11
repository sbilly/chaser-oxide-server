//! Integration tests for session management
//!
//! Comprehensive tests for browser, page, and element lifecycle management.

use std::sync::Arc;
use tokio::time::{timeout, Duration};

use crate::session::manager::SessionManagerImpl;
use crate::session::traits::{
    BrowserOptions, ElementRef, LoadState, NavigationOptions, PageOptions,
    ScreenshotFormat, ScreenshotOptions, SessionManager,
};

/// Helper function to create a test session manager
fn create_test_manager() -> Arc<SessionManagerImpl> {
    Arc::new(SessionManagerImpl::mock())
}

#[tokio::test]
async fn test_browser_lifecycle() {
    let manager = create_test_manager();

    // Create browser
    let browser_id = manager
        .create_browser(BrowserOptions {
            headless: true,
            window_width: 1920,
            window_height: 1080,
            ..Default::default()
        })
        .await
        .expect("Failed to create browser");

    // Verify browser exists
    let browser = manager
        .get_browser(&browser_id)
        .await
        .expect("Failed to get browser");
    assert_eq!(browser.id(), &browser_id);
    assert!(browser.is_active());

    // List browsers
    let browsers = manager.list_browsers().await.expect("Failed to list browsers");
    assert_eq!(browsers.len(), 1);
    assert_eq!(browsers[0], browser_id);

    // Close browser
    browser.close().await.expect("Failed to close browser");
    assert!(!browser.is_active());

    // Cleanup
    manager.cleanup().await.expect("Failed to cleanup");
    assert_eq!(manager.session_count(), 0);
}

#[tokio::test]
async fn test_page_lifecycle() {
    let manager = create_test_manager();

    // Create browser
    let browser_id = manager
        .create_browser(BrowserOptions::default())
        .await
        .expect("Failed to create browser");

    // Create page
    let page = manager
        .create_page(
            &browser_id,
            PageOptions {
                default_url: Some("about:blank".to_string()),
                viewport_width: 1920,
                viewport_height: 1080,
                ..Default::default()
            },
        )
        .await
        .expect("Failed to create page");

    assert!(!page.id().is_empty());
    assert_eq!(page.browser_id(), &browser_id);
    assert!(page.is_active());

    // Get page
    let retrieved_page = manager
        .get_page(page.id())
        .await
        .expect("Failed to get page");
    assert_eq!(retrieved_page.id(), page.id());

    // Close page
    page.close().await.expect("Failed to close page");
    assert!(!page.is_active());
}

#[tokio::test]
async fn test_page_navigation() {
    let manager = create_test_manager();

    // Create browser and page
    let browser_id = manager
        .create_browser(BrowserOptions::default())
        .await
        .expect("Failed to create browser");
    let page = manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .expect("Failed to create page");

    // Navigate
    let result = page
        .navigate(
            "https://example.com",
            NavigationOptions {
                timeout: 30000,
                wait_until: LoadState::Load,
            },
        )
        .await
        .expect("Failed to navigate");

    assert_eq!(result.url, "https://example.com");
    assert_eq!(result.status_code, 200);
}

#[tokio::test]
async fn test_page_content() {
    let manager = create_test_manager();

    // Create browser and page
    let browser_id = manager
        .create_browser(BrowserOptions::default())
        .await
        .expect("Failed to create browser");
    let page = manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .expect("Failed to create page");

    // Set content
    page.set_content("<html><body><h1>Test</h1></body></html>")
        .await
        .expect("Failed to set content");

    // Get content
    let content = page.get_content().await.expect("Failed to get content");
    assert!(content.contains("Test"));
}

#[tokio::test]
async fn test_page_evaluate() {
    let manager = create_test_manager();

    // Create browser and page
    let browser_id = manager
        .create_browser(BrowserOptions::default())
        .await
        .expect("Failed to create browser");
    let page = manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .expect("Failed to create page");

    // Evaluate script
    let result = page
        .evaluate("document.title", false)
        .await
        .expect("Failed to evaluate script");

    match result {
        crate::session::traits::EvaluationResult::String(title) => {
            assert!(!title.is_empty());
        }
        _ => panic!("Expected string result"),
    }

    // Evaluate number
    let result = page
        .evaluate("1 + 1", false)
        .await
        .expect("Failed to evaluate script");
    match result {
        crate::session::traits::EvaluationResult::Number(n) => {
            assert_eq!(n, 2.0);
        }
        _ => panic!("Expected number result"),
    }
}

#[tokio::test]
async fn test_page_screenshot() {
    let manager = create_test_manager();

    // Create browser and page
    let browser_id = manager
        .create_browser(BrowserOptions::default())
        .await
        .expect("Failed to create browser");
    let page = manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .expect("Failed to create page");

    // Take screenshot
    let screenshot = page
        .screenshot(ScreenshotOptions {
            format: ScreenshotFormat::Png,
            quality: None,
            full_page: false,
            clip: None,
        })
        .await
        .expect("Failed to take screenshot");

    assert!(!screenshot.is_empty());
}

#[tokio::test]
async fn test_page_viewport() {
    let manager = create_test_manager();

    // Create browser and page
    let browser_id = manager
        .create_browser(BrowserOptions::default())
        .await
        .expect("Failed to create browser");
    let page = manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .expect("Failed to create page");

    // Set viewport
    page.set_viewport(1280, 720, 1.0)
        .await
        .expect("Failed to set viewport");
}

#[tokio::test]
async fn test_page_navigation_history() {
    let manager = create_test_manager();

    // Create browser and page
    let browser_id = manager
        .create_browser(BrowserOptions::default())
        .await
        .expect("Failed to create browser");
    let page = manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .expect("Failed to create page");

    // Navigate to different URLs
    page.navigate("https://example.com", NavigationOptions::default())
        .await
        .expect("Failed to navigate");
    page.navigate("https://example.org", NavigationOptions::default())
        .await
        .expect("Failed to navigate");

    // Go back
    page.go_back().await.expect("Failed to go back");

    // Go forward
    page.go_forward().await.expect("Failed to go forward");
}

#[tokio::test]
async fn test_page_reload() {
    let manager = create_test_manager();

    // Create browser and page
    let browser_id = manager
        .create_browser(BrowserOptions::default())
        .await
        .expect("Failed to create browser");
    let page = manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .expect("Failed to create page");

    // Reload without cache
    page.reload(true).await.expect("Failed to reload");

    // Reload with cache
    page.reload(false).await.expect("Failed to reload");
}

#[tokio::test]
async fn test_multiple_pages_per_browser() {
    let manager = create_test_manager();

    // Create browser
    let browser_id = manager
        .create_browser(BrowserOptions::default())
        .await
        .expect("Failed to create browser");

    // Create multiple pages
    let page1 = manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .expect("Failed to create page 1");
    let page2 = manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .expect("Failed to create page 2");
    let page3 = manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .expect("Failed to create page 3");

    // Verify all pages exist
    let browser = manager
        .get_browser(&browser_id)
        .await
        .expect("Failed to get browser");
    let pages = browser.get_pages().await.expect("Failed to get pages");
    assert_eq!(pages.len(), 3);

    // Verify each page (using contains since HashMap doesn't guarantee order)
    let page_ids: Vec<_> = pages.iter().map(|p| p.id()).collect();
    assert!(page_ids.contains(&page1.id()));
    assert!(page_ids.contains(&page2.id()));
    assert!(page_ids.contains(&page3.id()));
}

#[tokio::test]
async fn test_multiple_browsers() {
    let manager = create_test_manager();

    // Create multiple browsers
    let browser1_id = manager
        .create_browser(BrowserOptions::default())
        .await
        .expect("Failed to create browser 1");
    let browser2_id = manager
        .create_browser(BrowserOptions::default())
        .await
        .expect("Failed to create browser 2");
    let browser3_id = manager
        .create_browser(BrowserOptions::default())
        .await
        .expect("Failed to create browser 3");

    // Verify all browsers exist
    assert_eq!(manager.session_count(), 3);

    let browsers = manager.list_browsers().await.expect("Failed to list browsers");
    assert_eq!(browsers.len(), 3);
    assert!(browsers.contains(&browser1_id));
    assert!(browsers.contains(&browser2_id));
    assert!(browsers.contains(&browser3_id));
}

#[tokio::test]
async fn test_concurrent_browsers() {
    let manager = create_test_manager();
    let mut handles = Vec::new();

    // Create 10 browsers concurrently
    for _ in 0..10 {
        let manager_clone = manager.clone();
        handles.push(tokio::spawn(async move {
            manager_clone
                .create_browser(BrowserOptions::default())
                .await
        }));
    }

    // Wait for all to complete
    let mut results = Vec::new();
    for handle in handles {
        results.push(timeout(Duration::from_secs(5), handle).await.unwrap().unwrap());
    }

    // All should succeed
    for result in results {
        assert!(result.is_ok());
    }

    assert_eq!(manager.session_count(), 10);
}

#[tokio::test]
async fn test_concurrent_pages() {
    let manager = create_test_manager();

    // Create browser
    let browser_id = manager
        .create_browser(BrowserOptions::default())
        .await
        .expect("Failed to create browser");

    let mut handles = Vec::new();

    // Create 20 pages concurrently
    for _ in 0..20 {
        let manager_clone = manager.clone();
        let browser_id_clone = browser_id.clone();
        handles.push(tokio::spawn(async move {
            manager_clone
                .create_page(&browser_id_clone, PageOptions::default())
                .await
        }));
    }

    // Wait for all to complete
    for handle in handles {
        timeout(Duration::from_secs(5), handle)
            .await
            .unwrap()
            .unwrap()
            .expect("Failed to create page");
    }

    // Verify all pages exist
    let browser = manager
        .get_browser(&browser_id)
        .await
        .expect("Failed to get browser");
    let pages = browser.get_pages().await.expect("Failed to get pages");
    assert_eq!(pages.len(), 20);
}

#[tokio::test]
async fn test_error_browser_not_found() {
    let manager = create_test_manager();

    let result = manager.get_browser("non-existent").await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        crate::Error::BrowserNotFound(_)
    ));

    let result = manager.close_browser("non-existent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_error_page_not_found() {
    let manager = create_test_manager();

    let result = manager.get_page("non-existent").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), crate::Error::PageNotFound(_)));

    let result = manager.close_page("non-existent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_cleanup_inactive_browsers() {
    let manager = create_test_manager();

    // Create browsers
    let browser1_id = manager
        .create_browser(BrowserOptions::default())
        .await
        .expect("Failed to create browser 1");
    let browser2_id = manager
        .create_browser(BrowserOptions::default())
        .await
        .expect("Failed to create browser 2");

    assert_eq!(manager.session_count(), 2);

    // Close one browser
    manager
        .close_browser(&browser1_id)
        .await
        .expect("Failed to close browser");

    // Cleanup
    manager.cleanup().await.expect("Failed to cleanup");

    // Should still have 1 active browser
    assert_eq!(manager.session_count(), 1);

    // Verify the correct browser remains
    let browsers = manager.list_browsers().await.expect("Failed to list browsers");
    assert_eq!(browsers.len(), 1);
    assert_eq!(browsers[0], browser2_id);
}

#[tokio::test]
async fn test_cascade_closing() {
    let manager = create_test_manager();

    // Create browser with pages
    let browser_id = manager
        .create_browser(BrowserOptions::default())
        .await
        .expect("Failed to create browser");

    manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .expect("Failed to create page 1");
    manager
        .create_page(&browser_id, PageOptions::default())
        .await
        .expect("Failed to create page 2");

    // Close browser
    manager
        .close_browser(&browser_id)
        .await
        .expect("Failed to close browser");

    // Pages should be closed
    let browser = manager.get_browser(&browser_id).await;
    assert!(browser.is_err());
}

#[tokio::test]
async fn test_isolation_between_browsers() {
    let manager = create_test_manager();

    // Create two browsers
    let browser1_id = manager
        .create_browser(BrowserOptions::default())
        .await
        .expect("Failed to create browser 1");
    let browser2_id = manager
        .create_browser(BrowserOptions::default())
        .await
        .expect("Failed to create browser 2");

    // Create pages in each
    let page1 = manager
        .create_page(&browser1_id, PageOptions::default())
        .await
        .expect("Failed to create page 1");
    let page2 = manager
        .create_page(&browser2_id, PageOptions::default())
        .await
        .expect("Failed to create page 2");

    // Verify pages belong to correct browsers
    assert_eq!(page1.browser_id(), &browser1_id);
    assert_eq!(page2.browser_id(), &browser2_id);

    // Close browser 1
    manager
        .close_browser(&browser1_id)
        .await
        .expect("Failed to close browser 1");

    // Browser 2 and its page should still be active
    let browser2 = manager
        .get_browser(&browser2_id)
        .await
        .expect("Failed to get browser 2");
    assert!(browser2.is_active());

    assert!(page2.is_active());
}
