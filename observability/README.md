# Polymera OS Observability Module

Comprehensive observability and telemetry for Polymera OS services with built-in privacy protection and RED/USE metrics.

## 🚀 Features

- **OpenTelemetry Integration**: Distributed tracing and metrics collection
- **RED Metrics**: Rate, Errors, Duration monitoring
- **USE Metrics**: Utilization, Saturation, Errors tracking
- **Privacy Protection**: Built-in PII redaction and data sanitization
- **Performance Monitoring**: p95/p99 latency tracking
- **Prometheus Export**: Metrics ready for Grafana dashboards
- **Multi-language Support**: Rust, Go, Python, TypeScript

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
polymera-observability = { path = "../observability", features = ["full"] }
```

## 🔧 Quick Start

### Basic Initialization

```rust
use polymera_observability::{init_observability, OtelConfig, MetricsConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize observability
    let (otel_provider, metrics_registry) = init_observability(
        "my-service",
        "1.0.0",
        Some(OtelConfig::default()),
        Some(MetricsConfig::default()),
    ).await?;

    // Your service logic here...

    Ok(())
}
```

### Request Timing with Metrics

```rust
use polymera_observability::{create_request_timer, RequestTimer};

async fn handle_request(registry: &PrivacyAwareRegistry) {
    let timer = create_request_timer(registry, "my-service", "/api/users", "GET");
    
    // Your request handling logic...
    
    // Automatically records metrics on success
    timer.success();
    
    // Or on error
    // timer.error("validation_failed");
}
```

### Distributed Tracing

```rust
use polymera_observability::{get_tracer, create_span, KeyValue};

let tracer = get_tracer("user-service");
let span = create_span(&tracer, "process_user");
span.set_attribute(KeyValue::new("user.id", "12345"));
span.set_attribute(KeyValue::new("operation", "create"));

// Your business logic...

span.end();
```

### Privacy-Aware Metrics

```rust
use polymera_observability::{record_business_metric_safe, BusinessMetricType};

// Sensitive data is automatically redacted
record_business_metric_safe(
    registry,
    BusinessMetricType::ActiveUsers,
    &[("user_type", "premium"), ("region", "us-east")],
    42.0,
);
```

## 📊 Metrics

### RED Metrics (Rate, Errors, Duration)

- **Request Rate**: `polymera_requests_total`
- **Error Rate**: `polymera_errors_total`
- **Request Duration**: `polymera_request_duration_seconds`
- **P95 Duration**: `polymera_request_duration_p95_seconds`
- **P99 Duration**: `polymera_request_duration_p99_seconds`

### USE Metrics (Utilization, Saturation, Errors)

- **Resource Utilization**: `polymera_resource_utilization`
- **Resource Saturation**: `polymera_resource_saturation`
- **Resource Errors**: `polymera_resource_errors_total`
- **Resource Capacity**: `polymera_resource_capacity`

### Business Metrics

- **Active Users**: `polymera_active_users`
- **Transaction Volume**: `polymera_transactions_total`
- **Contract Deployments**: `polymera_contract_deployments_total`
- **Attestation Operations**: `polymera_attestations_total`

## 🔒 Privacy Protection

### Privacy Modes

- **None**: No privacy protection
- **Redact**: Replace sensitive values with `[REDACTED]`
- **Hash**: Hash sensitive values with MD5
- **Drop**: Remove sensitive metrics entirely

### Sensitive Keys

Automatically detected and protected:
- `user_id`, `email`, `address`
- `private_key`, `secret`, `token`
- `password`, `api_key`, `session_id`
- `ip_address`

### Custom Privacy Rules

```rust
let config = MetricsConfig {
    privacy_mode: PrivacyMode::Redact,
    // ... other config
};
```

## 🏗️ Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Your Service  │───▶│  OTEL Collector  │───▶│   Prometheus    │
│                 │    │                  │    │                 │
│  - Tracing     │    │  - Processing    │    │  - Storage      │
│  - Metrics     │    │  - Filtering     │    │  - Querying     │
│  - Privacy     │    │  - Export        │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌──────────────────┐
                       │     Grafana      │
                       │                  │
                       │  - Dashboards    │
                       │  - Alerts        │
                       │  - Visualization │
                       └──────────────────┘
```

## 🐳 Monitoring Stack

### Local Development

