# Polymera OS Performance Harnesses

Performance testing harnesses that generate timing data and export metrics to OpenTelemetry/Prometheus for SLO validation and monitoring.

## 🎯 Overview

The harness suite provides three specialized performance testing tools:

- **XR Harness** (`xr.rs`): Extended Reality frame timing and Mean Time to Present (MTP) simulation
- **Network Harness** (`net.rs`): Mesh network RTT generation and synchronization simulation  
- **Wallet Harness** (`wallet.rs`): Wallet operations including anonymous authentication confirmation

All harnesses export metrics via OpenTelemetry to support automated SLO validation and performance monitoring.

## 📁 Directory Structure

```
perf/harness/
├── README.md                        # This documentation
├── Cargo.toml                       # Rust dependencies for all harnesses
├── BUILD                            # Bazel build configuration
├── xr.rs                           # XR timing harness
├── net.rs                          # Network RTT harness  
├── wallet.rs                       # Wallet confirmation harness
├── test_xr_metrics.sh              # XR metrics validation test
├── test_all_harnesses.sh           # Integration test for all harnesses
├── test_prometheus_integration.sh  # Prometheus integration test
└── docker-compose.yml              # Prometheus/OTLP stack for testing
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+
- OpenTelemetry Collector (optional)
- Prometheus (optional, for metrics collection)

### Building the Harnesses

```bash
# Using Cargo
cd perf/harness
cargo build --release

# Using Bazel
bazel build //perf/harness:xr_harness
bazel build //perf/harness:network_harness  
bazel build //perf/harness:wallet_harness
```

### Running Individual Harnesses

```bash
# XR harness with 90 FPS for 5 minutes
./target/release/xr_harness --fps 90 --duration 300

# Network harness with 20 peers for 5 minutes
./target/release/network_harness --peers 20 --duration 300

# Wallet harness with privacy overhead enabled
./target/release/wallet_harness --duration 300 --privacy-overhead
```

### Running All Harnesses (Integration Test)

```bash
# Test all harnesses with metrics validation
./test_all_harnesses.sh

# Test Prometheus integration
./test_prometheus_integration.sh
```

## 🔧 XR Harness (xr.rs)

### Purpose
Simulates Extended Reality (XR) frame rendering pipeline to generate Mean Time to Present (MTP) latency metrics for SLO validation.

### Key Metrics
- `xr_mtp_latency`: Mean Time to Present latency (target: <20ms p95)
- `xr_frame_render_duration`: Individual frame rendering time
- `xr_frames_total`: Total frames rendered counter
- `xr_frames_dropped_total`: Frame drop counter

### Configuration Options

```bash
./xr_harness \
    --fps 90 \                    # Target frame rate
    --duration 300 \              # Run duration in seconds
    --base-latency 8.0 \          # Base frame latency (ms)
    --variance 3.0 \              # Latency variance (ms)
    --degradation                 # Enable performance degradation simulation
```

### Configuration File (YAML)
```yaml
target_fps: 90
base_latency_ms: 8.0
latency_variance_ms: 3.0
frame_drop_probability: 0.001
duration_seconds: 300
enable_degradation: true
degradation_cycle_seconds: 120
```

### Usage Examples

```bash
# Standard XR performance testing
./xr_harness --fps 90 --duration 300 --base-latency 8.0

# High-performance mode (120 FPS)
./xr_harness --fps 120 --duration 300 --base-latency 6.0

# Stress testing with degradation
./xr_harness --fps 90 --duration 300 --degradation

# Custom configuration
./xr_harness --config xr_custom_config.yaml
```

## 🌐 Network Harness (net.rs)

### Purpose
Simulates mesh network operations including peer discovery, synchronization, and RTT measurement for distributed networking SLOs.

### Key Metrics
- `mesh_sync_latency`: Network synchronization latency (target: <60s p95)
- `peer_discovery_latency`: Peer discovery operation latency
- `mesh_rtt`: Round-trip time between peers
- `mesh_packets_lost_total`: Packet loss counter
- `mesh_peer_count`: Active peer count

### Configuration Options

```bash
./network_harness \
    --peers 20 \                  # Number of simulated peers
    --duration 300 \              # Run duration in seconds
    --base-rtt 25.0 \             # Base RTT (ms)
    --variance 15.0 \             # RTT variance (ms)
    --degradation                 # Enable network degradation
```

### Configuration File (YAML)
```yaml
peer_count: 20
base_rtt_ms: 25.0
rtt_variance_ms: 15.0
packet_loss_probability: 0.002
sync_interval_seconds: 30
discovery_interval_seconds: 60
duration_seconds: 300
enable_degradation: false
enable_partitioning: false
```

### Usage Examples

```bash
# Standard mesh network testing
./network_harness --peers 20 --duration 300

# Large-scale network simulation
./network_harness --peers 50 --duration 600

# Network degradation testing
./network_harness --peers 15 --degradation --partitioning

