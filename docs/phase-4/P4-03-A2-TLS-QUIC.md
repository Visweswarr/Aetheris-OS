# P4-03-A2: TLS/mTLS and QUIC Implementation

## 1. Introduction

This document details the implementation of broker-terminated TLS/mTLS and QUIC (HTTP/3) support for Aetheris OS, as part of Phase 4, Sub-phase A2 (P4-03-A2). All TLS and QUIC operations are brokered, capability-enforced, and use DID-bound certificates from the Personal Data Vault (PDV).

## 2. Architecture Overview

The TLS/QUIC subsystem extends the existing networking infrastructure with:

- **Broker-terminated TLS**: All TLS operations are handled by the broker, with private keys sealed in KeyVault
- **DID-bound certificates**: Certificates are bound to Decentralized Identifiers (DIDs) and managed through PDV
- **QUIC support**: HTTP/3 over QUIC with broker mediation
- **Policy enforcement**: OPA/WASM policies control TLS/QUIC access and configuration
- **Performance monitoring**: Comprehensive metrics and audit logging

## 3. Core Components

### 3.1. TLS Broker (`services/net/posixnet/src/tls.rs`)

The TLS broker provides the following RPC API:

- `tls_wrap(socket_id, profile, process_cap)` → `session_id`
- `tls_accept(listener_socket_id, profile, process_cap)` → `(session_id, socket_id)`
- `tls_peer(session_id)` → `TLSPeerInfo`
- `tls_shutdown(session_id)` → `()`
- `tls_rekey(session_id, new_profile)` → `()`

#### TLS Profiles

- **`pqc_hybrid`**: Post-Quantum Cryptography hybrid ciphers (Kyber + AES, Dilithium + ChaCha20)
- **`tls13_modern`**: TLS 1.3 with modern ciphers (AES-256-GCM, ChaCha20-Poly1305)
- **`intranet_fast`**: Fast intranet configuration with mTLS requirement

#### Capability Model

- `net:tls`: Required for TLS operations
- `net:quic`: Required for QUIC operations
- `net:all`: Grants all networking capabilities

### 3.2. QUIC Daemon (`services/net/quic/src/quicd.rs`)

The QUIC daemon provides HTTP/3 support with broker RPC:

- `quic_listen(addr, profile, process_cap)` → `listener_id`
- `quic_connect(addr, profile, process_cap)` → `conn_id`
- `quic_accept(listener_id, process_cap)` → `conn_id`
- `quic_open_bidi(conn_id, process_cap)` → `stream_id`
- `quic_read(stream_id, max_bytes, process_cap)` → `data`
- `quic_write(stream_id, data, process_cap)` → `bytes_written`
- `quic_close_stream(stream_id, process_cap)` → `()`
- `quic_close_connection(conn_id, process_cap)` → `()`

### 3.3. Certificate Operations (`services/identity/src/certops.rs`)

Certificate management with PDV integration:

- **CSR Generation**: Create certificate signing requests with DID binding
- **Certificate Issuance**: Issue DID-bound certificates with broker-side signing
- **KeyVault Integration**: Private keys sealed in KeyVault, never exposed to applications
- **Certificate Caching**: Efficient caching with rotation and OCSP stapling

#### KeyVault Operations

- `generate_key_pair(key_type, allowed_ops, allowed_processes, expires_at)` → `key_handle`
- `sign(key_handle, data)` → `signature`
- `verify(key_handle, data, signature)` → `bool`

## 4. Security Model

### 4.1. Capability Enforcement

All TLS/QUIC operations require appropriate capabilities:

```rust
// Example capability check
if !process_cap.contains("net:tls") && !process_cap.contains("net:all") {
    return Err("TLS capability required".to_string());
}
```

### 4.2. Policy Gating

OPA/WASM policies control:

- **SNI allowlists**: Which server names are permitted
- **ALPN allowlists**: Which application protocols are allowed
- **Cipher suite policies**: Which cipher suites are permitted
- **Certificate pinning**: SPKI/fingerprint validation
- **mTLS requirements**: When mutual TLS is mandatory

### 4.3. Key Security

- **No raw private keys in app space**: All keys remain sealed in KeyVault
- **Broker-side operations**: Signing, decryption, and key exchange handled by broker
- **DID-bound certificates**: Certificates tied to decentralized identifiers
- **Hot revocation**: Capability revocation immediately terminates sessions

