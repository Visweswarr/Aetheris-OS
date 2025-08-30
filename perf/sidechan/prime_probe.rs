//! Prime+Probe Cache Side-Channel Detection Microtest
//! 
//! This module implements a very small smoke test to detect gross cache leakage
//! in CI VMs using the prime+probe technique.

use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;

/// Cache line size in bytes (64 bytes for most modern x86_64 processors)
const CACHE_LINE_SIZE: usize = 64;

/// Number of cache lines to test
const CACHE_LINES_TO_TEST: usize = 1024; // 64KB total

/// Threshold for detecting significant cache timing differences (in nanoseconds)
const TIMING_THRESHOLD_NS: u64 = 100;

/// Number of test iterations
const TEST_ITERATIONS: usize = 1000;

/// Prime+Probe test result
#[derive(Debug, Clone)]
pub struct PrimeProbeResult {
    /// Test name/identifier
    pub test_name: String,
    /// Average timing difference in nanoseconds
    pub avg_timing_diff_ns: f64,
    /// Maximum timing difference in nanoseconds
    pub max_timing_diff_ns: u64,
    /// Minimum timing difference in nanoseconds
    pub min_timing_diff_ns: u64,
    /// Standard deviation of timing differences
    pub std_dev_ns: f64,
    /// Number of samples above threshold
    pub samples_above_threshold: usize,
    /// Test passed flag
    pub passed: bool,
    /// Detailed timing data
    pub timing_samples: Vec<u64>,
}

/// Prime+Probe test runner
pub struct PrimeProbeTester {
    /// Test buffer aligned to cache line boundaries
    test_buffer: Vec<u8>,
    /// Timing measurements
    timing_measurements: Vec<u64>,
    /// Test configuration
    config: TestConfig,
}

/// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Cache line size in bytes
    pub cache_line_size: usize,
    /// Number of cache lines to test
    pub cache_lines_to_test: usize,
    /// Timing threshold in nanoseconds
    pub timing_threshold_ns: u64,
    /// Number of test iterations
    pub test_iterations: usize,
    /// Enable verbose output
    pub verbose: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            cache_line_size: CACHE_LINE_SIZE,
            cache_lines_to_test: CACHE_LINES_TO_TEST,
            timing_threshold_ns: TIMING_THRESHOLD_NS,
            test_iterations: TEST_ITERATIONS,
            verbose: false,
        }
    }
}

impl PrimeProbeTester {
    /// Create new prime+probe tester
    pub fn new(config: TestConfig) -> Self {
        // Allocate buffer aligned to cache line boundaries
        let buffer_size = config.cache_lines_to_test * config.cache_line_size;
        let mut test_buffer = vec![0u8; buffer_size];
        
        // Ensure buffer is aligned to cache line boundary
        let ptr = test_buffer.as_mut_ptr() as usize;
        let alignment = ptr % config.cache_line_size;
        if alignment != 0 {
            let padding = config.cache_line_size - alignment;
            test_buffer = vec![0u8; buffer_size + padding];
        }
        
        Self {
            test_buffer,
            timing_measurements: Vec::new(),
            config,
        }
    }
    
    /// Run prime+probe test
    pub fn run_test(&mut self, test_name: &str) -> PrimeProbeResult {
        if self.config.verbose {
            println!("Running prime+probe test: {}", test_name);
        }
        
        self.timing_measurements.clear();
        
        // Run multiple iterations to get statistical significance
        for iteration in 0..self.config.test_iterations {
            if self.config.verbose && iteration % 100 == 0 {
                println!("  Iteration {}/{}", iteration + 1, self.config.test_iterations);
            }
            
            let timing = self.run_single_iteration();
            self.timing_measurements.push(timing);
        }
        
        // Analyze results
        let result = self.analyze_results(test_name);
        
        if self.config.verbose {
            println!("  Result: {}", if result.passed { "PASSED" } else { "FAILED" });
            println!("  Avg timing diff: {:.2} ns", result.avg_timing_diff_ns);
            println!("  Samples above threshold: {}/{}", 
                     result.samples_above_threshold, self.config.test_iterations);
        }
        
        result
    }
    
    /// Run a single prime+probe iteration
    fn run_single_iteration(&self) -> u64 {
        // Prime: Fill cache with our test data
        self.prime_cache();
        
        // Small delay to simulate some work
        std::thread::sleep(Duration::from_nanos(100));
        
        // Probe: Measure time to access cache lines
        let start_time = Instant::now();
        self.probe_cache();
        let end_time = Instant::now();
        
        let probe_time = end_time.duration_since(start_time).as_nanos() as u64;
        
        // Measure baseline access time
        let baseline_start = Instant::now();
        self.baseline_access();
        let baseline_end = Instant::now();
        
        let baseline_time = baseline_end.duration_since(baseline_start).as_nanos() as u64;
        
        // Return timing difference
        if probe_time > baseline_time {
            probe_time - baseline_time
        } else {
            0
        }
    }
    
