use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::vec;
use alloc::format;

pub mod issue_opener;

/// Configuration for flaky test detection
#[derive(Debug, Clone)]
pub struct FlakyDetectorConfig {
    /// Number of test runs to perform
    pub test_runs: u32,
    /// Maximum allowed variance (as percentage)
    pub max_variance_percent: f64,
    /// Minimum test duration in milliseconds
    pub min_test_duration_ms: u64,
    /// Maximum test duration in milliseconds
    pub max_test_duration_ms: u64,
    /// Whether to auto-open issues for flaky tests
    pub auto_open_issues: bool,
    /// Issue template for flaky test reports
    pub issue_template: String,
}

impl Default for FlakyDetectorConfig {
    fn default() -> Self {
        Self {
            test_runs: 5,
            max_variance_percent: 15.0,
            min_test_duration_ms: 100,
            max_test_duration_ms: 5000,
            auto_open_issues: true,
            issue_template: String::from("## Flaky Test Detected\n\n**Test**: {test_name}\n**Variance**: {variance}%\n**Runs**: {runs}\n**Threshold**: {threshold}%\n\n### Test Results\n{results}\n\n### Logs\n```\n{logs}\n```\n\n### Minidump (if available)\n{minidump_info}\n\n### Recommendations\n- Investigate timing dependencies\n- Check for race conditions\n- Verify resource cleanup\n- Review interrupt handling"),
        }
    }
}

/// Result of a single test run
#[derive(Debug, Clone)]
pub struct TestRunResult {
    /// Test run number
    pub run_number: u32,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Logs from this run
    pub logs: String,
    /// Minidump data if available
    pub minidump: Option<Vec<u8>>,
}

/// Flaky test detection result
#[derive(Debug, Clone)]
pub struct FlakyTestResult {
    /// Test name
    pub test_name: String,
    /// Whether the test is flaky
    pub is_flaky: bool,
    /// Calculated variance percentage
    pub variance_percent: f64,
    /// All test run results
    pub run_results: Vec<TestRunResult>,
    /// Summary statistics
    pub stats: TestStats,
    /// Recommendations for fixing flaky behavior
    pub recommendations: Vec<String>,
}

/// Test statistics
#[derive(Debug, Clone)]
pub struct TestStats {
    /// Mean duration across all runs
    pub mean_duration_ms: f64,
    /// Standard deviation of duration
    pub std_dev_ms: f64,
    /// Coefficient of variation (std_dev / mean)
    pub coefficient_of_variation: f64,
    /// Success rate percentage
    pub success_rate_percent: f64,
    /// Minimum duration
    pub min_duration_ms: u64,
    /// Maximum duration
    pub max_duration_ms: u64,
}

/// Flaky test detector
pub struct FlakyDetector {
    config: FlakyDetectorConfig,
    enabled: AtomicBool,
    total_tests: AtomicU64,
    flaky_tests: AtomicU64,
}

impl FlakyDetector {
    /// Create a new flaky detector with default configuration
    pub fn new() -> Self {
        Self {
            config: FlakyDetectorConfig::default(),
            enabled: AtomicBool::new(true),
            total_tests: AtomicU64::new(0),
            flaky_tests: AtomicU64::new(0),
        }
    }

    /// Create a new flaky detector with custom configuration
    pub fn with_config(config: FlakyDetectorConfig) -> Self {
        Self {
            config,
            enabled: AtomicBool::new(true),
            total_tests: AtomicU64::new(0),
            flaky_tests: AtomicU64::new(0),
        }
    }

