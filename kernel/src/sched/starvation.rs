//! Starvation Detector
//! 
//! This module monitors tasks that are ready but not scheduled for extended periods,
//! logging warnings and tracking starvation statistics.

use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use crate::sched::task_id::TaskId;
use crate::sched::task::TaskState;
use crate::log::{klog, Level, tags};
use crate::sched::{get_task, get_scheduler_stats};

/// Configuration for starvation detection
pub struct StarvationConfig {
    /// Warning threshold in milliseconds
    pub warning_threshold_ms: u64,
    /// Critical threshold in milliseconds
    pub critical_threshold_ms: u64,
    /// Whether starvation detection is enabled
    pub enabled: bool,
}

impl Default for StarvationConfig {
    fn default() -> Self {
        Self {
            warning_threshold_ms: 100,    // 100ms warning
            critical_threshold_ms: 500,   // 500ms critical
            enabled: true,
        }
    }
}

/// Starvation statistics for a single task
#[derive(Debug, Clone)]
pub struct TaskStarvationStats {
    /// Task ID
    pub task_id: TaskId,
    /// When the task became ready (timestamp in ms)
    pub ready_since_ms: u64,
    /// Total time spent ready but not scheduled (ms)
    pub total_starvation_ms: u64,
    /// Number of starvation warnings logged
    pub warning_count: u32,
    /// Number of critical starvation events
    pub critical_count: u32,
    /// Last starvation event timestamp
    pub last_starvation_ms: u64,
}

/// Global starvation detector state
pub struct StarvationDetector {
    /// Configuration
    config: StarvationConfig,
    /// Per-task starvation statistics
    task_stats: alloc::collections::BTreeMap<TaskId, TaskStarvationStats>,
    /// Global starvation counters
    total_warnings: AtomicUsize,
    total_critical: AtomicUsize,
    /// Last check timestamp
    last_check_ms: AtomicU64,
}

impl StarvationDetector {
    /// Create a new starvation detector with default configuration
    pub fn new() -> Self {
        Self {
            config: StarvationConfig::default(),
            task_stats: alloc::collections::BTreeMap::new(),
            total_warnings: AtomicUsize::new(0),
            total_critical: AtomicUsize::new(0),
            last_check_ms: AtomicU64::new(0),
        }
    }
    
    /// Create a new starvation detector with custom configuration
    pub fn with_config(config: StarvationConfig) -> Self {
        Self {
            config,
            task_stats: alloc::collections::BTreeMap::new(),
            total_warnings: AtomicUsize::new(0),
            total_critical: AtomicUsize::new(0),
            last_check_ms: AtomicU64::new(0),
        }
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: StarvationConfig) {
        self.config = config;
        klog!(Level::INFO, [tags::SCHED], 
            "Starvation detector config updated: warning={}ms, critical={}ms, enabled={}",
            self.config.warning_threshold_ms,
            self.config.critical_threshold_ms,
            self.config.enabled);
    }
    
    /// Record that a task has become ready
    pub fn record_task_ready(&mut self, task_id: TaskId, timestamp_ms: u64) {
        if !self.config.enabled {
            return;
        }
        
        let stats = TaskStarvationStats {
            task_id,
            ready_since_ms: timestamp_ms,
            total_starvation_ms: 0,
            warning_count: 0,
            critical_count: 0,
            last_starvation_ms: 0,
        };
        
        self.task_stats.insert(task_id, stats);
        
        klog!(Level::TRACE, [tags::SCHED], 
            "Task {} marked ready at {}ms", task_id.value(), timestamp_ms);
    }
    
    /// Record that a task has been scheduled
    pub fn record_task_scheduled(&mut self, task_id: TaskId, timestamp_ms: u64) {
        if !self.config.enabled {
            return;
        }
        
        if let Some(stats) = self.task_stats.get_mut(&task_id) {
            let wait_time = timestamp_ms.saturating_sub(stats.ready_since_ms);
            stats.total_starvation_ms += wait_time;
            
            if wait_time > 0 {
                klog!(Level::TRACE, [tags::SCHED], 
                    "Task {} scheduled after {}ms wait", task_id.value(), wait_time);
            }
            
            // Remove from tracking since task is now running
            self.task_stats.remove(&task_id);
        }
    }
    