## 5. Polyglot Bindings

### 5.1. C Shim

```c
// TLS operations
int tls_wrap(int sockfd, const char* profile, const char* process_cap);
int tls_peer(int session_id, struct tls_peer_info* info);
int tls_shutdown(int session_id);

// QUIC operations
int quic_connect(const char* addr, const char* profile, const char* process_cap);
int quic_open_bidi(int conn_id, const char* process_cap);
ssize_t quic_write(int stream_id, const void* data, size_t len, const char* process_cap);
ssize_t quic_read(int stream_id, void* buffer, size_t max_len, const char* process_cap);
```

### 5.2. Go (`go/tooling/netctl`)

```bash
# TLS commands
netctl tls-echo --listen :8443 --profile pqc_hybrid
netctl tls-client --addr 127.0.0.1:8443 --verify-sni demo.local

# QUIC commands
netctl quic-echo --listen :9443 --profile tls13_modern
netctl quic-client --addr 127.0.0.1:9443 --alpn h3
```

### 5.3. Node.js (TypeScript N-API)

```typescript
// TLS operations
const tlsSession = await aetherisTLS.wrap(socket, 'tls13_modern', processCap);
const peerInfo = await tlsSession.getPeer();
await tlsSession.shutdown();

// QUIC operations
const quicConn = await aetherisQUIC.connect('127.0.0.1:9443', 'tls13_modern', processCap);
const stream = await quicConn.openBidi();
await stream.write(Buffer.from('Hello QUIC!'));
const response = await stream.read(1024);
```

### 5.4. Rust (`rust/crates/aetheris-net`)

```rust
// TLS operations
let tls_session = aetheris_net::tls::wrap(socket_id, "tls13_modern", process_cap).await?;
let peer_info = tls_session.peer().await?;
tls_session.shutdown().await?;

// QUIC operations
let quic_conn = aetheris_net::quic::connect("127.0.0.1:9443", "tls13_modern", process_cap).await?;
let stream = quic_conn.open_bidi().await?;
stream.write(b"Hello QUIC!").await?;
let response = stream.read(1024).await?;
```

## 6. Performance Targets

### 6.1. TLS Performance

- **Handshake latency (p95)**: ≤ 80ms
- **Rekey latency (p95)**: ≤ 50ms
- **Session resume ratio**: ≥ 80%
- **mTLS overhead**: ≤ 20% additional latency

### 6.2. QUIC Performance

- **Connection establishment (p95)**: ≤ 20ms
- **Stream creation (p95)**: ≤ 5ms
- **Round-trip time (p95)**: ≤ 2ms (loopback)
- **Throughput**: ≥ 2 Gbps (loopback)

### 6.3. Certificate Operations

- **CSR generation**: ≤ 10ms
- **Certificate issuance**: ≤ 100ms
- **KeyVault operations**: ≤ 5ms per operation

## 7. Policy Configuration

### 7.1. TLS Policy Example

```rego
package tls

default allow = false

allow {
    input.operation == "connect"
    input.sni in data.tls.allowed_snis
    input.cipher_suite in data.tls.allowed_ciphers
    input.tls_version >= "1.3"
}

allowed_snis = [
    "demo.local",
    "api.example.com",
    "*.internal.company.com"
]

allowed_ciphers = [
    "TLS_AES_256_GCM_SHA384",
    "TLS_AES_128_GCM_SHA256",
    "TLS_AES_256_GCM_SHA384_PQC"
]
```

### 7.2. QUIC Policy Example

```rego
package quic

default allow = false

allow {
    input.operation == "connect"
    input.alpn in data.quic.allowed_alpn
    input.profile in data.quic.allowed_profiles
}

allowed_alpn = ["h3", "hq"]
allowed_profiles = ["tls13_modern", "pqc_hybrid"]
```

## 8. Monitoring and Observability

### 8.1. Metrics

- **TLS Metrics**: Handshake latency, cipher suite distribution, mTLS ratio, policy denials
- **QUIC Metrics**: Connection latency, stream creation rate, throughput, backpressure events
- **Certificate Metrics**: Issuance rate, cache hit ratio, KeyVault operation latency

