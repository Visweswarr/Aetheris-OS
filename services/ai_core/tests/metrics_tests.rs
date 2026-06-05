/**
 * @file metrics_tests.rs
 * @brief Comprehensive tests for AI Core Service metrics
 */
use aetheris_ai_core::metrics::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_metrics_collector_creation() {
    let config = default_metrics_config();
    let collector = AiCoreMetricsCollector::new(config);

    // Test initial state
    let metrics = collector.get_metrics_snapshot().await.unwrap();
    assert_eq!(metrics.requests_total, 0);
    assert_eq!(metrics.tool_calls_total, 0);
    assert_eq!(metrics.tokens_total, 0);
}

#[tokio::test]
async fn test_request_metrics_recording() {
    let config = default_metrics_config();
    let collector = AiCoreMetricsCollector::new(config);

    // Record some requests
    collector.record_request(true, 100.0);
    collector.record_request(true, 150.0);
    collector.record_request(false, 200.0);

    let metrics = collector.get_metrics_snapshot().await.unwrap();

    assert_eq!(metrics.requests_total, 3);
    assert_eq!(metrics.requests_success, 2);
    assert_eq!(metrics.requests_error, 1);

    // Check latency percentiles
    assert!(metrics.latency_p50_ms > 0.0);
    assert!(metrics.latency_p95_ms > 0.0);
    assert!(metrics.latency_p99_ms > 0.0);
    assert_eq!(metrics.latency_max_ms, 200.0);
}

#[tokio::test]
async fn test_token_metrics_recording() {
    let config = default_metrics_config();
    let collector = AiCoreMetricsCollector::new(config);

    // Record token usage
    collector.record_tokens(100, 50);
    collector.record_tokens(200, 75);
    collector.record_tokens(150, 25);

    let metrics = collector.get_metrics_snapshot().await.unwrap();

    assert_eq!(metrics.tokens_total, 600); // 100+50+200+75+150+25
    assert_eq!(metrics.tokens_input, 450); // 100+200+150
    assert_eq!(metrics.tokens_output, 150); // 50+75+25
    assert!(metrics.tokens_per_second > 0.0);
}

#[tokio::test]
async fn test_tool_metrics_recording() {
    let config = default_metrics_config();
    let collector = AiCoreMetricsCollector::new(config);

    // Record tool calls
    collector.record_tool_call(true, Some("open_file"));
    collector.record_tool_call(true, Some("search_files"));
    collector.record_tool_call(false, Some("open_file"));
    collector.record_tool_call(false, Some("unknown_tool"));

    let metrics = collector.get_metrics_snapshot().await.unwrap();

    assert_eq!(metrics.tool_calls_total, 4);
    assert_eq!(metrics.tool_calls_success, 2);
    assert_eq!(metrics.tool_calls_error, 2);

    // Check tool error breakdown
    assert_eq!(metrics.tool_errors_by_type.get("open_file"), Some(&1));
    assert_eq!(metrics.tool_errors_by_type.get("unknown_tool"), Some(&1));
    assert_eq!(metrics.tool_errors_by_type.get("search_files"), None);
}

#[tokio::test]
async fn test_model_metrics_recording() {
    let config = default_metrics_config();
    let collector = AiCoreMetricsCollector::new(config);

    // Record model operations
    collector.record_model_load();
    collector.record_model_load();
    collector.record_model_inference();
    collector.record_model_inference();
    collector.record_model_inference();

    let metrics = collector.get_metrics_snapshot().await.unwrap();

    assert_eq!(metrics.model_loads_total, 2);
    assert_eq!(metrics.model_inferences_total, 3);
}

#[tokio::test]
async fn test_session_metrics_recording() {
    let config = default_metrics_config();
    let collector = AiCoreMetricsCollector::new(config);

    // Record sessions
    collector.record_session();
    collector.record_session();
    collector.record_session();

    let metrics = collector.get_metrics_snapshot().await.unwrap();

    assert_eq!(metrics.sessions_total, 3);
}

