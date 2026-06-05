/// Task Management for Polymera OS Scheduler
/// 
/// This module defines the core task structures and state management
/// for the kernel's preemptive scheduler.

use core::fmt;

/// Unique task identifier
/// 
/// TaskId is a wrapper around u64 that provides type safety for task references.
/// Task ID 0 is reserved for the idle task.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskId(pub u64);

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TaskId {
    /// Create a new TaskId
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
    
    /// Get the raw ID value
    pub const fn as_u64(self) -> u64 {
        self.0
    }
    
    /// Check if this is the idle task
    pub const fn is_idle(self) -> bool {
        self.0 == 0
    }
    
    /// Get the idle task ID
    pub const fn idle() -> Self {
        Self(0)
    }
}

/// Task execution state
/// 
/// Represents the current state of a task in the scheduler's state machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    /// Task is ready to run and waiting in the runqueue
    Ready,
    /// Task is currently executing on a CPU
    Running,
    /// Task is blocked waiting for some event (I/O, synchronization, etc.)
    Blocked,
    /// Task has terminated and resources can be cleaned up
    Dead,
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskState::Ready => write!(f, "Ready"),
            TaskState::Running => write!(f, "Running"),
            TaskState::Blocked => write!(f, "Blocked"),
            TaskState::Dead => write!(f, "Dead"),
        }
    }
}

/// Task priority levels
/// 
/// Higher values indicate higher priority. The scheduler will prefer
/// to run higher priority tasks before lower priority ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// Lowest priority - background tasks
    Low = 0,
    /// Normal priority - default for user tasks
    Normal = 1,
    /// High priority - important system tasks
    High = 2,
    /// Real-time priority - time-critical tasks
    RealTime = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

/// Core task structure
/// 
/// Represents a single task/thread in the system. Contains the minimal
/// information needed for task scheduling and context switching.
#[derive(Clone, Copy, Debug)]
pub struct Task {
    /// Unique task identifier
    pub id: TaskId,
    
    /// Current task state
    pub state: TaskState,
    
    /// Top of kernel stack (used for context switching)
    /// 
    /// This points to the top of the kernel stack where the task's
    /// context (registers) are saved when the task is not running.
    pub kstack_top: u64,
    
    /// Task priority for scheduling decisions
    pub priority: TaskPriority,
    
    /// CPU time accounting (in timer ticks)
    pub cpu_time: u64,
    
    /// Time quantum remaining (in timer ticks)
    /// 
    /// When this reaches 0, the task should be preempted.
    /// Default quantum is 10ms (10 ticks at 1000Hz).
    pub time_slice: u32,
    
    /// Flag indicating this task should be preempted immediately
    /// 
    /// This is set when a high-priority message arrives for a blocked task,
    /// causing the current task to be preempted in favor of the woken RT task.
    pub preemption_requested: bool,
    
    /// Timestamp when task was last woken up (for wake-to-run latency tracking)
    /// 
    /// This is set when a blocked task is woken up and cleared when the task
    /// actually starts running. Used to measure wake-to-run latency.
    pub wake_timestamp: Option<u64>,

    /// CPU Affinity Mask (Bitmask of allowed CPUs)
    /// Default: !0 (All CPUs)
    pub affinity_mask: u64,

    /// Preferred NUMA Node
    /// Default: 0
    pub preferred_node: u32,
}

impl Task {
    /// Create a new task with the specified ID and kernel stack
    /// 
    /// # Arguments
    /// * `id` - Unique task identifier
    /// * `kstack_top` - Top address of the kernel stack for this task
    /// 
    /// # Returns
    /// A new Task instance in the Ready state with default priority
    pub fn new(id: u64, kstack_top: u64) -> Self {
        Self {
            id: TaskId(id),
            state: TaskState::Ready,
            kstack_top,
            priority: TaskPriority::default(),
            cpu_time: 0,
            time_slice: DEFAULT_TIME_SLICE,
            preemption_requested: false,
            wake_timestamp: None,
            affinity_mask: !0,
            preferred_node: 0,
        }
    }
    
    /// Create a new task with specified priority
    /// 
    /// # Arguments
    /// * `id` - Unique task identifier
    /// * `kstack_top` - Top address of the kernel stack for this task
    /// * `priority` - Task priority level
    /// 
    /// # Returns
    /// A new Task instance in the Ready state with the specified priority
    pub fn with_priority(id: u64, kstack_top: u64, priority: TaskPriority) -> Self {
        Self {
            id: TaskId(id),
            state: TaskState::Ready,
            kstack_top,
            priority,
            cpu_time: 0,
            time_slice: priority_to_time_slice(priority),
            preemption_requested: false,
            wake_timestamp: None,
            affinity_mask: !0,
            preferred_node: 0,
        }
    }
    
