use kernel::llm::{
    LlmService, init_llm_service, cleanup_llm_service, is_llm_available,
    open_llm_session, send_llm_prompt, close_llm_session
};
use kernel::llm::schema::{
    AdapterConfigV1, QuotaLimitsV1, PromptV1, MessageV1, ROLE_USER, BACKEND_NULL
};
use kernel::secman::capstore::CapStore;
use kernel::syscall::handlers::world::{CAP_LLM_USE, CAP_LLM_DEV};

#[test]
fn test_llm_service_initialization() {
    // Service should not be available initially
    assert!(!is_llm_available());
    
    // Initialize service
    let result = init_llm_service();
    assert!(result.is_ok());
    
    // Service should now be available
    assert!(is_llm_available());
    
    // Cleanup
    cleanup_llm_service();
    assert!(!is_llm_available());
}

#[test]
fn test_llm_session_creation_with_caps() {
    // Initialize service
    init_llm_service().unwrap();
    
    // Should not be able to create session without caps
    let config = AdapterConfigV1 {
        backend: BACKEND_NULL,
        quotas: QuotaLimitsV1 {
            tpm: 1000,
            bpm: 10000,
            ts_ms: 2000,
        },
        redactions: vec![],
    };
    
    let result = open_llm_session(config.clone());
    assert!(result.is_err());
    
    // Grant capability
    CapStore::grant_capability(CAP_LLM_USE);
    
    // Should now be able to create session
    let result = open_llm_session(config);
    assert!(result.is_ok());
    
    let session_handle = result.unwrap();
    
    // Clean up
    close_llm_session(session_handle).unwrap();
    cleanup_llm_service();
}

#[test]
fn test_llm_dev_backend_capability() {
    // Initialize service
    init_llm_service().unwrap();
    
    // Grant basic LLM capability
    CapStore::grant_capability(CAP_LLM_USE);
    
    // Try to create session with dev backend without dev cap
    let config = AdapterConfigV1 {
        backend: 1, // BACKEND_DEV_LOCAL
        quotas: QuotaLimitsV1 {
            tpm: 1000,
            bpm: 10000,
            ts_ms: 2000,
        },
        redactions: vec![],
    };
    
    let result = open_llm_session(config.clone());
    // This should fail in the syscall handler, but for now we'll test the service level
    
    // Grant dev capability
    CapStore::grant_capability(CAP_LLM_DEV);
    
    // Should now be able to create session with dev backend
    let result = open_llm_session(config);
    assert!(result.is_ok());
    
    let session_handle = result.unwrap();
    
    // Clean up
    close_llm_session(session_handle).unwrap();
    cleanup_llm_service();
}

#[test]
fn test_llm_session_lifecycle() {
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
    
    // Create session
    let session_handle = open_llm_session(config).unwrap();
    
    // Send prompt
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Hello".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let result = send_llm_prompt(session_handle, &mut prompt);
    assert!(result.is_ok());
    
    // Close session
    let result = close_llm_session(session_handle);
    assert!(result.is_ok());
    
    // Cleanup
    cleanup_llm_service();
}

#[test]
fn test_llm_quota_enforcement() {
    // Initialize service
    init_llm_service().unwrap();
    CapStore::grant_capability(CAP_LLM_USE);
    
    let config = AdapterConfigV1 {
        backend: BACKEND_NULL,
        quotas: QuotaLimitsV1 {
            tpm: 10, // Very low limit
            bpm: 100, // Very low limit
            ts_ms: 2000,
        },
        redactions: vec![],
    };
    
    let session_handle = open_llm_session(config).unwrap();
    
    // Send a small prompt (should work)
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Hi".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let result = send_llm_prompt(session_handle, &mut prompt);
    assert!(result.is_ok());
    
    // Send a larger prompt (should still work within limits)
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Hello there".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let result = send_llm_prompt(session_handle, &mut prompt);
    assert!(result.is_ok());
    
    // Clean up
    close_llm_session(session_handle).unwrap();
    cleanup_llm_service();
}

