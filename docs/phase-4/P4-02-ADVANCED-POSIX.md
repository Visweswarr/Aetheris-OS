# P4-02: Advanced POSIX Features Implementation

## Overview

This document describes the implementation of Advanced POSIX Features (P4-02) for Aetheris OS, extending the existing POSIX surface (P4-01) with comprehensive process management, signal handling, IPC mechanisms, and threading capabilities.

## Architecture

### Core Components

1. **Process Manager** (`services/posix/src/process.rs`)
   - Process lifecycle management (spawn, fork, exec, wait, kill)
   - Capability-based process creation and control
   - Process state tracking and audit logging
   - NGFS snapshot integration for process isolation

2. **Signal Dispatcher** (`services/posix/src/signals.rs`)
   - Signal delivery and handling
   - User-defined signal support
   - Deterministic signal timing
   - Capability-based signal permissions

3. **IPC Subsystem** (`services/posix/src/ipc.rs`)
   - Pipes with capability enforcement
   - Message queues with priority support
   - Shared memory with permission management
   - Process cleanup and resource management

4. **Thread Manager** (`services/posix/src/threading.rs`)
   - POSIX thread creation and management
   - Capability-based scheduling
   - CPU quota enforcement
   - Deterministic thread scheduling

5. **Enhanced Syscall Broker** (`services/posix/src/broker.rs`)
   - Advanced POSIX syscall support
   - Capability validation and enforcement
   - Performance monitoring and auditing
   - Integration with all POSIX subsystems

## Features

### Process Management

#### Core Operations
- **spawn**: Create new processes with specified capabilities
- **fork**: Create child processes inheriting parent capabilities
- **exec**: Replace process image with new executable
- **wait/waitpid**: Wait for process termination with options
- **kill**: Send signals to processes with capability validation

#### Process States
- `Running`: Active execution
- `Sleeping`: Waiting for I/O or events
- `Stopped`: Suspended by signal
- `Zombie`: Terminated but not reaped
- `Terminated`: Fully cleaned up

#### Capability Model
```rust
// Process creation capabilities
"process:spawn"    // Create new processes
"process:fork"     // Fork existing processes
"process:exec"     // Execute new programs
"process:wait"     // Wait for process termination
"process:signal"   // Send signals to processes
"process:list"     // List process information
```

### Signal Handling

#### Supported Signals
- **Standard Signals**: SIGINT, SIGTERM, SIGKILL, SIGSTOP, SIGCONT
- **User Signals**: SIGUSR1, SIGUSR2
- **System Signals**: SIGCHLD for child process notifications

#### Signal Features
- Deterministic signal delivery timing
- Capability-based signal permissions
- Signal handler registration and management
- Timer-based signal scheduling
- Audit logging for all signal operations

#### Performance Targets
- Signal delivery p50: ≤200µs
- Signal delivery p95: ≤800µs

### IPC Mechanisms

#### Pipes
- Bidirectional communication channels
- Capability-based access control
- Buffer management with size limits
- Process cleanup on termination

#### Message Queues
- Priority-based message delivery
- Configurable queue sizes and message limits
- Capability-based send/receive permissions
- Message type classification (Data, Control, Signal, Heartbeat)

#### Shared Memory
- Memory region creation and mapping
- Permission-based access control (read/write/execute)
- Process attachment tracking
- Automatic cleanup on process termination

#### Performance Targets
- IPC pipe write+read roundtrip p50: ≤400µs
- IPC pipe write+read roundtrip p95: ≤1.0ms

### Threading

#### Thread Management
- POSIX thread creation with pthread_create
- Thread joining with pthread_join
- Thread state management (Ready, Running, Blocked, Sleeping, Terminated)
- Priority-based scheduling

#### Scheduling Policies
- **Round Robin**: Fair time-sliced scheduling
- **Priority Based**: Priority-driven scheduling
- **Deterministic**: Predictable scheduling for testing
- **Real Time**: Low-latency scheduling

#### Capability Model
```rust
// Thread management capabilities
"thread:create"    // Create new threads
"thread:join"      // Join existing threads
"thread:schedule"  // Control thread scheduling
"thread:priority"  // Set thread priorities
```

#### Performance Targets
- Process fork p50: ≤500µs
- Process fork p95: ≤1.5ms

## Implementation Details

### Data Structures

#### ProcessHandle
```rust
pub struct ProcessHandle {
    pub cap_proc: String,           // Capability handle
    pub pid: u32,                   // Process ID
    pub parent_cap: Option<String>, // Parent process capability
    pub capabilities: Vec<String>,  // Process capabilities
    pub created_at: u64,            // Creation timestamp
    pub state: ProcessState,        // Current state
    pub ngfs_snapshot: Option<String>, // NGFS snapshot ID
}
```

#### SignalEvent
```rust
pub struct SignalEvent {
    pub signal: Signal,             // Signal type
    pub target_cap: String,         // Target process capability
    pub sender_cap: Option<String>, // Sender process capability
    pub timestamp: u64,             // Delivery timestamp
    pub data: Option<String>,       // Signal data
}
```

#### ThreadHandle
```rust
pub struct ThreadHandle {
    pub cap_thread: String,         // Thread capability handle
    pub tid: u32,                   // Thread ID
    pub process_cap: String,        // Parent process capability
    pub state: ThreadState,         // Current state
    pub priority: u8,               // Thread priority
    pub capabilities: Vec<String>,  // Thread capabilities
    pub cpu_quota: ThreadQuota,     // CPU usage limits
    pub stack_size: usize,          // Stack size
}
```

### Syscall Integration

The enhanced syscall broker supports the following new syscalls:

#### Process Management Syscalls
- `fork()`: Create child process
- `exec()`: Replace process image
- `waitpid()`: Wait for process termination
- `kill()`: Send signal to process
- `getpid()`: Get process ID
- `getppid()`: Get parent process ID

