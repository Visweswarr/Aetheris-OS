# ABI Features System

## Overview

The ABI Features System provides a mechanism for the kernel to advertise available capabilities and for user applications to adapt their behavior accordingly. This system enables forward compatibility and graceful degradation when certain kernel features are not available.

## Feature Flags

### Core Feature Constants

The system defines feature flags as bit positions in a 64-bit bitset:

```rust
pub const FEATURE_PQC_CAPS_V2: u64 = 1 << 0;      // Post-quantum cryptography capabilities v2
pub const FEATURE_APIC_TIMER: u64 = 1 << 1;       // Local APIC timer support
pub const FEATURE_HPET_FALLBACK: u64 = 1 << 2;    // HPET fallback timer support
pub const FEATURE_CAP_REPLAY_WIN: u64 = 1 << 3;   // Capability replay window protection
pub const FEATURE_MMU_AUDIT: u64 = 1 << 4;        // MMU hygiene and audit tools
pub const FEATURE_CRASH_ANALYSIS: u64 = 1 << 5;   // Enhanced crash analysis and minidumps
pub const FEATURE_SOAK_CHAOS: u64 = 1 << 6;       // Soak testing and chaos engineering
pub const FEATURE_SIDE_CHANNEL_GUARD: u64 = 1 << 7; // Side-channel vulnerability protection
pub const FEATURE_STREAM_AUTH: u64 = 1 << 8;      // IPC stream authentication
pub const FEATURE_DID_RESOLVER: u64 = 1 << 9;     // Decentralized identifier resolution
pub const FEATURE_SECURE_BOOT: u64 = 1 << 10;     // Secure boot and attestation
pub const FEATURE_ABI_HARDENING: u64 = 1 << 11;   // ABI hardening and validation
```

### Feature Categories

#### Cryptographic Features
- **PQC_CAPS_V2**: Post-quantum cryptography capabilities including Dilithium signatures and Kyber KEM
- **CAP_REPLAY_WIN**: Capability replay window protection with sliding nonce validation
- **STREAM_AUTH**: IPC stream authentication using Kyber-derived session keys
- **DID_RESOLVER**: Decentralized identifier resolution and trust anchors
- **SECURE_BOOT**: Secure boot and attestation with Dilithium2 signatures

#### Timing and Hardware Features
- **APIC_TIMER**: Local APIC timer support for high-precision timing
- **HPET_FALLBACK**: HPET fallback timer support when APIC is not available

#### Security and Safety Features
- **MMU_AUDIT**: MMU hygiene and audit tools for page table validation
- **SIDE_CHANNEL_GUARD**: Side-channel vulnerability protection and detection
- **ABI_HARDENING**: ABI hardening and validation with argument checking

#### Development and Testing Features
- **CRASH_ANALYSIS**: Enhanced crash analysis with minidumps and symbolization
- **SOAK_CHAOS**: Soak testing and chaos engineering for system stability

## Feature Dependencies

Some features depend on others being available:

```rust
// CAP_REPLAY_WIN depends on PQC_CAPS_V2
FEATURE_CAP_REPLAY_WIN => [FEATURE_PQC_CAPS_V2]

// STREAM_AUTH depends on PQC_CAPS_V2
FEATURE_STREAM_AUTH => [FEATURE_PQC_CAPS_V2]

// DID_RESOLVER depends on PQC_CAPS_V2
FEATURE_DID_RESOLVER => [FEATURE_PQC_CAPS_V2]

// SECURE_BOOT depends on PQC_CAPS_V2
FEATURE_SECURE_BOOT => [FEATURE_PQC_CAPS_V2]
```

## System Call Interface

### SYS_GET_FEATURES

**Syscall Number**: 21

**Arguments**: None

**Returns**: Feature flags bitset (u64)

**Description**: Returns the current kernel feature flags bitset, allowing user applications to determine which features are available.

