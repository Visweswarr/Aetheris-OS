//! Readiness APIs for POSIX networking
//! 
//! This module provides select/poll/epoll emulation for the POSIX networking subsystem.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc};
use serde::{Deserialize, Serialize};

/// Readiness event types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ReadinessEvent {
    /// Socket is ready for reading
    Read,
    /// Socket is ready for writing
    Write,
    /// Socket has an error condition
    Error,
    /// Socket is ready for accepting connections
    Accept,
}

/// Readiness interest types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ReadinessInterest {
    /// Interest in read events
    Read,
    /// Interest in write events
    Write,
    /// Interest in error events
    Error,
    /// Interest in accept events
    Accept,
}

/// Readiness subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessSubscription {
    /// Socket ID
    pub socket_id: String,
    /// Interests
    pub interests: Vec<ReadinessInterest>,
    /// Edge-triggered mode
    pub edge_triggered: bool,
    /// Subscription ID
    pub subscription_id: String,
    /// Process capability
    pub process_cap: String,
    /// Created timestamp
    pub created_at: Instant,
}

/// Readiness event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessEventData {
    /// Socket ID
    pub socket_id: String,
    /// Event type
    pub event: ReadinessEvent,
    /// Event timestamp
    pub timestamp: Instant,
    /// Additional data
    pub data: Option<Vec<u8>>,
}

/// Readiness manager
pub struct ReadinessManager {
    /// Active subscriptions
    subscriptions: Arc<RwLock<HashMap<String, ReadinessSubscription>>>,
    /// Event channel
    event_tx: mpsc::UnboundedSender<ReadinessEventData>,
    event_rx: Arc<RwLock<mpsc::UnboundedReceiver<ReadinessEventData>>>,
    /// Statistics
    stats: Arc<RwLock<ReadinessStats>>,
}

/// Readiness statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReadinessStats {
    /// Total subscriptions
    pub total_subscriptions: u64,
    /// Active subscriptions
    pub active_subscriptions: u64,
    /// Events generated
    pub events_generated: u64,
    /// Events delivered
    pub events_delivered: u64,
    /// Poll operations
    pub poll_operations: u64,
    /// Select operations
    pub select_operations: u64,
    /// Epoll operations
    pub epoll_operations: u64,
}

