# Kernel Debugging Runbook

## Overview

This runbook provides comprehensive guidance for debugging common kernel issues in Polymera OS, including boot failures, serial communication problems, fault analysis, and crash investigation. It serves as a quick reference for developers and system administrators.

## Table of Contents

- [Common Issues](#common-issues)
- [Minidump Analysis](#minidump-analysis)
- [Audit Trail Investigation](#audit-trail-investigation)
- [QEMU Debugging Flags](#qemu-debugging-flags)
- [Debugging Workflows](#debugging-workflows)
- [Troubleshooting Checklists](#troubleshooting-checklists)

## Common Issues

### Boot Hang Scenarios

#### 1. **Early Boot Hang (Before Serial Output)**

**Symptoms:**
- No serial output at all
- QEMU appears to start but shows no activity
- System appears frozen

**Debugging Steps:**
```bash
# 1. Check QEMU flags for early debugging
qemu-system-x86_64 \
  -kernel polymera-os-kernel \
  -nographic \
  -d guest_errors \
  -D qemu_debug.log \
  -serial file:serial.log

# 2. Check for hardware initialization issues
# Look for UEFI/BIOS errors in QEMU output
# Verify kernel entry point is reached
```

**Common Causes:**
- Kernel entry point not found
- UEFI/BIOS configuration issues
- Memory mapping problems
- Early panic before serial initialization

**Solutions:**
- Verify kernel binary is valid ELF
- Check memory layout in linker script
- Add early debug prints in boot assembly
- Verify UEFI/BIOS compatibility

#### 2. **Serial Initialization Hang**

**Symptoms:**
- Boot starts but stops at serial initialization
- No command prompt appears
- System appears responsive but no I/O

**Debugging Steps:**
```bash
# 1. Check serial configuration
# Look for serial port initialization messages
# Verify COM1/COM2 configuration

# 2. Test with different serial configurations
qemu-system-x86_64 \
  -kernel polymera-os-kernel \
  -nographic \
  -serial stdio \
  -serial file:serial.log

# 3. Check for serial driver conflicts
# Look for IRQ conflicts or port address issues
```

**Common Causes:**
- Serial port configuration mismatch
- IRQ conflicts
- Driver initialization failure
- Memory allocation failure during serial setup

**Solutions:**
- Verify serial port configuration
- Check for IRQ conflicts
- Add serial driver debug output
- Verify memory allocation during serial init

#### 3. **Task Scheduler Hang**

**Symptoms:**
- Boot completes but no task switching
- System appears frozen after initialization
- No response to input

**Debugging Steps:**
```bash
# 1. Check scheduler initialization
# Look for scheduler start messages
# Verify task creation and switching

# 2. Check for deadlock conditions
# Look for circular dependencies
# Verify interrupt handling

# 3. Check memory allocation
# Verify heap initialization
# Check for memory corruption
```

**Common Causes:**
- Scheduler initialization failure
- Deadlock in task creation
- Memory allocation failure
- Interrupt handler issues

**Solutions:**
- Add scheduler debug output
- Check task creation sequence
- Verify memory allocation
- Test interrupt handling

### Serial Communication Issues

#### 1. **Lost Serial Connection**

**Symptoms:**
- Serial output stops mid-boot
- No response to input
- Connection appears dead

**Debugging Steps:**
```bash
# 1. Check serial buffer status
# Look for buffer overflow messages
# Verify serial interrupt handling

# 2. Test with different serial configurations
qemu-system-x86_64 \
  -kernel polymera-os-kernel \
  -nographic \
  -serial stdio \
  -serial file:serial.log

# 3. Check for serial driver crashes
# Look for panic messages
# Verify driver state
```

**Common Causes:**
- Serial buffer overflow
- Driver crash or panic
- Interrupt handler failure
- Memory corruption in serial buffers

**Solutions:**
- Increase serial buffer size
- Add serial driver error handling
- Verify interrupt handler stability
- Check for memory corruption

#### 2. **Garbled Serial Output**

**Symptoms:**
- Unreadable characters in output
- Corrupted text or binary data
- Inconsistent formatting

**Debugging Steps:**
```bash
# 1. Check baud rate configuration
# Verify QEMU and kernel baud rate match
# Test with different baud rates

# 2. Check for encoding issues
# Verify UTF-8 handling
# Check for binary data corruption

# 3. Check for timing issues
# Verify serial timing
# Check for interrupt timing problems
```

**Common Causes:**
- Baud rate mismatch
- Encoding/decoding errors
- Timing issues
- Hardware emulation problems

**Solutions:**
- Match baud rates between QEMU and kernel
- Add encoding validation
- Fix timing issues
- Use different QEMU serial backend

### Fault Code Analysis

#### 1. **Page Fault (PF) Analysis**

**Fault Code: 0x0000 (Read) / 0x0002 (Write) / 0x0004 (Execute)**

**Debugging Steps:**
```bash
# 1. Check fault address (CR2 register)
# Look for invalid memory access
# Verify memory mapping

# 2. Check fault context
# Look for task/process information
# Verify stack state

# 3. Check memory permissions
# Verify page table entries
# Check for permission violations
```

**Common Causes:**
- Null pointer dereference
- Stack overflow
- Invalid memory access
- Permission violation

**Solutions:**
- Add null pointer checks
- Increase stack size
- Fix memory mapping
- Verify permissions

#### 2. **General Protection Fault (GPF) Analysis**

**Fault Code: 0x000D**

**Debugging Steps:**
```bash
# 1. Check fault context
# Look for invalid segment access
# Verify privilege level

# 2. Check instruction pointer
# Look for invalid instruction
# Verify code segment

# 3. Check stack state
# Verify stack segment
# Check for stack corruption
```

**Common Causes:**
- Invalid segment access
- Privilege level violation
- Invalid instruction
- Stack corruption

**Solutions:**
- Fix segment configuration
- Verify privilege levels
- Check for code corruption
- Fix stack issues

#### 3. **Double Fault Analysis**

**Fault Code: 0x0008**

**Debugging Steps:**
```bash
# 1. Check fault context
# Look for nested fault
# Verify interrupt handling

# 2. Check stack state
# Verify stack integrity
# Check for stack overflow

# 3. Check interrupt handlers
# Verify handler stability
# Check for infinite recursion
```

**Common Causes:**
- Nested page fault
- Stack overflow in fault handler
- Interrupt handler failure
- Memory corruption

**Solutions:**
- Fix primary fault
- Increase stack size
- Stabilize interrupt handlers
- Fix memory corruption

## Minidump Analysis

### Understanding Minidump Structure

The minidump system captures critical system state during crashes:

```rust
#[repr(C, packed)]
pub struct MinidumpHeader {
    pub magic: u32,           // 0xDEADBEEF
    pub version: u32,         // Minidump format version
    pub timestamp: u64,       // Crash timestamp
    pub build_hash: [u8; 32], // Kernel build hash
    pub flags: u32,           // Capture flags
}

#[repr(C, packed)]
pub struct MinidumpData {
    pub cpu_registers: CpuRegisters,
    pub fault_info: FaultInfo,
    pub audit_trail: [AuditEntry; 64],
    pub serial_buffer: [u8; 2048],
    pub kernel_stack: [u8; 4096],
}
```

### Analyzing Minidump Files

#### 1. **Basic Minidump Analysis**

```bash
# Decode minidump file
make decode-minidump FILE=crash.bin

# Or use the tool directly
cargo run --manifest-path tooling/minidump/Cargo.toml -- decode crash.bin
```

#### 2. **Interpreting Minidump Output**

**CPU Registers:**
```
CPU Registers:
  RAX: 0x0000000000000000  (Null pointer)
  RBX: 0x0000000000000000
  RCX: 0x0000000000000000
  RDX: 0x0000000000000000
  RSI: 0x0000000000000000
  RDI: 0x0000000000000000
  RBP: 0x0000000000000000
  RSP: 0x0000000000000000
  RIP: 0x0000000000000000  (Fault address)
  RFLAGS: 0x0000000000000000
  CR2: 0x0000000000000000  (Page fault address)
```

**Fault Information:**
```
Fault Information:
  Fault Type: Page Fault (0x0000)
  Fault Address: 0x0000000000000000
  Error Code: 0x0000000000000000
  Fault Context: User Mode
```

**Audit Trail:**
```
Audit Trail (Last 64 entries):
  [1] Task created: ID=1, Priority=Normal
  [2] Memory allocated: 1024 bytes at 0x1000
  [3] IPC message sent: From=1, To=2
  [4] Page fault: Address=0x0, Type=Read
  [5] Kernel panic: Page fault in user mode
```

#### 3. **Advanced Minidump Analysis**

**Memory Analysis:**
```bash
# Check for memory corruption patterns
grep -A 10 -B 10 "0xDEADBEEF" minidump_analysis.txt

# Look for stack corruption
grep "Stack corruption" minidump_analysis.txt

# Check for heap corruption
grep "Heap corruption" minidump_analysis.txt
```

**Timeline Analysis:**
```bash
# Extract timestamp information
grep "timestamp" minidump_analysis.txt

# Analyze crash sequence
grep "audit" minidump_analysis.txt | tail -20
```

### Common Minidump Patterns

#### 1. **Null Pointer Dereference**
```
Fault Address: 0x0000000000000000
Fault Type: Page Fault (Read)
Context: User Mode
Pattern: RAX/RBX/RCX contains 0x0
```

**Solutions:**
- Add null pointer checks
- Validate pointers before use
- Use safe dereferencing patterns

#### 2. **Stack Overflow**
```
Fault Address: High address (near stack limit)
Fault Type: Page Fault (Write)
Context: Kernel Mode
Pattern: RSP near stack boundary
```

**Solutions:**
- Increase stack size
- Check for infinite recursion
- Add stack overflow detection

#### 3. **Memory Corruption**
```
Fault Address: Valid but corrupted
Fault Type: Page Fault (Read/Write)
Context: Any Mode
Pattern: Unexpected data in memory
```

**Solutions:**
- Check for buffer overflows
- Verify memory allocation
- Add memory corruption detection

## Audit Trail Investigation

### Understanding Audit Entries

Audit entries provide a chronological record of system activity:

```rust
#[repr(C, packed)]
pub struct AuditEntry {
    pub timestamp: u64,      // Entry timestamp
    pub event_type: u32,     // Event type code
    pub task_id: u32,        // Associated task
    pub data: [u8; 32],      // Event-specific data
}
```

### Common Audit Event Types

#### 1. **System Events (0x0000-0x00FF)**
- `0x0001`: System boot
- `0x0002`: System shutdown
- `0x0003`: Kernel panic
- `0x0004`: System recovery

#### 2. **Task Events (0x0100-0x01FF)**
- `0x0101`: Task created
- `0x0102`: Task destroyed
- `0x0103`: Task started
- `0x0104`: Task stopped
- `0x0105`: Task blocked
- `0x0106`: Task unblocked

#### 3. **Memory Events (0x0200-0x02FF)**
- `0x0201`: Memory allocated
- `0x0202`: Memory freed
- `0x0203`: Page fault
- `0x0204`: Memory corruption detected

#### 4. **IPC Events (0x0300-0x03FF)**
- `0x0301`: Message sent
- `0x0302`: Message received
- `0x0303`: Message dropped
- `0x0304`: Queue overflow

#### 5. **Security Events (0x0400-0x04FF)**
- `0x0401`: Capability granted
- `0x0402`: Capability revoked
- `0x0403`: Access denied
- `0x0404`: Security violation

### Analyzing Audit Trails

#### 1. **Basic Audit Analysis**

```bash
# Show recent audit entries
polymera> audit 20

# Show specific event types
polymera> audit 50 | grep "Page fault"
polymera> audit 50 | grep "Memory allocated"
```

#### 2. **Timeline Analysis**

```bash
# Extract timestamp information
grep "timestamp" audit_log.txt

# Analyze event sequence
grep "Task created" audit_log.txt | tail -10
grep "Memory allocated" audit_log.txt | tail -10
```

#### 3. **Pattern Recognition**

**Memory Leak Pattern:**
```
[1] Memory allocated: 1024 bytes
[2] Memory allocated: 2048 bytes
[3] Memory allocated: 4096 bytes
[4] Task destroyed: ID=1
# No corresponding "Memory freed" entries
```

**Task Blocking Pattern:**
```
[1] Task blocked: ID=1, Reason=IPC wait
[2] Task blocked: ID=2, Reason=IPC wait
[3] Task blocked: ID=3, Reason=IPC wait
# All tasks waiting for IPC - potential deadlock
```

**Resource Exhaustion Pattern:**
```
[1] Memory allocated: 1024 bytes
[2] Memory allocated: 1024 bytes
[3] Memory allocation failed: Out of memory
# System running out of memory
```

### Audit Investigation Workflows

#### 1. **Crash Investigation**

```bash
# 1. Get crash timestamp
polymera> dump

# 2. Check audit entries around crash time
polymera> audit 100

# 3. Look for suspicious patterns
# - Memory allocations before crash
# - Task state changes
# - IPC activity
# - Security events

# 4. Correlate with minidump data
# - Check fault address
# - Verify task context
# - Analyze memory state
```

#### 2. **Performance Investigation**

```bash
# 1. Check system statistics
polymera> stats

# 2. Analyze scheduler activity
polymera> sched

# 3. Check audit trail for bottlenecks
polymera> audit 200 | grep "Task blocked"
polymera> audit 200 | grep "IPC"

# 4. Look for resource contention
# - Memory allocation patterns
# - Task blocking reasons
# - IPC queue activity
```

#### 3. **Security Investigation**

```bash
# 1. Check security events
polymera> audit 100 | grep "Security"

# 2. Look for access violations
polymera> audit 100 | grep "Access denied"
polymera> audit 100 | grep "Security violation"

# 3. Check capability changes
polymera> audit 100 | grep "Capability"
polymera> audit 100 | grep "Permission"
```

## QEMU Debugging Flags

### Essential QEMU Flags

#### 1. **Basic Debugging Flags**

```bash
# Enable guest error logging
-d guest_errors

# Enable CPU exception logging
-d cpu_reset

# Enable interrupt logging
-d int

# Enable memory access logging
-d guest_errors,unimp

# Enable all debug output (verbose)
-D qemu_debug.log
```

#### 2. **Serial and I/O Debugging**

```bash
# Multiple serial outputs
-serial stdio -serial file:serial.log

# Serial with debug info
-serial stdio -serial file:serial.log -D qemu_serial.log

# Enable serial debugging
-serial stdio -serial file:serial.log -d guest_errors

# Serial with timing info
-serial stdio -serial file:serial.log -d guest_errors,unimp
```

#### 3. **Memory and CPU Debugging**

```bash
# Enable memory access logging
-d guest_errors,unimp,op_opt

# Enable CPU state logging
-d cpu_reset,int,guest_errors

# Enable page fault logging
-d guest_errors,page

# Enable all CPU debugging
-d cpu_reset,int,guest_errors,unimp,op_opt
```

#### 4. **Advanced Debugging Flags**

```bash
# Enable all debugging (very verbose)
-d guest_errors,cpu_reset,int,unimp,op_opt,page

# Enable specific CPU features
-cpu qemu64,+apic,+x2apic

# Enable memory debugging
-m 512M -d guest_errors,page

# Enable interrupt debugging
-d int,guest_errors
```

### QEMU Debugging Cheat Sheet

#### **Basic Debugging**
```bash
# Minimal debugging
qemu-system-x86_64 -kernel kernel -nographic

# Basic error logging
qemu-system-x86_64 -kernel kernel -nographic -d guest_errors

# With debug log file
qemu-system-x86_64 -kernel kernel -nographic -d guest_errors -D debug.log
```

#### **Serial Debugging**
```bash
# Dual serial output
qemu-system-x86_64 -kernel kernel -nographic \
  -serial stdio -serial file:serial.log

# Serial with timing
qemu-system-x86_64 -kernel kernel -nographic \
  -serial stdio -serial file:serial.log -d guest_errors
```

#### **Memory Debugging**
```bash
# Memory access logging
qemu-system-x86_64 -kernel kernel -nographic \
  -d guest_errors,page -m 512M

# Full memory debugging
qemu-system-x86_64 -kernel kernel -nographic \
  -d guest_errors,page,unimp -m 512M
```

#### **CPU Debugging**
```bash
# CPU state logging
qemu-system-x86_64 -kernel kernel -nographic \
  -d cpu_reset,int,guest_errors

# Full CPU debugging
qemu-system-x86_64 -kernel kernel -nographic \
  -d cpu_reset,int,guest_errors,unimp,op_opt
```

#### **Complete Debugging Setup**
```bash
# Maximum debugging information
qemu-system-x86_64 \
  -kernel polymera-os-kernel \
  -nographic \
  -m 512M \
  -smp 2 \
  -cpu qemu64,+apic \
  -d guest_errors,cpu_reset,int,unimp,op_opt,page \
  -D qemu_debug.log \
  -serial stdio \
  -serial file:serial.log \
  -serial file:error.log
```

### Debug Output Interpretation

#### 1. **Guest Error Messages**
```
guest_errors: vaddr=0x0000000000000000 paddr=0x0000000000000000
guest_errors: vaddr=0x0000000000000000 paddr=0x0000000000000000
guest_errors: vaddr=0x0000000000000000 paddr=0x0000000000000000
```
**Meaning:** Page fault at address 0x0 (null pointer dereference)

#### 2. **CPU Reset Messages**
```
cpu_reset: CPU 0: APIC ID 0x00
cpu_reset: CPU 1: APIC ID 0x01
```
**Meaning:** CPU initialization and APIC configuration

#### 3. **Interrupt Messages**
```
int: type=13, vector=0x0d, error_code=0x0000
int: type=14, vector=0x0e, error_code=0x0000
```
**Meaning:** General protection fault (13) and page fault (14)

#### 4. **Memory Access Messages**
```
page: vaddr=0x0000000000000000 paddr=0x0000000000000000
page: vaddr=0x0000000000000000 paddr=0x0000000000000000
```
**Meaning:** Memory access attempts and page faults

## Debugging Workflows

### 1. **Boot Failure Investigation**

```bash
# Step 1: Enable maximum debugging
qemu-system-x86_64 \
  -kernel polymera-os-kernel \
  -nographic \
  -d guest_errors,cpu_reset,int,unimp,op_opt,page \
  -D qemu_debug.log \
  -serial stdio \
  -serial file:serial.log

# Step 2: Check QEMU debug output
cat qemu_debug.log | grep "guest_errors"
cat qemu_debug.log | grep "cpu_reset"
cat qemu_debug.log | grep "int"

# Step 3: Check serial output
cat serial.log | grep "panic"
cat serial.log | grep "error"
cat serial.log | grep "fault"

# Step 4: Analyze failure point
# Look for last successful operation
# Check for error messages
# Verify system state
```

### 2. **Crash Investigation**

```bash
# Step 1: Reproduce crash with debugging
qemu-system-x86_64 \
  -kernel polymera-os-kernel \
  -nographic \
  -d guest_errors,page \
  -D crash_debug.log \
  -serial stdio \
  -serial file:crash_serial.log

# Step 2: Analyze minidump
make decode-minidump FILE=crash.bin

# Step 3: Check audit trail
# Look for events leading to crash
# Check for resource exhaustion
# Verify task state changes

# Step 4: Correlate information
# Match minidump data with audit trail
# Check QEMU debug output
# Verify crash sequence
```

### 3. **Performance Investigation**

```bash
# Step 1: Enable performance monitoring
qemu-system-x86_64 \
  -kernel polymera-os-kernel \
  -nographic \
  -d guest_errors \
  -serial stdio \
  -serial file:perf.log

# Step 2: Monitor system activity
polymera> stats
polymera> sched
polymera> audit 100

# Step 3: Look for bottlenecks
# Check task blocking patterns
# Monitor memory allocation
# Watch IPC activity

# Step 4: Analyze patterns
# Identify resource contention
# Check for deadlocks
# Verify performance metrics
```

## Troubleshooting Checklists

### Boot Failure Checklist

- [ ] Verify kernel binary is valid ELF
- [ ] Check memory layout in linker script
- [ ] Verify UEFI/BIOS compatibility
- [ ] Check serial port configuration
- [ ] Verify interrupt handling
- [ ] Check memory allocation
- [ ] Verify task scheduler initialization

### Crash Investigation Checklist

- [ ] Collect minidump file
- [ ] Decode minidump data
- [ ] Check audit trail around crash time
- [ ] Analyze fault context and registers
- [ ] Check for memory corruption
- [ ] Verify task state
- [ ] Correlate with QEMU debug output

### Performance Issue Checklist

- [ ] Check system statistics
- [ ] Monitor task scheduler
- [ ] Analyze audit trail patterns
- [ ] Check for resource contention
- [ ] Monitor memory allocation
- [ ] Watch IPC activity
- [ ] Verify performance metrics

### Serial Communication Checklist

- [ ] Verify serial port configuration
- [ ] Check baud rate settings
- [ ] Verify interrupt handling
- [ ] Check for buffer overflow
- [ ] Verify driver stability
- [ ] Check for memory corruption
- [ ] Test with different serial backends

## Conclusion

This runbook provides comprehensive guidance for debugging kernel issues in Polymera OS. By following the workflows and using the appropriate debugging tools, developers can quickly identify and resolve common problems.

Remember to:
- Always enable appropriate debugging flags
- Collect comprehensive diagnostic information
- Correlate data from multiple sources
- Document findings for future reference
- Use systematic approaches to problem-solving

For additional help, consult the main documentation and test suites, or refer to the kernel source code for implementation details.

