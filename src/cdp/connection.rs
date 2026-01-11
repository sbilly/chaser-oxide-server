//! CDP WebSocket connection implementation
//!
//! This module provides WebSocket-based connection to Chrome DevTools Protocol.

use super::types::*;
use super::traits::{CdpConnection, CdpEvent, CdpError as CdpErrorResponse, CdpResponse};
use crate::Error;
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
use tracing::{debug, error, info, warn};

/// CDP timeout configuration
#[derive(Debug, Clone)]
struct CdpTimeoutConfig {
    /// Default timeout for most commands (seconds)
    default_timeout_secs: u64,
    /// Timeout for screenshot commands (seconds)
    screenshot_timeout_secs: u64,
    /// Timeout for page navigation commands (seconds)
    navigation_timeout_secs: u64,
    /// Timeout for JavaScript execution (seconds)
    execution_timeout_secs: u64,
}

impl Default for CdpTimeoutConfig {
    fn default() -> Self {
        Self {
            default_timeout_secs: 30,
            screenshot_timeout_secs: 90,
            navigation_timeout_secs: 60,
            execution_timeout_secs: 30,
        }
    }
}

impl CdpTimeoutConfig {
    /// Get timeout duration for a specific command method
    fn get_timeout_for_command(&self, method: &str) -> tokio::time::Duration {
        let method_lower = method.to_lowercase();

        // Screenshot commands need longer timeout
        if method_lower.contains("screenshot")
            || method_lower.contains("capture")
            || method_lower.contains("page.capture")
        {
            return tokio::time::Duration::from_secs(self.screenshot_timeout_secs);
        }

        // Navigation commands
        if method_lower.contains("navigate")
            || method_lower.starts_with("page.navigate")
            || method_lower.contains("reload")
        {
            return tokio::time::Duration::from_secs(self.navigation_timeout_secs);
        }

        // JavaScript execution
        if method_lower.contains("runtime.evaluate")
            || method_lower.contains("runtime.call")
        {
            return tokio::time::Duration::from_secs(self.execution_timeout_secs);
        }

        // Default timeout
        tokio::time::Duration::from_secs(self.default_timeout_secs)
    }
}

/// WebSocket connection state
#[derive(Debug, Clone, Copy, PartialEq)]
enum ConnectionState {
    Connecting,
    Connected,
    Disconnected,
    Closed,
}

/// Pending command response
#[derive(Debug)]
struct PendingCommand {
    /// Response channel sender
    sender: tokio::sync::oneshot::Sender<CdpResponse>,
    /// Command method (for logging)
    method: String,
}

/// CDP WebSocket connection implementation
#[derive(Debug)]
pub struct CdpWebSocketConnection {
    /// WebSocket URL
    url: String,
    /// WebSocket stream
    ws_stream: Arc<Mutex<Option<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>>>,
    /// Connection state
    state: Arc<RwLock<ConnectionState>>,
    /// Next command ID
    next_id: Arc<AtomicU64>,
    /// Pending commands (ID -> response sender)
    pending_commands: Arc<Mutex<std::collections::HashMap<u64, PendingCommand>>>,
    /// Event subscribers
    event_subscribers: Arc<Mutex<Vec<tokio::sync::mpsc::UnboundedSender<CdpEvent>>>>,
    /// Is connection active
    is_active: Arc<AtomicBool>,
    /// Timeout configuration
    timeout_config: CdpTimeoutConfig,
}

impl CdpWebSocketConnection {
    /// Create a new CDP WebSocket connection
    ///
    /// # Arguments
    /// * `url` - WebSocket URL (e.g., "ws://localhost:9222/devtools/page/ABC123")
    pub async fn new<S: Into<String>>(url: S) -> Result<Arc<Self>, Error> {
        let url = url.into();
        info!("Creating CDP WebSocket connection to {}", url);

        let connection = Arc::new(Self {
            url,
            ws_stream: Arc::new(Mutex::new(None)),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            next_id: Arc::new(AtomicU64::new(1)),
            pending_commands: Arc::new(Mutex::new(std::collections::HashMap::new())),
            event_subscribers: Arc::new(Mutex::new(Vec::new())),
            is_active: Arc::new(AtomicBool::new(false)),
            timeout_config: CdpTimeoutConfig::default(),
        });

        // Connect to WebSocket
        connection.connect().await?;

        Ok(connection)
    }

