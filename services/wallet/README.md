# Keystore Interface Service for Polymera OS

A unified keystore abstraction that provides comprehensive cryptographic operations with support for post-quantum cryptography, hybrid key schemes, and multiple backend implementations.

## Features

- **Post-Quantum Cryptography**: Support for Dilithium and Kyber algorithms
- **Hybrid Key Schemes**: Ed25519 + PQ combinations for enhanced security
- **Multiple Backends**: Software-based and KeyVault stub implementations
- **Secure Memory Management**: Zeroization and secure erasure of sensitive data
- **Comprehensive Operations**: Sign/verify, encrypt/decrypt, key encapsulation, key derivation
- **Key Lifecycle Management**: Generation, rotation, revocation, expiration
- **Metadata Management**: Tags, names, expiration times, usage tracking
- **Performance Monitoring**: Statistics and operation tracking
- **Session Key Types**: Purpose-bound, time-scoped, and amount-limited session keys with comprehensive validation

## Supported Key Types

### Classical Cryptography
- **Ed25519**: Fast elliptic curve digital signatures
- **AES256**: Advanced Encryption Standard (256-bit)
- **ChaCha20Poly1305**: High-performance authenticated encryption

### Post-Quantum Cryptography
- **Dilithium3**: NIST PQC signature algorithm (Level 3)
- **Dilithium5**: NIST PQC signature algorithm (Level 5)
- **Kyber512**: NIST PQC key encapsulation (Level 1)
- **Kyber768**: NIST PQC key encapsulation (Level 3)
- **Kyber1024**: NIST PQC key encapsulation (Level 5)

### Hybrid Schemes
- **Ed25519Dilithium3**: Ed25519 + Dilithium3 combination
- **Ed25519Kyber512**: Ed25519 + Kyber512 combination

## Key Purposes

- **Sign/Verify**: Digital signatures and verification
- **Encrypt/Decrypt**: Symmetric encryption and decryption
- **KeyEncapsulation/KeyDecapsulation**: Key exchange protocols
- **KeyAgreement**: Key agreement protocols
- **KeyDerivation**: Key derivation from base keys

## Session Key Types

The wallet service includes comprehensive session key types that provide fine-grained access control:

- **Purpose-Bound Sessions**: Authentication, API access, data access, network, device, and custom sessions
- **Time-Based Constraints**: Start/end times, duration limits, time windows, and timezone requirements
- **Amount-Based Constraints**: Operation limits, data volume limits, financial limits, and rate limiting
- **Geographic Constraints**: Country, region, city, and radius-based access control
- **Network Constraints**: IP ranges, network types, domains, and VPN requirements
- **Device Constraints**: Device types, manufacturers, and security feature requirements
- **Data Constraints**: Data types, classifications, and source restrictions

For detailed information about session key types, see [SESSION_README.md](SESSION_README.md).

## Architecture

### Core Components

1. **Keystore Interface** (`keystore.rs`)
   - Defines the unified API for all cryptographic operations
   - Provides type-safe enums for key types and purposes
   - Includes factory pattern for backend creation

2. **Backend Abstraction** (`backend/mod.rs`)
   - Common interface for different backend implementations
   - Error handling and utility functions
   - Secure memory management utilities

3. **Software Backend** (`backend/software.rs`)
   - Local cryptographic operations
   - Secure key storage with encryption
   - Master key management

4. **KeyVault Backend** (`backend/keyvault.rs`)
   - Azure Key Vault integration (stub implementation)
   - Remote key management
   - Cloud-based security

5. **Service Layer** (`lib.rs`)
   - High-level service coordination
   - Backend management
   - Utility functions

6. **Session Key Types** (`session.rs`)
   - Purpose-bound, time-scoped, and amount-limited session keys
   - Comprehensive validation and constraint checking
   - Geographic, network, device, and data access control

### Security Features

- **Secure Memory**: All sensitive data uses `Zeroizing<T>` wrapper
- **Key Encryption**: Key material encrypted with master key
- **Secure Erasure**: Memory zeroization on key deletion
- **Constant-Time Operations**: Security-critical comparisons
- **Access Control**: Purpose-based operation validation

