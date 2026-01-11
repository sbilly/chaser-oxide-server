//! EventService gRPC implementation
//!
//! Provides gRPC server implementation for event streaming operations.

use crate::Error;
use crate::services::event::dispatcher::{DispatcherEvent, EventDispatcher};
use crate::services::traits::{ConsoleEvent, ConsoleLevel, EventType, NetworkEvent, PageEvent};
use std::sync::Arc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

// Compile-time event type mapping using PHF
use phf::phf_map;

/// Static event type mapping from proto i32 to EventType enum
/// Uses compile-time hash map for O(1) lookup without runtime allocation
static EVENT_TYPE_MAP: phf::Map<i32, EventType> = phf_map! {
    1i32 => EventType::PageLoaded,
    2i32 => EventType::PageNavigated,
    3i32 => EventType::PageClosed,
    11i32 => EventType::ConsoleLog,
    15i32 => EventType::ConsoleError,
    17i32 => EventType::RequestSent,
    18i32 => EventType::ResponseReceived,
    21i32 => EventType::JsException,
    24i32 => EventType::DialogOpened,
};

/// Convert EventType enum to proto i32 value
const fn event_type_to_i32(event_type: EventType) -> i32 {
    match event_type {
        EventType::PageLoaded => 1,
        EventType::PageNavigated => 2,
        EventType::PageClosed => 3,
        EventType::ConsoleLog => 11,
        EventType::ConsoleError => 15,
        EventType::RequestSent => 17,
        EventType::ResponseReceived => 18,
        EventType::JsException => 21,
        EventType::DialogOpened => 24,
    }
}

// Import generated protobuf types
use crate::chaser_oxide::v1::{
    event_service_server::{EventService as EventServiceTrait, EventServiceServer},
    ConsoleEvent as ConsoleEventProto,
    NetworkEvent as NetworkEventProto,
    PageEvent as PageEventProto,
    LogLevel,
    *,
};

/// EventService gRPC server
#[derive(Clone)]
pub struct EventGrpcService {
    dispatcher: Arc<EventDispatcher>,
}

impl EventGrpcService {
    /// Create a new EventService gRPC server
    pub fn new(dispatcher: Arc<EventDispatcher>) -> Self {
        Self { dispatcher }
    }

    /// Convert to tonic server
    pub fn into_server(self) -> EventServiceServer<Self> {
        EventServiceServer::new(self)
    }

    /// Convert EventType from proto to trait
    fn convert_event_type(event_type: i32) -> std::result::Result<EventType, Error> {
        EVENT_TYPE_MAP
            .get(&event_type)
            .copied()
            .ok_or_else(|| Error::internal(format!("Invalid event type: {}", event_type)))
    }

    /// Convert PageEvent to proto
    #[allow(dead_code)]
    fn convert_page_event(event: &PageEvent, event_type: page_event::PageEventType) -> PageEventProto {
        PageEventProto {
            event_type: event_type as i32,
            url: event.url.clone(),
            title: event.title.clone().unwrap_or_default(),
            status_code: 0,
            error_message: String::new(),
            load_time: 0,
        }
    }

    /// Convert ConsoleEvent to proto
    fn convert_console_event(event: &ConsoleEvent) -> ConsoleEventProto {
        ConsoleEventProto {
            level: match event.level {
                ConsoleLevel::Log => LogLevel::Log as i32,
                ConsoleLevel::Debug => LogLevel::Debug as i32,
                ConsoleLevel::Info => LogLevel::Info as i32,
                ConsoleLevel::Warn => LogLevel::Warn as i32,
                ConsoleLevel::Error => LogLevel::Error as i32,
            },
            args: event.args.clone(),
            url: String::new(),
            line: 0,
            column: 0,
            stack_trace: String::new(),
        }
    }

    /// Convert NetworkEvent to proto
    fn convert_network_event(event: &NetworkEvent) -> NetworkEventProto {
        NetworkEventProto {
            event_type: network_event::NetworkEventType::Sent as i32,
            request_id: String::new(),
            url: event.url.clone(),
            method: event.method.clone(),
            headers: Default::default(),
            status_code: event.status_code as i32,
            from_cache: false,
            resource_type: String::new(),
            request_size: 0,
            response_size: 0,
            duration: 0.0,
            timing: 0.0,
        }
    }
}

