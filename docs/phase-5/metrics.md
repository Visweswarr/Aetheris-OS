# AI Core Service Telemetry and Metrics

## Overview

The AI Core Service includes comprehensive telemetry and metrics collection to monitor performance, track usage patterns, and identify issues. All metrics are collected locally by default with privacy controls to prevent data leakage.

## Features

- **Local Storage**: Metrics stored in NGFS shards for persistence and privacy
- **Privacy Controls**: Off by default for cloud export with multiple privacy modes
- **Comprehensive Metrics**: Latency percentiles, token throughput, tool errors, system metrics
- **CLI Access**: Easy command-line interface for viewing and managing metrics
- **Real-time Collection**: Continuous metrics collection with configurable intervals

## Metrics Collected

### Request Metrics
- **Total Requests**: Count of all requests processed
- **Success Rate**: Percentage of successful requests
- **Error Rate**: Percentage of failed requests

### Latency Metrics
- **P50 (Median)**: 50th percentile latency
- **P95**: 95th percentile latency  
- **P99**: 99th percentile latency
- **Maximum**: Highest observed latency

### Token Metrics
- **Tokens per Second**: Average token processing rate
- **Total Tokens**: Cumulative token count
- **Input Tokens**: Tokens in user requests
- **Output Tokens**: Tokens in AI responses

### Tool Metrics
- **Tool Calls Total**: Count of all tool invocations
- **Tool Success Rate**: Percentage of successful tool calls
- **Tool Errors by Type**: Breakdown of errors by tool name

### Model Metrics
- **Model Loads**: Number of model loading operations
- **Inferences**: Count of model inference operations
- **Memory Usage**: Memory consumed by models

### System Metrics
- **CPU Usage**: System CPU utilization percentage
- **Memory Usage**: System memory consumption
- **Disk Usage**: Disk space utilization

### Session Metrics
- **Active Sessions**: Currently active user sessions
- **Total Sessions**: Cumulative session count
- **Average Duration**: Mean session duration

## Privacy Controls

### Privacy Modes

1. **Disabled**: No metrics collection
2. **Local Only**: Local collection, no cloud export (default)
3. **Anonymized**: Local collection with anonymized cloud export
4. **Full**: Complete collection with explicit consent

### Data Protection

- **Local Storage**: All metrics stored locally in NGFS shards
- **No Cloud Export by Default**: Cloud export disabled unless explicitly enabled
- **Anonymization**: Sensitive data removed in anonymized mode
- **Retention Limits**: Automatic cleanup of old metrics data

## Configuration

### Default Configuration

```rust
MetricsConfig {
    enabled: true,
    collection_interval_secs: 60,
    retention_days: 30,
    enable_ngfs: true,
    ngfs_shard_path: "~/.aetheris/metrics/ai_core",
    enable_cloud_export: false, // Off by default
    cloud_export_endpoint: None,
    privacy_mode: PrivacyMode::LocalOnly,
    max_samples_per_metric: 10000,
}
```

### Environment Variables

- `AETHERIS_METRICS_ENABLED`: Enable/disable metrics collection
- `AETHERIS_METRICS_PRIVACY_MODE`: Set privacy mode
- `AETHERIS_METRICS_CLOUD_EXPORT`: Enable cloud export
- `AETHERIS_METRICS_COLLECTION_INTERVAL`: Collection interval in seconds

## CLI Commands

### View Metrics

```bash
# Show current metrics in table format
devctl ai metrics show

# Show metrics in JSON format
devctl ai metrics show --format json

# Include historical data
devctl ai metrics show --include-history

# Filter by time range
devctl ai metrics show --time-range 24h

# Filter by metrics type
devctl ai metrics show --type latency
```

### Export Metrics

```bash
# Export to JSON file
devctl ai metrics export metrics.json --format json

# Export to CSV
devctl ai metrics export metrics.csv --format csv

# Export to Prometheus format
devctl ai metrics export metrics.prom --format prometheus

# Include historical data
devctl ai metrics export metrics.json --include-history --time-range 7d
```

