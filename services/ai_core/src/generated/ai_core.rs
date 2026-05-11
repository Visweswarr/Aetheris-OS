//! Auto-generated stub for protobuf types
//! This file is generated when protoc is not available

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// AI Core Message wrapper
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AiCoreMessage {
    pub message_type: Option<ai_core_message::MessageType>,
    pub message_id: String,
    pub timestamp: u64,
    pub session_id: String,
    pub cap_token: Option<CapToken>,
}

/// Custom encode error for CBOR serialization
#[derive(Debug)]
pub struct CborEncodeError(String);

impl std::fmt::Display for CborEncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CBOR encode error: {}", self.0)
    }
}

impl std::error::Error for CborEncodeError {}

/// Custom decode error for CBOR deserialization
#[derive(Debug)]
pub struct CborDecodeError(String);

impl std::fmt::Display for CborDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CBOR decode error: {}", self.0)
    }
}

impl std::error::Error for CborDecodeError {}

impl AiCoreMessage {
    pub fn encode(&self, buf: &mut Vec<u8>) -> Result<(), CborEncodeError> {
        let data = serde_cbor::to_vec(self).map_err(|e| CborEncodeError(e.to_string()))?;
        buf.extend_from_slice(&data);
        Ok(())
    }

    pub fn decode(buf: &[u8]) -> Result<Self, CborDecodeError> {
        serde_cbor::from_slice(buf).map_err(|e| CborDecodeError(e.to_string()))
    }
}

pub mod ai_core_message {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub enum MessageType {
        PingRequest(super::PingRequest),
        PingResponse(super::PingResponse),
        ChatRequest(super::ChatRequest),
        ChatResponse(super::ChatResponse),
        ToolCallRequest(super::ToolCallRequest),
        ToolCallResponse(super::ToolCallResponse),
        ErrorResponse(super::ErrorResponse),
        NotificationActionRequest(super::NotificationActionRequest),
        NotificationActionResponse(super::NotificationActionResponse),
        PlanRequest(super::PlanRequest),
        PlanResponse(super::PlanResponse),
        PlanStatusRequest(super::PlanStatusRequest),
        PlanStatusResponse(super::PlanStatusResponse),
        PlanListRequest(super::PlanListRequest),
        PlanListResponse(super::PlanListResponse),
    }
}

/// Capability token
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CapToken {
    pub token_id: String,
    pub capability: String,
    pub expires_at: u64,
    pub issuer: String,
    pub signature: Vec<u8>,
}

/// Ping request
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PingRequest {
    pub client_id: String,
    pub version: String,
}

/// Ping response
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PingResponse {
    pub server_id: String,
    pub version: String,
    pub server_time: u64,
    pub status: Option<ServiceStatus>,
}

/// Service status
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub state: i32,
    pub active_sessions: i32,
    pub uptime_seconds: u64,
    pub system_metrics: Option<SystemMetrics>,
}

/// System metrics
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f64,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub active_connections: i32,
    pub requests_processed: u64,
}

/// Chat request
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChatRequest {
    pub prompt: String,
    pub message: String,
    pub stream: bool,
    pub conversation_id: Option<String>,
    pub config: Option<ChatConfig>,
    pub context: Vec<MessageContext>,
    pub conversation_history: Option<Vec<ConversationTurn>>,
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub language: Option<String>,
    pub enable_function_calling: bool,
    pub allowed_functions: Vec<String>,
}

/// Chat configuration
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChatConfig {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: i32,
    pub top_p: f32,
    pub top_k: i32,
    pub enable_tools: bool,
    pub allowed_tools: Vec<String>,
}

/// Message context
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MessageContext {
    pub role: String,
    pub content: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

/// Conversation turn
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

/// Chat response
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChatResponse {
    pub response: String,
    pub is_complete: bool,
    pub metrics: Option<ChatMetrics>,
    pub tool_calls: Vec<ToolCall>,
}

/// Chat metrics
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChatMetrics {
    pub processing_time_ms: i64,
    pub tokens_generated: i32,
    pub tokens_input: i32,
    pub confidence: f32,
    pub model_used: String,
}

/// Tool call
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub call_id: String,
    pub arguments: String,
}

/// Tool call request
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ToolCallRequest {
    pub tool_name: String,
    pub call_id: String,
    pub parameters: serde_json::Value,
    pub arguments: String,
    pub cap_token: Option<CapToken>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
}

/// Tool call response
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ToolCallResponse {
    pub result: String,
    pub success: bool,
    pub error_message: String,
    pub metrics: Option<ToolMetrics>,
    pub cbor_payload: Option<Vec<u8>>,
    pub call_id: String,
    pub tool_version: String,
    pub metadata: Option<HashMap<String, String>>,
    pub warnings: Vec<String>,
    pub exit_code: i32,
}

/// Tool metrics
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ToolMetrics {
    pub execution_time_ms: i64,
    pub memory_used_mb: u64,
    pub cache_hit: bool,
    pub tool_version: String,
}

/// Function result
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FunctionResult {
    pub function_name: String,
    pub call_id: String,
    pub success: bool,
    pub result: String,
    pub cbor_payload: Option<Vec<u8>>,
    pub error_message: Option<String>,
    pub exit_code: i32,
    pub metadata: Option<HashMap<String, String>>,
    pub execution_time_ms: u64,
    pub memory_used_mb: u64,
}

/// Error response
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: i32,
    pub message: String,
    pub details: String,
    pub timestamp: u64,
}

/// Error code enum
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum ErrorCode {
    #[default]
    UnknownError = 0,
    InvalidRequest = 1,
    Unauthorized = 2,
    NotFound = 3,
    RateLimited = 4,
    InternalError = 8,
    Timeout = 9,
}

/// Service state enum
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum ServiceState {
    #[default]
    Starting = 0,
    Running = 1,
    Stopping = 2,
    Stopped = 3,
    Error = 4,
}

/// Notification action request
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NotificationActionRequest {
    pub notification_id: String,
    pub action_id: String,
    pub tool_name: String,
    pub cbor_parameters: Vec<u8>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Notification action response
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NotificationActionResponse {
    pub notification_id: String,
    pub action_id: String,
    pub success: bool,
    pub cbor_result: Vec<u8>,
    pub error_message: String,
    pub timestamp: i64,
}

/// Plan request
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PlanRequest {
    pub goal: String,
    pub context: Option<String>,
    pub constraints: Vec<String>,
}

/// Plan response
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PlanResponse {
    pub plan_id: String,
    pub steps: Vec<PlanStep>,
    pub estimated_duration_ms: u64,
}

/// Plan step
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PlanStep {
    pub step_id: String,
    pub description: String,
    pub tool_name: Option<String>,
    pub parameters: Option<String>,
    pub dependencies: Vec<String>,
}

/// Plan status request
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PlanStatusRequest {
    pub plan_id: String,
}

/// Plan status response
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PlanStatusResponse {
    pub plan_id: String,
    pub status: String,
    pub completed_steps: Vec<String>,
    pub current_step: Option<String>,
    pub error: Option<String>,
}

/// Plan list request
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PlanListRequest {
    pub limit: i32,
    pub offset: i32,
}

/// Plan list response
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PlanListResponse {
    pub plans: Vec<PlanSummary>,
    pub total: i32,
}

/// Plan summary
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PlanSummary {
    pub plan_id: String,
    pub goal: String,
    pub status: String,
    pub created_at: u64,
}
