/// Scheduler Module for Polymera OS
/// 
/// This module provides the core scheduling functionality including task management,
/// runqueue operations, context switching, and timer integration.

pub mod task;
pub mod task_id;
pub mod runqueue;
pub mod context;
pub mod sys;
pub mod tick;
pub mod lock;
pub mod starvation;
pub mod fairness;

#[cfg(debug_assertions)]
pub mod test;

use crate::{kprintln, klog};
use spin::Mutex;
use lazy_static::lazy_static;

// Re-export key types for external use
pub use self::task::{Task, TaskId, TaskState, TaskPriority};
pub use self::task_id::TaskId as TaskIdV2;
pub use self::runqueue::RunQueue;
pub use self::context::CpuContext;

/// Global scheduler state
static NEXT_TASK_ID: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(1);
static CURRENT_TASK_ID: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

lazy_static! {
    /// Global runqueue for ready tasks
    static ref GLOBAL_RUNQUEUE: Mutex<runqueue::RunQueue> = Mutex::new(runqueue::RunQueue::new());
    
    /// Task storage (simplified for Phase 1)
    static ref TASK_STORAGE: Mutex<[Option<task::Task>; 256]> = Mutex::new([None; 256]);
    
    /// Current task context
    static ref CURRENT_CONTEXT: Mutex<context::CpuContext> = Mutex::new(context::CpuContext::new());
}

/// Initialize the scheduler subsystem
/// 
/// Sets up the initial runqueue and creates the idle task
pub fn init_sched() {
    kprintln!("[SCHED] Initializing scheduler subsystem");
    
    // Initialize the global runqueue
    let mut rq = GLOBAL_RUNQUEUE.lock();
    *rq = runqueue::RunQueue::new();
    drop(rq);
    
    // Create and register the idle task (task ID 0)
    let idle_task = task::Task::new(0, 0); // Idle task runs on current stack
    {
        let mut storage = TASK_STORAGE.lock();
        storage[0] = Some(idle_task);
    }
    
    // Set current task to idle
    CURRENT_TASK_ID.store(0, core::sync::atomic::Ordering::Relaxed);
    
    // Initialize starvation detector
    crate::sched::starvation::init();
    
    klog!(INFO, "[SCHED] Scheduler initialized with idle task");
    klog!(INFO, "[SCHED] Runqueue capacity: {}", runqueue::RunQueue::capacity());
}

/// Main scheduling function
/// 
/// Picks the next task from the runqueue and performs context switch
pub fn schedule() {
    // Check for starvation before scheduling
    crate::sched::starvation::check_starvation();
    
    // Get next task from runqueue
    let next_task_id = {
        let mut runqueue = GLOBAL_RUNQUEUE.lock();
        runqueue.pop()
    };
    
    let next_task_id = match next_task_id {
        Some(id) => id,
        None => {
            // No tasks to run, stay with current task or idle
            let current = get_current_task_id();
            if current == 0 {
                return; // Already running idle task
            } else {
                task::TaskId(0) // Switch to idle task
            }
        }
    };
    
    // Get current task ID
    let current_task_id = get_current_task_id();
    
    // If switching to the same task, no need to context switch
    if current_task_id == next_task_id.0 {
        return;
    }
    
    klog!(TRACE, "[SCHED] Context switch: {} -> {}", current_task_id, next_task_id.0);
    
    // Clear preemption flag for the task being switched to
    if let Some(task) = get_task(next_task_id) {
        if task.is_preemption_requested() {
            klog!(TRACE, "[SCHED] Clearing preemption flag for task {}", next_task_id.0);
            clear_task_preemption(next_task_id);
        }
    }
    
    // Trace the process switch for statistics
    crate::trace::trace_process_switch(current_task_id, next_task_id.0);
    
    // Perform context switch
    context_switch_to(next_task_id);
}

/// Get the current task ID
pub fn get_current_task_id() -> u64 {
    CURRENT_TASK_ID.load(core::sync::atomic::Ordering::Relaxed)
}

/// Set the current task ID
pub fn set_current_task_id(task_id: u64) {
    CURRENT_TASK_ID.store(task_id, core::sync::atomic::Ordering::Relaxed);
}