    /// Create the idle task (task ID 0)
    /// 
    /// The idle task runs when no other tasks are ready and consumes
    /// minimal resources.
    pub fn idle() -> Self {
        Self {
            id: TaskId::idle(),
            state: TaskState::Running, // Idle task starts running
            kstack_top: 0, // Idle task uses the current stack
            priority: TaskPriority::Low,
            cpu_time: 0,
            time_slice: u32::MAX, // Idle task never gets preempted by time
            preemption_requested: false,
            wake_timestamp: None,
            affinity_mask: !0,
            preferred_node: 0,
        }
    }
    
    /// Check if this task is the idle task
    pub fn is_idle(&self) -> bool {
        self.id.is_idle()
    }
    
    /// Update the task's state
    /// 
    /// # Arguments
    /// * `new_state` - The new state to transition to
    pub fn set_state(&mut self, new_state: TaskState) {
        self.state = new_state;
    }
    
    /// Check if the task is ready to run
    pub fn is_ready(&self) -> bool {
        self.state == TaskState::Ready
    }
    
    /// Check if the task is currently running
    pub fn is_running(&self) -> bool {
        self.state == TaskState::Running
    }
    
    /// Check if the task is blocked
    pub fn is_blocked(&self) -> bool {
        self.state == TaskState::Blocked
    }
    
    /// Check if the task is dead
    pub fn is_dead(&self) -> bool {
        self.state == TaskState::Dead
    }
    
    /// Add CPU time to this task
    /// 
    /// Called by the scheduler on each timer tick when this task is running.
    /// 
    /// # Arguments
    /// * `ticks` - Number of timer ticks to add
    pub fn add_cpu_time(&mut self, ticks: u64) {
        self.cpu_time = self.cpu_time.saturating_add(ticks);
    }
    
    /// Decrement the time slice
    /// 
    /// Returns true if the time slice has expired and the task should be preempted.
    /// 
    /// # Returns
    /// `true` if the time slice has expired, `false` otherwise
    pub fn tick_time_slice(&mut self) -> bool {
        if self.time_slice > 0 {
            self.time_slice -= 1;
        }
        self.time_slice == 0
    }
    
    /// Reset the time slice to the default value for this task's priority
    pub fn reset_time_slice(&mut self) {
        self.time_slice = priority_to_time_slice(self.priority);
    }
    
    /// Get remaining time slice
    pub fn remaining_time_slice(&self) -> u32 {
        self.time_slice
    }
    
    /// Request immediate preemption of this task
    /// 
    /// Called when a high-priority message arrives for a blocked task,
    /// causing the current task to be preempted in favor of the woken RT task.
    pub fn request_preemption(&mut self) {
        self.preemption_requested = true;
    }
    
    /// Check if preemption has been requested for this task
    /// 
    /// # Returns
    /// `true` if preemption is requested, `false` otherwise
    pub fn is_preemption_requested(&self) -> bool {
        self.preemption_requested
    }
    
    /// Clear the preemption request flag
    /// 
    /// Called after the task has been preempted or when the flag is no longer needed.
    pub fn clear_preemption_request(&mut self) {
        self.preemption_requested = false;
    }
    
    /// Set the wake timestamp for this task
    /// 
    /// Called when a blocked task is woken up to track wake-to-run latency.
    /// 
    /// # Arguments
    /// * `timestamp` - Current timestamp in milliseconds
    pub fn set_wake_timestamp(&mut self, timestamp: u64) {
        self.wake_timestamp = Some(timestamp);
    }
    
    /// Get the wake timestamp for this task
    /// 
    /// Returns the timestamp when this task was last woken up, if any.
    /// 
    /// # Returns
    /// Some(timestamp) if the task was woken up, None otherwise
    pub fn get_wake_timestamp(&self) -> Option<u64> {
        self.wake_timestamp
    }
    
    /// Clear the wake timestamp for this task
    /// 
    /// Called when the task actually starts running to complete wake-to-run latency measurement.
    pub fn clear_wake_timestamp(&mut self) {
        self.wake_timestamp = None;
    }
    
    /// Get CPU utilization as a percentage (simplified)
    /// 
    /// # Arguments
    /// * `total_ticks` - Total system uptime in ticks
    /// 
    /// # Returns
    /// CPU utilization as a value between 0.0 and 100.0
    pub fn cpu_utilization(&self, total_ticks: u64) -> f32 {
        if total_ticks == 0 {
            0.0
        } else {
            (self.cpu_time as f32 / total_ticks as f32) * 100.0
        }
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Task[{}]: state={}, priority={:?}, cpu_time={}, time_slice={}",
            self.id, self.state, self.priority, self.cpu_time, self.time_slice
        )
    }
}

