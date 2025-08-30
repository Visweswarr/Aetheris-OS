use kernel::llm::schema::{
    PromptV1, MessageV1, ToolDefV1, ToolCallV1, CompletionChunkV1,
    RedactionRuleV1, AdapterConfigV1, QuotaLimitsV1,
    ROLE_USER, ROLE_ASSISTANT, BACKEND_NULL, FINISH_STOP,
    schema_hash, serialize_prompt, deserialize_prompt,
    serialize_chunk, deserialize_chunk, serialize_config, deserialize_config
};

#[test]
fn test_schema_hash_stability() {
    let hash1 = schema_hash();
    let hash2 = schema_hash();
    assert_eq!(hash1, hash2, "Schema hash should be stable across calls");
    
    // Verify hash is not all zeros
    assert_ne!(hash1, [0u8; 32], "Schema hash should not be all zeros");
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
fn test_tool_def_roundtrip() {
    let tool = ToolDefV1 {
        name: "calculator".to_string(),
        description: "Basic arithmetic operations".to_string(),
        parameters: r#"{"type": "object", "properties": {"expression": {"type": "string"}}}"#.to_string(),
    };

    let serialized = serde_cbor::to_vec(&tool).unwrap();
    let deserialized: ToolDefV1 = serde_cbor::from_slice(&serialized).unwrap();
    
    assert_eq!(tool, deserialized, "Tool definition should roundtrip correctly");
}

#[test]
fn test_tool_call_roundtrip() {
    let tool_call = ToolCallV1 {
        name: "calculator".to_string(),
        args: r#"{"expression": "2 + 2"}"#.to_string(),
    };

    let serialized = serde_cbor::to_vec(&tool_call).unwrap();
    let deserialized: ToolCallV1 = serde_cbor::from_slice(&serialized).unwrap();
    
    assert_eq!(tool_call, deserialized, "Tool call should roundtrip correctly");
}

#[test]
fn test_quota_limits_roundtrip() {
    let quotas = QuotaLimitsV1 {
        tpm: 1000,
        bpm: 10000,
        ts_ms: 2000,
    };

    let serialized = serde_cbor::to_vec(&quotas).unwrap();
    let deserialized: QuotaLimitsV1 = serde_cbor::from_slice(&serialized).unwrap();
    
    assert_eq!(quotas, deserialized, "Quota limits should roundtrip correctly");
}

#[test]
fn test_redaction_rule_roundtrip() {
    let rule = RedactionRuleV1 {
        name: "phone".to_string(),
        pattern: "phone".to_string(),
        replacement: "[PHONE]".to_string(),
    };

    let serialized = serde_cbor::to_vec(&rule).unwrap();
    let deserialized: RedactionRuleV1 = serde_cbor::from_slice(&serialized).unwrap();
    
    assert_eq!(rule, deserialized, "Redaction rule should roundtrip correctly");
}

#[test]
fn test_message_roles() {
    assert_eq!(ROLE_USER, 1);
    assert_eq!(ROLE_ASSISTANT, 2);
}

#[test]
fn test_backend_types() {
    assert_eq!(BACKEND_NULL, 0);
}

#[test]
fn test_finish_reasons() {
    assert_eq!(FINISH_STOP, 0);
}

#[test]
fn test_schema_version() {
    use kernel::llm::schema::SCHEMA_VERSION;
    assert_eq!(SCHEMA_VERSION, 1);
}

#[test]
fn test_prompt_with_tools() {
    let prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Calculate 2+2".to_string() },
        ],
        tools: Some(vec![
            ToolDefV1 {
                name: "calculator".to_string(),
                description: "Basic arithmetic".to_string(),
                parameters: "{}".to_string(),
            },
        ]),
        max_tokens: Some(50),
        temperature: Some(0.1),
    };

    let serialized = serialize_prompt(&prompt).unwrap();
    let deserialized = deserialize_prompt(&serialized).unwrap();
    
    assert_eq!(prompt, deserialized, "Prompt with tools should roundtrip correctly");
    assert_eq!(deserialized.tools.as_ref().unwrap().len(), 1);
}

#[test]
fn test_completion_chunk_with_tool() {
    let chunk = CompletionChunkV1 {
        seq: 1,
        token: None,
        tool: Some(ToolCallV1 {
            name: "calculator".to_string(),
            args: r#"{"expression": "2+2"}"#.to_string(),
        }),
        finish: None,
    };

    let serialized = serialize_chunk(&chunk).unwrap();
    let deserialized = deserialize_chunk(&serialized).unwrap();
    
    assert_eq!(chunk, deserialized, "Completion chunk with tool should roundtrip correctly");
}

#[test]
fn test_completion_chunk_with_finish() {
    let chunk = CompletionChunkV1 {
        seq: 1,
        token: Some("Hello".to_string()),
        tool: None,
        finish: Some(FINISH_STOP),
    };

    let serialized = serialize_chunk(&chunk).unwrap();
    let deserialized = deserialize_chunk(&serialized).unwrap();
    
    assert_eq!(chunk, deserialized, "Completion chunk with finish should roundtrip correctly");
}

#[test]
fn test_adapter_config_with_multiple_redactions() {
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
            RedactionRuleV1 {
                name: "phone".to_string(),
                pattern: "phone".to_string(),
                replacement: "[PHONE]".to_string(),
            },
            RedactionRuleV1 {
                name: "key".to_string(),
                pattern: "key".to_string(),
                replacement: "[KEY]".to_string(),
            },
        ],
    };

    let serialized = serialize_config(&config).unwrap();
    let deserialized = deserialize_config(&serialized).unwrap();
    
    assert_eq!(config, deserialized, "Adapter config with multiple redactions should roundtrip correctly");
    assert_eq!(deserialized.redactions.len(), 3);
}