**Example Usage**:
```rust
use userland_stubs::sys_get_features;

let features = sys_get_features();
if features & FEATURE_PQC_CAPS_V2 != 0 {
    // PQC capabilities are available
    println!("PQC capabilities supported");
} else {
    // Fall back to classical cryptography
    println!("Using classical cryptography");
}
```

## ABI Negotiation

### AbiNegotiationHeader Structure

```rust
#[repr(C, packed)]
pub struct AbiNegotiationHeader {
    pub magic: [u8; 4],           // "ABIN" magic number
    pub version: u32,             // Protocol version (currently 1)
    pub required_features: u64,   // Features that must be available
    pub optional_features: u64,   // Features that are nice to have
    pub payload_size: u32,        // Size of the payload in bytes
    pub reserved: [u8; 16],      // Reserved for future use
}
```

### Feature Negotiation Process

1. **Header Validation**: Check magic number and version compatibility
2. **Feature Check**: Verify all required features are available
3. **Result**: Return success or list of missing features

### Negotiation Results

```rust
pub enum FeatureNegotiationResult {
    Success,                           // All required features available
    MissingFeatures(Vec<u64>),        // Some required features missing
    InvalidHeader,                     // Invalid ABI negotiation header
    UnsupportedVersion,                // Unsupported protocol version
}
```

### Example ABI Negotiation

```rust
use kernel::abi::{AbiNegotiationHeader, negotiate_abi_features, FEATURE_PQC_CAPS_V2};

// Create negotiation header
let header = AbiNegotiationHeader::new(
    FEATURE_PQC_CAPS_V2,  // Required: PQC capabilities
    0,                     // Optional: none
    1024                   // Payload size
);

// Negotiate features
match negotiate_abi_features(&header) {
    FeatureNegotiationResult::Success => {
        println!("All required features available");
        // Proceed with payload processing
    }
    FeatureNegotiationResult::MissingFeatures(missing) => {
        println!("Missing features: {:?}", missing);
        // Handle missing features gracefully
    }
    FeatureNegotiationResult::InvalidHeader => {
        println!("Invalid ABI negotiation header");
        // Reject the payload
    }
    FeatureNegotiationResult::UnsupportedVersion => {
        println!("Unsupported ABI version");
        // Reject the payload
    }
}
```

## User Application Integration

### Feature Detection at Initialization

User applications should check for required features at startup:

```rust
use userland_stubs::sys_get_features;

fn check_required_features() -> Result<(), String> {
    let available_features = sys_get_features();
    
    // Check for PQC capabilities
    if available_features & FEATURE_PQC_CAPS_V2 == 0 {
        return Err("PQC capabilities not available".to_string());
    }
    
    // Check for APIC timer
    if available_features & FEATURE_APIC_TIMER == 0 {
        println!("Warning: APIC timer not available, using fallback");
    }
    
    Ok(())
}

fn main() {
    // Check features before proceeding
    if let Err(e) = check_required_features() {
        eprintln!("Feature check failed: {}", e);
        std::process::exit(1);
    }
    
    // Application can now use available features
    println!("All required features available");
}
```

### Graceful Degradation

Applications should implement fallback behavior for optional features:

```rust
fn initialize_cryptography() {
    let features = sys_get_features();
    
    if features & FEATURE_PQC_CAPS_V2 != 0 {
        // Use PQC capabilities
        initialize_pqc_crypto();
    } else if features & FEATURE_ABI_HARDENING != 0 {
        // Use hardened classical crypto
        initialize_hardened_crypto();
    } else {
        // Use basic crypto
        initialize_basic_crypto();
    }
}

fn initialize_timing() {
    let features = sys_get_features();
    
    if features & FEATURE_APIC_TIMER != 0 {
        // Use high-precision APIC timer
        initialize_apic_timer();
    } else if features & FEATURE_HPET_FALLBACK != 0 {
        // Use HPET fallback
        initialize_hpet_timer();
    } else {
        // Use basic timer
        initialize_basic_timer();
    }
}
```

## Kernel Integration

