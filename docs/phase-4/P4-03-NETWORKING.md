# P4-03: POSIX Networking & Namespaces

## Overview

This document describes the implementation of brokered, policy-gated POSIX networking and namespaces for Aetheris OS. The networking subsystem provides capability-aware, secure networking with support for sockets, namespaces, TLS/mTLS, QUIC, and libp2p overlay networking.

## Architecture

### Core Components

1. **Socket Broker** (`services/net/posixnet`)
   - Mediates all network operations through capability-based access control
   - Supports AF_INET/AF_INET6, SOCK_STREAM/SOCK_DGRAM
   - Non-blocking I/O with edge-triggered readiness
   - Emulates select/poll/epoll atop broker event loop

2. **Network Namespaces** (`services/net/posixnet/src/ns.rs`)
   - Per-process network isolation
   - Classes: none, local, mesh, wan
   - Policy-gated egress with bandwidth limits
   - DNS policy enforcement

3. **Policy Engine** (`services/net/posixnet/src/policy.rs`)
   - OPA (Open Policy Agent) integration
   - WASM-based policy evaluation
   - Hot-revocation with SIGCAP_REVOKED_NET
   - Deny-by-default with capability enforcement

4. **TLS/mTLS Support** (`services/net/posixnet/src/tls.rs`)
   - PQC (Post-Quantum Cryptography) support
   - DID-bound certificates from PDV
   - KeyVault integration (no raw key exposure)
   - SNI/domain pinning with cipher suite allowlists

5. **QUIC/HTTP3** (`services/net/quic`)
   - Quinn-based QUIC implementation
   - Stream multiplexing with backpressure
   - Broker-mediated read/write operations
   - Audit logging for all operations

6. **libp2p Overlay** (`services/polynet/src/overlay.rs`)
   - Mesh networking with DID identity
   - NAT traversal via UDP hole punching
   - Gossipsub for control topics
   - Peer discovery and routing

### Polyglot Bindings

1. **C Shim** (`c/libc_aetheris/src/net.c`)
   - Standard socket API compatibility
   - Broker RPC integration
   - Poll/select emulation

2. **Go Integration** (`go/posixshim/net.go`)
   - netpoll hooking to broker
   - Environment variable activation (`AETH_NET=broker`)
   - Transparent broker mediation

3. **Node.js Bindings** (`tooling/ts/node-posix-bridge/src/net.ts`)
   - N-API module for net.Socket/dgram
   - Async broker communication
   - Optional QUIC binding

4. **Rust Crate** (`rust/crates/aetheris-net`)
   - Async traits over broker channels
   - Feature-flagged broker integration
   - High-performance networking primitives

## API Reference

### Socket Broker API

#### Core Operations

```rust
// Create socket
let socket_id = broker.socket(AF_INET, SOCK_STREAM, 0, process_cap, namespace).await?;

// Bind socket
broker.bind(socket_id, address, process_cap).await?;

// Listen
broker.listen(socket_id, backlog, process_cap).await?;

// Accept connection
let new_socket = broker.accept(socket_id, process_cap).await?;

// Connect
broker.connect(socket_id, address, process_cap).await?;

// Send/Receive
let bytes_sent = broker.send(socket_id, data, flags, process_cap).await?;
let data = broker.recv(socket_id, buffer_size, flags, process_cap).await?;

// Close
broker.close(socket_id, process_cap).await?;
```

#### Polling Operations

```rust
// Poll sockets
let events = broker.poll(sockets, timeout, process_cap).await?;

// Select emulation
let ready_fds = broker.select(read_fds, write_fds, except_fds, timeout, process_cap).await?;
```

### Network Namespaces

#### Namespace Management

```rust
// Create namespace
let namespace = NetworkNamespace::new(
    "demo".to_string(),
    "Demo Namespace".to_string(),
    NamespaceClass::Local,
    config
);

// Set process namespace
namespace_manager.set_process_namespace(process_cap, namespace_id).await?;

// Check address access
let allowed = namespace.is_address_allowed(&address);
```

