# Policy Engine Baseline

A WebAssembly-based policy engine for Polymera OS that compiles Rego policies to WASM and provides sandboxed execution with comprehensive security controls.

## Overview

The Policy Engine Baseline provides a robust, secure, and performant foundation for policy-based access control in Polymera OS. It compiles Open Policy Agent (OPA) Rego policies to WebAssembly and executes them in a sandboxed environment with strict resource limits and comprehensive audit logging.

## Features

### Core Functionality
- **Rego to WASM Compilation**: Convert OPA Rego policies to WebAssembly modules
- **Sandboxed Execution**: Secure policy evaluation with resource isolation
- **Host Function API**: Rich set of built-in functions for policy logic
- **Resource Limits**: Configurable memory, CPU, and execution time limits
- **Policy Caching**: Intelligent caching for improved performance
- **Audit Logging**: Comprehensive logging of all policy decisions

### Security Features
- **Default Deny**: All operations denied unless explicitly allowed
- **Input Validation**: Comprehensive input sanitization and validation
- **Resource Isolation**: Strict limits on memory and execution time
- **Function Sandboxing**: Controlled access to host system functions
- **Audit Trail**: Complete record of policy evaluation decisions

### Performance Features
- **WASM Execution**: High-performance policy evaluation
- **Policy Caching**: Reduce compilation and loading overhead
- **Resource Monitoring**: Real-time performance metrics
- **Optimized Builds**: Release mode with LTO optimization
- **Benchmarking**: Built-in performance measurement tools

## Architecture

### Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Rego Policy  │    │  Policy Engine  │    │   WASM Host    │
│                 │    │                 │    │                 │
│  - examples.rego│───▶│  - PolicyEngine │───▶│  - WasmHost    │
│  - compile.sh   │    │  - Config       │    │  - Functions   │
│                 │    │  - Cache        │    │  - Limits      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Data Flow

1. **Policy Compilation**: Rego policies compiled to WASM using OPA
2. **Policy Loading**: WASM modules loaded into the policy engine
3. **Request Processing**: Policy requests evaluated against loaded policies
4. **Sandboxed Execution**: Policies executed in WASM host environment
5. **Result Processing**: Policy decisions returned with rationale and metadata
6. **Audit Logging**: All decisions logged for compliance and debugging

## Quick Start

### Prerequisites

- Rust 1.70+ with Cargo
- Open Policy Agent (OPA) 0.58.0+
- CMake 3.16+ (for WASM compilation)
- Git

### Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/polymera-os/polymera-os.git
   cd polymera-os/services/policy
   ```

2. **Build the service**:
   ```bash
   cargo build --release
   ```

3. **Compile example policies**:
   ```bash
   cd ../../policy
   chmod +x compile.sh
   ./compile.sh
   ```

4. **Run tests**:
   ```bash
   cd ../services/policy
   cargo test
   cargo run --bin policy-test
   ```

### Basic Usage

```rust
use polymera_policy::{
    DefaultPolicyService, PolicyRequest, PolicyContext,
    PolicyEngineConfig
};

// Create policy service
let config = PolicyEngineConfig::default();
let service = DefaultPolicyService::new(config)?;

// Load policy
service.load_policy(Path::new("examples.wasm"))?;

// Create policy request
let input = serde_json::json!({
    "user": {"role": "developer"},
    "file": {"path": "/home/user/doc.txt", "size": 1024}
});

let request = PolicyRequest::new("examples".to_string(), input)
    .with_user("developer".to_string())
    .with_rule("file_access".to_string());

// Evaluate policy
let response = service.evaluate(&request)?;

if response.is_allowed() {
    println!("Access granted: {}", response.reason());
} else {
    println!("Access denied: {}", response.reason());
}
```

## Policy Examples

### File Access Policy

```rego
package polymera.policy

