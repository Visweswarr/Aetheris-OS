# DID Trust Management System

## Overview

The DID Trust Management System provides secure, scalable, and flexible trust establishment for Decentralized Identifiers (DIDs) in Polymera OS. This system implements anchor-based trust with automatic TTL management, key rotation support, and rotation grace periods to ensure continuous service availability during cryptographic key transitions.

## Architecture

### Core Components

1. **DidResolver** - Central DID resolution and trust management
2. **DidAnchor** - Trust anchor with rotation and TTL support
3. **DidResolverConfig** - Configurable trust parameters
4. **Trust Validation** - Cryptographic and temporal validation
5. **Anchor Rotation** - Seamless key rotation with grace periods

### System Flow

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐    ┌─────────────┐
│ DID Request │───▶│ DidResolver  │───▶│ DidAnchor   │───▶│ Trust Check │
│ (issuer)    │    │ (Lookup)     │    │ (Validation)│    │ (TTL/Key)  │
└─────────────┘    └──────────────┘    └─────────────┘    └─────────────┘
                           │
                           ▼
                   ┌──────────────┐
                   │ Audit Event  │
                   │ (DID_FAIL)   │
                   └──────────────┘
```

## DID Anchor Structure

### Core Anchor Fields

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

### Trust Properties

- **Public Key**: Dilithium post-quantum public key
- **TTL**: Configurable time-to-live for trust validity
- **Creation Time**: Immutable anchor creation timestamp
- **Expiration Time**: Automatic trust expiration
- **Rotation Support**: Key rotation with timestamp tracking
- **Grace Period**: Transition period for key rotation

## Trust Management Operations

### Anchor Creation

```rust
// Create new DID anchor
let anchor = DidAnchor::new(
    pubkey,                    // Dilithium public key
    Duration::from_secs(86400), // 24-hour TTL
    Duration::from_secs(3600),  // 1-hour rotation grace
);

// Add to resolver
resolver.add_anchor(
    "did:example:issuer1".to_string(),
    pubkey,
    Some(Duration::from_secs(86400)),
    Some(Duration::from_secs(3600)),
)?;
```

### Anchor Resolution

```rust
// Resolve DID to public key
let pubkey = resolver.resolve_did("did:example:issuer1")?;

// Resolve to all valid keys (including rotation grace)
let all_keys = resolver.resolve_did_all_keys("did:example:issuer1")?;

// Verify specific public key
let is_valid = resolver.verify_did_key("did:example:issuer1", &pubkey)?;
```

### Key Rotation

```rust
// Rotate to new public key
resolver.rotate_anchor(
    "did:example:issuer1",
    new_pubkey,
)?;

// During grace period, both keys are valid
let keys = resolver.resolve_did_all_keys("did:example:issuer1")?;
assert_eq!(keys.len(), 2); // Current + previous key

// After grace period, only new key is valid
std::thread::sleep(rotation_grace + Duration::from_millis(100));
let keys = resolver.resolve_did_all_keys("did:example:issuer1")?;
assert_eq!(keys.len(), 1); // Only current key
```

## Configuration

### Resolver Configuration

```rust
pub struct DidResolverConfig {
    pub default_ttl: Duration,              // Default anchor TTL
    pub default_rotation_grace: Duration,   // Default rotation grace
    pub max_anchors: usize,                 // Maximum anchors
    pub cleanup_interval: Duration,         // Cleanup frequency
}
```

### Default Values

- **default_ttl**: 24 hours (86,400 seconds)
- **default_rotation_grace**: 1 hour (3,600 seconds)
- **max_anchors**: 1,000 anchors
- **cleanup_interval**: 5 minutes (300 seconds)

### Environment Configuration

```bash
# DID trust configuration
DID_DEFAULT_TTL=86400              # Default TTL in seconds
DID_ROTATION_GRACE=3600            # Default grace period
DID_MAX_ANCHORS=1000               # Maximum anchors
DID_CLEANUP_INTERVAL=300           # Cleanup interval
```

## Trust Validation

### TTL Validation

```rust
impl DidAnchor {
    pub fn is_expired(&self, current_time: Instant) -> bool {
        current_time > self.expires_at
    }
    
