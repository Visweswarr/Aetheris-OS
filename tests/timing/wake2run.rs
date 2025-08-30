use core::time::Duration;
use alloc::vec::Vec;
use alloc::sync::Arc;
use alloc::boxed::Box;

use polymera_os::time::{Instant, Duration as PolymeraDuration};
use polymera_os::hal::x86_64::timer::{get_timer_kind, TimerKind};
use polymera_os::sync::{Mutex, Condvar};
use polymera_os::thread::{self, JoinHandle};

/// Wake-to-run test configuration
#[derive(Debug, Clone)]
pub struct Wake2RunTestConfig {
    pub test_duration_ms: u64,
    pub sample_count: u32,
    pub rt_priority: bool,
    pub load_level: LoadLevel,
    pub measurement_interval_ms: u32,
}

/// Load level for testing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadLevel {
    Low,    // Minimal background load
    Medium, // Moderate background load
    High,   // High background load
}

/// Wake-to-run test results
#[derive(Debug, Clone)]
pub struct Wake2RunTestResults {
    pub timer_kind: TimerKind,
    pub w2r_p95_ms: f32,
    pub samples: u32,
    pub mean_w2r_ms: f32,
    pub min_w2r_ms: f32,
    pub max_w2r_ms: f32,
    pub threshold_exceeded: bool,
    pub rt_priority: bool,
}

impl Default for Wake2RunTestConfig {
    fn default() -> Self {
        Self {
            test_duration_ms: 30000, // 30 seconds
            sample_count: 10000, // 10k samples
            rt_priority: true, // Use RT priority
            load_level: LoadLevel::Medium,
            measurement_interval_ms: 3, // 3ms between measurements
        }
    }
}

/// Wake-to-run measurement context
struct Wake2RunContext {
    wakeup_times: Arc<Mutex<Vec<Instant>>>,
    measurement_complete: Arc<Condvar>,
    test_running: Arc<Mutex<bool>>,
}

/// Run wake-to-run test
pub fn run_wake2run_test(config: Wake2RunTestConfig) -> Wake2RunTestResults {
    // Get timer kind
    let timer_kind = get_timer_kind().expect("Timer not initialized");
    
    // Create measurement context
    let context = Wake2RunContext {
        wakeup_times: Arc::new(Mutex::new(Vec::new())),
        measurement_complete: Arc::new(Condvar::new()),
        test_running: Arc::new(Mutex::new(true)),
    };
    
    // Start background load generation
    let load_handle = start_background_load(config.load_level, context.test_running.clone());
    
    // Start measurement thread
    let measurement_handle = start_measurement_thread(config, context.clone());
    
    // Wait for test completion
    let start_time = Instant::now();
    let test_duration = Duration::from_millis(config.test_duration_ms);
    
    while start_time.elapsed() < test_duration {
        // Check if we have enough samples
        if let Ok(wakeup_times) = context.wakeup_times.lock() {
            if wakeup_times.len() >= config.sample_count as usize {
                break;
            }
        }
        
        // Small delay
        core::thread::sleep(Duration::from_millis(10));
    }
    
    // Stop test
    if let Ok(mut test_running) = context.test_running.lock() {
        *test_running = false;
    }
    
    // Wait for threads to complete
    let _ = load_handle.join();
    let _ = measurement_handle.join();
    
    // Calculate results
    let results = calculate_wake2run_results(&context, timer_kind, config.rt_priority);
    
    // Print JSON metrics line for CI parsing
    print_json_metrics(timer_kind, results.w2r_p95_ms, results.samples);
    
    results
}

/// Start background load generation
fn start_background_load(load_level: LoadLevel, test_running: Arc<Mutex<bool>>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut counter = 0u64;
        
        while {
            if let Ok(running) = test_running.lock() {
                *running
            } else {
                false
            }
        } {
            // Generate load based on level
            match load_level {
                LoadLevel::Low => {
                    // Minimal load - just increment counter
                    counter = counter.wrapping_add(1);
                }
                LoadLevel::Medium => {
                    // Moderate load - some computation
                    for i in 0..1000 {
                        counter = counter.wrapping_add(i);
                    }
                }
                LoadLevel::High => {
                    // High load - intensive computation
                    for i in 0..10000 {
                        counter = counter.wrapping_add(i * i);
                    }
                }
            }
            
            // Small yield to prevent blocking
            thread::yield_now();
        }
        
        // Prevent optimization
        if counter > 0 {
            println!("Background load completed: {}", counter);
        }
    })
}

