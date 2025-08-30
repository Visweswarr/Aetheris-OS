/// Demo Tasks for Polymera OS
/// 
/// This module implements demonstration tasks that showcase the cooperative
/// scheduler and system call functionality.

use crate::{kprintln, klog};
use crate::sched::{TaskId, create_task, enqueue_task};
use crate::sched::sys::KernelThreads;

/// Demo task 1 - Counter printing task
/// 
/// This task continuously prints a counter and yields the CPU to other tasks.
/// 
/// # Arguments
/// * `_arg` - Unused argument (required by kernel thread interface)
/// 
/// # Returns
/// Exit code (should not return in this demo)
pub fn demo_task_1(_arg: u64) -> i32 {
    kprintln!("[DEMO] Task T1 starting");
    
    let mut counter = 0u64;
    
    loop {
        counter += 1;
        
        // Print counter every iteration
        kprintln!("[T1] Counter: {}", counter);
        
        // Add some visual separation for readability
        if counter % 10 == 0 {
            kprintln!("[T1] --- Reached {} iterations ---", counter);
        }
        
        // Yield CPU to other tasks (cooperative scheduling)
        sys_yield_demo();
        
        // Add a small delay to make output readable
        for _ in 0..100000 {
            core::hint::spin_loop();
        }
        
        // Exit after a reasonable number of iterations for demo
        if counter >= 50 {
            kprintln!("[T1] Demo complete, exiting after {} iterations", counter);
            sys_exit_demo(1);
        }
    }
}

/// Demo task 2 - Counter printing task
/// 
/// This task continuously prints a counter and yields the CPU to other tasks.
/// 
/// # Arguments
/// * `_arg` - Unused argument (required by kernel thread interface)
/// 
/// # Returns
/// Exit code (should not return in this demo)
pub fn demo_task_2(_arg: u64) -> i32 {
    kprintln!("[DEMO] Task T2 starting");
    
    let mut counter = 0u64;
    
    loop {
        counter += 1;
        
        // Print counter with different formatting to distinguish from T1
        kprintln!("[T2] *** Counter: {} ***", counter);
        
        // Add some visual separation for readability
        if counter % 5 == 0 {
            kprintln!("[T2] *** Milestone: {} iterations ***", counter);
        }
        
        // Yield CPU to other tasks (cooperative scheduling)
        sys_yield_demo();
        
        // Add a slightly different delay to create scheduling variation
        for _ in 0..150000 {
            core::hint::spin_loop();
        }
        
        // Exit after a reasonable number of iterations for demo
        if counter >= 30 {
            kprintln!("[T2] Demo complete, exiting after {} iterations", counter);
            sys_exit_demo(2);
        }
    }
}

/// Demo task 3 - Monitoring task
/// 
/// This task monitors the system and prints periodic status updates.
/// 
/// # Arguments
/// * `_arg` - Unused argument
/// 
/// # Returns
/// Exit code
pub fn demo_monitor_task(_arg: u64) -> i32 {
    kprintln!("[DEMO] Monitor task starting");
    
    let mut iterations = 0u64;
    
    loop {
        iterations += 1;
        
        // Print system status every few iterations
        if iterations % 3 == 0 {
            let uptime = crate::hal::x86_64::timer::get_uptime_seconds();
            let stats = crate::sched::get_scheduler_stats();
            
            kprintln!("[MONITOR] System status check #{}", iterations);
            kprintln!("[MONITOR] Uptime: {}s, Active tasks: {}", uptime, stats.total_tasks);
        }
        
        // Yield less frequently to allow other tasks more CPU time
        if iterations % 2 == 0 {
            sys_yield_demo();
        }
        
        // Longer delay for monitoring task
        for _ in 0..300000 {
            core::hint::spin_loop();
        }
        
        // Exit after monitoring for a while
        if iterations >= 20 {
            kprintln!("[MONITOR] Monitoring complete, exiting after {} checks", iterations);
            sys_exit_demo(0);
        }
    }
}

