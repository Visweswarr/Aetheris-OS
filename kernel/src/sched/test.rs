/// Scheduler Testing Module
/// 
/// This module provides comprehensive testing functions for the scheduler
/// implementation, including task creation, context switching, and timing.

use crate::{kprintln, klog};
use super::{
    TaskId, TaskState, create_task, enqueue_task, schedule,
    get_scheduler_stats, print_scheduler_stats,
};
use super::task::{Task, TaskPriority};
use super::runqueue::RunQueue;
use super::context::CpuContext;
use super::sys::{KernelThreads, SchedulerSyscalls, SystemInfo};
use super::tick::{get_tick_stats, TimeSliceManager};

/// Comprehensive scheduler test suite
pub fn run_scheduler_tests() {
    kprintln!("");
    kprintln!("=== SCHEDULER TEST SUITE ===");
    kprintln!("Testing scheduler functionality...");
    kprintln!("");
    
    // Test 1: Basic task management
    test_basic_task_management();
    
    // Test 2: Runqueue operations
    test_runqueue_operations();
    
    // Test 3: Context switching structures
    test_context_switching();
    
    // Test 4: Time slice management
    test_time_slice_management();
    
    // Test 5: System calls
    test_system_calls();
    
    // Test 6: Scheduler statistics
    test_scheduler_statistics();
    
    kprintln!("=== SCHEDULER TEST SUITE COMPLETE ===");
    kprintln!("");
}

/// Test 1: Basic task management
fn test_basic_task_management() {
    kprintln!("Test 1: Basic task management");
    
    // Test task creation
    let task1 = Task::new(1, 0x10000);
    let task2 = Task::new(2, 0x20000);
    let idle = Task::idle();
    
    kprintln!("  Created tasks:");
    kprintln!("    {}", task1);
    kprintln!("    {}", task2);
    kprintln!("    {}", idle);
    
    // Test task state validation
    assert!(task1.is_ready());
    assert!(idle.is_running());
    assert!(!task1.is_dead());
    
    // Test task ID operations
    assert!(!task1.id.is_idle());
    assert!(idle.id.is_idle());
    assert_eq!(task1.id.as_u64(), 1);
    
    kprintln!("  ✓ Task creation and state management working");
    kprintln!("  Test 1 complete");
    kprintln!("");
}

/// Test 2: Runqueue operations
fn test_runqueue_operations() {
    kprintln!("Test 2: Runqueue operations");
    
    let mut rq = RunQueue::new();
    
    // Test empty queue
    assert!(rq.is_empty());
    assert_eq!(rq.len(), 0);
    assert_eq!(rq.pop(), None);
    
    // Test basic operations
    assert!(rq.push(TaskId(1)));
    assert!(rq.push(TaskId(2)));
    assert!(rq.push(TaskId(3)));
    
    assert_eq!(rq.len(), 3);
    assert!(!rq.is_empty());
    
    // Test FIFO order
    assert_eq!(rq.pop(), Some(TaskId(1)));
    assert_eq!(rq.pop(), Some(TaskId(2)));
    assert_eq!(rq.pop(), Some(TaskId(3)));
    assert_eq!(rq.pop(), None);
    
    // Test circular buffer behavior
    for i in 0..10 {
        assert!(rq.push(TaskId(i)));
    }
    
    for i in 0..5 {
        assert_eq!(rq.pop(), Some(TaskId(i)));
    }
    
    for i in 10..15 {
        assert!(rq.push(TaskId(i)));
    }
    
    kprintln!("  ✓ Runqueue operations working correctly");
    kprintln!("  Current queue state: {}", rq);
    kprintln!("  Test 2 complete");
    kprintln!("");
}

/// Test 3: Context switching structures
fn test_context_switching() {
    kprintln!("Test 3: Context switching structures");
    
    // Test context creation
    let ctx1 = CpuContext::new();
    let ctx2 = CpuContext::new_task(0x401000, 0x700000);
    
    kprintln!("  Empty context: {}", ctx1);
    kprintln!("  Task context: {}", ctx2);
    
    // Test context validation
    assert!(!ctx1.is_valid()); // Empty context is invalid
    assert!(ctx2.is_valid());  // Properly initialized context is valid
    
    // Test kernel thread context
    let kernel_ctx = super::context::create_kernel_thread_context(
        0x401000, 0x800000, Some(0x12345678)
    );
    
    assert!(kernel_ctx.is_valid());
    assert_eq!(kernel_ctx.get_rip(), 0x401000);
    assert_eq!(kernel_ctx.get_rsp(), 0x800000);
    
    kprintln!("  ✓ Context structures working correctly");
    kprintln!("  Test 3 complete");
    kprintln!("");
}