/// Default time slice in timer ticks (10ms at 1000Hz = 10 ticks)
pub const DEFAULT_TIME_SLICE: u32 = 10;

/// Convert priority to time slice
/// 
/// Higher priority tasks get longer time slices to reduce context switching overhead.
/// 
/// # Arguments
/// * `priority` - Task priority level
/// 
/// # Returns
/// Time slice in timer ticks
fn priority_to_time_slice(priority: TaskPriority) -> u32 {
    match priority {
        TaskPriority::Low => 5,        // 5ms
        TaskPriority::Normal => 10,    // 10ms  
        TaskPriority::High => 20,      // 20ms
        TaskPriority::RealTime => 50,  // 50ms
    }
}

/// Task creation parameters
/// 
/// Used when creating new tasks to specify their initial configuration.
#[derive(Debug, Clone)]
pub struct TaskParams {
    /// Task priority
    pub priority: TaskPriority,
    
    /// Initial kernel stack size (in bytes)
    pub stack_size: usize,
    
    /// Task name for debugging
    pub name: Option<&'static str>,
}

impl Default for TaskParams {
    fn default() -> Self {
        Self {
            priority: TaskPriority::Normal,
            stack_size: DEFAULT_STACK_SIZE,
            name: None,
        }
    }
}

/// Default kernel stack size (4KB)
pub const DEFAULT_STACK_SIZE: usize = 4096;

/// Task statistics for monitoring and debugging
#[derive(Debug, Clone, Copy)]
pub struct TaskStats {
    pub id: TaskId,
    pub state: TaskState,
    pub priority: TaskPriority,
    pub cpu_time: u64,
    pub time_slice: u32,
    pub cpu_utilization: f32,
}

impl TaskStats {
    /// Create task statistics from a task
    /// 
    /// # Arguments
    /// * `task` - The task to create statistics for
    /// * `total_ticks` - Total system uptime for CPU utilization calculation
    /// 
    /// # Returns
    /// TaskStats structure with current task information
    pub fn from_task(task: &Task, total_ticks: u64) -> Self {
        Self {
            id: task.id,
            state: task.state,
            priority: task.priority,
            cpu_time: task.cpu_time,
            time_slice: task.time_slice,
            cpu_utilization: task.cpu_utilization(total_ticks),
        }
    }
}

/// Task state transition validation
/// 
/// Ensures that task state transitions are valid according to the state machine.
/// 
/// # Arguments
/// * `from` - Current state
/// * `to` - Desired new state
/// 
/// # Returns
/// `true` if the transition is valid, `false` otherwise
pub fn is_valid_state_transition(from: TaskState, to: TaskState) -> bool {
    use TaskState::*;
    
    match (from, to) {
        // Ready can transition to Running or Dead
        (Ready, Running) | (Ready, Dead) => true,
        
        // Running can transition to Ready, Blocked, or Dead
        (Running, Ready) | (Running, Blocked) | (Running, Dead) => true,
        
        // Blocked can transition to Ready or Dead
        (Blocked, Ready) | (Blocked, Dead) => true,
        
        // Dead is terminal (no transitions out)
        (Dead, _) => false,
        
        // Self-transitions are always valid (no-op)
        (state, new_state) if state == new_state => true,
        
        // All other transitions are invalid
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_creation() {
        let task = Task::new(42, 0x1000);
        assert_eq!(task.id.0, 42);
        assert_eq!(task.state, TaskState::Ready);
        assert_eq!(task.kstack_top, 0x1000);
        assert_eq!(task.priority, TaskPriority::Normal);
        assert_eq!(task.time_slice, DEFAULT_TIME_SLICE);
    }
    
    #[test]
    fn test_idle_task() {
        let idle = Task::idle();
        assert!(idle.is_idle());
        assert_eq!(idle.id.0, 0);
        assert_eq!(idle.state, TaskState::Running);
    }
    
    #[test]
    fn test_state_transitions() {
        assert!(is_valid_state_transition(TaskState::Ready, TaskState::Running));
        assert!(is_valid_state_transition(TaskState::Running, TaskState::Blocked));
        assert!(!is_valid_state_transition(TaskState::Dead, TaskState::Ready));
    }
    
    #[test]
    fn test_time_slice() {
        let mut task = Task::new(1, 0x1000);
        assert_eq!(task.time_slice, DEFAULT_TIME_SLICE);
        
        // Tick down time slice
        for _ in 0..DEFAULT_TIME_SLICE - 1 {
            assert!(!task.tick_time_slice());
        }
        
        // Last tick should expire the time slice
        assert!(task.tick_time_slice());
        assert_eq!(task.time_slice, 0);
    }
}