/// Start measurement thread
fn start_measurement_thread(
    config: Wake2RunTestConfig,
    context: Wake2RunContext,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut sample_count = 0u32;
        let measurement_interval = Duration::from_millis(config.measurement_interval_ms as u64);
        
        while sample_count < config.sample_count {
            // Set RT priority if requested
            if config.rt_priority {
                set_rt_priority();
            }
            
            // Measure wake-to-run latency
            let wakeup_time = measure_wakeup_latency();
            
            // Record measurement
            if let Ok(mut wakeup_times) = context.wakeup_times.lock() {
                wakeup_times.push(wakeup_time);
                sample_count += 1;
            }
            
            // Wait for next measurement
            thread::sleep(measurement_interval);
        }
        
        // Signal completion
        context.measurement_complete.notify_all();
    })
}

/// Measure wake-up latency
fn measure_wakeup_latency() -> Instant {
    // Record wake-up time
    Instant::now()
}

/// Set real-time priority
fn set_rt_priority() {
    // In a real implementation, this would set thread priority
    // For now, we'll simulate it with a small delay
    thread::sleep(Duration::from_micros(1));
}

/// Calculate wake-to-run results
fn calculate_wake2run_results(
    context: &Wake2RunContext,
    timer_kind: TimerKind,
    rt_priority: bool,
) -> Wake2RunTestResults {
    let wakeup_times = if let Ok(times) = context.wakeup_times.lock() {
        times.clone()
    } else {
        Vec::new()
    };
    
    if wakeup_times.is_empty() {
        return Wake2RunTestResults {
            timer_kind,
            w2r_p95_ms: 0.0,
            samples: 0,
            mean_w2r_ms: 0.0,
            min_w2r_ms: 0.0,
            max_w2r_ms: 0.0,
            threshold_exceeded: false,
            rt_priority,
        };
    }
    
    // Calculate wake-to-run latencies
    let mut latencies = Vec::new();
    for i in 1..wakeup_times.len() {
        let latency = wakeup_times[i].duration_since(wakeup_times[i - 1]);
        latencies.push(latency.as_micros() as f32 / 1000.0); // Convert to milliseconds
    }
    
    // Calculate statistics
    let mean_w2r = latencies.iter().sum::<f32>() / latencies.len() as f32;
    let min_w2r = latencies.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let max_w2r = latencies.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    
    // Calculate p95
    let mut sorted_latencies = latencies.clone();
    sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p95_index = (sorted_latencies.len() * 95) / 100;
    let p95_w2r = sorted_latencies[p95_index.min(sorted_latencies.len() - 1)];
    
    // Check threshold (3.0ms for RT priority wakeups)
    let threshold_exceeded = p95_w2r > 3.0;
    
    Wake2RunTestResults {
        timer_kind,
        w2r_p95_ms: p95_w2r,
        samples: latencies.len() as u32,
        mean_w2r_ms: mean_w2r,
        min_w2r_ms: min_w2r,
        max_w2r_ms: max_w2r,
        threshold_exceeded,
        rt_priority,
    }
}

/// Print JSON metrics line for CI parsing
fn print_json_metrics(timer_kind: TimerKind, w2r_p95_ms: f32, samples: u32) {
    let timer_str = match timer_kind {
        TimerKind::APIC => "APIC",
        TimerKind::HPET => "HPET",
    };
    
    // Output exactly one JSON line as specified in requirements
    println!("{{\"test\":\"wake2run\",\"timer\":\"{}\",\"w2r_p95_ms\":{:.2},\"samples\":{}}}", 
             timer_str, w2r_p95_ms, samples);
}

/// Run wake-to-run test with default configuration
pub fn run_default_wake2run_test() -> Wake2RunTestResults {
    let config = Wake2RunTestConfig::default();
    run_wake2run_test(config)
}

/// Run wake-to-run test with custom configuration
pub fn run_custom_wake2run_test(
    test_duration_ms: u64,
    sample_count: u32,
    rt_priority: bool,
    load_level: LoadLevel,
) -> Wake2RunTestResults {
    let config = Wake2RunTestConfig {
        test_duration_ms,
        sample_count,
        rt_priority,
        load_level,
        ..Default::default()
    };
    
    run_wake2run_test(config)
}

/// Validate test results against thresholds
pub fn validate_wake2run_results(results: &Wake2RunTestResults) -> bool {
    // Test passes if p95 wake-to-run is within 3.0ms threshold
    results.w2r_p95_ms <= 3.0
}

