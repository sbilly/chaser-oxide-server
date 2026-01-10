//! Common test utilities
//!
//! This module provides shared test helpers and fixtures for all integration tests.

use chaser_oxide::session::{
    SessionManager, BrowserOptions, PageOptions, NavigationOptions,
};
use std::sync::Arc;

/// Setup a test browser with default options
pub async fn setup_test_browser<S>(
    session_manager: &Arc<S>,
) -> Result<String, Box<dyn std::error::Error>>
where
    S: SessionManager + Send + Sync + 'static,
{
    let options = BrowserOptions {
        headless: true,
        window_width: 1920,
        window_height: 1080,
        ..Default::default()
    };

    let browser_id = session_manager.create_browser(options).await?;
    Ok(browser_id)
}

/// Setup a test page and navigate to URL
pub async fn setup_test_page<S>(
    session_manager: &Arc<S>,
    browser_id: &str,
    url: &str,
) -> Result<String, Box<dyn std::error::Error>>
where
    S: SessionManager + Send + Sync + 'static,
{
    let page = session_manager
        .create_page(browser_id, PageOptions::default())
        .await?;

    let nav_options = NavigationOptions::default();
    page.navigate(url, nav_options).await?;

    Ok(page.id().to_string())
}

/// Teardown test browser
pub async fn teardown_test_browser<S>(
    session_manager: &Arc<S>,
    browser_id: &str,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: SessionManager + Send + Sync + 'static,
{
    session_manager.close_browser(browser_id).await?;
    Ok(())
}

/// Wait for page to load
pub async fn wait_for_load<S>(
    session_manager: &Arc<S>,
    page_id: &str,
    timeout_ms: u64,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: SessionManager + Send + Sync + 'static,
{
    let page = session_manager.get_page(page_id).await?;

    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(timeout_ms);

    while start.elapsed() < timeout {
        // Simple polling - in real implementation you'd use proper load events
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    Ok(())
}

/// Get test HTML content
pub fn get_test_html() -> String {
    r#"
<!DOCTYPE html>
<html>
<head>
    <title>Test Page</title>
</head>
<body>
    <h1 id="title">Hello World</h1>
    <button id="click-me">Click Me</button>
    <input id="text-input" type="text" />
    <div id="output"></div>
</body>
</html>
    "#.to_string()
}

/// Create a simple test server URL
pub fn get_test_url() -> String {
    "data:text/html;charset=utf-8,".to_string()
        + &urlencoding::encode(&get_test_html())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chaser_oxide::session::SessionManagerImpl;

    #[tokio::test]
    async fn test_setup_teardown() {
        let session_manager = Arc::new(SessionManagerImpl::mock());

        let browser_id = setup_test_browser(&session_manager).await.unwrap();
        assert!(!browser_id.is_empty());

        teardown_test_browser(&session_manager, &browser_id)
            .await
            .unwrap();
    }
}