    /// Enable or disable flaky detection
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    /// Check if flaky detection is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Run a test multiple times to detect flaky behavior
    pub fn detect_flaky_test<F>(&self, test_name: &str, test_fn: F) -> FlakyTestResult
    where
        F: Fn() -> (bool, String, Option<Vec<u8>>),
    {
        if !self.is_enabled() {
            // Return a single run result if detection is disabled
            let (success, logs, minidump) = test_fn();
            let result = TestRunResult {
                run_number: 1,
                duration_ms: 0,
                success,
                error: if success { None } else { Some("Test failed".to_string()) },
                logs,
                minidump,
            };
            
            return FlakyTestResult {
                test_name: test_name.to_string(),
                is_flaky: false,
                variance_percent: 0.0,
                run_results: vec![result],
                stats: TestStats {
                    mean_duration_ms: 0.0,
                    std_dev_ms: 0.0,
                    coefficient_of_variation: 0.0,
                    success_rate_percent: if success { 100.0 } else { 0.0 },
                    min_duration_ms: 0,
                    max_duration_ms: 0,
                },
                recommendations: vec![],
            };
        }

        self.total_tests.fetch_add(1, Ordering::Relaxed);

        let mut run_results = Vec::new();
        let mut durations = Vec::new();
        let mut success_count = 0;

        // Run the test multiple times
        for run in 1..=self.config.test_runs {
            let start_time = crate::determinism::get_global_time_ms();
            
            let (success, logs, minidump) = test_fn();
            
            let end_time = crate::determinism::get_global_time_ms();
            let duration = end_time.saturating_sub(start_time);

            if success {
                success_count += 1;
                durations.push(duration);
            }

            let result = TestRunResult {
                run_number: run,
                duration_ms: duration,
                success,
                error: if success { None } else { Some("Test failed".to_string()) },
                logs,
                minidump,
            };

            run_results.push(result);

            // Check if test duration is within bounds
            if duration < self.config.min_test_duration_ms || duration > self.config.max_test_duration_ms {
                // Test duration is out of bounds, mark as potentially flaky
                let recommendations = vec![
                    "Test duration is outside expected bounds".to_string(),
                    format!("Expected: {}ms - {}ms, Got: {}ms", 
                           self.config.min_test_duration_ms, 
                           self.config.max_test_duration_ms, 
                           duration),
                    "Check for timing dependencies or resource contention".to_string(),
                ];

                return FlakyTestResult {
                    test_name: test_name.to_string(),
                    is_flaky: true,
                    variance_percent: 100.0, // Mark as flaky due to duration issues
                    run_results,
                    stats: self.calculate_stats(&durations, success_count, self.config.test_runs),
                    recommendations,
                };
            }
        }

        // Calculate variance from successful runs
        let stats = self.calculate_stats(&durations, success_count, self.config.test_runs);
        let is_flaky = self.is_test_flaky(&stats, success_count);

        if is_flaky {
            self.flaky_tests.fetch_add(1, Ordering::Relaxed);
        }

        let recommendations = if is_flaky {
            self.generate_recommendations(&stats, success_count)
        } else {
            vec![]
        };

        FlakyTestResult {
            test_name: test_name.to_string(),
            is_flaky,
            variance_percent: stats.coefficient_of_variation * 100.0,
            run_results,
            stats,
            recommendations,
        }
    }

    /// Check if a test should be considered flaky
    fn is_test_flaky(&self, stats: &TestStats, success_count: u32) -> bool {
        let success_rate = (success_count as f64 / self.config.test_runs as f64) * 100.0;
        
        // Test is flaky if:
        // 1. Success rate is below 100%
        // 2. Coefficient of variation exceeds threshold
        // 3. Standard deviation is significant
        success_rate < 100.0 || 
        stats.coefficient_of_variation * 100.0 > self.config.max_variance_percent ||
        stats.std_dev_ms > (stats.mean_duration_ms * 0.1) // 10% of mean
    }

    /// Calculate test statistics
    fn calculate_stats(&self, durations: &[u64], success_count: u32, total_runs: u32) -> TestStats {
        if durations.is_empty() {
            return TestStats {
                mean_duration_ms: 0.0,
                std_dev_ms: 0.0,
                coefficient_of_variation: 0.0,
                success_rate_percent: 0.0,
                min_duration_ms: 0,
                max_duration_ms: 0,
            };
        }

        let mean = durations.iter().map(|&x| x as f64).sum::<f64>() / durations.len() as f64;
        let variance = durations.iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>() / durations.len() as f64;
        let std_dev = if variance > 0.0 {
            let mut x = variance;
            for _ in 0..20 {
                x = 0.5 * (x + variance / x);
            }
            x
        } else {
            0.0
        };
        let coefficient_of_variation = if mean > 0.0 { std_dev / mean } else { 0.0 };
        let min_duration = *durations.iter().min().unwrap_or(&0);
        let max_duration = *durations.iter().max().unwrap_or(&0);
        let success_rate = (success_count as f64 / total_runs as f64) * 100.0;

        TestStats {
            mean_duration_ms: mean,
            std_dev_ms: std_dev,
            coefficient_of_variation,
            success_rate_percent: success_rate,
            min_duration_ms: min_duration,
            max_duration_ms: max_duration,
        }
    }

    /// Generate recommendations for fixing flaky behavior
    fn generate_recommendations(&self, stats: &TestStats, success_count: u32) -> Vec<String> {
        let mut recommendations = Vec::new();

        if success_count < self.config.test_runs {
            recommendations.push("Test has intermittent failures".to_string());
            recommendations.push("Check for race conditions or timing dependencies".to_string());
            recommendations.push("Verify resource cleanup and initialization".to_string());
        }

        if stats.coefficient_of_variation * 100.0 > self.config.max_variance_percent {
            recommendations.push("Test has high timing variance".to_string());
            recommendations.push("Check for external dependencies or resource contention".to_string());
            recommendations.push("Consider adding timeouts or retry logic".to_string());
        }

        if stats.std_dev_ms > (stats.mean_duration_ms * 0.1) {
            recommendations.push("Test has inconsistent performance".to_string());
            recommendations.push("Investigate performance bottlenecks".to_string());
            recommendations.push("Check for memory leaks or resource exhaustion".to_string());
        }

        recommendations
    }

