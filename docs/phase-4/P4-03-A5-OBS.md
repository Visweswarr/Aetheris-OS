# Phase 4.3 - Advanced Networking Observability & Chaos Engineering

## Overview

This document describes the **Observability & Chaos Engineering** implementation for Aetheris OS networking, providing comprehensive end-to-end tracing, structured metrics collection, and chaos injection capabilities for testing network resilience.

## Architecture

### Core Components

1. **End-to-End Tracing** (`services/net/posixnet/src/trace.rs`)
   - OpenTelemetry-compatible tracing across all networking layers
   - Cross-layer correlation from syscalls through Rust broker to Go CLI and TypeScript bridge
   - Span-based distributed tracing with baggage propagation

2. **Structured Metrics** (`services/net/posixnet/src/metrics.rs`)
   - Comprehensive metrics collection (p50/p95/p99 latency, throughput, CPU, RSS, error counts)
   - Prometheus-compatible export format
   - JSON export for analysis and reporting

3. **Chaos Engineering** (`services/net/posixnet/src/chaos.rs`)
   - Random packet loss injection
   - Latency and jitter injection
   - Connection churn (rapid open/close cycles)
   - Deterministic results with seeded RNG

4. **Go CLI Extensions**
   - `go/tooling/netctl/chaos.go` - Chaos injection commands
   - `go/tooling/netctl/trace.go` - Tracing commands
   - `go/tooling/netctl/metrics.go` - Metrics collection and export

5. **TypeScript Bridge** (`tooling/ts/net-chaos.ts`)
   - Node.js chaos injection and metrics collection
   - Prometheus metrics server
   - JSON export capabilities

6. **Python Validator** (`tooling/python/net_obs_validator.py`)
   - Metrics and chaos validation
   - Error budget validation
   - Latency regression detection

## Features

### End-to-End Tracing

- **OpenTelemetry Compatibility**: Full OpenTelemetry JSON format support
- **Cross-Layer Correlation**: Trace requests from syscalls through all networking layers
- **Span Management**: Automatic span creation, attribute addition, and event recording
- **Baggage Propagation**: Context propagation across service boundaries
- **Export Formats**: JSON and OpenTelemetry formats

### Structured Metrics

- **Comprehensive Collection**: RPS, latency percentiles, throughput, CPU, memory, errors
- **Multiple Export Formats**: Prometheus text format and JSON
- **Real-time Monitoring**: HTTP endpoints for live metrics access
- **Statistical Analysis**: Histograms, summaries, and quantiles
- **System Integration**: CPU, memory, and GC statistics

### Chaos Engineering

- **Packet Loss Injection**: Configurable packet loss rates (0-100%)
- **Latency Injection**: Base latency with configurable jitter
- **Connection Churn**: Rapid connection open/close cycles
- **Deterministic Results**: Seeded RNG for reproducible tests
- **Resource Stress**: CPU and memory stress injection
- **Bandwidth Limiting**: Configurable bandwidth constraints

## Usage

### Command Line Interface

#### Chaos Engineering

```bash
# Basic chaos injection
./netctl chaos inject --loss 5% --latency 100ms --duration 10m

# Advanced chaos injection
./netctl chaos inject --loss 0.05 --latency 100 --jitter 20 --churn 10 --cpu 50 --memory 100

# Stop chaos injections
./netctl chaos stop

# View chaos statistics
./netctl chaos stats
```

#### Tracing

```bash
# Start trace collection
./netctl trace start --output traces/ --sample-rate 1.0

# Stop trace collection
./netctl trace stop

# Export traces
./netctl trace export --output traces/exported.json --format otel
```

#### Metrics

```bash
# Start metrics server
./netctl metrics serve --port 9090 --json metrics/network_metrics.json

# Collect metrics for duration
./netctl metrics collect --duration 5m --interval 1s --output metrics/collected.json

# Export metrics
./netctl metrics export --output metrics/exported.json --format prometheus
```

### TypeScript/Node.js

```bash
# Run chaos injection
cd tooling/ts
node net-chaos.js chaos --loss 0.05 --latency 100 --duration 60000

# Start metrics server
node net-chaos.js metrics --port 9090 --json metrics.json
```

### Python Validation

```bash
# Validate metrics file
python tooling/python/net_obs_validator.py validate-metrics metrics/network_metrics.json

# Validate Prometheus endpoint
python tooling/python/net_obs_validator.py validate-prometheus --url http://localhost:9090

# Validate chaos results
python tooling/python/net_obs_validator.py validate-chaos chaos_results.json

# Validate error budget
python tooling/python/net_obs_validator.py validate-error-budget \
  current_metrics.json baseline_metrics.json --threshold 0.01

# Validate latency regression
python tooling/python/net_obs_validator.py validate-latency-regression \
  current_metrics.json baseline_metrics.json --threshold 0.2

# Run chaos test
python tooling/python/net_obs_validator.py run-chaos-test \
  --loss 0.02 --latency 100 --duration 120000
```