#### Namespace Classes

- **none**: No network access
- **local**: Loopback and localhost only
- **mesh**: libp2p overlay networking
- **wan**: Full internet access (policy-gated)

### Policy Engine

#### Policy Rules

```rego
package aetheris.net

# Allow local network access
allow {
    input.operation == "connect"
    input.target_address =~ "127\\..*"
}

# Deny WAN access without capability
deny {
    input.operation == "connect"
    input.target_address !~ "127\\..*"
    not input.process_cap[_] == "net:wan"
}
```

#### Policy Evaluation

```rust
// Check policy
let decision = policy_engine.check_connect(process_cap, address).await?;
if !decision.allowed {
    return Err(NetError::PolicyDenied(decision.reason));
}
```

### TLS/mTLS

#### TLS Configuration

```rust
let tls_config = TLSConfig {
    tls_version: TLSVersion::TLSv13,
    enable_pqc: true,
    enable_mtls: true,
    cert_file: Some("cert.pem".to_string()),
    key_file: Some("key.pem".to_string()),
    verify_peer: true,
};
```

#### TLS Operations

```rust
// Wrap socket with TLS
let tls_session = tls_manager.wrap_socket(socket_id, tls_config).await?;

// Get peer certificate
let peer_cert = tls_manager.get_peer_certificate(session_id).await?;

// Verify peer
let verified = tls_manager.verify_peer_certificate(session_id, hostname).await?;
```

### QUIC/HTTP3

#### QUIC Operations

```rust
// Create QUIC connection
let connection = quic_daemon.create_connection(address, sni, process_cap, namespace).await?;

// Create stream
let stream = quic_daemon.create_stream(connection_id, StreamType::Bidirectional, process_cap).await?;

// Send/Receive data
let bytes_sent = quic_daemon.send_data(stream_id, data, process_cap).await?;
let data = quic_daemon.receive_data(stream_id, buffer_size, process_cap).await?;
```

### libp2p Overlay

#### Overlay Operations

```rust
// Create overlay node
let overlay = OverlayNode::new(config, did_peer_id)?;

// Start overlay
overlay.start().await?;

// Connect to peer
overlay.connect_to_peer(peer_addr).await?;

// Publish message
overlay.publish_message(topic, message).await?;

// Subscribe to topic
overlay.subscribe_to_topic(topic).await?;
```

## Configuration

### Broker Configuration

```toml
[broker]
max_sockets_per_process = 1024
default_buffer_size = 65536
connection_timeout = "30s"
listen_backlog = 128
enable_tls = true
enable_quic = true
enable_libp2p = true

[policy]
enable_policy = true
default_decision = false
cache_size = 1000
```

### Namespace Configuration

```toml
[namespace.local]
class = "local"
allowed_hosts = ["127.0.0.0/8", "::1/128"]
allowed_ports = []
allowed_protocols = ["tcp", "udp"]
egress_budget_bps = 100000000
connection_rate = 1000

[namespace.mesh]
class = "mesh"
allowed_hosts = ["10.0.0.0/8", "fd00::/8"]
allowed_protocols = ["tcp", "udp", "quic"]
enable_libp2p = true
```

### TLS Configuration

```toml
[tls]
tls_version = "1.3"
enable_pqc = true
enable_mtls = true
cert_file = "/etc/aetheris/certs/server.pem"
key_file = "/etc/aetheris/keys/server.key"
ca_file = "/etc/aetheris/certs/ca.pem"
verify_client_cert = true
verify_server_cert = true
```

## Usage Examples

### Basic Socket Operations

```c
// C example
int sockfd = socket(AF_INET, SOCK_STREAM, 0);
struct sockaddr_in addr = {0};
addr.sin_family = AF_INET;
addr.sin_port = htons(8080);
addr.sin_addr.s_addr = inet_addr("127.0.0.1");

bind(sockfd, (struct sockaddr*)&addr, sizeof(addr));
listen(sockfd, 128);

int clientfd = accept(sockfd, NULL, NULL);
send(clientfd, "Hello", 5, 0);
close(clientfd);
close(sockfd);
```

