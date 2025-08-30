# Polymera OS Demo Tasks - Cooperative Round-Robin Scheduling

## Overview

This module implements demonstration tasks that showcase the cooperative scheduler and round-robin task switching functionality in Polymera OS. The demo tasks are designed to visually demonstrate how tasks yield the CPU to each other using system calls.

## Demo Tasks Implemented

### **T1 - Demo Task 1**
```rust
pub fn demo_task_1(_arg: u64) -> i32 {
    let mut counter = 0u64;
    loop {
        counter += 1;
        kprintln!("[T1] Counter: {}", counter);
        sys_yield_demo();
        // ... exit after 50 iterations
    }
}
```

**Behavior:**
- Prints incrementing counter with `[T1]` prefix
- Yields CPU after each iteration using `sys_yield()`
- Exits after 50 iterations
- Visual separator every 10 iterations

### **T2 - Demo Task 2**
```rust
pub fn demo_task_2(_arg: u64) -> i32 {
    let mut counter = 0u64;
    loop {
        counter += 1;
        kprintln!("[T2] *** Counter: {} ***", counter);
        sys_yield_demo();
        // ... exit after 30 iterations
    }
}
```

**Behavior:**
- Prints incrementing counter with `[T2] ***` prefix for visual distinction
- Yields CPU after each iteration using `sys_yield()`
- Exits after 30 iterations
- Visual milestone every 5 iterations

### **Monitor Task**
```rust
pub fn demo_monitor_task(_arg: u64) -> i32 {
    loop {
        // Print system status every 3 iterations
        let uptime = get_uptime_seconds();
        let stats = get_scheduler_stats();
        kprintln!("[MONITOR] Uptime: {}s, Active tasks: {}", uptime, stats.total_tasks);
        sys_yield_demo();
        // ... exit after 20 checks
    }
}
```

**Behavior:**
- Monitors system status and scheduler statistics
- Yields less frequently to allow other tasks more CPU time
- Provides periodic system health reports

## Enhanced Demo Tasks

### **Fast Counter Task**
- Yields after every iteration (maximally cooperative)
- Short delay between iterations
- Demonstrates high-frequency yielding

### **Slow Counter Task**
- Yields every 2 iterations
- Longer delay between iterations
- Demonstrates variable yielding patterns

### **Yielding Task**
- Always yields to be maximally cooperative
- Variable delay based on iteration count
- Periodically reports scheduler statistics

## Cooperative Scheduling Flow

```
Boot Sequence:
├── Initialize Scheduler
├── Initialize Syscalls  
├── Create Init Process
├── Spawn Demo Tasks (T1, T2, Monitor)
└── Spawn Enhanced Demo Tasks (Fast, Slow, Yielder)

Task Execution Flow:
T1 → sys_yield() → T2 → sys_yield() → Monitor → sys_yield() → Fast → sys_yield() → Slow → sys_yield() → Yielder → sys_yield() → T1 (cycle continues)
```

## Expected Output Pattern

### **Boot Sequence**
```
[PolymeraCore] Starting demo tasks
[DEMO] Spawning demo tasks for cooperative scheduling demonstration
[DEMO] Created demo task T1 with ID 2
[DEMO] Created demo task T2 with ID 3  
[DEMO] Created monitor task with ID 4
[DEMO] Demo task spawning complete - 3 tasks created
[PolymeraCore] Starting enhanced demo
[DEMO] Created fast counter task with ID 5
[DEMO] Created slow counter task with ID 6
[DEMO] Created yielding task with ID 7
[DEMO] Enhanced demo: 3 tasks with different behaviors
```

### **Round-Robin Execution**
```
[T1] Counter: 1
[sys] yield from task 2
[T2] *** Counter: 1 ***
[sys] yield from task 3
[MONITOR] System status check #1
[MONITOR] Uptime: 5s, Active tasks: 7
[sys] yield from task 4
[FAST1] Rapid count: 1
[sys] yield from task 5
[SLOW2] Deliberate count: 1
[sys] yield from task 6
[YIELD3] Cooperative iteration: 1
[sys] yield from task 7
[T1] Counter: 2
[sys] yield from task 2
[T2] *** Counter: 2 ***
...
```

### **Task Completion**
```
[T2] Demo complete, exiting after 30 iterations
[sys] exit 2 from task 3
[T1] Demo complete, exiting after 50 iterations  
[sys] exit 1 from task 2
[MONITOR] Monitoring complete, exiting after 20 checks
[sys] exit 0 from task 4
```

## Key Demonstration Features

### **1. Cooperative Yielding**
- Tasks voluntarily give up CPU using `sys_yield()`
- Demonstrates proper task cooperation
- Shows round-robin scheduling in action

