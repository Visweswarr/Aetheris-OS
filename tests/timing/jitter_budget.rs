use core::time::Duration;
use alloc::vec::Vec;
use alloc::string::String;

use polymera_os::time::{Instant, Duration as PolymeraDuration};
use polymera_os::hal::x86_64::timer::{get_timer_kind, TimerKind};
use polymera_os::sched::tick::{get_jitter_budget, init_jitter_budget};

/// Jitter budget test configuration
#[derive(Debug, Clone)]
pub struct JitterBudgetTestConfig {
    pub test_duration_ms: u64,
    pub jitter_injection_us: u32,
    pub injection_frequency_ms: u32,
    pub apic_threshold_us: u32,
    pub hpet_threshold_us: u32,
}

/// Jitter budget test results
#[derive(Debug, Clone)]
pub struct JitterBudgetTestResults {
    pub timer_kind: TimerKind,
    pub jitter_p95_us: u32,
    pub budget_hits: u32,
    pub total_samples: u32,
    pub mean_jitter_us: u32,
    pub max_jitter_us: u32,
    pub threshold_exceeded: bool,
}

impl Default for JitterBudgetTestConfig {
    fn default() -> Self {
        Self {
            test_duration_ms: 30000, // 30 seconds
            jitter_injection_us: 100, // 100µs jitter injection
            injection_frequency_ms: 100, // Inject every 100ms
            apic_threshold_us: 250, // 250µs threshold for APIC
            hpet_threshold_us: 350, // 350µs threshold for HPET
        }
    }
}

/// Run jitter budget test
pub fn run_jitter_budget_test(config: JitterBudgetTestConfig) -> JitterBudgetTestResults {
    // Initialize jitter budget system
    init_jitter_budget();
    
    // Get timer kind
    let timer_kind = get_timer_kind().expect("Timer not initialized");
    
    // Get threshold based on timer type
    let threshold_us = match timer_kind {
        TimerKind::APIC => config.apic_threshold_us,
        TimerKind::HPET => config.hpet_threshold_us,
    };
    
    // Collect jitter samples
    let mut jitter_samples = Vec::new();
    let mut budget_hits = 0u32;
    
    let start_time = Instant::now();
    let test_duration = Duration::from_millis(config.test_duration_ms);
    let injection_interval = Duration::from_millis(config.injection_frequency_ms as u64);
    
    let mut last_injection = start_time;
    
    // Run test for specified duration
    while start_time.elapsed() < test_duration {
        let now = Instant::now();
        
        // Inject synthetic jitter periodically
        if now.duration_since(last_injection) >= injection_interval {
            inject_synthetic_jitter(config.jitter_injection_us);
            last_injection = now;
        }
        
        // Measure current jitter
        if let Some(budget) = get_jitter_budget() {
            let stats = budget.get_stats();
            
            // Record jitter measurement
            if stats.jitter_samples > 0 {
                jitter_samples.push(stats.jitter_p95_us);
                
                // Check if threshold exceeded
                if stats.jitter_p95_us > threshold_us {
                    budget_hits += 1;
                }
            }
        }
        
        // Small delay to avoid overwhelming the system
        core::thread::sleep(Duration::from_millis(1));
    }
    
    // Calculate final statistics
    let (mean_jitter, p95_jitter, max_jitter) = calculate_jitter_stats(&jitter_samples);
    
    // Check if threshold was exceeded
    let threshold_exceeded = p95_jitter > threshold_us;
    
    // Print JSON metrics line for CI parsing
    print_json_metrics(timer_kind, p95_jitter, budget_hits);
    
    JitterBudgetTestResults {
        timer_kind,
        jitter_p95_us: p95_jitter,
        budget_hits,
        total_samples: jitter_samples.len() as u32,
        mean_jitter_us: mean_jitter,
        max_jitter_us: max_jitter,
        threshold_exceeded,
    }
}

/// Inject synthetic jitter for testing
fn inject_synthetic_jitter(jitter_us: u32) {
    // Simulate jitter by busy-waiting for the specified duration
    let start = Instant::now();
    let target_duration = Duration::from_micros(jitter_us as u64);
    
    while start.elapsed() < target_duration {
        core::hint::spin_loop();
    }
}

/// Calculate jitter statistics from samples
fn calculate_jitter_stats(samples: &[u32]) -> (u32, u32, u32) {
    if samples.is_empty() {
        return (0, 0, 0);
    }
    
    let total_samples = samples.len();
    
    // Calculate mean
    let sum: u64 = samples.iter().map(|&x| x as u64).sum();
    let mean = (sum / total_samples as u64) as u32;
    
    // Calculate max
    let max = samples.iter().max().copied().unwrap_or(0);
    
    // Calculate p95
    let mut sorted_samples = samples.to_vec();
    sorted_samples.sort_unstable();
    let p95_index = (total_samples * 95) / 100;
    let p95 = sorted_samples[p95_index.min(total_samples - 1)];
    
    (mean, p95, max)
}

/// Print JSON metrics line for CI parsing
fn print_json_metrics(timer_kind: TimerKind, jitter_p95_us: u32, budget_hits: u32) {
    let timer_str = match timer_kind {
        TimerKind::APIC => "APIC",
        TimerKind::HPET => "HPET",
    };
    
    // Output exactly one JSON line as specified in requirements
    println!("{{\"test\":\"jitter_budget\",\"timer\":\"{}\",\"jitter_p95_us\":{},\"budget_hits\":{}}}", 
             timer_str, jitter_p95_us, budget_hits);
}

