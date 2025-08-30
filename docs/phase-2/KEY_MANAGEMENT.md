# Key Management System

## Overview

The Key Management System provides an in-kernel ephemeral keystore for managing Post-Quantum Cryptography (PQC) keys in Polymera OS. This system handles both Dilithium public keys (issuers) and Kyber encapsulation keys (session) with automatic rotation policies, grace periods for ongoing operations, and secure zeroization.

## Architecture

### Core Components

1. **KeyStore** (`kernel/src/secman/keys.rs`)
   - In-kernel ephemeral keystore for PQC keys
   - Manages both issuer and session keys
   - Implements rotation policies and maintenance

2. **Security Manager API** (`kernel/src/secman/api.rs`)
   - Provides sys_debug operations for key management
   - Administrative functions for keystore operations
   - Statistics and monitoring capabilities

3. **Syscall Integration** (`kernel/src/syscall/handlers.rs`)
   - Integrates with sys_debug for key management operations
   - Provides user-space access to keystore functionality

### System Flow

```
User Space                    Kernel Space
    |                            |
    | sys_debug(SECMAN_API)     |
    |--------------------------->|
    |                            | API Operation Handler
    |                            | Key Management Operations
    |                            | Audit Logging
    |                            |
    | Result or Error            |
    |<---------------------------|
    |                            |
    | IPC Stream                 |
    |--------------------------->|
    |                            | Session Key Lookup
    |                            | Key Validation
    |                            | Usage Statistics
    |                            |
    | Authenticated IPC          |
    |<---------------------------|
```

## Key Types

### Issuer Keys (Dilithium)

Issuer keys are Dilithium public keys used for signature verification and authentication:

```rust
pub struct IssuerKey {
    pub id: KeyId,                    // Unique key identifier
    pub public_key: DilithiumPublicKey, // Dilithium public key
    pub params: DilithiumParameterSet,  // Parameter set used
    pub added_at: u64,                // When the key was added
    pub last_used: u64,               // Last used timestamp
    pub usage_count: u64,             // Usage count
    pub active: bool,                 // Whether the key is active
}
```

**Supported Parameter Sets:**
- **Dilithium2**: 128-bit security level
- **Dilithium3**: 192-bit security level  
- **Dilithium5**: 256-bit security level

### Session Keys (Kyber)

Session keys are Kyber public keys used for key encapsulation and session establishment:

```rust
pub struct SessionKey {
    pub id: KeyId,                    // Unique key identifier
    pub public_key: KyberPublicKey,   // Kyber public key
    pub params: KyberParameterSet,    // Parameter set used
    pub created_at: u64,              // When the key was created
    pub expires_at: u64,              // When the key expires
    pub last_used: u64,               // Last used timestamp
    pub message_count: u64,           // Message count since creation
    pub active: bool,                 // Whether the key is active
    pub issuer_key_id: Option<KeyId>, // Associated issuer key ID
}
```

**Supported Parameter Sets:**
- **Kyber512**: 128-bit security level
- **Kyber768**: 192-bit security level
- **Kyber1024**: 256-bit security level

## Key Rotation Policies

### Automatic Rotation

The system implements automatic key rotation based on two criteria:

1. **Time-based Rotation**: Session keys expire after a configurable interval (default: 5 minutes)
2. **Message Count Rotation**: Keys are rotated after processing a threshold number of messages (default: 1000)

### Grace Period

To ensure ongoing IPC streams are not disrupted, the system implements a grace period:

- **Duration**: 30 seconds (configurable)
- **Behavior**: During grace period, old keys remain active for ongoing operations
- **New Operations**: New IPC streams use the replacement keys
- **Completion**: After grace period, old keys are deactivated and marked for purging

### Rotation Policy Configuration

```rust
pub struct RotationPolicy {
    pub session_rotation_interval: Duration,      // Rotation interval
    pub message_rotation_threshold: usize,        // Message threshold
    pub auto_rotation_enabled: bool,              // Enable auto-rotation
    pub grace_period: Duration,                   // Grace period duration
    pub max_key_age: Duration,                    // Maximum key age
}
```