    /// Check for starvation and log warnings
    pub fn check_starvation(&mut self, current_time_ms: u64) {
        if !self.config.enabled {
            return;
        }
        
        let mut warnings_logged = 0;
        let mut critical_logged = 0;
        
        // Check each ready task for starvation
        for (task_id, stats) in self.task_stats.iter_mut() {
            let wait_time = current_time_ms.saturating_sub(stats.ready_since_ms);
            
            if wait_time >= self.config.critical_threshold_ms {
                // Critical starvation - task has been waiting too long
                if stats.last_starvation_ms < current_time_ms.saturating_sub(1000) {
                    // Log critical warning (rate-limited to once per second)
                    klog!(Level::WARN, [tags::SCHED], 
                        "CRITICAL STARVATION: Task {} ready for {}ms (>{}ms threshold)",
                        task_id.value(), wait_time, self.config.critical_threshold_ms);
                    
                    stats.critical_count += 1;
                    stats.last_starvation_ms = current_time_ms;
                    critical_logged += 1;
                }
            } else if wait_time >= self.config.warning_threshold_ms {
                // Warning level starvation
                if stats.last_starvation_ms < current_time_ms.saturating_sub(500) {
                    // Log warning (rate-limited to once per 500ms)
                    klog!(Level::WARN, [tags::SCHED], 
                        "STARVATION WARNING: Task {} ready for {}ms (>{}ms threshold)",
                        task_id.value(), wait_time, self.config.warning_threshold_ms);
                    
                    stats.warning_count += 1;
                    stats.last_starvation_ms = current_time_ms;
                    warnings_logged += 1;
                }
            }
        }
        
        // Update global counters
        if warnings_logged > 0 {
            self.total_warnings.fetch_add(warnings_logged, Ordering::Relaxed);
        }
        if critical_logged > 0 {
            self.total_critical.fetch_add(critical_logged, Ordering::Relaxed);
        }
        
        // Update last check timestamp
        self.last_check_ms.store(current_time_ms, Ordering::Relaxed);
        
        if warnings_logged > 0 || critical_logged > 0 {
            klog!(Level::INFO, [tags::SCHED], 
                "Starvation check: {} warnings, {} critical events logged",
                warnings_logged, critical_logged);
        }
    }
    
    /// Get starvation statistics for a specific task
    pub fn get_task_stats(&self, task_id: TaskId) -> Option<&TaskStarvationStats> {
        self.task_stats.get(&task_id)
    }
    
    /// Get global starvation statistics
    pub fn get_global_stats(&self) -> GlobalStarvationStats {
        GlobalStarvationStats {
            total_warnings: self.total_warnings.load(Ordering::Relaxed),
            total_critical: self.total_critical.load(Ordering::Relaxed),
            currently_starving: self.task_stats.len(),
            last_check_ms: self.last_check_ms.load(Ordering::Relaxed),
            config: self.config.clone(),
        }
    }
    
    /// Get all task starvation statistics
    pub fn get_all_task_stats(&self) -> &alloc::collections::BTreeMap<TaskId, TaskStarvationStats> {
        &self.task_stats
    }
    
    /// Reset all statistics
    pub fn reset_stats(&mut self) {
        self.task_stats.clear();
        self.total_warnings.store(0, Ordering::Relaxed);
        self.total_critical.store(0, Ordering::Relaxed);
        self.last_check_ms.store(0, Ordering::Relaxed);
        
        klog!(Level::INFO, [tags::SCHED], "Starvation detector statistics reset");
    }
    
    /// Check if a specific task is currently starving
    pub fn is_task_starving(&self, task_id: TaskId) -> bool {
        self.task_stats.contains_key(&task_id)
    }
    
    /// Get the current wait time for a task
    pub fn get_task_wait_time(&self, task_id: TaskId, current_time_ms: u64) -> Option<u64> {
        self.task_stats.get(&task_id)
            .map(|stats| current_time_ms.saturating_sub(stats.ready_since_ms))
    }
}

/// Global starvation statistics
#[derive(Debug, Clone)]
pub struct GlobalStarvationStats {
    /// Total number of starvation warnings logged
    pub total_warnings: usize,
    /// Total number of critical starvation events
    pub total_critical: usize,
    /// Number of tasks currently experiencing starvation
    pub currently_starving: usize,
    /// Timestamp of last starvation check
    pub last_check_ms: u64,
    /// Current configuration
    pub config: StarvationConfig,
}

/// Global starvation detector instance
static STARVATION_DETECTOR: spin::Mutex<StarvationDetector> = spin::Mutex::new(StarvationDetector::new());

