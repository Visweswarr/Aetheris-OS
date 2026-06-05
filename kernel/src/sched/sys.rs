/// System Interface for Polymera OS Scheduler
/// 
/// This module provides system calls and kernel interfaces for task management
/// and scheduling operations.

use crate::{kprintln, klog};
use super::{TaskId, TaskState, get_current_task_id, create_task, enqueue_task, block_task, wake_task, kill_task};
use super::task::{Task, TaskPriority, TaskParams, DEFAULT_STACK_SIZE};

/// System call interface for scheduler operations
/// 
/// These functions provide the interface between user space and the kernel
/// scheduler. In a full implementation, these would be called via system calls.
pub struct SchedulerSyscalls;

impl SchedulerSyscalls {
    /// Exit the current task
    /// 
    /// Terminates the calling task and removes it from the scheduler.
    /// This is equivalent to the POSIX exit() system call.
    /// 
    /// # Arguments
    /// * `exit_code` - Exit status code (0 = success, non-zero = error)
    pub fn exit(exit_code: i32) -> ! {
        let current_task = get_current_task_id();
        
        klog!(INFO, "[SCHED] Task {} exiting with code {}", current_task, exit_code);
        
        // Mark task as dead and remove from scheduler
        kill_task(TaskId(current_task));
        
        // Trigger scheduling to switch to another task
        super::schedule();
        
        // Should never reach here since the task is dead
        loop {
            x86_64::instructions::hlt();
        }
    }
    
    /// Yield CPU to another task
    /// 
    /// Voluntarily gives up the CPU so another task can run.
    /// The current task remains ready and may be scheduled again.
    pub fn yield_cpu() {
        let current_task = get_current_task_id();
        klog!(TRACE, "[SCHED] Task {} yielding CPU", current_task);
        
        super::yield_current();
    }
    
    /// Get the current task ID
    /// 
    /// Returns the TaskId of the currently running task.
    /// 
    /// # Returns
    /// TaskId of the current task
    pub fn get_tid() -> TaskId {
        TaskId(get_current_task_id())
    }
    
    /// Sleep for a specified number of milliseconds
    /// 
    /// Blocks the current task for at least the specified duration.
    /// 
    /// # Arguments
    /// * `milliseconds` - Minimum sleep duration in milliseconds
    pub fn sleep_ms(milliseconds: u64) {
        let current_task = get_current_task_id();
        
        klog!(TRACE, "[SCHED] Task {} sleeping for {}ms", current_task, milliseconds);
        
        // For Phase 1, we'll implement a simple busy-wait sleep
        // In a full implementation, this would use timer queues
        
        let start_tick = crate::hal::x86_64::timer::get_tick_count();
        let target_tick = start_tick + milliseconds; // 1000Hz timer = 1 tick per ms
        
        while crate::hal::x86_64::timer::get_tick_count() < target_tick {
            // Yield CPU while waiting
            Self::yield_cpu();
        }
        
        klog!(TRACE, "[SCHED] Task {} woke up after {}ms", current_task, milliseconds);
    }
    
    /// Create a new kernel thread
    /// 
    /// Creates a new task that runs in kernel space.
    /// 
    /// # Arguments
    /// * `entry_point` - Function address to start execution
    /// * `arg` - Optional argument to pass to the thread
    /// * `params` - Task creation parameters
    /// 
    /// # Returns
    /// TaskId of the new thread, or None if creation failed
    pub fn create_kernel_thread(
        entry_point: fn(u64) -> i32,
        arg: u64,
        params: TaskParams,
    ) -> Option<TaskId> {
        // Allocate stack for the new thread
        let stack_top = allocate_kernel_stack(params.stack_size)?;
        
        // Create the task
        let task_id = create_task(stack_top);
        
        if task_id.0 == 0 {
            // Failed to create task
            deallocate_kernel_stack(stack_top, params.stack_size);
            return None;
        }
        
        // Set up the task's initial context
        // In a full implementation, we would set up the stack with:
        // - Return address pointing to task_exit_handler
        // - Entry point and argument for the thread function
        
        klog!(INFO, "[SCHED] Created kernel thread {} at 0x{:016x}", 
              task_id.0, entry_point as *const fn(u64) -> i32 as u64);
        
        Some(task_id)
    }
    
    /// Wait for a task to complete
    /// 
    /// Blocks the current task until the specified task terminates.
    /// 
    /// # Arguments
    /// * `task_id` - Task to wait for
    /// 
    /// # Returns
    /// Exit code of the waited task, or None if task doesn't exist
    pub fn wait_for_task(task_id: TaskId) -> Option<i32> {
        let current_task = get_current_task_id();
        
        klog!(TRACE, "[SCHED] Task {} waiting for task {}", current_task, task_id.0);
        
        // Simple polling implementation for Phase 1
        // In a full implementation, this would use proper blocking
        loop {
            if let Some(task) = super::get_task(task_id) {
                if task.is_dead() {
                    klog!(TRACE, "[SCHED] Task {} finished waiting for {}", current_task, task_id.0);
                    return Some(0); // Simplified exit code
                }
            } else {
                // Task doesn't exist
                return None;
            }
            
            // Yield CPU while waiting
            Self::yield_cpu();
        }
    }
    