#[tonic::async_trait]
impl EventServiceTrait for EventGrpcService {
    type SubscribeStream = ReceiverStream<std::result::Result<Event, Status>>;

    #[instrument(skip(self, request))]
    async fn subscribe(
        &self,
        request: Request<Streaming<SubscribeRequest>>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        info!("Subscribe request received");

        let mut stream = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        // Clone dispatcher reference for the task
        let dispatcher = Arc::clone(&self.dispatcher);

        // Spawn task to handle subscription requests
        tokio::spawn(async move {
            // Subscription ID
            let subscription_id = Arc::new(std::sync::RwLock::new(None::<String>));

            loop {
                match stream.message().await {
                    Ok(Some(req)) => {
                        debug!("Received subscribe request: action={:?}", req.action);

                        match req.action {
                            0 => {
                                // ACTION_UNSPECIFIED
                                warn!("Unspecified action");
                            }
                            1 => {
                                // ACTION_SUBSCRIBE
                                if let Some(subscription) = req.subscription {
                                    // Convert event types
                                    let event_types: std::result::Result<Vec<EventType>, Error> = subscription
                                        .event_types
                                        .into_iter()
                                        .map(Self::convert_event_type)
                                        .collect();

                                    if let Ok(types) = event_types {
                                        // Extract page_id and browser_id from subscription target
                                        let (page_id, browser_id) = match &subscription.target {
                                            Some(target) => match target {
                                                subscription::Target::PageId(id) => (Some(id.clone()), None),
                                                subscription::Target::BrowserId(id) => (None, Some(id.clone())),
                                                _ => (None, None),
                                            },
                                            None => (None, None),
                                        };

                                        match dispatcher
                                            .subscribe(
                                                page_id,
                                                browser_id,
                                                types,
                                            )
                                            .await
                                        {
                                            Ok((sub_id, mut event_rx)) => {
                                                // Store subscription ID
                                                *subscription_id.write().unwrap() = Some(sub_id.clone());

                                                info!("Subscription created: {}", sub_id);

                                                // Spawn task to forward events
                                                let tx_clone = tx.clone();
                                                let sub_id_clone = sub_id.clone();
                                                tokio::spawn(async move {
                                                    while let Ok(event) = event_rx.recv().await {
                                                        let proto_event = match event {
                                                            DispatcherEvent::Page(page_ev) => Event {
                                                                metadata: Some(EventMetadata {
                                                                    event_id: Uuid::new_v4().to_string(),
                                                                    r#type: event_type_to_i32(EventType::PageLoaded),
                                                                    timestamp: chrono::Utc::now().timestamp_millis(),
                                                                    browser_id: String::new(),
                                                                    page_id: String::new(),
                                                                    frame_id: String::new(),
                                                                    extra: Default::default(),
                                                                }),
                                                                data: Some(event::Data::PageEvent(
                                                                    PageEventProto {
                                                                        event_type: page_event::PageEventType::Loaded as i32,
                                                                        url: page_ev.url,
                                                                        title: page_ev.title.unwrap_or_default(),
                                                                        status_code: 0,
                                                                        error_message: String::new(),
                                                                        load_time: 0,
                                                                    },
                                                                )),
                                                                subscription_id: sub_id_clone.clone(),
                                                            },
                                                            DispatcherEvent::Console(console_ev) => Event {
                                                                metadata: Some(EventMetadata {
                                                                    event_id: Uuid::new_v4().to_string(),
                                                                    r#type: event_type_to_i32(EventType::ConsoleLog),
                                                                    timestamp: chrono::Utc::now().timestamp_millis(),
                                                                    browser_id: String::new(),
                                                                    page_id: String::new(),
                                                                    frame_id: String::new(),
                                                                    extra: Default::default(),
                                                                }),
                                                                data: Some(event::Data::ConsoleEvent(
                                                                    Self::convert_console_event(&console_ev),
                                                                )),
                                                                subscription_id: sub_id_clone.clone(),
                                                            },
                                                            DispatcherEvent::Network(net_ev) => Event {
                                                                metadata: Some(EventMetadata {
                                                                    event_id: Uuid::new_v4().to_string(),
                                                                    r#type: event_type_to_i32(EventType::RequestSent),
                                                                    timestamp: chrono::Utc::now().timestamp_millis(),
                                                                    browser_id: String::new(),
                                                                    page_id: String::new(),
                                                                    frame_id: String::new(),
                                                                    extra: Default::default(),
                                                                }),
                                                                data: Some(event::Data::NetworkEvent(
                                                                    Self::convert_network_event(&net_ev),
                                                                )),
                                                                subscription_id: sub_id_clone.clone(),
                                                            },
                                                        };

                                                        if tx_clone.send(Ok(proto_event)).await.is_err() {
                                                            break;
                                                        }
                                                    }
                                                });

                                                // Send success response
                                                let _ = tx
                                                    .send(Ok(Event {
                                                        metadata: Some(EventMetadata {
                                                            event_id: Uuid::new_v4().to_string(),
                                                            r#type: event_type_to_i32(EventType::PageLoaded),
                                                            timestamp: chrono::Utc::now().timestamp_millis(),
                                                            browser_id: String::new(),
                                                            page_id: String::new(),
                                                            frame_id: String::new(),
                                                            extra: Default::default(),
                                                        }),
                                                        data: None,
                                                        subscription_id: sub_id.clone(),
                                                    }))
                                                    .await;
                                            }
                                            Err(e) => {
                                                error!("Failed to create subscription: {}", e);
                                                let _ = tx
                                                    .send(Err(Status::internal(format!(
                                                        "Subscription failed: {}",
                                                        e
                                                    ))))
                                                    .await;
                                            }
                                        }
                                    }
                                }
                            }
                            2 => {
                                // ACTION_UNSUBSCRIBE
                                let sub_id = req.subscription_id;
                                if !sub_id.is_empty() {
                                    match dispatcher.unsubscribe(&sub_id).await {
                                        Ok(_) => {
                                            info!("Unsubscribed: {}", sub_id);
                                            *subscription_id.write().unwrap() = None;
                                        }
                                        Err(e) => {
                                            error!("Failed to unsubscribe: {}", e);
                                        }
                                    }
                                }
                            }
                            3 => {
                                // ACTION_LIST_SUBSCRIPTIONS
                                let subs = dispatcher.list_subscriptions().await;
                                info!("Listing subscriptions: count={}", subs.len());

                                // Send subscription list
                                let _ = tx
                                    .send(Ok(Event {
                                        metadata: Some(EventMetadata {
                                            event_id: Uuid::new_v4().to_string(),
                                            r#type: event_type_to_i32(EventType::PageLoaded),
                                            timestamp: chrono::Utc::now().timestamp_millis(),
                                            browser_id: String::new(),
                                            page_id: String::new(),
                                            frame_id: String::new(),
                                            extra: Default::default(),
                                        }),
                                        data: None,
                                        subscription_id: format!("{:?}", subs),
                                    }))
                                    .await;
                            }
                            4 => {
                                // ACTION_PING
                                debug!("Ping received");
                                // Send pong - use a valid event type for Ping events
                                let _ = tx
                                    .send(Ok(Event {
                                        metadata: Some(EventMetadata {
                                            event_id: Uuid::new_v4().to_string(),
                                            r#type: event_type_to_i32(EventType::PageLoaded), // Use a valid event type instead of 0
                                            timestamp: chrono::Utc::now().timestamp_millis(),
                                            browser_id: String::new(),
                                            page_id: String::new(),
                                            frame_id: String::new(),
                                            extra: Default::default(),
                                        }),
                                        data: None,
                                        subscription_id: "pong".to_string(),
                                    }))
                                    .await;
                            }
                            _ => {
                                warn!("Unknown action: {}", req.action);
                            }
                        }
                    }
                    Ok(None) => {
                        info!("Client closed stream");
                        break;
                    }
                    Err(e) => {
                        error!("Stream error: {}", e);
                        break;
                    }
                }
            }

            // Cleanup subscription when client disconnects
            // Extract subscription_id before await to avoid holding lock across await
            let sub_id_to_cleanup = subscription_id.read().unwrap().clone();
            if let Some(sub_id) = sub_id_to_cleanup {
                debug!("Cleaning up subscription: {}", sub_id);
                let _ = dispatcher.unsubscribe(&sub_id).await;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_creation() {
        let dispatcher = Arc::new(EventDispatcher::new(100));
        let service = EventGrpcService::new(dispatcher);
        assert!(true);
    }
}
