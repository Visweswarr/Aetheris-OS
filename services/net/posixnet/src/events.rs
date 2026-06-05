//! Event system for POSIX networking
//! 
//! This module provides an event-driven architecture for handling socket events,
//! I/O readiness, and asynchronous operations in the POSIX networking subsystem.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, broadcast};
use tokio::time::{timeout, sleep};
use uuid::Uuid;

/// Event loop for handling socket events
pub struct EventLoop {
    /// Event channels
    event_channels: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<Event>>>>,
    /// Event handlers
    event_handlers: Arc<Mutex<Vec<Box<dyn EventHandler + Send + Sync>>>>,
    /// Event queue
    event_queue: Arc<Mutex<Vec<Event>>>,
    /// Running flag
    running: Arc<Mutex<bool>>,
    /// Event statistics
    stats: Arc<Mutex<EventStats>>,
}

/// Event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    /// Socket read ready
    SocketReadReady,
    /// Socket write ready
    SocketWriteReady,
    /// Socket error
    SocketError,
    /// Socket closed
    SocketClosed,
    /// Socket connected
    SocketConnected,
    /// Socket listening
    SocketListening,
    /// TLS handshake complete
    TLSHandshakeComplete,
    /// TLS handshake failed
    TLSHandshakeFailed,
    /// DNS resolution complete
    DNSResolutionComplete,
    /// DNS resolution failed
    DNSResolutionFailed,
    /// Policy decision
    PolicyDecision,
    /// Namespace change
    NamespaceChange,
    /// Custom event
    Custom(String),
}

/// Event token for identifying events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EventToken {
    /// Token ID
    pub id: String,
    /// Token type
    pub token_type: EventTokenType,
    /// Associated socket ID
    pub socket_id: Option<String>,
    /// Process capability
    pub process_cap: Option<String>,
    /// Creation time
    pub created_at: Instant,
}

/// Event token type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventTokenType {
    /// Socket token
    Socket,
    /// Process token
    Process,
    /// Namespace token
    Namespace,
    /// Policy token
    Policy,
    /// Custom token
    Custom(String),
}

/// Event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event ID
    pub id: String,
    /// Event type
    pub event_type: EventType,
    /// Event token
    pub token: EventToken,
    /// Event data
    pub data: EventData,
    /// Event timestamp
    pub timestamp: Instant,
    /// Event priority
    pub priority: EventPriority,
    /// Event source
    pub source: String,
}

/// Event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventData {
    /// Socket event data
    Socket {
        socket_id: String,
        state: String,
        error: Option<String>,
    },
    /// TLS event data
    TLS {
        session_id: String,
        handshake_time: Duration,
        cipher_suite: String,
        tls_version: String,
    },
    /// DNS event data
    DNS {
        domain: String,
        addresses: Vec<String>,
        error: Option<String>,
    },
    /// Policy event data
    Policy {
        decision: String,
        reason: String,
        context: HashMap<String, String>,
    },
    /// Namespace event data
    Namespace {
        namespace_id: String,
        old_namespace: Option<String>,
        new_namespace: String,
    },
    /// Custom event data
    Custom {
        data: HashMap<String, String>,
    },
}

/// Event priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    /// Low priority
    Low = 0,
    /// Normal priority
    Normal = 1,
    /// High priority
    High = 2,
    /// Critical priority
    Critical = 3,
}

/// Event handler trait
pub trait EventHandler: Send + Sync {
    /// Handle an event
    fn handle_event(&self, event: &Event);
    
    /// Get handler name
    fn get_name(&self) -> &str;
    
    /// Check if handler can handle event type
    fn can_handle(&self, event_type: &EventType) -> bool;
}

/// Event statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventStats {
    /// Total events processed
    pub total_events: u64,
    /// Events by type
    pub events_by_type: HashMap<EventType, u64>,
    /// Events by priority
    pub events_by_priority: HashMap<EventPriority, u64>,
    /// Average processing time
    pub avg_processing_time: Duration,
    /// Total processing time
    pub total_processing_time: Duration,
    /// Event queue size
    pub queue_size: usize,
    /// Active handlers
    pub active_handlers: usize,
}

impl EventLoop {
    /// Create a new event loop
    pub fn new() -> Self {
        Self {
            event_channels: Arc::new(Mutex::new(HashMap::new())),
            event_handlers: Arc::new(Mutex::new(Vec::new())),
            event_queue: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
            stats: Arc::new(Mutex::new(EventStats::default())),
        }
    }

    /// Start the event loop
    pub async fn start(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            return;
        }
        *running = true;
        drop(running);

        // Start event processing loop
        let event_queue = self.event_queue.clone();
        let event_handlers = self.event_handlers.clone();
        let running = self.running.clone();
        let stats = self.stats.clone();

