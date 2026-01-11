//! Event dispatcher module
//!
//! Provides event streaming and subscription management using broadcast channels.

use crate::error::{Error, Result};
use crate::services::traits::{ConsoleEvent, ConsoleLevel, EventType, NetworkEvent, PageEvent};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

/// Event filter function type
type EventFilter = Box<dyn Fn(&DispatcherEvent) -> bool + Send + Sync>;

/// Filtered receiver that only receives events matching specified event types
pub struct FilteredReceiver {
    /// Inner broadcast receiver
    inner: broadcast::Receiver<DispatcherEvent>,
    /// Subscription ID
    #[allow(dead_code)]
    id: String,
    /// Event types to filter by
    #[allow(dead_code)]
    event_types: Vec<EventType>,
    /// Created at timestamp
    #[allow(dead_code)]
    created_at: std::time::Instant,
    /// Filter function
    #[allow(dead_code)]
    filter: Option<EventFilter>,
    /// Subscriptions map (for checking subscription validity)
    #[allow(dead_code)]
    subscriptions: Arc<RwLock<HashMap<String, Subscription>>>,
}

impl FilteredReceiver {
    /// Create a new filtered receiver
    fn new(
        inner: broadcast::Receiver<DispatcherEvent>,
        subscription_id: String,
        event_types: Vec<EventType>,
        subscriptions: Arc<RwLock<HashMap<String, Subscription>>>,
    ) -> Self {
        Self {
            inner,
            id: subscription_id,
            event_types,
            created_at: std::time::Instant::now(),
            filter: None,
            subscriptions,
        }
    }

    /// Receive next filtered event
    pub async fn recv(&mut self) -> Result<DispatcherEvent> {
        loop {
            match self.inner.recv().await {
                Ok(event) => {
                    // Check if event type matches subscription
                    if self.matches_event_type(&event) {
                        return Ok(event);
                    }
                    // Otherwise, continue waiting for next event
                }
                Err(e) => {
                    return Err(Error::internal(format!(
                        "Failed to receive event: {}",
                        e
                    )));
                }
            }
        }
    }

    /// Try to receive next filtered event without blocking
    pub fn try_recv(&mut self) -> Result<DispatcherEvent> {
        loop {
            match self.inner.try_recv() {
                Ok(event) => {
                    if self.matches_event_type(&event) {
                        return Ok(event);
                    }
                    // Continue to next event
                }
                Err(broadcast::error::TryRecvError::Empty) => {
                    return Err(Error::internal("No events available".to_string()));
                }
                Err(broadcast::error::TryRecvError::Lagged(n)) => {
                    debug!(
                        "Receiver lagged behind by {} messages, catching up",
                        n
                    );
                    return Err(Error::internal(format!("Lagged by {} messages", n)));
                }
                Err(e) => {
                    return Err(Error::internal(format!("Channel error: {}", e)));
                }
            }
        }
    }

    /// Check if event matches the subscription's event types
    fn matches_event_type(&self, event: &DispatcherEvent) -> bool {
        // If no event types specified, receive all events
        if self.event_types.is_empty() {
            return true;
        }

        // Check if event type is in subscription list
        let event_type = match event {
            DispatcherEvent::Page(_) => EventType::PageLoaded,
            DispatcherEvent::Console(_) => EventType::ConsoleLog,
            DispatcherEvent::Network(_) => EventType::RequestSent,
        };

        self.event_types.contains(&event_type)
    }
}

/// Event dispatcher
///
/// Manages event subscriptions and broadcasts events to subscribers.
pub struct EventDispatcher {
    /// Broadcast channel for events
    tx: broadcast::Sender<DispatcherEvent>,
    /// Channel capacity
    channel_capacity: usize,
    /// Active subscriptions
    subscriptions: Arc<RwLock<HashMap<String, Subscription>>>,
}

/// Subscription filter
#[derive(Debug, Clone, Default)]
pub struct SubscriptionFilter {
    /// Filter by URL pattern
    pub url_pattern: Option<String>,
    /// Filter by status codes
    pub status_codes: Option<Vec<u16>>,
    /// Filter by resource types
    pub resource_types: Option<Vec<String>>,
    /// Filter by console log levels
    pub log_levels: Option<Vec<ConsoleLevel>>,
}