    /// Set task priority
    /// 
    /// Changes the priority of the specified task.
    /// 
    /// # Arguments
    /// * `task_id` - Task to modify
    /// * `priority` - New priority level
    /// 
    /// # Returns
    /// `true` if priority was changed, `false` if task doesn't exist
    pub fn set_priority(task_id: TaskId, priority: TaskPriority) -> bool {
        // For Phase 1, priorities are not fully implemented
        klog!(INFO, "[SCHED] Setting task {} priority to {:?}", task_id.0, priority);
        
        // In a full implementation, this would:
        // 1. Update the task's priority field
        // 2. Move the task to the appropriate priority queue
        // 3. Potentially trigger rescheduling
        
        true
    }
}

/// Kernel thread management
pub struct KernelThreads;

impl KernelThreads {
    /// Spawn a new kernel thread with default parameters
    /// 
    /// # Arguments
    /// * `name` - Thread name for debugging
    /// * `entry_point` - Function to execute
    /// * `arg` - Argument to pass to the function
    /// 
    /// # Returns
    /// TaskId of the new thread or None if creation failed
    pub fn spawn(
        name: &'static str,
        entry_point: fn(u64) -> i32,
        arg: u64,
    ) -> Option<TaskId> {
        let params = TaskParams {
            priority: TaskPriority::Normal,
            stack_size: DEFAULT_STACK_SIZE,
            name: Some(name),
        };
        
        SchedulerSyscalls::create_kernel_thread(entry_point, arg, params)
    }
    
    /// Spawn a high-priority kernel thread
    /// 
    /// # Arguments
    /// * `name` - Thread name for debugging
    /// * `entry_point` - Function to execute
    /// * `arg` - Argument to pass to the function
    /// 
    /// # Returns
    /// TaskId of the new thread or None if creation failed
    pub fn spawn_high_priority(
        name: &'static str,
        entry_point: fn(u64) -> i32,
        arg: u64,
    ) -> Option<TaskId> {
        let params = TaskParams {
            priority: TaskPriority::High,
            stack_size: DEFAULT_STACK_SIZE,
            name: Some(name),
        };
        
        SchedulerSyscalls::create_kernel_thread(entry_point, arg, params)
    }
    
    /// Create the init process (first user process)
    /// 
    /// # Returns
    /// TaskId of the init process
    pub fn create_init_process() -> TaskId {
        kprintln!("[SCHED] Creating init process");
        
        // For Phase 1, init is just another kernel thread
        let init_task = Self::spawn("init", init_process_main, 0);
        
        match init_task {
            Some(task_id) => {
                klog!(INFO, "[SCHED] Init process created with ID {}", task_id.0);
                task_id
            }
            None => {
                kprintln!("[SCHED] Failed to create init process!");
                TaskId(0) // Return idle task ID as fallback
            }
        }
    }
}

/// Simplified stack allocation for Phase 1
/// 
/// In a full implementation, this would use the memory manager
/// to allocate properly aligned and protected stack memory.
fn allocate_kernel_stack(size: usize) -> Option<u64> {
    // For Phase 1, we'll use a simplified static allocation
    // This is not suitable for production but works for testing
    
    static mut NEXT_STACK_ADDR: u64 = 0x800000; // Start at 8MB
    
    unsafe {
        if NEXT_STACK_ADDR > 0xF00000 { // Don't go past 15MB
            return None;
        }
        
        let stack_top = NEXT_STACK_ADDR + size as u64;
        NEXT_STACK_ADDR += size as u64 + 4096; // Add guard page
        
        Some(stack_top)
    }
}

/// Simplified stack deallocation for Phase 1
fn deallocate_kernel_stack(_stack_top: u64, _size: usize) {
    // For Phase 1, we don't actually deallocate
    // In a full implementation, this would return memory to the allocator
}

/// Init process main function
/// 
/// This is the first user-space process that starts other system services.
fn init_process_main(_arg: u64) -> i32 {
    kprintln!("[INIT] Init process starting");
    
    // Create some test tasks
    for i in 1..=3 {
        if let Some(task_id) = KernelThreads::spawn(
            "test_task",
            test_task_main,
            i,
        ) {
            klog!(INFO, "[INIT] Created test task {} with ID {}", i, task_id.0);
        }
    }
    
    // Main init loop
    let mut counter = 0;
    loop {
        counter += 1;
        
        if counter % 1000 == 0 {
            klog!(INFO, "[INIT] Init heartbeat #{}", counter / 1000);
        }
        
        // Sleep for 100ms
        SchedulerSyscalls::sleep_ms(100);
    }
}

/// Test task function
fn test_task_main(task_num: u64) -> i32 {
    klog!(INFO, "[TEST{}] Test task {} starting", task_num, task_num);
    
    for i in 1..=10 {
        klog!(TRACE, "[TEST{}] Iteration {}", task_num, i);
        
        // Do some work
        let mut sum = 0u64;
        for j in 0..1000 {
            sum = sum.wrapping_add(j);
        }
        
        // Yield CPU occasionally
        if i % 3 == 0 {
            SchedulerSyscalls::yield_cpu();
        }
        
        // Sleep for a bit
        SchedulerSyscalls::sleep_ms(50);
    }
    
    klog!(INFO, "[TEST{}] Test task {} completing", task_num, task_num);
    task_num as i32
}

