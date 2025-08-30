# IPC Module Design Documentation - Polymera OS

## Overview

The Inter-Process Communication (IPC) module provides a comprehensive communication framework for Polymera OS, enabling secure and efficient message passing between processes. The implementation includes message types, queues, channels, inboxes, and a complete system call interface.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     IPC Architecture                        │
├─────────────────────────────────────────────────────────────┤
│  System Calls Layer (sys.rs)                               │
│  ├─ IpcSyscall enum                                        │
│  ├─ IpcSyscallHandler                                      │
│  └─ Convenience functions                                  │
├─────────────────────────────────────────────────────────────┤
│  Channel Management Layer (queues.rs)                      │
│  ├─ ChannelManager                                         │
│  ├─ Channel                                                │
│  ├─ MessageQueue                                           │
│  └─ Inbox                                                  │
├─────────────────────────────────────────────────────────────┤
│  Message Types Layer (types.rs)                            │
│  ├─ Message                                                │
│  ├─ MessageHeader                                          │
│  ├─ MessagePayload                                         │
│  └─ Process/Channel/Message IDs                            │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Message Types (types.rs)

#### Message Structure

```rust
pub struct Message {
    pub header: MessageHeader,
    pub payload: MessagePayload,
}
```

**Key Features:**
- **Complete message encapsulation** with header and payload
- **Type safety** with strongly typed IDs and enums
- **Integrity verification** with checksums
- **Timestamp tracking** for message aging
- **Priority support** for message ordering

#### Message Header

```rust
pub struct MessageHeader {
    pub id: MessageId,
    pub sender: ProcessId,
    pub receiver: ProcessId,
    pub msg_type: MessageType,
    pub priority: MessagePriority,
    pub flags: MessageFlags,
    pub payload_size: usize,
    pub timestamp: u64,
    pub request_id: Option<MessageId>,
    pub timeout: Option<u64>,
    pub sequence: u64,
    pub checksum: u32,
}
```

**Features:**
- **Unique identification** with MessageId
- **Process identification** with ProcessId types
- **Message classification** with MessageType enum
- **Priority handling** with MessagePriority levels
- **Flags system** for message properties (ACK, urgent, secure, etc.)
- **Request-response correlation** with request_id
- **Timeout support** for time-limited messages
- **Integrity checking** with checksums

#### Message Types

```rust
pub enum MessageType {
    Request,        // Expects response
    Response,       // Response to request
    Notification,   // One-way message
    Signal,         // System signals
    Data,          // Data transfer
    Control,       // System control
    Error,         // Error messages
}
```

#### Message Priorities

```rust
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,  // System messages
}
```

#### Message Flags

```rust
pub struct MessageFlags {
    pub ack_required: bool,
    pub urgent: bool,
    pub reliable: bool,
    pub secure: bool,
    pub broadcast: bool,
    pub compressed: bool,
}
```

#### Message Payloads

```rust
pub enum MessagePayload {
    Empty,
    Data(Vec<u8>),
    Text(Vec<u8>),
    Structured(BTreeMap<Vec<u8>, Vec<u8>>),
    FileDescriptor(u32),
    MemoryRegion { start_addr: u64, size: usize, permissions: u32 },
    Signal { signal: u32, data: u64 },
    Error { code: u32, message: Vec<u8> },
}
```

**Payload Types:**
- **Empty**: No payload
- **Data**: Raw binary data
- **Text**: UTF-8 text messages
- **Structured**: Key-value pairs
- **FileDescriptor**: File descriptor passing
- **MemoryRegion**: Shared memory descriptors
- **Signal**: System signals with data
- **Error**: Error codes with messages

### 2. Queue Management (queues.rs)

#### Message Queue

```rust
pub struct MessageQueue {
    messages: VecDeque<Message>,
    max_size: usize,
    total_enqueued: u64,
    total_dequeued: u64,
    max_size_reached: u64,
}
```

**Features:**
- **Priority ordering** - Higher priority messages are delivered first
- **Capacity limits** - Prevents unbounded memory usage
- **Statistics tracking** - Monitors queue usage and performance
- **Overflow protection** - Rejects messages when full
- **Efficient operations** - O(n) insertion for priority, O(1) dequeue

#### Channel Implementation