    /// Get detector statistics
    pub fn get_stats(&self) -> DetectorStats {
        DetectorStats {
            total_tests: self.total_tests.load(Ordering::Relaxed),
            flaky_tests: self.flaky_tests.load(Ordering::Relaxed),
            enabled: self.is_enabled(),
        }
    }

    /// Reset detector statistics
    pub fn reset_stats(&self) {
        self.total_tests.store(0, Ordering::Relaxed);
        self.flaky_tests.store(0, Ordering::Relaxed);
    }

    /// Update detector configuration
    pub fn update_config(&mut self, config: FlakyDetectorConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &FlakyDetectorConfig {
        &self.config
    }
}

/// Detector statistics
#[derive(Debug, Clone)]
pub struct DetectorStats {
    pub total_tests: u64,
    pub flaky_tests: u64,
    pub enabled: bool,
}

/// IPC test runner for flaky detection
pub struct IpcTestRunner {
    detector: FlakyDetector,
}

impl IpcTestRunner {
    /// Create a new IPC test runner
    pub fn new() -> Self {
        Self {
            detector: FlakyDetector::new(),
        }
    }

    /// Create a new IPC test runner with custom configuration
    pub fn with_config(config: FlakyDetectorConfig) -> Self {
        Self {
            detector: FlakyDetector::with_config(config),
        }
    }

    /// Run a simple IPC test to detect flaky behavior
    pub fn run_simple_ipc_test(&self, test_name: &str) -> FlakyTestResult {
        self.detector.detect_flaky_test(test_name, || {
            // Simple IPC test: send and receive a message
            let start_time = crate::determinism::get_global_time_ms();
            
            // Simulate IPC operation
            let success = self.run_single_ipc_test();
            let logs = self.collect_test_logs();
            let minidump = if !success { self.collect_minidump() } else { None };
            
            (success, logs, minidump)
        })
    }

    /// Run a single IPC test iteration
    fn run_single_ipc_test(&self) -> bool {
        // This would be replaced with actual IPC test logic
        // For now, simulate a test that might be flaky
        use core::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        
        // Simulate occasional failures (every 7th run)
        if count % 7 == 0 {
            false
        } else {
            true
        }
    }

    /// Collect test logs
    fn collect_test_logs(&self) -> String {
        // This would collect actual test logs
        // For now, return a placeholder
        "Test execution completed\n".to_string()
    }

    /// Collect minidump if test failed
    fn collect_minidump(&self) -> Option<Vec<u8>> {
        // This would collect actual minidump data
        // For now, return None
        None
    }

    /// Get the underlying flaky detector
    pub fn detector(&self) -> &FlakyDetector {
        &self.detector
    }
}

/// Global flaky detector instance
static GLOBAL_DETECTOR: spin::Mutex<Option<FlakyDetector>> = spin::Mutex::new(None);

/// Initialize the global flaky detector
pub fn init() {
    let mut detector = GLOBAL_DETECTOR.lock();
    *detector = Some(FlakyDetector::new());
    
    // Initialize issue opener
    issue_opener::init();
}

/// Get the global flaky detector
pub fn get_detector() -> Option<spin::MutexGuard<'static, Option<FlakyDetector>>> {
    GLOBAL_DETECTOR.try_lock()
}

/// Run flaky detection on a test
pub fn detect_flaky_test<F>(test_name: &str, test_fn: F) -> Option<FlakyTestResult>
where
    F: Fn() -> (bool, String, Option<Vec<u8>>),
{
    if let Some(detector) = get_detector() {
        if let Some(ref detector) = *detector {
            Some(detector.detect_flaky_test(test_name, test_fn))
        } else {
            None
        }
    } else {
        None
    }
}

/// Get flaky detector statistics
pub fn get_stats() -> Option<DetectorStats> {
    if let Some(detector) = get_detector() {
        if let Some(ref detector) = *detector {
            Some(detector.get_stats())
        } else {
            None
        }
    } else {
        None
    }
}

/// Detect flaky test and automatically create issue if flaky
pub fn detect_flaky_and_report<F>(test_name: &str, test_fn: F) -> Option<FlakyTestResult>
where
    F: Fn() -> (bool, String, Option<Vec<u8>>),
{
    if let Some(result) = detect_flaky_test(test_name, test_fn) {
        // If test is flaky, automatically create an issue
        if result.is_flaky {
            match issue_opener::create_flaky_test_issue(&result) {
                Ok(issue_url) => {
                    crate::kprintln!("Flaky test detected! Issue created: {}", issue_url);
                }
                Err(e) => {
                    crate::kprintln!("Failed to create issue for flaky test: {}", e);
                }
            }
        }
        Some(result)
    } else {
        None
    }
}
