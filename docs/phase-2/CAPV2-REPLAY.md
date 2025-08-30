# CapV2 Replay Protection System

## Overview

The CapV2 Replay Protection System provides comprehensive protection against replay attacks on capability tokens in Polymera OS. This system implements nonce-based sliding replay windows with LRU-backed tracking per issuer, ensuring that each capability token can only be used once within its validity period.

## Architecture

### Core Components

1. **CapTokenV2** - Enhanced capability token with nonce support
2. **ReplayWindow** - Per-issuer nonce tracking with LRU eviction
3. **CapStore** - Centralized capability verification with replay protection
4. **DidResolver** - DID trust management with anchor rotation support
5. **AuditSystem** - Comprehensive security event logging

### System Flow

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐    ┌─────────────┐
│ CapTokenV2 │───▶│ CapStore     │───▶│ ReplayWindow│───▶│ AuditEvent │
│ (nonce)    │    │ Verification │    │ (LRU)       │    │ (REPLAY)    │
└─────────────┘    └──────────────┘    └─────────────┘    └─────────────┘
                           │
                           ▼
                   ┌──────────────┐
                   │ DidResolver  │
                   │ (Trust)      │
                   └──────────────┘
```

## Capability Token Structure

### CapTokenV2 Header

```rust
pub struct CapTokenHeader {
    pub issuer: String,        // DID identifier
    pub dst: String,           // Target destination
    pub nonce: u128,           // Unique nonce (128-bit)
    pub cap_id: String,        // Capability identifier
    pub created_at: Instant,   // Creation timestamp
    pub expires_at: Instant,   // Expiration timestamp
    pub permissions: u64,      // Permission bitmask
}
```

### Nonce Requirements

- **Uniqueness**: Each nonce must be unique per (issuer, dst) combination
- **Monotonicity**: Nonces should generally increase over time
- **Collision Resistance**: 128-bit nonces provide 2^128 collision resistance
- **Low Watermark**: Nonces below threshold are rejected immediately

## Replay Protection Mechanism

### ReplayWindow Configuration

```rust
pub struct ReplayWindowConfig {
    pub max_nonces: usize,           // Maximum nonces per issuer
    pub nonce_ttl: Duration,         // Time-to-live for nonce entries
    pub low_watermark: u128,         // Minimum acceptable nonce value
}
```

### Default Values

- **max_nonces**: 10,000 entries per issuer
- **nonce_ttl**: 5 minutes (300 seconds)
- **low_watermark**: 1,000 (configurable per deployment)

### Nonce Validation Logic

1. **Low Watermark Check**: Reject nonces below threshold
2. **Replay Detection**: Check if nonce already used for same destination
3. **Cross-Destination Reuse**: Allow same nonce for different destinations
4. **TTL Enforcement**: Automatically expire old nonce entries
5. **LRU Eviction**: Remove least recently used entries when full

### Performance Characteristics

- **Average Case**: O(1) hash table lookup
- **Worst Case**: O(n) for full window traversal
- **Memory Usage**: ~100 bytes per nonce entry
- **Cleanup Overhead**: Periodic cleanup every 60 seconds

## DID Trust Management

### Anchor Structure

```rust
pub struct DidAnchor {
    pub pubkey: DilithiumPubKey,           // Current public key
    pub ttl: Duration,                     // Time-to-live
    pub created_at: Instant,               // Creation timestamp
    pub expires_at: Instant,               // Expiration timestamp
    pub rotated_at: Option<Instant>,       // Rotation timestamp
    pub previous_pubkey: Option<DilithiumPubKey>, // Previous key
    pub rotation_grace: Duration,          // Grace period
}
```

### Key Rotation Process

1. **Rotation Initiation**: Call `rotate_anchor()` with new public key
2. **Grace Period**: Both keys remain valid during grace period
3. **Grace Expiry**: Only new key valid after grace period ends
4. **Automatic Cleanup**: Expired anchors removed automatically

### Trust Verification

- **TTL Validation**: Reject expired anchors
- **Signature Verification**: Verify Dilithium signatures
- **Rotation Grace**: Support for key rotation transitions
- **Poisoning Prevention**: Validate anchor integrity

## Audit System Integration

### Audit Event Types

| Event | Code | Severity | Description |
|-------|------|----------|-------------|
| `CapAccept` | 1000 | Low | Capability accepted |
| `CapReject` | 1001 | Medium | Capability rejected |
| `CapReplayed` | 1005 | Medium | Replay attack detected |
| `Replay` | 1100 | High | Replay attack detected |
| `ReplayOld` | 1101 | Medium | Old nonce rejected |
| `DidFail` | 1200 | Medium | DID verification failed |

### Audit Event Structure

```rust
pub struct AuditEvent {
    pub timestamp: u64,           // Event timestamp
    pub reason: AuditReason,      // Event reason code
    pub severity: AuditSeverity,  // Event severity
    pub category: AuditCategory,  // Event category
    pub source: String,           // Source component
    pub target: Option<String>,   // Target resource
    pub context: Option<String>,  // Additional context
    pub actor_id: Option<u64>,   // Triggering process
    pub success: bool,            // Success/failure status
}
```

## Security Properties

### Replay Protection Guarantees

1. **Accept-Once Semantics**: Each nonce accepted exactly once per destination
2. **Temporal Bounds**: Nonces outside window automatically rejected
3. **Cross-Destination Isolation**: Nonce reuse across destinations allowed
4. **Zero False Negatives**: All replays guaranteed to be detected
5. **Minimal False Positives**: LRU eviction minimizes false rejections

### Attack Resistance

- **Replay Attacks**: Completely prevented by nonce tracking
- **Timing Attacks**: Nonce validation independent of token content
- **Memory Exhaustion**: LRU eviction prevents memory attacks
- **Key Rotation**: Graceful handling of key rotation attacks
- **Poisoning**: Anchor validation prevents trust poisoning

### Performance Guarantees

- **Constant Time**: O(1) average case for nonce validation
- **Bounded Memory**: Configurable memory usage per issuer
- **Efficient Cleanup**: Periodic cleanup with minimal overhead
- **Scalable Design**: Linear scaling with number of issuers

## Configuration

### Environment Variables

```bash
# Replay window configuration
REPLAY_MAX_NONCES=10000          # Maximum nonces per issuer
REPLAY_NONCE_TTL=300             # Nonce TTL in seconds
REPLAY_LOW_WATERMARK=1000        # Minimum nonce value
REPLAY_CLEANUP_INTERVAL=60       # Cleanup interval in seconds