## Configuration

### Tracing Configuration

```rust
let config = TraceConfig {
    enabled: true,
    sample_rate: 1.0,                    // 0.0 to 1.0
    max_spans: 10000,                    // Maximum spans to keep
    export_interval: Duration::from_secs(30),
    output_file: Some("traces/network_traces.json".to_string()),
    service_name: "aetheris-net".to_string(),
    service_version: "1.0.0".to_string(),
};
```

### Metrics Configuration

```rust
let config = MetricsConfig {
    enabled: true,
    export_interval: Duration::from_secs(15),
    prometheus_port: Some(9090),
    json_output: Some("metrics/network_metrics.json".to_string()),
    service_name: "aetheris-net".to_string(),
    service_version: "1.0.0".to_string(),
};
```

### Chaos Configuration

```rust
let config = ChaosConfig {
    enabled: true,
    seed: Some(42),                      // For deterministic results
    duration: Duration::from_secs(60),
    packet_loss_rate: 0.05,              // 5% packet loss
    latency_ms: 100.0,                   // 100ms base latency
    latency_jitter_ms: 20.0,             // ±20ms jitter
    connection_churn_rate: 10.0,         // 10 connections/sec
    connection_churn_duration: Duration::from_secs(1),
    bandwidth_limit_mbps: Some(100.0),   // 100 Mbps limit
    cpu_stress_percent: Some(50.0),      // 50% CPU stress
    memory_stress_mb: Some(100.0),       // 100 MB memory stress
};
```

## Observability Data Formats

### OpenTelemetry Trace Format

```json
{
  "traceId": "12345678-1234-1234-1234-123456789abc",
  "spanId": "87654321-4321-4321-4321-cba987654321",
  "parentSpanId": "11111111-1111-1111-1111-111111111111",
  "name": "network_operation",
  "kind": 2,
  "startTimeUnixNano": "1640995200000000000",
  "endTimeUnixNano": "1640995200100000000",
  "durationNano": "100000000",
  "status": {
    "code": 1,
    "message": null
  },
  "attributes": [
    {
      "key": "operation",
      "value": {
        "type": "string",
        "value": "tls_handshake"
      }
    }
  ],
  "events": [
    {
      "name": "handshake_started",
      "timeUnixNano": "1640995200000000000",
      "attributes": []
    }
  ],
  "links": []
}
```

### Prometheus Metrics Format

```
# HELP network_requests_total Total number of network requests
# TYPE network_requests_total counter
network_requests_total 1000

# HELP network_latency_ms Network latency in milliseconds
# TYPE network_latency_ms histogram
network_latency_ms_bucket{le="0.001"} 0
network_latency_ms_bucket{le="0.005"} 0
network_latency_ms_bucket{le="0.01"} 0
network_latency_ms_bucket{le="0.05"} 0
network_latency_ms_bucket{le="0.1"} 0
network_latency_ms_bucket{le="0.5"} 0
network_latency_ms_bucket{le="1.0"} 0
network_latency_ms_bucket{le="5.0"} 0
network_latency_ms_bucket{le="10.0"} 0
network_latency_ms_bucket{le="50.0"} 0
network_latency_ms_bucket{le="100.0"} 0
network_latency_ms_bucket{le="+Inf"} 1000
network_latency_ms_count 1000
network_latency_ms_sum 5000.0

# HELP network_cpu_usage_percent CPU usage percentage
# TYPE network_cpu_usage_percent gauge
network_cpu_usage_percent 25.5
```

### Chaos Results Format

```json
{
  "chaos_type": "packet_loss",
  "applied": true,
  "duration": 60000,
  "packets_dropped": 3000,
  "packets_delayed": 0,
  "connections_churned": 0,
  "error_count": 150
}
```

## CI Integration

### Workflow Configuration

The CI workflow (`.github/workflows/p4-03-net-obs.yml`) includes:

1. **Observability Tests**: Metrics, tracing, and chaos validation
2. **Chaos Mini-Tests**: Fast chaos tests for CI (loss 2%, 2m duration)
3. **Error Budget Validation**: Fail if errors >1% or RTT > baseline+20%
4. **Metrics & Tracing Integration**: End-to-end observability testing
5. **TypeScript Observability**: Node.js chaos and metrics testing

### Error Budget and Regression Detection

- **Error Budget**: Fail if error rate exceeds 1% threshold
- **Latency Regression**: Fail if p95 RTT exceeds baseline+20%
- **Chaos Validation**: Ensure chaos injections are working correctly
- **Artifact Collection**: Export traces, metrics, and chaos results

### Artifact Collection