## Installation

### Prerequisites

- Rust 1.70+ with Cargo
- Bazel (for build system integration)
- OpenSSL development libraries

### Using Cargo

```bash
cd services/wallet
cargo build --release
cargo test
```

### Using Bazel

```bash
bazel build //services/wallet:all
bazel test //services/wallet:wallet_tests
```

## Usage

### Basic Operations

```rust
use polymera_wallet::{KeystoreService, KeystoreConfig, KeyType, KeyPurpose};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create service with software backend
    let config = KeystoreConfig::default();
    let service = KeystoreService::new(config).await?;
    
    // Generate signing key
    let metadata = service.generate_key(
        KeyType::Ed25519,
        vec![KeyPurpose::Sign, KeyPurpose::Verify],
        Some("My Signing Key".to_string()),
        None,
        None,
    ).await?;
    
    // Sign data
    let data = b"Hello, World!";
    let signature = service.sign(&metadata.id, data, None).await?;
    
    // Verify signature
    let is_valid = service.verify(&metadata.id, data, &signature.signature, None).await?;
    println!("Signature valid: {}", is_valid);
    
    Ok(())
}
```

### Key Management

```rust
// Generate encryption key
let enc_key = service.generate_key(
    KeyType::AES256,
    vec![KeyPurpose::Encrypt, KeyPurpose::Decrypt],
    Some("Encryption Key".to_string()),
    None,
    None,
).await?;

// Encrypt data
let data = b"Secret message";
let encrypted = service.encrypt(&enc_key.id, data, None).await?;

// Decrypt data
let decrypted = service.decrypt(&enc_key.id, &encrypted).await?;
assert_eq!(decrypted, data);

// Rotate key
let new_key = service.rotate_key(
    &enc_key.id,
    Some(KeyType::ChaCha20Poly1305),
    None,
).await?;

// Revoke old key
service.revoke_key(&enc_key.id, Some("Key rotation")).await?;
```

### Key Encapsulation

```rust
// Generate KEM key
let kem_key = service.generate_key(
    KeyType::Kyber512,
    vec![KeyPurpose::KeyEncapsulation, KeyPurpose::KeyDecapsulation],
    None,
    None,
    None,
).await?;

// Encapsulate key
let result = service.encapsulate_key(&kem_key.id, None).await?;

// Decapsulate key
let shared_secret = service.decapsulate_key(
    &kem_key.id,
    &result.encapsulated_key,
    None,
).await?;

assert_eq!(shared_secret, result.shared_secret);
```

### Key Derivation

```rust
// Generate base key
let base_key = service.generate_key(
    KeyType::AES256,
    vec![KeyPurpose::KeyDerivation],
    None,
    None,
    None,
).await?;

// Derive new key
let salt = vec![1u8; 32];
let params = KeyDerivationParams {
    salt,
    iterations: 100000,
    key_length: 32,
    algorithm: "Argon2id".to_string(),
};

let derived_key = service.derive_key(&base_key.id, &params).await?;
```

## CLI Usage

The service includes a comprehensive command-line interface:

```bash
# Generate a new signing key
polymera-wallet generate --key-type ed25519 --purposes sign,verify --name "My Key"

# List all keys
polymera-wallet list

# Sign data
polymera-wallet sign --key-id <key-id> --data "Hello, World!"

# Verify signature
polymera-wallet verify --key-id <key-id> --data "Hello, World!" --signature-file signature.bin

# Encrypt data
polymera-wallet encrypt --key-id <key-id> --data "Secret message"

# Show statistics
polymera-wallet stats

# Run tests
polymera-wallet test
```

## Configuration

### Keystore Configuration

```yaml
# config/keystore.yaml
backend: "software"  # or "keyvault"
secure_erasure: true
key_rotation_interval: 2592000  # 30 days in seconds
max_key_lifetime: 31536000      # 1 year in seconds
encryption_algorithm: "AES256-GCM"
signature_algorithm: "Ed25519"
key_derivation_algorithm: "Argon2id"
storage_path: "data/keystore"
```

### KeyVault Configuration

