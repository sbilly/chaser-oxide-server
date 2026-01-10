//! Page service tests

#[cfg(test)]
mod tests {
    use super::super::Service;
    use crate::session::mock::MockSessionManager;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_page_service_creation() {
        let session_manager = Arc::new(MockSessionManager::new());
        let service = Service::new(session_manager);
        assert!(true, "PageService created successfully");
    }

    #[tokio::test]
    async fn test_navigation_options_conversion() {
        use crate::chaser_oxide::v1::NavigationOptions;

        let proto_opts = NavigationOptions {
            timeout: 30000,
            wait_until: crate::chaser_oxide::v1::navigation_options::LoadState::NetworkIdle as i32,
            ..Default::default()
        };

        let opts = Service::<MockSessionManager>::proto_to_navigation_options(proto_opts);
        assert_eq!(opts.timeout, 30000);
    }

    #[tokio::test]
    async fn test_evaluation_result_conversion() {
        use crate::services::traits::EvaluationResult;

        let result = EvaluationResult::String("test".to_string());
        let proto = Service::<MockSessionManager>::evaluation_result_to_proto(result);
        assert!(matches!(
            proto.response,
            Some(crate::chaser_oxide::v1::evaluation_result::Response::StringValue(_))
        ));
        assert_eq!(proto.r#type, "string");
    }
}