### Configure Metrics

```bash
# Enable metrics collection
devctl ai metrics config --enable

# Disable metrics collection
devctl ai metrics config --disable

# Set privacy mode
devctl ai metrics config --privacy-mode local-only

# Set collection interval
devctl ai metrics config --collection-interval 30

# Enable cloud export
devctl ai metrics config --enable-cloud-export

# Disable cloud export
devctl ai metrics config --disable-cloud-export
```

### Reset Metrics

```bash
# Reset all metrics (with confirmation)
devctl ai metrics reset

# Reset without confirmation
devctl ai metrics reset --confirm
```

## API Usage

### Rust API

```rust
use aetheris_ai_core::metrics::*;

// Initialize metrics collector
let config = default_metrics_config();
let mut collector = AiCoreMetricsCollector::new(config);
collector.start().await?;

// Record metrics
record_request(true, 150.0); // success=true, latency=150ms
record_tokens(100, 50); // input=100, output=50
record_tool_call(true, Some("open_file"));
record_model_load();
record_model_inference();
record_session();

// Get metrics snapshot
let metrics = collector.get_metrics_snapshot().await?;
println!("Requests: {}", metrics.requests_total);
println!("Latency P95: {}ms", metrics.latency_p95_ms);
```

### Global Metrics Functions

```rust
// Record request metrics
record_request(success: bool, latency_ms: f64);

// Record token usage
record_tokens(input_tokens: u64, output_tokens: u64);

// Record tool call
record_tool_call(success: bool, tool_name: Option<&str>);

// Record model operations
record_model_load();
record_model_inference();

// Record session
record_session();
```

## Storage Format

### NGFS Shard Structure

```
~/.aetheris/metrics/ai_core/
├── shards/
│   ├── 2024-01-15-10-00-00.cbor
│   ├── 2024-01-15-11-00-00.cbor
│   └── 2024-01-15-12-00-00.cbor
├── index.json
└── config.cbor
```

### Metrics Data Format

```json
{
  "requests_total": 1250,
  "requests_success": 1180,
  "requests_error": 70,
  "latency_p50_ms": 45.2,
  "latency_p95_ms": 125.8,
  "latency_p99_ms": 250.3,
  "latency_max_ms": 500.1,
  "tokens_per_second": 12.5,
  "tokens_total": 15680,
  "tokens_input": 8920,
  "tokens_output": 6760,
  "tool_calls_total": 340,
  "tool_calls_success": 315,
  "tool_calls_error": 25,
  "tool_errors_by_type": {
    "open_file": 8,
    "search_files": 12,
    "create_note": 3,
    "unknown_tool": 2
  },
  "model_loads_total": 15,
  "model_inferences_total": 1180,
  "model_memory_usage_mb": 2048.5,
  "cpu_usage_percent": 23.4,
  "memory_usage_mb": 1024.8,
  "disk_usage_mb": 5120.2,
  "active_sessions": 3,
  "sessions_total": 45,
  "session_duration_avg_secs": 180.5,
  "timestamp": 1705320000
}
```

## Integration

### AI Core Service Integration

The metrics system is automatically integrated into the AI Core Service:

1. **IPC Handler**: Records request latency and success/failure
2. **Chat Handler**: Records token usage and model inference
3. **Tool Handler**: Records tool call success/failure and errors
4. **Session Manager**: Records session creation and duration

### Automatic Collection

Metrics are collected automatically during normal AI Core Service operation:

- Request processing latency
- Token generation and consumption
- Tool execution success/failure
- Model loading and inference
- System resource usage
- Session lifecycle events

## Monitoring and Alerting

### Key Metrics to Monitor

1. **Latency P95**: Should be under 200ms for good user experience
2. **Error Rate**: Should be under 5% for production systems
3. **Token Throughput**: Monitor for performance degradation
4. **Tool Error Rate**: Identify problematic tools
5. **Memory Usage**: Prevent memory leaks
6. **CPU Usage**: Monitor for resource constraints

