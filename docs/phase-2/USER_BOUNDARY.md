# User Task Boundary Crossing System

## Overview

The User Task Boundary Crossing System is a foundational component of Polymera OS that enables secure execution of user tasks with controlled access to kernel services. This system establishes the boundary between user space and kernel space, providing capability-based security, memory isolation, and efficient syscall routing.

## Architecture

### Core Components

1. **User Task Image Header** (`kernel/src/exec/header.rs`)
   - Defines the structure and format of user task images
   - Supports multiple formats: Polymera OS native, WASI, and ELF
   - Includes integrity validation and capability requirements

2. **User Task Loader** (`kernel/src/exec/loader.rs`)
   - Validates image headers and required capabilities
   - Creates user task contexts with memory mapping
   - Sets up syscall gates for kernel access

3. **SYS_EXEC Syscall** (`kernel/src/syscall/handlers.rs`)
   - Integrates with the loader system
   - Provides the user interface for task creation
   - Handles errors and audit logging

### System Flow

```
User Space                    Kernel Space
    |                            |
    | SYS_EXEC(image, caps)     |
    |--------------------------->|
    |                            | Header Validation
    |                            | Capability Verification
    |                            | Memory Allocation
    |                            | Syscall Gate Setup
    |                            |
    | PID (success) or EINVAL   |
    |<---------------------------|
    |                            |
    | Syscall via Gate          |
    |--------------------------->|
    |                            | Syscall Handler
    |                            | Capability Check
    |                            | Service Execution
    |                            |
    | Result                    |
    |<---------------------------|
```

## User Task Image Header

### Structure

The user task image header provides a minimal, extensible format for defining user tasks:

```rust
pub struct UserTaskHeader {
    pub magic: [u8; 8],           // Format identification
    pub version: u32,             // Header version
    pub format: ImageFormat,      // Image format type
    pub entry_point: u64,         // Entry point address
    pub required_caps: Vec<CapabilityRequirement>, // Required capabilities
    pub stack_size: usize,        // Stack size in bytes
    pub data_regions: Vec<DataRegion>, // Data regions with integrity
    pub image_hash: [u8; 32],     // Image integrity hash
    pub header_checksum: u32,     // Header checksum
    pub reserved: [u8; 64],       // Reserved for future use
}
```

### Supported Formats

1. **Polymera OS Native** (`POLYMERA`)
   - Magic: `[0x50, 0x4F, 0x4C, 0x59, 0x4D, 0x45, 0x52, 0x41]`
   - Optimized for Polymera OS features
   - Full capability and data region support

2. **WASI Compatible** (`WASI`)
   - Magic: `[0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]`
   - WebAssembly System Interface compatibility
   - Limited capability support (basic syscalls)

3. **ELF Compatible** (`ELF`)
   - Magic: `[0x7F, 0x45, 0x4C, 0x46, ...]`
   - Executable and Linkable Format compatibility
   - Standard ELF features with Polymera extensions

### Capability Requirements

Each user task specifies required capabilities for execution:

```rust
pub struct CapabilityRequirement {
    pub cap_type: String,         // Capability type (e.g., "SYS_EXIT")
    pub permission_level: u32,    // Required permission level
    pub resource_id: u64,         // Resource identifier
    pub cap_token: Option<CapTokenV2>, // Optional capability token
}
```

### Data Regions

Data regions define memory areas with specific protection and integrity:

```rust
pub struct DataRegion {
    pub name: String,             // Region name
    pub start_address: u64,       // Start address
    pub size: usize,              // Size in bytes
    pub integrity_hash: [u8; 32], // SHA-256 integrity hash
    pub protection: MemoryProtection, // Memory protection flags
}
```

## User Task Loader

### Loading Process

The loader performs several validation and setup steps:

1. **Header Parsing**
   - Extracts header information from binary data
   - Validates magic numbers and format compatibility
   - Checks header version and structure integrity

2. **Header Validation**
   - Verifies entry point validity
   - Checks stack size limits
   - Validates header checksum
   - Ensures capability and data region limits

3. **Capability Verification**
   - Checks required capabilities against available ones
   - Validates permission levels
   - Authenticates capability tokens

4. **Memory Allocation**
   - Allocates user memory space
   - Maps data regions with appropriate protection
   - Sets up user stack with protection