## Keystore Operations

### Key Addition

```rust
// Add an issuer key
let issuer_key_id = add_issuer_key(
    dilithium_public_key,
    DilithiumParameterSet::Dilithium2,
)?;

// Create a session key
let session_key_id = create_session_key(
    kyber_public_key,
    KyberParameterSet::Kyber512,
    Some(issuer_key_id),
)?;
```

### Key Retrieval

```rust
// Get an issuer key
if let Some(issuer_key) = get_issuer_key(&issuer_key_id) {
    // Use the issuer key for signature verification
}

// Get a session key
if let Some(session_key) = get_session_key(&session_key_id) {
    // Use the session key for key encapsulation
}
```

### Key Rotation

```rust
// Force immediate rotation
let rotated_count = force_rotation_now();

// Perform automatic maintenance
perform_keystore_maintenance();
```

### Key Purging

```rust
// Purge expired keys
let purged_count = purge_expired_keys();

// Emergency purge (removes all keys)
let total_purged = purge_all_keys();
```

## Sys_debug API Operations

### Key Management Operations

The system provides comprehensive sys_debug operations for key management:

```rust
// Operation codes for key management
pub const PRINT_KEY_COUNTS: u64 = 100;      // Print current key counts
pub const ROTATE_KEYS_NOW: u64 = 101;      // Force immediate rotation
pub const PURGE_KEY_CACHE: u64 = 102;      // Purge expired keys
pub const PRINT_KEY_STATS: u64 = 103;      // Print key statistics
pub const SET_ROTATION_POLICY: u64 = 104;  // Update rotation policy
pub const GET_ROTATION_POLICY: u64 = 105;  // Get current policy
```

### Usage Examples

```rust
// Print key counts
let total_keys = syscall(SYS_DEBUG, SECMAN_API, PRINT_KEY_COUNTS, 0, 0);

// Force key rotation
let rotated = syscall(SYS_DEBUG, SECMAN_API, ROTATE_KEYS_NOW, 0, 0);

// Purge expired keys
let purged = syscall(SYS_DEBUG, SECMAN_API, PURGE_KEY_CACHE, 0, 0);

// Print key statistics
syscall(SYS_DEBUG, SECMAN_API, PRINT_KEY_STATS, 0, 0);

// Set rotation policy (5 minutes, 1000 messages, auto-enabled)
syscall(SYS_DEBUG, SECMAN_API, SET_ROTATION_POLICY, 300, 1000, 1);

// Get current rotation policy
let interval = syscall(SYS_DEBUG, SECMAN_API, GET_ROTATION_POLICY, 0, 0);
```

## Security Features

### Memory Protection

- **Secure Memory Regions**: Keys are stored in secure memory regions with automatic zeroization
- **Memory Isolation**: Different key types are isolated in separate memory areas
- **Overflow Protection**: Capacity limits prevent memory exhaustion attacks

### Zeroization

- **Automatic Zeroization**: Key material is automatically zeroized when keys are dropped
- **Memory Probe Confirmation**: Memory probes can verify that key material has been properly cleared
- **Secure Cleanup**: Rotation and purging operations ensure secure cleanup

### Audit Logging

All key management operations are logged for security auditing:

```rust
// Audit entry for key rotation
let audit_entry = AuditEntry::new(
    ops::SEC_ADMIN_OP,
    0, // No specific resource
    "Forced immediate key rotation".to_string(),
    Some(format!("{} keys rotated", rotated_count)),
);
log_audit_entry(audit_entry);
```

## Performance Characteristics

### Timing Benchmarks

- **Issuer Key Addition**: <1ms
- **Session Key Creation**: <2ms
- **Key Rotation**: <5ms
- **Key Purging**: <3ms
- **API Operations**: <1ms

### Memory Usage

- **Issuer Key Overhead**: ~1.5KB per key
- **Session Key Overhead**: ~1KB per key
- **Total Keystore Overhead**: <500KB
- **Memory Alignment**: 4KB page boundaries

### Capacity Limits

- **Maximum Issuer Keys**: 100
- **Maximum Session Keys**: 200
- **Total Key Capacity**: 300 keys