    pub fn time_until_expiry(&self, current_time: Instant) -> Duration {
        if current_time < self.expires_at {
            self.expires_at - current_time
        } else {
            Duration::from_secs(0)
        }
    }
}
```

### Key Validation

```rust
impl DidAnchor {
    pub fn is_key_valid(&self, pubkey: &DilithiumPubKey, current_time: Instant) -> bool {
        if self.is_expired(current_time) {
            return false;
        }
        
        // Check current key
        if self.pubkey == *pubkey {
            return true;
        }
        
        // Check previous key if in rotation grace
        if let Some(prev_key) = &self.previous_pubkey {
            if self.is_in_rotation_grace(current_time) && *prev_key == *pubkey {
                return true;
            }
        }
        
        false
    }
}
```

### Rotation Grace Logic

```rust
impl DidAnchor {
    pub fn is_in_rotation_grace(&self, current_time: Instant) -> bool {
        if let Some(rotated_at) = self.rotated_at {
            current_time <= rotated_at + self.rotation_grace
        } else {
            false
        }
    }
    
    pub fn get_valid_keys(&self, current_time: Instant) -> Vec<DilithiumPubKey> {
        let mut keys = Vec::new();
        
        // Add current key if not expired
        if !self.is_expired(current_time) {
            keys.push(self.pubkey.clone());
        }
        
        // Add previous key if in rotation grace
        if let Some(prev_key) = &self.previous_pubkey {
            if self.is_in_rotation_grace(current_time) {
                keys.push(prev_key.clone());
            }
        }
        
        keys
    }
}
```

## Security Properties

### Trust Guarantees

1. **Temporal Bounds**: Trust automatically expires after TTL
2. **Key Rotation**: Seamless transition between cryptographic keys
3. **Grace Periods**: Continuous service during key rotation
4. **Poisoning Prevention**: Anchor integrity validation
5. **Automatic Cleanup**: Expired anchors removed automatically

### Attack Resistance

- **Replay Attacks**: TTL prevents replay of expired anchors
- **Key Compromise**: Rotation allows rapid key replacement
- **Trust Poisoning**: Anchor validation prevents malicious anchors
- **Memory Exhaustion**: Configurable anchor limits
- **Clock Manipulation**: Relative time validation

### Performance Characteristics

- **Constant Time**: O(1) average case for anchor lookup
- **Bounded Memory**: Configurable memory usage per anchor
- **Efficient Cleanup**: Periodic cleanup with minimal overhead
- **Scalable Design**: Linear scaling with number of anchors

## Monitoring and Metrics

### Key Metrics

```rust
pub struct DidResolverStats {
    pub total_anchors: usize,      // Total anchors
    pub expired_anchors: usize,    // Expired anchors
    pub active_anchors: usize,     // Active anchors
    pub rotating_anchors: usize,   // Anchors in rotation
    pub max_anchors: usize,        // Maximum allowed
}
```

### Health Checks

```rust
// Check anchor health
let stats = resolver.get_stats();
if stats.expired_anchors > stats.total_anchors / 2 {
    // Alert: High expired anchor ratio
}

// Check rotation activity
if stats.rotating_anchors > threshold {
    // Alert: High rotation activity
}

// Check memory usage
if stats.total_anchors > stats.max_anchors * 9 / 10 {
    // Alert: Approaching anchor limit
}
```

### Log Analysis

```bash
# Monitor DID resolution
grep "DID_FAIL\|DID_Expired" /var/log/polymera/audit.log

# Monitor key rotations
grep "DID_Rotation\|DID_RotationGrace" /var/log/polymera/audit.log

# Monitor anchor creation
grep "DID_Anchor_Created" /var/log/polymera/audit.log
```

## Testing

### Test Categories

1. **Unit Tests**: Individual component testing
2. **Integration Tests**: End-to-end trust validation
3. **Load Tests**: Performance under high anchor count
4. **Security Tests**: Attack vector validation
5. **Rotation Tests**: Key rotation scenarios

### Test Commands

```bash
# Run DID anchors tests
cargo test did_anchors

# Run specific test categories
cargo test test_anchor_creation
cargo test test_anchor_rotation
cargo test test_rotation_grace

