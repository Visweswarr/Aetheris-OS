/// Demo Tasks Module
/// 
/// This module provides demonstration tasks for showcasing kernel functionality.

use crate::{kprintln, klog};

/// Simple demo task that prints a message
pub fn simple_demo_task(arg: u64) -> i32 {
    kprintln!("[DEMO_TASK] Running simple demo task with arg: {}", arg);
    
    for i in 0..5 {
        kprintln!("[DEMO_TASK] Iteration {}", i);
        // Yield to other tasks
        crate::sched::sys::sys_yield();
    }
    
    kprintln!("[DEMO_TASK] Demo task complete");
    0
}

/// Counter demo task
pub fn counter_demo_task(start: u64) -> i32 {
    kprintln!("[COUNTER_TASK] Starting counter from {}", start);
    
    let mut counter = start;
    for _ in 0..10 {
        counter += 1;
        kprintln!("[COUNTER_TASK] Counter: {}", counter);
        crate::sched::sys::sys_yield();
    }
    
    counter as i32
}

/// Initialize demo tasks module
pub fn init_demo_tasks() {
    kprintln!("[DEMO_TASKS] Demo tasks module initialized");
}

/// Test demo tasks
pub fn test_demo_tasks() {
    kprintln!("Testing demo tasks...");
    kprintln!("  ✓ Demo tasks module loaded");
}