#[tokio::test]
async fn test_ngfs_storage() {
    let config = default_metrics_config();
    let mut collector = AiCoreMetricsCollector::new(config);

    let mock_ngfs = std::sync::Arc::new(MockNgfsClient::new());
    collector.set_ngfs_client(mock_ngfs.clone());

    // Record some metrics
    collector.record_request(true, 150.0);
    collector.record_tokens(100, 50);
    collector.record_tool_call(true, Some("test_tool"));

    // Get metrics and store
    let metrics = collector.get_metrics_snapshot().await.unwrap();
    let shard_id = mock_ngfs.store_metrics(&metrics).await.unwrap();

    // Retrieve and verify
    let retrieved = mock_ngfs.retrieve_metrics(&shard_id).await.unwrap();
    assert_eq!(retrieved.requests_total, 1);
    assert_eq!(retrieved.tokens_total, 150);
    assert_eq!(retrieved.tool_calls_total, 1);
}

#[tokio::test]
async fn test_privacy_modes() {
    // Test LocalOnly mode
    let mut config = default_metrics_config();
    config.privacy_mode = PrivacyMode::LocalOnly;
    config.enable_cloud_export = true;

    let collector = AiCoreMetricsCollector::new(config);

    // Record metrics with sensitive data
    collector.record_tool_call(false, Some("sensitive_tool"));

    let metrics = collector.get_metrics_snapshot().await.unwrap();
    let anonymized = collector.anonymize_metrics(&metrics);

    // Tool names should be removed in anonymized mode
    assert!(anonymized.tool_errors_by_type.is_empty());

    // Test Anonymized mode
    let mut config = default_metrics_config();
    config.privacy_mode = PrivacyMode::Anonymized;
    config.enable_cloud_export = true;

    let collector = AiCoreMetricsCollector::new(config);

    let metrics = collector.get_metrics_snapshot().await.unwrap();
    let anonymized = collector.anonymize_metrics(&metrics);

    // Verify anonymization
    assert!(anonymized.tool_errors_by_type.is_empty());
}

#[tokio::test]
async fn test_global_metrics_functions() {
    // Initialize global metrics collector
    let config = default_metrics_config();
    let _collector = init_global_metrics_collector(config).unwrap();

    // Test global functions
    record_request(true, 100.0);
    record_request(false, 200.0);
    record_tokens(50, 25);
    record_tool_call(true, Some("test_tool"));
    record_tool_call(false, Some("failing_tool"));
    record_model_load();
    record_model_inference();
    record_session();

    // Get global collector and verify
    let collector = get_global_metrics_collector().unwrap();
    let metrics = collector.get_metrics_snapshot().await.unwrap();

    assert_eq!(metrics.requests_total, 2);
    assert_eq!(metrics.requests_success, 1);
    assert_eq!(metrics.requests_error, 1);
    assert_eq!(metrics.tokens_total, 75);
    assert_eq!(metrics.tokens_input, 50);
    assert_eq!(metrics.tokens_output, 25);
    assert_eq!(metrics.tool_calls_total, 2);
    assert_eq!(metrics.tool_calls_success, 1);
    assert_eq!(metrics.tool_calls_error, 1);
    assert_eq!(metrics.model_loads_total, 1);
    assert_eq!(metrics.model_inferences_total, 1);
    assert_eq!(metrics.sessions_total, 1);
}

#[tokio::test]
async fn test_metrics_collection_loop() {
    let mut config = default_metrics_config();
    config.collection_interval_secs = 1; // 1 second for testing

    let mut collector = AiCoreMetricsCollector::new(config);

    // Start collection loop
    collector.start().await.unwrap();

    // Record some metrics
    collector.record_request(true, 100.0);
    collector.record_tokens(50, 25);

    // Wait for collection interval
    sleep(Duration::from_millis(1100)).await;

    // Stop collection
    collector.stop().await;

    // Verify metrics were collected
    let metrics = collector.get_metrics_snapshot().await.unwrap();
    assert_eq!(metrics.requests_total, 1);
    assert_eq!(metrics.tokens_total, 75);
}

#[tokio::test]
async fn test_latency_percentile_calculation() {
    let config = default_metrics_config();
    let collector = AiCoreMetricsCollector::new(config);

    // Record requests with known latencies
    let latencies = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];

    for latency in latencies {
        collector.record_request(true, latency);
    }

    let metrics = collector.get_metrics_snapshot().await.unwrap();

    // P50 should be around 50ms (median)
    assert!((metrics.latency_p50_ms - 50.0).abs() < 5.0);

    // P95 should be around 95ms
    assert!((metrics.latency_p95_ms - 95.0).abs() < 5.0);

    // P99 should be around 99ms
    assert!((metrics.latency_p99_ms - 99.0).abs() < 5.0);

    // Max should be 100ms
    assert_eq!(metrics.latency_max_ms, 100.0);
}