/// Create a new task
/// 
/// Returns the TaskId of the newly created task
pub fn create_task(kstack_top: u64) -> TaskId {
    let task_id = NEXT_TASK_ID.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    let task = Task::new(task_id, kstack_top);
    
    // Store the task
    {
        let mut storage = TASK_STORAGE.lock();
        if let Some(slot) = storage.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(task);
        } else {
            kprintln!("[SCHED] Warning: Task storage full, cannot create task {}", task_id);
            return TaskId(0); // Return invalid task ID
        }
    }
    
    // Add to runqueue
    {
        let mut runqueue = GLOBAL_RUNQUEUE.lock();
        runqueue.push(TaskId(task_id), task.priority);
    }
    
    klog!(INFO, "[SCHED] Created task {} with stack at 0x{:016x}", task_id, kstack_top);
    TaskId(task_id)
}

/// Get task by ID
pub fn get_task(task_id: TaskId) -> Option<Task> {
    let storage = TASK_STORAGE.lock();
    for task_opt in storage.iter() {
        if let Some(task) = task_opt {
            if task.id == task_id {
                return Some(*task);
            }
        }
    }
    None
}

/// Set the preemption flag for a task
pub fn request_task_preemption(task_id: TaskId) {
    let mut storage = TASK_STORAGE.lock();
    for task_opt in storage.iter_mut() {
        if let Some(ref mut task) = task_opt {
            if task.id == task_id {
                task.request_preemption();
                klog!(TRACE, "[SCHED] Preemption requested for task {}", task_id.0);
                return;
            }
        }
    }
}

/// Clear the preemption flag for a task
pub fn clear_task_preemption(task_id: TaskId) {
    let mut storage = TASK_STORAGE.lock();
    for task_opt in storage.iter_mut() {
        if let Some(ref mut task) = task_opt {
            if task.id == task_id {
                task.clear_preemption_request();
                klog!(TRACE, "[SCHED] Preemption flag cleared for task {}", task_id.0);
                return;
            }
        }
    }
}

/// Set the wake timestamp for a task
pub fn set_task_wake_timestamp(task_id: TaskId, timestamp: u64) {
    let mut storage = TASK_STORAGE.lock();
    for task_opt in storage.iter_mut() {
        if let Some(ref mut task) = task_opt {
            if task.id == task_id {
                task.set_wake_timestamp(timestamp);
                klog!(TRACE, "[SCHED] Wake timestamp set for task {}: {}ms", task_id.0, timestamp);
                return;
            }
        }
    }
}

/// Clear the wake timestamp for a task
pub fn clear_task_wake_timestamp(task_id: TaskId) {
    let mut storage = TASK_STORAGE.lock();
    for task_opt in storage.iter_mut() {
        if let Some(ref mut task) = task_opt {
            if task.id == task_id {
                task.clear_wake_timestamp();
                klog!(TRACE, "[SCHED] Wake timestamp cleared for task {}", task_id.0);
                return;
            }
        }
    }
}

/// Update task state
pub fn set_task_state(task_id: TaskId, new_state: TaskState) {
    let mut storage = TASK_STORAGE.lock();
    for task_opt in storage.iter_mut() {
        if let Some(task) = task_opt {
            if task.id == task_id {
                let old_state = task.state;
                task.state = new_state;
                klog!(TRACE, "[SCHED] Task {} state: {:?} -> {:?}", task_id.0, old_state, new_state);
                return;
            }
        }
    }
}

/// Add task to runqueue (make it ready to run)
pub fn enqueue_task(task_id: TaskId) {
    // Set task state to Ready
    set_task_state(task_id, TaskState::Ready);
    
    // Record task ready for starvation detection
    crate::sched::starvation::record_task_ready(task_id);
    
    // Get task priority
    let priority = get_task(task_id).map(|t| t.priority).unwrap_or(TaskPriority::Normal);

    // Add to runqueue
    let mut runqueue = GLOBAL_RUNQUEUE.lock();
    runqueue.push(task_id, priority);
    
    klog!(TRACE, "[SCHED] Task {} enqueued", task_id.0);
}