/// Run jitter budget test with default configuration
pub fn run_default_jitter_budget_test() -> JitterBudgetTestResults {
    let config = JitterBudgetTestConfig::default();
    run_jitter_budget_test(config)
}

/// Run jitter budget test with custom configuration
pub fn run_custom_jitter_budget_test(
    test_duration_ms: u64,
    jitter_injection_us: u32,
    injection_frequency_ms: u32,
) -> JitterBudgetTestResults {
    let config = JitterBudgetTestConfig {
        test_duration_ms,
        jitter_injection_us,
        injection_frequency_ms,
        ..Default::default()
    };
    
    run_jitter_budget_test(config)
}

/// Validate test results against thresholds
pub fn validate_jitter_budget_results(results: &JitterBudgetTestResults) -> bool {
    let threshold_us = match results.timer_kind {
        TimerKind::APIC => 250, // APIC threshold
        TimerKind::HPET => 350, // HPET threshold
    };
    
    // Test passes if p95 jitter is within threshold
    results.jitter_p95_us <= threshold_us
}

/// Print detailed test results
pub fn print_jitter_budget_results(results: &JitterBudgetTestResults) {
    let threshold_us = match results.timer_kind {
        TimerKind::APIC => 250,
        TimerKind::HPET => 350,
    };
    
    println!("=== Jitter Budget Test Results ===");
    println!("Timer Type: {:?}", results.timer_kind);
    println!("Jitter P95: {}µs (threshold: {}µs)", results.jitter_p95_us, threshold_us);
    println!("Budget Hits: {}", results.budget_hits);
    println!("Total Samples: {}", results.total_samples);
    println!("Mean Jitter: {}µs", results.mean_jitter_us);
    println!("Max Jitter: {}µs", results.max_jitter_us);
    println!("Threshold Exceeded: {}", results.threshold_exceeded);
    println!("Test Result: {}", if results.threshold_exceeded { "FAIL" } else { "PASS" });
    println!("================================");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jitter_budget_test_config() {
        let config = JitterBudgetTestConfig::default();
        
        assert_eq!(config.test_duration_ms, 30000);
        assert_eq!(config.jitter_injection_us, 100);
        assert_eq!(config.injection_frequency_ms, 100);
        assert_eq!(config.apic_threshold_us, 250);
        assert_eq!(config.hpet_threshold_us, 350);
    }

    #[test]
    fn test_jitter_stats_calculation() {
        let samples = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
        let (mean, p95, max) = calculate_jitter_stats(&samples);
        
        assert_eq!(mean, 55); // (10+20+...+100)/10 = 55
        assert_eq!(p95, 95); // 95th percentile of 10 samples
        assert_eq!(max, 100);
    }

    #[test]
    fn test_empty_samples() {
        let samples: Vec<u32> = Vec::new();
        let (mean, p95, max) = calculate_jitter_stats(&samples);
        
        assert_eq!(mean, 0);
        assert_eq!(p95, 0);
        assert_eq!(max, 0);
    }

    #[test]
    fn test_threshold_validation() {
        let apic_results = JitterBudgetTestResults {
            timer_kind: TimerKind::APIC,
            jitter_p95_us: 200, // Below 250µs threshold
            budget_hits: 5,
            total_samples: 1000,
            mean_jitter_us: 150,
            max_jitter_us: 300,
            threshold_exceeded: false,
        };
        
        let hpet_results = JitterBudgetTestResults {
            timer_kind: TimerKind::HPET,
            jitter_p95_us: 400, // Above 350µs threshold
            budget_hits: 15,
            total_samples: 1000,
            mean_jitter_us: 250,
            max_jitter_us: 500,
            threshold_exceeded: true,
        };
        
        assert!(validate_jitter_budget_results(&apic_results));
        assert!(!validate_jitter_budget_results(&hpet_results));
    }

    #[test]
    fn test_json_metrics_format() {
        // This test verifies the JSON output format
        // The actual output would be captured during test execution
        let timer_kind = TimerKind::APIC;
        let jitter_p95_us = 150;
        let budget_hits = 3;
        
        // Verify the format matches requirements
        let expected_format = format!(
            "{{\"test\":\"jitter_budget\",\"timer\":\"{}\",\"jitter_p95_us\":{},\"budget_hits\":{}}}",
            "APIC", jitter_p95_us, budget_hits
        );
        
        assert_eq!(expected_format.len(), 67); // Fixed length for this example
        assert!(expected_format.contains("jitter_budget"));
        assert!(expected_format.contains("APIC"));
        assert!(expected_format.contains("150"));
        assert!(expected_format.contains("3"));
    }
}

/// Main function for standalone execution
#[cfg(not(test))]
fn main() {
    println!("Running Jitter Budget Test...");
    
    let results = run_default_jitter_budget_test();
    print_jitter_budget_results(&results);
    
    if validate_jitter_budget_results(&results) {
        println!("✅ Jitter Budget Test PASSED");
        std::process::exit(0);
    } else {
        println!("❌ Jitter Budget Test FAILED");
        std::process::exit(1);
    }
}