```go
// Go example
package main

import (
    "net"
    "os"
)

func main() {
    os.Setenv("AETH_NET", "broker")
    
    listener, err := net.Listen("tcp", ":8080")
    if err != nil {
        panic(err)
    }
    
    for {
        conn, err := listener.Accept()
        if err != nil {
            continue
        }
        
        go handleConnection(conn)
    }
}

func handleConnection(conn net.Conn) {
    defer conn.Close()
    conn.Write([]byte("Hello from Aetheris OS"))
}
```

```typescript
// Node.js example
import { initializeBroker, BrokerSocket } from './net';

async function main() {
    await initializeBroker();
    
    const server = new BrokerSocket();
    await server.listen(8080);
    
    while (true) {
        const client = await server.accept();
        await client.write('Hello from Aetheris OS');
        await client.close();
    }
}

main().catch(console.error);
```

```rust
// Rust example
use aetheris_net::{initialize, tcp_listener, tcp_stream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize().await?;
    
    let listener = tcp_listener("0.0.0.0:8080").await?;
    
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            let _ = stream.write_all(b"Hello from Aetheris OS").await;
        });
    }
    
    Ok(())
}
```

### Network Namespace Management

```bash
# Create namespace
netctl ns create demo --class local

# List namespaces
netctl ns list

# Set process namespace
netctl ns set process123 demo

# Delete namespace
netctl ns delete demo
```

### Policy Management

```bash
# Show policies
netctl policy show

# Revoke capability
netctl cap revoke process123 net:wan

# Show policy decisions
netctl policy decisions
```

### TLS/mTLS Operations

```bash
# Generate certificate
netctl tls generate --subject "CN=example.com" --validity 365

# Test TLS connection
netctl tls test --host example.com --port 443

# Show TLS sessions
netctl tls sessions
```

### QUIC Operations

```bash
# Start QUIC server
netctl quic server --listen :8443 --cert cert.pem --key key.pem

# Connect QUIC client
netctl quic client --addr 127.0.0.1:8443

# Show QUIC connections
netctl quic connections
```

### libp2p Overlay

```bash
# Start overlay node
netctl mesh start --listen :4001

# Connect to peer
netctl mesh connect /ip4/192.168.1.100/tcp/4001/p2p/QmPeerId

# Publish message
netctl mesh publish --topic "test" --message "Hello mesh"

# Show peers
netctl mesh peers
```

## Performance Targets

### Latency Targets

- **Socket Creation**: ≤ 100µs p50, ≤ 500µs p95
- **Connection Establishment**: ≤ 1ms p50, ≤ 5ms p95
- **Data Transfer**: ≤ 50µs p50, ≤ 200µs p95
- **Policy Evaluation**: ≤ 10µs p50, ≤ 50µs p95

### Throughput Targets

- **Loopback**: ≥ 2 Gbps
- **Local Network**: ≥ 1 Gbps
- **WAN (policy-gated)**: ≥ 100 Mbps
- **QUIC Streams**: ≥ 500 Mbps

### Scalability Targets

- **Concurrent Sockets**: ≥ 10,000 per process
- **Concurrent Connections**: ≥ 1,000 per namespace
- **Policy Rules**: ≥ 1,000 active rules
- **Namespace Isolation**: 100% capability enforcement

## Security Considerations

### Capability Enforcement

- All network operations require appropriate capabilities
- Capabilities are checked at broker level
- Hot-revocation supported with immediate effect
- Audit logging for all capability decisions

### Policy Enforcement

- Deny-by-default policy
- OPA-based policy evaluation
- WASM-compiled policies for performance
- Policy versioning and rollback support

### Key Management

- All TLS keys stored in KeyVault
- No raw key exposure to applications
- DID-bound certificates
- PQC algorithm support

### Network Isolation

- Per-process namespace isolation
- Bandwidth limiting and rate limiting
- DNS policy enforcement
- Protocol allowlisting

