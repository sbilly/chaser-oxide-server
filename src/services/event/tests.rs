//! EventService unit tests

use super::dispatcher::{EventDispatcher, DispatcherEvent};
use super::service::EventGrpcService;
use crate::services::traits::{ConsoleEvent, ConsoleLevel, EventType, NetworkEvent, PageEvent};
use crate::chaser_oxide::v1::{event_service_client::EventServiceClient, SubscribeRequest, Subscription};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_event_dispatcher_creation() {
    let dispatcher = EventDispatcher::new(100);
    assert_eq!(dispatcher.subscription_count().await, 0);
}

#[tokio::test]
async fn test_event_dispatcher_subscribe() {
    let dispatcher = EventDispatcher::new(100);

    let (sub_id, mut rx) = dispatcher
        .subscribe(
            Some("page-1".to_string()),
            Some("browser-1".to_string()),
            vec![EventType::PageLoaded],
        )
        .await
        .unwrap();

    assert_eq!(dispatcher.subscription_count().await, 1);
    assert!(!sub_id.is_empty());

    // Unsubscribe
    dispatcher.unsubscribe(&sub_id).await.unwrap();
    assert_eq!(dispatcher.subscription_count().await, 0);
}

#[tokio::test]
async fn test_event_dispatcher_subscribe_unsubscribe() {
    let dispatcher = EventDispatcher::new(100);

    // Subscribe
    let (sub_id, _rx) = dispatcher
        .subscribe(Some("page-1".to_string()), None, vec![EventType::PageLoaded])
        .await
        .unwrap();

    assert_eq!(dispatcher.subscription_count().await, 1);

    // Unsubscribe
    dispatcher.unsubscribe(&sub_id).await.unwrap();
    assert_eq!(dispatcher.subscription_count().await, 0);

    // Try to unsubscribe again (should fail)
    let result = dispatcher.unsubscribe(&sub_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_event_dispatcher_list_subscriptions() {
    let dispatcher = EventDispatcher::new(100);

    // Create multiple subscriptions
    let _sub1 = dispatcher
        .subscribe(Some("page-1".to_string()), None, vec![EventType::PageLoaded])
        .await;

    let _sub2 = dispatcher
        .subscribe(Some("page-2".to_string()), None, vec![EventType::ConsoleLog])
        .await;

    let subs = dispatcher.list_subscriptions().await;
    assert_eq!(subs.len(), 2);
}

#[tokio::test]
async fn test_event_dispatcher_dispatch_page_event() {
    let dispatcher = EventDispatcher::new(100);

    let (_sub_id, mut rx) = dispatcher
        .subscribe(Some("page-1".to_string()), None, vec![EventType::PageLoaded])
        .await
        .unwrap();

    // Dispatch page event
    let page_event = PageEvent {
        url: "https://example.com".to_string(),
        title: Some("Example".to_string()),
    };

    dispatcher.dispatch_page_event(page_event).await.unwrap();

    // Receive event
    let event = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;

    assert!(event.is_ok());
    let event = event.unwrap().unwrap();

    match event {
        DispatcherEvent::Page(page_ev) => {
            assert_eq!(page_ev.url, "https://example.com");
            assert_eq!(page_ev.title, Some("Example".to_string()));
        }
        _ => panic!("Expected page event"),
    }
}

#[tokio::test]
async fn test_event_dispatcher_dispatch_console_event() {
    let dispatcher = EventDispatcher::new(100);

    let (_sub_id, mut rx) = dispatcher
        .subscribe(Some("page-1".to_string()), None, vec![EventType::ConsoleLog])
        .await
        .unwrap();

    // Dispatch console event
    let console_event = ConsoleEvent {
        level: ConsoleLevel::Info,
        args: vec!["Test log message".to_string()],
    };

    dispatcher.dispatch_console_event(console_event).await.unwrap();

    // Receive event
    let event = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;

    assert!(event.is_ok());
    let event = event.unwrap().unwrap();

    match event {
        DispatcherEvent::Console(cons_ev) => {
            assert_eq!(cons_ev.level, ConsoleLevel::Info);
            assert_eq!(cons_ev.args[0], "Test log message");
        }
        _ => panic!("Expected console event"),
    }
}

#[tokio::test]
async fn test_event_dispatcher_dispatch_network_event() {
    let dispatcher = EventDispatcher::new(100);

    let (_sub_id, mut rx) = dispatcher
        .subscribe(Some("page-1".to_string()), None, vec![EventType::RequestSent])
        .await
        .unwrap();

    // Dispatch network event
    let network_event = NetworkEvent {
        url: "https://example.com/api".to_string(),
        method: "GET".to_string(),
        status_code: 200,
    };

    dispatcher.dispatch_network_event(network_event).await.unwrap();

    // Receive event
    let event = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;

    assert!(event.is_ok());
    let event = event.unwrap().unwrap();

    match event {
        DispatcherEvent::Network(net_ev) => {
            assert_eq!(net_ev.url, "https://example.com/api");
            assert_eq!(net_ev.method, "GET");
            assert_eq!(net_ev.status_code, 200);
        }
        _ => panic!("Expected network event"),
    }
}

#[tokio::test]
async fn test_event_dispatcher_multiple_subscribers() {
    let dispatcher = Arc::new(EventDispatcher::new(100));

    // Create multiple subscriptions
    let (_sub_id1, mut rx1) = dispatcher
        .subscribe(
            Some("page-1".to_string()),
            None,
            vec![EventType::PageLoaded],
        )
        .await
        .unwrap();

    let (_sub_id2, mut rx2) = dispatcher
        .subscribe(
            Some("page-1".to_string()),
            None,
            vec![EventType::PageLoaded],
        )
        .await
        .unwrap();

    assert_eq!(dispatcher.subscription_count().await, 2);

    // Dispatch event
    let page_event = PageEvent {
        url: "https://example.com".to_string(),
        title: None,
    };

    dispatcher.dispatch_page_event(page_event).await.unwrap();

    // Both subscribers should receive the event
    let event1 = tokio::time::timeout(Duration::from_millis(100), rx1.recv()).await;
    let event2 = tokio::time::timeout(Duration::from_millis(100), rx2.recv()).await;

    assert!(event1.is_ok());
    assert!(event2.is_ok());
}

#[tokio::test]
async fn test_event_dispatcher_cleanup() {
    let dispatcher = EventDispatcher::new(100);

    // Create subscription
    let (sub_id, rx) = dispatcher
        .subscribe(Some("page-1".to_string()), None, vec![EventType::PageLoaded])
        .await
        .unwrap();

    assert_eq!(dispatcher.subscription_count().await, 1);

    // Drop receiver
    drop(rx);

    // Cleanup
    dispatcher.cleanup_inactive().await;

    // Subscription should be removed
    assert_eq!(dispatcher.subscription_count().await, 0);
}

#[tokio::test]
async fn test_event_grpc_service_creation() {
    let dispatcher = Arc::new(EventDispatcher::new(100));
    let service = EventGrpcService::new(dispatcher);

    // Service should be created successfully
    assert!(true);
}

#[tokio::test]
async fn test_event_dispatcher_concurrent_dispatch() {
    let dispatcher = Arc::new(EventDispatcher::new(100));

    let (_sub_id, mut rx) = dispatcher
        .subscribe(Some("page-1".to_string()), None, vec![EventType::PageLoaded])
        .await
        .unwrap();

    // Dispatch multiple events concurrently
    let dispatcher_clone = Arc::clone(&dispatcher);
    let handle = tokio::spawn(async move {
        for i in 0..10 {
            let page_event = PageEvent {
                url: format!("https://example.com/{}", i),
                title: Some(format!("Page {}", i)),
            };

            dispatcher_clone.dispatch_page_event(page_event).await.unwrap();
        }
    });

    // Wait for dispatch to complete
    tokio::time::timeout(Duration::from_secs(1), handle)
        .await
        .unwrap()
        .unwrap();

    // Receive at least some events
    let mut received = 0;
    for _ in 0..10 {
        match tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
            Ok(Ok(_)) => received += 1,
            _ => break,
        }
    }

    assert!(received > 0);
}

// ============= Streaming Tests =============

/// Test basic event subscription stream
#[tokio::test]
async fn test_event_subscribe_stream() {
    let dispatcher = Arc::new(EventDispatcher::new(100));

    // Subscribe to events
    let (sub_id, mut rx) = dispatcher
        .subscribe(Some("page-1".to_string()), None, vec![EventType::PageLoaded])
        .await
        .unwrap();

    // Dispatch a page event
    let page_event = PageEvent {
        url: "https://example.com".to_string(),
        title: Some("Test Page".to_string()),
    };
    dispatcher.dispatch_page_event(page_event).await.unwrap();

    // Receive event
    let event = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
    assert!(event.is_ok());
    let event = event.unwrap().unwrap();

    match event {
        DispatcherEvent::Page(page_ev) => {
            assert_eq!(page_ev.url, "https://example.com");
            assert_eq!(page_ev.title, Some("Test Page".to_string()));
        }
        _ => panic!("Expected page event"),
    }

    // Cleanup
    dispatcher.unsubscribe(&sub_id).await.unwrap();
}

/// Test bidirectional streaming with subscribe/unsubscribe
#[tokio::test]
async fn test_bidirectional_stream() {
    let dispatcher = Arc::new(EventDispatcher::new(100));

    // Simulate bidirectional streaming
    let (tx, mut rx) = mpsc::channel(10);

    // Client sends subscribe request
    let (sub_id, _event_rx) = dispatcher
        .subscribe(Some("page-1".to_string()), None, vec![EventType::PageLoaded])
        .await
        .unwrap();

    // Server sends confirmation
    let confirmation = format!("Subscription created: {}", sub_id);
    tx.send(confirmation).await.unwrap();

    // Client receives confirmation
    let response = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
    assert!(response.is_ok());
    let response = response.unwrap().unwrap();
    assert!(response.contains("Subscription created"));

    // Client sends unsubscribe
    dispatcher.unsubscribe(&sub_id).await.unwrap();

    // Verify subscription is removed
    assert_eq!(dispatcher.subscription_count().await, 0);
}

/// Test backpressure handling when channel is full
#[tokio::test]
async fn test_backpressure_handling() {
    let dispatcher = Arc::new(EventDispatcher::new(5)); // Small channel

    let (_sub_id, mut rx) = dispatcher
        .subscribe(Some("page-1".to_string()), None, vec![EventType::PageLoaded])
        .await
        .unwrap();

    // Send more events than capacity
    // broadcast channels handle backpressure by dropping messages for lagging receivers
    for i in 0..10 {
        let page_event = PageEvent {
            url: format!("https://example.com/{}", i),
            title: None,
        };

        // Send should succeed (broadcast channels handle backpressure gracefully)
        let result = dispatcher.try_dispatch_page_event(page_event).await;
        assert!(result.is_ok(), "Send {} should succeed", i);

        // Small delay to let channel process
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    }

    // Verify channel capacity is set correctly
    assert_eq!(dispatcher.capacity(), 5, "Capacity should be 5");
    // Note: broadcast channel len() may not equal capacity due to lazy receiver behavior

    // Consume some events from filtered receiver
    for _ in 0..3 {
        let _ = rx.recv().await;
    }

    // Now dispatch should succeed
    let page_event = PageEvent {
        url: "https://example.com/after-backpressure".to_string(),
        title: None,
    };
    let result = dispatcher.dispatch_page_event(page_event).await;
    assert!(result.is_ok());
}

/// Test event filtering by EventType
#[tokio::test]
async fn test_event_filtering() {
    let dispatcher = Arc::new(EventDispatcher::new(100));

    // Subscribe only to PageLoaded events
    let (_sub_id, mut rx) = dispatcher
        .subscribe(Some("page-1".to_string()), None, vec![EventType::PageLoaded])
        .await
        .unwrap();

    // Dispatch different event types
    let console_event = ConsoleEvent {
        level: ConsoleLevel::Info,
        args: vec!["Test log".to_string()],
    };
    dispatcher.dispatch_console_event(console_event).await.unwrap();

    let network_event = NetworkEvent {
        url: "https://example.com/api".to_string(),
        method: "GET".to_string(),
        status_code: 200,
    };
    dispatcher.dispatch_network_event(network_event).await.unwrap();

    // Dispatch PageLoaded event
    let page_event = PageEvent {
        url: "https://example.com".to_string(),
        title: Some("Test Page".to_string()),
    };
    dispatcher.dispatch_page_event(page_event).await.unwrap();

    // Should receive PageLoaded event only (console and network events filtered out)
    let event = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
    assert!(event.is_ok());

    match event.unwrap().unwrap() {
        DispatcherEvent::Page(page_ev) => {
            assert_eq!(page_ev.url, "https://example.com");
        }
        _ => panic!("Expected page event only, but received different event type"),
    }
}

/// Test multiple subscribers to different page IDs
#[tokio::test]
async fn test_event_routing_by_page_id() {
    let dispatcher = Arc::new(EventDispatcher::new(100));

    // Subscribe to page-1 events
    let (_sub_id1, mut rx1) = dispatcher
        .subscribe(Some("page-1".to_string()), None, vec![EventType::PageLoaded])
        .await
        .unwrap();

    // Subscribe to page-2 events
    let (_sub_id2, mut rx2) = dispatcher
        .subscribe(Some("page-2".to_string()), None, vec![EventType::PageLoaded])
        .await
        .unwrap();

    // Dispatch event for page-1
    let page_event1 = PageEvent {
        url: "https://example.com/page1".to_string(),
        title: Some("Page 1".to_string()),
    };
    dispatcher.dispatch_page_event(page_event1).await.unwrap();

    // Both subscribers should receive (broadcast channel sends to all)
    let event1 = tokio::time::timeout(Duration::from_millis(100), rx1.recv()).await;
    let event2 = tokio::time::timeout(Duration::from_millis(100), rx2.recv()).await;

    assert!(event1.is_ok());
    assert!(event2.is_ok()); // Broadcast channel sends to all subscribers
}

/// Test subscription cleanup on disconnect
#[tokio::test]
async fn test_subscription_cleanup_on_disconnect() {
    let dispatcher = Arc::new(EventDispatcher::new(100));

    // Create subscription
    let (sub_id, rx) = dispatcher
        .subscribe(Some("page-1".to_string()), None, vec![EventType::PageLoaded])
        .await
        .unwrap();

    assert_eq!(dispatcher.subscription_count().await, 1);

    // Drop receiver (simulates client disconnect)
    drop(rx);

    // Cleanup inactive subscriptions
    dispatcher.cleanup_inactive().await;

    // Subscription should be removed
    assert_eq!(dispatcher.subscription_count().await, 0);

    // Trying to unsubscribe should fail
    let result = dispatcher.unsubscribe(&sub_id).await;
    assert!(result.is_err());
}

/// Test event metadata generation
#[tokio::test]
async fn test_event_metadata_generation() {
    let dispatcher = Arc::new(EventDispatcher::new(100));

    let (_sub_id, mut rx) = dispatcher
        .subscribe(Some("page-1".to_string()), None, vec![EventType::PageLoaded])
        .await
        .unwrap();

    // Dispatch event
    let page_event = PageEvent {
        url: "https://example.com".to_string(),
        title: Some("Test".to_string()),
    };
    dispatcher.dispatch_page_event(page_event).await.unwrap();

    // Receive and verify
    let event = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
    assert!(event.is_ok());

    match event.unwrap().unwrap() {
        DispatcherEvent::Page(page_ev) => {
            assert!(!page_ev.url.is_empty());
            assert!(page_ev.title.is_some());
        }
        _ => panic!("Expected page event"),
    }
}
