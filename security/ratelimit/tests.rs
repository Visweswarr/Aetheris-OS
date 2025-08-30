//! Comprehensive tests for the rate limiting system
//!
//! Tests cover token bucket mechanics, DID-based tracking, burst allowances,
//! penalty mechanisms, and HTTP 429 response handling.

use super::*;
use std::time::{Duration, SystemTime};
use tokio::time::{sleep, timeout};

// Test utilities
fn create_test_rules() -> RateLimitRules {
    RateLimitRules {
        default: BucketConfig {
            capacity: 10,
            refill_amount: 5,
            refill_interval: Duration::from_secs(10),
            initial_tokens: Some(10),
            allow_burst: true,
            burst_capacity: Some(5),
            burst_cooldown: Some(Duration::from_secs(30)),
        },
        endpoints: HashMap::from([
            ("api/auth".to_string(), BucketConfig {
                capacity: 5,
                refill_amount: 1,
                refill_interval: Duration::from_secs(60),
                initial_tokens: Some(5),
                allow_burst: false,
                burst_capacity: None,
                burst_cooldown: None,
            }),
            ("api/upload".to_string(), BucketConfig {
                capacity: 2,
                refill_amount: 1,
                refill_interval: Duration::from_secs(300), // 5 minutes
                initial_tokens: Some(2),
                allow_burst: true,
                burst_capacity: Some(1),
                burst_cooldown: Some(Duration::from_secs(600)), // 10 minutes
            }),
        ]),
        did_overrides: HashMap::from([
            ("did:premium:user123".to_string(), BucketConfig {
                capacity: 100,
                refill_amount: 50,
                refill_interval: Duration::from_secs(10),
                initial_tokens: Some(100),
                allow_burst: true,
                burst_capacity: Some(50),
                burst_cooldown: Some(Duration::from_secs(60)),
            }),
        ]),
        global_limit: Some(BucketConfig {
            capacity: 1000,
            refill_amount: 100,
            refill_interval: Duration::from_secs(60),
            initial_tokens: Some(1000),
            allow_burst: false,
            burst_capacity: None,
            burst_cooldown: None,
        }),
    }
}

fn create_test_penalty_config() -> PenaltyConfig {
    PenaltyConfig {
        enabled: true,
        violation_threshold: 3,
        base_penalty_duration: Duration::from_secs(60), // 1 minute for testing
        escalation_multiplier: 2.0,
        max_penalty_duration: Duration::from_secs(600), // 10 minutes for testing
        violation_window: Duration::from_secs(300), // 5 minutes for testing
        penalty_types: HashMap::from([
            ("rate_limit_violation".to_string(), 1.0),
            ("severe_violation".to_string(), 2.0),
            ("abuse_detection".to_string(), 5.0),
        ]),
    }
}

fn create_test_rate_limiter() -> RateLimiter {
    RateLimiter::new(create_test_rules(), create_test_penalty_config())
}

#[tokio::test]
async fn test_basic_rate_limiting() {
    let limiter = create_test_rate_limiter();
    let did = "did:test:user1";
    let endpoint = "api/test";

    // Should allow requests within capacity
    for i in 0..10 {
        let result = limiter.check_rate_limit(did, endpoint, 1).await;
        assert!(result.is_ok(), "Request {} should be allowed", i);
        
        let decision = result.unwrap();
        assert!(decision.allowed);
        assert_eq!(decision.tokens_remaining, 10 - 1 - i);
    }

    // Should deny the 11th request
    let result = limiter.check_rate_limit(did, endpoint, 1).await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        RateLimitError::RateLimitExceeded { did: error_did } => {
            assert_eq!(error_did, did);
        }
        _ => panic!("Expected RateLimitExceeded error"),
    }
}

