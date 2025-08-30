# Privacy Budgets - Polymera OS

## 🎯 Overview

The Privacy Budgets system provides **differential privacy protection** through token bucket rate limiting and privacy budget management. It ensures APIs respect privacy constraints and prevent data leakage through excessive queries, implementing the principle of **privacy by design**.

## 🔒 Key Features

- **Token Bucket Rate Limiting**: Per-API and per-user privacy budget enforcement
- **Differential Privacy**: Laplace, Gaussian, and Exponential noise distributions
- **Multi-Level Budgets**: Global, API-specific, and user-specific budget management
- **Automatic Refresh**: Configurable budget restoration intervals
- **Web Framework Integration**: Actix-web and Axum middleware support
- **Comprehensive Metrics**: Real-time monitoring and alerting
- **Audit Logging**: Complete request/response tracking for compliance
- **YAML Configuration**: Flexible, human-readable configuration management

## 🏗️ Architecture

### Core Components

```
Privacy Budget System
├── Budget Manager (budget.rs)
│   ├── Token Bucket Implementation
│   ├── Budget Validation & Enforcement
│   ├── Differential Privacy Noise
│   └── Metrics Collection
├── Middleware (middleware.rs)
│   ├── HTTP Request/Response Processing
│   ├── Cost Calculation
│   ├── Framework Integrations
│   └── Audit Logging
└── Configuration (config.yaml)
    ├── Global Budget Settings
    ├── API-Specific Configurations
    ├── User Budget Definitions
    └── Differential Privacy Parameters
```

### Data Flow

1. **Request Arrival**: HTTP request enters middleware
2. **Cost Calculation**: Request cost determined based on method, path, user, etc.
3. **Budget Check**: Privacy budget manager validates request against available budgets
4. **Enforcement**: Request allowed/denied based on budget availability
5. **Response Processing**: Privacy headers added, metrics updated
6. **Audit Logging**: Complete request/response logged for compliance

## 🚀 Quick Start

### 1. Basic Usage

```rust
use polymera_privacy::{PrivacyBudgetManager, PrivacyBudgetConfig};

// Load configuration
let config = PrivacyBudgetConfig::default();
let manager = PrivacyBudgetManager::new(config)?;

// Create budget request
let request = BudgetRequest {
    request_id: "req-123".to_string(),
    api_path: "/api/v1/users".to_string(),
    method: "GET".to_string(),
    user_id: Some("user-456".to_string()),
    timestamp: Instant::now(),
    cost: 5.0,
    metadata: HashMap::new(),
};

// Check budget
let response = manager.check_budget(&request).await?;

if response.allowed {
    println!("Request allowed. Remaining budget: {}", response.remaining_global_budget);
} else {
    println!("Request denied: {}", response.error_message.unwrap());
}
```

### 2. Middleware Integration

#### Actix-web

```rust
use actix_web::{web, App, HttpServer};
use polymera_privacy::middleware::integrations::actix_web::PrivacyBudgetMiddlewareTransform;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = PrivacyBudgetConfig::default();
    let budget_manager = Arc::new(PrivacyBudgetManager::new(config).unwrap());
    let middleware_config = MiddlewareConfig::default();
    let middleware = Arc::new(PrivacyBudgetMiddleware::new(budget_manager, middleware_config));

    HttpServer::new(move || {
        App::new()
            .wrap(PrivacyBudgetMiddlewareTransform::new(middleware.clone()))
            .service(web::resource("/api/users").to(get_users))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

#### Axum

```rust
use axum::{routing::get, Router};
use polymera_privacy::middleware::integrations::axum::privacy_budget_middleware;

