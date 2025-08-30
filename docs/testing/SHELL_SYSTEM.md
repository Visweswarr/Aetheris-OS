# Shell System Documentation

## Overview

The Polymera OS Shell System provides a minimal line-oriented command-line interface accessible via serial for debugging, monitoring, and controlling the kernel. The shell runs as a dedicated task with normal priority and provides a comprehensive set of built-in commands for system administration and debugging.

## Architecture

### Core Components

1. **Shell Module** (`kernel/src/shell/mod.rs`)
   - Command registration and execution
   - Line parsing and argument handling
   - Built-in command implementations

2. **Serial Interface** (`kernel/src/shell/serial.rs`)
   - Serial input/output handling
   - Line buffering and history
   - Special key handling (Ctrl+C, Ctrl+L, etc.)

3. **Shell Task**
   - Runs as a dedicated kernel task
   - Processes serial input continuously
   - Manages command execution lifecycle

### Design Principles

- **Minimal and Robust**: Simple command structure with comprehensive error handling
- **No Kernel Panics**: All errors are caught and reported gracefully
- **Serial-First**: Designed for serial console access
- **Task-Based**: Runs as a separate kernel task for isolation

## Commands

### Core Commands

#### `help`
Shows available commands and their descriptions.

**Usage**: `help`
**Arguments**: None
**Output**: List of all available commands with descriptions

**Example**:
```
polymera> help
Available commands:
==================

  help         - Show available commands
  stats        - Show system statistics
  audit        - Show audit log tail [count]
  sched        - Show scheduler information
  dump         - Dump minidump to serial
  fault        - Toggle fault injection [on|off|status]
  clear        - Clear screen
  echo         - Echo arguments
```

#### `stats`
Displays comprehensive system statistics.

**Usage**: `stats`
**Arguments**: None
**Output**: System uptime, ticks, context switches, IPC metrics, memory usage, and task counts

**Example**:
```
polymera> stats
System Statistics:
==================

Uptime: 1250 ms
Ticks: 12500
Context Switches: 45
Messages Sent: 128
Messages Received: 128
Page Faults: 0
Active Tasks: 3
Blocked Tasks: 1
```

#### `audit [count]`
Shows the last N audit log entries.

**Usage**: `audit [count]`
**Arguments**: 
- `count` (optional): Number of entries to show (1-100, default: 10)
**Output**: Recent audit log entries with timestamps and details

**Example**:
```
polymera> audit 5
Audit Log (last 5 entries):
================================

[1] System initialized - Task 1 started
[2] Task 2 created - Priority: Normal
[3] IPC message sent - From: Task1, To: Task2
[4] Memory allocation - Size: 1024 bytes
[5] Context switch - From: Task1, To: Task2
```

#### `sched`
Displays detailed scheduler information.

**Usage**: `sched`
**Arguments**: None
**Output**: Current task details, scheduler state, and task queue information

**Example**:
```
polymera> sched
Scheduler Information:
=====================

Current Task: Task 1 (ID: 1)
Priority: Normal
State: Running
CPU Time: 1250 ms
Context Switches: 45

Task Queue:
  Ready: 2 tasks
  Blocked: 1 task
  Sleeping: 0 tasks
```

#### `dump`
Shows minidump system information and status.

**Usage**: `dump`
**Arguments**: None
**Output**: Minidump buffer status, last crash information, and build hash

**Example**:
```
polymera> dump
Minidump Information:
====================

Minidump Status: Available
Buffer Size: 64 KB
Last Crash: None (system running normally)
Build Hash: a1b2c3d4e5f6...

To generate a test minidump, trigger a kernel panic.
```

#### `fault [action]`
Controls fault injection system.

**Usage**: `fault [action]`
**Arguments**:
- `on`: Enable fault injection
- `off`: Disable fault injection  
- `status`: Show current status (default)
**Output**: Fault injection system status and control feedback

**Example**:
```
polymera> fault status
Fault Injection Status:
======================

Inbox Overflow: Disabled
Allocation Failure: Disabled
Timer Jitter: Disabled
Overall Status: Disabled

polymera> fault on
Enabling fault injection...
Fault injection enabled
```

#### `clear`
Clears the terminal screen.

**Usage**: `clear`
**Arguments**: None
**Output**: ANSI clear screen sequence

**Example**:
```
polymera> clear
[Screen clears and cursor moves to top-left]
polymera>
```

#### `echo [text...]`
Echoes arguments to the terminal.

**Usage**: `echo [text...]`
**Arguments**: Text to echo (optional)
**Output**: The provided text or empty string

**Example**:
```
polymera> echo hello world
hello world
polymera> echo
[No output]
```

## Serial Interface Features

### Input Handling

- **Line Buffering**: Up to 256 characters per line
- **Command History**: Last 50 commands with up/down arrow navigation
- **Special Keys**: Ctrl+C (cancel), Ctrl+L (clear), Ctrl+U (clear before cursor), Ctrl+K (clear after cursor), Ctrl+W (clear word)
- **Tab Completion**: Expands to 4 spaces (placeholder for future command completion)

### Output Features

- **ANSI Support**: Basic ANSI escape sequences for screen control
- **Error Handling**: Graceful error reporting without system crashes
- **Logging Integration**: Commands can use kernel logging system

## Error Handling

### Command Errors

