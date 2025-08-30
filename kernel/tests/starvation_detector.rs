//! Starvation Detector Tests
//! 
//! These tests verify that the starvation detector correctly identifies
//! tasks that are ready but not scheduled for extended periods.

use crate::sched::starvation::{StarvationConfig, StarvationDetector, GlobalStarvationStats};
use crate::sched::task_id::TaskId;
use crate::sched::task::TaskPriority;

/// Test basic starvation detector functionality
pub fn test_starvation_detector_basic() -> Result<(), &'static str> {
    kprintln!("Testing basic starvation detector functionality...");
    
    // Test detector creation
    let detector = StarvationDetector::new();
    assert!(detector.config.enabled, "Detector should be enabled by default");
    assert_eq!(detector.config.warning_threshold_ms, 100, "Default warning threshold should be 100ms");
    assert_eq!(detector.config.critical_threshold_ms, 500, "Default critical threshold should be 500ms");
    
    // Test custom configuration
    let custom_config = StarvationConfig {
        warning_threshold_ms: 50,
        critical_threshold_ms: 200,
        enabled: false,
    };
    let custom_detector = StarvationDetector::with_config(custom_config);
    assert!(!custom_detector.config.enabled, "Custom detector should respect disabled config");
    assert_eq!(custom_detector.config.warning_threshold_ms, 50, "Custom warning threshold should be 50ms");
    assert_eq!(custom_detector.config.critical_threshold_ms, 200, "Custom critical threshold should be 200ms");
    
    kprintln!("✅ Basic starvation detector functionality tests passed!");
    Ok(())
}

/// Test task ready and scheduled recording
pub fn test_task_recording() -> Result<(), &'static str> {
    kprintln!("Testing task ready and scheduled recording...");
    
    let mut detector = StarvationDetector::new();
    
    // Test task ready recording
    let task_id = TaskId::new();
    let timestamp = 1000;
    
    detector.record_task_ready(task_id, timestamp);
    assert!(detector.is_task_starving(task_id), "Task should be marked as starving");
    assert_eq!(detector.get_task_wait_time(task_id, timestamp + 100), Some(100), "Task should have 100ms wait time");
    
    // Test task scheduled recording
    let scheduled_time = timestamp + 150;
    detector.record_task_scheduled(task_id, scheduled_time);
    assert!(!detector.is_task_starving(task_id), "Task should no longer be starving after scheduling");
    
    kprintln!("✅ Task recording tests passed!");
    Ok(())
}

/// Test starvation warning thresholds
pub fn test_starvation_warnings() -> Result<(), &'static str> {
    kprintln!("Testing starvation warning thresholds...");
    
    let mut detector = StarvationDetector::new();
    
    // Create a task that will trigger warnings
    let task_id = TaskId::new();
    let ready_time = 1000;
    
    detector.record_task_ready(task_id, ready_time);
    
    // Check at warning threshold (100ms)
    let check_time = ready_time + 150; // 150ms wait (above 100ms warning threshold)
    detector.check_starvation(check_time);
    
    let stats = detector.get_task_stats(task_id).unwrap();
    assert_eq!(stats.warning_count, 1, "Task should have 1 warning");
    assert_eq!(stats.critical_count, 0, "Task should have 0 critical events");
    
    // Check at critical threshold (500ms)
    let critical_time = ready_time + 600; // 600ms wait (above 500ms critical threshold)
    detector.check_starvation(critical_time);
    
    let stats = detector.get_task_stats(task_id).unwrap();
    assert_eq!(stats.warning_count, 1, "Task should still have 1 warning (rate-limited)");
    assert_eq!(stats.critical_count, 1, "Task should have 1 critical event");
    
    kprintln!("✅ Starvation warning threshold tests passed!");
    Ok(())
}

/// Test rate limiting of warnings
pub fn test_warning_rate_limiting() -> Result<(), &'static str> {
    kprintln!("Testing warning rate limiting...");
    
    let mut detector = StarvationDetector::new();
    
    let task_id = TaskId::new();
    let ready_time = 1000;
    
    detector.record_task_ready(task_id, ready_time);
    
    // Check multiple times within rate limit period
    for i in 1..=5 {
        let check_time = ready_time + 150; // Always 150ms wait
        detector.check_starvation(check_time);
        
        let stats = detector.get_task_stats(task_id).unwrap();
        // Should only log once due to rate limiting (500ms between warnings)
        assert_eq!(stats.warning_count, 1, "Task should have only 1 warning due to rate limiting (check {})", i);
    }
    
    // Check after rate limit period
    let later_time = ready_time + 700; // 700ms wait, 550ms after first check
    detector.check_starvation(later_time);
    
    let stats = detector.get_task_stats(task_id).unwrap();
    assert_eq!(stats.warning_count, 2, "Task should have 2 warnings after rate limit period");
    
    kprintln!("✅ Warning rate limiting tests passed!");
    Ok(())
}