file_access = {
    "allowed": true,
    "reason": "File access granted",
    "permissions": ["read", "write"],
    "constraints": {
        "max_size": "100MB",
        "allowed_extensions": [".txt", ".md", ".rs", ".go", ".py", ".js"],
        "restricted_paths": ["/etc/", "/sys/", "/proc/"]
    }
} {
    # User has appropriate role
    input.user.role == "developer"
    
    # File path is not restricted
    not startswith(input.file.path, "/etc/")
    not startswith(input.file.path, "/sys/")
    not startswith(input.file.path, "/proc/")
    
    # File extension is allowed
    input.file.extension in [".txt", ".md", ".rs", ".go", ".py", ".js"]
    
    # File size is within limits
    input.file.size <= 104857600  # 100MB in bytes
}
```

### Network Access Policy

```rego
package polymera.policy

network_access = {
    "allowed": true,
    "reason": "Network access granted",
    "constraints": {
        "allowed_protocols": ["HTTP", "HTTPS", "SSH"],
        "allowed_ports": [22, 80, 443, 8080, 8443],
        "blocked_domains": ["malware.example.com", "phishing.example.com"]
    }
} {
    # Protocol is allowed
    input.network.protocol in ["HTTP", "HTTPS", "SSH"]
    
    # Port is allowed
    input.network.port in [22, 80, 443, 8080, 8443]
    
    # Domain is not blocked
    not input.network.domain in ["malware.example.com", "phishing.example.com"]
    
    # For HTTPS, require valid certificate
    input.network.protocol == "HTTPS"
    input.network.certificate.valid == true
}
```

## Configuration

### Policy Engine Configuration

```rust
use polymera_policy::PolicyEngineConfig;
use std::time::Duration;
use std::path::PathBuf;

let config = PolicyEngineConfig {
    // Execution limits
    max_execution_time: Duration::from_millis(100),
    max_memory_bytes: 64 * 1024 * 1024, // 64MB
    
    // Audit logging
    enable_audit_logging: true,
    audit_log_path: Some(PathBuf::from("/var/log/polymera/policy_audit.log")),
    
    // Caching
    policy_cache_size: 100,
    
    // Metrics
    enable_metrics: true,
};
```

### Resource Limits

```rust
use polymera_policy::ResourceLimits;

let limits = ResourceLimits {
    max_memory_bytes: 128 * 1024 * 1024, // 128MB
    max_execution_time: Duration::from_millis(200),
    max_function_calls: 2000,
    max_table_size: 20000,
    max_instances: 20,
    max_tables: 20,
    max_memories: 20,
};
```

## Host Functions

The WASM host provides a rich set of built-in functions for policy logic:

### JSON Functions
- `json_parse(string)`: Parse JSON string to object
- `json_stringify(object)`: Convert object to JSON string

### String Functions
- `string_length(string)`: Get string length
- `string_substring(string, start, end)`: Extract substring

### Array Functions
- `array_length(array)`: Get array length
- `array_push(array, element)`: Add element to array

### Math Functions
- `math_max(a, b)`: Get maximum of two numbers
- `math_min(a, b)`: Get minimum of two numbers

### Time Functions
- `time_now()`: Get current timestamp

### Logging Functions
- `log_info(message)`: Log info message
- `log_warning(message)`: Log warning message
- `log_error(message)`: Log error message

## API Reference

### Core Types

#### PolicyEngine
Main policy engine for loading and evaluating policies.

```rust
pub struct PolicyEngine {
    // ... implementation details
}