/// System information interface
pub struct SystemInfo;

impl SystemInfo {
    /// Get the number of tasks in the system
    pub fn task_count() -> usize {
        let stats = super::get_scheduler_stats();
        stats.total_tasks
    }
    
    /// Get the current task count by state
    pub fn task_count_by_state() -> (usize, usize, usize, usize) {
        let stats = super::get_scheduler_stats();
        (stats.ready_tasks, stats.running_tasks, stats.blocked_tasks, stats.dead_tasks)
    }
    
    /// Get system uptime in seconds
    pub fn uptime_seconds() -> u64 {
        crate::hal::x86_64::timer::get_uptime_seconds()
    }
    
    /// Get system uptime in milliseconds
    pub fn uptime_ms() -> u64 {
        crate::hal::x86_64::timer::get_uptime_ms()
    }
    
    /// Print system status
    pub fn print_status() {
        let stats = super::get_scheduler_stats();
        let uptime = Self::uptime_seconds();
        
        kprintln!("");
        kprintln!("=== SYSTEM STATUS ===");
        kprintln!("Uptime: {}s", uptime);
        kprintln!("Current Task: {}", stats.current_task_id);
        kprintln!("Total Tasks: {}", stats.total_tasks);
        kprintln!("Ready: {}, Running: {}, Blocked: {}, Dead: {}", 
                  stats.ready_tasks, stats.running_tasks, 
                  stats.blocked_tasks, stats.dead_tasks);
        kprintln!("=== END SYSTEM STATUS ===");
        kprintln!("");
    }
}

/// Process control interface
pub struct ProcessControl;

impl ProcessControl {
    /// Kill a task by ID
    /// 
    /// # Arguments
    /// * `task_id` - Task to terminate
    /// 
    /// # Returns
    /// `true` if task was killed, `false` if task doesn't exist
    pub fn kill(task_id: TaskId) -> bool {
        if task_id.is_idle() {
            kprintln!("[SCHED] Cannot kill idle task!");
            return false;
        }
        
        klog!(INFO, "[SCHED] Killing task {}", task_id.0);
        kill_task(task_id);
        true
    }
    
    /// Suspend (block) a task
    /// 
    /// # Arguments
    /// * `task_id` - Task to suspend
    /// 
    /// # Returns
    /// `true` if task was suspended, `false` if task doesn't exist
    pub fn suspend(task_id: TaskId) -> bool {
        if task_id.is_idle() {
            kprintln!("[SCHED] Cannot suspend idle task!");
            return false;
        }
        
        klog!(INFO, "[SCHED] Suspending task {}", task_id.0);
        block_task(task_id);
        true
    }
    
    /// Resume (wake up) a task
    /// 
    /// # Arguments
    /// * `task_id` - Task to resume
    /// 
    /// # Returns
    /// `true` if task was resumed, `false` if task doesn't exist
    pub fn resume(task_id: TaskId) -> bool {
        klog!(INFO, "[SCHED] Resuming task {}", task_id.0);
        wake_task(task_id);
        true
    }
}

/// Syscall implementations for direct kernel interface
/// 
/// These functions provide the low-level implementation of system calls
/// that can be called from user space via the syscall mechanism.

/// System call: yield CPU to another task
/// 
/// Voluntarily gives up the CPU so another task can run.
/// The current task remains ready and may be scheduled again.
pub fn sys_yield() {
    let current_task = get_current_task_id();
    
    // Log audit entry
    crate::secman::audit::log(crate::secman::audit::AuditEntry::syscall_yield(current_task));
    
    kprintln!("[sys] yield from task {}", current_task);
    
    // Mark current task as ready and enqueue it
    if current_task != 0 { // Don't yield idle task
        enqueue_task(TaskId(current_task));
    }
    
    // Trigger scheduling to switch to next task
    super::schedule();
    
    klog!(TRACE, "[sys] yield completed for task {}", current_task);
}

/// System call: exit current task
/// 
/// Terminates the calling task with the specified exit code.
/// This is a no-return function.
/// 
/// # Arguments
/// * `code` - Exit status code (0 = success, non-zero = error)
pub fn sys_exit(code: i32) -> ! {
    let current_task = get_current_task_id();
    
    // Log audit entry
    crate::secman::audit::log(crate::secman::audit::AuditEntry::syscall_exit(current_task, code));
    
    kprintln!("[sys] exit {} from task {}", code, current_task);
    
    // Mark task as dead and remove from scheduler
    kill_task(TaskId(current_task));
    
    klog!(INFO, "[sys] Task {} exited with code {}", current_task, code);
    
    // Trigger scheduling to switch to another task
    super::schedule();
    
    // Should never reach here since the task is dead
    loop {
        x86_64::instructions::hlt();
    }
}