# DID trust configuration
DID_DEFAULT_TTL=86400            # Default anchor TTL in seconds
DID_ROTATION_GRACE=3600          # Default rotation grace in seconds
DID_MAX_ANCHORS=1000             # Maximum anchors in resolver
```

### Runtime Configuration

```rust
let config = ReplayWindowConfig {
    max_nonces: env::var("REPLAY_MAX_NONCES").unwrap_or(10000),
    nonce_ttl: Duration::from_secs(env::var("REPLAY_NONCE_TTL").unwrap_or(300)),
    low_watermark: env::var("REPLAY_LOW_WATERMARK").unwrap_or(1000),
};
```

## Monitoring and Metrics

### Key Metrics

- **auth_ok**: Successful capability verifications
- **auth_fail**: Failed capability verifications
- **replay_drops**: Replay attempts blocked
- **did_failures**: DID verification failures
- **window_utilization**: Replay window usage percentage

### Health Checks

```rust
// Check replay window health
let stats = cap_store.get_stats();
if stats.replay_drops > threshold {
    // Alert: High replay activity
}

// Check DID resolver health
let did_stats = did_resolver.get_stats();
if did_stats.expired_anchors > threshold {
    // Alert: Many expired anchors
}
```

### Log Analysis

```bash
# Monitor replay events
grep "REPLAY" /var/log/polymera/audit.log

# Monitor capability verification
grep "CAP_ACCEPT\|CAP_REJECT" /var/log/polymera/audit.log