- **Traces**: OpenTelemetry JSON format
- **Metrics**: Prometheus text format and JSON
- **Chaos Results**: JSON with injection statistics
- **Validation Reports**: Human-readable validation results

## Monitoring and Alerting

### Key Metrics

- **Request Rate**: Requests per second (RPS)
- **Latency Percentiles**: P50, P95, P99 latency
- **Error Rate**: Percentage of failed requests
- **System Resources**: CPU, memory, RSS usage
- **Chaos Statistics**: Packets dropped, connections churned

### Alerting Thresholds

- **Error Rate**: > 1% triggers alert
- **Latency P95**: > baseline + 20% triggers alert
- **CPU Usage**: > 80% triggers alert
- **Memory Usage**: > 90% triggers alert
- **Chaos Failures**: Any chaos injection failure triggers alert

### Dashboard Integration

```bash
# Prometheus metrics endpoint
curl http://localhost:9090/metrics

# JSON metrics export
curl http://localhost:9090/api/v1/metrics

# Trace export
curl http://localhost:9090/api/v1/traces
```

## Chaos Engineering Best Practices

### Test Design

1. **Start Small**: Begin with low-impact chaos (1% packet loss)
2. **Gradual Increase**: Gradually increase chaos intensity
3. **Baseline Comparison**: Always compare against baseline metrics
4. **Deterministic Results**: Use seeded RNG for reproducible tests
5. **Error Budget**: Set clear error budget thresholds

### Chaos Types

1. **Packet Loss**: Test network resilience to dropped packets
2. **Latency**: Test application behavior under high latency
3. **Jitter**: Test stability under variable latency
4. **Connection Churn**: Test connection pool management
5. **Resource Stress**: Test under CPU/memory pressure

### Safety Measures

1. **Time Limits**: Always set maximum chaos duration
2. **Error Thresholds**: Stop chaos if error rate exceeds threshold
3. **Rollback Plans**: Have immediate rollback procedures
4. **Monitoring**: Continuous monitoring during chaos tests
5. **Gradual Rollout**: Start with non-production environments

## Troubleshooting

### Common Issues

1. **High Memory Usage**
   - Reduce trace sample rate
   - Limit maximum spans
   - Increase export frequency

2. **Chaos Not Working**
   - Check chaos configuration
   - Verify RNG seed
   - Validate injection parameters

3. **Metrics Not Exporting**
   - Check Prometheus endpoint
   - Verify JSON file permissions
   - Validate metrics format

4. **Tracing Overhead**
   - Reduce sample rate
   - Filter low-value spans
   - Optimize attribute collection

### Debugging

```bash
# Enable debug logging
RUST_LOG=debug ./netctl chaos inject --loss 0.01 --duration 30s

# Check metrics endpoint
curl -v http://localhost:9090/metrics

# Validate trace format
python tooling/python/net_obs_validator.py validate-traces traces/network_traces.json

# Check chaos statistics
./netctl chaos stats
```

## Performance Considerations

### Tracing Overhead

- **Sample Rate**: 1.0 = 100% overhead, 0.1 = 10% overhead
- **Span Limits**: Keep max spans reasonable (10k-100k)
- **Export Frequency**: Balance between real-time and performance
- **Attribute Limits**: Limit attributes per span

### Metrics Overhead

- **Collection Interval**: 15s default, adjust based on needs
- **Histogram Buckets**: Limit bucket count for performance
- **Export Format**: Prometheus text is faster than JSON
- **Memory Usage**: Monitor metrics memory consumption

### Chaos Overhead

- **Injection Frequency**: Higher frequency = higher overhead
- **Deterministic RNG**: Seeded RNG has minimal overhead
- **Resource Stress**: CPU/memory stress affects performance
- **Network Impact**: Packet loss affects throughput

## Future Enhancements

### Planned Features

1. **Distributed Tracing**: Multi-node trace correlation
2. **Real-time Dashboards**: Web-based observability UI
3. **Machine Learning**: Automated anomaly detection
4. **Advanced Chaos**: Network partition simulation
5. **Cloud Integration**: Cloud-native observability

### Performance Improvements

1. **Zero-Copy Tracing**: Reduce memory allocations
2. **Async Export**: Non-blocking trace/metrics export
3. **Compression**: Compress trace data for storage
4. **Sampling**: Intelligent sampling based on load
5. **Caching**: Cache frequently accessed metrics

## References

- [OpenTelemetry Specification](https://opentelemetry.io/docs/specs/)
- [Prometheus Metrics](https://prometheus.io/docs/concepts/metric_types/)
- [Chaos Engineering Principles](https://principlesofchaos.org/)
- [Distributed Tracing Best Practices](https://opentelemetry.io/docs/best-practices/)
- [Observability Patterns](https://www.oreilly.com/library/view/distributed-systems-observability/9781492033431/)