# Custom network conditions
./network_harness --base-rtt 50.0 --variance 20.0
```

## 💰 Wallet Harness (wallet.rs)

### Purpose
Simulates wallet operations including anonymous authentication confirmation, session key validation, and keystore operations for privacy and security SLOs.

### Key Metrics
- `auth_anonymous_confirm_latency`: Anonymous auth confirmation (target: <3s p95)
- `session_validation_latency`: Session key validation latency (target: <25ms p95)
- `keystore_operations_latency`: Keystore operation latency (target: <50ms p95)
- `privacy_budget_processing_latency`: Privacy budget processing time
- `wallet_operations_total`: Total operations counter
- `wallet_errors_total`: Error counter

### Configuration Options

```bash
./wallet_harness \
    --duration 300 \              # Run duration in seconds
    --auth-latency 1800.0 \       # Base anonymous auth latency (ms)
    --session-latency 15.0 \      # Base session validation latency (ms)
    --keystore-latency 30.0 \     # Base keystore operation latency (ms)
    --privacy-overhead \          # Enable privacy computation overhead
    --crypto-delays               # Enable cryptographic processing delays
```

### Configuration File (YAML)
```yaml
base_auth_latency_ms: 1800.0
auth_latency_variance_ms: 600.0
base_session_latency_ms: 15.0
session_latency_variance_ms: 8.0
base_keystore_latency_ms: 30.0
keystore_latency_variance_ms: 15.0
auth_interval_seconds: 45
session_interval_seconds: 5
keystore_interval_seconds: 10
duration_seconds: 300
enable_privacy_overhead: true
enable_crypto_delays: true
operation_success_rate: 0.999
```

### Usage Examples

```bash
# Standard wallet operation testing
./wallet_harness --duration 300

# High-privacy mode testing
./wallet_harness --duration 300 --privacy-overhead --crypto-delays

# Fast session validation testing
./wallet_harness --session-latency 10.0 --duration 180

# Custom operation intervals
./wallet_harness --auth-latency 2000.0 --keystore-latency 40.0
```

## 📊 OpenTelemetry Integration

### Supported Exporters

All harnesses support multiple OpenTelemetry exporters:

- **OTLP HTTP**: Default exporter to `http://localhost:4318/v1/metrics`
- **Prometheus**: Direct Prometheus metrics export (feature flag)
- **Console**: Console output for debugging

### Metric Labels and Attributes

Each harness adds relevant labels to metrics:

**XR Harness:**
```
xr_mtp_latency{display_mode="stereo",resolution="2160x1200"}
xr_frame_render_duration{frame_type="normal"}
```

**Network Harness:**
```
mesh_sync_latency{sync_type="full",peer_count="5"}
mesh_rtt{message_type="data",protocol="quic"}
```

**Wallet Harness:**
```
auth_anonymous_confirm_latency{auth_type="anonymous",privacy_level="maximum"}
session_validation_latency{validation_type="full",constraint_checks="5"}
keystore_operations_latency{operation="sign",algorithm="dilithium"}
```

### OTLP Configuration

Set the OTLP endpoint via command line or environment:

```bash
# Command line
./xr_harness --otlp-endpoint http://otel-collector:4318/v1/metrics

# Environment variable
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
./xr_harness
```

## 📈 Prometheus Integration

### Metrics Collection

Start Prometheus and OTLP collector:

```bash
# Using Docker Compose
docker-compose up -d

# Verify services
curl http://localhost:9090/api/v1/label/__name__/values  # Prometheus
curl http://localhost:4318/v1/metrics                   # OTLP collector
```

### Sample Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'otel-collector'
    static_configs:
      - targets: ['localhost:8888']  # OTLP collector metrics endpoint
    
  - job_name: 'polymera-harnesses'
    static_configs:
      - targets: ['localhost:8080']  # Direct harness metrics (if enabled)
```

### Key Prometheus Queries

```promql
# XR MTP p95 latency
histogram_quantile(0.95, rate(xr_mtp_latency_bucket[5m]))

# Mesh sync p95 latency  
histogram_quantile(0.95, rate(mesh_sync_latency_bucket[5m]))

# Anonymous auth p95 latency
histogram_quantile(0.95, rate(auth_anonymous_confirm_latency_bucket[5m]))

# Session validation p95 latency
histogram_quantile(0.95, rate(session_validation_latency_bucket[5m]))

# Overall frame rate
rate(xr_frames_total[1m]) * 60

# Network packet loss rate
rate(mesh_packets_lost_total[5m]) / rate(mesh_packets_total[5m])
```

## 🧪 Testing

### Unit Tests

```bash
# Run unit tests for all harnesses
cargo test

# Run specific harness tests
cargo test --bin xr_harness
cargo test --bin network_harness
cargo test --bin wallet_harness
```

### Integration Tests

```bash
# Test individual harness metrics
./test_xr_metrics.sh

# Test all harnesses together
./test_all_harnesses.sh

# Test Prometheus integration
./test_prometheus_integration.sh
```

### Bazel Tests

```bash
# Run all harness tests
bazel test //perf/harness:harness_tests

# Test specific components
bazel test //perf/harness:test_xr_metrics
bazel test //perf/harness:test_all_harnesses
bazel test //perf/harness:test_prometheus_integration
```

### SLO Validation Testing

```bash
# Generate metrics and validate against SLOs
./test_all_harnesses.sh