/// Initialize the starvation detector
pub fn init() {
    let mut detector = STARVATION_DETECTOR.lock();
    klog!(Level::INFO, [tags::SCHED], 
        "Initializing starvation detector with warning threshold {}ms, critical threshold {}ms",
        detector.config.warning_threshold_ms,
        detector.config.critical_threshold_ms);
}

/// Record that a task has become ready
pub fn record_task_ready(task_id: TaskId) {
    let current_time = crate::log::get_current_time_ms();
    let mut detector = STARVATION_DETECTOR.lock();
    detector.record_task_ready(task_id, current_time);
}

/// Record that a task has been scheduled
pub fn record_task_scheduled(task_id: TaskId) {
    let current_time = crate::log::get_current_time_ms();
    let mut detector = STARVATION_DETECTOR.lock();
    detector.record_task_scheduled(task_id, current_time);
}

/// Check for starvation and log warnings
pub fn check_starvation() {
    let current_time = crate::log::get_current_time_ms();
    let mut detector = STARVATION_DETECTOR.lock();
    detector.check_starvation(current_time);
}

/// Get global starvation statistics
pub fn get_starvation_stats() -> GlobalStarvationStats {
    let detector = STARVATION_DETECTOR.lock();
    detector.get_global_stats()
}

/// Get starvation statistics for a specific task
pub fn get_task_starvation_stats(task_id: TaskId) -> Option<TaskStarvationStats> {
    let detector = STARVATION_DETECTOR.lock();
    detector.get_task_stats(task_id).cloned()
}

/// Update starvation detector configuration
pub fn update_config(config: StarvationConfig) {
    let mut detector = STARVATION_DETECTOR.lock();
    detector.update_config(config);
}

/// Reset all starvation statistics
pub fn reset_starvation_stats() {
    let mut detector = STARVATION_DETECTOR.lock();
    detector.reset_stats();
}

/// Check if a task is currently starving
pub fn is_task_starving(task_id: TaskId) -> bool {
    let detector = STARVATION_DETECTOR.lock();
    detector.is_task_starving(task_id)
}

/// Get the current wait time for a task
pub fn get_task_wait_time(task_id: TaskId) -> Option<u64> {
    let current_time = crate::log::get_current_time_ms();
    let detector = STARVATION_DETECTOR.lock();
    detector.get_task_wait_time(task_id, current_time)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_starvation_detector_creation() {
        let detector = StarvationDetector::new();
        assert!(detector.config.enabled);
        assert_eq!(detector.config.warning_threshold_ms, 100);
        assert_eq!(detector.config.critical_threshold_ms, 500);
    }
    
    #[test]
    fn test_task_ready_recording() {
        let mut detector = StarvationDetector::new();
        let task_id = TaskId::new();
        let timestamp = 1000;
        
        detector.record_task_ready(task_id, timestamp);
        assert!(detector.is_task_starving(task_id));
        assert_eq!(detector.get_task_wait_time(task_id, timestamp + 100), Some(100));
    }
    
    #[test]
    fn test_task_scheduled_recording() {
        let mut detector = StarvationDetector::new();
        let task_id = TaskId::new();
        let ready_time = 1000;
        let scheduled_time = 1100;
        
        detector.record_task_ready(task_id, ready_time);
        detector.record_task_scheduled(task_id, scheduled_time);
        
        assert!(!detector.is_task_starving(task_id));
    }
    
    #[test]
    fn test_starvation_warning() {
        let mut detector = StarvationDetector::new();
        let task_id = TaskId::new();
        let ready_time = 1000;
        let check_time = 1150; // 150ms wait (above 100ms warning threshold)
        
        detector.record_task_ready(task_id, ready_time);
        detector.check_starvation(check_time);
        
        let stats = detector.get_task_stats(task_id).unwrap();
        assert_eq!(stats.warning_count, 1);
    }
    
    #[test]
    fn test_starvation_critical() {
        let mut detector = StarvationDetector::new();
        let task_id = TaskId::new();
        let ready_time = 1000;
        let check_time = 1600; // 600ms wait (above 500ms critical threshold)
        
        detector.record_task_ready(task_id, ready_time);
        detector.check_starvation(check_time);
        
        let stats = detector.get_task_stats(task_id).unwrap();
        assert_eq!(stats.critical_count, 1);
    }
}