```rust
pub struct Channel {
    pub id: ChannelId,
    pub sender: ProcessId,
    pub receiver: ProcessId,
    pub queue: MessageQueue,
    pub created_at: u64,
    pub state: ChannelState,
    pub permissions: ChannelPermissions,
}
```

**Channel States:**
```rust
pub enum ChannelState {
    Active,    // Normal operation
    Suspended, // Temporarily disabled
    Closed,    // Permanently closed
}
```

**Channel Features:**
- **Unidirectional communication** - Clear sender/receiver roles
- **State management** - Active, suspended, or closed states
- **Permission control** - Fine-grained access control
- **Message validation** - Ensures sender authorization
- **Size limits** - Prevents oversized messages
- **Lifecycle management** - Proper creation and cleanup

#### Process Inbox

```rust
pub struct Inbox {
    pub owner: ProcessId,
    pub channels: Vec<ChannelId>,
    pub queue: MessageQueue,
    pub stats: InboxStats,
}
```

**Inbox Features:**
- **Multi-channel aggregation** - Receives from multiple channels
- **Process-specific** - Each process has its own inbox
- **Unified interface** - Single point for message retrieval
- **Statistics tracking** - Monitors inbox activity
- **Automatic delivery** - Messages delivered when sent through channels

#### Channel Manager

```rust
pub struct ChannelManager {
    channels: BTreeMap<ChannelId, Channel>,
    inboxes: BTreeMap<ProcessId, Inbox>,
    next_channel_id: u64,
    stats: ChannelManagerStats,
}
```

**Manager Features:**
- **Global coordination** - Manages all channels and inboxes
- **Unique ID generation** - Ensures unique channel identifiers
- **Resource tracking** - Monitors system-wide IPC usage
- **Cleanup operations** - Removes expired messages
- **Statistics collection** - System-wide IPC metrics

### 3. System Call Interface (sys.rs)

#### IPC System Calls

```rust
pub enum IpcSyscall {
    CreateChannel = 100,
    DestroyChannel = 101,
    SendMessage = 102,
    ReceiveMessage = 103,
    ReceiveFromInbox = 104,
    HasMessages = 105,
    MessageCount = 106,
    ListChannels = 107,
    GetChannelInfo = 108,
    SetChannelPermissions = 109,
    CleanupExpired = 110,
    GetIpcStats = 111,
}
```

#### System Call Handler

```rust
pub struct IpcSyscallHandler {
    calls_handled: u64,
    calls_failed: u64,
}
```

**Handler Features:**
- **Authorization checking** - Validates caller permissions
- **Parameter validation** - Ensures valid arguments
- **Error handling** - Proper error propagation
- **Statistics tracking** - Monitors syscall usage
- **Logging integration** - Detailed operation logging

## API Reference

### Core Functions

#### Channel Management

```rust
/// Create a new communication channel
pub fn create_channel(
    sender: ProcessId,
    receiver: ProcessId,
    queue_size: usize,
) -> IpcResult<ChannelId>

/// Destroy a communication channel
pub fn destroy_channel(channel_id: ChannelId) -> IpcResult<()>

/// Send a message through a channel
pub fn send_message(channel_id: ChannelId, message: Message) -> IpcResult<()>

/// Receive a message from a channel
pub fn receive_message(channel_id: ChannelId) -> IpcResult<Message>

/// Receive a message from process inbox
pub fn receive_from_inbox(process_id: ProcessId) -> IpcResult<Message>
```

#### Information Queries

```rust
/// Check if a process has pending messages
pub fn has_messages(process_id: ProcessId) -> bool

/// Get number of pending messages for a process
pub fn message_count(process_id: ProcessId) -> usize

/// List all channels for a process
pub fn list_channels(process_id: ProcessId) -> Vec<ChannelId>
```

#### Maintenance Operations

```rust
/// Cleanup expired messages
pub fn cleanup_expired() -> usize

/// Get channel manager statistics
pub fn get_manager_stats() -> ChannelManagerStats

/// Get IPC system statistics
pub fn get_ipc_stats() -> IpcStats
```

### Convenience Functions