# Use generated metrics with SLO checker
../check_slo \
    --config ../slo.yaml \
    --metrics integration_metrics.json \
    --env production \
    --fail-on-violation
```

## 🔧 Configuration Management

### Environment-Specific Configs

Create environment-specific configuration files:

```bash
# Development environment (relaxed)
cp xr_config.yaml xr_dev_config.yaml
# Edit to increase latency tolerances

# Production environment (strict)  
cp xr_config.yaml xr_prod_config.yaml
# Edit to match production performance targets

# Load testing (stress)
cp xr_config.yaml xr_stress_config.yaml
# Edit to enable degradation and higher loads
```

### Runtime Configuration

Override configuration at runtime:

```bash
# Override specific values
./xr_harness \
    --config xr_base_config.yaml \
    --fps 120 \
    --duration 600 \
    --degradation

# Environment-specific overrides
HARNESS_ENV=production ./wallet_harness --config wallet_config.yaml
```

## 📊 Performance Characteristics

### Resource Usage

| Harness | CPU Usage | Memory Usage | Network Usage |
|---------|-----------|--------------|---------------|
| XR      | 5-15%     | 10-50MB      | Minimal       |
| Network | 3-10%     | 5-30MB       | Low           |
| Wallet  | 2-8%      | 5-25MB       | Minimal       |

### Throughput Metrics

| Harness | Operations/sec | Metrics/sec | Data Rate     |
|---------|----------------|-------------|---------------|
| XR      | 90 frames/sec  | 300-500     | 10-50 KB/sec  |
| Network | 5-20 ops/sec   | 100-300     | 5-30 KB/sec   |
| Wallet  | 10-50 ops/sec  | 200-400     | 8-40 KB/sec   |

### Scaling Guidelines

- **Single harness**: Can run continuously for hours
- **Multiple harnesses**: Test concurrent execution for realistic load
- **Large-scale testing**: Use multiple instances with different configs
- **CI/CD integration**: Short duration tests (30-60s) for fast feedback

## 🔍 Troubleshooting

### Common Issues

#### High Latency Values
```bash
# Check if degradation is enabled
grep "enable_degradation" config.yaml

# Reduce variance for more consistent results
./harness --variance 1.0

# Check system load
top -p $(pgrep harness)
```

#### Missing Metrics
```bash
# Verify OTLP collector is running
curl http://localhost:4318/v1/metrics

# Check harness logs for export errors
./harness 2>&1 | grep -i "export\|error\|fail"

# Test with console export
RUST_LOG=debug ./harness
```

#### Performance Issues
```bash
# Reduce operation frequency
./harness --duration 60  # Shorter test duration

# Lower resource usage
./network_harness --peers 5  # Fewer simulated peers
./xr_harness --fps 30        # Lower frame rate
```

### Debug Mode

Enable detailed logging:

```bash
# Debug level logging
RUST_LOG=debug ./harness

# Trace level logging (very verbose)
RUST_LOG=trace ./harness

# Module-specific logging
RUST_LOG=xr_harness=debug,opentelemetry=info ./xr_harness
```

### Validation Commands

```bash
# Verify harness is exporting metrics
./harness --duration 30 2>&1 | grep -c "recorded\|exported"

# Check metric format
./harness --duration 10 2>&1 | grep "latency\|duration\|count"

# Validate SLO targets
grep -A 5 "p95" harness_output.log
```

## 🔄 CI/CD Integration

### GitHub Actions Integration

```yaml
# .github/workflows/performance-testing.yml
- name: Run performance harnesses
  run: |
    cd perf/harness
    ./test_all_harnesses.sh
    
- name: Validate SLOs
  run: |
    cd perf
    ./check_slo \
      --config slo.yaml \
      --metrics harness/integration_metrics.json \
      --env staging \
      --fail-on-violation
```

### Continuous Monitoring

```bash
# Scheduled harness runs for monitoring
0 */6 * * * /path/to/xr_harness --duration 300 --config production.yaml
15 */6 * * * /path/to/network_harness --duration 300 --config production.yaml  
30 */6 * * * /path/to/wallet_harness --duration 300 --config production.yaml
```

## 🤝 Contributing

### Adding New Harnesses

1. Create new harness file (e.g., `storage.rs`)
2. Implement OpenTelemetry metrics export
3. Add to `Cargo.toml` as new binary
4. Create corresponding test script
5. Update `BUILD` file with new targets
6. Add to integration test suite

### Modifying Existing Harnesses

1. Update harness implementation
2. Add/modify metrics as needed
3. Update configuration schema
4. Update tests to cover changes
5. Update documentation

### Code Style

- Follow Rust standard conventions
- Use `cargo fmt` for formatting
- Run `cargo clippy` for linting
- Add comprehensive error handling
- Include unit tests for new functionality

---

The performance harnesses provide comprehensive timing data generation for Polymera OS SLO validation, supporting both development and production monitoring workflows through OpenTelemetry and Prometheus integration.