/// Subscription information
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Subscription {
    /// Subscription ID
    id: String,
    /// Page ID to filter events
    page_id: Option<String>,
    /// Browser ID to filter events
    browser_id: Option<String>,
    /// Event types to subscribe to
    event_types: Vec<EventType>,
    /// Subscription timestamp
    created_at: chrono::DateTime<chrono::Utc>,
    /// Event filters
    filter: SubscriptionFilter,
}

/// Event that can be dispatched
#[derive(Debug, Clone)]
pub enum DispatcherEvent {
    /// Page event
    Page(PageEvent),
    /// Console event
    Console(ConsoleEvent),
    /// Network event
    Network(NetworkEvent),
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new(channel_capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(channel_capacity);

        Self {
            tx,
            channel_capacity,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribe to events
    #[instrument(skip(self))]
    pub async fn subscribe(
        &self,
        page_id: Option<String>,
        browser_id: Option<String>,
        event_types: Vec<EventType>,
    ) -> Result<(String, FilteredReceiver)> {
        let subscription_id = Uuid::new_v4().to_string();

        // Create subscription with default filter
        let subscription = Subscription {
            id: subscription_id.clone(),
            page_id,
            browser_id,
            event_types: event_types.clone(),
            created_at: chrono::Utc::now(),
            filter: SubscriptionFilter::default(),
        };

        // Store subscription
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription_id.clone(), subscription);

        // Create receiver and wrap with filter
        let rx = self.tx.subscribe();
        let rx = FilteredReceiver::new(rx, subscription_id.clone(), event_types, self.subscriptions.clone());

        info!("Created subscription: {}", subscription_id);

        Ok((subscription_id, rx))
    }

    /// Subscribe to events with filters
    #[instrument(skip(self))]
    pub async fn subscribe_with_filters(
        &self,
        page_id: Option<String>,
        browser_id: Option<String>,
        event_types: Vec<EventType>,
        filter: SubscriptionFilter,
    ) -> Result<(String, FilteredReceiver)> {
        let subscription_id = Uuid::new_v4().to_string();

        // Create subscription with custom filter
        let subscription = Subscription {
            id: subscription_id.clone(),
            page_id,
            browser_id,
            event_types: event_types.clone(),
            created_at: chrono::Utc::now(),
            filter,
        };

        // Store subscription
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription_id.clone(), subscription);

        // Create receiver and wrap with filter
        let rx = self.tx.subscribe();
        let rx = FilteredReceiver::new(rx, subscription_id.clone(), event_types, self.subscriptions.clone());

        info!("Created subscription with filters: {}", subscription_id);

        Ok((subscription_id, rx))
    }

    /// Unsubscribe from events
    #[instrument(skip(self))]
    pub async fn unsubscribe(&self, subscription_id: &str) -> Result<()> {
        let mut subscriptions = self.subscriptions.write().await;

        if subscriptions.remove(subscription_id).is_some() {
            info!("Removed subscription: {}", subscription_id);
            Ok(())
        } else {
            warn!("Subscription not found: {}", subscription_id);
            Err(Error::internal(format!(
                "Subscription not found: {}",
                subscription_id
            )))
        }
    }