impl ReadinessManager {
    /// Create a new readiness manager
    pub fn new() -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx: Arc::new(RwLock::new(event_rx)),
            stats: Arc::new(RwLock::new(ReadinessStats::default())),
        }
    }

    /// Subscribe to readiness events
    pub async fn subscribe(
        &self,
        socket_id: String,
        interests: Vec<ReadinessInterest>,
        edge_triggered: bool,
        process_cap: String,
    ) -> Result<String, String> {
        let subscription_id = format!("sub_{}_{}", socket_id, uuid::Uuid::new_v4());
        
        let subscription = ReadinessSubscription {
            socket_id: socket_id.clone(),
            interests,
            edge_triggered,
            subscription_id: subscription_id.clone(),
            process_cap,
            created_at: Instant::now(),
        };
        
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription_id.clone(), subscription);
        
        let mut stats = self.stats.write().await;
        stats.total_subscriptions += 1;
        stats.active_subscriptions += 1;
        
        Ok(subscription_id)
    }

    /// Unsubscribe from readiness events
    pub async fn unsubscribe(&self, subscription_id: &str) -> Result<(), String> {
        let mut subscriptions = self.subscriptions.write().await;
        if subscriptions.remove(subscription_id).is_some() {
            let mut stats = self.stats.write().await;
            stats.active_subscriptions = stats.active_subscriptions.saturating_sub(1);
            Ok(())
        } else {
            Err("Subscription not found".to_string())
        }
    }

    /// Notify readiness event
    pub async fn notify_event(
        &self,
        socket_id: String,
        event: ReadinessEvent,
        data: Option<Vec<u8>>,
    ) -> Result<(), String> {
        let event_data = ReadinessEventData {
            socket_id,
            event,
            timestamp: Instant::now(),
            data,
        };
        
        if let Err(_) = self.event_tx.send(event_data) {
            return Err("Event channel closed".to_string());
        }
        
        let mut stats = self.stats.write().await;
        stats.events_generated += 1;
        
        Ok(())
    }

    /// Poll for readiness events (select/poll emulation)
    pub async fn poll(
        &self,
        socket_ids: Vec<String>,
        interests: Vec<ReadinessInterest>,
        timeout: Option<Duration>,
    ) -> Result<Vec<ReadinessEventData>, String> {
        let mut stats = self.stats.write().await;
        stats.poll_operations += 1;
        drop(stats);
        
        let mut events = Vec::new();
        let mut event_rx = self.event_rx.write().await;
        
        let start_time = Instant::now();
        
        loop {
            // Check timeout
            if let Some(timeout_duration) = timeout {
                if start_time.elapsed() >= timeout_duration {
                    break;
                }
            }
            
            // Try to receive event with timeout
            let event_result = if let Some(timeout_duration) = timeout {
                let remaining = timeout_duration - start_time.elapsed();
                if remaining > Duration::from_millis(0) {
                    tokio::time::timeout(remaining, event_rx.recv()).await
                } else {
                    break;
                }
            } else {
                Ok(Some(event_rx.recv().await))
            };
            
            match event_result {
                Ok(Some(event)) => {
                    // Check if this event matches our interests
                    if socket_ids.contains(&event.socket_id) && 
                       interests.contains(&event.event.into()) {
                        events.push(event);
                        
                        let mut stats = self.stats.write().await;
                        stats.events_delivered += 1;
                    }
                }
                Ok(None) => break, // Channel closed
                Err(_) => break, // Timeout
            }
        }
        
        Ok(events)
    }

    /// Select operation (POSIX select emulation)
    pub async fn select(
        &self,
        read_sockets: Vec<String>,
        write_sockets: Vec<String>,
        error_sockets: Vec<String>,
        timeout: Option<Duration>,
    ) -> Result<SelectResult, String> {
        let mut stats = self.stats.write().await;
        stats.select_operations += 1;
        drop(stats);
        
        let mut read_ready = Vec::new();
        let mut write_ready = Vec::new();
        let mut error_ready = Vec::new();
        
        let mut event_rx = self.event_rx.write().await;
        let start_time = Instant::now();
        
        loop {
            // Check timeout
            if let Some(timeout_duration) = timeout {
                if start_time.elapsed() >= timeout_duration {
                    break;
                }
            }
            
            // Try to receive event with timeout
            let event_result = if let Some(timeout_duration) = timeout {
                let remaining = timeout_duration - start_time.elapsed();
                if remaining > Duration::from_millis(0) {
                    tokio::time::timeout(remaining, event_rx.recv()).await
                } else {
                    break;
                }
            } else {
                Ok(Some(event_rx.recv().await))
            };
            
            match event_result {
                Ok(Some(event)) => {
                    match event.event {
                        ReadinessEvent::Read | ReadinessEvent::Accept => {
                            if read_sockets.contains(&event.socket_id) {
                                read_ready.push(event.socket_id);
                            }
                        }
                        ReadinessEvent::Write => {
                            if write_sockets.contains(&event.socket_id) {
                                write_ready.push(event.socket_id);
                            }
                        }
                        ReadinessEvent::Error => {
                            if error_sockets.contains(&event.socket_id) {
                                error_ready.push(event.socket_id);
                            }
                        }
                    }
                    
                    let mut stats = self.stats.write().await;
                    stats.events_delivered += 1;
                }
                Ok(None) => break, // Channel closed
                Err(_) => break, // Timeout
            }
        }
        
        Ok(SelectResult {
            read_ready,
            write_ready,
            error_ready,
        })
    }

    /// Epoll operation (Linux epoll emulation)
    pub async fn epoll_wait(
        &self,
        epoll_fd: i32,
        max_events: usize,
        timeout: Option<Duration>,
    ) -> Result<Vec<EpollEvent>, String> {
        let mut stats = self.stats.write().await;
        stats.epoll_operations += 1;
        drop(stats);
        
        let mut events = Vec::new();
        let mut event_rx = self.event_rx.write().await;
        
        let start_time = Instant::now();
        
        loop {
            // Check if we have enough events
            if events.len() >= max_events {
                break;
            }
            
            // Check timeout
            if let Some(timeout_duration) = timeout {
                if start_time.elapsed() >= timeout_duration {
                    break;
                }
            }
            
            // Try to receive event with timeout
            let event_result = if let Some(timeout_duration) = timeout {
                let remaining = timeout_duration - start_time.elapsed();
                if remaining > Duration::from_millis(0) {
                    tokio::time::timeout(remaining, event_rx.recv()).await
                } else {
                    break;
                }
            } else {
                Ok(Some(event_rx.recv().await))
            };
            
            match event_result {
                Ok(Some(event)) => {
                    let epoll_event = EpollEvent {
                        socket_id: event.socket_id,
                        events: event.event.into(),
                        data: event.data,
                    };
                    events.push(epoll_event);
                    
                    let mut stats = self.stats.write().await;
                    stats.events_delivered += 1;
                }
                Ok(None) => break, // Channel closed
                Err(_) => break, // Timeout
            }
        }
        
        Ok(events)
    }

    /// Get readiness statistics
    pub async fn get_stats(&self) -> ReadinessStats {
        self.stats.read().await.clone()
    }

    /// Cleanup expired subscriptions
    pub async fn cleanup_expired_subscriptions(&self, max_age: Duration) -> usize {
        let now = Instant::now();
        let mut subscriptions = self.subscriptions.write().await;
        let mut expired = Vec::new();
        
        for (subscription_id, subscription) in subscriptions.iter() {
            if now.duration_since(subscription.created_at) > max_age {
                expired.push(subscription_id.clone());
            }
        }
        
        for subscription_id in &expired {
            subscriptions.remove(subscription_id);
        }
        
        let mut stats = self.stats.write().await;
        stats.active_subscriptions = stats.active_subscriptions.saturating_sub(expired.len() as u64);
        
        expired.len()
    }
}

