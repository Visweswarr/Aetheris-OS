# P4-03-A1: POSIX Sockets and Readiness APIs

## Overview

This document describes the implementation of core POSIX socket operations and readiness APIs (select/poll/epoll emulation) for Aetheris OS. This is the first phase (A1) of the broader P4-03 networking implementation, focusing on essential socket functionality with capability enforcement and policy gating.

## Architecture

### Core Components

1. **Socket Broker** (`services/net/posixnet`)
   - Mediates all socket operations through capability-based access control
   - Implements standard POSIX socket APIs: `socket()`, `bind()`, `listen()`, `accept()`, `connect()`, `send()`, `recv()`, `close()`
   - Provides non-blocking I/O support with readiness event notifications
   - Enforces network policies and audits all operations

2. **Readiness Manager** (`services/net/posixnet/src/readiness.rs`)
   - Emulates `select()`, `poll()`, and `epoll()` APIs
   - Supports both edge-triggered and level-triggered modes
   - Manages readiness subscriptions and event delivery
   - Provides unified readiness events across different APIs

3. **Syscall Broker** (`services/net/posixnet/src/syscall.rs`)
   - Processes syscall requests from polyglot shims
   - Validates capabilities and enforces policies
   - Routes requests to appropriate handlers
   - Returns standardized responses with error codes

4. **Audit Logger** (`services/net/posixnet/src/audit.rs`)
   - Logs all network operations for security and debugging
   - Supports different audit levels (Debug, Info, Warning, Error, Critical)
   - Provides structured logging with JSON export
   - Maintains configurable audit log size limits

### Polyglot Bindings

1. **C Shim** (`c/libc_aetheris/src/net.c`)
   - Provides POSIX-compatible socket API
   - Forwards all operations to the networking broker
   - Maintains socket state and file descriptor mappings
   - Supports `socket()`, `bind()`, `listen()`, `accept()`, `connect()`, `send()`, `recv()`, `close()`, `fcntl()`, `select()`, `poll()`, `epoll_*()`, `getsockopt()`, `setsockopt()`, `shutdown()`

2. **Go Tools** (`go/tooling/netctl/`)
   - `netctl` command-line tool for network management
   - `echo-server` and `echo-client` for testing
   - `sockets` command to list active sockets
   - `capabilities` command to show network capabilities
   - `audit` command to view audit logs
   - `stats` command to show network statistics

3. **Node.js Bridge** (`tooling/ts/node-posix-bridge/src/net.ts`)
   - TypeScript bindings for Node.js applications
   - `AetherisTCPSocket` and `AetherisUDPSocket` classes
   - Event-driven API compatible with Node.js patterns
   - Support for non-blocking I/O and readiness events

4. **Rust Crate** (`rust/crates/aetheris-net/`)
   - `NetworkBroker` client for Rust applications
   - Async/await API for socket operations
   - Type-safe socket management
   - Integration with Tokio runtime

## API Reference

### Socket Operations

#### `socket(domain, type, protocol)`
Creates a new socket with the specified domain, type, and protocol.

**Parameters:**
- `domain`: `AF_INET` (IPv4) or `AF_INET6` (IPv6)
- `type`: `SOCK_STREAM` (TCP) or `SOCK_DGRAM` (UDP)
- `protocol`: Protocol number (usually 0)

**Returns:** File descriptor on success, -1 on error

**Capability Required:** `net:socket`

#### `bind(sockfd, addr, addrlen)`
Binds a socket to a local address.

**Parameters:**
- `sockfd`: Socket file descriptor
- `addr`: Local address structure
- `addrlen`: Length of address structure

**Returns:** 0 on success, -1 on error

**Capability Required:** `net:socket`

#### `listen(sockfd, backlog)`
Marks a socket as listening for incoming connections.

**Parameters:**
- `sockfd`: Socket file descriptor
- `backlog`: Maximum number of pending connections

**Returns:** 0 on success, -1 on error

**Capability Required:** `net:socket`

#### `accept(sockfd, addr, addrlen)`
Accepts an incoming connection on a listening socket.

**Parameters:**
- `sockfd`: Listening socket file descriptor
- `addr`: Address structure for client address (optional)
- `addrlen`: Length of address structure

**Returns:** New socket file descriptor on success, -1 on error

**Capability Required:** `net:socket`

#### `connect(sockfd, addr, addrlen)`
Establishes a connection to a remote address.

**Parameters:**
- `sockfd`: Socket file descriptor
- `addr`: Remote address structure
- `addrlen`: Length of address structure

**Returns:** 0 on success, -1 on error

**Capability Required:** `net:socket`

#### `send(sockfd, buf, len, flags)`
Sends data on a connected socket.