```rust
/// Send a text message to a process
pub fn send_text_message(
    sender: ProcessId,
    receiver: ProcessId,
    text: &str,
) -> IpcResult<()>

/// Send a signal to a process
pub fn send_signal(
    sender: ProcessId,
    receiver: ProcessId,
    signal: u32,
    data: u64,
) -> IpcResult<()>

/// Send an error message to a process
pub fn send_error_message(
    sender: ProcessId,
    receiver: ProcessId,
    error_code: u32,
    error_text: &str,
) -> IpcResult<()>

/// Request-response pattern helper
pub fn send_request_and_wait_response(
    sender: ProcessId,
    receiver: ProcessId,
    request_payload: MessagePayload,
    timeout_ms: u64,
) -> IpcResult<Message>
```

### Message Creation Helpers

```rust
/// Create a simple text message
pub fn create_text_message(
    sender: ProcessId,
    receiver: ProcessId,
    text: &str,
) -> Message

/// Create a data message
pub fn create_data_message(
    sender: ProcessId,
    receiver: ProcessId,
    data: &[u8],
) -> Message

/// Create a signal message
pub fn create_signal_message(
    sender: ProcessId,
    receiver: ProcessId,
    signal: u32,
    data: u64,
) -> Message

/// Create an error message
pub fn create_error_message(
    sender: ProcessId,
    receiver: ProcessId,
    error_code: u32,
    error_text: &str,
) -> Message
```

## Security Model

### Process Isolation

- **Sender Validation**: Messages can only be sent by authorized processes
- **Receiver Protection**: Processes can only receive messages intended for them
- **Channel Access Control**: Only channel participants can use the channel
- **Inbox Privacy**: Processes can only access their own inbox

### Permission System

```rust
pub struct ChannelPermissions {
    pub can_send: bool,
    pub can_receive: bool,
    pub can_close: bool,
    pub can_modify: bool,
}
```

### Authorization Checks

- **Create Channel**: Only sender can create channel
- **Send Message**: Only authorized senders can send
- **Receive Message**: Only intended receivers can receive
- **Inbox Access**: Only process owner can access inbox
- **Channel Management**: Only participants can manage channel

## Error Handling

### Error Types

```rust
pub enum IpcError {
    ChannelNotFound,
    ChannelFull,
    ChannelEmpty,
    InvalidProcessId,
    PermissionDenied,
    InvalidMessage,
    MessageTooLarge,
    ChannelExists,
    OutOfMemory,
    Timeout,
}
```

### Error Handling Strategy

1. **Validation at Entry**: All parameters validated before processing
2. **Permission Checks**: Authorization verified for all operations
3. **Resource Limits**: Capacity and size limits enforced
4. **Graceful Degradation**: System continues operating on errors
5. **Detailed Logging**: All errors logged with context

## Performance Characteristics

### Time Complexity

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| create_channel | O(log n) | BTreeMap insertion |
| destroy_channel | O(log n) | BTreeMap removal |
| send_message | O(k) | k = queue size for priority insertion |
| receive_message | O(1) | VecDeque pop_front |
| has_messages | O(log n) | BTreeMap lookup |
| message_count | O(log n) | BTreeMap lookup |

### Space Complexity

- **Message Header**: ~120 bytes
- **Message Payload**: Variable (user data + small overhead)
- **Channel**: ~200 bytes + queue overhead
- **Inbox**: ~150 bytes + queue overhead
- **Manager Overhead**: ~80 bytes + BTreeMap nodes

### Memory Management

- **Bounded Queues**: Prevents memory exhaustion
- **Message Size Limits**: Controls maximum message size
- **Channel Limits**: Maximum number of channels
- **Automatic Cleanup**: Expired message removal
- **Statistics Tracking**: Memory usage monitoring

## Configuration

### Constants

```rust
pub const MAX_MESSAGE_SIZE: usize = 4096;  // 4KB max message size
pub const MAX_CHANNELS: usize = 1024;      // Maximum number of channels
pub const DEFAULT_QUEUE_SIZE: usize = 32;  // Default queue size
```

### Limits

- **Message Size**: 4KB maximum to prevent memory exhaustion
- **Channel Count**: 1024 maximum to limit resource usage
- **Queue Size**: Configurable per channel, default 32 messages
- **Inbox Size**: 4x default queue size for aggregation

## Usage Examples

### Basic Messaging