#[tokio::test]
async fn test_rate_limit_headers() {
    let limiter = create_test_rate_limiter();
    let did = "did:test:user2";
    let endpoint = "api/test";

    let result = limiter.check_rate_limit(did, endpoint, 3).await.unwrap();
    
    assert!(result.allowed);
    assert_eq!(result.tokens_remaining, 7);
    
    // Check that headers are properly generated
    assert!(result.headers.contains_key("X-RateLimit-Limit"));
    assert!(result.headers.contains_key("X-RateLimit-Remaining"));
    assert!(result.headers.contains_key("X-RateLimit-Reset"));
    assert!(result.headers.contains_key("X-RateLimit-Burst-Limit"));
    assert!(result.headers.contains_key("X-RateLimit-Burst-Remaining"));
    
    assert_eq!(result.headers["X-RateLimit-Limit"], "10");
    assert_eq!(result.headers["X-RateLimit-Remaining"], "7");
    assert_eq!(result.headers["X-RateLimit-Burst-Limit"], "5");
    assert_eq!(result.headers["X-RateLimit-Burst-Remaining"], "5");
}

#[tokio::test]
async fn test_token_bucket_refill() {
    let limiter = create_test_rate_limiter();
    let did = "did:test:user3";
    let endpoint = "api/test";

    // Consume all tokens
    for _ in 0..10 {
        limiter.check_rate_limit(did, endpoint, 1).await.unwrap();
    }

    // Should be rate limited now
    let result = limiter.check_rate_limit(did, endpoint, 1).await;
    assert!(result.is_err());

    // Wait for partial refill (config: 5 tokens per 10 seconds)
    sleep(Duration::from_secs(10)).await;

    // Should now allow 5 more requests
    for i in 0..5 {
        let result = limiter.check_rate_limit(did, endpoint, 1).await;
        assert!(result.is_ok(), "Request {} should be allowed after refill", i);
    }

    // Should be rate limited again
    let result = limiter.check_rate_limit(did, endpoint, 1).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_burst_allowance() {
    let limiter = create_test_rate_limiter();
    let did = "did:test:user4";
    let endpoint = "api/test";

    // Consume all normal tokens
    for _ in 0..10 {
        limiter.check_rate_limit(did, endpoint, 1).await.unwrap();
    }

    // Should now use burst capacity (5 tokens)
    for i in 0..5 {
        let result = limiter.check_rate_limit(did, endpoint, 1).await;
        assert!(result.is_ok(), "Burst request {} should be allowed", i);
    }

    // Should be completely rate limited now
    let result = limiter.check_rate_limit(did, endpoint, 1).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_burst_cooldown() {
    let limiter = create_test_rate_limiter();
    let did = "did:test:user5";
    let endpoint = "api/test";

    // Consume all tokens including burst
    for _ in 0..15 {
        let _ = limiter.check_rate_limit(did, endpoint, 1).await;
    }

    // Wait for normal refill but not burst cooldown
    sleep(Duration::from_secs(10)).await;

    // Should get 5 normal tokens but no burst yet
    for i in 0..5 {
        let result = limiter.check_rate_limit(did, endpoint, 1).await;
        assert!(result.is_ok(), "Normal request {} should be allowed", i);
    }

    // Burst should still be on cooldown
    let result = limiter.check_rate_limit(did, endpoint, 1).await;
    assert!(result.is_err());

    // TODO: Test burst cooldown expiry (would require longer wait or time mocking)
}

#[tokio::test]
async fn test_endpoint_specific_limits() {
    let limiter = create_test_rate_limiter();
    let did = "did:test:user6";

    // Test auth endpoint (5 tokens, no burst)
    for i in 0..5 {
        let result = limiter.check_rate_limit(did, "api/auth", 1).await;
        assert!(result.is_ok(), "Auth request {} should be allowed", i);
    }

    // Should be rate limited now
    let result = limiter.check_rate_limit(did, "api/auth", 1).await;
    assert!(result.is_err());

    // But other endpoints should still work
    let result = limiter.check_rate_limit(did, "api/other", 1).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_did_specific_overrides() {
    let limiter = create_test_rate_limiter();
    let premium_did = "did:premium:user123";
    let regular_did = "did:test:user7";
    let endpoint = "api/test";

    // Premium user should have higher limits (100 tokens)
    for i in 0..50 {
        let result = limiter.check_rate_limit(premium_did, endpoint, 1).await;
        assert!(result.is_ok(), "Premium request {} should be allowed", i);
    }

    // Regular user should have normal limits (10 tokens)
    for i in 0..10 {
        let result = limiter.check_rate_limit(regular_did, endpoint, 1).await;
        assert!(result.is_ok(), "Regular request {} should be allowed", i);
    }

    // Regular user should be rate limited
    let result = limiter.check_rate_limit(regular_did, endpoint, 1).await;
    assert!(result.is_err());

    // But premium user should still have capacity
    let result = limiter.check_rate_limit(premium_did, endpoint, 1).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_penalty_system() {
    let limiter = create_test_rate_limiter();
    let did = "did:test:violator";
    let endpoint = "api/test";

    // Trigger violations by exceeding rate limit multiple times
    for violation in 0..5 {
        // Consume all tokens
        for _ in 0..15 {
            let _ = limiter.check_rate_limit(did, endpoint, 1).await;
        }

        // Try to make more requests to trigger violations
        for _ in 0..3 {
            let result = limiter.check_rate_limit(did, endpoint, 5).await;
            assert!(result.is_err());
        }

        // Wait a bit between violation attempts
        sleep(Duration::from_millis(100)).await;
    }

    // Check if penalty was applied
    let status = limiter.get_rate_limit_status(did, endpoint).await.unwrap();
    assert!(status.penalty.is_some(), "Penalty should be applied after violations");

    let penalty = status.penalty.unwrap();
    assert_eq!(penalty.penalty_type, "rate_limit_violation");
    assert!(penalty.violation_count >= 3);
    assert!(penalty.expires_at > SystemTime::now());

    // Requests should be denied due to penalty
    let result = limiter.check_rate_limit(did, endpoint, 1).await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        RateLimitError::PenaltyActive { penalty_type, .. } => {
            assert_eq!(penalty_type, "rate_limit_violation");
        }
        _ => panic!("Expected PenaltyActive error"),
    }
}

#[tokio::test]
async fn test_penalty_escalation() {
    let limiter = create_test_rate_limiter();
    let did = "did:test:repeat_violator";
    let endpoint = "api/test";

    // Apply manual penalty to simulate previous violation
    limiter.apply_penalty(did, "rate_limit_violation", Duration::from_millis(10)).await.unwrap();

    // Wait for penalty to expire
    sleep(Duration::from_millis(20)).await;

    // Trigger new violations
    for _ in 0..15 {
        let _ = limiter.check_rate_limit(did, endpoint, 1).await;
    }
    
    for _ in 0..5 {
        let _ = limiter.check_rate_limit(did, endpoint, 10).await;
    }

    // Check escalated penalty
    let status = limiter.get_rate_limit_status(did, endpoint).await.unwrap();
    if let Some(penalty) = status.penalty {
        assert!(penalty.escalation_level > 0, "Penalty should be escalated");
        // Escalated penalty should be longer than base duration
        let penalty_duration = penalty.expires_at.duration_since(penalty.applied_at).unwrap();
        assert!(penalty_duration > Duration::from_secs(60));
    }
}

#[tokio::test]
async fn test_global_rate_limit() {
    let limiter = create_test_rate_limiter();
    
    // Create many DIDs to test global limit
    let mut dids = Vec::new();
    for i in 0..50 {
        dids.push(format!("did:test:global_user{}", i));
    }

    // Each DID consumes tokens from global bucket
    let mut successful_requests = 0;
    for did in &dids {
        for tokens in 1..=20 {
            match limiter.check_rate_limit(did, "api/test", tokens).await {
                Ok(_) => successful_requests += tokens,
                Err(_) => break, // Hit global limit
            }
        }
    }

    // Should hit global limit before all local limits
    // Global limit is 1000 tokens, so we shouldn't be able to make unlimited requests
    assert!(successful_requests <= 1000, "Global rate limit should be enforced");
}

#[tokio::test]
async fn test_rate_limit_status() {
    let limiter = create_test_rate_limiter();
    let did = "did:test:status_check";
    let endpoint = "api/test";

    // Make some requests
    for _ in 0..7 {
        limiter.check_rate_limit(did, endpoint, 1).await.unwrap();
    }

    // Check status without consuming tokens
    let status = limiter.get_rate_limit_status(did, endpoint).await.unwrap();
    assert!(status.allowed);
    assert_eq!(status.tokens_remaining, 3);

    // Status check shouldn't consume tokens
    let status2 = limiter.get_rate_limit_status(did, endpoint).await.unwrap();
    assert_eq!(status2.tokens_remaining, 3);

    // Actual request should consume tokens
    let result = limiter.check_rate_limit(did, endpoint, 1).await.unwrap();
    assert_eq!(result.tokens_remaining, 2);
}

#[tokio::test]
async fn test_rate_limit_reset() {
    let limiter = create_test_rate_limiter();
    let did = "did:test:reset";
    let endpoint = "api/test";

    // Consume all tokens
    for _ in 0..15 {
        let _ = limiter.check_rate_limit(did, endpoint, 1).await;
    }

    // Should be rate limited
    let result = limiter.check_rate_limit(did, endpoint, 1).await;
    assert!(result.is_err());

    // Reset rate limit
    limiter.reset_rate_limit(did, Some(endpoint)).await.unwrap();

    // Should be allowed again
    let result = limiter.check_rate_limit(did, endpoint, 1).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_manual_penalty_application() {
    let limiter = create_test_rate_limiter();
    let did = "did:test:manual_penalty";
    let endpoint = "api/test";

    // Apply manual penalty
    limiter.apply_penalty(did, "abuse_detection", Duration::from_secs(5)).await.unwrap();

    // Requests should be denied
    let result = limiter.check_rate_limit(did, endpoint, 1).await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        RateLimitError::PenaltyActive { penalty_type, .. } => {
            assert_eq!(penalty_type, "abuse_detection");
        }
        _ => panic!("Expected PenaltyActive error"),
    }

    // Wait for penalty to expire
    sleep(Duration::from_secs(6)).await;

    // Should be allowed again
    let result = limiter.check_rate_limit(did, endpoint, 1).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_statistics_collection() {
    let limiter = create_test_rate_limiter();
    let did1 = "did:test:stats1";
    let did2 = "did:test:stats2";

    // Make requests with different DIDs
    for _ in 0..5 {
        limiter.check_rate_limit(did1, "api/test", 1).await.unwrap();
        limiter.check_rate_limit(did2, "api/other", 1).await.unwrap();
    }

    // Trigger some violations
    for _ in 0..20 {
        let _ = limiter.check_rate_limit(did1, "api/test", 1).await;
    }

    // Get aggregate statistics
    let stats = limiter.get_statistics(None).await.unwrap();
    assert!(stats["total_dids"].as_u64().unwrap() >= 2);
    assert!(stats["total_requests"].as_u64().unwrap() >= 10);

    // Get DID-specific statistics
    let did1_stats = limiter.get_statistics(Some(did1)).await.unwrap();
    assert_eq!(did1_stats["did"], did1);
    assert!(did1_stats["total_requests"].as_u64().unwrap() >= 5);
}

#[tokio::test]
async fn test_cleanup_functionality() {
    let limiter = create_test_rate_limiter();
    let did = "did:test:cleanup";

    // Apply a short penalty
    limiter.apply_penalty(did, "test_penalty", Duration::from_millis(10)).await.unwrap();

    // Verify penalty exists
    let status = limiter.get_rate_limit_status(did, "api/test").await.unwrap();
    assert!(status.penalty.is_some());

    // Wait for penalty to expire
    sleep(Duration::from_millis(20)).await;

    // Run cleanup
    let cleaned_count = limiter.cleanup().await.unwrap();
    assert!(cleaned_count > 0);

    // Penalty should be cleaned up
    let status = limiter.get_rate_limit_status(did, "api/test").await.unwrap();
    assert!(status.penalty.is_none());
}

#[tokio::test]
async fn test_violation_severity_calculation() {
    let limiter = create_test_rate_limiter();

    // Test different severity levels
    assert_eq!(limiter.calculate_violation_severity(2, 1.0), ViolationSeverity::Minor);
    assert_eq!(limiter.calculate_violation_severity(5, 1.0), ViolationSeverity::Moderate);
    assert_eq!(limiter.calculate_violation_severity(10, 1.0), ViolationSeverity::Severe);
    assert_eq!(limiter.calculate_violation_severity(100, 1.0), ViolationSeverity::Critical);
}

#[tokio::test]
async fn test_concurrent_access() {
    let limiter = Arc::new(create_test_rate_limiter());
    let did = "did:test:concurrent";
    let endpoint = "api/test";

    // Spawn multiple concurrent requests
    let mut handles = vec![];
    for i in 0..20 {
        let limiter_clone = Arc::clone(&limiter);
        let did_clone = did.to_string();
        let endpoint_clone = endpoint.to_string();
        
        let handle = tokio::spawn(async move {
            limiter_clone.check_rate_limit(&did_clone, &endpoint_clone, 1).await
        });
        handles.push(handle);
    }

    // Collect results
    let mut success_count = 0;
    let mut error_count = 0;
    
    for handle in handles {
        match handle.await.unwrap() {
            Ok(_) => success_count += 1,
            Err(_) => error_count += 1,
        }
    }

    // Should allow exactly 10 requests (normal capacity) plus up to 5 burst
    assert!(success_count <= 15);
    assert!(success_count >= 10); // At least normal capacity should succeed
    assert_eq!(success_count + error_count, 20);
}

#[tokio::test]
async fn test_window_reset_behavior() {
    let limiter = create_test_rate_limiter();
    let did = "did:test:window_reset";
    let endpoint = "api/test";

    // Consume all tokens
    for _ in 0..10 {
        limiter.check_rate_limit(did, endpoint, 1).await.unwrap();
    }

    // Record the reset time
    let status = limiter.get_rate_limit_status(did, endpoint).await.unwrap();
    let reset_time = status.reset_time;

    // Should be rate limited now
    let result = limiter.check_rate_limit(did, endpoint, 1).await;
    assert!(result.is_err());

    // Wait for the reset window (partial refill should occur)
    sleep(Duration::from_secs(10)).await;

    // Should now allow more requests
    let result = limiter.check_rate_limit(did, endpoint, 1).await;
    assert!(result.is_ok(), "Request should be allowed after window reset");

    // Verify the reset time has been updated
    let new_status = limiter.get_rate_limit_status(did, endpoint).await.unwrap();
    assert!(new_status.reset_time > reset_time);
}

#[tokio::test]
async fn test_http_429_equivalent_behavior() {
    let limiter = create_test_rate_limiter();
    let did = "did:test:http429";
    let endpoint = "api/test";

    // Helper function to simulate HTTP middleware behavior
    async fn simulate_http_request(
        limiter: &RateLimiter,
        did: &str,
        endpoint: &str,
    ) -> (u16, HashMap<String, String>) {
        match limiter.check_rate_limit(did, endpoint, 1).await {
            Ok(decision) => (200, decision.headers),
            Err(RateLimitError::RateLimitExceeded { .. }) => {
                // Get headers for 429 response
                let status = limiter.get_rate_limit_status(did, endpoint).await.unwrap();
                (429, status.headers)
            }
            Err(RateLimitError::PenaltyActive { .. }) => {
                // Get headers for penalty response
                let status = limiter.get_rate_limit_status(did, endpoint).await.unwrap();
                (429, status.headers)
            }
            Err(_) => (500, HashMap::new()),
        }
    }

    // Make successful requests
    for i in 0..10 {
        let (status_code, headers) = simulate_http_request(&limiter, did, endpoint).await;
        assert_eq!(status_code, 200, "Request {} should return 200", i);
        assert_eq!(headers["X-RateLimit-Remaining"], (9 - i).to_string());
    }

    // Next request should return 429
    let (status_code, headers) = simulate_http_request(&limiter, did, endpoint).await;
    assert_eq!(status_code, 429);
    assert_eq!(headers["X-RateLimit-Remaining"], "0");
    assert!(headers.contains_key("X-RateLimit-Reset"));

    // Verify reset time is in the future
    let reset_timestamp: u64 = headers["X-RateLimit-Reset"].parse().unwrap();
    let current_timestamp = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    assert!(reset_timestamp > current_timestamp);
}

// Performance and stress tests
#[tokio::test]
async fn test_high_load_performance() {
    let limiter = Arc::new(create_test_rate_limiter());
    let start_time = std::time::Instant::now();
    
    // Simulate high load with many DIDs
    let mut handles = vec![];
    for did_num in 0..100 {
        let limiter_clone = Arc::clone(&limiter);
        let handle = tokio::spawn(async move {
            let did = format!("did:test:load{}", did_num);
            let mut success_count = 0;
            
            for _ in 0..10 {
                if limiter_clone.check_rate_limit(&did, "api/test", 1).await.is_ok() {
                    success_count += 1;
                }
            }
            success_count
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    let mut total_success = 0;
    for handle in handles {
        total_success += handle.await.unwrap();
    }

    let duration = start_time.elapsed();
    println!("High load test: {} successful requests in {:?}", total_success, duration);

    // Should complete in reasonable time
    assert!(duration < Duration::from_secs(5), "High load test should complete quickly");
    assert!(total_success > 0, "At least some requests should succeed");
}

// Helper function for integration tests
pub async fn test_rate_limiter_integration() -> RateLimiter {
    create_test_rate_limiter()
}

// Middleware integration test
#[tokio::test]
async fn test_middleware_integration_pattern() {
    let limiter = Arc::new(create_test_rate_limiter());
    
    // Simulate middleware function
    async fn rate_limit_middleware(
        limiter: Arc<RateLimiter>,
        did: String,
        endpoint: String,
    ) -> Result<(), (u16, String)> {
        match limiter.check_rate_limit(&did, &endpoint, 1).await {
            Ok(_) => Ok(()),
            Err(RateLimitError::RateLimitExceeded { .. }) => {
                Err((429, "Rate limit exceeded".to_string()))
            }
            Err(RateLimitError::PenaltyActive { penalty_type, until }) => {
                Err((429, format!("Penalty active: {} until {:?}", penalty_type, until)))
            }
            Err(e) => Err((500, format!("Internal error: {}", e))),
        }
    }

    // Test successful request
    let result = rate_limit_middleware(
        Arc::clone(&limiter),
        "did:test:middleware".to_string(),
        "api/test".to_string(),
    ).await;
    assert!(result.is_ok());

    // Exhaust rate limit
    for _ in 0..15 {
        let _ = rate_limit_middleware(
            Arc::clone(&limiter),
            "did:test:middleware".to_string(),
            "api/test".to_string(),
        ).await;
    }

    // Should return 429
    let result = rate_limit_middleware(
        Arc::clone(&limiter),
        "did:test:middleware".to_string(),
        "api/test".to_string(),
    ).await;
    assert!(result.is_err());
    
    let (status_code, error_message) = result.unwrap_err();
    assert_eq!(status_code, 429);
    assert!(error_message.contains("Rate limit exceeded"));
}