### Feature Manager Initialization

The feature manager is initialized during kernel boot:

```rust
// In kernel/src/boot.rs
pub fn init() {
    // ... other initialization ...
    
    // Initialize ABI feature manager
    kprintln!("[PolymeraCore] Initializing ABI feature manager");
    crate::abi::init_feature_manager();
    
    // ... continue initialization ...
}
```

### Feature Availability Detection

The kernel automatically detects available features:

```rust
impl FeatureManager {
    fn enable_available_features(&mut self) {
        // Enable features that are always available
        self.enable_feature(FEATURE_ABI_HARDENING);
        
        // Check for PQC support
        if self.check_pqc_support() {
            self.enable_feature(FEATURE_PQC_CAPS_V2);
            self.enable_feature(FEATURE_CAP_REPLAY_WIN);
            self.enable_feature(FEATURE_STREAM_AUTH);
            self.enable_feature(FEATURE_DID_RESOLVER);
            self.enable_feature(FEATURE_SECURE_BOOT);
        }
        
        // Check for APIC timer support
        if self.check_apic_timer_support() {
            self.enable_feature(FEATURE_APIC_TIMER);
        }
        
        // Check for HPET support
        if self.check_hpet_support() {
            self.enable_feature(FEATURE_HPET_FALLBACK);
        }
        
        // Enable debug features in debug builds
        if cfg!(debug_assertions) {
            self.enable_feature(FEATURE_MMU_AUDIT);
        }
        
        // Enable features that are always available
        self.enable_feature(FEATURE_CRASH_ANALYSIS);
        self.enable_feature(FEATURE_SOAK_CHAOS);
        self.enable_feature(FEATURE_SIDE_CHANNEL_GUARD);
    }
}
```

## Error Handling

### Missing Feature Errors

When a required feature is not available, the kernel returns `ENOTSUP` (Operation not supported):

```rust
// In user application
let features = sys_get_features();
if features & FEATURE_PQC_CAPS_V2 == 0 {
    // Handle missing feature
    eprintln!("PQC capabilities not supported by kernel");
    std::process::exit(1);
}
```

### ABI Negotiation Failures

ABI negotiation failures should be handled gracefully:

```rust
fn process_user_payload(header: &AbiNegotiationHeader, payload: &[u8]) -> Result<(), String> {
    match negotiate_abi_features(header) {
        FeatureNegotiationResult::Success => {
            // Process payload
            process_payload(payload)
        }
        FeatureNegotiationResult::MissingFeatures(missing) => {
            let missing_names: Vec<String> = missing.iter()
                .filter_map(|&id| get_feature_name(id))
                .collect();
            Err(format!("Missing required features: {}", missing_names.join(", ")))
        }
        FeatureNegotiationResult::InvalidHeader => {
            Err("Invalid ABI negotiation header".to_string())
        }
        FeatureNegotiationResult::UnsupportedVersion => {
            Err("Unsupported ABI version".to_string())
        }
    }
}
```

## Testing and Validation

### Feature Flag Testing

Test that feature flags are properly defined and accessible:

```rust
#[test]
fn test_feature_flags() {
    // Test that feature flags are unique
    let flags = [
        FEATURE_PQC_CAPS_V2,
        FEATURE_APIC_TIMER,
        FEATURE_HPET_FALLBACK,
        FEATURE_CAP_REPLAY_WIN,
        FEATURE_MMU_AUDIT,
        FEATURE_CRASH_ANALYSIS,
        FEATURE_SOAK_CHAOS,
        FEATURE_SIDE_CHANNEL_GUARD,
        FEATURE_STREAM_AUTH,
        FEATURE_DID_RESOLVER,
        FEATURE_SECURE_BOOT,
        FEATURE_ABI_HARDENING,
    ];
    
    // Check for duplicates
    for i in 0..flags.len() {
        for j in (i + 1)..flags.len() {
            assert_ne!(flags[i], flags[j], "Duplicate feature flags detected");
        }
    }
}
```

