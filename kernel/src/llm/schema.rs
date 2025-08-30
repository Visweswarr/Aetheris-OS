use serde::{Deserialize, Serialize};
use serde_cbor;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;

/// LLM prompt schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct PromptV1 {
    /// Messages in the conversation
    pub messages: Vec<MessageV1>,
    /// Optional tool definitions
    pub tools: Option<Vec<ToolDefV1>>,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Temperature for generation (0.0 to 2.0)
    pub temperature: Option<f32>,
}

/// Message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct MessageV1 {
    /// Role of the message sender
    pub role: u8,
    /// Message content
    pub content: String,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct ToolDefV1 {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool parameters schema
    pub parameters: String,
}

/// Tool call (preview-only signal)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct ToolCallV1 {
    /// Tool name
    pub name: String,
    /// Tool arguments (JSON string)
    pub args: String,
}

/// Completion chunk (token or finish)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct CompletionChunkV1 {
    /// Sequence number
    pub seq: u32,
    /// Token text (if present)
    pub token: Option<String>,
    /// Tool call (if present)
    pub tool: Option<ToolCallV1>,
    /// Finish reason (if present)
    pub finish: Option<u8>,
}

/// Redaction rule for privacy protection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct RedactionRuleV1 {
    /// Rule name
    pub name: String,
    /// Pattern to match (simplified regex)
    pub pattern: String,
    /// Replacement text
    pub replacement: String,
}

/// LLM adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct AdapterConfigV1 {
    /// Backend type
    pub backend: u8,
    /// Quota limits
    pub quotas: QuotaLimitsV1,
    /// Redaction rules
    pub redactions: Vec<RedactionRuleV1>,
}

/// Quota limits for LLM operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct QuotaLimitsV1 {
    /// Tokens per minute
    pub tpm: u32,
    /// Bytes per minute
    pub bpm: u32,
    /// Time slice per request (milliseconds)
    pub ts_ms: u32,
}

/// LLM session handle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct SessionHandle(pub u64);

/// LLM metrics from backend
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct LlmMetrics {
    /// Tokens generated
    pub tokens: u32,
    /// Bytes processed
    pub bytes: u32,
    /// Time taken (milliseconds)
    pub time_ms: u32,
    /// Finish reason
    pub finish_reason: u8,
}

/// Message roles
pub const ROLE_SYSTEM: u8 = 0;
pub const ROLE_USER: u8 = 1;
pub const ROLE_ASSISTANT: u8 = 2;
pub const ROLE_TOOL: u8 = 3;

/// Backend types
pub const BACKEND_NULL: u8 = 0;
pub const BACKEND_DEV_LOCAL: u8 = 1;

/// Finish reasons
pub const FINISH_STOP: u8 = 0;
pub const FINISH_LENGTH: u8 = 1;
pub const FINISH_TOOL_CALL: u8 = 2;
pub const FINISH_ERROR: u8 = 3;

/// Schema version constant
pub const SCHEMA_VERSION: u16 = 1;

/// Compute schema hash for LLM schemas
pub fn schema_hash() -> [u8; 32] {
    use blake3::Hasher;
    
    let mut hasher = blake3::Hasher::new();
    
    // Hash the schema structure deterministically
    let schema_def = r#"
        PromptV1: messages, tools, max_tokens, temperature
        MessageV1: role, content
        ToolDefV1: name, description, parameters
        ToolCallV1: name, args
        CompletionChunkV1: seq, token, tool, finish
        RedactionRuleV1: name, pattern, replacement
        AdapterConfigV1: backend, quotas, redactions
        QuotaLimitsV1: tpm, bpm, ts_ms
        LlmMetrics: tokens, bytes, time_ms, finish_reason
    "#;
    
    hasher.update(schema_def.as_bytes());
    hasher.finalize().into()
}

/// Serialize prompt to CBOR
pub fn serialize_prompt(prompt: &PromptV1) -> Result<Vec<u8>, &'static str> {
    serde_cbor::to_vec(prompt).map_err(|_| "Failed to serialize prompt")
}

/// Deserialize prompt from CBOR
pub fn deserialize_prompt(data: &[u8]) -> Result<PromptV1, &'static str> {
    serde_cbor::from_slice(data).map_err(|_| "Failed to deserialize prompt")
}

/// Serialize completion chunk to CBOR
pub fn serialize_chunk(chunk: &CompletionChunkV1) -> Result<Vec<u8>, &'static str> {
    serde_cbor::to_vec(chunk).map_err(|_| "Failed to serialize completion chunk")
}

/// Deserialize completion chunk from CBOR
pub fn deserialize_chunk(data: &[u8]) -> Result<CompletionChunkV1, &'static str> {
    serde_cbor::from_slice(data).map_err(|_| "Failed to deserialize completion chunk")
}

/// Serialize adapter config to CBOR
pub fn serialize_config(config: &AdapterConfigV1) -> Result<Vec<u8>, &'static str> {
    serde_cbor::to_vec(config).map_err(|_| "Failed to serialize adapter config")
}

/// Deserialize adapter config from CBOR
pub fn deserialize_config(data: &[u8]) -> Result<AdapterConfigV1, &'static str> {
    serde_cbor::from_slice(data).map_err(|_| "Failed to deserialize adapter config")
}

