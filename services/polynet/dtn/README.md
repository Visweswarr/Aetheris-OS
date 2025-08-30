# DTN Envelope Service for Polymera OS

A Delay-Tolerant Networking (DTN) envelope service that provides reliable message delivery in challenging network conditions with TTL-based expiration, acknowledgment tracking, and multiple routing strategies.

## Features

- **TTL-based Expiration**: Automatic cleanup of expired envelopes
- **Nonce-based Duplicate Detection**: Prevents duplicate message processing
- **Acknowledgment Tracking**: Configurable acknowledgment requirements and tracking
- **Multiple Routing Strategies**: Flood, Directed, Opportunistic, Epidemic, and Spray-and-Wait
- **Cryptographic Security**: Ed25519 signatures and optional payload encryption
- **Persistent Storage**: Sled-based storage with configurable eviction policies
- **Geographic & Network Constraints**: Location and capability-based routing
- **Priority Handling**: Priority-based envelope processing and eviction
- **Comprehensive Testing**: Unit tests, integration tests, and performance benchmarks

## Architecture

The DTN service consists of several key components:

### Core Components

- **`DtnEnvelope`**: Main message container with TTL, routing, and security information
- **`DtnStore`**: Persistent storage with automatic cleanup and eviction policies
- **`DtnRouter`**: Routing engine with multiple strategy implementations
- **`DtnSecurity`**: Cryptographic operations and signature verification
- **`DtnService`**: High-level service coordination

### Routing Strategies

1. **Flood**: Send to all available nodes
2. **Directed**: Find direct path to destination
3. **Opportunistic**: Forward when opportunity arises
4. **Epidemic**: Spread to all nodes for maximum coverage
5. **Spray-and-Wait**: Limited copies for controlled dissemination

### Storage Policies

- **FIFO**: First in, first out
- **LRU**: Least recently used
- **Priority**: Priority-based eviction
- **TTL**: Time-to-live based
- **Size**: Size-based eviction

## Installation

### Prerequisites

- Rust 1.70+
- Bazel 6.0+
- Protocol Buffers compiler

### Building with Bazel

```bash
# Build the library
bazel build //services/polynet/dtn:polynet_dtn

# Build the binary
bazel build //services/polynet/dtn:polynet_dtn_bin

# Run tests
bazel test //services/polynet/dtn:polynet_dtn_test_suite

# Run benchmarks
bazel run //services/polynet/dtn:dtn_benchmarks
```

### Building with Cargo

```bash
cd services/polynet/dtn

# Build
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

## Usage

### Command Line Interface

The DTN service provides a comprehensive CLI for testing and management:

```bash
# Start the service
polynet-dtn start --port 8080 --data-dir data/dtn

# Send a test envelope
polynet-dtn send \
  --source "did:polynet:source" \
  --destination "did:polynet:dest" \
  --message-type "test" \
  --payload "Hello DTN!" \
  --ttl 3600 \
  --priority 100

# Receive envelopes for a node
polynet-dtn receive --node-id "did:polynet:dest" --max-envelopes 10

# Query envelope status
polynet-dtn query --envelope-id "envelope-uuid-here"

# Clean up expired envelopes
polynet-dtn cleanup

# Show service statistics
polynet-dtn stats

# Run basic tests
polynet-dtn test
```

### Programmatic Usage

```rust
use polynet_dtn::{DtnService, DtnEnvelope, StoreConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create service
    let config = StoreConfig::default();
    let service = DtnService::new(config)?;
    
    // Generate security key
    service.generate_signing_key().await?;
    
    // Create and send envelope
    let envelope = DtnEnvelope::new(
        "did:polynet:source".to_string(),
        "did:polynet:dest".to_string(),
        3600,
        "test_message".to_string(),
        b"test payload".to_vec(),
        100,
    );
    
    let envelope_id = service.send_envelope(envelope).await?;
    println!("Envelope sent: {}", envelope_id);
    
    // Receive envelopes
    let received = service.receive_envelopes("did:polynet:dest", 10).await?;
    println!("Received {} envelopes", received.len());
    
    Ok(())
}
```

## Configuration

### Store Configuration

```rust
use polynet_dtn::StoreConfig;

let config = StoreConfig {
    max_envelopes: 1000,
    max_envelope_size: 1024 * 1024, // 1MB
    max_ttl: 86400, // 24 hours
    eviction_policy: EvictionPolicy::Ttl,
    respect_priority: true,
    detect_duplicates: true,
    cleanup_interval: 300, // 5 minutes
    persistence_path: "data/dtn_store".to_string(),
};
```

### Service Configuration

```rust
use polynet_dtn::DtnServiceConfig;