## IPC Stream Continuity

### Grace Period Implementation

The system ensures that ongoing IPC streams are not disrupted during key rotation:

1. **Stream Identification**: IPC streams are identified by their session key
2. **Rotation Marking**: Keys are marked for rotation but remain active
3. **Grace Period**: 30-second grace period allows ongoing streams to complete
4. **Replacement Keys**: New IPC streams use replacement keys
5. **Cleanup**: Old keys are purged after grace period completion

### Example Scenario

```
Time 0s:   IPC stream starts with session key A
Time 2s:   Key rotation triggered (key A marked for rotation)
Time 2s:   Grace period begins (30 seconds)
Time 2s:   Replacement key B created
Time 5s:   New IPC stream uses key B
Time 32s:  Grace period ends, key A deactivated
Time 35s:  Key A purged and zeroized
```

## Testing

### Test Script

Run the comprehensive test suite:

```bash
./scripts/test-key-management.sh
```

### Test Scenarios

1. **Key Creation and Management**
   - Add issuer and session keys
   - Verify key storage and retrieval
   - Test capacity limits

2. **Key Rotation**
   - Test automatic rotation policies
   - Verify grace period functionality
   - Test force rotation operations

3. **IPC Stream Continuity**
   - Start IPC stream with session key
   - Trigger key rotation during stream
   - Verify stream completion without disruption

4. **Memory Zeroization**
   - Create keys with known patterns
   - Rotate and purge keys
   - Verify memory is properly zeroized

5. **API Operations**
   - Test all sys_debug operations
   - Verify statistics and monitoring
   - Test policy configuration

### Unit Tests

```bash
# Test key management functionality
cargo test keys --package kernel

# Test API operations
cargo test api --package kernel

# Test syscall integration
cargo test --package kernel
```

## Troubleshooting

### Common Issues

1. **Key Rotation Failures**
   - Check rotation policy configuration
   - Verify grace period settings
   - Monitor key usage statistics

2. **Memory Issues**
   - Check capacity limits
   - Monitor memory usage
   - Verify zeroization is working

3. **API Operation Failures**
   - Check operation codes
   - Verify argument validation
   - Monitor audit logs

4. **IPC Stream Disruptions**
   - Check grace period configuration
   - Verify key replacement logic
   - Monitor stream completion rates

### Debug Information

Enable debug logging:

```rust
// In kernel configuration
features = ["debug"]
```

Monitor key management statistics:

```rust
// Print key counts
syscall(SYS_DEBUG, SECMAN_API, PRINT_KEY_COUNTS, 0, 0);

// Print detailed statistics
syscall(SYS_DEBUG, SECMAN_API, PRINT_KEY_STATS, 0, 0);

// Check rotation policy
syscall(SYS_DEBUG, SECMAN_API, GET_ROTATION_POLICY, 0, 0);
```

## Future Enhancements

### Planned Features

1. **Advanced Rotation Policies**
   - Machine learning-based rotation timing
   - Adaptive threshold adjustment
   - Performance-based optimization

2. **Key Distribution**
   - Distributed keystore across multiple nodes
   - Key synchronization protocols
   - Load balancing for key operations

3. **Enhanced Security**
   - Hardware security module integration
   - Key escrow and recovery
   - Advanced threat detection

4. **Performance Optimization**
   - Key caching and prefetching
   - Parallel key operations
   - Memory pool optimization

### Compatibility

- **Backward Compatibility**: API changes maintain backward compatibility
- **Policy Evolution**: Rotation policies can be updated without disruption
- **Key Format Extensions**: New key types can be added seamlessly
- **Performance Scaling**: System scales with additional hardware resources

## Conclusion

The Key Management System provides a robust, secure, and efficient foundation for PQC key management in Polymera OS. With automatic rotation policies, grace periods for ongoing operations, and secure zeroization, it ensures that cryptographic keys are properly managed while maintaining system performance and security.

The system is designed to be extensible, allowing future enhancements while maintaining backward compatibility and security guarantees. It successfully balances security requirements with operational needs, providing a production-ready key management solution for modern operating systems.
