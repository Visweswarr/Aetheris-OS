use crate::flaky_detector::{
    FlakyDetector, FlakyDetectorConfig, IpcTestRunner, 
    detect_flaky_and_report, get_stats
};

/// Run all flaky detector tests
pub fn run_all_flaky_detector_tests() -> Result<(), String> {
    kprintln!("[TEST] Running flaky detector tests...");
    
    // Test 1: Basic flaky detection
    test_basic_flaky_detection()?;
    
    // Test 2: Custom configuration
    test_custom_configuration()?;
    
    // Test 3: IPC test runner
    test_ipc_test_runner()?;
    
    // Test 4: Issue auto-creation
    test_issue_auto_creation()?;
    
    // Test 5: Statistics collection
    test_statistics_collection()?;
    
    kprintln!("[TEST] All flaky detector tests PASSED");
    Ok(())
}

/// Test basic flaky detection functionality
fn test_basic_flaky_detection() -> Result<(), String> {
    kprintln!("[TEST] Testing basic flaky detection...");
    
    let detector = FlakyDetector::new();
    
    // Test with a stable test (should not be flaky)
    let result = detector.detect_flaky_test("stable_test", || {
        (true, "Test passed".to_string(), None)
    });
    
    if result.is_flaky {
        return Err("Stable test incorrectly marked as flaky".to_string());
    }
    
    // Test with a flaky test (should be flaky)
    let result = detector.detect_flaky_test("flaky_test", || {
        use core::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        let success = count % 5 != 0; // Fail every 5th run
        
        let logs = format!("Test run {} completed with success={}", count, success);
        let minidump = if !success { Some(vec![0xDE, 0xAD, 0xBE, 0xEF]) } else { None };
        
        (success, logs, minidump)
    });
    
    if !result.is_flaky {
        return Err("Flaky test incorrectly marked as stable".to_string());
    }
    
    // Verify statistics
    if result.stats.success_rate_percent >= 100.0 {
        return Err("Flaky test should have success rate < 100%".to_string());
    }
    
    kprintln!("[TEST] Basic flaky detection PASSED");
    Ok(())
}

/// Test custom configuration
fn test_custom_configuration() -> Result<(), String> {
    kprintln!("[TEST] Testing custom configuration...");
    
    let config = FlakyDetectorConfig {
        test_runs: 3,
        max_variance_percent: 5.0, // Very strict
        min_test_duration_ms: 50,
        max_test_duration_ms: 1000,
        auto_open_issues: false,
        issue_template: "Custom template".to_string(),
    };
    
    let detector = FlakyDetector::with_config(config);
    
    // Test with a test that has moderate variance
    let result = detector.detect_flaky_test("moderate_variance_test", || {
        use core::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        let success = true; // Always succeed
        let logs = format!("Test run {} completed", count);
        
        (success, logs, None)
    });
    
    // With strict variance threshold, this should be marked as flaky
    if !result.is_flaky {
        return Err("Test with moderate variance should be marked as flaky with strict threshold".to_string());
    }
    
    kprintln!("[TEST] Custom configuration PASSED");
    Ok(())
}

/// Test IPC test runner
fn test_ipc_test_runner() -> Result<(), String> {
    kprintln!("[TEST] Testing IPC test runner...");
    
    let runner = IpcTestRunner::new();
    
    // Test simple IPC test
    let result = runner.run_simple_ipc_test("ipc_test");
    
    // Verify result structure
    if result.test_name != "ipc_test" {
        return Err("Test name not set correctly".to_string());
    }
    
    if result.run_results.len() != 5 { // Default 5 runs
        return Err(format!("Expected 5 test runs, got {}", result.run_results.len()));
    }
    
    // Verify each run has correct run number
    for (i, run) in result.run_results.iter().enumerate() {
        if run.run_number != (i + 1) as u32 {
            return Err(format!("Run number mismatch: expected {}, got {}", i + 1, run.run_number));
        }
    }
    
    kprintln!("[TEST] IPC test runner PASSED");
    Ok(())
}