### Alerting Thresholds

```yaml
alerts:
  high_latency:
    condition: latency_p95_ms > 500
    severity: warning
    
  high_error_rate:
    condition: requests_error / requests_total > 0.1
    severity: critical
    
  low_token_throughput:
    condition: tokens_per_second < 5
    severity: warning
    
  high_memory_usage:
    condition: memory_usage_mb > 8192
    severity: warning
```

## Troubleshooting

### Common Issues

1. **Metrics Not Collecting**
   - Check if metrics are enabled: `devctl ai metrics config`
   - Verify NGFS storage is accessible
   - Check collection interval settings

2. **High Latency**
   - Monitor P95 latency trends
   - Check system resource usage
   - Review tool execution times

3. **High Error Rate**
   - Check tool error breakdown
   - Review capability token validation
   - Monitor system resource constraints

4. **Storage Issues**
   - Check NGFS shard directory permissions
   - Monitor disk space usage
   - Review retention policy settings

### Debug Commands

```bash
# Check metrics configuration
devctl ai metrics config

# View detailed metrics
devctl ai metrics show --format json

# Export metrics for analysis
devctl ai metrics export debug.json --include-history

# Reset metrics if corrupted
devctl ai metrics reset --confirm
```

## Security Considerations

### Data Privacy

- **Local Storage Only**: Default configuration stores data locally
- **No Cloud Export**: Cloud export disabled by default
- **Anonymization**: Sensitive data removed in anonymized mode
- **Retention Limits**: Automatic cleanup prevents data accumulation

### Access Control

- **File Permissions**: NGFS shards protected by file system permissions
- **Capability Tokens**: Metrics access controlled by capability system
- **Audit Logging**: All metrics operations logged for security

### Compliance

- **GDPR Compliance**: Privacy controls support data protection requirements
- **Data Minimization**: Only necessary metrics collected
- **User Consent**: Explicit consent required for full data collection
- **Right to Deletion**: Metrics can be reset/deleted on demand

## Performance Impact

### Overhead

- **Minimal CPU Impact**: <1% CPU overhead for metrics collection
- **Low Memory Usage**: <10MB memory for metrics storage
- **Efficient Storage**: CBOR format for compact storage
- **Async Collection**: Non-blocking metrics collection

### Optimization

- **Sampling**: Configurable sampling rates for high-volume scenarios
- **Batch Processing**: Metrics collected in batches to reduce overhead
- **Lazy Evaluation**: Expensive calculations performed only when needed
- **Cleanup**: Automatic cleanup of old data to prevent storage bloat

## Future Enhancements

### Planned Features

1. **Real-time Dashboards**: Web-based metrics visualization
2. **Advanced Analytics**: Machine learning for anomaly detection
3. **Custom Metrics**: User-defined metrics collection
4. **Distributed Metrics**: Multi-node metrics aggregation
5. **Integration**: Prometheus/Grafana integration
6. **Alerting**: Built-in alerting system

### API Extensions

1. **Custom Collectors**: Plugin system for custom metrics
2. **Metrics Queries**: SQL-like query language for metrics
3. **Aggregation**: Time-series aggregation functions
4. **Export Formats**: Additional export formats (InfluxDB, etc.)

## Conclusion

The AI Core Service telemetry system provides comprehensive monitoring capabilities while maintaining strong privacy controls. The system is designed to be lightweight, secure, and easy to use, providing valuable insights into AI Core Service performance and usage patterns.

Key benefits:

- **Privacy-First**: Local storage with optional anonymized cloud export
- **Comprehensive**: Covers all aspects of AI Core Service operation
- **Easy to Use**: Simple CLI commands for all operations
- **Secure**: Built-in privacy controls and access restrictions
- **Performant**: Minimal overhead with efficient storage
- **Extensible**: Plugin system for custom metrics and exporters