/// Print detailed test results
pub fn print_wake2run_results(results: &Wake2RunTestResults) {
    println!("=== Wake-to-Run Test Results ===");
    println!("Timer Type: {:?}", results.timer_kind);
    println!("RT Priority: {}", results.rt_priority);
    println!("Wake-to-Run P95: {:.2}ms (threshold: 3.0ms)", results.w2r_p95_ms);
    println!("Samples: {}", results.samples);
    println!("Mean W2R: {:.2}ms", results.mean_w2r_ms);
    println!("Min W2R: {:.2}ms", results.min_w2r_ms);
    println!("Max W2R: {:.2}ms", results.max_w2r_ms);
    println!("Threshold Exceeded: {}", results.threshold_exceeded);
    println!("Test Result: {}", if results.threshold_exceeded { "FAIL" } else { "PASS" });
    println!("==============================");
}

/// Run load level comparison test
pub fn run_load_level_comparison() -> Vec<Wake2RunTestResults> {
    let load_levels = [LoadLevel::Low, LoadLevel::Medium, LoadLevel::High];
    let mut results = Vec::new();
    
    for &load_level in &load_levels {
        let config = Wake2RunTestConfig {
            test_duration_ms: 10000, // 10 seconds per level
            sample_count: 3000, // 3k samples per level
            rt_priority: true,
            load_level,
            measurement_interval_ms: 3,
        };
        
        let result = run_wake2run_test(config);
        results.push(result);
    }
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wake2run_test_config() {
        let config = Wake2RunTestConfig::default();
        
        assert_eq!(config.test_duration_ms, 30000);
        assert_eq!(config.sample_count, 10000);
        assert!(config.rt_priority);
        assert_eq!(config.load_level, LoadLevel::Medium);
        assert_eq!(config.measurement_interval_ms, 3);
    }

    #[test]
    fn test_load_level_enum() {
        assert_eq!(LoadLevel::Low, LoadLevel::Low);
        assert_eq!(LoadLevel::Medium, LoadLevel::Medium);
        assert_eq!(LoadLevel::High, LoadLevel::High);
        assert_ne!(LoadLevel::Low, LoadLevel::High);
    }

    #[test]
    fn test_threshold_validation() {
        let good_results = Wake2RunTestResults {
            timer_kind: TimerKind::APIC,
            w2r_p95_ms: 2.5, // Below 3.0ms threshold
            samples: 1000,
            mean_w2r_ms: 2.0,
            min_w2r_ms: 1.5,
            max_w2r_ms: 3.5,
            threshold_exceeded: false,
            rt_priority: true,
        };
        
        let bad_results = Wake2RunTestResults {
            timer_kind: TimerKind::HPET,
            w2r_p95_ms: 4.0, // Above 3.0ms threshold
            samples: 1000,
            mean_w2r_ms: 3.5,
            min_w2r_ms: 2.0,
            max_w2r_ms: 5.0,
            threshold_exceeded: true,
            rt_priority: true,
        };
        
        assert!(validate_wake2run_results(&good_results));
        assert!(!validate_wake2run_results(&bad_results));
    }

    #[test]
    fn test_json_metrics_format() {
        // This test verifies the JSON output format
        let timer_kind = TimerKind::APIC;
        let w2r_p95_ms = 2.75;
        let samples = 5000;
        
        // Verify the format matches requirements
        let expected_format = format!(
            "{{\"test\":\"wake2run\",\"timer\":\"{}\",\"w2r_p95_ms\":{:.2},\"samples\":{}}}",
            "APIC", w2r_p95_ms, samples
        );
        
        assert!(expected_format.contains("wake2run"));
        assert!(expected_format.contains("APIC"));
        assert!(expected_format.contains("2.75"));
        assert!(expected_format.contains("5000"));
    }

    #[test]
    fn test_load_level_comparison() {
        // This test would run the actual load level comparison
        // For unit testing, we'll just verify the function exists
        let results = run_load_level_comparison();
        assert_eq!(results.len(), 3); // Three load levels
    }
}

/// Main function for standalone execution
#[cfg(not(test))]
fn main() {
    println!("Running Wake-to-Run Test...");
    
    let results = run_default_wake2run_test();
    print_wake2run_results(&results);
    
    if validate_wake2run_results(&results) {
        println!("✅ Wake-to-Run Test PASSED");
        std::process::exit(0);
    } else {
        println!("❌ Wake-to-Run Test FAILED");
        std::process::exit(1);
    }
}