/// Apply redaction rules to a prompt
pub fn apply_redactions(prompt: &mut PromptV1, rules: &[RedactionRuleV1]) {
    for message in &mut prompt.messages {
        for rule in rules {
            // Simple string replacement (in a real implementation, this would use regex)
            if rule.pattern.contains("email") {
                message.content = message.content.replace("@", "[EMAIL]");
            }
            if rule.pattern.contains("phone") {
                message.content = message.content.replace("+1-", "[PHONE]");
            }
            if rule.pattern.contains("key") {
                message.content = message.content.replace("sk-", "[KEY]");
            }
        }
    }
}

/// Validate prompt size limits
pub fn validate_prompt_size(prompt: &PromptV1) -> Result<(), &'static str> {
    let total_size: usize = prompt.messages.iter()
        .map(|m| m.content.len())
        .sum();
    
    if total_size > 32 * 1024 {
        return Err("Prompt exceeds 32 KiB limit");
    }
    
    Ok(())
}

/// Validate completion chunk size
pub fn validate_chunk_size(chunk: &CompletionChunkV1) -> Result<(), &'static str> {
    let token_size = chunk.token.as_ref().map_or(0, |t| t.len());
    let tool_size = chunk.tool.as_ref().map_or(0, |t| t.name.len() + t.args.len());
    
    if token_size + tool_size > 32 * 1024 {
        return Err("Completion chunk exceeds 32 KiB limit");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_hash_stability() {
        let hash1 = schema_hash();
        let hash2 = schema_hash();
        assert_eq!(hash1, hash2, "Schema hash should be stable across calls");
    }

    #[test]
    fn test_prompt_roundtrip() {
        let prompt = PromptV1 {
            messages: vec![
                MessageV1 { role: ROLE_USER, content: "Hello".to_string() },
                MessageV1 { role: ROLE_ASSISTANT, content: "Hi there!".to_string() },
            ],
            tools: None,
            max_tokens: Some(100),
            temperature: Some(0.7),
        };

        let serialized = serialize_prompt(&prompt).unwrap();
        let deserialized = deserialize_prompt(&serialized).unwrap();
        
        assert_eq!(prompt, deserialized, "Prompt should roundtrip correctly");
    }

    #[test]
    fn test_completion_chunk_roundtrip() {
        let chunk = CompletionChunkV1 {
            seq: 1,
            token: Some("Hello".to_string()),
            tool: None,
            finish: None,
        };

        let serialized = serialize_chunk(&chunk).unwrap();
        let deserialized = deserialize_chunk(&serialized).unwrap();
        
        assert_eq!(chunk, deserialized, "Completion chunk should roundtrip correctly");
    }

    #[test]
    fn test_adapter_config_roundtrip() {
        let config = AdapterConfigV1 {
            backend: BACKEND_NULL,
            quotas: QuotaLimitsV1 {
                tpm: 1000,
                bpm: 10000,
                ts_ms: 2000,
            },
            redactions: vec![
                RedactionRuleV1 {
                    name: "email".to_string(),
                    pattern: "email".to_string(),
                    replacement: "[EMAIL]".to_string(),
                },
            ],
        };

        let serialized = serialize_config(&config).unwrap();
        let deserialized = deserialize_config(&serialized).unwrap();
        
        assert_eq!(config, deserialized, "Adapter config should roundtrip correctly");
    }

    #[test]
    fn test_redaction_application() {
        let mut prompt = PromptV1 {
            messages: vec![
                MessageV1 { 
                    role: ROLE_USER, 
                    content: "My email is user@example.com".to_string() 
                },
            ],
            tools: None,
            max_tokens: None,
            temperature: None,
        };

        let rules = vec![
            RedactionRuleV1 {
                name: "email".to_string(),
                pattern: "email".to_string(),
                replacement: "[EMAIL]".to_string(),
            },
        ];

        apply_redactions(&mut prompt, &rules);
        
        assert!(prompt.messages[0].content.contains("[EMAIL]"));
        assert!(!prompt.messages[0].content.contains("@"));
    }

    #[test]
    fn test_prompt_size_validation() {
        let large_content = "x".repeat(64 * 1024); // 64 KiB
        let prompt = PromptV1 {
            messages: vec![
                MessageV1 { role: ROLE_USER, content: large_content },
            ],
            tools: None,
            max_tokens: None,
            temperature: None,
        };

        let result = validate_prompt_size(&prompt);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Prompt exceeds 32 KiB limit");
    }

    #[test]
    fn test_chunk_size_validation() {
        let large_token = "x".repeat(64 * 1024); // 64 KiB
        let chunk = CompletionChunkV1 {
            seq: 1,
            token: Some(large_token),
            tool: None,
            finish: None,
        };

        let result = validate_chunk_size(&chunk);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Completion chunk exceeds 32 KiB limit");
    }

    #[test]
    fn test_message_roles() {
        assert_eq!(ROLE_SYSTEM, 0);
        assert_eq!(ROLE_USER, 1);
        assert_eq!(ROLE_ASSISTANT, 2);
        assert_eq!(ROLE_TOOL, 3);
    }

    #[test]
    fn test_backend_types() {
        assert_eq!(BACKEND_NULL, 0);
        assert_eq!(BACKEND_DEV_LOCAL, 1);
    }

    #[test]
    fn test_finish_reasons() {
        assert_eq!(FINISH_STOP, 0);
        assert_eq!(FINISH_LENGTH, 1);
        assert_eq!(FINISH_TOOL_CALL, 2);
        assert_eq!(FINISH_ERROR, 3);
    }
}
