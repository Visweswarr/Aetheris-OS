# Development Keyvault Integration

## Overview

The Development Keyvault Integration provides kernel hooks to load issuer anchors and session metadata from a file-backed persistent storage system at boot time. This system is designed for development and testing purposes only and should **NOT** be used in production environments.

## Architecture

### Components

1. **Development Keyvault Service** (`services/keyvault-dev/`)
   - `crypto.rs`: AES-GCM encryption/decryption for data at rest
   - `store.rs`: File-backed persistence for issuer anchors and session metadata
   - `lib.rs`: Main service interface and orchestration

2. **Kernel Integration Module** (`kernel/src/secman/dev_keyvault.rs`)
   - Compile-time feature-gated integration
   - Boot-time loading of anchors and session data
   - Integration with DID resolver and keystore

3. **Key Rotation Tool** (`tools/rotate-dev-vault-key.rs`)
   - Command-line interface for managing the development vault master key
   - Backup, restore, and rotation operations

### Data Flow

```
Boot Process:
1. Kernel initializes with dev-keyvault feature
2. Security Manager loads development keyvault integration
3. Integration loads issuer anchors from persistent storage
4. Integration loads session metadata from persistent storage
5. Data is registered with DID resolver and keystore
6. Integration reports statistics and completion status
```

## Features

### Compile-Time Feature Flag

The integration is controlled by the `dev-keyvault` feature flag in `kernel/Cargo.toml`:

```toml
[features]
dev-keyvault = []
```

### Issuer Anchor Persistence

- **Storage**: File-backed with AES-GCM encryption
- **Data**: DID documents, public keys, trust anchors, TTL values
- **Format**: JSON with encrypted payload
- **Location**: Configurable base directory (default: `./dev-keyvault/`)

### Session Metadata Persistence

- **Storage**: File-backed with AES-GCM encryption
- **Data**: Session keys, expiration times, usage statistics
- **Format**: JSON with encrypted payload
- **Grace Periods**: Maintained across reboots

### Key Rotation

- **Master Key**: AES-256-GCM key for vault encryption
- **Rotation**: Secure key rotation with backup/restore
- **Tools**: Command-line interface for key management

## Usage

### Enabling the Integration

1. **Build the kernel with the feature flag:**
   ```bash
   cd kernel
   cargo build --features dev-keyvault --release
   ```

2. **The integration automatically initializes at boot time**

### Key Rotation Operations

```bash
# Rotate the master key
cargo run --bin rotate-dev-vault-key -- rotate

# Create a backup
cargo run --bin rotate-dev-vault-key -- backup

# Restore from backup
cargo run --bin rotate-dev-vault-key -- restore --backup-path ./backup/

# Export the current key
cargo run --bin rotate-dev-vault-key -- export --output ./current-key.hex

# Import a key
cargo run --bin rotate-dev-vault-key -- import --input ./new-key.hex

# Check status
cargo run --bin rotate-dev-vault-key -- status
```

### Configuration

The development keyvault can be configured through environment variables:

```bash
# Base directory for vault storage
export DEV_KEYVAULT_BASE_DIR="./dev-keyvault"

# Master key file path
export DEV_KEYVAULT_MASTER_KEY="./dev-keyvault/master.key"

# Enable debug logging
export DEV_KEYVAULT_DEBUG=1
```

## Security Considerations

### Development-Only Warning

⚠️ **CRITICAL**: This system is designed for development and testing only. It should never be used in production environments.

### Encryption at Rest

- **Algorithm**: AES-256-GCM
- **Key Derivation**: Random generation with secure entropy
- **IV**: Unique per encryption operation
- **Authentication**: GCM provides authenticated encryption

### Key Management

- **Master Key**: Stored in separate file with restricted permissions
- **Rotation**: Secure rotation with backup/restore procedures
- **Access Control**: File system permissions control access

### Data Validation

- **Input Validation**: All data is validated before processing
- **Parameter Sets**: Only supported PQC parameter sets are accepted
- **TTL Enforcement**: Time-based expiration is enforced

## Testing

### Integration Tests

```bash
# Run the test script
chmod +x scripts/test-dev-keyvault.sh
./scripts/test-dev-keyvault.sh
```

### CI/CD Integration