### **2. System Call Integration**
```rust
fn sys_yield_demo() {
    let current_task = crate::sched::get_current_task_id();
    klog!(TRACE, "[DEMO] Task {} yielding CPU", current_task);
    crate::sched::sys::sys_yield();
    klog!(TRACE, "[DEMO] Task {} resumed execution", current_task);
}
```

### **3. Task Lifecycle Management**
```rust
fn sys_exit_demo(code: i32) -> ! {
    let current_task = crate::sched::get_current_task_id();
    kprintln!("[DEMO] Task {} exiting with code {}", current_task, code);
    crate::sched::sys::sys_exit(code);
}
```

### **4. Visual Output Distinction**
- `[T1]` - Simple counter format
- `[T2] ***` - Emphasized format for easy distinction
- `[MONITOR]` - System status updates
- `[FAST#]` - High-frequency task
- `[SLOW#]` - Low-frequency task
- `[YIELD#]` - Cooperative task

## Scheduler Analysis

### **Performance Metrics**
```rust
pub fn analyze_demo_behavior() {
    let tick_stats = crate::sched::tick::get_tick_stats();
    let (total_syscalls, invalid_syscalls) = crate::syscall::get_syscall_stats();
    
    kprintln!("Scheduler performance: {}", tick_stats);
    kprintln!("Syscall activity: {} total, {} invalid", total_syscalls, invalid_syscalls);
}
```

### **Expected Behavior Validation**
- **Round-Robin**: Tasks should alternate in FIFO order
- **Yielding**: Each `sys_yield()` should switch to next ready task
- **Completion**: Tasks should exit cleanly with proper exit codes
- **Resource Cleanup**: Exited tasks should be removed from scheduler

## Integration Points

### **Boot Integration**
```rust
// In boot.rs
pub fn init() {
    // ... HAL and scheduler initialization
    let _demo_tasks = crate::demo::spawn_demo_tasks();
    let _enhanced_demo_tasks = crate::demo::spawn_enhanced_demo();
}
```

### **Scheduler Integration**
```rust
// Direct calls to scheduler functions
crate::sched::sys::sys_yield();     // Cooperative yielding
crate::sched::sys::sys_exit(code);  // Task termination
```

### **Syscall Integration**
```rust
// Uses the syscall interface for task management
[sys] yield from task X
[sys] exit Y from task X
```

## Testing and Validation

### **Test Functions**
```rust
pub fn test_cooperative_scheduling();  // Basic functionality test
pub fn spawn_demo_tasks() -> usize;    // Create standard demo tasks
pub fn spawn_enhanced_demo() -> usize; // Create advanced demo tasks
pub fn analyze_demo_behavior();        // Performance analysis
```

### **Validation Criteria**
1. **Task Creation**: All demo tasks should be created successfully
2. **Round-Robin**: Tasks should execute in alternating pattern
3. **Yielding**: `sys_yield()` calls should trigger context switches
4. **Exit Codes**: Tasks should exit with correct status codes
5. **Resource Management**: No memory leaks or resource exhaustion

## Future Enhancements

### **Phase 2 Improvements**
- **Preemptive Scheduling**: Add timer-based preemption to demo
- **Priority Demonstration**: Tasks with different priority levels
- **IPC Demo**: Inter-task communication using PolyBus
- **Real-time Demo**: Tasks with timing constraints

### **Advanced Demos**
- **Producer/Consumer**: Classic synchronization demo
- **Dining Philosophers**: Deadlock avoidance demonstration
- **Load Testing**: High-frequency task creation/destruction
- **Performance Benchmarks**: Scheduler overhead measurement

## Usage Instructions

### **Running the Demo**
1. Build and boot the kernel
2. Demo tasks start automatically after scheduler initialization
3. Watch serial output for round-robin task switching
4. Observe task alternation pattern in the logs

### **Customizing Demo Behavior**
```rust
// Modify task iteration counts
if counter >= 50 { sys_exit_demo(1); }  // T1 exits after 50
if counter >= 30 { sys_exit_demo(2); }  // T2 exits after 30

// Adjust yielding frequency
sys_yield_demo();              // Yield every iteration
if i % 2 == 0 { sys_yield_demo(); }  // Yield every 2 iterations
```

### **Performance Monitoring**
```rust
// Add to any demo task for monitoring
let stats = crate::sched::get_scheduler_stats();
kprintln!("Current: {} ready, {} running tasks", stats.ready_tasks, stats.running_tasks);
```

This demo implementation provides a comprehensive showcase of cooperative multitasking with clear visual output, proper resource management, and extensive integration with the kernel's scheduling and syscall systems.