## Monitoring and Observability

### Metrics

- Socket creation/destruction rates
- Connection establishment rates
- Data transfer rates
- Policy decision rates
- Error rates by type

### Logging

- All network operations logged
- Policy decisions logged
- Capability grants/revocations logged
- Security events logged

### Tracing

- Request tracing across broker
- Performance profiling
- Dependency mapping
- Error correlation

## Testing

### Unit Tests

```bash
# Run unit tests
cargo test --package posixnet
cargo test --package quic
cargo test --package polynet
```

### Integration Tests

```bash
# Run integration tests
cargo test --test integration
```

### Performance Tests

```bash
# Run performance benchmarks
cargo bench --package posixnet
cargo bench --package quic
```

### Security Tests

```bash
# Run security tests
cargo test --test security
```

## Deployment

### Service Startup

```bash
# Start networking services
systemctl start aetheris-posixnet
systemctl start aetheris-quicd
systemctl start aetheris-polynet
```

### Configuration

```bash
# Apply configuration
aetheris-ctl config apply /etc/aetheris/networking.toml
```

### Monitoring

```bash
# Check service status
systemctl status aetheris-posixnet
systemctl status aetheris-quicd
systemctl status aetheris-polynet

# View logs
journalctl -u aetheris-posixnet -f
```

## Troubleshooting

### Common Issues

1. **Socket Creation Fails**
   - Check process capabilities
   - Verify namespace configuration
   - Check resource limits

2. **Connection Timeout**
   - Verify network policy
   - Check firewall rules
   - Verify DNS resolution

3. **TLS Handshake Fails**
   - Check certificate validity
   - Verify key permissions
   - Check cipher suite support

4. **QUIC Connection Fails**
   - Check QUIC service status
   - Verify port availability
   - Check NAT traversal

5. **Policy Denial**
   - Review policy rules
   - Check process capabilities
   - Verify namespace class

### Debug Commands

```bash
# Show broker status
netctl broker status

# Show namespace details
netctl ns show <namespace>

# Show socket details
netctl socket show <socket_id>

# Show policy decisions
netctl policy decisions --process <process_cap>

# Show TLS sessions
netctl tls sessions

# Show QUIC connections
netctl quic connections

# Show mesh peers
netctl mesh peers
```

### Log Analysis

```bash
# Filter network logs
journalctl -u aetheris-posixnet | grep "SOCKET_CREATED"

# Filter policy logs
journalctl -u aetheris-posixnet | grep "POLICY_DECISION"

# Filter error logs
journalctl -u aetheris-posixnet | grep "ERROR"
```

## Future Enhancements

### Planned Features

1. **Advanced Load Balancing**
   - Layer 4/7 load balancing
   - Health checking
   - Circuit breakers

2. **Service Mesh Integration**
   - Istio compatibility
   - mTLS between services
   - Traffic management

3. **Advanced Monitoring**
   - Prometheus metrics
   - Grafana dashboards
   - Distributed tracing

4. **Performance Optimizations**
   - Zero-copy networking
   - Kernel bypass options
   - Hardware acceleration

### Research Areas

1. **Quantum-Safe Networking**
   - Post-quantum cryptography
   - Quantum key distribution
   - Quantum-resistant protocols

2. **AI-Enhanced Networking**
   - Intelligent routing
   - Anomaly detection
   - Predictive scaling

3. **Edge Computing**
   - Edge-to-edge communication
   - Latency optimization
   - Bandwidth management

## Conclusion

The Aetheris OS networking subsystem provides a comprehensive, secure, and high-performance networking solution with strong isolation, policy enforcement, and polyglot support. The broker-mediated architecture ensures that all network operations are capability-aware and auditable, while the namespace system provides strong isolation between processes.

The implementation supports modern networking protocols including TLS 1.3, QUIC/HTTP3, and libp2p overlay networking, making it suitable for a wide range of applications from traditional client-server to peer-to-peer and mesh networking scenarios.