```rust
// Create channel
let channel_id = create_channel(sender_pid, receiver_pid, 10)?;

// Send message
let message = Message::new_notification(
    sender_pid,
    receiver_pid,
    MessagePayload::from_text("Hello, World!"),
);
send_message(channel_id, message)?;

// Receive message
let received = receive_message(channel_id)?;
println!("Received: {}", received.as_text().unwrap());

// Cleanup
destroy_channel(channel_id)?;
```

### Request-Response Pattern

```rust
// Send request
let request = Message::new_request(
    client_pid,
    server_pid,
    MessagePayload::from_text("GET /status"),
);
let request_id = request.header.id;
send_message(request_channel, request)?;

// Wait for response
let response = receive_message(response_channel)?;
if response.is_response_to(request_id) {
    println!("Response: {}", response.as_text().unwrap());
}
```

### Using Inbox

```rust
// Check for messages
if has_messages(my_pid) {
    let count = message_count(my_pid);
    println!("Have {} pending messages", count);
    
    // Process all messages
    for _ in 0..count {
        let message = receive_from_inbox(my_pid)?;
        handle_message(message);
    }
}
```

### System Calls

```rust
// Using system call interface
let args = IpcSyscallArgs::CreateChannel {
    sender: my_pid,
    receiver: target_pid,
    queue_size: 20,
};

match handle_ipc_syscall(100, args, my_pid)? {
    IpcSyscallResult::ChannelId(id) => {
        println!("Created channel: {}", id);
    }
    _ => panic!("Unexpected result"),
}
```

## Testing

### Test Coverage

The IPC module includes comprehensive unit tests covering:

1. **Message Creation and Validation**
   - Message header integrity
   - Payload type handling
   - Checksum verification
   - Timeout handling

2. **Queue Operations**
   - Priority ordering
   - Capacity limits
   - Overflow handling
   - Statistics tracking

3. **Channel Management**
   - Channel lifecycle
   - Permission enforcement
   - State transitions
   - Error conditions

4. **Inbox Functionality**
   - Multi-channel aggregation
   - Message delivery
   - Process isolation
   - Statistics collection

5. **System Call Interface**
   - Authorization checking
   - Parameter validation
   - Error propagation
   - Result handling

6. **Performance Characteristics**
   - Large message handling
   - Queue capacity limits
   - Memory usage patterns
   - Cleanup operations

### Test File Structure

```
kernel/tests/ipc_tests.rs
├─ test_message_creation()
├─ test_message_types()
├─ test_message_payloads()
├─ test_message_queue()
├─ test_channel_functionality()
├─ test_inbox_functionality()
├─ test_ipc_syscalls()
├─ test_convenience_functions()
├─ test_error_conditions()
├─ test_message_priorities()
├─ test_large_messages()
└─ test_channel_capacity()
```

## Integration

### Kernel Integration

The IPC module integrates with the kernel through:

1. **Boot Sequence**: `init_ipc()` called during kernel initialization
2. **System Calls**: IPC syscalls integrated with kernel syscall dispatcher  
3. **Process Management**: Process IDs from scheduler integration
4. **Memory Management**: Uses kernel memory allocators
5. **Logging**: Integrated with kernel logging system

### Module Dependencies

```rust
// External dependencies
extern crate alloc;

// Internal dependencies
use crate::{kprintln, klog};  // Logging
use crate::sched::ProcessId;  // Process management (future)
use crate::mm::{...};         // Memory management (future)
```

## Future Enhancements

### Planned Features

1. **Asynchronous Operations**
   - Non-blocking send/receive
   - Event-driven message handling
   - Async/await support

2. **Message Serialization**
   - Structured data serialization
   - Cross-architecture compatibility
   - Compression support

3. **Advanced Security**
   - Message encryption
   - Digital signatures
   - Access control lists

4. **Network IPC**
   - Remote process communication
   - Network message routing
   - Distributed system support

5. **Performance Optimizations**
   - Zero-copy message passing
   - Shared memory optimization
   - Lock-free data structures

6. **Debugging Tools**
   - Message tracing
   - Performance profiling
   - Visual debugging interface

This comprehensive IPC implementation provides a solid foundation for inter-process communication in Polymera OS, with a focus on security, performance, and ease of use. The modular design allows for future enhancements while maintaining backward compatibility.