let service_config = DtnServiceConfig {
    store_config: config,
    enable_security: true,
    enable_routing: true,
    cleanup_interval: 300,
};
```

## API Reference

### Core Types

#### DtnEnvelope

```rust
pub struct DtnEnvelope {
    pub id: String,
    pub source: String,
    pub destination: String,
    pub ttl: u64,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
    pub nonce: Vec<u8>,
    pub priority: u32,
    pub message_type: String,
    pub payload: Vec<u8>,
    pub routing: RoutingInfo,
    pub ack_requirements: AckRequirements,
    pub security: SecurityInfo,
    pub metadata: HashMap<String, String>,
}
```

#### RoutingInfo

```rust
pub struct RoutingInfo {
    pub intermediate_nodes: Vec<String>,
    pub max_hops: u32,
    pub hop_count: u32,
    pub strategy: RoutingStrategy,
    pub geo_constraints: Option<GeographicConstraints>,
    pub net_constraints: Option<NetworkConstraints>,
}
```

#### AckRequirements

```rust
pub struct AckRequirements {
    pub required: bool,
    pub ack_type: AckType,
    pub ack_timeout: u64,
    pub max_retries: u32,
    pub required_ack_nodes: Vec<String>,
    pub ack_records: Vec<AckRecord>,
}
```

### Service Methods

#### Core Operations

- `send_envelope(envelope: DtnEnvelope) -> Result<String, Error>`
- `receive_envelopes(node_id: &str, max_envelopes: usize) -> Result<Vec<DtnEnvelope>, Error>`
- `acknowledge_envelope(envelope_id: &str, ack_node: &str, ack_type: AckType, status: AckStatus, message: &str) -> Result<(), Error>`
- `query_envelope(envelope_id: &str) -> Result<Option<DtnEnvelope>, Error>`
- `cleanup_expired() -> Result<usize, Error>`

#### Routing Management

- `add_routing_node(node: RoutingNode) -> Result<(), Error>`
- `remove_routing_node(node_id: &str) -> Result<(), Error>`

#### Security Operations

- `generate_signing_key() -> Result<Vec<u8>, Error>`
- `import_verifying_key(key_bytes: &[u8], key_id: &str) -> Result<(), Error>`
- `verify_envelope_signature(envelope: &DtnEnvelope) -> Result<bool, Error>`

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test modules
cargo test envelope_tests
cargo test store_tests
cargo test routing_tests
cargo test security_tests

# Run with logging
RUST_LOG=debug cargo test

# Run integration tests
cargo test --test integration
```

### Test Coverage

The test suite covers:

- **Unit Tests**: Individual component functionality
- **Integration Tests**: End-to-end service operations
- **Performance Tests**: Benchmarking critical operations
- **Edge Cases**: Expired envelopes, duplicate detection, routing failures
- **Security**: Signature verification, encryption/decryption
- **Storage**: Eviction policies, persistence, cleanup

### Key Test Scenarios

1. **Expired Envelope Handling**: Verify expired envelopes are properly dropped
2. **Acknowledgment Path**: Test complete acknowledgment workflow
3. **Routing Strategies**: Validate all routing strategy implementations
4. **Security Operations**: Test signing, verification, and encryption
5. **Storage Management**: Test eviction policies and cleanup
6. **Performance**: Benchmark envelope operations

## Performance

### Benchmarks

The service includes performance benchmarks for:

- Envelope creation: ~1000 envelopes/second
- Envelope signing: ~100 signatures/second
- Storage operations: ~100 store/retrieve operations/second

### Optimization Tips

1. **Batch Operations**: Group multiple envelopes for bulk processing
2. **Async Operations**: Use async/await for I/O-bound operations
3. **Memory Management**: Configure appropriate storage limits
4. **Cleanup Frequency**: Adjust cleanup intervals based on TTL patterns

## Security

### Cryptographic Features

- **Ed25519 Signatures**: Fast, secure digital signatures
- **SHA-256 Hashing**: Cryptographic hash functions for integrity
- **Optional Encryption**: Payload encryption support
- **Key Management**: Secure key generation and storage

### Security Levels

- **None**: No security features
- **Low**: Basic integrity checks
- **Medium**: Digital signatures
- **High**: Signatures + encryption
- **Critical**: Maximum security features

## Contributing

### Development Setup

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

### Code Style

- Follow Rust formatting guidelines (`rustfmt`)
- Use meaningful variable and function names
- Add comprehensive documentation
- Include unit tests for all new code
- Follow error handling patterns

### Testing Guidelines

- Write tests for all public APIs
- Include edge case testing
- Test error conditions
- Validate performance characteristics
- Ensure test coverage >90%

## License

This project is licensed under the MIT License - see the [LICENSE](../../../LICENSE) file for details.

## Related Documentation

- [Polymera OS Overview](../../../README.md)
- [Network Services](../README.md)
- [Protocol Buffers](../../../proto/README.md)
- [Bazel Build System](../../../BUILD)

## Support

For questions, issues, or contributions:

1. Check existing issues and documentation
2. Create a new issue with detailed description
3. Join the community discussions
4. Review contribution guidelines

---

**Note**: This is a development version of the DTN Envelope Service. API stability is not guaranteed until version 1.0.0.
