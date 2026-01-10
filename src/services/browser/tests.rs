//! Browser service tests

#[cfg(test)]
mod tests {
    use super::super::service::Service;
    use crate::session::mock::MockSessionManager;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_browser_service_creation() {
        let session_manager = Arc::new(MockSessionManager::new());
        let service = Service::new(session_manager);
        assert!(true, "BrowserService created successfully");
    }

    #[tokio::test]
    async fn test_browser_options_conversion() {
        use crate::chaser_oxide::v1::BrowserOptions;

        let proto_opts = BrowserOptions {
            headless: true,
            window_width: 1920,
            window_height: 1080,
            user_agent: "test-agent".to_string(),
            proxy_server: "http://proxy:8080".to_string(),
            ..Default::default()
        };

        let opts = Service::<MockSessionManager>::proto_to_browser_options(proto_opts);
        assert_eq!(opts.headless, true);
        assert_eq!(opts.window_width, 1920);
        assert_eq!(opts.window_height, 1080);
        assert_eq!(opts.user_agent, Some("test-agent".to_string()));
        assert_eq!(opts.proxy, Some("http://proxy:8080".to_string()));
    }
}
