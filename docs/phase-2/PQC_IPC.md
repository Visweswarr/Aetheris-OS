# PQC-Authenticated IPC System

## Overview

The PQC-Authenticated IPC system represents a significant security enhancement to Polymera OS's inter-process communication infrastructure. This system integrates post-quantum cryptography (PQC) with capability-based security to provide secure, auditable, and performant message authentication for all IPC operations.

## Architecture

### Core Components

#### 1. IPC Header v2 (`kernel/src/ipc/header.rs`)
The enhanced IPC header structure includes:

- **Message ID**: Unique identifier for each message
- **MAC Tag**: 16-byte message authentication code
- **Capability ID**: 128-bit identifier linking to the capability token
- **Authentication Mode**: Capability-only or full authentication
- **Session Key**: Optional ephemeral key for Kyber KEM

```rust
pub struct IpcHeaderV2 {
    pub msg_id: MessageId,
    pub sender: ProcessId,
    pub receiver: ProcessId,
    pub msg_type: MessageType,
    pub priority: MessagePriority,
    pub flags: MessageFlags,
    pub payload_size: usize,
    pub timestamp: u64,
    pub mac_tag: [u8; MAC_TAG_SIZE],
    pub cap_id: [u8; CAP_ID_SIZE],
    pub auth_mode: AuthMode,
    pub session_key: Option<[u8; 32]>,
}
```

#### 2. Authentication Manager (`kernel/src/ipc/auth.rs`)
The central authentication system provides:

- **Dual Authentication Modes**: Capability-only (fast path) and full authentication
- **MAC Calculation/Verification**: Using Blake3 keyed hashing (stub implementation)
- **Performance Monitoring**: Authentication overhead tracking
- **Statistics Collection**: Success/failure rates and performance metrics

#### 3. PQC Helper Functions (`kernel/src/ipc/pqc_helpers.rs`)
Utility functions for:

- **Token Conversion**: Legacy capability tokens to CapTokenV2
- **Header Creation**: Automatic authentication mode selection
- **Session Key Generation**: Ephemeral keys for Kyber KEM

### Authentication Modes

#### Capability-Only Authentication
- **Use Case**: Small messages (< 64 bytes) with REALTIME flags
- **Performance**: Fastest path with minimal overhead
- **Security**: Relies on capability token validation only
- **Audit**: Still logged for security monitoring

#### Full Authentication
- **Use Case**: Larger messages (≥ 64 bytes) or high-security requirements
- **Performance**: Additional overhead for MAC calculation/verification
- **Security**: Capability token + message authentication code
- **Features**: Ephemeral session keys, replay protection

## Security Features

### 1. Post-Quantum Cryptography Integration
- **Dilithium2 Signatures**: For capability token validation
- **Kyber768 Encapsulation**: For ephemeral session key generation
- **Quantum-Resistant**: Protection against future quantum attacks

### 2. Message Authentication
- **MAC Tags**: 16-byte authentication codes for message integrity
- **Session Keys**: Ephemeral keys for enhanced security
- **Tamper Detection**: Automatic rejection of modified messages

### 3. Capability Enforcement
- **Token Validation**: PQC-signed capability verification
- **Scope Checking**: Fine-grained permission validation
- **Audit Logging**: Comprehensive security event recording

### 4. Replay Protection
- **Nonce-Based**: Unique identifiers for each message
- **Time Validation**: Expiration and validity window checking
- **Cache Management**: LRU-based nonce tracking

## Performance Characteristics

### 1. Authentication Overhead
- **Target**: < 15% median overhead vs non-authenticated baseline
- **Capability-Only**: Minimal overhead (< 5%)
- **Full Authentication**: Moderate overhead (10-15%)
- **Optimization**: Fast-path bypass for small messages

### 2. Scalability
- **Message Size**: Efficient handling from 16 bytes to 1MB+
- **Concurrent Operations**: Thread-safe authentication manager
- **Memory Usage**: Compact header representation
- **Cache Efficiency**: LRU-based optimization

### 3. Real-Time Support
- **REALTIME Flag**: Special handling for time-critical messages
- **MAC Skipping**: Optional authentication for small RT messages
- **Priority Handling**: Different authentication levels by priority

## Implementation Details

### 1. Message Flow

#### Sending Process
```rust
// 1. Validate capability token
let cap_token = check_capability(sender_pid, dst);

// 2. Create authenticated header
let header_v2 = create_ipc_header_v2(msg, &cap_token);

// 3. Authenticate message
let auth_result = authenticate_ipc_message(&header_v2, &msg.payload, &cap_token);

// 4. Send if authenticated
if auth_result.authenticated {
    send_message_with_wakeup(sender_pid, dst, msg);
}
```

#### Receiving Process
```rust
// 1. Receive message with header
let message = receive_message();

// 2. Verify authentication
let auth_result = verify_message_authentication(&message);

// 3. Process if verified
if auth_result.authenticated {
    process_message(message);
}
```