/// Wrapper for sys_yield that adds demo logging
fn sys_yield_demo() {
    let current_task = crate::sched::get_current_task_id();
    klog!(TRACE, "[DEMO] Task {} yielding CPU", current_task);
    
    // Call the actual syscall implementation
    crate::sched::sys::sys_yield();
    
    klog!(TRACE, "[DEMO] Task {} resumed execution", current_task);
}

/// Wrapper for sys_exit that adds demo logging
fn sys_exit_demo(code: i32) -> ! {
    let current_task = crate::sched::get_current_task_id();
    kprintln!("[DEMO] Task {} exiting with code {}", current_task, code);
    
    // Call the actual syscall implementation
    crate::sched::sys::sys_exit(code);
}

/// Create and spawn demo tasks
/// 
/// This function creates the demo tasks and adds them to the scheduler.
/// It should be called during kernel initialization.
/// 
/// # Returns
/// Number of demo tasks created
pub fn spawn_demo_tasks() -> usize {
    kprintln!("[DEMO] Spawning demo tasks for cooperative scheduling demonstration");
    
    let mut task_count = 0;
    
    // Create demo task T1
    if let Some(task_id) = KernelThreads::spawn("demo_task_1", demo_task_1, 0) {
        kprintln!("[DEMO] Created demo task T1 with ID {}", task_id.0);
        task_count += 1;
    } else {
        kprintln!("[DEMO] Failed to create demo task T1");
    }
    
    // Create demo task T2
    if let Some(task_id) = KernelThreads::spawn("demo_task_2", demo_task_2, 0) {
        kprintln!("[DEMO] Created demo task T2 with ID {}", task_id.0);
        task_count += 1;
    } else {
        kprintln!("[DEMO] Failed to create demo task T2");
    }
    
    // Create monitoring task
    if let Some(task_id) = KernelThreads::spawn("demo_monitor", demo_monitor_task, 0) {
        kprintln!("[DEMO] Created monitor task with ID {}", task_id.0);
        task_count += 1;
    } else {
        kprintln!("[DEMO] Failed to create monitor task");
    }
    
    kprintln!("[DEMO] Demo task spawning complete - {} tasks created", task_count);
    kprintln!("[DEMO] Tasks will demonstrate cooperative round-robin scheduling");
    kprintln!("[DEMO] Watch for T1/T2 counter alternation and monitor status updates");
    
    task_count
}

/// Test cooperative scheduling behavior
/// 
/// This function can be called to verify that the cooperative scheduler
/// is working correctly with the demo tasks.
pub fn test_cooperative_scheduling() {
    kprintln!("");
    kprintln!("=== COOPERATIVE SCHEDULING TEST ===");
    
    // Get initial scheduler state
    let initial_stats = crate::sched::get_scheduler_stats();
    kprintln!("Initial scheduler state:");
    kprintln!("  Total tasks: {}", initial_stats.total_tasks);
    kprintln!("  Ready tasks: {}", initial_stats.ready_tasks);
    kprintln!("  Current task: {}", initial_stats.current_task_id);
    
    // Spawn demo tasks
    let demo_task_count = spawn_demo_tasks();
    
    // Get updated scheduler state
    let updated_stats = crate::sched::get_scheduler_stats();
    kprintln!("");
    kprintln!("After spawning demo tasks:");
    kprintln!("  Total tasks: {}", updated_stats.total_tasks);
    kprintln!("  Ready tasks: {}", updated_stats.ready_tasks);
    kprintln!("  Demo tasks created: {}", demo_task_count);
    
    kprintln!("");
    kprintln!("Cooperative scheduling test initiated.");
    kprintln!("Demo tasks will now run and demonstrate round-robin behavior.");
    kprintln!("=== END TEST SETUP ===");
    kprintln!("");
}