    /// Prime the cache by accessing all test cache lines
    fn prime_cache(&self) {
        for i in 0..self.config.cache_lines_to_test {
            let offset = i * self.config.cache_line_size;
            // Touch each cache line to bring it into cache
            let _value = self.test_buffer[offset];
            
            // Prevent compiler optimization
            std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::Acquire);
        }
    }
    
    /// Probe the cache by measuring access time
    fn probe_cache(&self) {
        for i in 0..self.config.cache_lines_to_test {
            let offset = i * self.config.cache_line_size;
            // Access each cache line and measure timing
            let _value = self.test_buffer[offset];
            
            // Prevent compiler optimization
            std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::Acquire);
        }
    }
    
    /// Baseline access for comparison
    fn baseline_access(&self) {
        // Access a small subset for baseline measurement
        for i in 0..16 {
            let offset = i * self.config.cache_line_size;
            let _value = self.test_buffer[offset];
            std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::Acquire);
        }
    }
    
    /// Analyze test results
    fn analyze_results(&self, test_name: &str) -> PrimeProbeResult {
        if self.timing_measurements.is_empty() {
            return PrimeProbeResult {
                test_name: test_name.to_string(),
                avg_timing_diff_ns: 0.0,
                max_timing_diff_ns: 0,
                min_timing_diff_ns: 0,
                std_dev_ns: 0.0,
                samples_above_threshold: 0,
                passed: true,
                timing_samples: vec![],
            };
        }
        
        // Calculate statistics
        let sum: u64 = self.timing_measurements.iter().sum();
        let avg = sum as f64 / self.timing_measurements.len() as f64;
        
        let max = *self.timing_measurements.iter().max().unwrap_or(&0);
        let min = *self.timing_measurements.iter().min().unwrap_or(&0);
        
        // Calculate standard deviation
        let variance = self.timing_measurements.iter()
            .map(|&x| {
                let diff = x as f64 - avg;
                diff * diff
            })
            .sum::<f64>() / self.timing_measurements.len() as f64;
        let std_dev = variance.sqrt();
        
        // Count samples above threshold
        let samples_above_threshold = self.timing_measurements.iter()
            .filter(|&&x| x > self.config.timing_threshold_ns)
            .count();
        
        // Determine if test passed
        let passed = samples_above_threshold < self.config.test_iterations / 10; // Less than 10% above threshold
        
        PrimeProbeResult {
            test_name: test_name.to_string(),
            avg_timing_diff_ns: avg,
            max_timing_diff_ns: max,
            min_timing_diff_ns: min,
            std_dev_ns: std_dev,
            samples_above_threshold,
            passed,
            timing_samples: self.timing_measurements.clone(),
        }
    }
    
    /// Run comprehensive test suite
    pub fn run_test_suite(&mut self) -> Vec<PrimeProbeResult> {
        let mut results = Vec::new();
        
        // Test 1: Basic prime+probe
        results.push(self.run_test("basic_prime_probe"));
        
        // Test 2: Eviction-based test
        results.push(self.run_eviction_test());
        
        // Test 3: Timing attack simulation
        results.push(self.run_timing_attack_simulation());
        
        results
    }
    
    /// Run eviction-based cache test
    fn run_eviction_test(&mut self) -> PrimeProbeResult {
        if self.config.verbose {
            println!("Running eviction-based cache test");
        }
        
        self.timing_measurements.clear();
        
        for iteration in 0..self.config.test_iterations {
            if self.config.verbose && iteration % 100 == 0 {
                println!("  Eviction iteration {}/{}", iteration + 1, self.config.test_iterations);
            }
            
            // Prime cache
            self.prime_cache();
            
            // Evict by accessing different memory
            self.evict_cache();
            
            // Probe and measure
            let timing = self.measure_eviction_timing();
            self.timing_measurements.push(timing);
        }
        
        self.analyze_results("eviction_based_test")
    }
    
    /// Evict cache by accessing different memory regions
    fn evict_cache(&self) {
        // Create a large buffer to evict our test data
        let eviction_size = self.config.cache_lines_to_test * self.config.cache_line_size * 2;
        let mut eviction_buffer = vec![0u8; eviction_size];
        
        // Access eviction buffer to fill cache
        for i in 0..eviction_buffer.len() {
            eviction_buffer[i] = i as u8;
            std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::Acquire);
        }
    }
    
    /// Measure timing after eviction
    fn measure_eviction_timing(&self) -> u64 {
        let start_time = Instant::now();
        self.probe_cache();
        let end_time = Instant::now();
        
        end_time.duration_since(start_time).as_nanos() as u64
    }
    
    /// Run timing attack simulation
    fn run_timing_attack_simulation(&mut self) -> PrimeProbeResult {
        if self.config.verbose {
            println!("Running timing attack simulation");
        }
        
        self.timing_measurements.clear();
        
        for iteration in 0..self.config.test_iterations {
            if self.config.verbose && iteration % 100 == 0 {
                println!("  Timing attack iteration {}/{}", iteration + 1, self.config.test_iterations);
            }
            
            // Simulate secret-dependent access pattern
            let secret_value = iteration % 256;
            let timing = self.simulate_secret_access(secret_value);
            self.timing_measurements.push(timing);
        }
        
        self.analyze_results("timing_attack_simulation")
    }
    
    /// Simulate secret-dependent memory access
    fn simulate_secret_access(&self, secret_value: u8) -> u64 {
        // Create a pattern based on secret value
        let access_pattern = (secret_value as usize) % self.config.cache_lines_to_test;
        
        let start_time = Instant::now();
        
        // Access specific cache line based on secret
        let offset = access_pattern * self.config.cache_line_size;
        let _value = self.test_buffer[offset];
        
        std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::Acquire);
        
        let end_time = Instant::now();
        end_time.duration_since(start_time).as_nanos() as u64
    }
}