/// Remove task from scheduling (mark as blocked)
pub fn block_task(task_id: TaskId) {
    set_task_state(task_id, TaskState::Blocked);
    klog!(TRACE, "[SCHED] Task {} blocked", task_id.0);
}

/// Wake up a blocked task
pub fn wake_task(task_id: TaskId) {
    set_task_state(task_id, TaskState::Ready);
    enqueue_task(task_id);
    
    // Record wake timestamp for wake-to-run latency tracking
    let wake_timestamp = crate::log::get_current_time_ms();
    set_task_wake_timestamp(task_id, wake_timestamp);
    
    klog!(TRACE, "[SCHED] Task {} woken up at {}ms", task_id.0, wake_timestamp);
}

/// Set task state (public interface)
pub fn set_state(task_id: TaskId, new_state: TaskState) {
    set_task_state(task_id, new_state);
}

/// Wake up a task (public interface)
pub fn wake(task_id: TaskId) {
    wake_task(task_id);
}

/// Block current task and schedule next task
pub fn block_current_and_schedule() {
    let current_task_id = get_current_task_id();
    
    if current_task_id != 0 { // Don't block idle task
        block_task(TaskId(current_task_id));
        klog!(TRACE, [crate::log::tags::SCHED], "Current task {} blocked, triggering schedule", current_task_id);
        schedule();
    }
}

/// Preempt current task if needed based on priority
/// Returns true if preemption occurred
pub fn preempt(should_preempt: bool) -> bool {
    if should_preempt {
        let current_task_id = get_current_task_id();
        
        if current_task_id != 0 { // Don't preempt idle task
            // Re-enqueue current task to give it another chance later
            enqueue_task(TaskId(current_task_id));
                            klog!(TRACE, [crate::log::tags::SCHED], "Preempting task {}", current_task_id);
            schedule();
            return true;
        }
    }
    false
}

/// Terminate a task
pub fn kill_task(task_id: TaskId) {
    set_task_state(task_id, TaskState::Dead);
    
    // Remove from storage
    let mut storage = TASK_STORAGE.lock();
    for task_opt in storage.iter_mut() {
        if let Some(task) = task_opt {
            if task.id == task_id {
                *task_opt = None;
                klog!(INFO, "[SCHED] Task {} terminated", task_id.0);
                return;
            }
        }
    }
}

/// Perform context switch to specified task
fn context_switch_to(next_task_id: TaskId) {
    let current_task_id = get_current_task_id();
    
    // Get task contexts
    let (current_context, next_context) = {
        let storage = TASK_STORAGE.lock();
        
        let next_task = storage.iter()
            .find_map(|t| t.as_ref().filter(|task| task.id == next_task_id));
        
        match next_task {
            Some(next) => {
                // For Phase 1, we'll use simplified context switching
                // current_context is just a dummy since current_task might be dead
                (CpuContext::new(), CpuContext::from_stack(next.kstack_top))
            }
            None => {
                klog!(TRACE, "[SCHED] Context switch failed: invalid next task ID");
                return;
            }
        }
    };
    
    // Update current task state and next task state
    if current_task_id != 0 { // Don't change idle task state
        set_task_state(TaskId(current_task_id), TaskState::Ready);
    }
    set_task_state(next_task_id, TaskState::Running);
    
    // Calculate and record wake-to-run latency if this task was woken up
    if let Some(next_task) = get_task(next_task_id) {
        if let Some(wake_timestamp) = next_task.get_wake_timestamp() {
            let current_time = crate::log::get_current_time_ms();
            if current_time >= wake_timestamp {
                let latency_ms = current_time - wake_timestamp;
                let latency_us = (latency_ms * 1000) as u32; // Convert to microseconds
                
                // Record the wake-to-run latency
                crate::trace::record_wake_to_run_latency(latency_us);
                
                klog!(TRACE, "[SCHED] Task {} wake-to-run latency: {}ms ({}μs)", 
                      next_task_id.0, latency_ms, latency_us);
                
                // Clear the wake timestamp since the task is now running
                clear_task_wake_timestamp(next_task_id);
            }
        }
    }
    
    // Update current task ID
    set_current_task_id(next_task_id.0);
    
    // Record task scheduled for starvation detection
    crate::sched::starvation::record_task_scheduled(next_task_id);
    
    // Perform the actual context switch
    // For Phase 1, this is a simplified implementation
    // In a full kernel, we'd use the assembly context_switch function
    unsafe {
        let mut current_ctx = CURRENT_CONTEXT.lock();
        *current_ctx = next_context;
        
        // In a real implementation, we would call:
        // context::context_switch(&mut current_context, &next_context);
        
        // For now, we just update our tracking
        klog!(TRACE, "[SCHED] Context switched to task {}", next_task_id.0);
    }
}

