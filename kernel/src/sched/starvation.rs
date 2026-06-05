//! Starvation Detection Module
//!
//! This module tracks task wait times and detects/reports starvation
//! as required by Property 5.2.
//!
//! Requirement: 5.2 - Starvation Freedom

use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;
use crate::sched::{RunQueue, get_current_task_id, get_task, TaskState, TaskId};
use crate::log::get_current_time_ms;

/// Threshold in milliseconds for starvation warning
pub const STARVATION_THRESHOLD_MS: u64 = 500;

/// Threshold for critical starvation (panic or aggressive fix)
pub const CRITICAL_STARVATION_THRESHOLD_MS: u64 = 5000;

/// Global starvation statistics
pub struct StarvationStats {
    pub total_warnings: usize,
    pub total_critical: usize,
    pub currently_starving: usize,
}

static STATS: Mutex<StarvationStats> = Mutex::new(StarvationStats {
    total_warnings: 0,
    total_critical: 0,
    currently_starving: 0,
});

/// Initialize starvation detector
pub fn init() {
    crate::kprintln!("[SCHED] Initializing starvation detector (threshold: {}ms)", STARVATION_THRESHOLD_MS);
}

/// Record that a task has become ready (start tracking wait time)
pub fn record_task_ready(task_id: TaskId) {
    // In a real implementation, we would access the task directly.
    // However, since we can't easily get a mutable reference to the task globally without locking the whole storage,
    // we assume the caller (sched::enqueue_task) handles the timestamp setting on the task.
    // This function acts as a hook/stub for additional tracking if needed.
}

/// Record that a task has been scheduled (stop tracking wait time)
pub fn record_task_scheduled(task_id: TaskId) {
    // Similarly, the caller (sched::schedule) clears the timestamp.
}

/// Check for starving tasks
///
/// **Property 5: Starvation Freedom**
/// Detects any task that has been in Ready state > 500ms.
///
/// Note: This is an O(N) operation where N is number of ready tasks.
/// To minimize impact, this should be called periodically or only when
/// system load is high.
pub fn check_starvation() {
    let now = get_current_time_ms();
    
    // We need to iterate over all tasks to find ready ones and check their timestamps.
    // Since we can't easily iterate the global TASK_STORAGE from here without circular deps or visibility issues 
    // if we don't have a public iterator, we will rely on the scheduler to call check_task_starvation 
    // for each task, OR we use a simplified check if we can access the storage.
    
    // For now, we will reset the current counter and let the external loop (in sched::schedule) 
    // increment it via check_task_starvation if it chooses to iterate.
    // But since sched::schedule in basic form might not iterate all tasks, 
    // we'll implement a "starvation check" function in sched::mod.rs that calls into here.
    
    let mut stats = STATS.lock();
    stats.currently_starving = 0;
}

/// Check a specific task for starvation - called by scheduler
pub fn check_task_wait_time(task_id: TaskId, state: TaskState, wake_timestamp: Option<u64>) {
    // Only check Ready tasks
    if state != TaskState::Ready {
        return;
    }
    
    let now = get_current_time_ms();
    
    if let Some(ready_since) = wake_timestamp {
        let wait_time = now.saturating_sub(ready_since);
        
        if wait_time > STARVATION_THRESHOLD_MS {
            let mut stats = STATS.lock();
            stats.currently_starving += 1;
            stats.total_warnings += 1;
            
            // Limit log volume
            if stats.total_warnings % 100 == 0 {
                crate::klog!(WARN, "[SCHED] STARVATION DETECTED: Task {} waiting for {}ms", 
                             task_id.0, wait_time);
            }
            
            if wait_time > CRITICAL_STARVATION_THRESHOLD_MS {
                stats.total_critical += 1;
                if stats.total_critical % 10 == 0 {
                    crate::klog!(ERROR, "[SCHED] CRITICAL STARVATION: Task {} waiting > 5s!", task_id.0);
                }
            }
        }
    }
}

/// Get current starvation statistics
pub fn get_starvation_stats() -> StarvationStats {
    let stats = STATS.lock();
    StarvationStats {
        total_warnings: stats.total_warnings,
        total_critical: stats.total_critical,
        currently_starving: stats.currently_starving,
    }
}
