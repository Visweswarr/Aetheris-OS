//! Intent Bus Adapter for Polyglot Runtime
//!
//! This module provides integration between the Polyglot Runtime and the
//! Intent Kernel for cross-runtime IPC. It enables sandboxes to communicate
//! via the Intent Bus using AI Tool Call message types.
//!
//! # Message Types
//!
//! - `AiInvoke`: Invoke an AI tool or function
//! - `AiQuery`: Query AI state or capabilities
//! - `AiStream`: Stream data to/from AI services

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;

use crate::klog;
use super::bridge::{Message, Destination, PolyglotBridge};
use super::sandbox::SandboxId;
use super::backend::RuntimeError;

/// AI Tool Call message types for Intent Bus integration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum AiMessageType {
    /// Invoke an AI tool or function
    AiInvoke = 1,
    /// Query AI state or capabilities
    AiQuery = 2,
    /// Stream data to/from AI services
    AiStream = 3,
    /// Response to an AI invocation
    AiResponse = 4,
    /// Error response
    AiError = 5,
}

impl AiMessageType {
    /// Convert from u16
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(AiMessageType::AiInvoke),
            2 => Some(AiMessageType::AiQuery),
            3 => Some(AiMessageType::AiStream),
            4 => Some(AiMessageType::AiResponse),
            5 => Some(AiMessageType::AiError),
            _ => None,
        }
    }

    /// Convert to string for message_type field
    pub fn as_str(&self) -> &'static str {
        match self {
            AiMessageType::AiInvoke => "ai.invoke",
            AiMessageType::AiQuery => "ai.query",
            AiMessageType::AiStream => "ai.stream",
            AiMessageType::AiResponse => "ai.response",
            AiMessageType::AiError => "ai.error",
        }
    }
}

/// AI Invoke request payload
#[derive(Debug, Clone)]
pub struct AiInvokeRequest {
    /// Tool or function name to invoke
    pub tool_name: String,
    /// Arguments for the tool (CBOR-encoded)
    pub arguments: Vec<u8>,
    /// Request ID for correlation
    pub request_id: u64,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
}

/// AI Query request payload
#[derive(Debug, Clone)]
pub struct AiQueryRequest {
    /// Query type (e.g., "capabilities", "status")
    pub query_type: String,
    /// Query parameters
    pub parameters: BTreeMap<String, String>,
    /// Request ID for correlation
    pub request_id: u64,
}

/// AI Stream request payload
#[derive(Debug, Clone)]
pub struct AiStreamRequest {
    /// Stream ID
    pub stream_id: u64,
    /// Stream operation (open, data, close)
    pub operation: StreamOperation,
    /// Data chunk (for data operation)
    pub data: Vec<u8>,
}

/// Stream operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamOperation {
    /// Open a new stream
    Open,
    /// Send data on stream
    Data,
    /// Close the stream
    Close,
}

/// Message handler registration
#[derive(Debug, Clone)]
pub struct MessageHandler {
    /// Handler ID
    pub id: u64,
    /// Message type to handle
    pub message_type: AiMessageType,
    /// Sandbox that registered the handler
    pub sandbox_id: SandboxId,
}

/// Intent Bus Adapter for cross-runtime IPC
pub struct IntentBusAdapter {
    /// Registered message handlers
    handlers: Vec<MessageHandler>,
    /// Next handler ID
    next_handler_id: u64,
    /// Pending messages for sandboxes
    pending_messages: BTreeMap<u64, Vec<Message>>,
    /// Message routing table (topic -> sandbox IDs)
    topic_subscriptions: BTreeMap<String, Vec<SandboxId>>,
}