    /// Establish WebSocket connection
    async fn connect(&self) -> Result<(), Error> {
        let mut state = self.state.write().await;
        if *state != ConnectionState::Disconnected {
            return Err(Error::internal("Connection is not in disconnected state"));
        }

        *state = ConnectionState::Connecting;
        drop(state);

        info!("Connecting to WebSocket: {}", self.url);

        match connect_async(&self.url).await {
            Ok((ws_stream, _)) => {
                let mut stream_guard = self.ws_stream.lock().await;
                *stream_guard = Some(ws_stream);
                drop(stream_guard);

                let mut state = self.state.write().await;
                *state = ConnectionState::Connected;
                self.is_active.store(true, Ordering::SeqCst);
                drop(state);

                info!("WebSocket connection established");

                // Start message loop - we need to clone the Arcs we need
                let ws_stream = Arc::clone(&self.ws_stream);
                let pending_commands = Arc::clone(&self.pending_commands);
                let event_subscribers = Arc::clone(&self.event_subscribers);
                let is_active = Arc::clone(&self.is_active);
                let next_id = Arc::clone(&self.next_id);

                info!("Starting message loop for CDP connection");

                tokio::spawn(async move {
                    info!("Message loop task started");
                    if let Err(e) = Self::message_loop_with_arcs(
                        ws_stream,
                        pending_commands,
                        event_subscribers,
                        is_active,
                        next_id,
                    ).await {
                        error!("Message loop error: {}", e);
                    }
                    info!("Message loop task exited");
                });

                Ok(())
            }
            Err(e) => {
                let mut state = self.state.write().await;
                *state = ConnectionState::Disconnected;
                drop(state);

                Err(Error::websocket(format!("Failed to connect: {}", e)))
            }
        }
    }

    /// Message processing loop
    #[allow(dead_code)]
    async fn message_loop(&self) -> Result<(), Error> {
        while self.is_active.load(Ordering::SeqCst) {
            // Get WebSocket stream
            let mut stream_guard = self.ws_stream.lock().await;
            let ws_stream = match stream_guard.as_mut() {
                Some(stream) => stream,
                None => {
                    drop(stream_guard);
                    warn!("WebSocket stream not available");
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    continue;
                }
            };

            // Receive next message with timeout
            let message = match tokio::time::timeout(
                tokio::time::Duration::from_secs(30),
                ws_stream.next(),
            )
            .await
            {
                Ok(Some(Ok(msg))) => msg,
                Ok(Some(Err(e))) => {
                    drop(stream_guard);
                    error!("WebSocket error: {}", e);
                    return Err(Error::websocket(format!("WebSocket error: {}", e)));
                }
                Ok(None) => {
                    drop(stream_guard);
                    warn!("WebSocket stream closed");
                    break;
                }
                Err(_) => {
                    // Timeout, continue loop
                    continue;
                }
            };

            drop(stream_guard);

            // Process message
            match message {
                Message::Text(text) => {
                    if let Err(e) = self.handle_message(&text).await {
                        error!("Error handling message: {}", e);
                    }
                }
                Message::Close(_) => {
                    info!("WebSocket close frame received");
                    break;
                }
                Message::Ping(data) => {
                    // Send pong
                    let mut stream_guard = self.ws_stream.lock().await;
                    if let Some(stream) = stream_guard.as_mut() {
                        if let Err(e) = stream.send(Message::Pong(data)).await {
                            error!("Failed to send pong: {}", e);
                        }
                    }
                }
                _ => {
                    // Ignore other message types
                }
            }
        }

        Ok(())
    }