### Syscall Testing

Test the SYS_GET_FEATURES syscall:

```rust
#[test]
fn test_get_features_syscall() {
    let features = sys_get_features();
    
    // Should return a valid bitset
    assert!(features != 0);
    
    // ABI_HARDENING should always be available
    assert!(features & FEATURE_ABI_HARDENING != 0);
}
```

### ABI Negotiation Testing

Test ABI negotiation functionality:

```rust
#[test]
fn test_abi_negotiation() {
    // Test successful negotiation
    let header = AbiNegotiationHeader::new(
        FEATURE_ABI_HARDENING,
        0,
        0
    );
    
    let result = negotiate_abi_features(&header);
    assert!(matches!(result, FeatureNegotiationResult::Success));
    
    // Test missing features
    let header = AbiNegotiationHeader::new(
        FEATURE_PQC_CAPS_V2 | FEATURE_ABI_HARDENING,
        0,
        0
    );
    
    let result = negotiate_abi_features(&header);
    match result {
        FeatureNegotiationResult::Success => {
            // PQC is available in this test environment
        }
        FeatureNegotiationResult::MissingFeatures(missing) => {
            assert!(missing.contains(&FEATURE_PQC_CAPS_V2));
        }
        _ => panic!("Unexpected result"),
    }
}
```

## Future Extensions

### Additional Feature Flags

Future versions may add more feature flags:

```rust
// Potential future features
pub const FEATURE_VIRTUALIZATION: u64 = 1 << 12;    // Hardware virtualization support
pub const FEATURE_IOMMU: u64 = 1 << 13;             // IOMMU support
pub const FEATURE_SECURE_ENCLAVE: u64 = 1 << 14;    // Secure enclave support
pub const FEATURE_MEMORY_ENCRYPTION: u64 = 1 << 15; // Memory encryption support
```

### Feature Versioning

Features may support versioning for backward compatibility:

```rust
pub struct FeatureVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

pub struct FeatureInfo {
    pub id: u64,
    pub name: &'static str,
    pub version: Option<FeatureVersion>,
    pub min_kernel_version: Option<FeatureVersion>,
    pub deprecated: bool,
    pub replacement: Option<u64>,
}
```

### Dynamic Feature Discovery

Future versions may support dynamic feature discovery:

```rust
pub enum FeatureDiscovery {
    Static,     // Features known at compile time
    Dynamic,    // Features discovered at runtime
    Hybrid,     // Combination of static and dynamic
}
```

## Best Practices

### Feature Usage Guidelines

1. **Check Features Early**: Verify required features at application startup
2. **Graceful Degradation**: Implement fallback behavior for optional features
3. **Error Handling**: Provide clear error messages for missing features
4. **Documentation**: Document feature requirements and dependencies
5. **Testing**: Test with different feature combinations

### Security Considerations

1. **Feature Validation**: Validate feature claims from user applications
2. **Capability Checking**: Ensure features align with application capabilities
3. **Audit Logging**: Log feature usage for security monitoring
4. **Access Control**: Restrict feature access based on security policies

### Performance Considerations

1. **Feature Caching**: Cache feature information to avoid repeated syscalls
2. **Lazy Loading**: Load feature-dependent code only when needed
3. **Optimization**: Use feature flags for compile-time optimizations
4. **Monitoring**: Track feature usage for performance analysis

## Conclusion

The ABI Features System provides a robust foundation for kernel-user application compatibility and forward compatibility. By using feature flags and ABI negotiation, applications can adapt to different kernel capabilities while maintaining security and performance.

This system enables:
- **Forward Compatibility**: New applications work with older kernels
- **Backward Compatibility**: Old applications work with newer kernels
- **Graceful Degradation**: Applications adapt to available features
- **Security**: Feature access is controlled and audited
- **Performance**: Optimizations based on available features

The system is designed to be extensible, allowing new features to be added without breaking existing applications or requiring kernel updates.