impl IntentBusAdapter {
    /// Create a new Intent Bus adapter
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            next_handler_id: 1,
            pending_messages: BTreeMap::new(),
            topic_subscriptions: BTreeMap::new(),
        }
    }

    /// Register a message handler for AI Tool Calls
    pub fn register_handler(
        &mut self,
        sandbox_id: SandboxId,
        message_type: AiMessageType,
    ) -> u64 {
        let handler_id = self.next_handler_id;
        self.next_handler_id += 1;

        self.handlers.push(MessageHandler {
            id: handler_id,
            message_type,
            sandbox_id,
        });

        klog!(INFO, "[INTENT_ADAPTER] Registered handler {} for {:?} from sandbox {:?}",
              handler_id, message_type, sandbox_id);

        handler_id
    }

    /// Unregister a message handler
    pub fn unregister_handler(&mut self, handler_id: u64) -> bool {
        if let Some(pos) = self.handlers.iter().position(|h| h.id == handler_id) {
            self.handlers.remove(pos);
            klog!(INFO, "[INTENT_ADAPTER] Unregistered handler {}", handler_id);
            true
        } else {
            false
        }
    }

    /// Subscribe a sandbox to a topic
    pub fn subscribe(&mut self, sandbox_id: SandboxId, topic: &str) {
        let subscribers = self.topic_subscriptions
            .entry(topic.into())
            .or_insert_with(Vec::new);
        
        if !subscribers.contains(&sandbox_id) {
            subscribers.push(sandbox_id);
            klog!(INFO, "[INTENT_ADAPTER] Sandbox {:?} subscribed to topic '{}'",
                  sandbox_id, topic);
        }
    }

    /// Unsubscribe a sandbox from a topic
    pub fn unsubscribe(&mut self, sandbox_id: SandboxId, topic: &str) {
        if let Some(subscribers) = self.topic_subscriptions.get_mut(topic) {
            subscribers.retain(|&id| id != sandbox_id);
            klog!(INFO, "[INTENT_ADAPTER] Sandbox {:?} unsubscribed from topic '{}'",
                  sandbox_id, topic);
        }
    }

    /// Route a message to the appropriate destination
    pub fn route_message(
        &mut self,
        bridge: &PolyglotBridge,
        message: Message,
    ) -> Result<(), RuntimeError> {
        match &message.destination {
            Destination::Sandbox(target_id) => {
                // Direct message to a specific sandbox
                self.queue_message(*target_id, message);
                Ok(())
            }
            Destination::Topic(topic) => {
                // Publish to all subscribers
                let subscribers_to_route = if let Some(subscribers) = self.topic_subscriptions.get(topic) {
                    subscribers.clone()
                } else {
                    Vec::new()
                };
                for sandbox_id in subscribers_to_route {
                    if sandbox_id != message.source {
                        self.queue_message(sandbox_id, message.clone());
                    }
                }
                Ok(())
            }
            Destination::Broadcast => {
                // Broadcast to all sandboxes (except sender)
                let all_sandbox_ids: Vec<SandboxId> = self.pending_messages
                    .keys()
                    .map(|&id| SandboxId(id))
                    .collect();
                
                for sandbox_id in all_sandbox_ids {
                    if sandbox_id != message.source {
                        self.queue_message(sandbox_id, message.clone());
                    }
                }
                Ok(())
            }
        }
    }

    /// Queue a message for a sandbox
    fn queue_message(&mut self, sandbox_id: SandboxId, message: Message) {
        let queue = self.pending_messages
            .entry(sandbox_id.0)
            .or_insert_with(Vec::new);
        queue.push(message);
        
        klog!(TRACE, "[INTENT_ADAPTER] Queued message for sandbox {:?}", sandbox_id);
    }

    /// Receive a message for a sandbox (non-blocking)
    pub fn receive_message(&mut self, sandbox_id: SandboxId) -> Option<Message> {
        if let Some(queue) = self.pending_messages.get_mut(&sandbox_id.0) {
            if !queue.is_empty() {
                return Some(queue.remove(0));
            }
        }
        None
    }

    /// Check if there are pending messages for a sandbox
    pub fn has_pending_messages(&self, sandbox_id: SandboxId) -> bool {
        self.pending_messages
            .get(&sandbox_id.0)
            .map(|q| !q.is_empty())
            .unwrap_or(false)
    }

    /// Get the number of pending messages for a sandbox
    pub fn pending_message_count(&self, sandbox_id: SandboxId) -> usize {
        self.pending_messages
            .get(&sandbox_id.0)
            .map(|q| q.len())
            .unwrap_or(0)
    }

    /// Create an AI Invoke message
    pub fn create_ai_invoke_message(
        &self,
        source: SandboxId,
        destination: Destination,
        request: AiInvokeRequest,
    ) -> Message {
        // Serialize the request to CBOR (simplified)
        let mut payload = Vec::new();
        payload.extend_from_slice(request.tool_name.as_bytes());
        payload.push(0); // null terminator
        payload.extend_from_slice(&request.arguments);

        Message {
            id: request.request_id,
            source,
            destination,
            message_type: AiMessageType::AiInvoke.as_str().into(),
            payload,
            timestamp: crate::security::get_current_time_ms(),
        }
    }

    /// Create an AI Query message
    pub fn create_ai_query_message(
        &self,
        source: SandboxId,
        destination: Destination,
        request: AiQueryRequest,
    ) -> Message {
        // Serialize the request (simplified)
        let payload = request.query_type.as_bytes().to_vec();

        Message {
            id: request.request_id,
            source,
            destination,
            message_type: AiMessageType::AiQuery.as_str().into(),
            payload,
            timestamp: crate::security::get_current_time_ms(),
        }
    }

    /// Create an AI Response message
    pub fn create_ai_response_message(
        &self,
        source: SandboxId,
        destination: SandboxId,
        request_id: u64,
        response_data: Vec<u8>,
    ) -> Message {
        Message {
            id: request_id,
            source,
            destination: Destination::Sandbox(destination),
            message_type: AiMessageType::AiResponse.as_str().into(),
            payload: response_data,
            timestamp: crate::security::get_current_time_ms(),
        }
    }

    /// Create an AI Error message
    pub fn create_ai_error_message(
        &self,
        source: SandboxId,
        destination: SandboxId,
        request_id: u64,
        error_message: &str,
    ) -> Message {
        Message {
            id: request_id,
            source,
            destination: Destination::Sandbox(destination),
            message_type: AiMessageType::AiError.as_str().into(),
            payload: error_message.as_bytes().to_vec(),
            timestamp: crate::security::get_current_time_ms(),
        }
    }

    /// Clean up handlers and subscriptions for a terminated sandbox
    pub fn cleanup_sandbox(&mut self, sandbox_id: SandboxId) {
        // Remove handlers
        self.handlers.retain(|h| h.sandbox_id != sandbox_id);

        // Remove from topic subscriptions
        for subscribers in self.topic_subscriptions.values_mut() {
            subscribers.retain(|&id| id != sandbox_id);
        }

        // Remove pending messages
        self.pending_messages.remove(&sandbox_id.0);

        klog!(INFO, "[INTENT_ADAPTER] Cleaned up sandbox {:?}", sandbox_id);
    }

    /// Get all handlers for a message type
    pub fn get_handlers(&self, message_type: AiMessageType) -> Vec<&MessageHandler> {
        self.handlers
            .iter()
            .filter(|h| h.message_type == message_type)
            .collect()
    }
}

impl Default for IntentBusAdapter {
    fn default() -> Self {
        Self::new()
    }
}