/// Test global statistics
pub fn test_global_statistics() -> Result<(), &'static str> {
    kprintln!("Testing global statistics...");
    
    let mut detector = StarvationDetector::new();
    
    // Create multiple tasks to trigger warnings
    let task1 = TaskId::new();
    let task2 = TaskId::new();
    let ready_time = 1000;
    
    detector.record_task_ready(task1, ready_time);
    detector.record_task_ready(task2, ready_time);
    
    // Check for starvation
    let check_time = ready_time + 150;
    detector.check_starvation(check_time);
    
    // Get global statistics
    let global_stats = detector.get_global_stats();
    assert_eq!(global_stats.total_warnings, 2, "Should have 2 total warnings");
    assert_eq!(global_stats.total_critical, 0, "Should have 0 critical events");
    assert_eq!(global_stats.currently_starving, 2, "Should have 2 currently starving tasks");
    
    // Schedule one task
    detector.record_task_scheduled(task1, ready_time + 200);
    
    let global_stats = detector.get_global_stats();
    assert_eq!(global_stats.currently_starving, 1, "Should have 1 currently starving task after scheduling one");
    
    kprintln!("✅ Global statistics tests passed!");
    Ok(())
}

/// Test configuration updates
pub fn test_configuration_updates() -> Result<(), &'static str> {
    kprintln!("Testing configuration updates...");
    
    let mut detector = StarvationDetector::new();
    
    // Update configuration
    let new_config = StarvationConfig {
        warning_threshold_ms: 25,
        critical_threshold_ms: 100,
        enabled: true,
    };
    
    detector.update_config(new_config);
    assert_eq!(detector.config.warning_threshold_ms, 25, "Warning threshold should be updated to 25ms");
    assert_eq!(detector.config.critical_threshold_ms, 100, "Critical threshold should be updated to 100ms");
    
    // Test with new thresholds
    let task_id = TaskId::new();
    let ready_time = 1000;
    
    detector.record_task_ready(task_id, ready_time);
    
    // Check at 50ms (above new warning threshold of 25ms)
    let check_time = ready_time + 50;
    detector.check_starvation(check_time);
    
    let stats = detector.get_task_stats(task_id).unwrap();
    assert_eq!(stats.warning_count, 1, "Task should have 1 warning with new threshold");
    
    kprintln!("✅ Configuration update tests passed!");
    Ok(())
}

/// Test statistics reset
pub fn test_statistics_reset() -> Result<(), &'static str> {
    kprintln!("Testing statistics reset...");
    
    let mut detector = StarvationDetector::new();
    
    // Create some statistics
    let task_id = TaskId::new();
    let ready_time = 1000;
    
    detector.record_task_ready(task_id, ready_time);
    detector.check_starvation(ready_time + 150);
    
    // Verify statistics exist
    let global_stats = detector.get_global_stats();
    assert!(global_stats.total_warnings > 0, "Should have some warnings before reset");
    
    // Reset statistics
    detector.reset_stats();
    
    // Verify statistics are cleared
    let global_stats = detector.get_global_stats();
    assert_eq!(global_stats.total_warnings, 0, "Warnings should be reset to 0");
    assert_eq!(global_stats.total_critical, 0, "Critical events should be reset to 0");
    assert_eq!(global_stats.currently_starving, 0, "Currently starving should be reset to 0");
    
    kprintln!("✅ Statistics reset tests passed!");
    Ok(())
}

/// Test disabled starvation detection
pub fn test_disabled_detection() -> Result<(), &'static str> {
    kprintln!("Testing disabled starvation detection...");
    
    let mut detector = StarvationDetector::new();
    
    // Disable detection
    let disabled_config = StarvationConfig {
        warning_threshold_ms: 100,
        critical_threshold_ms: 500,
        enabled: false,
    };
    detector.update_config(disabled_config);
    
    // Create a task
    let task_id = TaskId::new();
    let ready_time = 1000;
    
    detector.record_task_ready(task_id, ready_time);
    
    // Check for starvation (should not log anything)
    let check_time = ready_time + 150;
    detector.check_starvation(check_time);
    
    let stats = detector.get_task_stats(task_id).unwrap();
    assert_eq!(stats.warning_count, 0, "Task should have 0 warnings when detection is disabled");
    
    kprintln!("✅ Disabled detection tests passed!");
    Ok(())
}

/// Run all starvation detector tests
pub fn run_all_starvation_detector_tests() -> Result<(), &'static str> {
    kprintln!("🚀 Running starvation detector tests...");
    
    test_starvation_detector_basic()?;
    test_task_recording()?;
    test_starvation_warnings()?;
    test_warning_rate_limiting()?;
    test_global_statistics()?;
    test_configuration_updates()?;
    test_statistics_reset()?;
    test_disabled_detection()?;
    
    kprintln!("🎉 All starvation detector tests completed successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic() {
        assert!(test_starvation_detector_basic().is_ok());
    }
    
    #[test]
    fn test_recording() {
        assert!(test_task_recording().is_ok());
    }
    
    #[test]
    fn test_warnings() {
        assert!(test_starvation_warnings().is_ok());
    }
    
    #[test]
    fn test_rate_limiting() {
        assert!(test_warning_rate_limiting().is_ok());
    }
    
    #[test]
    fn test_global_stats() {
        assert!(test_global_statistics().is_ok());
    }
    
    #[test]
    fn test_config_updates() {
        assert!(test_configuration_updates().is_ok());
    }
    
    #[test]
    fn test_reset() {
        assert!(test_statistics_reset().is_ok());
    }
    
    #[test]
    fn test_disabled() {
        assert!(test_disabled_detection().is_ok());
    }
    
    #[test]
    fn test_all() {
        assert!(run_all_starvation_detector_tests().is_ok());
    }
}