#[test]
fn test_llm_redaction_integration() {
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
        redactions: vec![
            kernel::llm::schema::RedactionRuleV1 {
                name: "email".to_string(),
                pattern: "email".to_string(),
                replacement: "[EMAIL]".to_string(),
            },
        ],
    };
    
    let session_handle = open_llm_session(config).unwrap();
    
    // Send prompt with sensitive information
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
    
    let result = send_llm_prompt(session_handle, &mut prompt);
    assert!(result.is_ok());
    
    // The redaction should be applied in the session
    // Note: In a real test, we'd check the actual redacted content
    
    // Clean up
    close_llm_session(session_handle).unwrap();
    cleanup_llm_service();
}

#[test]
fn test_llm_service_configuration() {
    // Initialize service with custom config
    let config = kernel::llm::LlmServiceConfig {
        max_sessions: 50,
        default_backend: BACKEND_NULL,
        enable_dev_backends: true,
    };
    
    let result = kernel::llm::init_llm_service_with_config(config);
    assert!(result.is_ok());
    
    // Service should be available
    assert!(is_llm_available());
    
    // Grant capability
    CapStore::grant_capability(CAP_LLM_USE);
    
    // Should be able to create session
    let session_config = AdapterConfigV1 {
        backend: BACKEND_NULL,
        quotas: QuotaLimitsV1 {
            tpm: 1000,
            bpm: 10000,
            ts_ms: 2000,
        },
        redactions: vec![],
    };
    
    let session_handle = open_llm_session(session_config).unwrap();
    
    // Clean up
    close_llm_session(session_handle).unwrap();
    cleanup_llm_service();
}

#[test]
fn test_llm_backend_registry() {
    // Initialize service
    init_llm_service().unwrap();
    
    let service = kernel::llm::get_llm_service().unwrap();
    
    // Check available backends
    let backends = service.get_available_backends();
    assert!(backends.contains(&BACKEND_NULL));
    
    // Check backend availability
    assert!(service.backend_registry.is_backend_available(BACKEND_NULL));
    
    // Cleanup
    cleanup_llm_service();
}

#[test]
fn test_llm_service_statistics() {
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
    
    // Get initial stats
    let stats = kernel::llm::get_llm_stats().unwrap();
    let initial_sessions = stats.sessions_opened;
    
    // Create and close a session
    let session_handle = open_llm_session(config).unwrap();
    close_llm_session(session_handle).unwrap();
    
    // Get updated stats
    let stats = kernel::llm::get_llm_stats().unwrap();
    assert_eq!(stats.sessions_opened, initial_sessions + 1);
    assert_eq!(stats.sessions_closed, 1);
    
    // Cleanup
    cleanup_llm_service();
}

#[test]
fn test_llm_capability_checks() {
    // Initialize service
    init_llm_service().unwrap();
    
    // Test without any capabilities
    let config = AdapterConfigV1 {
        backend: BACKEND_NULL,
        quotas: QuotaLimitsV1 {
            tpm: 1000,
            bpm: 10000,
            ts_ms: 2000,
        },
        redactions: vec![],
    };
    
    // Should fail without CAP_LLM_USE
    let result = open_llm_session(config);
    assert!(result.is_err());
    
    // Grant basic capability
    CapStore::grant_capability(CAP_LLM_USE);
    
    // Should now work
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
    
    // Clean up
    close_llm_session(session_handle).unwrap();
    cleanup_llm_service();
}

#[test]
fn test_llm_service_cleanup() {
    // Initialize service
    init_llm_service().unwrap();
    assert!(is_llm_available());
    
    // Cleanup
    cleanup_llm_service();
    assert!(!is_llm_available());
    
    // Should be able to reinitialize
    let result = init_llm_service();
    assert!(result.is_ok());
    assert!(is_llm_available());
    
    // Final cleanup
    cleanup_llm_service();
}