### 8.2. Audit Logging

All TLS/QUIC operations are audited:

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "event_type": "tls_handshake",
  "process_cap": "net:tls",
  "session_id": "tls_session_123",
  "profile": "tls13_modern",
  "sni": "demo.local",
  "cipher_suite": "TLS_AES_256_GCM_SHA384",
  "handshake_time_ms": 45.2,
  "policy_decision": "allow"
}
```

## 9. Build and Run Instructions

### 9.1. Build Targets

```bash
# Core services
bazel build //services/net/posixnet:posixnet
bazel build //services/net/quic:quicd
bazel build //services/identity:certops

# Polyglot bindings
bazel build //c/libc_aetheris:net_shim
bazel build //go/tooling/netctl:netctl
bazel build //rust/crates/aetheris-net:aetheris_net
bazel build //tooling/ts/node-posix-bridge:net

# Validation
bazel build //tooling/python:tls_quic_check
```

### 9.2. Run Examples

```bash
# Start services
bazel run //services/net/posixnet:posixnet &
bazel run //services/net/quic:quicd &

# TLS echo server/client
bazel run //go/tooling/netctl:netctl -- tls-echo --listen :8443 --profile pqc_hybrid
bazel run //go/tooling/netctl:netctl -- tls-client --addr 127.0.0.1:8443 --verify-sni demo.local

# QUIC echo server/client
bazel run //go/tooling/netctl:netctl -- quic-echo --listen :9443 --profile tls13_modern
bazel run //go/tooling/netctl:netctl -- quic-client --addr 127.0.0.1:9443 --alpn h3

# Validation
bazel run //tooling/python:tls_quic_check
```

## 10. Testing and Validation

### 10.1. Unit Tests

- TLS broker functionality
- QUIC daemon operations
- Certificate operations
- Policy enforcement
- Capability checks

### 10.2. Integration Tests

- End-to-end TLS handshake
- QUIC connection and streaming
- mTLS authentication
- Policy denial scenarios
- Performance benchmarks

### 10.3. Security Tests

- Capability enforcement
- Policy bypass attempts
- Certificate validation
- KeyVault security
- Audit logging verification

## 11. Migration and Compatibility

### 11.1. Backward Compatibility

- Existing P4-03-A1 socket operations remain unchanged
- TLS/QUIC features are additive and opt-in
- No breaking changes to existing APIs

### 11.2. Feature Flags

- `AETH_TLS_ENABLED`: Enable TLS broker
- `AETH_QUIC_ENABLED`: Enable QUIC daemon
- `AETH_PQC_ENABLED`: Enable PQC cipher suites
- `AETH_MTLS_REQUIRED`: Require mTLS for all connections

## 12. Future Enhancements

### 12.1. Planned Features

- **Certificate transparency**: Integration with CT logs
- **OCSP stapling**: Real-time certificate status
- **Session resumption**: Efficient session reuse
- **0-RTT support**: Early data transmission
- **Connection migration**: Seamless connection handoff

### 12.2. Performance Optimizations

- **Hardware acceleration**: AES-NI, AVX-512 support
- **Zero-copy I/O**: Reduced memory copies
- **Connection pooling**: Reuse of established connections
- **Predictive pre-connection**: Proactive connection establishment

## 13. Success Criteria

The P4-03-A2 implementation is considered successful when:

- ✅ All TLS profiles (pqc_hybrid, tls13_modern, intranet_fast) work correctly
- ✅ QUIC connections and streams operate within performance targets
- ✅ mTLS authentication functions properly with DID-bound certificates
- ✅ Policy enforcement blocks unauthorized operations
- ✅ All polyglot bindings (C, Go, Node.js, Rust) pass equivalent tests
- ✅ Performance baselines are met (handshake p95 ≤ 80ms, QUIC RTT p95 ≤ 2ms)
- ✅ Security requirements are satisfied (no raw keys in app space)
- ✅ Audit logging captures all significant events

## 14. Success Banner

```
[POSIX NET/A2] TLS+mTLS+QUIC OK | handshake p95<=80ms | QUIC RTT p95<=2ms | mTLS=on | keys=brokered | policy=OPA/WASM | caps=enforced
```

This completes the P4-03-A2 implementation, providing a solid foundation for advanced networking features in Aetheris OS.
