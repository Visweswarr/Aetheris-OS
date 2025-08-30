use kernel::llm::session::{QuotaState, LlmSession, ChunkRing};
use kernel::llm::schema::{AdapterConfigV1, QuotaLimitsV1, PromptV1, MessageV1, ROLE_USER, BACKEND_NULL};

#[test]
fn test_quota_state_creation() {
    let limits = QuotaLimitsV1 {
        tpm: 1000,
        bpm: 10000,
        ts_ms: 2000,
    };
    
    let quotas = QuotaState::new(&limits);
    let (tpm, bpm, ts_ms) = quotas.get_limits();
    
    assert_eq!(tpm, 1000);
    assert_eq!(bpm, 10000);
    assert_eq!(ts_ms, 2000);
}

#[test]
fn test_quota_checking_basic() {
    let limits = QuotaLimitsV1 {
        tpm: 100,
        bpm: 1000,
        ts_ms: 2000,
    };
    
    let quotas = QuotaState::new(&limits);
    
    // Should pass
    assert!(quotas.check_quotas(50, 500).is_ok());
    
    // Should fail
    assert!(quotas.check_quotas(150, 500).is_err());
    assert!(quotas.check_quotas(50, 1500).is_err());
}

#[test]
fn test_quota_consumption() {
    let limits = QuotaLimitsV1 {
        tpm: 100,
        bpm: 1000,
        ts_ms: 2000,
    };
    
    let quotas = QuotaState::new(&limits);
    
    // Initial usage should be zero
    let (tpm_usage, bpm_usage) = quotas.get_usage();
    assert_eq!(tpm_usage, 0);
    assert_eq!(bpm_usage, 0);
    
    // Consume some quotas
    quotas.consume_quotas(30, 300);
    
    let (tpm_usage, bpm_usage) = quotas.get_usage();
    assert_eq!(tpm_usage, 30);
    assert_eq!(bpm_usage, 300);
    
    // Should still be under limit
    assert!(quotas.check_quotas(50, 500).is_ok());
    
    // Should fail when exceeding
    assert!(quotas.check_quotas(80, 500).is_err()); // 30 + 80 = 110 > 100
    assert!(quotas.check_quotas(50, 800).is_err()); // 300 + 800 = 1100 > 1000
}

#[test]
fn test_quota_reset() {
    let limits = QuotaLimitsV1 {
        tpm: 100,
        bpm: 1000,
        ts_ms: 2000,
    };
    
    let quotas = QuotaState::new(&limits);
    
    // Consume some quotas
    quotas.consume_quotas(50, 500);
    
    let (tpm_usage, bpm_usage) = quotas.get_usage();
    assert_eq!(tpm_usage, 50);
    assert_eq!(bpm_usage, 500);
    
    // Reset quotas
    quotas.reset();
    
    let (tpm_usage, bpm_usage) = quotas.get_usage();
    assert_eq!(tpm_usage, 0);
    assert_eq!(bpm_usage, 0);
    
    // Should pass after reset
    assert!(quotas.check_quotas(50, 500).is_ok());
}

#[test]
fn test_quota_time_based_reset() {
    let limits = QuotaLimitsV1 {
        tpm: 100,
        bpm: 1000,
        ts_ms: 2000,
    };
    
    let quotas = QuotaState::new(&limits);
    
    // Consume some quotas
    quotas.consume_quotas(50, 500);
    
    // Simulate time passing (more than 60 seconds)
    quotas.update_clock(70_000); // 70 seconds
    
    // Quotas should be reset
    let (tpm_usage, bpm_usage) = quotas.get_usage();
    assert_eq!(tpm_usage, 0);
    assert_eq!(bpm_usage, 0);
    
    // Should pass after time-based reset
    assert!(quotas.check_quotas(50, 500).is_ok());
}

#[test]
fn test_session_quota_enforcement() {
    let config = AdapterConfigV1 {
        backend: BACKEND_NULL,
        quotas: QuotaLimitsV1 {
            tpm: 100,
            bpm: 1000,
            ts_ms: 2000,
        },
        redactions: vec![],
    };
    
    let mut session = LlmSession::new(1, config).unwrap();
    
    // Create a prompt that should fit within quotas
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Hello".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    // Should pass quota check
    let result = session.enforce_quotas(prompt.messages[0].content.len(), 10);
    assert!(result.is_ok());
    
    // Create a prompt that exceeds quotas
    let large_content = "x".repeat(2000); // 2000 bytes
    let result = session.enforce_quotas(large_content.len(), 150); // 150 tokens
    
    // Should fail quota check
    assert!(result.is_err());
}