    /// List active subscriptions
    #[instrument(skip(self))]
    pub async fn list_subscriptions(&self) -> Vec<(String, Option<String>, Option<String>)> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions
            .iter()
            .map(|(id, sub)| (id.clone(), sub.page_id.clone(), sub.browser_id.clone()))
            .collect()
    }

    /// Dispatch a page event
    #[instrument(skip(self))]
    pub async fn dispatch_page_event(&self, event: PageEvent) -> Result<()> {
        debug!("Dispatching page event");

        match self.tx.send(DispatcherEvent::Page(event)) {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Failed to dispatch page event: {}", e);
                Err(Error::internal(format!("Failed to dispatch event: {}", e)))
            }
        }
    }

    /// Dispatch a console event
    #[instrument(skip(self))]
    pub async fn dispatch_console_event(&self, event: ConsoleEvent) -> Result<()> {
        debug!("Dispatching console event");

        match self.tx.send(DispatcherEvent::Console(event)) {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Failed to dispatch console event: {}", e);
                Err(Error::internal(format!("Failed to dispatch event: {}", e)))
            }
        }
    }

    /// Dispatch a network event
    #[instrument(skip(self))]
    pub async fn dispatch_network_event(&self, event: NetworkEvent) -> Result<()> {
        debug!("Dispatching network event");

        match self.tx.send(DispatcherEvent::Network(event)) {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Failed to dispatch network event: {}", e);
                Err(Error::internal(format!("Failed to dispatch event: {}", e)))
            }
        }
    }

    /// Check if the channel is at capacity (backpressure detection)
    pub fn is_at_capacity(&self) -> bool {
        self.tx.receiver_count() > 0 && self.tx.len() >= self.channel_capacity
    }

    /// Get current channel capacity
    pub fn capacity(&self) -> usize {
        self.channel_capacity
    }

    /// Get current channel length (number of unconsumed messages)
    pub fn len(&self) -> usize {
        self.tx.len()
    }

    /// Check if there are no unconsumed messages
    pub fn is_empty(&self) -> bool {
        self.tx.is_empty()
    }

    /// Try dispatch a page event with backpressure handling
    ///
    /// For broadcast channels, backpressure is handled by dropping old messages
    /// for lagging receivers rather than failing sends.
    #[instrument(skip(self))]
    pub async fn try_dispatch_page_event(&self, event: PageEvent) -> Result<()> {
        debug!("Trying to dispatch page event");

        // broadcast channels handle backpressure by dropping messages for lagging receivers
        // Send only fails if there are no receivers, which is expected behavior
        match self.tx.send(DispatcherEvent::Page(event)) {
            Ok(_) => Ok(()),
            Err(e) => {
                // This only happens when there are no receivers
                debug!("No receivers for page event: {}", e);
                Err(Error::internal(format!("No receivers: {}", e)))
            }
        }
    }

    /// Clean up inactive subscriptions
    #[instrument(skip(self))]
    pub async fn cleanup_inactive(&self) {
        let mut subscriptions = self.subscriptions.write().await;

        // Remove subscriptions with no receivers
        let receiver_count = self.tx.receiver_count();
        if receiver_count == 0 {
            subscriptions.clear();
            debug!("Cleaned up all subscriptions (no active receivers)");
        }
    }

    /// Get subscription count
    pub async fn subscription_count(&self) -> usize {
        self.subscriptions.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dispatcher_creation() {
        let dispatcher = EventDispatcher::new(100);
        assert_eq!(dispatcher.subscription_count().await, 0);
    }

    #[tokio::test]
    async fn test_subscribe() {
        let dispatcher = EventDispatcher::new(100);

        let (sub_id, mut rx) = dispatcher
            .subscribe(Some("page-1".to_string()), None, vec![EventType::PageLoaded])
            .await
            .unwrap();

        assert_eq!(dispatcher.subscription_count().await, 1);
        assert!(!sub_id.is_empty());

        // Unsubscribe
        dispatcher.unsubscribe(&sub_id).await.unwrap();
        assert_eq!(dispatcher.subscription_count().await, 0);
    }

    #[tokio::test]
    async fn test_dispatch_event() {
        let dispatcher = EventDispatcher::new(100);

        let (_sub_id, mut rx) = dispatcher
            .subscribe(Some("page-1".to_string()), None, vec![EventType::PageLoaded])
            .await
            .unwrap();

        // Dispatch event
        let page_event = PageEvent {
            url: "https://example.com".to_string(),
            title: Some("Example".to_string()),
        };

        dispatcher.dispatch_page_event(page_event).await.unwrap();

        // Receive event
        let event = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            rx.recv(),
        )
        .await;

        assert!(event.is_ok());
    }
}