/// Yield the current task (voluntarily give up CPU)
pub fn yield_current() {
    let current_task_id = get_current_task_id();
    
    if current_task_id != 0 { // Don't yield idle task
        // Re-enqueue current task
        enqueue_task(TaskId(current_task_id));
    }
    
    // Trigger scheduling
    schedule();
}

/// Get scheduler statistics
pub fn get_scheduler_stats() -> SchedulerStats {
    let runqueue = GLOBAL_RUNQUEUE.lock();
    let storage = TASK_STORAGE.lock();
    
    let mut stats = SchedulerStats {
        total_tasks: 0,
        ready_tasks: runqueue.len(),
        running_tasks: 0,
        blocked_tasks: 0,
        dead_tasks: 0,
        current_task_id: get_current_task_id(),
        starvation_warnings: 0,
        starvation_critical: 0,
        currently_starving: 0,
    };
    
    // Get starvation statistics
    let starvation_stats = crate::sched::starvation::get_starvation_stats();
    stats.starvation_warnings = starvation_stats.total_warnings;
    stats.starvation_critical = starvation_stats.total_critical;
    stats.currently_starving = starvation_stats.currently_starving;
    
    for task_opt in storage.iter() {
        if let Some(task) = task_opt {
            stats.total_tasks += 1;
            match task.state {
                TaskState::Ready => {}, // Already counted in ready_tasks
                TaskState::Running => stats.running_tasks += 1,
                TaskState::Blocked => stats.blocked_tasks += 1,
                TaskState::Dead => stats.dead_tasks += 1,
            }
        }
    }
    
    stats
}

/// Print scheduler statistics
pub fn print_scheduler_stats() {
    let stats = get_scheduler_stats();
    
    kprintln!("");
    kprintln!("=== SCHEDULER STATISTICS ===");
    kprintln!("Current Task ID: {}", stats.current_task_id);
    kprintln!("Total Tasks: {}", stats.total_tasks);
    kprintln!("Ready Tasks: {}", stats.ready_tasks);
    kprintln!("Running Tasks: {}", stats.running_tasks);
    kprintln!("Blocked Tasks: {}", stats.blocked_tasks);
    kprintln!("Dead Tasks: {}", stats.dead_tasks);
    kprintln!("Starvation Warnings: {}", stats.starvation_warnings);
    kprintln!("Starvation Critical: {}", stats.starvation_critical);
    kprintln!("Currently Starving: {}", stats.currently_starving);
    kprintln!("=== END SCHEDULER STATISTICS ===");
    kprintln!("");
}

/// Scheduler statistics structure
#[derive(Debug, Clone, Copy)]
pub struct SchedulerStats {
    pub total_tasks: usize,
    pub ready_tasks: usize,
    pub running_tasks: usize,
    pub blocked_tasks: usize,
    pub dead_tasks: usize,
    pub current_task_id: u64,
    pub starvation_warnings: usize,
    pub starvation_critical: usize,
    pub currently_starving: usize,
}

/// Test function for scheduler functionality
#[allow(dead_code)]
pub fn test_scheduler() {
    kprintln!("Testing scheduler functionality...");
    
    // Print initial stats
    print_scheduler_stats();
    
    // Test task creation
    let task1 = create_task(0x1000);
    let task2 = create_task(0x2000);
    
    kprintln!("Created tasks: {} and {}", task1.0, task2.0);
    
    // Print stats after task creation
    print_scheduler_stats();
    
    // Test scheduling
    schedule();
    
    kprintln!("Scheduler test completed");
}