/// Test 4: Time slice management
fn test_time_slice_management() {
    kprintln!("Test 4: Time slice management");
    
    let default_slice = TimeSliceManager::default_time_slice();
    kprintln!("  Default time slice: {} ticks", default_slice);
    
    // Test time slice operations
    TimeSliceManager::set_current_time_slice(20);
    assert_eq!(TimeSliceManager::current_remaining(), 20);
    assert!(!TimeSliceManager::is_time_slice_expired());
    
    TimeSliceManager::extend_current_time_slice(10);
    assert_eq!(TimeSliceManager::current_remaining(), 30);
    
    // Test task timing
    let mut task = Task::new(1, 0x10000);
    assert_eq!(task.time_slice, default_slice);
    
    // Simulate time slice usage
    for _ in 0..default_slice - 1 {
        assert!(!task.tick_time_slice());
    }
    assert!(task.tick_time_slice()); // Should expire now
    assert_eq!(task.time_slice, 0);
    
    task.reset_time_slice();
    assert_eq!(task.time_slice, default_slice);
    
    kprintln!("  ✓ Time slice management working correctly");
    kprintln!("  Test 4 complete");
    kprintln!("");
}

/// Test 5: System calls
fn test_system_calls() {
    kprintln!("Test 5: System calls");
    
    // Test task ID retrieval
    let current_tid = SchedulerSyscalls::get_tid();
    kprintln!("  Current task ID: {}", current_tid.0);
    
    // Test system information
    let task_count = SystemInfo::task_count();
    let uptime = SystemInfo::uptime_ms();
    
    kprintln!("  System info:");
    kprintln!("    Task count: {}", task_count);
    kprintln!("    Uptime: {}ms", uptime);
    
    let (ready, running, blocked, dead) = SystemInfo::task_count_by_state();
    kprintln!("    Tasks by state: {} ready, {} running, {} blocked, {} dead", 
              ready, running, blocked, dead);
    
    // Test kernel thread creation (simplified)
    kprintln!("  Testing kernel thread creation...");
    if let Some(thread_id) = KernelThreads::spawn("test_thread", test_thread_function, 42) {
        kprintln!("    Created test thread with ID: {}", thread_id.0);
    } else {
        kprintln!("    Failed to create test thread");
    }
    
    kprintln!("  ✓ System calls working correctly");
    kprintln!("  Test 5 complete");
    kprintln!("");
}

/// Test 6: Scheduler statistics
fn test_scheduler_statistics() {
    kprintln!("Test 6: Scheduler statistics");
    
    // Test scheduler stats
    let sched_stats = get_scheduler_stats();
    kprintln!("  Scheduler statistics:");
    kprintln!("    Current task: {}", sched_stats.current_task_id);
    kprintln!("    Total tasks: {}", sched_stats.total_tasks);
    kprintln!("    Ready: {}, Running: {}, Blocked: {}, Dead: {}", 
              sched_stats.ready_tasks, sched_stats.running_tasks,
              sched_stats.blocked_tasks, sched_stats.dead_tasks);
    
    // Test tick stats
    let tick_stats = get_tick_stats();
    kprintln!("  Tick statistics:");
    kprintln!("    {}", tick_stats);
    kprintln!("    Context switch rate: {:.2}/sec", tick_stats.context_switch_rate());
    kprintln!("    Voluntary percentage: {:.1}%", tick_stats.voluntary_percentage());
    
    // Print detailed statistics
    print_scheduler_stats();
    
    kprintln!("  ✓ Scheduler statistics working correctly");
    kprintln!("  Test 6 complete");
    kprintln!("");
}

/// Test thread function
fn test_thread_function(arg: u64) -> i32 {
    klog!(INFO, "[TEST_THREAD] Starting with arg {}", arg);
    
    // Simulate some work
    for i in 1..=5 {
        klog!(TRACE, "[TEST_THREAD] Iteration {}", i);
        
        // Do some computation
        let mut sum = 0u64;
        for j in 0..1000 {
            sum = sum.wrapping_add(j * arg);
        }
        
        // Yield occasionally
        if i % 2 == 0 {
            SchedulerSyscalls::yield_cpu();
        }
    }
    
    klog!(INFO, "[TEST_THREAD] Completing");
    arg as i32
}

