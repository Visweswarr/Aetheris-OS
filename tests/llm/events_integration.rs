use kernel::llm::{
    LlmService, init_llm_service, cleanup_llm_service, is_llm_available,
    open_llm_session, send_llm_prompt, receive_llm_chunks, close_llm_session
};
use kernel::llm::schema::{
    AdapterConfigV1, QuotaLimitsV1, PromptV1, MessageV1, ROLE_USER, BACKEND_NULL
};
use kernel::secman::capstore::CapStore;
use kernel::syscall::handlers::world::CAP_LLM_USE;

#[test]
fn test_llm_token_streaming() {
    // Initialize service
    init_llm_service().unwrap();
    CapStore::grant_capability(CAP_LLM_USE);
    
    let config = AdapterConfigV1 {
        backend: BACKEND_NULL,
        quotas: QuotaLimitsV1 {
            tpm: 1000,
            bpm: 10000,
            ts_ms: 2000,
        },
        redactions: vec![],
    };
    
    let session_handle = open_llm_session(config).unwrap();
    
    // Send prompt that will generate multiple tokens
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Hello world".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let result = send_llm_prompt(session_handle, &mut prompt);
    assert!(result.is_ok());
    
    // Receive chunks
    let chunks = receive_llm_chunks(session_handle, 10).unwrap();
    
    // Should have multiple chunks
    assert!(chunks.len() >= 3); // "Hello", "world", finish
    
    // Check first token
    assert_eq!(chunks[0].seq, 1);
    assert_eq!(chunks[0].token, Some("Hello".to_string()));
    
    // Check second token
    assert_eq!(chunks[1].seq, 2);
    assert_eq!(chunks[1].token, Some("world".to_string()));
    
    // Check finish chunk
    let last_chunk = chunks.last().unwrap();
    assert!(last_chunk.finish.is_some());
    
    // Clean up
    close_llm_session(session_handle).unwrap();
    cleanup_llm_service();
}

#[test]
fn test_llm_event_publishing() {
    // Initialize service
    init_llm_service().unwrap();
    CapStore::grant_capability(CAP_LLM_USE);
    
    let config = AdapterConfigV1 {
        backend: BACKEND_NULL,
        quotas: QuotaLimitsV1 {
            tpm: 1000,
            bpm: 10000,
            ts_ms: 2000,
        },
        redactions: vec![],
    };
    
    let session_handle = open_llm_session(config).unwrap();
    
    // Send prompt
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Test".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let result = send_llm_prompt(session_handle, &mut prompt);
    assert!(result.is_ok());
    
    // The service should have published events to Event Fabric
    // In a real test, we'd verify these events were received
    
    // Clean up
    close_llm_session(session_handle).unwrap();
    cleanup_llm_service();
}

#[test]
fn test_llm_session_events() {
    // Initialize service
    init_llm_service().unwrap();
    CapStore::grant_capability(CAP_LLM_USE);
    
    let config = AdapterConfigV1 {
        backend: BACKEND_NULL,
        quotas: QuotaLimitsV1 {
            tpm: 1000,
            bpm: 10000,
            ts_ms: 2000,
        },
        redactions: vec![],
    };
    
    // Opening session should publish "llm.session.open" event
    let session_handle = open_llm_session(config).unwrap();
    
    // Closing session should publish "llm.session.close" event
    let result = close_llm_session(session_handle);
    assert!(result.is_ok());
    
    // Cleanup
    cleanup_llm_service();
}

#[test]
fn test_llm_finish_events() {
    // Initialize service
    init_llm_service().unwrap();
    CapStore::grant_capability(CAP_LLM_USE);
    
    let config = AdapterConfigV1 {
        backend: BACKEND_NULL,
        quotas: QuotaLimitsV1 {
            tpm: 1000,
            bpm: 10000,
            ts_ms: 2000,
        },
        redactions: vec![],
    };
    
    let session_handle = open_llm_session(config).unwrap();
    
    // Send prompt
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Single".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let result = send_llm_prompt(session_handle, &mut prompt);
    assert!(result.is_ok());
    
    // Receive chunks
    let chunks = receive_llm_chunks(session_handle, 10).unwrap();
    
    // Should have finish event
    let last_chunk = chunks.last().unwrap();
    assert!(last_chunk.finish.is_some());
    
    // The service should have published "llm.finish" event
    // In a real test, we'd verify this event was received
    
    // Clean up
    close_llm_session(session_handle).unwrap();
    cleanup_llm_service();
}

#[test]
fn test_llm_tool_call_events() {
    // Initialize service
    init_llm_service().unwrap();
    CapStore::grant_capability(CAP_LLM_USE);
    
    let config = AdapterConfigV1 {
        backend: BACKEND_NULL,
        quotas: QuotaLimitsV1 {
            tpm: 1000,
            bpm: 10000,
            ts_ms: 2000,
        },
        redactions: vec![],
    };
    
    let session_handle = open_llm_session(config).unwrap();
    
    // Send prompt with tools
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Use tool".to_string() },
        ],
        tools: Some(vec![
            kernel::llm::schema::ToolDefV1 {
                name: "test_tool".to_string(),
                description: "Test tool".to_string(),
                parameters: "{}".to_string(),
            },
        ]),
        max_tokens: None,
        temperature: None,
    };
    
    let result = send_llm_prompt(session_handle, &mut prompt);
    assert!(result.is_ok());
    
    // The service should handle tool definitions
    // In a real implementation, this might generate tool call events
    
    // Clean up
    close_llm_session(session_handle).unwrap();
    cleanup_llm_service();
}

