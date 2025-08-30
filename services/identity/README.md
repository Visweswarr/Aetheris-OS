# DID Identity Service for Polymera OS

A Decentralized Identifier (DID) service that provides post-quantum cryptography support, hybrid key schemes, and local DID document storage for Polymera OS.

## Features

- **Post-Quantum Cryptography**: Support for Dilithium and Kyber algorithms
- **Hybrid Key Schemes**: Ed25519 + PQ combinations for enhanced security
- **Local DID Storage**: File-backed registry with persistent storage
- **Key Rotation**: Secure key rotation with automatic revocation
- **DID Resolution**: Fast local DID document resolution
- **Multiple Key Purposes**: Authentication, assertion, key agreement, and more
- **Comprehensive Testing**: Unit tests, integration tests, and performance benchmarks

## Architecture

The DID Identity service consists of several key components:

### Core Components

- **`DidIdentity`**: Main DID operations (create, resolve, rotate keys)
- **`DidStore`**: Local file-backed storage with Sled database
- **`IdentityService`**: High-level service coordination
- **`DidDocument`**: W3C DID document structure
- **`DidKey`**: Cryptographic key management

### Supported Key Types

1. **Classical Cryptography**
   - **Ed25519**: Fast, secure digital signatures

2. **Post-Quantum Cryptography**
   - **Dilithium3**: NIST PQC signature algorithm (Level 3)
   - **Dilithium5**: NIST PQC signature algorithm (Level 5)
   - **Kyber512**: NIST PQC key encapsulation (Level 1)
   - **Kyber768**: NIST PQC key encapsulation (Level 3)
   - **Kyber1024**: NIST PQC key encapsulation (Level 5)

3. **Hybrid Schemes**
   - **Ed25519 + Dilithium3**: Classical + PQ signature combination
   - **Ed25519 + Kyber512**: Classical signature + PQ KEM combination

### Key Purposes

- **Authentication**: Proving identity
- **Assertion**: Making claims
- **Key Agreement**: Establishing shared secrets
- **Key Encapsulation**: Secure key transport
- **Capability Invocation**: Executing actions
- **Capability Delegation**: Granting permissions

## Installation

### Prerequisites

- Rust 1.70+
- Bazel 6.0+

### Building with Bazel

```bash
# Build the library
bazel build //services/identity:polymera_identity

# Build the binary
bazel build //services/identity:polymera_identity_bin

# Run tests
bazel test //services/identity:polymera_identity_test_suite

# Run benchmarks
bazel run //services/identity:identity_benchmarks
```

### Building with Cargo

```bash
cd services/identity

# Build
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

## Usage

### Command Line Interface

The DID identity service provides a comprehensive CLI for management:

```bash
# Create a new DID
polymera-identity create \
  --method polynet \
  --key-types ed25519,dilithium3 \
  --purposes authentication,assertion

# Resolve a DID
polymera-identity resolve --did did:polynet:abc123

# Rotate keys
polymera-identity rotate \
  --did did:polynet:abc123 \
  --key-ids key1,key2 \
  --new-key-types kyber512,dilithium5 \
  --new-purposes authentication,key-agreement

# List all DIDs
polymera-identity list

# Get DID keys
polymera-identity keys --did did:polynet:abc123

# Deactivate a DID
polymera-identity deactivate \
  --did did:polynet:abc123 \
  --reason "Testing deactivation"

# Show service statistics
polymera-identity stats

# Run basic tests
polymera-identity test
```

### Programmatic Usage

```rust
use polymera_identity::{IdentityService, KeyType, KeyPurpose};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create service
    let service = IdentityService::new("data/identity".into()).await?;
    
    // Create DID with post-quantum keys
    let (did, doc) = service.create_did(
        "polynet",
        vec![KeyType::Ed25519, KeyType::Dilithium3],
        vec![KeyPurpose::Authentication, KeyPurpose::Assertion],
    ).await?;
    
    println!("Created DID: {}", did);
    
    // Resolve DID
    let resolved = service.resolve_did(&did).await?;
    assert!(resolved.is_some());
    
    // Get keys
    let keys = service.get_did_keys(&did).await?;
    println!("DID has {} keys", keys.len());
    
    // Rotate keys
    let key_ids: Vec<String> = keys.iter().map(|k| k.id.clone()).collect();
    let updated_doc = service.rotate_keys(
        &did,
        key_ids,
        vec![KeyType::Kyber512],
        vec![KeyPurpose::Authentication],
    ).await?;
    
    println!("Rotated keys, new version: {}", updated_doc.version_id);
    
    Ok(())
}
```

## Configuration

### Store Configuration

```rust
use polymera_identity::StoreConfig;

let config = StoreConfig {
    max_dids: 10000,
    max_keys_per_did: 100,
    cleanup_interval: 3600, // 1 hour
    persistence_path: "data/did_store".into(),
};
```

### Service Configuration

```rust
use polymera_identity::IdentityServiceConfig;