### 2. Authentication Decision Logic

```rust
fn determine_auth_mode(payload_size: usize, flags: MessageFlags) -> AuthMode {
    if payload_size < 64 && flags.contains(MessageFlags::REALTIME) {
        AuthMode::CapabilityOnly
    } else {
        AuthMode::FullAuth
    }
}
```

### 3. MAC Calculation

```rust
fn calculate_mac(header: &IpcHeaderV2, payload: &[u8], session_key: &[u8; 32]) -> [u8; 16] {
    // Current: Blake3 keyed hashing (stub)
    // Future: PQC KDF + proper MAC
    
    let mut input = Vec::new();
    input.extend_from_slice(session_key);
    input.extend_from_slice(&header.to_bytes());
    input.extend_from_slice(payload);
    
    blake3_keyed_hash(&input)
}
```

## Testing Strategy

### 1. Unit Tests
- **Header Creation**: Authentication mode selection
- **MAC Calculation**: Integrity verification
- **Token Conversion**: Legacy to v2 migration
- **Serialization**: Header encoding/decoding

### 2. Integration Tests
- **Ping-Pong**: Authentication on/off scenarios
- **Performance**: Overhead measurement and validation
- **CapTokens v2**: Integration with capability system
- **Audit Logging**: Security event recording

### 3. Adversarial Tests
- **MAC Tampering**: Rejection of modified messages
- **Invalid Tokens**: Capability validation failure
- **Session Key Attacks**: Missing or corrupted keys
- **Size Attacks**: Message size manipulation

### 4. Performance Tests
- **Baseline Measurement**: Non-authenticated IPC
- **Overhead Calculation**: Authentication cost analysis
- **Scalability Testing**: Large message handling
- **Concurrent Operations**: Multi-threaded performance

## Usage Examples

### 1. Basic Message Sending

```rust
use crate::ipc::sys::sys_send;
use crate::ipc::types::Message;

// Create message
let message = Message::new(
    MessageId::new(123),
    ProcessId::new(1001),
    ProcessId::new(2001),
    MessageType::Data,
    b"Hello, World!".to_vec(),
);

// Send with automatic authentication
let result = sys_send(2001, &message);
match result {
    Ok(()) => println!("Message sent successfully"),
    Err(e) => println!("Send failed: {}", e),
}
```

### 2. High-Performance Real-Time Messages

```rust
use crate::ipc::header::IpcHeaderV2;
use crate::ipc::auth::IpcAuthManager;

// Create RT message with small payload
let mut header = IpcHeaderV2::new_capability_only(
    MessageId::new(456),
    ProcessId::new(1001),
    ProcessId::new(2001),
    MessageType::Control,
    32, // Small payload
    &cap_token,
);

// Add REALTIME flag for fast-path
header.flags = MessageFlags::REALTIME;

// This will use capability-only authentication
assert!(header.can_skip_mac());
```

### 3. High-Security Large Messages

```rust
// Create large message requiring full authentication
let header = IpcHeaderV2::new_full_auth(
    MessageId::new(789),
    ProcessId::new(1001),
    ProcessId::new(2001),
    MessageType::Data,
    1024, // Large payload
    &cap_token,
    session_key,
);

// This will use full authentication with MAC
assert!(header.requires_full_auth());
assert!(header.session_key.is_some());
```

## Migration Strategy

### 1. Phase 1: Infrastructure
- [x] IPC Header v2 implementation
- [x] Authentication manager
- [x] Helper functions
- [x] Basic integration

### 2. Phase 2: Integration
- [ ] Full sys_send/sys_recv integration
- [ ] Performance optimization
- [ ] Real-world testing
- [ ] Documentation updates

### 3. Phase 3: Enhancement
- [ ] PQC KDF integration
- [ ] Hardware acceleration
- [ ] Advanced features
- [ ] Production deployment

## Future Enhancements

### 1. Advanced PQC Features
- **Multi-Algorithm Support**: Kyber + Dilithium combinations
- **Threshold Signatures**: Multi-party authentication
- **Quantum Key Distribution**: Enhanced key management

### 2. Performance Optimizations
- **Hardware Acceleration**: PQC-specific hardware
- **Batch Processing**: Multiple message authentication
- **Parallel Verification**: Concurrent MAC validation

### 3. Security Enhancements
- **Advanced Threat Modeling**: Sophisticated attack scenarios
- **Behavioral Analysis**: Anomaly detection
- **Zero-Knowledge Proofs**: Privacy-preserving authentication

## Conclusion

The PQC-Authenticated IPC system provides:

- **Quantum-Resistant Security**: Protection against future threats
- **High Performance**: Minimal overhead for most operations
- **Flexible Authentication**: Adaptive security based on requirements
- **Comprehensive Auditing**: Complete security event tracking
- **Easy Integration**: Seamless upgrade from existing IPC

This system establishes a solid foundation for secure inter-process communication in the post-quantum era while maintaining the performance characteristics required for real-time systems.
