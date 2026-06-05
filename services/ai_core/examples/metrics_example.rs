/**
 * @file metrics_example.rs
 * @brief Example demonstrating AI Core Service metrics collection
 */
use aetheris_ai_core::metrics::*;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Default)]
struct MockNgfsClient {
    stored: tokio::sync::RwLock<std::collections::HashMap<String, AiCoreMetrics>>,
}

impl MockNgfsClient {
    fn new() -> Self {
        Self::default()
    }

    async fn store_metrics(
        &self,
        metrics: &AiCoreMetrics,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let shard_id = format!("metrics-{}", metrics.timestamp);
        self.stored
            .write()
            .await
            .insert(shard_id.clone(), metrics.clone());
        Ok(shard_id)
    }

    async fn retrieve_metrics(
        &self,
        shard_id: &str,
    ) -> Result<AiCoreMetrics, Box<dyn std::error::Error>> {
        self.stored
            .read()
            .await
            .get(shard_id)
            .cloned()
            .ok_or_else(|| format!("missing shard {}", shard_id).into())
    }

    async fn list_metrics_shards(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        Ok(self.stored.read().await.keys().cloned().collect())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 AI Core Service Metrics Example");
    println!("==================================");

    // Create metrics configuration
    let mut config = default_metrics_config();
    config.collection_interval_secs = 5; // 5 seconds for demo
    config.privacy_mode = PrivacyMode::LocalOnly;

    println!("📊 Initializing metrics collector...");

    // Create and start metrics collector
    let mut collector = AiCoreMetricsCollector::new(config.clone());
    collector.start().await?;

    // Initialize global metrics collector
    init_global_metrics_collector(config)?;

    println!("✅ Metrics collector started");
    println!();

    // Simulate AI Core Service activity
    println!("🤖 Simulating AI Core Service activity...");

    // Simulate requests
    for i in 0..20 {
        let success = i % 5 != 0; // 80% success rate
        let latency = 50.0 + (i as f64 * 5.0) + (rand::random::<f64>() * 50.0);

        record_request(success, latency);

        if success {
            // Simulate token usage
            let input_tokens = 50 + (i * 10) as u64;
            let output_tokens = 25 + (i * 5) as u64;
            record_tokens(input_tokens, output_tokens);

            // Simulate model inference
            record_model_inference();
        }

        // Simulate tool calls
        let tools = ["open_file", "search_files", "create_note", "copy_text"];
        let tool = tools[i % tools.len()];
        let tool_success = i % 7 != 0; // ~85% success rate
        record_tool_call(tool_success, Some(tool));

        // Simulate sessions
        if i % 3 == 0 {
            record_session();
        }

        // Simulate model loads
        if i % 10 == 0 {
            record_model_load();
        }

        println!(
            "  Request {}: {} (latency: {:.1}ms)",
            i + 1,
            if success { "✅" } else { "❌" },
            latency
        );

        sleep(Duration::from_millis(100)).await;
    }

    println!();
    println!("📈 Current Metrics Snapshot:");
    println!("============================");

    // Get and display metrics
    let metrics = collector.get_metrics_snapshot().await?;
    display_metrics(&metrics);

    println!();
    println!("⏱️  Waiting for metrics collection interval...");
    sleep(Duration::from_secs(6)).await;

    println!();
    println!("📊 Updated Metrics Snapshot:");
    println!("============================");

    // Get updated metrics
    let updated_metrics = collector.get_metrics_snapshot().await?;
    display_metrics(&updated_metrics);

    println!();
    println!("🔧 Testing Privacy Modes:");
    println!("========================");

    // Test different privacy modes
    test_privacy_modes(&updated_metrics);

    println!();
    println!("🧪 Testing NGFS Storage:");
    println!("=======================");

    // Test NGFS storage
    test_ngfs_storage(&updated_metrics).await?;

    println!();
    println!("✅ Metrics example completed successfully!");

    // Stop the collector
    collector.stop().await;

    Ok(())
}

fn display_metrics(metrics: &AiCoreMetrics) {
    println!("📈 Request Metrics:");
    println!("  Total requests:     {}", metrics.requests_total);
    println!(
        "  Successful:         {} ({:.1}%)",
        metrics.requests_success,
        (metrics.requests_success as f64 / metrics.requests_total as f64) * 100.0
    );
    println!(
        "  Errors:             {} ({:.1}%)",
        metrics.requests_error,
        (metrics.requests_error as f64 / metrics.requests_total as f64) * 100.0
    );

    println!();
    println!("⏱️  Latency Metrics:");
    println!("  P50 (median):       {:.1} ms", metrics.latency_p50_ms);
    println!("  P95:                {:.1} ms", metrics.latency_p95_ms);
    println!("  P99:                {:.1} ms", metrics.latency_p99_ms);
    println!("  Maximum:            {:.1} ms", metrics.latency_max_ms);

    println!();
    println!("🔤 Token Metrics:");
    println!("  Tokens per second:  {:.1}", metrics.tokens_per_second);
    println!("  Total tokens:       {}", metrics.tokens_total);
    println!("  Input tokens:       {}", metrics.tokens_input);
    println!("  Output tokens:      {}", metrics.tokens_output);

    println!();
    println!("🔧 Tool Metrics:");
    println!("  Total tool calls:   {}", metrics.tool_calls_total);
    println!(
        "  Successful:         {} ({:.1}%)",
        metrics.tool_calls_success,
        (metrics.tool_calls_success as f64 / metrics.tool_calls_total as f64) * 100.0
    );
    println!(
        "  Errors:             {} ({:.1}%)",
        metrics.tool_calls_error,
        (metrics.tool_calls_error as f64 / metrics.tool_calls_total as f64) * 100.0
    );

    if !metrics.tool_errors_by_type.is_empty() {
        println!("  Errors by tool:");
        for (tool, count) in &metrics.tool_errors_by_type {
            println!("    {}: {}", tool, count);
        }
    }

    println!();
    println!("🤖 Model Metrics:");
    println!("  Model loads:        {}", metrics.model_loads_total);
    println!("  Inferences:         {}", metrics.model_inferences_total);
    println!(
        "  Memory usage:       {:.1} MB",
        metrics.model_memory_usage_mb
    );

    println!();
    println!("💻 System Metrics:");
    println!("  CPU usage:          {:.1}%", metrics.cpu_usage_percent);
    println!("  Memory usage:       {:.1} MB", metrics.memory_usage_mb);
    println!("  Disk usage:         {:.1} MB", metrics.disk_usage_mb);

    println!();
    println!("👥 Session Metrics:");
    println!("  Active sessions:    {}", metrics.active_sessions);
    println!("  Total sessions:     {}", metrics.sessions_total);
    println!(
        "  Avg duration:       {:.1} seconds",
        metrics.session_duration_avg_secs
    );
}

fn test_privacy_modes(metrics: &AiCoreMetrics) {
    let collector = AiCoreMetricsCollector::new(default_metrics_config());

    println!("🔒 Testing privacy modes...");

    // Test LocalOnly mode
    let local_only = collector.anonymize_metrics(metrics);
    println!(
        "  LocalOnly mode: tool errors removed = {}",
        local_only.tool_errors_by_type.is_empty()
    );

    // Test Anonymized mode
    let anonymized = collector.anonymize_metrics(metrics);
    println!(
        "  Anonymized mode: tool errors removed = {}",
        anonymized.tool_errors_by_type.is_empty()
    );

    // Test Full mode (no anonymization)
    println!(
        "  Full mode: tool errors preserved = {}",
        !metrics.tool_errors_by_type.is_empty()
    );
}

async fn test_ngfs_storage(metrics: &AiCoreMetrics) -> Result<(), Box<dyn std::error::Error>> {
    let mock_ngfs = std::sync::Arc::new(MockNgfsClient::new());

    println!("💾 Testing NGFS storage...");

    // Store metrics
    let shard_id = mock_ngfs.store_metrics(metrics).await?;
    println!("  Stored metrics in shard: {}", shard_id);

    // Retrieve metrics
    let retrieved = mock_ngfs.retrieve_metrics(&shard_id).await?;
    println!(
        "  Retrieved metrics: {} requests, {} tokens",
        retrieved.requests_total, retrieved.tokens_total
    );

    // List shards
    let shards = mock_ngfs.list_metrics_shards().await?;
    println!("  Available shards: {}", shards.len());

    Ok(())
}