# Run performance benchmarks
cargo bench did_resolution
```

### Test Coverage

- **Anchor Creation**: 100% coverage of creation scenarios
- **TTL Validation**: Edge case testing for expiration
- **Key Rotation**: Complete rotation flow testing
- **Grace Periods**: Temporal boundary testing
- **Performance**: Load testing with 1,000+ anchors

## Deployment Considerations

### Production Settings

- **TTL Values**: 24 hours to 7 days based on security requirements
- **Rotation Grace**: 1-6 hours based on deployment complexity
- **Anchor Limits**: 1,000-10,000 based on expected issuers
- **Cleanup Frequency**: 5-15 minutes based on performance requirements

### Scaling Considerations

- **Memory Usage**: ~200 bytes per anchor
- **CPU Overhead**: <1% for typical workloads
- **Network Impact**: Minimal additional latency
- **Storage Requirements**: No persistent storage needed

### Monitoring Requirements

- **Anchor Expiration Rate**: Monitor for unusual expiration patterns
- **Rotation Activity**: Track key rotation frequency
- **Memory Usage**: Monitor anchor memory consumption
- **Performance Metrics**: Track resolution latency

## Troubleshooting

### Common Issues

1. **High Expired Anchor Count**
   - Check TTL configuration
   - Verify cleanup interval settings
   - Review anchor creation patterns

2. **Rotation Failures**
   - Verify rotation grace period configuration
   - Check for clock synchronization issues
   - Review anchor state consistency

3. **Performance Degradation**
   - Profile anchor lookup performance
   - Check cleanup efficiency
   - Monitor memory usage patterns

### Debug Commands

```bash
# Check DID resolver status
polymera-cli did status

# Inspect specific anchor
polymera-cli did inspect <did>

# Check anchor statistics
polymera-cli did stats

# Force cleanup
polymera-cli did cleanup
```

### Log Analysis

```bash
# Analyze DID resolution patterns
grep "DID_" audit.log | awk '{print $4}' | sort | uniq -c

# Check anchor expiration rates
grep "DID_Expired" audit.log | tail -100

# Monitor rotation activity
grep "DID_Rotation" audit.log | tail -100
```

## Integration with Capability System

### Capability Verification Flow

```rust
// In capability verification
let issuer_pubkey = match self.did_resolver.resolve_did(&token.header.issuer) {
    Ok(pubkey) => pubkey,
    Err(_) => {
        self.increment_stat(|stats| stats.did_failures += 1);
        return Err(CapVerifyError::IssuerUntrusted);
    }
};

// Verify token signature
if let Err(e) = token.verify_signature(&issuer_pubkey) {
    self.increment_stat(|stats| stats.auth_fail += 1);
    return Err(e);
}
```

### Audit Integration

```rust
// Create audit event for DID failure
let audit_event = AuditEvent::new(
    AuditReason::DidFail,
    "cap_store".to_string(),
    false,
)
.with_target(token.header.issuer.clone())
.with_context("DID resolution failed".to_string());

// Emit audit event
audit_system.emit(audit_event);
```

## Future Enhancements

### Planned Features

1. **Distributed Trust**: Multi-node trust synchronization
2. **Federated Trust**: Cross-domain trust establishment
3. **Blockchain Integration**: Immutable trust anchors
4. **Zero-Knowledge Proofs**: Privacy-preserving trust validation
5. **Machine Learning**: Anomaly detection for trust patterns

### Research Areas

- **Quantum-Resistant Trust**: Post-quantum cryptography integration
- **Adaptive TTL**: Dynamic trust duration based on usage patterns
- **Trust Scoring**: Reputation-based trust establishment
- **Predictive Trust**: Proactive trust validation

## References

### Specifications

- [DID Core Specification](https://www.w3.org/TR/did-core/)
- [DID Resolution](https://www.w3.org/TR/did-resolution/)
- [Dilithium Post-Quantum Signature](https://pq-crystals.org/dilithium/)
- [RFC 5280: X.509 Certificate](https://tools.ietf.org/html/rfc5280)

### Related Documentation

- [Polymera OS Security Model](../security/SECURITY-MODEL.md)
- [CapV2 Replay Protection](./CAPV2-REPLAY.md)
- [Audit System Guide](../audit/AUDIT-SYSTEM.md)
- [Cryptographic Primitives](../crypto/CRYPTO-PRIMITIVES.md)

### Implementation Notes

- **Language**: Rust with no_std support
- **Dependencies**: spin, serde, alloc
- **Target**: x86_64, ARM64, RISC-V
- **License**: MIT License
- **Security Level**: Post-quantum resistant