5. **Syscall Gate Setup**
   - Installs syscall instruction at gate address
   - Configures kernel routing for syscalls
   - Sets appropriate memory permissions

### User Task Context

Each loaded user task has a context that tracks its state:

```rust
pub struct UserTaskContext {
    pub pid: u64,                 // Process ID
    pub header: UserTaskHeader,    // Task header
    pub memory_regions: Vec<UserMemoryRegion>, // Memory mapping
    pub stack_info: UserStackInfo, // Stack information
    pub capability_tokens: Vec<CapTokenV2>, // Available capabilities
    pub syscall_gate: u64,        // Syscall gate address
    pub entry_point: u64,         // Entry point address
    pub status: UserTaskStatus,    // Task status
    pub created_at: u64,          // Creation timestamp
}
```

## SYS_EXEC Syscall

### Interface

```rust
// SYS_EXEC = 11
fn handle_exec(a0: u64, a1: u64, a2: u64, a3: u64) -> u64
```

**Parameters:**
- `a0` - Image data pointer
- `a1` - Image data length
- `a2` - Capabilities pointer
- `a3` - Capabilities count

**Returns:**
- Process ID on success
- Error code on failure

### Error Handling

The syscall returns appropriate error codes:

- `EINVAL` (2) - Invalid argument (corrupted header, missing capabilities)
- `ENOMEM` (12) - Out of memory
- `EPERM` (1) - Permission denied

### Audit Logging

All execution verification failures are logged:

```rust
// Audit entry for execution verification failure
let audit_entry = AuditEntry::exec_verify_fail(0, reason.to_string());
```

## Memory Management

### User Memory Layout

```
User Memory Space (16MB - 256MB)
├── Task 1000: 0x1000000 - 0x1010000 (64KB)
│   ├── Stack: 0x1000000 - 0x1004000
│   ├── Data: 0x1004000 - 0x1008000
│   └── Gate: 0x1008000 - 0x1009000
├── Task 1001: 0x1010000 - 0x1020000 (64KB)
│   ├── Stack: 0x1010000 - 0x1014000
│   ├── Data: 0x1014000 - 0x1018000
│   └── Gate: 0x1018000 - 0x1019000
└── ... (additional tasks)
```

### Memory Protection

Each memory region has specific protection flags:

- **Read** (`r`) - Read access allowed
- **Write** (`w`) - Write access allowed
- **Execute** (`x`) - Execute access allowed
- **Shared** (`s`) - Memory shared between tasks

### Stack Protection

User stacks include protection features:

- Red-zone canaries for overflow detection
- Guard pages for boundary protection
- Size limits enforced by hardware
- Automatic cleanup on task termination

## Security Features

### Capability-Based Access Control

- **Required Capabilities**: Tasks must specify required capabilities
- **Permission Levels**: Hierarchical permission system
- **Token Validation**: PQC-signed capability tokens
- **Audit Trail**: Complete logging of access decisions

### Memory Isolation

- **Address Space Separation**: Each task has isolated memory
- **Protection Enforcement**: Hardware-enforced memory protection
- **Integrity Checking**: Data region integrity verification
- **Stack Protection**: Overflow detection and prevention

### Syscall Security

- **Gate Validation**: Syscall gates are task-specific
- **Capability Checking**: Each syscall validates required capabilities
- **Parameter Validation**: Input parameter sanitization
- **Resource Limits**: Enforced limits on resource usage

## Performance Characteristics

### Timing Benchmarks

- **Header Validation**: <15µs total
  - Magic validation: <1µs
  - Version checking: <1µs
  - Entry point validation: <1µs
  - Stack size validation: <1µs
  - Checksum verification: <10µs

- **Capability Verification**: <20µs total
  - Capability lookup: <5µs per capability
  - Permission checking: <1µs per capability
  - Token validation: <10µs per capability

- **Task Creation**: <200µs total
  - Context allocation: <50µs
  - Memory mapping: <100µs
  - Syscall gate setup: <25µs

- **Syscall Overhead**: <10µs per call
  - Gate entry: <1µs
  - Kernel routing: <2µs
  - Handler dispatch: <5µs

### Memory Usage