/// Enhanced demo with different task behaviors
pub fn spawn_enhanced_demo() -> usize {
    kprintln!("[DEMO] Starting enhanced cooperative scheduling demo");
    
    let mut task_count = 0;
    
    // Fast counter task
    if let Some(task_id) = KernelThreads::spawn("fast_counter", fast_counter_task, 1) {
        kprintln!("[DEMO] Created fast counter task with ID {}", task_id.0);
        task_count += 1;
    }
    
    // Slow counter task  
    if let Some(task_id) = KernelThreads::spawn("slow_counter", slow_counter_task, 2) {
        kprintln!("[DEMO] Created slow counter task with ID {}", task_id.0);
        task_count += 1;
    }
    
    // Yielding task
    if let Some(task_id) = KernelThreads::spawn("yielder", yielding_task, 3) {
        kprintln!("[DEMO] Created yielding task with ID {}", task_id.0);
        task_count += 1;
    }
    
    kprintln!("[DEMO] Enhanced demo: {} tasks with different behaviors", task_count);
    
    task_count
}

/// Fast counter task - yields frequently
fn fast_counter_task(task_num: u64) -> i32 {
    kprintln!("[FAST{}] Starting fast counter task", task_num);
    
    for i in 1..=20 {
        kprintln!("[FAST{}] Rapid count: {}", task_num, i);
        
        // Yield every iteration for maximum cooperation
        sys_yield_demo();
        
        // Short delay
        for _ in 0..50000 {
            core::hint::spin_loop();
        }
    }
    
    kprintln!("[FAST{}] Fast counter complete", task_num);
    task_num as i32
}

/// Slow counter task - yields less frequently  
fn slow_counter_task(task_num: u64) -> i32 {
    kprintln!("[SLOW{}] Starting slow counter task", task_num);
    
    for i in 1..=10 {
        kprintln!("[SLOW{}] Deliberate count: {}", task_num, i);
        
        // Yield every 2 iterations
        if i % 2 == 0 {
            sys_yield_demo();
        }
        
        // Longer delay
        for _ in 0..200000 {
            core::hint::spin_loop();
        }
    }
    
    kprintln!("[SLOW{}] Slow counter complete", task_num);
    task_num as i32
}

/// Yielding task - demonstrates cooperative behavior
fn yielding_task(task_num: u64) -> i32 {
    kprintln!("[YIELD{}] Starting yielding task", task_num);
    
    for i in 1..=15 {
        kprintln!("[YIELD{}] Cooperative iteration: {}", task_num, i);
        
        // Always yield to be maximally cooperative
        sys_yield_demo();
        
        // Variable delay based on iteration
        for _ in 0..(i * 25000) {
            core::hint::spin_loop();
        }
        
        // Print scheduler stats occasionally
        if i % 5 == 0 {
            let stats = crate::sched::get_scheduler_stats();
            kprintln!("[YIELD{}] Scheduler check: {} ready tasks", task_num, stats.ready_tasks);
        }
    }
    
    kprintln!("[YIELD{}] Yielding task complete", task_num);
    task_num as i32
}

/// Demo task behavior analysis
pub fn analyze_demo_behavior() {
    kprintln!("");
    kprintln!("=== DEMO TASK BEHAVIOR ANALYSIS ===");
    
    // Get tick statistics to analyze scheduler behavior
    let tick_stats = crate::sched::tick::get_tick_stats();
    kprintln!("Scheduler performance:");
    kprintln!("  {}", tick_stats);
    
    // Get syscall statistics
    let (total_syscalls, invalid_syscalls) = crate::syscall::get_syscall_stats();
    kprintln!("Syscall activity:");
    kprintln!("  Total syscalls: {}", total_syscalls);
    kprintln!("  Invalid syscalls: {}", invalid_syscalls);
    kprintln!("  Success rate: {:.1}%", 
              (total_syscalls - invalid_syscalls) as f32 / total_syscalls as f32 * 100.0);
    
    // System uptime
    let uptime = crate::hal::x86_64::timer::get_uptime_seconds();
    kprintln!("System uptime: {}s", uptime);
    
    kprintln!("=== END ANALYSIS ===");
    kprintln!("");
}