The system includes GitHub Actions workflows that:
- Build the kernel with dev-keyvault feature
- Test the integration initialization
- Verify anchor and session loading
- Check statistics reporting
- Test reboot continuity (simulated)

### Test Coverage

- ✅ Service compilation and linking
- ✅ Kernel integration initialization
- ✅ Issuer anchor loading and registration
- ✅ Session metadata loading and registration
- ✅ Integration statistics and reporting
- ✅ Reboot continuity simulation
- ✅ Session grace period maintenance

## Troubleshooting

### Common Issues

1. **Integration not initializing**
   - Check that `dev-keyvault` feature is enabled
   - Verify the kernel builds successfully
   - Check kernel logs for initialization errors

2. **Anchors not loading**
   - Verify the development keyvault service is running
   - Check file permissions on vault directory
   - Verify master key file exists and is readable

3. **Session grace periods not maintained**
   - Check that session metadata is being persisted
   - Verify TTL values are being loaded correctly
   - Check for clock synchronization issues

### Debug Mode

Enable debug logging by setting the environment variable:

```bash
export DEV_KEYVAULT_DEBUG=1
```

This will provide detailed logging of:
- Integration initialization steps
- Anchor and session loading operations
- Registration with kernel subsystems
- Error conditions and recovery attempts

### Log Analysis

Key log messages to monitor:

```
[DEV-KEYVAULT] Initializing development keyvault integration...
[DEV-KEYVAULT] Loading issuer anchors...
[DEV-KEYVAULT] Loading session metadata...
[DEV-KEYVAULT] Integration initialized successfully
[DEV-KEYVAULT] Loaded X issuer anchors
[DEV-KEYVAULT] Loaded Y session metadata entries
```

## Performance

### Boot Time Impact

- **Initialization**: ~10-50ms depending on data volume
- **Anchor Loading**: ~1-5ms per anchor
- **Session Loading**: ~1-3ms per session
- **Total Impact**: Typically <100ms for development workloads

### Memory Usage

- **Integration State**: ~1KB
- **Per Anchor**: ~2-4KB depending on key size
- **Per Session**: ~1-2KB depending on metadata size
- **Total**: Typically <1MB for development workloads

### Storage Requirements

- **Issuer Anchors**: ~2-5KB per anchor
- **Session Metadata**: ~1-3KB per session
- **Encryption Overhead**: ~16 bytes per encrypted block
- **Total**: Typically <10MB for development workloads

## Future Enhancements

### Planned Features

1. **Real-time Updates**: Live reloading of vault changes
2. **Audit Logging**: Comprehensive audit trail of operations
3. **Performance Metrics**: Detailed performance monitoring
4. **Health Checks**: Automated health monitoring and reporting

### Integration Improvements

1. **Service Discovery**: Automatic detection of running services
2. **Configuration Management**: Centralized configuration system
3. **Error Recovery**: Automatic recovery from common failures
4. **Monitoring**: Integration with system monitoring tools

## API Reference

### Core Functions

```rust
// Initialize the integration
pub fn init_dev_keyvault_integration() -> Result<(), DevKeyVaultError>

// Get integration statistics
pub fn get_dev_keyvault_stats() -> Option<DevKeyVaultStats>

// Test the integration
pub fn test_dev_keyvault_integration()

// Print statistics
pub fn print_dev_keyvault_stats()
```

### Data Structures

```rust
pub struct DevKeyVaultIntegration {
    pub initialized: bool,
    pub issuer_anchors_loaded: usize,
    pub session_metadata_loaded: usize,
}

pub struct DevKeyVaultStats {
    pub initialized: bool,
    pub issuer_anchors_loaded: usize,
    pub session_metadata_loaded: usize,
}
```

### Error Types

```rust
pub enum DevKeyVaultError {
    ServiceNotAvailable,
    FailedToLoadIssuerAnchor(String),
    FailedToLoadSessionMetadata(String),
    InvalidPublicKeyData,
    UnsupportedParameterSet(String),
}
```

## Conclusion

The Development Keyvault Integration provides a robust foundation for development and testing of Polymera OS's security features. It ensures that issuer anchors and session metadata persist across reboots while maintaining security through encryption at rest.

The system is designed to be lightweight, secure, and easy to use in development environments, while providing comprehensive testing and validation capabilities through CI/CD integration.

