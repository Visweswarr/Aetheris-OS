//! LLM schema stubs

use alloc::string::String;
use alloc::vec::Vec;

/// Session handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionHandle(pub u64);

// Backend identifiers
pub const BACKEND_NULL: u8 = 0;
pub const BACKEND_LOCAL: u8 = 1;
pub const BACKEND_REMOTE: u8 = 2;

// Message roles
pub const ROLE_USER: u32 = 0;
pub const ROLE_SYSTEM: u32 = 1;
pub const ROLE_ASSISTANT: u32 = 2;

/// Per-message structure used by chat-style prompts.
#[derive(Debug, Clone)]
pub struct MessageV1 {
    pub role: u32,
    pub content: String,
}

/// Quota limits applied to an LLM adapter.
#[derive(Debug, Clone, Default)]
pub struct QuotaLimitsV1 {
    pub tpm: u32,
    pub bpm: u32,
    pub ts_ms: u32,
}

/// Prompt V1
#[derive(Debug, Clone)]
pub struct PromptV1 {
    pub messages: Vec<MessageV1>,
    pub tools: Option<Vec<ToolCallV1>>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

/// Tool call structure used by streaming chunks and prompt tool specs.
#[derive(Debug, Clone)]
pub struct ToolCallV1 {
    pub name: String,
}

/// Completion chunk V1
#[derive(Debug, Clone)]
pub struct CompletionChunkV1 {
    pub text: String,
    pub is_final: bool,
    /// Monotonically increasing sequence number within a single LLM session.
    /// Used by whylog and the streaming consumer to detect dropped chunks.
    pub seq: u64,
    pub token: Option<String>,
    pub tool: Option<ToolCallV1>,
    pub finish: Option<bool>,
}

/// Adapter config V1
#[derive(Debug, Clone)]
pub struct AdapterConfigV1 {
    pub backend: u8,
    pub quotas: QuotaLimitsV1,
    pub redactions: Vec<String>,
}

/// Serialize prompt
pub fn serialize_prompt(_prompt: &PromptV1) -> Vec<u8> {
    Vec::new()
}

/// Deserialize prompt
pub fn deserialize_prompt(_data: &[u8]) -> Option<PromptV1> {
    None
}

/// Serialize chunk
pub fn serialize_chunk(_chunk: &CompletionChunkV1) -> Vec<u8> {
    Vec::new()
}

/// Deserialize chunk
pub fn deserialize_chunk(_data: &[u8]) -> Option<CompletionChunkV1> {
    None
}

/// Serialize config
pub fn serialize_config(_config: &AdapterConfigV1) -> Vec<u8> {
    Vec::new()
}

/// Deserialize config
pub fn deserialize_config(_data: &[u8]) -> Option<AdapterConfigV1> {
    None
}

/// LLM request
#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub session_id: u64,
    pub prompt: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl LlmRequest {
    pub fn new(session_id: u64, prompt: String) -> Self {
        Self {
            session_id,
            prompt,
            max_tokens: 1024,
            temperature: 0.7,
        }
    }
}

/// LLM response
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub session_id: u64,
    pub text: String,
    pub tokens_used: u32,
    pub finish_reason: FinishReason,
}

/// Finish reason
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinishReason {
    Stop,
    Length,
    Error,
}

/// LLM event types
#[derive(Debug, Clone)]
pub enum LlmEvent {
    TextChunk { session_id: u64, seq: u64, text: String },
    ToolCall { session_id: u64, seq: u64, tool: String },
    Finish { session_id: u64, seq: u64, finish: bool },
}