impl PolicyEngine {
    pub fn new(config: PolicyEngineConfig) -> Result<Self, PolicyEngineError>;
    pub fn load_policy(&self, policy_path: &Path) -> Result<String, PolicyEngineError>;
    pub fn evaluate(&self, policy_name: &str, input: &Value, rule: Option<&str>) -> Result<PolicyResult, PolicyEngineError>;
    pub fn get_stats(&self) -> PolicyEngineStats;
    pub fn clear_cache(&self);
}
```

#### PolicyResult
Result of policy evaluation.

```rust
pub struct PolicyResult {
    pub allowed: bool,
    pub reason: String,
    pub code: Option<String>,
    pub constraints: Option<Value>,
    pub metadata: EvaluationMetadata,
}
```

#### PolicyContext
Context information for policy evaluation.

```rust
pub struct PolicyContext {
    pub request_id: String,
    pub user: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub source: Option<String>,
    pub metadata: HashMap<String, Value>,
}
```

### Service Interface

#### PolicyService
Trait defining the policy service interface.

```rust
pub trait PolicyService {
    fn evaluate(&self, request: &PolicyRequest) -> Result<PolicyResponse, PolicyEngineError>;
    fn load_policy(&self, policy_path: &Path) -> Result<String, PolicyEngineError>;
    fn reload_policy(&self, policy_path: &Path) -> Result<(), PolicyEngineError>;
    fn validate_policy(&self, policy_path: &Path) -> Result<(), PolicyEngineError>;
    fn get_stats(&self) -> PolicyEngineStats;
    fn clear_cache(&self);
    fn get_cache_info(&self) -> HashMap<String, (Instant, u64)>;
}
```

## Testing

### Unit Tests

```bash
# Run all unit tests
cargo test

# Run tests with specific features
cargo test --features "full"

# Run tests with output
cargo test -- --nocapture
```

### Integration Tests

```bash
# Run integration test binary
cargo run --bin policy-test

# Run with specific test
cargo run --bin policy-test -- --test-threads=1
```

### Performance Tests

```bash
# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench policy_bench
```

### Policy Validation

```bash
# Compile and validate policies
cd ../../policy
./compile.sh

# Validate compiled policies
./build/validate_policies.sh
```

## Performance

### Benchmarks

Typical performance characteristics on Intel i7-10700K @ 3.80GHz:

- **Policy Loading**: ~5-10ms per policy
- **Policy Evaluation**: ~0.1-1ms per request
- **Memory Usage**: ~1-5MB per loaded policy
- **Cache Hit Rate**: >95% for repeated policies

### Optimization Tips

1. **Policy Caching**: Enable policy caching for frequently used policies
2. **Resource Limits**: Set appropriate resource limits for your use case
3. **Batch Evaluation**: Group multiple policy evaluations when possible
4. **Policy Optimization**: Use efficient Rego patterns and avoid complex queries

## Security Considerations

### Sandboxing
- All policy execution occurs in isolated WASM environments
- Host function access is strictly controlled and logged
- Resource limits prevent denial-of-service attacks

### Input Validation
- All inputs are validated before policy evaluation
- JSON schema validation can be enforced
- Malicious inputs are rejected before processing

### Audit Logging
- All policy decisions are logged with full context
- Execution traces capture function calls and resource usage
- Logs can be integrated with SIEM systems

### Resource Limits
- Memory usage is strictly limited and monitored
- Execution time is bounded to prevent hanging
- Function call counts are tracked and limited

## Deployment

### Production Configuration

```toml
# config.toml
[policy_engine]
max_execution_time = "100ms"
max_memory_bytes = 67108864  # 64MB
enable_audit_logging = true
audit_log_path = "/var/log/polymera/policy_audit.log"
policy_cache_size = 1000
enable_metrics = true

[resource_limits]
max_memory_bytes = 134217728  # 128MB
max_execution_time = "200ms"
max_function_calls = 5000
max_table_size = 50000
max_instances = 50
max_tables = 50
max_memories = 50
```

### Docker Deployment

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/policy-server /usr/local/bin/
COPY --from=builder /app/policies /etc/polymera/policies/

EXPOSE 8080
CMD ["policy-server"]
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: policy-engine
spec:
  replicas: 3
  selector:
    matchLabels:
      app: policy-engine
  template:
    metadata:
      labels:
        app: policy-engine
    spec:
      containers:
      - name: policy-engine
        image: polymera/policy-engine:latest
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "500m"
        volumeMounts:
        - name: policies
          mountPath: /etc/polymera/policies
        - name: audit-logs
          mountPath: /var/log/polymera
      volumes:
      - name: policies
        configMap:
          name: policy-config
      - name: audit-logs
        emptyDir: {}
```