**Parameters:**
- `sockfd`: Socket file descriptor
- `buf`: Data buffer
- `len`: Length of data
- `flags`: Send flags

**Returns:** Number of bytes sent on success, -1 on error

**Capability Required:** `net:socket`

#### `recv(sockfd, buf, len, flags)`
Receives data from a connected socket.

**Parameters:**
- `sockfd`: Socket file descriptor
- `buf`: Buffer for received data
- `len`: Buffer length
- `flags`: Receive flags

**Returns:** Number of bytes received on success, -1 on error

**Capability Required:** `net:socket`

#### `close(sockfd)`
Closes a socket and frees associated resources.

**Parameters:**
- `sockfd`: Socket file descriptor

**Returns:** 0 on success, -1 on error

**Capability Required:** `net:socket`

### Readiness APIs

#### `select(nfds, readfds, writefds, exceptfds, timeout)`
Monitors multiple file descriptors for readiness.

**Parameters:**
- `nfds`: Highest file descriptor number + 1
- `readfds`: Set of file descriptors to monitor for reading
- `writefds`: Set of file descriptors to monitor for writing
- `exceptfds`: Set of file descriptors to monitor for exceptions
- `timeout`: Timeout value

**Returns:** Number of ready file descriptors on success, -1 on error

#### `poll(fds, nfds, timeout)`
Monitors multiple file descriptors for readiness.

**Parameters:**
- `fds`: Array of pollfd structures
- `nfds`: Number of file descriptors
- `timeout`: Timeout in milliseconds

**Returns:** Number of ready file descriptors on success, -1 on error

#### `epoll_create(size)`
Creates an epoll instance.

**Parameters:**
- `size`: Hint for the number of file descriptors (ignored)

**Returns:** Epoll file descriptor on success, -1 on error

#### `epoll_ctl(epfd, op, fd, event)`
Controls an epoll instance.

**Parameters:**
- `epfd`: Epoll file descriptor
- `op`: Operation (EPOLL_CTL_ADD, EPOLL_CTL_MOD, EPOLL_CTL_DEL)
- `fd`: File descriptor to control
- `event`: Event structure

**Returns:** 0 on success, -1 on error

#### `epoll_wait(epfd, events, maxevents, timeout)`
Waits for events on an epoll instance.

**Parameters:**
- `epfd`: Epoll file descriptor
- `events`: Array to store events
- `maxevents`: Maximum number of events
- `timeout`: Timeout in milliseconds

**Returns:** Number of events on success, -1 on error

### Socket Options

#### `getsockopt(sockfd, level, optname, optval, optlen)`
Gets a socket option value.

**Parameters:**
- `sockfd`: Socket file descriptor
- `level`: Option level (SOL_SOCKET, etc.)
- `optname`: Option name
- `optval`: Buffer for option value
- `optlen`: Length of buffer

**Returns:** 0 on success, -1 on error

#### `setsockopt(sockfd, level, optname, optval, optlen)`
Sets a socket option value.

**Parameters:**
- `sockfd`: Socket file descriptor
- `level`: Option level (SOL_SOCKET, etc.)
- `optname`: Option name
- `optval`: Option value buffer
- `optlen`: Length of value

**Returns:** 0 on success, -1 on error

**Supported Options:**
- `SO_NONBLOCK`: Non-blocking mode
- `SO_REUSEADDR`: Address reuse
- `SO_KEEPALIVE`: Keep-alive
- `SO_BROADCAST`: Broadcast
- `SO_RCVBUF`: Receive buffer size
- `SO_SNDBUF`: Send buffer size
- `SO_ERROR`: Socket error

## Capability Model

### Network Capabilities

- `net:socket`: Basic socket operations (create, bind, listen, accept, connect, send, recv, close)
- `net:all`: All network operations including advanced features
- `net:limited`: Limited network operations (read-only, local-only)

### Capability Enforcement

All socket operations are checked against the process's network capabilities:

1. **Socket Creation**: Requires `net:socket` capability
2. **Binding**: Requires `net:socket` capability
3. **Listening**: Requires `net:socket` capability
4. **Accepting**: Requires `net:socket` capability
5. **Connecting**: Requires `net:socket` capability
6. **Sending/Receiving**: Requires `net:socket` capability
7. **Closing**: Requires `net:socket` capability

### Policy Enforcement

Network policies are enforced at the broker level:

1. **Address Restrictions**: Policies can restrict which addresses can be bound or connected to
2. **Port Restrictions**: Policies can restrict which ports can be used
3. **Protocol Restrictions**: Policies can restrict which protocols can be used
4. **Rate Limiting**: Policies can limit connection rates and bandwidth
5. **DNS Policy**: Policies can control DNS resolution

