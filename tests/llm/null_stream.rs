use kernel::llm::backend::{NullBackend, LlmBackend};
use kernel::llm::schema::{PromptV1, MessageV1, ROLE_USER, ROLE_ASSISTANT, FINISH_STOP};
use kernel::llm::session::ChunkRing;

#[test]
fn test_null_backend_creation() {
    let backend = NullBackend::new();
    assert!(backend.is_available());
    assert_eq!(backend.get_backend_type(), 0); // BACKEND_NULL
}

#[test]
fn test_null_backend_simple_prompt() {
    let backend = NullBackend::new();
    let prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Hello".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let mut sink = ChunkRing::new(100);
    let result = backend.send(&prompt, &mut sink);
    
    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert_eq!(metrics.tokens, 2); // "Hello" + finish
    assert_eq!(metrics.finish_reason, FINISH_STOP);
    
    // Check chunks
    let chunks = sink.get_all();
    assert_eq!(chunks.len(), 2);
    
    // First chunk should be the token
    assert_eq!(chunks[0].seq, 1);
    assert_eq!(chunks[0].token, Some("Hello".to_string()));
    assert!(chunks[0].finish.is_none());
    
    // Second chunk should be finish
    assert_eq!(chunks[1].seq, 2);
    assert!(chunks[1].token.is_none());
    assert_eq!(chunks[1].finish, Some(FINISH_STOP));
}

#[test]
fn test_null_backend_multiple_words() {
    let backend = NullBackend::new();
    let prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Hello world".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let mut sink = ChunkRing::new(100);
    let result = backend.send(&prompt, &mut sink);
    
    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert_eq!(metrics.tokens, 3); // "Hello", "world", finish
    
    let chunks = sink.get_all();
    assert_eq!(chunks.len(), 3);
    
    // Check token chunks
    assert_eq!(chunks[0].token, Some("Hello".to_string()));
    assert_eq!(chunks[1].token, Some("world".to_string()));
    
    // Check finish chunk
    assert_eq!(chunks[2].finish, Some(FINISH_STOP));
}