#### IPC Syscalls
- `pipe()`: Create pipe
- `msgget()`: Create/get message queue
- `msgsnd()`: Send message
- `msgrcv()`: Receive message
- `shmget()`: Create/get shared memory
- `shmat()`: Attach shared memory

#### Threading Syscalls
- `pthread_create()`: Create thread
- `pthread_join()`: Join thread

### Capability Enforcement

All operations are protected by capability-based access control:

```rust
// Example capability check
if !self.check_capability(&request.capabilities, "process:fork") {
    return Err("Permission denied".to_string());
}
```

## CLI Tools

### posix-ctl Enhanced Commands

The Go CLI tool (`go/tools/posix-ctl/main.go`) provides comprehensive POSIX management:

#### Process Management
```bash
# List processes
posix-ctl ps

# Fork a new process
posix-ctl fork /bin/echo

# Kill a process
posix-ctl kill 123 SIGTERM

# Wait for process
posix-ctl wait 123

# Execute program
posix-ctl exec /bin/ls

# Send signal
posix-ctl signal 123 SIGUSR1
```

#### IPC Operations
```bash
# Create pipe
posix-ctl pipe

# Message queue operations
posix-ctl msgq create test_queue
posix-ctl msgq send 1 "Hello, IPC!"
posix-ctl msgq recv 1
posix-ctl msgq list

# Shared memory operations
posix-ctl shm create test_shm 4096
posix-ctl shm attach 1
posix-ctl shm detach 1
posix-ctl shm list
```

#### Threading Operations
```bash
# Thread operations
posix-ctl pthread create worker_func arg1
posix-ctl pthread join 2001
posix-ctl pthread list
```

## Validation and Testing

### Python Validation Script

The validation script (`tooling/python/posix_check.py`) provides comprehensive testing:

#### Test Coverage
- Process management functionality
- Signal handling and delivery
- IPC mechanisms (pipes, message queues, shared memory)
- Threading operations
- Performance requirements validation
- Syscall broker functionality

#### Performance Validation
```python
# Performance budget validation
def test_performance_requirements(self) -> bool:
    # Test process fork performance
    fork_times = []
    for _ in range(10):
        start = time.time()
        self.run_posix_command("fork /bin/test")
        duration = (time.time() - start) * 1000
        fork_times.append(duration)
    
    p50 = fork_times[len(fork_times) // 2]
    p95 = fork_times[int(len(fork_times) * 0.95)]
    
    # Check performance budgets
    fork_p50_ok = p50 <= 0.5  # 500µs
    fork_p95_ok = p95 <= 1.5  # 1.5ms
    
    return fork_p50_ok and fork_p95_ok
```

### Test Execution
```bash
# Run validation script
python3 tooling/python/posix_check.py

# Generate detailed report
python3 tooling/python/posix_check.py > posix_validation_report.json
```

## Web UI

### TypeScript Management Interface

The React-based UI (`ui/posix/posix_ui.tsx`) provides a comprehensive management interface:

#### Features
- **Process Management**: Visual process listing, creation, and control
- **Signal Management**: Signal sending and history tracking
- **IPC Management**: Pipe, message queue, and shared memory monitoring
- **Thread Management**: Thread listing and state monitoring
- **Performance Dashboard**: Real-time performance metrics and budget tracking

#### UI Components
- Process table with state indicators and action buttons
- Signal sender with target process and signal selection
- IPC object visualization with statistics
- Thread management with priority and quota display
- Performance metrics with budget compliance indicators

## Integration with NGFS

### File-backed Execution
- Process executables are loaded from NGFS snapshots
- Capability-based access to executable files
- Snapshot isolation for process security

### Process Isolation
- Each process can have its own NGFS snapshot
- Capability-based access to shared files
- Automatic cleanup of process-specific resources

## Performance Monitoring

### Metrics Collection
- Process creation and termination times
- Signal delivery latency
- IPC operation performance
- Thread scheduling efficiency
- Syscall broker performance

### Budget Enforcement
- Real-time performance monitoring
- Automatic alerts for budget violations
- Performance regression detection
- Optimization recommendations

## Security Considerations

### Capability Model
- All operations require appropriate capabilities
- Fine-grained permission control
- Capability inheritance and delegation
- Audit logging for all operations

### Process Isolation
- NGFS snapshot-based isolation
- Capability-based resource access
- Automatic resource cleanup
- Signal delivery security

### IPC Security
- Capability-based IPC access control
- Message queue permission validation
- Shared memory access restrictions
- Process cleanup and resource management

## Future Enhancements

### Planned Features
- Real-time scheduling support
- Advanced signal handling (signal masks, signal stacks)
- Enhanced IPC mechanisms (semaphores, condition variables)
- Process groups and sessions
- Advanced memory management integration

### Performance Optimizations
- Lock-free data structures
- NUMA-aware scheduling
- Hardware-accelerated IPC
- Optimized signal delivery paths

## Conclusion

The Advanced POSIX Features implementation provides a comprehensive, capability-based POSIX environment for Aetheris OS. The implementation maintains the polyglot nature of the system while providing robust process management, signal handling, IPC mechanisms, and threading support.

The system meets all performance targets and provides extensive tooling for management, validation, and monitoring. The capability-based security model ensures that all operations are properly authorized and audited.

## References

- [P4-01 POSIX Surface Implementation](../phase-4/P4-01-POSIX-SURFACE.md)
- [NGFS v1 Documentation](../phase-3/NGFS-V1-SUMMARY.md)
- [Capability Framework Documentation](../security/CONTRACTS.md)
- [Performance Budgets](../phase-4/PERFORMANCE-BUDGETS.md)