#[test]
fn test_session_quota_consumption() {
    let config = AdapterConfigV1 {
        backend: BACKEND_NULL,
        quotas: QuotaLimitsV1 {
            tpm: 100,
            bpm: 1000,
            ts_ms: 2000,
        },
        redactions: vec![],
    };
    
    let mut session = LlmSession::new(1, config).unwrap();
    
    // Get initial usage
    let (initial_tpm, initial_bpm) = session.quotas.get_usage();
    assert_eq!(initial_tpm, 0);
    assert_eq!(initial_bpm, 0);
    
    // Send a prompt (this will consume quotas)
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Hello world".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let result = session.send_prompt(&mut prompt);
    assert!(result.is_ok());
    
    // Check that quotas were consumed
    let (tpm_usage, bpm_usage) = session.quotas.get_usage();
    assert!(tpm_usage > 0);
    assert!(bpm_usage > 0);
}

#[test]
fn test_quota_edge_cases() {
    let limits = QuotaLimitsV1 {
        tpm: 100,
        bpm: 1000,
        ts_ms: 2000,
    };
    
    let quotas = QuotaState::new(&limits);
    
    // Test exact limits
    assert!(quotas.check_quotas(100, 1000).is_ok());
    
    // Test exceeding by 1
    assert!(quotas.check_quotas(101, 1000).is_err());
    assert!(quotas.check_quotas(100, 1001).is_err());
    
    // Test zero consumption
    assert!(quotas.check_quotas(0, 0).is_ok());
}

#[test]
fn test_quota_clock_update() {
    let limits = QuotaLimitsV1 {
        tpm: 100,
        bpm: 1000,
        ts_ms: 2000,
    };
    
    let quotas = QuotaState::new(&limits);
    
    // Consume some quotas
    quotas.consume_quotas(50, 500);
    
    // Update clock to just before reset threshold
    quotas.update_clock(59_000); // 59 seconds
    
    // Quotas should not be reset yet
    let (tpm_usage, bpm_usage) = quotas.get_usage();
    assert_eq!(tpm_usage, 50);
    assert_eq!(bpm_usage, 500);
    
    // Update clock to trigger reset
    quotas.update_clock(60_000); // 60 seconds
    
    // Quotas should be reset
    let (tpm_usage, bpm_usage) = quotas.get_usage();
    assert_eq!(tpm_usage, 0);
    assert_eq!(bpm_usage, 0);
}

#[test]
fn test_session_quota_limits() {
    let config = AdapterConfigV1 {
        backend: BACKEND_NULL,
        quotas: QuotaLimitsV1 {
            tpm: 50,
            bpm: 500,
            ts_ms: 1000,
        },
        redactions: vec![],
    };
    
    let mut session = LlmSession::new(1, config).unwrap();
    
    // Test that we can't exceed the configured limits
    let (tpm_limit, bpm_limit, ts_limit) = session.quotas.get_limits();
    assert_eq!(tpm_limit, 50);
    assert_eq!(bpm_limit, 500);
    assert_eq!(ts_limit, 1000);
    
    // Try to consume more than the limit
    let result = session.quotas.check_quotas(60, 600);
    assert!(result.is_err());
}

#[test]
fn test_quota_usage_tracking() {
    let limits = QuotaLimitsV1 {
        tpm: 100,
        bpm: 1000,
        ts_ms: 2000,
    };
    
    let quotas = QuotaState::new(&limits);
    
    // Track multiple consumption operations
    quotas.consume_quotas(10, 100);
    quotas.consume_quotas(20, 200);
    quotas.consume_quotas(15, 150);
    
    let (tpm_usage, bpm_usage) = quotas.get_usage();
    assert_eq!(tpm_usage, 45); // 10 + 20 + 15
    assert_eq!(bpm_usage, 450); // 100 + 200 + 150
    
    // Should still be under limit
    assert!(quotas.check_quotas(50, 500).is_ok());
    
    // Should fail when exceeding
    assert!(quotas.check_quotas(60, 500).is_err()); // 45 + 60 = 105 > 100
}

#[test]
fn test_session_quota_integration() {
    let config = AdapterConfigV1 {
        backend: BACKEND_NULL,
        quotas: QuotaLimitsV1 {
            tpm: 100,
            bpm: 1000,
            ts_ms: 2000,
        },
        redactions: vec![],
    };
    
    let mut session = LlmSession::new(1, config).unwrap();
    
    // Send multiple prompts to test quota accumulation
    for i in 0..3 {
        let mut prompt = PromptV1 {
            messages: vec![
                MessageV1 { 
                    role: ROLE_USER, 
                    content: format!("Message {}", i) 
                },
            ],
            tools: None,
            max_tokens: None,
            temperature: None,
        };
        
        let result = session.send_prompt(&mut prompt);
        assert!(result.is_ok());
    }
    
    // Check that quotas were consumed
    let (tpm_usage, bpm_usage) = session.quotas.get_usage();
    assert!(tpm_usage > 0);
    assert!(bpm_usage > 0);
    
    // Should still be able to send more (within limits)
    let mut prompt = PromptV1 {
        messages: vec![
            MessageV1 { role: ROLE_USER, content: "Final message".to_string() },
        ],
        tools: None,
        max_tokens: None,
        temperature: None,
    };
    
    let result = session.send_prompt(&mut prompt);
    assert!(result.is_ok());
}