#[test]
fn test_null_backend_conversation_history() {
    let backend = NullBackend::new();
    let prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Hi".to_string() },
            MessageV1 { role: ROLE_ASSISTANT, content: "Hello there!".to_string() },
            MessageV1 { role: ROLE_USER, content: "How are you?".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let mut sink = ChunkRing::new(100);
    let result = backend.send(&prompt, &mut sink);
    
    assert!(result.is_ok());
    let metrics = result.unwrap();
    
    // Should only process the last user message
    let chunks = sink.get_all();
    assert_eq!(chunks.len(), 4); // "How", "are", "you?", finish
    
    // Check that it's based on the last user message
    assert_eq!(chunks[0].token, Some("How".to_string()));
    assert_eq!(chunks[1].token, Some("are".to_string()));
    assert_eq!(chunks[2].token, Some("you?".to_string()));
}

#[test]
fn test_null_backend_no_user_message() {
    let backend = NullBackend::new();
    let prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_ASSISTANT, content: "Hello there!".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let mut sink = ChunkRing::new(100);
    let result = backend.send(&prompt, &mut sink);
    
    assert!(result.is_ok());
    let metrics = result.unwrap();
    
    // Should provide default response
    let chunks = sink.get_all();
    assert_eq!(chunks.len(), 3); // "Hello", "world", finish
    
    assert_eq!(chunks[0].token, Some("Hello".to_string()));
    assert_eq!(chunks[1].token, Some("world".to_string()));
    assert_eq!(chunks[2].finish, Some(FINISH_STOP));
}

#[test]
fn test_null_backend_deterministic() {
    let backend = NullBackend::new();
    let prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Test message".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    // Run multiple times
    let mut sink1 = ChunkRing::new(100);
    let result1 = backend.send(&prompt, &mut sink1);
    
    let mut sink2 = ChunkRing::new(100);
    let result2 = backend.send(&prompt, &mut sink2);
    
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    
    let metrics1 = result1.unwrap();
    let metrics2 = result2.unwrap();
    
    // Metrics should be identical
    assert_eq!(metrics1.tokens, metrics2.tokens);
    assert_eq!(metrics1.bytes, metrics2.bytes);
    assert_eq!(metrics1.finish_reason, metrics2.finish_reason);
    
    // Chunks should be identical
    let chunks1 = sink1.get_all();
    let chunks2 = sink2.get_all();
    
    assert_eq!(chunks1.len(), chunks2.len());
    for (chunk1, chunk2) in chunks1.iter().zip(chunks2.iter()) {
        assert_eq!(chunk1.seq, chunk2.seq);
        assert_eq!(chunk1.token, chunk2.token);
        assert_eq!(chunk1.finish, chunk2.finish);
    }
}

#[test]
fn test_null_backend_empty_prompt() {
    let backend = NullBackend::new();
    let prompt = PromptV1 {
        messages: vec![],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let mut sink = ChunkRing::new(100);
    let result = backend.send(&prompt, &mut sink);
    
    assert!(result.is_ok());
    let metrics = result.unwrap();
    
    // Should provide default response
    let chunks = sink.get_all();
    assert_eq!(chunks.len(), 3); // "Hello", "world", finish
}

#[test]
fn test_null_backend_whitespace_handling() {
    let backend = NullBackend::new();
    let prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "  Multiple   spaces  ".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let mut sink = ChunkRing::new(100);
    let result = backend.send(&prompt, &mut sink);
    
    assert!(result.is_ok());
    let metrics = result.unwrap();
    
    let chunks = sink.get_all();
    // Should split on whitespace and ignore empty tokens
    assert_eq!(chunks.len(), 4); // "Multiple", "spaces", finish
    
    assert_eq!(chunks[0].token, Some("Multiple".to_string()));
    assert_eq!(chunks[1].token, Some("spaces".to_string()));
}

#[test]
fn test_null_backend_sequence_numbers() {
    let backend = NullBackend::new();
    let prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "One two three".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let mut sink = ChunkRing::new(100);
    let result = backend.send(&prompt, &mut sink);
    
    assert!(result.is_ok());
    
    let chunks = sink.get_all();
    assert_eq!(chunks.len(), 4); // "One", "two", "three", finish
    
    // Check sequence numbers are sequential
    for (i, chunk) in chunks.iter().enumerate() {
        assert_eq!(chunk.seq, (i + 1) as u32);
    }
}

#[test]
fn test_null_backend_metrics_accuracy() {
    let backend = NullBackend::new();
    let prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Hello world".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let mut sink = ChunkRing::new(100);
    let result = backend.send(&prompt, &mut sink);
    
    assert!(result.is_ok());
    let metrics = result.unwrap();
    
    // Check token count
    assert_eq!(metrics.tokens, 3); // "Hello", "world", finish
    
    // Check byte count (should include actual token bytes)
    let expected_bytes = "Hello".len() + "world".len();
    assert_eq!(metrics.bytes as usize, expected_bytes);
    
    // Check finish reason
    assert_eq!(metrics.finish_reason, FINISH_STOP);
    
    // Check time (should be minimal for null backend)
    assert!(metrics.time_ms <= 10); // Should be very fast
}

#[test]
fn test_null_backend_tool_calls() {
    let backend = NullBackend::new();
    let prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Use calculator".to_string() },
        ],
        tools: Some(vec![]), // Empty tools list
        max_tokens: None,
        temperature: None,
    };
    
    let mut sink = ChunkRing::new(100);
    let result = backend.send(&prompt, &mut sink);
    
    assert!(result.is_ok());
    
    let chunks = sink.get_all();
    // Should still work with tools (even if empty)
    assert_eq!(chunks.len(), 3); // "Use", "calculator", finish
}

#[test]
fn test_null_backend_large_prompt() {
    let backend = NullBackend::new();
    let content = "word ".repeat(100); // 100 words
    let prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: content.clone() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let mut sink = ChunkRing::new(1000);
    let result = backend.send(&prompt, &mut sink);
    
    assert!(result.is_ok());
    let metrics = result.unwrap();
    
    // Should handle large prompts
    assert_eq!(metrics.tokens, 101); // 100 words + finish
    assert!(metrics.bytes > 0);
    
    let chunks = sink.get_all();
    assert_eq!(chunks.len(), 101);
    
    // Check last chunk is finish
    assert_eq!(chunks[100].finish, Some(FINISH_STOP));
}