- **Unknown Commands**: Clear error message with suggestion to use `help`
- **Invalid Arguments**: Specific error messages for argument validation
- **Execution Failures**: Graceful fallback with error reporting

### System Errors

- **Serial Errors**: Non-fatal errors logged to kernel log
- **Memory Errors**: Graceful handling of allocation failures
- **Task Errors**: Shell task continues running even if individual commands fail

### Error Recovery

- **Input Validation**: All input is validated before processing
- **Graceful Degradation**: System continues operating even with shell errors
- **User Feedback**: Clear error messages help users understand and fix issues

## Integration

### Kernel Integration

- **Task Scheduler**: Shell runs as a normal priority task
- **Memory Management**: Uses kernel memory allocation for buffers and history
- **Logging System**: Integrates with kernel logging for error reporting
- **System Calls**: Commands can invoke kernel functionality via syscalls

### Phase 1.5 Integration

- **Crash Dump System**: `dump` command shows minidump status
- **Fault Injection**: `fault` command controls fault injection system
- **Scheduler Analysis**: `sched` command shows scheduler state
- **Audit System**: `audit` command displays security audit logs

## Testing

### Test Coverage

The shell system includes comprehensive tests covering:

- **Command Functionality**: All built-in commands tested individually
- **Error Handling**: Invalid input, malformed commands, edge cases
- **Integration**: Command sequences, rapid execution, robustness
- **Serial Interface**: Input buffering, history management, special keys

### Test Commands

```bash
# Run all shell tests
cargo test shell

# Run specific test categories
cargo test test_shell_creation
cargo test test_command_line_processing
cargo test test_error_handling
```

### Test Scenarios

1. **Basic Functionality**: Command registration, execution, output
2. **Input Validation**: Empty lines, whitespace, special characters
3. **Error Conditions**: Unknown commands, invalid arguments, system failures
4. **Stress Testing**: Rapid command execution, long inputs, memory pressure
5. **Integration Testing**: Command sequences, system state changes

## Usage Examples

### Basic System Monitoring

```bash
polymera> stats
polymera> sched
polymera> audit 20
```

### Fault Injection Testing

```bash
polymera> fault status
polymera> fault on
polymera> stats
polymera> fault off
```

### System Debugging

```bash
polymera> clear
polymera> help
polymera> echo "Testing shell functionality"
polymera> dump
```

### Command History

```bash
polymera> stats
polymera> sched
polymera> audit 5
# Use up arrow to recall previous commands
# Use Ctrl+R to search history (future feature)
```

## Future Enhancements

### Planned Features

1. **Command Completion**: Tab completion for command names and arguments
2. **Scripting Support**: Basic script execution and flow control
3. **Configuration**: Persistent shell configuration and preferences
4. **Plugin System**: Extensible command system for kernel modules
5. **Remote Access**: Network-based shell access (future networking support)

### Potential Commands

- **`ps`**: Process/task listing
- **`top`**: Real-time system monitoring
- **`kill`**: Task termination
- **`mount`**: Filesystem management
- **`netstat`**: Network statistics (when networking is implemented)

## Security Considerations

### Access Control

- **Serial Only**: Shell is only accessible via serial interface
- **No Network**: No remote access capabilities (by design)
- **Kernel Context**: Shell runs in kernel space with full privileges

### Input Validation

- **Sanitization**: All input is validated and sanitized
- **Length Limits**: Strict limits on input length and command complexity
- **Character Filtering**: Control characters and special sequences are handled safely

### Error Reporting

- **No Information Leakage**: Error messages don't reveal sensitive system details
- **Graceful Degradation**: System continues operating even with shell failures
- **Audit Logging**: All shell activity is logged for security monitoring

## Performance Characteristics

### Resource Usage

- **Memory**: ~4KB stack + dynamic allocation for buffers and history
- **CPU**: Minimal overhead, mostly idle waiting for input
- **Serial**: Efficient buffering with minimal interrupt overhead

### Scalability

- **Single Instance**: One shell instance per system
- **Command Complexity**: O(1) command lookup, O(n) argument processing
- **History Management**: Fixed-size circular buffer for command history

### Optimization

- **Lazy Initialization**: Commands and features initialized on first use
- **Efficient Parsing**: Simple tokenization with minimal memory allocation
- **Buffer Reuse**: Input/output buffers are reused to minimize allocations

## Troubleshooting

### Common Issues

1. **Shell Not Responding**
   - Check serial connection and baud rate
   - Verify shell task is running (`sched` command)
   - Check kernel logs for error messages

2. **Commands Not Working**
   - Verify command syntax with `help`
   - Check for error messages in command output
   - Ensure required kernel subsystems are initialized

3. **Input Not Recognized**
   - Check line length (max 256 characters)
   - Verify special characters are handled correctly
   - Test with simple commands like `echo` or `help`

### Debug Commands

```bash
# Check system status
polymera> stats

# Verify scheduler operation
polymera> sched

# Check for errors
polymera> audit 50

# Test basic functionality
polymera> echo "test"
polymera> help
```

## Conclusion

The Polymera OS Shell System provides a robust, feature-rich command-line interface for kernel administration and debugging. With comprehensive error handling, extensive testing, and seamless kernel integration, it serves as a reliable tool for system administrators and developers working with the kernel.

The shell's design emphasizes simplicity, reliability, and extensibility, making it an essential component of the Phase 1.5 kernel stabilization effort.