let service_config = IdentityServiceConfig {
    store_path: "data/identity".into(),
    max_dids: 10000,
    max_keys_per_did: 100,
    cleanup_interval: 3600,
};
```

## API Reference

### Core Types

#### DidDocument

```rust
pub struct DidDocument {
    pub id: String,
    pub controller: Option<String>,
    pub verification_methods: Vec<VerificationMethod>,
    pub authentication: Vec<String>,
    pub assertion_method: Vec<String>,
    pub key_agreement: Vec<String>,
    pub key_encapsulation: Vec<String>,
    pub capability_invocation: Vec<String>,
    pub capability_delegation: Vec<String>,
    pub services: Vec<Service>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub version_id: String,
    pub next_update: Option<DateTime<Utc>>,
    pub proof: Option<Proof>,
}
```

#### DidKey

```rust
pub struct DidKey {
    pub id: String,
    pub key_type: KeyType,
    pub public_key: Vec<u8>,
    pub private_key: Option<Vec<u8>>,
    pub created: DateTime<Utc>,
    pub expires: Option<DateTime<Utc>>,
    pub revoked: bool,
    pub purpose: Vec<KeyPurpose>,
}
```

#### VerificationMethod

```rust
pub struct VerificationMethod {
    pub id: String,
    pub key_type: String,
    pub controller: String,
    pub public_key_multibase: Option<String>,
}
```

### Service Methods

#### Core Operations

- `create_did(method: &str, key_types: Vec<KeyType>, purposes: Vec<KeyPurpose>) -> Result<(String, DidDocument), Error>`
- `resolve_did(did: &str) -> Result<Option<DidDocument>, Error>`
- `rotate_keys(did: &str, key_ids: Vec<String>, new_key_types: Vec<KeyType>, new_purposes: Vec<KeyPurpose>) -> Result<DidDocument, Error>`
- `get_did_keys(did: &str) -> Result<Vec<DidKey>, Error>`
- `deactivate_did(did: &str, reason: Option<String>) -> Result<(), Error>`

#### Management Operations

- `list_dids() -> Result<Vec<String>, Error>`
- `get_stats() -> Result<ServiceStats, Error>`
- `cleanup_expired() -> Result<usize, Error>`

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test modules
cargo test did_identity_tests
cargo test performance_tests

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
- **Key Scenarios**: Create → resolve → rotate key workflow
- **Error Handling**: Invalid inputs and edge cases
- **Hybrid Keys**: Post-quantum cryptography combinations

### Key Test Scenarios

1. **DID Lifecycle**: Complete create → resolve → rotate key workflow
2. **Key Types**: All supported cryptographic algorithms
3. **Hybrid Schemes**: Ed25519 + PQ combinations
4. **Key Rotation**: Single and multiple key rotation scenarios
5. **Error Handling**: Invalid DIDs, missing keys, storage errors
6. **Performance**: DID creation, resolution, and key rotation benchmarks

## Performance

### Benchmarks

The service includes performance benchmarks for:

- **DID Creation**: ~100 DIDs/second
- **Key Generation**: ~50 post-quantum keys/second
- **DID Resolution**: ~1000 resolutions/second
- **Key Rotation**: ~10 rotations/second

### Optimization Tips

1. **Batch Operations**: Group multiple DID operations
2. **Async Operations**: Use async/await for I/O-bound operations
3. **Memory Management**: Configure appropriate storage limits
4. **Cleanup Frequency**: Adjust cleanup intervals based on usage patterns

## Security

### Cryptographic Features

- **Post-Quantum Ready**: Dilithium and Kyber algorithms
- **Hybrid Security**: Classical + PQ combinations
- **Key Rotation**: Secure key replacement with revocation
- **Purpose Binding**: Keys bound to specific use cases
- **Local Storage**: No external dependencies for key management

### Security Levels

- **Classical**: Ed25519 signatures (fast, secure)
- **Post-Quantum**: Dilithium signatures, Kyber KEM (future-proof)
- **Hybrid**: Ed25519 + PQ combinations (maximum security)

## Storage

### Local File-Backed Registry

The service uses a local file-backed registry for development:

- **Sled Database**: Embedded key-value store
- **Persistent Storage**: Automatic persistence to disk
- **Memory Caching**: Fast in-memory access with disk backup
- **Automatic Cleanup**: Expired key cleanup and maintenance

### Data Structure

```
data/identity/
├── did_documents/     # DID document storage
├── did_keys/         # Cryptographic key storage
├── did_metadata/     # DID metadata and status
├── revoked_keys/     # Revoked key tracking
└── deactivated_dids/ # Deactivated DID records
```

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
- [Identity Services](../README.md)
- [Post-Quantum Cryptography](../../../docs/PQC.md)
- [Bazel Build System](../../../BUILD)

## Support

For questions, issues, or contributions:

1. Check existing issues and documentation
2. Create a new issue with detailed description
3. Join the community discussions
4. Review contribution guidelines

---

**Note**: This is a development version of the DID Identity Service. API stability is not guaranteed until version 1.0.0.