/// Run all side-channel tests
pub fn run_all_tests(verbose: bool) -> Vec<PrimeProbeResult> {
    let config = TestConfig {
        verbose,
        ..Default::default()
    };
    
    let mut tester = PrimeProbeTester::new(config);
    tester.run_test_suite()
}

/// Generate test report
pub fn generate_report(results: &[PrimeProbeResult]) -> String {
    let mut report = String::new();
    report.push_str("=== Side-Channel Detection Test Report ===\n\n");
    
    let total_tests = results.len();
    let passed_tests = results.iter().filter(|r| r.passed).count();
    let failed_tests = total_tests - passed_tests;
    
    report.push_str(&format!("Total Tests: {}\n", total_tests));
    report.push_str(&format!("Passed: {}\n", passed_tests));
    report.push_str(&format!("Failed: {}\n", failed_tests));
    report.push_str(&format!("Success Rate: {:.1}%\n\n", 
                            (passed_tests as f64 / total_tests as f64) * 100.0));
    
    for result in results {
        report.push_str(&format!("Test: {}\n", result.test_name));
        report.push_str(&format!("  Status: {}\n", if result.passed { "PASSED" } else { "FAILED" }));
        report.push_str(&format!("  Avg Timing Diff: {:.2} ns\n", result.avg_timing_diff_ns));
        report.push_str(&format!("  Max Timing Diff: {} ns\n", result.max_timing_diff_ns));
        report.push_str(&format!("  Std Dev: {:.2} ns\n", result.std_dev_ns));
        report.push_str(&format!("  Samples Above Threshold: {}/{}\n", 
                                result.samples_above_threshold, result.timing_samples.len()));
        report.push_str("\n");
    }
    
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prime_probe_tester_creation() {
        let config = TestConfig::default();
        let tester = PrimeProbeTester::new(config);
        assert_eq!(tester.test_buffer.len(), CACHE_LINES_TO_TEST * CACHE_LINE_SIZE);
    }
    
    #[test]
    fn test_single_iteration() {
        let config = TestConfig {
            test_iterations: 1,
            verbose: false,
            ..Default::default()
        };
        let mut tester = PrimeProbeTester::new(config);
        let result = tester.run_test("test");
        assert!(!result.timing_samples.is_empty());
    }
    
    #[test]
    fn test_result_analysis() {
        let config = TestConfig::default();
        let tester = PrimeProbeTester::new(config);
        
        // Create dummy timing data
        let mut dummy_tester = PrimeProbeTester::new(config);
        dummy_tester.timing_measurements = vec![100, 200, 300, 400, 500];
        
        let result = dummy_tester.analyze_results("dummy_test");
        assert_eq!(result.avg_timing_diff_ns, 300.0);
        assert_eq!(result.max_timing_diff_ns, 500);
        assert_eq!(result.min_timing_diff_ns, 100);
    }
    
    #[test]
    fn test_report_generation() {
        let results = vec![
            PrimeProbeResult {
                test_name: "test1".to_string(),
                avg_timing_diff_ns: 100.0,
                max_timing_diff_ns: 200,
                min_timing_diff_ns: 50,
                std_dev_ns: 25.0,
                samples_above_threshold: 5,
                passed: true,
                timing_samples: vec![100, 150, 200],
            },
            PrimeProbeResult {
                test_name: "test2".to_string(),
                avg_timing_diff_ns: 300.0,
                max_timing_diff_ns: 500,
                min_timing_diff_ns: 100,
                std_dev_ns: 100.0,
                samples_above_threshold: 20,
                passed: false,
                timing_samples: vec![300, 400, 500],
            },
        ];
        
        let report = generate_report(&results);
        assert!(report.contains("Total Tests: 2"));
        assert!(report.contains("Passed: 1"));
        assert!(report.contains("Failed: 1"));
        assert!(report.contains("test1"));
        assert!(report.contains("test2"));
    }
}

/// Main function for standalone execution
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let verbose = args.len() > 1 && args[1] == "--verbose";
    
    println!("Running Side-Channel Detection Tests...");
    println!("This is a smoke test to detect gross cache leakage in CI VMs.\n");
    
    let results = run_all_tests(verbose);
    let report = generate_report(&results);
    
    println!("{}", report);
    
    // Exit with appropriate code
    let all_passed = results.iter().all(|r| r.passed);
    if all_passed {
        println!("✅ All tests PASSED - No significant cache leakage detected");
        std::process::exit(0);
    } else {
        println!("❌ Some tests FAILED - Potential cache leakage detected");
        std::process::exit(1);
    }
}