```yaml
# config/keyvault.yaml
backend: "keyvault"
connection_string: "https://your-vault.vault.azure.net/"
authentication:
  auth_type: "service_principal"
  credentials:
    tenant_id: "your-tenant-id"
    client_id: "your-client-id"
    client_secret: "your-client-secret"
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test modules
cargo test keystore_tests
cargo test performance_tests

# Run with logging
RUST_LOG=debug cargo test

# Run benchmarks
cargo bench
```

### Test Coverage

The test suite covers:

- **Unit Tests**: Individual component functionality
- **Integration Tests**: End-to-end workflows
- **Performance Tests**: Benchmarking and performance validation
- **Error Handling**: Edge cases and error scenarios
- **Security Tests**: Secure erasure and memory management

### Key Test Scenarios

1. **Complete Lifecycle**: Generate → Use → Rotate → Revoke → Delete
2. **Different Key Types**: All supported cryptographic algorithms
3. **Hybrid Schemes**: Combined classical and post-quantum keys
4. **Key Management**: Metadata, expiration, cleanup
5. **Error Handling**: Invalid operations, missing keys
6. **Performance**: Generation, signing, encryption benchmarks

## Performance

### Benchmarks

Typical performance characteristics (on modern hardware):

- **Key Generation**:
  - Ed25519: ~1ms
  - Dilithium3: ~10ms
  - Kyber512: ~5ms
  - AES256: ~0.1ms

- **Signing**:
  - Ed25519: ~0.5ms (1KB data)
  - Dilithium3: ~5ms (1KB data)

- **Encryption**:
  - AES256-GCM: ~0.1ms (1KB data)
  - ChaCha20Poly1305: ~0.2ms (1KB data)

### Optimization Tips

1. **Key Reuse**: Reuse keys for multiple operations
2. **Batch Operations**: Group related operations
3. **Async Operations**: Use async/await for I/O-bound operations
4. **Memory Management**: Monitor memory usage for large key sets

## Security Considerations

### Best Practices

1. **Key Rotation**: Regularly rotate keys (recommended: 30-90 days)
2. **Access Control**: Limit key access to necessary purposes only
3. **Secure Storage**: Use secure storage backends in production
4. **Audit Logging**: Monitor key usage and access patterns
5. **Key Backup**: Implement secure key backup and recovery

### Threat Model

The service protects against:

- **Memory Attacks**: Secure memory management and zeroization
- **Key Extraction**: Encrypted key storage and access control
- **Unauthorized Use**: Purpose-based operation validation
- **Key Compromise**: Revocation and rotation mechanisms

### Compliance

- **FIPS 140-2**: Cryptographic module validation
- **NIST PQC**: Post-quantum cryptography standards
- **SOC 2**: Security and availability controls
- **GDPR**: Data protection and privacy

## Contributing

### Development Setup

1. Clone the repository
2. Install Rust toolchain
3. Install Bazel (optional)
4. Run tests: `cargo test`
5. Check formatting: `cargo fmt`
6. Run linter: `cargo clippy`

### Code Style

- Follow Rust conventions and idioms
- Use meaningful variable and function names
- Include comprehensive documentation
- Write tests for all new functionality
- Ensure proper error handling

### Testing Guidelines

- Write unit tests for all public functions
- Include integration tests for workflows
- Add performance benchmarks for critical paths
- Test error conditions and edge cases
- Maintain high test coverage (>90%)

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## Related Documentation

- [Polymera OS Architecture](../docs/ARCHITECTURE.md)
- [Security Guidelines](../docs/SECURITY.md)
- [API Reference](../docs/API.md)
- [Deployment Guide](../docs/DEPLOYMENT.md)

## Support

### Getting Help

- **Documentation**: Check this README and related docs
- **Issues**: Report bugs and feature requests via GitHub
- **Discussions**: Join community discussions
- **Security**: Report security issues privately

### Community

- **GitHub**: [Polymera OS Repository](https://github.com/polymera-os)
- **Discord**: Join our community server
- **Matrix**: #polymera-os:matrix.org

---

**Note**: This is a development version of the Keystore Interface Service. API stability is not guaranteed until version 1.0.0.