- **Header Size**: 136 bytes (fixed)
- **Context Overhead**: ~2KB per task
- **Memory Alignment**: 4KB page boundaries
- **Stack Default**: 64KB per task

## Usage Examples

### Creating a Simple User Task

```rust
// Create a minimal user task header
let mut header = UserTaskHeader::new(
    ImageFormat::Polymera,
    0x1000,           // Entry point
    64 * 1024,        // 64KB stack
);

// Add required capabilities
header.add_required_capability("SYS_EXIT".to_string(), 1, 0)?;
header.add_required_capability("SYS_WRITE".to_string(), 1, 1)?;

// Add data region
header.add_data_region(
    "text".to_string(),
    0x1000,           // Start address
    0x1000,           // Size
    [0xAA; 32],       // Integrity hash
    MemoryProtection {
        read: true,
        write: false,
        execute: true,
        shared: false,
    },
)?;

// Update checksum
header.update_checksum();

// Serialize header
let image_data = header.serialize();

// Load user task
let pid = create_user_task(&image_data, &capabilities)?;
println!("User task created with PID: {}", pid);
```

### Loading via SYS_EXEC

```rust
// In user space
let image_ptr = /* pointer to image data */;
let image_len = /* length of image data */;
let caps_ptr = /* pointer to capabilities */;
let caps_count = /* number of capabilities */;

let result = syscall(SYS_EXEC, image_ptr, image_len, caps_ptr, caps_count);
if result >= 1000 {
    println!("Task created with PID: {}", result);
} else {
    println!("Failed to create task: error {}", result);
}
```

## Testing

### Test Script

Run the comprehensive test suite:

```bash
./scripts/test-user-boundary.sh
```

### Unit Tests

```bash
# Test header functionality
cargo test header --package kernel

# Test loader functionality
cargo test loader --package kernel

# Test syscall integration
cargo test --package kernel
```

### Test Scenarios

1. **Valid Image Loading**
   - Correct header with valid capabilities
   - Expected: Task created successfully

2. **Invalid Header Rejection**
   - Corrupted header or invalid entry point
   - Expected: EINVAL with audit logging

3. **Capability Verification Failure**
   - Missing or insufficient capabilities
   - Expected: EINVAL with audit logging

4. **Memory Allocation Failure**
   - Insufficient memory or invalid regions
   - Expected: ENOMEM with cleanup

## Troubleshooting

### Common Issues

1. **Header Parsing Failures**
   - Check magic number compatibility
   - Verify header version
   - Ensure proper data alignment

2. **Capability Verification Errors**
   - Verify required capabilities are available
   - Check permission levels
   - Validate capability tokens

3. **Memory Allocation Problems**
   - Check available memory
   - Verify address space conflicts
   - Ensure proper page alignment

4. **Syscall Gate Issues**
   - Verify gate address setup
   - Check memory permissions
   - Ensure proper instruction encoding

### Debug Information

Enable debug logging:

```rust
// In kernel configuration
features = ["debug"]
```

Monitor audit logs:

```rust
// Check audit entries for failures
let audit_entries = get_audit_entries();
for entry in audit_entries {
    if entry.operation == ops::EXEC_VERIFY_FAIL {
        println!("Execution verification failed: {}", entry.details);
    }
}
```

## Future Enhancements

### Planned Features

1. **Dynamic Loading**
   - Runtime capability acquisition
   - Lazy memory allocation
   - On-demand resource loading

2. **Advanced Security**
   - Seccomp-style syscall filtering
   - Memory encryption
   - Advanced capability delegation

3. **Performance Optimizations**
   - Header caching
   - Capability pre-validation
   - Memory pool management

4. **Format Support**
   - Additional WASM formats
   - Native binary support
   - Script interpreter integration

### Compatibility

- **Backward Compatibility**: Header versioning ensures compatibility
- **Format Extensions**: Reserved fields allow future enhancements
- **Capability Evolution**: Extensible capability system
- **API Stability**: Stable syscall interface

## Conclusion

The User Task Boundary Crossing System provides a secure, efficient foundation for user task execution in Polymera OS. With capability-based security, comprehensive validation, and efficient syscall routing, it enables safe user space execution while maintaining system integrity and performance.

The system is designed to be extensible, allowing future enhancements while maintaining backward compatibility and security guarantees.