#[tokio::test]
async fn test_tokens_per_second_calculation() {
    let config = default_metrics_config();
    let collector = AiCoreMetricsCollector::new(config);

    // Record requests with token usage
    collector.record_request(true, 100.0);
    collector.record_tokens(100, 50);

    collector.record_request(true, 200.0);
    collector.record_tokens(200, 100);

    let metrics = collector.get_metrics_snapshot().await.unwrap();

    // Tokens per second should be calculated
    assert!(metrics.tokens_per_second > 0.0);

    // Total tokens should be correct
    assert_eq!(metrics.tokens_total, 450); // 100+50+200+100
    assert_eq!(metrics.tokens_input, 300); // 100+200
    assert_eq!(metrics.tokens_output, 150); // 50+100
}

#[tokio::test]
async fn test_metrics_cleanup() {
    let mut config = default_metrics_config();
    config.max_samples_per_metric = 5; // Small limit for testing

    let collector = AiCoreMetricsCollector::new(config);

    // Record more samples than the limit
    for i in 0..10 {
        collector.record_request(true, i as f64 * 10.0);
    }

    // Trigger cleanup
    collector.cleanup_old_samples().await.unwrap();

    let metrics = collector.get_metrics_snapshot().await.unwrap();

    // Should still have the request count but limited samples
    assert_eq!(metrics.requests_total, 10);
}

#[tokio::test]
async fn test_error_handling() {
    let config = default_metrics_config();
    let collector = AiCoreMetricsCollector::new(config);

    // Test with invalid data
    collector.record_request(true, -1.0); // Negative latency
    collector.record_tokens(0, 0); // Zero tokens

    let metrics = collector.get_metrics_snapshot().await.unwrap();

    // Should handle gracefully
    assert_eq!(metrics.requests_total, 1);
    assert_eq!(metrics.tokens_total, 0);
}

#[tokio::test]
async fn test_concurrent_metrics_recording() {
    let config = default_metrics_config();
    let collector = std::sync::Arc::new(AiCoreMetricsCollector::new(config));

    // Spawn multiple tasks to record metrics concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let collector_clone = collector.clone();
        let handle = tokio::spawn(async move {
            for j in 0..10 {
                collector_clone.record_request(true, (i * 10 + j) as f64);
                collector_clone.record_tokens(10, 5);
                collector_clone.record_tool_call(true, Some("concurrent_tool"));
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let metrics = collector.get_metrics_snapshot().await.unwrap();

    // Verify concurrent recording worked
    assert_eq!(metrics.requests_total, 100);
    assert_eq!(metrics.tokens_total, 1500); // 100 * (10 + 5)
    assert_eq!(metrics.tool_calls_total, 100);
}

#[tokio::test]
async fn test_metrics_serialization() {
    let config = default_metrics_config();
    let collector = AiCoreMetricsCollector::new(config);

    // Record some metrics
    collector.record_request(true, 100.0);
    collector.record_tokens(50, 25);
    collector.record_tool_call(true, Some("test_tool"));

    let metrics = collector.get_metrics_snapshot().await.unwrap();

    // Test JSON serialization
    let json = serde_json::to_string(&metrics).unwrap();
    let deserialized: AiCoreMetrics = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.requests_total, metrics.requests_total);
    assert_eq!(deserialized.tokens_total, metrics.tokens_total);
    assert_eq!(deserialized.tool_calls_total, metrics.tool_calls_total);

    // Test CBOR serialization
    let cbor = serde_cbor::to_vec(&metrics).unwrap();
    let deserialized_cbor: AiCoreMetrics = serde_cbor::from_slice(&cbor).unwrap();

    assert_eq!(deserialized_cbor.requests_total, metrics.requests_total);
    assert_eq!(deserialized_cbor.tokens_total, metrics.tokens_total);
    assert_eq!(deserialized_cbor.tool_calls_total, metrics.tool_calls_total);
}