#[test]
fn test_llm_event_ordering() {
    // Initialize service
    init_llm_service().unwrap();
    CapStore::grant_capability(CAP_LLM_USE);
    
    let config = AdapterConfigV1 {
        backend: BACKEND_NULL,
        quotas: QuotaLimitsV1 {
            tpm: 1000,
            bpm: 10000,
            ts_ms: 2000,
        },
        redactions: vec![],
    };
    
    let session_handle = open_llm_session(config).unwrap();
    
    // Send prompt
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "One two three".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let result = send_llm_prompt(session_handle, &mut prompt);
    assert!(result.is_ok());
    
    // Receive chunks
    let chunks = receive_llm_chunks(session_handle, 10).unwrap();
    
    // Check that sequence numbers are in order
    for (i, chunk) in chunks.iter().enumerate() {
        assert_eq!(chunk.seq, (i + 1) as u32);
    }
    
    // Clean up
    close_llm_session(session_handle).unwrap();
    cleanup_llm_service();
}

#[test]
fn test_llm_event_metrics() {
    // Initialize service
    init_llm_service().unwrap();
    CapStore::grant_capability(CAP_LLM_USE);
    
    let config = AdapterConfigV1 {
        backend: BACKEND_NULL,
        quotas: QuotaLimitsV1 {
            tpm: 1000,
            bpm: 10000,
            ts_ms: 2000,
        },
        redactions: vec![],
    };
    
    let session_handle = open_llm_session(config).unwrap();
    
    // Send prompt
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Metrics test".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let result = send_llm_prompt(session_handle, &mut prompt);
    assert!(result.is_ok());
    
    // Get service stats
    let stats = kernel::llm::get_llm_stats().unwrap();
    
    // Should have processed some prompts
    assert!(stats.prompts_sent > 0);
    assert!(stats.tokens_generated > 0);
    assert!(stats.bytes_processed > 0);
    
    // Clean up
    close_llm_session(session_handle).unwrap();
    cleanup_llm_service();
}

#[test]
fn test_llm_event_fabric_integration() {
    // Initialize service
    init_llm_service().unwrap();
    CapStore::grant_capability(CAP_LLM_USE);
    
    let config = AdapterConfigV1 {
        backend: BACKEND_NULL,
        quotas: QuotaLimitsV1 {
            tpm: 1000,
            bpm: 10000,
            ts_ms: 2000,
        },
        redactions: vec![],
    };
    
    let session_handle = open_llm_session(config).unwrap();
    
    // Send multiple prompts to test event accumulation
    for i in 0..3 {
        let mut prompt = PromptV1 {
            messages: vec![
                MessageV1 { 
                    role: ROLE_USER, 
                    content: format!("Prompt {}", i) 
                },
            ],
            tools: None,
            max_tokens: None,
            temperature: None,
        };
        
        let result = send_llm_prompt(session_handle, &mut prompt);
        assert!(result.is_ok());
        
        // Receive chunks for each prompt
        let chunks = receive_llm_chunks(session_handle, 10).unwrap();
        assert!(!chunks.is_empty());
    }
    
    // Check stats
    let stats = kernel::llm::get_llm_stats().unwrap();
    assert_eq!(stats.prompts_sent, 3);
    
    // Clean up
    close_llm_session(session_handle).unwrap();
    cleanup_llm_service();
}

#[test]
fn test_llm_event_error_handling() {
    // Initialize service
    init_llm_service().unwrap();
    CapStore::grant_capability(CAP_LLM_USE);
    
    let config = AdapterConfigV1 {
        backend: BACKEND_NULL,
        quotas: QuotaLimitsV1 {
            tpm: 1000,
            bpm: 10000,
            ts_ms: 2000,
        },
        redactions: vec![],
    };
    
    let session_handle = open_llm_session(config).unwrap();
    
    // Try to receive chunks before sending prompt
    let chunks = receive_llm_chunks(session_handle, 10).unwrap();
    // Should return empty list, not error
    
    // Send prompt
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Error test".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let result = send_llm_prompt(session_handle, &mut prompt);
    assert!(result.is_ok());
    
    // Now should have chunks
    let chunks = receive_llm_chunks(session_handle, 10).unwrap();
    assert!(!chunks.is_empty());
    
    // Clean up
    close_llm_session(session_handle).unwrap();
    cleanup_llm_service();
}

#[test]
fn test_llm_event_cleanup() {
    // Initialize service
    init_llm_service().unwrap();
    CapStore::grant_capability(CAP_LLM_USE);
    
    let config = AdapterConfigV1 {
        backend: BACKEND_NULL,
        quotas: QuotaLimitsV1 {
            tpm: 1000,
            bpm: 10000,
            ts_ms: 2000,
        },
        redactions: vec![],
    };
    
    let session_handle = open_llm_session(config).unwrap();
    
    // Send prompt
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Cleanup test".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let result = send_llm_prompt(session_handle, &mut prompt);
    assert!(result.is_ok());
    
    // Close session (should publish close event)
    let result = close_llm_session(session_handle);
    assert!(result.is_ok());
    
    // Cleanup service (should clean up all event handlers)
    cleanup_llm_service();
}