/// Test issue auto-creation
fn test_issue_auto_creation() -> Result<(), String> {
    kprintln!("[TEST] Testing issue auto-creation...");
    
    // Test with a flaky test that should trigger issue creation
    let result = detect_flaky_and_report("issue_test", || {
        use core::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        let success = count % 3 != 0; // Fail every 3rd run
        
        let logs = format!("Test run {} completed with success={}", count, success);
        let minidump = if !success { Some(vec![0xCA, 0xFE, 0xBA, 0xBE]) } else { None };
        
        (success, logs, minidump)
    });
    
    if let Some(result) = result {
        if !result.is_flaky {
            return Err("Test should be marked as flaky".to_string());
        }
        
        // Verify recommendations are generated
        if result.recommendations.is_empty() {
            return Err("Flaky test should have recommendations".to_string());
        }
        
        // Verify statistics are calculated
        if result.stats.mean_duration_ms <= 0.0 {
            return Err("Statistics should be calculated for flaky test".to_string());
        }
    } else {
        return Err("Flaky detection should return a result".to_string());
    }
    
    kprintln!("[TEST] Issue auto-creation PASSED");
    Ok(())
}

/// Test statistics collection
fn test_statistics_collection() -> Result<(), String> {
    kprintln!("[TEST] Testing statistics collection...");
    
    let detector = FlakyDetector::new();
    
    // Run a test to generate statistics
    let _result = detector.detect_flaky_test("stats_test", || {
        (true, "Test passed".to_string(), None)
    });
    
    // Get detector statistics
    let stats = detector.get_stats();
    
    if stats.total_tests == 0 {
        return Err("Total tests count should be > 0".to_string());
    }
    
    // Reset statistics
    detector.reset_stats();
    
    let stats_after_reset = detector.get_stats();
    if stats_after_reset.total_tests != 0 {
        return Err("Statistics should be reset to 0".to_string());
    }
    
    kprintln!("[TEST] Statistics collection PASSED");
    Ok(())
}

/// Test edge cases and error conditions
pub fn test_flaky_detector_edge_cases() -> Result<(), String> {
    kprintln!("[TEST] Testing flaky detector edge cases...");
    
    let detector = FlakyDetector::new();
    
    // Test with disabled detector
    detector.set_enabled(false);
    let result = detector.detect_flaky_test("disabled_test", || {
        (false, "Test failed".to_string(), None)
    });
    
    if result.is_flaky {
        return Err("Disabled detector should not mark tests as flaky".to_string());
    }
    
    if result.run_results.len() != 1 {
        return Err("Disabled detector should only run test once".to_string());
    }
    
    // Re-enable detector
    detector.set_enabled(true);
    
    // Test with empty test function
    let result = detector.detect_flaky_test("empty_test", || {
        (true, "".to_string(), None)
    });
    
    if result.test_name != "empty_test" {
        return Err("Test name should be preserved".to_string());
    }
    
    kprintln!("[TEST] Edge cases PASSED");
    Ok(())
}

/// Test performance characteristics
pub fn test_flaky_detector_performance() -> Result<(), String> {
    kprintln!("[TEST] Testing flaky detector performance...");
    
    let config = FlakyDetectorConfig {
        test_runs: 10, // More runs for performance testing
        max_variance_percent: 10.0,
        min_test_duration_ms: 10,
        max_test_duration_ms: 100,
        auto_open_issues: false,
        issue_template: String::new(),
    };
    
    let detector = FlakyDetector::with_config(config);
    
    // Measure detection time
    let start_time = crate::determinism::get_global_time_ms();
    
    let _result = detector.detect_flaky_test("perf_test", || {
        (true, "Performance test".to_string(), None)
    });
    
    let end_time = crate::determinism::get_global_time_ms();
    let detection_time = end_time.saturating_sub(start_time);
    
    // Detection should complete within reasonable time
    if detection_time > 1000 { // 1 second
        return Err(format!("Flaky detection took too long: {}ms", detection_time));
    }
    
    kprintln!("[TEST] Performance test PASSED - detection time: {}ms", detection_time);
    Ok(())
}

/// Run comprehensive flaky detector tests
pub fn run_comprehensive_flaky_detector_tests() -> Result<(), String> {
    kprintln!("[TEST] Running comprehensive flaky detector tests...");
    
    // Basic functionality tests
    run_all_flaky_detector_tests()?;
    
    // Edge case tests
    test_flaky_detector_edge_cases()?;
    
    // Performance tests
    test_flaky_detector_performance()?;
    
    kprintln!("[TEST] All comprehensive flaky detector tests PASSED");
    Ok(())
}