impl Default for ReadinessManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Select result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectResult {
    /// Sockets ready for reading
    pub read_ready: Vec<String>,
    /// Sockets ready for writing
    pub write_ready: Vec<String>,
    /// Sockets with errors
    pub error_ready: Vec<String>,
}

/// Epoll event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpollEvent {
    /// Socket ID
    pub socket_id: String,
    /// Event flags
    pub events: u32,
    /// User data
    pub data: Option<Vec<u8>>,
}

/// Convert ReadinessEvent to u32 flags
impl From<ReadinessEvent> for u32 {
    fn from(event: ReadinessEvent) -> u32 {
        match event {
            ReadinessEvent::Read => 0x001, // EPOLLIN
            ReadinessEvent::Write => 0x004, // EPOLLOUT
            ReadinessEvent::Error => 0x008, // EPOLLERR
            ReadinessEvent::Accept => 0x001, // EPOLLIN for accept
        }
    }
}

/// Convert ReadinessEvent to ReadinessInterest
impl From<ReadinessEvent> for ReadinessInterest {
    fn from(event: ReadinessEvent) -> ReadinessInterest {
        match event {
            ReadinessEvent::Read | ReadinessEvent::Accept => ReadinessInterest::Read,
            ReadinessEvent::Write => ReadinessInterest::Write,
            ReadinessEvent::Error => ReadinessInterest::Error,
        }
    }
}

/// Convert ReadinessInterest to ReadinessEvent
impl From<ReadinessInterest> for ReadinessEvent {
    fn from(interest: ReadinessInterest) -> ReadinessEvent {
        match interest {
            ReadinessInterest::Read => ReadinessEvent::Read,
            ReadinessInterest::Write => ReadinessEvent::Write,
            ReadinessInterest::Error => ReadinessEvent::Error,
            ReadinessInterest::Accept => ReadinessEvent::Accept,
        }
    }
}