```bash
cd infra/grafana
./up.sh
```

### Services

- **Grafana**: http://localhost:3000 (admin/polymera123)
- **Prometheus**: http://localhost:9090
- **Jaeger**: http://localhost:16686
- **OTEL Collector**: http://localhost:4317 (gRPC), http://localhost:4318 (HTTP)

### Health Checks

```bash
# Check all services
curl http://localhost:3000/api/health    # Grafana
curl http://localhost:9090/-/healthy     # Prometheus
curl http://localhost:8888/              # OTEL Collector
curl http://localhost:16686/             # Jaeger
```

## 🧪 Testing

### Unit Tests

```bash
cargo test --features full
```

### Integration Tests

```bash
cargo test --features full --test integration
```

### End-to-End Testing

```bash
./test_observability.sh
```

### Performance Testing

```bash
cargo run --bin observability-bench --features full
```

## 📈 Dashboards

### Polymera OS Overview

- System overview with key metrics
- RED metrics visualization
- USE metrics tracking
- Business metrics display
- Real-time updates every 30 seconds

### Custom Dashboards

Create custom dashboards in Grafana using the provided metrics:

```promql
# Request rate by service
rate(polymera_requests_total[5m])

# Error rate by endpoint
rate(polymera_errors_total[5m])

# P95 latency by service
polymera_request_duration_p95_seconds

# Resource utilization
polymera_resource_utilization
```

## 🔧 Configuration

### OpenTelemetry Configuration

```rust
let otel_config = OtelConfig {
    service_name: "my-service".to_string(),
    service_version: "1.0.0".to_string(),
    service_namespace: "polymera-os".to_string(),
    otlp_endpoint: Some("http://localhost:4317".to_string()),
    otlp_protocol: Protocol::Grpc,
    sampling_rate: 1.0,
    dev_mode: true,
    batch_config: BatchConfig::default(),
};
```

### Metrics Configuration

```rust
let metrics_config = MetricsConfig {
    enabled: true,
    port: 9090,
    path: "/metrics".to_string(),
    enable_histograms: true,
    custom_buckets: vec![0.001, 0.01, 0.1, 1.0, 10.0],
    privacy_mode: PrivacyMode::Redact,
};
```

## 🚨 Alerts

### Example Alert Rules

```yaml
groups:
  - name: polymera_alerts
    rules:
      - alert: HighErrorRate
        expr: rate(polymera_errors_total[5m]) > 0.1
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"
          
      - alert: HighLatency
        expr: polymera_request_duration_p95_seconds > 1.0
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High latency detected"
```

## 🔍 Troubleshooting

### Common Issues

1. **OTEL Collector not responding**
   - Check if Docker containers are running
   - Verify port 4317/4318 are accessible
   - Check container logs: `docker logs polymera-otel-collector`

2. **Metrics not appearing in Prometheus**
   - Verify scrape configuration
   - Check target health status
   - Ensure metrics endpoint is accessible

3. **Grafana dashboards not loading**
   - Check datasource configuration
   - Verify Prometheus is accessible
   - Check dashboard provisioning

### Debug Mode

Enable debug logging:

```rust
let config = OtelConfig {
    dev_mode: true,
    // ... other config
};
```

### Logs

Check container logs for detailed information:

```bash
docker logs polymera-otel-collector
docker logs polymera-prometheus
docker logs polymera-grafana
```

## 📚 API Reference

### Core Functions

- `init_observability()` - Initialize observability system
- `shutdown_observability()` - Graceful shutdown
- `create_request_timer()` - Create request timer
- `record_business_metric_safe()` - Record business metrics
- `create_sensitive_span()` - Create privacy-aware spans

### Structs

- `OtelConfig` - OpenTelemetry configuration
- `MetricsConfig` - Metrics configuration
- `PrivacyAwareRegistry` - Metrics registry
- `RequestTimer` - Request timing wrapper

### Enums

- `PrivacyMode` - Privacy protection levels
- `BusinessMetricType` - Business metric types

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## 📄 License

MIT OR Apache-2.0

## 🆘 Support

- **Documentation**: [Polymera OS Docs](https://docs.polymera-os.org)
- **Issues**: [GitHub Issues](https://github.com/polymera-os/polymera-os/issues)
- **Discord**: [Polymera OS Community](https://discord.gg/polymera-os)