        tokio::spawn(async move {
            Self::event_processing_loop(event_queue, event_handlers, running, stats).await;
        });
    }

    /// Stop the event loop
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }

    /// Register an event handler
    pub fn register_handler(&self, handler: Box<dyn EventHandler + Send + Sync>) {
        self.event_handlers.lock().unwrap().push(handler);
    }

    /// Unregister an event handler
    pub fn unregister_handler(&self, handler_name: &str) {
        let mut handlers = self.event_handlers.lock().unwrap();
        handlers.retain(|h| h.get_name() != handler_name);
    }

    /// Create an event token
    pub fn create_token(
        &self,
        token_type: EventTokenType,
        socket_id: Option<String>,
        process_cap: Option<String>,
    ) -> EventToken {
        EventToken {
            id: Uuid::new_v4().to_string(),
            token_type,
            socket_id,
            process_cap,
            created_at: Instant::now(),
        }
    }

    /// Emit an event
    pub fn emit_event(&self, event: Event) {
        // Add to event queue
        {
            let mut queue = self.event_queue.lock().unwrap();
            queue.push(event.clone());
        }

        // Update statistics
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_events += 1;
            *stats.events_by_type.entry(event.event_type.clone()).or_insert(0) += 1;
            *stats.events_by_priority.entry(event.priority.clone()).or_insert(0) += 1;
            stats.queue_size = self.event_queue.lock().unwrap().len();
        }

        // Send to event channels
        {
            let channels = self.event_channels.lock().unwrap();
            for (_, tx) in channels.iter() {
                let _ = tx.send(event.clone());
            }
        }
    }

    /// Create an event channel
    pub fn create_event_channel(&self, name: String) -> mpsc::UnboundedReceiver<Event> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.event_channels.lock().unwrap().insert(name, tx);
        rx
    }

    /// Remove an event channel
    pub fn remove_event_channel(&self, name: &str) {
        self.event_channels.lock().unwrap().remove(name);
    }

    /// Wait for events with timeout
    pub async fn wait_for_events(
        &self,
        event_types: Vec<EventType>,
        timeout_duration: Option<Duration>,
    ) -> Result<Vec<Event>, String> {
        let mut events = Vec::new();
        let start_time = Instant::now();

        loop {
            // Check timeout
            if let Some(timeout) = timeout_duration {
                if start_time.elapsed() > timeout {
                    break;
                }
            }

            // Get events from queue
            let mut queue = self.event_queue.lock().unwrap();
            let mut found_events = Vec::new();

            for (i, event) in queue.iter().enumerate() {
                if event_types.contains(&event.event_type) {
                    found_events.push(i);
                }
            }

            // Remove found events from queue
            for &i in found_events.iter().rev() {
                if let Some(event) = queue.remove(i) {
                    events.push(event);
                }
            }

            if !events.is_empty() {
                break;
            }

            // Wait a bit before checking again
            drop(queue);
            sleep(Duration::from_millis(10)).await;
        }

        Ok(events)
    }

    /// Get event statistics
    pub fn get_stats(&self) -> EventStats {
        let mut stats = self.stats.lock().unwrap().clone();
        stats.queue_size = self.event_queue.lock().unwrap().len();
        stats.active_handlers = self.event_handlers.lock().unwrap().len();
        stats
    }

    /// Clear event queue
    pub fn clear_queue(&self) {
        self.event_queue.lock().unwrap().clear();
    }

    /// Event processing loop
    async fn event_processing_loop(
        event_queue: Arc<Mutex<Vec<Event>>>,
        event_handlers: Arc<Mutex<Vec<Box<dyn EventHandler + Send + Sync>>>>,
        running: Arc<Mutex<bool>>,
        stats: Arc<Mutex<EventStats>>,
    ) {
        loop {
            // Check if we should stop
            {
                let running_guard = running.lock().unwrap();
                if !*running_guard {
                    break;
                }
            }

            // Process events from queue
            let mut events_to_process = Vec::new();
            {
                let mut queue = event_queue.lock().unwrap();
                if !queue.is_empty() {
                    events_to_process = queue.drain(..).collect();
                }
            }

            // Process each event
            for event in events_to_process {
                let start_time = Instant::now();
                
                // Get handlers that can handle this event
                let handlers = {
                    let handlers = event_handlers.lock().unwrap();
                    handlers.iter()
                        .filter(|h| h.can_handle(&event.event_type))
                        .collect::<Vec<_>>()
                };

                // Call handlers
                for handler in handlers {
                    handler.handle_event(&event);
                }

                // Update statistics
                let processing_time = start_time.elapsed();
                {
                    let mut stats = stats.lock().unwrap();
                    stats.total_processing_time += processing_time;
                    stats.avg_processing_time = if stats.total_events > 0 {
                        Duration::from_nanos(
                            stats.total_processing_time.as_nanos() as u64 / stats.total_events
                        )
                    } else {
                        Duration::ZERO
                    };
                }
            }

            // Sleep for a short time to avoid busy waiting
            sleep(Duration::from_millis(1)).await;
        }
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}

/// Default event handler for logging
pub struct LoggingEventHandler {
    name: String,
}

impl LoggingEventHandler {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl EventHandler for LoggingEventHandler {
    fn handle_event(&self, event: &Event) {
        println!("[{}] Event: {:?} - {:?}", self.name, event.event_type, event.data);
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn can_handle(&self, _event_type: &EventType) -> bool {
        true // Handle all events
    }
}

/// Socket event handler
pub struct SocketEventHandler {
    name: String,
    socket_id: String,
}

impl SocketEventHandler {
    pub fn new(name: String, socket_id: String) -> Self {
        Self { name, socket_id }
    }
}

impl EventHandler for SocketEventHandler {
    fn handle_event(&self, event: &Event) {
        if let Some(event_socket_id) = &event.token.socket_id {
            if event_socket_id == &self.socket_id {
                println!("[{}] Socket {} event: {:?}", self.name, self.socket_id, event.event_type);
            }
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn can_handle(&self, event_type: &EventType) -> bool {
        matches!(event_type, 
            EventType::SocketReadReady | 
            EventType::SocketWriteReady | 
            EventType::SocketError | 
            EventType::SocketClosed |
            EventType::SocketConnected |
            EventType::SocketListening
        )
    }
}

/// Policy event handler
pub struct PolicyEventHandler {
    name: String,
}

impl PolicyEventHandler {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl EventHandler for PolicyEventHandler {
    fn handle_event(&self, event: &Event) {
        if matches!(event.event_type, EventType::PolicyDecision) {
            println!("[{}] Policy decision: {:?}", self.name, event.data);
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn can_handle(&self, event_type: &EventType) -> bool {
        matches!(event_type, EventType::PolicyDecision)
    }
}