## Monitoring and Observability

### Metrics

The policy engine provides comprehensive metrics:

- **Policy Evaluations**: Total, successful, and failed evaluations
- **Performance**: Execution time, memory usage, cache hit rates
- **Resource Usage**: Memory allocation, function calls, instances
- **Errors**: Error types, frequencies, and patterns

### Logging

Structured logging with configurable levels:

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "level": "INFO",
  "policy_name": "file_access",
  "request_id": "req-123",
  "user": "developer",
  "source": "192.168.1.100",
  "input": {"file": {"path": "/home/user/doc.txt"}},
  "result": {"allowed": true, "reason": "Access granted"},
  "execution_time_us": 150,
  "memory_usage_bytes": 2048
}
```

### Health Checks

```bash
# Health check endpoint
curl http://localhost:8080/health

# Metrics endpoint
curl http://localhost:8080/metrics

# Policy status
curl http://localhost:8080/policies/status
```

## Troubleshooting

### Common Issues

1. **Policy Compilation Failures**
   - Check OPA version compatibility
   - Verify Rego syntax
   - Check file permissions

2. **WASM Loading Errors**
   - Ensure WASM files are valid
   - Check file paths and permissions
   - Verify WASM module compatibility

3. **Performance Issues**
   - Monitor resource usage
   - Check cache hit rates
   - Review policy complexity

4. **Memory Issues**
   - Adjust memory limits
   - Monitor memory allocation patterns
   - Check for memory leaks in policies

### Debug Mode

Enable debug logging for troubleshooting:

```rust
use tracing_subscriber;

tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

### Profiling

Use built-in profiling tools:

```bash
# Profile policy execution
cargo run --bin policy-bench -- --profile

# Memory profiling
cargo run --bin policy-bench -- --memory-profile
```

## Contributing

### Development Setup

1. **Fork the repository**
2. **Create a feature branch**
3. **Make your changes**
4. **Add tests for new functionality**
5. **Run the test suite**
6. **Submit a pull request**

### Code Style

- Follow Rust coding standards
- Use meaningful variable and function names
- Add comprehensive documentation
- Include unit tests for all new code
- Follow error handling patterns

### Testing Guidelines

- Write unit tests for all public APIs
- Include integration tests for complex workflows
- Add performance benchmarks for critical paths
- Test error conditions and edge cases
- Ensure test coverage >90%

## License

This project is licensed under the MIT License or Apache License 2.0 - see the [LICENSE](../LICENSE) file for details.

## Support

- **Documentation**: [docs.polymera-os.org](https://docs.polymera-os.org)
- **Issues**: [GitHub Issues](https://github.com/polymera-os/polymera-os/issues)
- **Discussions**: [GitHub Discussions](https://github.com/polymera-os/polymera-os/discussions)
- **Email**: [team@polymera-os.org](mailto:team@polymera-os.org)

## Roadmap

### Upcoming Features

- **Policy Versioning**: Support for policy versioning and rollbacks
- **Distributed Policies**: Multi-node policy distribution and synchronization
- **Policy Templates**: Reusable policy templates and inheritance
- **Advanced Host Functions**: Additional built-in functions for common use cases
- **Policy Testing Framework**: Comprehensive testing and validation tools

### Long-term Goals

- **Policy Language Extensions**: Support for additional policy languages
- **Machine Learning Integration**: ML-based policy optimization and adaptation
- **Policy Marketplace**: Community-driven policy sharing and distribution
- **Real-time Policy Updates**: Dynamic policy updates without service restart
- **Cross-platform Support**: Support for additional operating systems and architectures