#[tokio::main]
async fn main() {
    let config = PrivacyBudgetConfig::default();
    let budget_manager = Arc::new(PrivacyBudgetManager::new(config).unwrap());
    let middleware_config = MiddlewareConfig::default();
    let middleware = Arc::new(PrivacyBudgetMiddleware::new(budget_manager, middleware_config));

    let app = Router::new()
        .route("/api/users", get(get_users))
        .layer(axum::middleware::from_fn_with_state(
            middleware.clone(),
            privacy_budget_middleware,
        ));

    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

### 3. Configuration

```yaml
# privacy/config.yaml
global:
  total_budget: 10000.0
  time_window: 3600  # 1 hour
  max_per_request: 1000.0
  min_per_request: 0.1
  strict_mode: true

apis:
  users:
    path: "/api/v1/users"
    methods: ["GET", "POST", "PUT", "DELETE"]
    budget: 2000.0
    time_window: 3600
    cost_per_request: 10.0
    max_requests: 1000
    rate_limit: true

users:
  admin:
    user_id: "admin"
    personal_budget: 5000.0
    time_window: 3600
    cost_per_request: 5.0
    priority: 10

differential_privacy:
  epsilon: 1.0
  delta: 1e-5
  sensitivity: 1.0
  noise_type: "Laplace"
  adaptive_noise: true
```

## 📊 Budget Management

### Budget Types

#### 1. Global Budget
- **Purpose**: System-wide privacy protection
- **Scope**: All APIs and users combined
- **Refresh**: Manual or automatic at configurable intervals
- **Enforcement**: Strict mode available

#### 2. API-Specific Budgets
- **Purpose**: Per-endpoint privacy control
- **Scope**: Individual API endpoints or groups
- **Features**: Method-specific costs, rate limiting
- **Configuration**: Path patterns, burst sizes

#### 3. User-Specific Budgets
- **Purpose**: Per-user privacy constraints
- **Scope**: Individual user accounts
- **Features**: Priority levels, personal limits
- **Refresh**: Partial or full restoration

### Budget Calculation

```rust
// Automatic cost calculation based on request characteristics
let cost = utils::calculate_request_cost(
    "POST",                    // HTTP method
    "/api/v1/analytics",      // Path
    Some("user-123"),         // User ID
    2048,                     // Body size (bytes)
    &[("limit", "100")],      // Query parameters
    &headers,                  // Request headers
);

// Method-based multipliers
// GET: 1.0x, POST: 2.0x, PUT: 1.5x, DELETE: 1.5x, PATCH: 1.2x

// Path-based adjustments
// /analytics, /ml: 2.0x (data-intensive)
// /health, /status: 0.1x (health checks)

// Size-based costs
// 0.1 per KB of request/response data

// Authentication discounts
// Bearer tokens: 0.8x (authenticated requests)
```

## 🔐 Differential Privacy

### Noise Distributions

#### 1. Laplace Distribution
- **Use Case**: General-purpose noise addition
- **Parameters**: Epsilon (privacy budget), sensitivity
- **Formula**: `noise = -scale * ln(1 - 2|u|) * sign(u)`
- **Properties**: Symmetric, heavy-tailed

#### 2. Gaussian Distribution
- **Use Case**: When delta > 0 is acceptable
- **Parameters**: Epsilon, delta, sensitivity
- **Formula**: `noise = N(0, σ²)` where `σ = sensitivity * √(2ln(1/δ)) / ε`
- **Properties**: Normal distribution, lighter tails

#### 3. Exponential Distribution
- **Use Case**: One-sided noise addition
- **Parameters**: Epsilon, sensitivity
- **Formula**: `noise = -scale * ln(u)`
- **Properties**: Positive-only, memoryless

### Implementation

```rust
// Add differential privacy noise
let original_value = 100.0;
let noisy_value = manager.add_differential_privacy_noise(original_value)?;

// Configure noise parameters
let dp_config = DifferentialPrivacyConfig {
    epsilon: 1.0,        // Privacy budget (lower = more private)
    delta: 1e-5,         // Failure probability
    sensitivity: 1.0,    // Maximum change in output
    noise_type: NoiseType::Laplace,
    adaptive_noise: true,
};
```

## 📈 Monitoring & Metrics

### Available Metrics

#### 1. Request Metrics
- `privacy_budget_total_requests`: Total requests processed
- `privacy_budget_requests_allowed`: Requests that passed budget checks
- `privacy_budget_requests_denied`: Requests rejected due to budget exhaustion
- `privacy_budget_budget_violations`: Total budget violations

#### 2. Budget Usage Metrics
- `privacy_budget_consumed`: Total budget consumed
- `privacy_budget_global_usage`: Current global budget usage
- `privacy_budget_api_usage`: Per-API budget consumption
- `privacy_budget_user_usage`: Per-user budget consumption

#### 3. Performance Metrics
- `privacy_budget_response_time`: Request processing time
- `privacy_budget_rate_limit_hits`: Rate limiting statistics

### Prometheus Integration

```yaml
# Enable Prometheus metrics export
monitoring:
  enabled: true
  metrics_export: true
  export_format: "prometheus"
  interval: 60
```

### Grafana Dashboards

```yaml
# Enable Grafana dashboard
monitoring:
  dashboard: true
  dashboard_interval: 30
```

## 🚨 Alerting & Notifications

### Alert Configuration

```yaml
monitoring:
  alerting: true
  alert_channels: ["webhook", "email", "slack"]
  webhook_url: "https://alerts.polymera-os.org/webhook"
  
advanced:
  violation_alerts: true
  alert_threshold: 80.0  # Alert at 80% budget consumption
```

### Alert Types

1. **Budget Exhaustion**: When any budget reaches 0
2. **High Usage**: When budget usage exceeds threshold (default: 80%)
3. **Rate Limit Hits**: When rate limiting is frequently triggered
4. **Anomaly Detection**: Unusual request patterns or budget consumption

## 🔧 Configuration Reference

### Global Configuration

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `total_budget` | f64 | 10000.0 | Total global privacy budget |
| `time_window` | u64 | 3600 | Budget refresh interval (seconds) |
| `max_per_request` | f64 | 1000.0 | Maximum budget per request |
| `min_per_request` | f64 | 0.1 | Minimum budget per request |
| `strict_mode` | bool | true | Reject requests when budget exhausted |

### API Configuration

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `path` | String | - | API endpoint path pattern |
| `methods` | Vec<String> | - | Allowed HTTP methods |
| `budget` | f64 | - | Privacy budget for this API |
| `time_window` | u64 | - | Budget refresh interval |
| `cost_per_request` | f64 | - | Base cost per request |
| `max_requests` | Option<u32> | None | Maximum requests per time window |
| `rate_limit` | bool | false | Enable rate limiting |
| `burst_size` | Option<u32> | None | Rate limit burst size |

### User Configuration

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `user_id` | String | - | Unique user identifier |
| `personal_budget` | f64 | - | Personal privacy budget |
| `time_window` | u64 | - | Budget refresh interval |
| `cost_per_request` | f64 | - | Cost per request for this user |
| `max_requests` | Option<u32> | None | Maximum requests per time window |
| `priority` | u8 | 5 | User priority level (1-10) |

### Differential Privacy Configuration

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `epsilon` | f64 | 1.0 | Privacy budget parameter |
| `delta` | f64 | 1e-5 | Failure probability |
| `sensitivity` | f64 | 1.0 | Maximum change in output |
| `noise_type` | String | "Laplace" | Noise distribution type |
| `adaptive_noise` | bool | false | Enable adaptive noise scaling |

## 🧪 Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test test_privacy_budget_manager

# Run with output
cargo test -- --nocapture
```

### Integration Tests

```bash
# Run integration tests
cargo test --test integration

# Run with specific features
cargo test --features "actix-web,axum"
```

### Test Coverage

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

## 🔍 Troubleshooting

### Common Issues

#### 1. Budget Exhaustion
**Symptoms**: 429 Too Many Requests errors
**Solutions**:
- Increase budget allocations
- Reduce cost per request
- Enable partial refresh
- Check for budget leaks

#### 2. Configuration Errors
**Symptoms**: System startup failures
**Solutions**:
- Validate YAML syntax
- Check parameter ranges
- Verify file permissions
- Use configuration validation

#### 3. Performance Issues
**Symptoms**: High latency, memory usage
**Solutions**:
- Enable caching
- Optimize token bucket operations
- Use async processing
- Monitor resource usage

### Debug Mode

```yaml
development:
  debug_mode: true
  performance_profiling: true
  memory_profiling: true
```

### Logging

```rust
// Enable debug logging
env_logger::Builder::new()
    .filter_level(log::LevelFilter::Debug)
    .init();

// Privacy budget specific logging
log::debug!("Budget check: {:?}", request);
log::info!("Budget response: {:?}", response);
```

## 📚 API Reference

### Core Types

#### PrivacyBudgetManager
```rust
pub struct PrivacyBudgetManager {
    // Main budget management interface
}

impl PrivacyBudgetManager {
    pub fn new(config: PrivacyBudgetConfig) -> Result<Self, PrivacyBudgetError>
    pub async fn check_budget(&self, request: &BudgetRequest) -> Result<BudgetResponse, PrivacyBudgetError>
    pub async fn get_metrics(&self) -> PrivacyMetrics
    pub async fn refresh_budgets(&self) -> Result<(), PrivacyBudgetError>
    pub fn add_differential_privacy_noise(&self, value: f64) -> Result<f64, PrivacyBudgetError>
}
```

#### PrivacyBudgetMiddleware
```rust
pub struct PrivacyBudgetMiddleware {
    // HTTP middleware for privacy budget enforcement
}

impl PrivacyBudgetMiddleware {
    pub fn new(budget_manager: Arc<PrivacyBudgetManager>, config: MiddlewareConfig) -> Self
    pub async fn process_request(&self, context: &HttpRequestContext) -> Result<BudgetResponse, PrivacyBudgetError>
    pub async fn process_response(&self, request_context: &HttpRequestContext, response_context: &HttpResponseContext)
    pub async fn get_metrics(&self) -> MiddlewareMetrics
}
```

### Error Types

```rust
pub enum PrivacyBudgetError {
    InsufficientBudget { message: String },
    RateLimitExceeded { message: String },
    ConfigurationError { message: String },
    ApiNotFound { api_path: String },
    UserNotFound { user_id: String },
    InvalidRequest { message: String },
    DifferentialPrivacyError { message: String },
}
```

## 🔄 Best Practices

### 1. Budget Sizing
- **Start Conservative**: Begin with smaller budgets and increase based on usage
- **Monitor Usage**: Track budget consumption patterns
- **Plan for Peaks**: Account for traffic spikes and seasonal variations
- **User Segmentation**: Different budget levels for different user types

### 2. Cost Calculation
- **Method-Based**: Higher costs for data-modifying operations
- **Path-Based**: Lower costs for health checks, higher for data-intensive endpoints
- **Size-Based**: Proportional costs for large requests/responses
- **User-Based**: Discounts for authenticated users

### 3. Differential Privacy
- **Epsilon Selection**: Balance privacy vs. utility (0.1-10.0 range)
- **Delta Management**: Keep delta very small (< 1e-5)
- **Sensitivity Analysis**: Understand maximum possible output changes
- **Noise Distribution**: Choose based on use case requirements

### 4. Monitoring & Alerting
- **Real-Time Metrics**: Monitor budget consumption continuously
- **Proactive Alerts**: Set thresholds below 100% exhaustion
- **Trend Analysis**: Identify usage patterns and anomalies
- **Capacity Planning**: Use metrics for infrastructure scaling

### 5. Security & Compliance
- **Audit Logging**: Complete request/response tracking
- **Access Control**: Secure configuration and metrics access
- **Data Retention**: Configure appropriate log retention periods
- **Compliance**: GDPR, CCPA, HIPAA support features

## 🚀 Performance Optimization

### 1. Caching
```yaml
performance:
  budget_caching: true
  cache_ttl: 300  # 5 minutes
```

### 2. Async Processing
```yaml
performance:
  async_processing: true
  worker_threads: 4
```

### 3. Batch Processing
```yaml
performance:
  batch_processing: true
  batch_size: 100
  batch_timeout: 5
```

### 4. Connection Pooling
```yaml
performance:
  connection_pooling: true
  pool_size: 100
```

## 🔮 Future Enhancements

### Planned Features

- [ ] **Machine Learning Integration**: ML-based budget prediction and optimization
- [ ] **Advanced Noise Mechanisms**: Custom noise distributions and adaptive scaling
- [ ] **Federated Privacy**: Multi-party privacy budget coordination
- [ ] **Privacy Budget Markets**: Dynamic budget allocation and trading
- [ ] **Advanced Analytics**: Privacy-preserving analytics and insights
- [ ] **Blockchain Integration**: Immutable privacy budget tracking
- [ ] **Edge Computing**: Distributed privacy budget enforcement
- [ ] **Real-Time Learning**: Adaptive budget adjustment based on usage patterns

### Integration Opportunities

- **Kubernetes**: Native K8s privacy budget operators
- **Service Mesh**: Istio/Envoy privacy budget policies
- **API Gateways**: Kong, Tyk privacy budget plugins
- **Cloud Providers**: AWS, GCP, Azure privacy budget services
- **Monitoring**: Prometheus, Grafana, Datadog native support

## 📄 License

This project is licensed under the MIT License or Apache License 2.0 - see the [LICENSE](../LICENSE) file for details.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

### Development Setup

```bash
# Clone and setup
git clone https://github.com/polymera-os/polymera-os.git
cd polymera-os/privacy

# Install dependencies
cargo build

# Run tests
cargo test

# Run with specific features
cargo test --features "actix-web,axum"

# Format code
cargo fmt

# Lint code
cargo clippy
```

## 🆘 Support

- **Documentation**: [Polymera OS Docs](https://docs.polymera-os.org)
- **Issues**: [GitHub Issues](https://github.com/polymera-os/polymera-os/issues)
- **Discord**: [Polymera OS Community](https://discord.gg/polymera-os)
- **Email**: [support@polymera-os.org](mailto:support@polymera-os.org)

## 📋 Changelog

### v0.1.0 (Current)
- Initial implementation of privacy budget system
- Token bucket rate limiting
- Differential privacy noise mechanisms
- Web framework middleware (Actix-web, Axum)
- YAML configuration support
- Comprehensive metrics and monitoring
- Audit logging and compliance features
- Extensive testing and documentation

---

*The Privacy Budgets system ensures that Polymera OS maintains the highest standards of privacy protection while providing powerful, flexible API management capabilities.*