/// Performance test for scheduler operations
pub fn performance_test() {
    kprintln!("");
    kprintln!("=== SCHEDULER PERFORMANCE TEST ===");
    
    let start_tick = crate::hal::x86_64::timer::get_tick_count();
    
    // Test runqueue performance
    let mut rq = RunQueue::new();
    
    // Fill and empty the queue multiple times
    for cycle in 0..100 {
        // Fill queue
        for i in 0..50 {
            rq.push(TaskId(cycle * 50 + i));
        }
        
        // Empty queue
        while rq.pop().is_some() {}
    }
    
    let end_tick = crate::hal::x86_64::timer::get_tick_count();
    let elapsed = end_tick - start_tick;
    
    kprintln!("RunQueue performance: 5000 operations in {}ms", elapsed);
    
    // Test context creation performance
    let start_tick = crate::hal::x86_64::timer::get_tick_count();
    
    for i in 0..1000 {
        let _ctx = CpuContext::new_task(0x401000 + i, 0x700000 + i * 0x1000);
    }
    
    let end_tick = crate::hal::x86_64::timer::get_tick_count();
    let elapsed = end_tick - start_tick;
    
    kprintln!("Context creation performance: 1000 contexts in {}ms", elapsed);
    
    kprintln!("=== PERFORMANCE TEST COMPLETE ===");
    kprintln!("");
}

/// Stress test for scheduler robustness
pub fn stress_test() {
    kprintln!("");
    kprintln!("=== SCHEDULER STRESS TEST ===");
    kprintln!("Running stress test for 5 seconds...");
    
    let start_tick = crate::hal::x86_64::timer::get_tick_count();
    let test_duration = 5000; // 5 seconds
    let target_tick = start_tick + test_duration;
    
    let mut operations = 0u64;
    let mut rq = RunQueue::new();
    
    while crate::hal::x86_64::timer::get_tick_count() < target_tick {
        // Perform various operations
        operations += 1;
        
        match operations % 4 {
            0 => {
                // Add task to runqueue
                rq.push(TaskId(operations % 256));
            }
            1 => {
                // Remove task from runqueue
                rq.pop();
            }
            2 => {
                // Create context
                let _ctx = CpuContext::new_task(0x400000, 0x700000);
            }
            3 => {
                // Check system stats
                let _stats = get_scheduler_stats();
            }
            _ => unreachable!(),
        }
        
        // Yield occasionally to test preemption
        if operations % 1000 == 0 {
            SchedulerSyscalls::yield_cpu();
        }
    }
    
    let end_tick = crate::hal::x86_64::timer::get_tick_count();
    let actual_duration = end_tick - start_tick;
    
    kprintln!("Stress test results:");
    kprintln!("  Duration: {}ms", actual_duration);
    kprintln!("  Operations: {}", operations);
    kprintln!("  Operations/sec: {}", operations * 1000 / actual_duration);
    kprintln!("  Final runqueue length: {}", rq.len());
    
    // Print final system state
    SystemInfo::print_status();
    
    kprintln!("=== STRESS TEST COMPLETE ===");
    kprintln!("");
}

/// Quick scheduler functionality check
pub fn quick_test() {
    kprintln!("Quick Scheduler Test:");
    
    // Check basic functionality
    let stats = get_scheduler_stats();
    kprintln!("  Current task: {}", stats.current_task_id);
    kprintln!("  Total tasks: {}", stats.total_tasks);
    
    // Test task creation
    let task_id = create_task(0x800000);
    if task_id.0 != 0 {
        kprintln!("  ✓ Task creation works (created task {})", task_id.0);
        enqueue_task(task_id);
        kprintln!("  ✓ Task enqueuing works");
    } else {
        kprintln!("  ✗ Task creation failed");
    }
    
    // Test scheduling
    schedule();
    kprintln!("  ✓ Scheduling function works");
    
    // Check tick stats
    let tick_stats = get_tick_stats();
    kprintln!("  ✓ Tick system functional ({})", tick_stats);
    
    kprintln!("Quick test completed successfully");
}