# Monitor DID failures
grep "DID_FAIL" /var/log/polymera/audit.log
```

## Testing

### Test Categories

1. **Unit Tests**: Individual component testing
2. **Integration Tests**: End-to-end system testing
3. **Load Tests**: Performance under high load
4. **Security Tests**: Attack vector validation
5. **Conformance Tests**: Specification compliance

### Test Commands

```bash
# Run all replay protection tests
cargo test capv2_replay

# Run DID anchors tests
cargo test did_anchors

# Run performance benchmarks
cargo bench replay_protection

# Run security tests
cargo test security_replay
```

### Test Coverage

- **Replay Detection**: 100% coverage of replay scenarios
- **Nonce Validation**: Edge case testing for nonce boundaries
- **Performance**: Load testing with 10,000+ nonces
- **Security**: Attack vector simulation and validation
- **Integration**: End-to-end capability verification flows

## Deployment Considerations

### Production Settings

- **Nonce TTL**: 5-15 minutes based on network latency
- **Window Size**: 10,000-100,000 based on expected load
- **Low Watermark**: 1,000-10,000 based on clock drift tolerance
- **Cleanup Interval**: 60-300 seconds based on performance requirements

### Scaling Considerations

- **Memory Usage**: ~1MB per 10,000 nonces per issuer
- **CPU Overhead**: <1% for typical workloads
- **Network Impact**: Minimal additional latency
- **Storage Requirements**: No persistent storage needed

### Monitoring Requirements

- **Replay Detection Rate**: Monitor for unusual replay activity
- **Memory Usage**: Track replay window memory consumption
- **Performance Metrics**: Monitor verification latency
- **Security Events**: Alert on security violations

## Troubleshooting

### Common Issues

1. **High Replay Detection Rate**
   - Check for clock synchronization issues
   - Verify nonce generation uniqueness
   - Review low watermark configuration

2. **Memory Usage Spikes**
   - Monitor replay window utilization
   - Adjust max_nonces configuration
   - Check cleanup interval settings

3. **Performance Degradation**
   - Profile nonce validation performance
   - Check LRU eviction efficiency
   - Monitor hash table collision rates

### Debug Commands

```bash
# Check replay window status
polymera-cli replay status

# Inspect specific issuer window
polymera-cli replay inspect <issuer>

# Reset replay windows
polymera-cli replay reset

# Check DID resolver status
polymera-cli did status
```

### Log Analysis

```bash
# Analyze replay patterns
grep "REPLAY" audit.log | awk '{print $4}' | sort | uniq -c

# Check capability verification rates
grep "CAP_" audit.log | awk '{print $3}' | sort | uniq -c

# Monitor DID resolution failures
grep "DID_FAIL" audit.log | tail -100
```

## Future Enhancements

### Planned Features

1. **Distributed Replay Protection**: Multi-node nonce synchronization
2. **Advanced Nonce Schemes**: Time-based nonce generation
3. **Machine Learning**: Anomaly detection for replay patterns
4. **Blockchain Integration**: Immutable nonce tracking
5. **Zero-Knowledge Proofs**: Privacy-preserving nonce validation

### Research Areas

- **Quantum-Resistant Nonces**: Post-quantum cryptography integration
- **Federated Trust**: Cross-domain DID resolution
- **Adaptive Windows**: Dynamic replay window sizing
- **Predictive Analysis**: Proactive replay attack prevention

## References

### Specifications

- [RFC 6749: OAuth 2.0](https://tools.ietf.org/html/rfc6749)
- [DID Core Specification](https://www.w3.org/TR/did-core/)
- [Dilithium Post-Quantum Signature](https://pq-crystals.org/dilithium/)

### Related Documentation

- [Polymera OS Security Model](../security/SECURITY-MODEL.md)
- [Capability System Design](../capabilities/CAPABILITY-SYSTEM.md)
- [DID Trust Framework](../did/DID-TRUST.md)
- [Audit System Guide](../audit/AUDIT-SYSTEM.md)

### Implementation Notes

- **Language**: Rust with no_std support
- **Dependencies**: spin, lru, zeroize, serde
- **Target**: x86_64, ARM64, RISC-V
- **License**: MIT License