    /// Message processing loop with Arc parameters (for spawned tasks)
    ///
    /// CRITICAL: Uses try_lock to periodically release the lock and allow send_command to send.
    async fn message_loop_with_arcs(
        ws_stream: Arc<Mutex<Option<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>>>,
        pending_commands: Arc<Mutex<std::collections::HashMap<u64, PendingCommand>>>,
        event_subscribers: Arc<Mutex<Vec<tokio::sync::mpsc::UnboundedSender<CdpEvent>>>>,
        is_active: Arc<AtomicBool>,
        _next_id: Arc<AtomicU64>,
    ) -> Result<(), Error> {
        info!("CDP message loop: Starting message processing loop");

        while is_active.load(Ordering::SeqCst) {
            // Use try_lock to avoid blocking send_command
            // If lock is available, try to receive a message with short timeout
            let mut stream_guard = match ws_stream.try_lock() {
                Ok(guard) => guard,
                Err(_) => {
                    // Lock is held by send_command, yield and retry
                    tokio::task::yield_now().await;
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    continue;
                }
            };

            let ws_stream_ref = match stream_guard.as_mut() {
                Some(stream) => stream,
                None => {
                    warn!("WebSocket stream not available");
                    drop(stream_guard);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    continue;
                }
            };

            // Try to receive with short timeout (100ms) then release lock
            let message_result = tokio::time::timeout(
                tokio::time::Duration::from_millis(100),
                ws_stream_ref.next(),
            ).await;

            // Lock is released here when stream_guard is dropped
            drop(stream_guard);

            match message_result {
                Ok(Some(Ok(msg))) => {
                    info!("Received message from WebSocket: {:?}", msg);
                    // Process message (lock is released during processing)
                    match msg {
                        Message::Text(text) => {
                            if let Err(e) = Self::handle_message_with_arcs(
                                &text,
                                &pending_commands,
                                &event_subscribers,
                            ).await {
                                error!("Error handling message: {}", e);
                            }
                        }
                        Message::Close(_) => {
                            info!("WebSocket close frame received");
                            break;
                        }
                        Message::Ping(data) => {
                            // Send pong (briefly reacquires lock)
                            let mut stream_guard = ws_stream.lock().await;
                            if let Some(stream) = stream_guard.as_mut() {
                                if let Err(e) = stream.send(Message::Pong(data)).await {
                                    error!("Failed to send pong: {}", e);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Some(Err(e))) => {
                    let error_msg = e.to_string();
                    error!("WebSocket error: {}", error_msg);

                    // Check if connection is already closed - gracefully deactivate instead of erroring
                    if error_msg.contains("ConnectionClosed")
                        || error_msg.contains("AlreadyClosed")
                        || error_msg.contains("connection closed")
                    {
                        warn!("WebSocket connection closed, deactivating connection");
                        is_active.store(false, Ordering::SeqCst);
                        break;
                    }

                    // For other errors, return error
                    return Err(Error::websocket(format!("WebSocket error: {}", e)));
                }
                Ok(None) => {
                    warn!("WebSocket stream closed");
                    break;
                }
                Err(_) => {
                    // Short timeout (100ms), loop again and release lock
                    debug!("Short timeout, continuing message loop");
                }
            }
        }

        Ok(())
    }

    /// Handle incoming WebSocket message with Arc parameters
    async fn handle_message_with_arcs(
        text: &str,
        pending_commands: &Arc<Mutex<std::collections::HashMap<u64, PendingCommand>>>,
        event_subscribers: &Arc<Mutex<Vec<tokio::sync::mpsc::UnboundedSender<CdpEvent>>>>,
    ) -> Result<(), Error> {
        info!("Processing received message: {}", text);

        // Try to parse as response first
        if let Ok(response) = serde_json::from_str::<CdpRpcResponse>(text) {
            return Self::handle_response_with_arcs(response, pending_commands).await;
        }

        // Try to parse as notification/event
        if let Ok(notification) = serde_json::from_str::<CdpNotification>(text) {
            return Self::handle_notification_with_arcs(notification, event_subscribers).await;
        }

        warn!("Unknown message format: {}", text);
        Ok(())
    }

    /// Handle incoming WebSocket message
    #[allow(dead_code)]
    async fn handle_message(&self, text: &str) -> Result<(), Error> {
        debug!("Received message: {}", text);

        // Try to parse as response first
        if let Ok(response) = serde_json::from_str::<CdpRpcResponse>(text) {
            return self.handle_response(response).await;
        }

        // Try to parse as notification/event
        if let Ok(notification) = serde_json::from_str::<CdpNotification>(text) {
            return self.handle_notification(notification).await;
        }

        warn!("Unknown message format: {}", text);
        Ok(())
    }

    /// Handle CDP response with Arc parameters
    async fn handle_response_with_arcs(
        response: CdpRpcResponse,
        pending_commands: &Arc<Mutex<std::collections::HashMap<u64, PendingCommand>>>,
    ) -> Result<(), Error> {
        info!("Handling response for command ID: {}", response.id);
        let mut pending = pending_commands.lock().await;

        if let Some(pending_cmd) = pending.remove(&response.id) {
            info!("Found pending command for ID {}: {}", response.id, pending_cmd.method);

            let cdp_response = CdpResponse {
                id: response.id,
                result: Some(response.result),
                error: response.error.map(|e| CdpErrorResponse {
                    code: e.code,
                    message: e.message,
                    data: e.data,
                }),
            };

            // Send response to waiter
            let _ = pending_cmd.sender.send(cdp_response);
        } else {
            warn!("Received response for unknown command ID: {}", response.id);
        }

        Ok(())
    }

    /// Handle CDP notification/event with Arc parameters
    async fn handle_notification_with_arcs(
        notification: CdpNotification,
        event_subscribers: &Arc<Mutex<Vec<tokio::sync::mpsc::UnboundedSender<CdpEvent>>>>,
    ) -> Result<(), Error> {
        debug!("Received event: {}", notification.method);

        let event = CdpEvent {
            method: notification.method,
            params: notification.params,
            session_id: notification.session_id,
        };

        // Broadcast to all subscribers
        let mut subscribers = event_subscribers.lock().await;
        let mut dead_subscribers = Vec::new();

        for (i, sender) in subscribers.iter().enumerate() {
            if sender.send(event.clone()).is_err() {
                dead_subscribers.push(i);
            }
        }

        // Remove dead subscribers
        for i in dead_subscribers.into_iter().rev() {
            subscribers.remove(i);
        }

        Ok(())
    }

    /// Handle CDP response
    #[allow(dead_code)]
    async fn handle_response(&self, response: CdpRpcResponse) -> Result<(), Error> {
        let mut pending = self.pending_commands.lock().await;

        if let Some(pending_cmd) = pending.remove(&response.id) {
            debug!("Received response for command {}: {}", response.id, pending_cmd.method);

            let cdp_response = CdpResponse {
                id: response.id,
                result: Some(response.result),
                error: response.error.map(|e| CdpErrorResponse {
                    code: e.code,
                    message: e.message,
                    data: e.data,
                }),
            };

            // Send response to waiter
            let _ = pending_cmd.sender.send(cdp_response);
        } else {
            warn!("Received response for unknown command ID: {}", response.id);
        }

        Ok(())
    }

    /// Handle CDP notification/event
    #[allow(dead_code)]
    async fn handle_notification(&self, notification: CdpNotification) -> Result<(), Error> {
        debug!("Received event: {}", notification.method);

        let event = CdpEvent {
            method: notification.method,
            params: notification.params,
            session_id: notification.session_id,
        };

        // Broadcast to all subscribers
        let mut subscribers = self.event_subscribers.lock().await;
        let mut dead_subscribers = Vec::new();

        for (i, sender) in subscribers.iter().enumerate() {
            if sender.send(event.clone()).is_err() {
                dead_subscribers.push(i);
            }
        }

        // Remove dead subscribers
        for i in dead_subscribers.into_iter().rev() {
            subscribers.remove(i);
        }

        Ok(())
    }

    /// Send WebSocket message
    async fn send_message(&self, message: Message) -> Result<(), Error> {
        let mut stream_guard = self.ws_stream.lock().await;
        let ws_stream = stream_guard
            .as_mut()
            .ok_or_else(|| Error::websocket("WebSocket stream not available"))?;

        info!("WebSocket: Sending message: {:?}", message);

        ws_stream
            .send(message)
            .await
            .map_err(|e| Error::websocket(format!("Failed to send message: {}", e)))?;

        info!("WebSocket: Message sent successfully");

        Ok(())
    }
}

#[async_trait]
impl CdpConnection for CdpWebSocketConnection {
    /// Send a CDP command and wait for response
    async fn send_command(&self, method: &str, params: serde_json::Value) -> Result<CdpResponse, Error> {
        // Check if connection is active
        if !self.is_active.load(Ordering::SeqCst) {
            return Err(Error::websocket("Connection is not active"));
        }

        // Generate command ID
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        // Create request
        let request = CdpRequest {
            id,
            method: method.to_string(),
            params: if params.is_null() {
                None
            } else {
                Some(params)
            },
            session_id: None,
        };

        // Serialize request
        let json = serde_json::to_string(&request)
            .map_err(|e| Error::cdp(format!("Failed to serialize request: {}", e)))?;

        info!("Sending CDP command {}: {} with params: {}", id, method, json);

        // Create response channel
        let (sender, receiver) = tokio::sync::oneshot::channel();

        // Register pending command
        {
            let mut pending = self.pending_commands.lock().await;
            pending.insert(
                id,
                PendingCommand {
                    sender,
                    method: method.to_string(),
                },
            );
        }

        // Send request
        self.send_message(Message::Text(json)).await?;

        // Get intelligent timeout based on command type
        let timeout_duration = self.timeout_config.get_timeout_for_command(method);
        info!("Using timeout of {:?} for command {}", timeout_duration, method);

        // Wait for response with timeout
        match tokio::time::timeout(timeout_duration, receiver).await {
            Ok(Ok(response)) => {
                // Check for CDP error
                if let Some(error) = &response.error {
                    return Err(Error::cdp(format!(
                        "{}: {} (code: {})",
                        error.message, error.code,
                        error.data.as_ref().map_or("".to_string(), |d| d.to_string())
                    )));
                }
                Ok(response)
            }
            Ok(Err(_)) => Err(Error::timeout(format!("Command {} response channel closed", id))),
            Err(_) => {
                // Clean up pending command
                let mut pending = self.pending_commands.lock().await;
                pending.remove(&id);
                Err(Error::timeout(format!("Command {} timed out", id)))
            }
        }
    }

    /// Subscribe to CDP events
    async fn listen_events(&self) -> Result<tokio::sync::mpsc::Receiver<CdpEvent>, Error> {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);

        // Use unbounded channel for event broadcasting
        let (unbounded_sender, mut unbounded_receiver) = tokio::sync::mpsc::unbounded_channel();

        let mut subscribers = self.event_subscribers.lock().await;
        subscribers.push(unbounded_sender);
        drop(subscribers);

        // Forward events to bounded channel
        tokio::spawn(async move {
            while let Some(event) = unbounded_receiver.recv().await {
                if sender.send(event).await.is_err() {
                    break;
                }
            }
        });

        Ok(receiver)
    }

    /// Close the connection
    async fn close(&self) -> Result<(), Error> {
        info!("Closing CDP WebSocket connection");

        self.is_active.store(false, Ordering::SeqCst);

        let mut stream_guard = self.ws_stream.lock().await;
        if let Some(ws_stream) = stream_guard.as_mut() {
            ws_stream
                .close(None)
                .await
                .map_err(|e| Error::websocket(format!("Failed to close WebSocket: {}", e)))?;
        }

        let mut state = self.state.write().await;
        *state = ConnectionState::Closed;

        Ok(())
    }

    /// Check if connection is active
    fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state_transitions() {
        let state = Arc::new(RwLock::new(ConnectionState::Disconnected));

        // Test state transitions
        {
            let mut s = state.blocking_write();
            *s = ConnectionState::Connecting;
        }

        {
            let s = state.blocking_read();
            assert_eq!(*s, ConnectionState::Connecting);
        }
    }
}