## Performance Targets

### Latency Targets

- **Socket Creation**: ≤ 100µs p50, ≤ 500µs p95
- **Connection Establishment**: ≤ 1ms p50, ≤ 5ms p95
- **Data Transfer**: ≤ 50µs p50, ≤ 200µs p95
- **Readiness Events**: ≤ 10µs p50, ≤ 50µs p95

### Throughput Targets

- **Loopback TCP**: ≥ 2 Gbps
- **Loopback UDP**: ≥ 2 Gbps
- **Concurrent Connections**: ≥ 10,000
- **Readiness Events**: ≥ 100,000 events/second

### Memory Usage

- **Per Socket**: ≤ 1 KB
- **Broker Overhead**: ≤ 100 KB
- **Audit Log**: ≤ 10 MB (configurable)

## Testing

### Unit Tests

```bash
# Run Rust unit tests
cargo test --package posixnet

# Run Go tests
go test ./go/tooling/netctl/...

# Run Node.js tests
npm test --prefix tooling/ts/node-posix-bridge
```

### Integration Tests

```bash
# Test echo server/client
netctl echo-server --tcp :8080 &
netctl echo-client --tcp 127.0.0.1:8080 --message "hello" --count 10

# Test socket listing
netctl sockets

# Test capabilities
netctl capabilities

# Test audit logging
netctl audit --count 20
```

### Performance Tests

```bash
# Benchmark TCP echo
netctl echo-server --tcp :8080 &
netctl echo-client --tcp 127.0.0.1:8080 --message "test" --count 1000

# Benchmark UDP echo
netctl echo-server --udp :8080 &
netctl echo-client --udp 127.0.0.1:8080 --message "test" --count 1000
```

## Configuration

### Broker Configuration

```toml
[broker]
max_sockets_per_process = 1024
default_buffer_size = 65536
connection_timeout = "30s"
listen_backlog = 128
enable_audit = true
audit_log_size = 10000
```

### Policy Configuration

```rego
package aetheris.net.policy

default allow = false

allow {
    input.operation == "socket"
    input.capability == "net:socket"
}

allow {
    input.operation == "bind"
    input.address == "127.0.0.1"
    input.port >= 1024
}

allow {
    input.operation == "connect"
    input.address == "127.0.0.1"
    input.port >= 1024
}
```

## Security Considerations

### Capability Isolation

- Each process has its own network capabilities
- Capabilities are checked on every network operation
- No raw host file descriptors are exposed to applications
- All network operations are brokered and audited

### Policy Enforcement

- Deny-by-default policy for all network operations
- Policies are evaluated using OPA (Open Policy Agent)
- Policies can be updated without restarting the system
- All policy decisions are logged for audit

### Audit Logging

- All network operations are logged with timestamps
- Audit logs include process capabilities and operation details
- Logs can be exported in JSON or CSV format
- Configurable log retention and size limits

## Troubleshooting

### Common Issues

1. **Permission Denied**: Check process network capabilities
2. **Address In Use**: Check if port is already bound
3. **Connection Refused**: Check if target is listening
4. **Timeout**: Check network connectivity and policies
5. **Resource Exhausted**: Check socket limits and memory usage

### Debugging

```bash
# Check active sockets
netctl sockets --verbose

# Check audit log
netctl audit --level error

# Check network statistics
netctl stats --json

# Check capabilities
netctl capabilities --process <process_cap>
```

### Performance Tuning

1. **Buffer Sizes**: Adjust `SO_RCVBUF` and `SO_SNDBUF`
2. **Non-blocking I/O**: Use `SO_NONBLOCK` for better performance
3. **Keep-alive**: Enable `SO_KEEPALIVE` for long-lived connections
4. **Address Reuse**: Enable `SO_REUSEADDR` for server sockets

## Future Enhancements

### P4-03-A2: Network Namespaces

- Per-process network namespaces
- Namespace isolation and routing
- Cross-namespace communication policies

### P4-03-A3: Advanced Networking

- TLS/SSL support with PQC algorithms
- QUIC protocol support
- libp2p overlay networking
- Advanced policy features

### P4-03-A4: Performance Optimization

- Zero-copy networking
- Kernel bypass for high-performance applications
- Hardware acceleration support
- Advanced load balancing

## References

- [POSIX.1-2017 Socket Interface](https://pubs.opengroup.org/onlinepubs/9699919799/functions/socket.html)
- [Linux epoll Interface](https://man7.org/linux/man-pages/man7/epoll.7.html)
- [Open Policy Agent](https://www.openpolicyagent.org/)
- [Aetheris OS Capability Model](../P4-01-POSIX-SURFACE.md#capability-model)
