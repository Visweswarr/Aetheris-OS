use std::process::{Command, exit};
use std::time::{Duration, Instant};

/// Enhanced Phase 1 gate configuration
#[derive(Debug, Clone)]
pub struct EnhancedGateConfig {
    pub ipc_p50_threshold_us: u32,      // 150μs (stricter)
    pub ipc_p95_threshold_us: u32,      // 800μs (stricter)
    pub wake_to_run_p95_threshold_ms: u32, // 4ms (stricter)
    pub boot_time_threshold_ms: u32,    // 1.5s (stricter)
    pub test_time_threshold_ms: u32,    // 4s (stricter)
    pub enable_regression_detection: bool,
    pub regression_tolerance_percent: f32,
}

impl Default for EnhancedGateConfig {
    fn default() -> Self {
        Self {
            ipc_p50_threshold_us: 150,      // 150μs
            ipc_p95_threshold_us: 800,      // 800μs
            wake_to_run_p95_threshold_ms: 4, // 4ms
            boot_time_threshold_ms: 1500,   // 1.5s
            test_time_threshold_ms: 4000,   // 4s
            enable_regression_detection: true,
            regression_tolerance_percent: 3.0, // 3% tolerance
        }
    }
}

/// Gate validation result
#[derive(Debug, Clone)]
pub struct GateValidationResult {
    pub overall_success: bool,
    pub performance_metrics: PerformanceMetrics,
    pub regression_detected: bool,
    pub execution_time_ms: u64,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub ipc_p50_us: Option<u32>,
    pub ipc_p95_us: Option<u32>,
    pub wake_to_run_p95_ms: Option<u32>,
    pub boot_time_ms: u64,
    pub test_time_ms: u64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            ipc_p50_us: None,
            ipc_p95_us: None,
            wake_to_run_p95_ms: None,
            boot_time_ms: 0,
            test_time_ms: 0,
        }
    }
}

/// Enhanced Phase 1 gate validator
pub struct EnhancedPhase1GateValidator {
    config: EnhancedGateConfig,
    baseline_metrics: Option<PerformanceMetrics>,
}

impl EnhancedPhase1GateValidator {
    /// Create new enhanced gate validator
    pub fn new(config: EnhancedGateConfig) -> Self {
        Self {
            config,
            baseline_metrics: None,
        }
    }

    /// Run enhanced Phase 1 gates
    pub fn run_gates(&mut self) -> GateValidationResult {
        let start_time = Instant::now();
        
        println!("🚀 Starting Enhanced Phase 1 Gates Validation...");
        println!("================================================");
        
        // Load baseline metrics if regression detection is enabled
        if self.config.enable_regression_detection {
            self.load_baseline_metrics();
        }
        
        // Run QEMU performance test
        println!("📊 Running QEMU Performance Test...");
        let qemu_result = self.run_qemu_performance_test();
        
        // Measure boot and test times
        println!("⏱️  Measuring Boot and Test Times...");
        let boot_time = self.measure_boot_time();
        let test_time = self.measure_test_execution_time();
        
        // Collect performance metrics
        let mut metrics = PerformanceMetrics::new();
        metrics.boot_time_ms = boot_time;
        metrics.test_time_ms = test_time;
        
        if let Ok(perf_data) = qemu_result {
            metrics.ipc_p50_us = perf_data.ipc_p50_us;
            metrics.ipc_p95_us = perf_data.ipc_p95_us;
            metrics.wake_to_run_p95_ms = perf_data.wake_to_run_p95_ms;
        }
        
        // Validate performance thresholds
        let (performance_errors, performance_warnings) = self.validate_performance_thresholds(&metrics);
        
        // Check for regressions
        let regression_detected = self.check_regressions(&metrics);
        
        // Determine overall success
        let overall_success = performance_errors.is_empty() && !regression_detected;
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        let result = GateValidationResult {
            overall_success,
            performance_metrics: metrics,
            regression_detected,
            execution_time_ms: execution_time,
            errors: performance_errors,
            warnings: performance_warnings,
        };
        
        // Print results
        self.print_results(&result);
        
        result
    }

    /// Run QEMU performance test
    fn run_qemu_performance_test(&self) -> Result<PerformanceMetrics, String> {
        let output = Command::new("bash")
            .arg("tooling/qemu/run_x86_64.sh")
            .arg("--performance-test")
            .arg("--timeout")
            .arg("30")
            .output()
            .map_err(|e| format!("Failed to run QEMU: {}", e))?;
        
        if !output.status.success() {
            return Err("QEMU execution failed".to_string());
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse performance metrics from output
        let mut metrics = PerformanceMetrics::new();
        
        // Extract IPC metrics
        if let Some(cap) = output_str.find("IPC P50:") {
            if let Some(value) = self.extract_metric_value(&output_str[cap..]) {
                metrics.ipc_p50_us = Some(value as u32);
            }
        }
        
        if let Some(cap) = output_str.find("IPC P95:") {
            if let Some(value) = self.extract_metric_value(&output_str[cap..]) {
                metrics.ipc_p95_us = Some(value as u32);
            }
        }
        
        if let Some(cap) = output_str.find("Wake-to-Run P95:") {
            if let Some(value) = self.extract_metric_value(&output_str[cap..]) {
                metrics.wake_to_run_p95_ms = Some(value as u32);
            }
        }
        
        Ok(metrics)
    }

    /// Extract metric value from string
    fn extract_metric_value(&self, text: &str) -> Option<f64> {
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() >= 2 {
            parts[1].parse::<f64>().ok()
        } else {
            None
        }
    }

    /// Measure kernel boot time
    fn measure_boot_time(&self) -> u64 {
        // TODO: Implement actual boot time measurement
        1200 // Placeholder: 1.2 seconds
    }

    /// Measure test execution time
    fn measure_test_execution_time(&self) -> u64 {
        // TODO: Implement actual test execution time measurement
        2500 // Placeholder: 2.5 seconds
    }

    /// Load baseline metrics
    fn load_baseline_metrics(&mut self) {
        // TODO: Load from baseline file
        let mut baseline = PerformanceMetrics::new();
        baseline.ipc_p50_us = Some(140);
        baseline.ipc_p95_us = Some(750);
        baseline.wake_to_run_p95_ms = Some(3);
        baseline.boot_time_ms = 1100;
        baseline.test_time_ms = 2300;
        
        self.baseline_metrics = Some(baseline);
        println!("  📊 Loaded baseline metrics");
    }

    /// Validate performance thresholds
    fn validate_performance_thresholds(&self, metrics: &PerformanceMetrics) -> (Vec<String>, Vec<String>) {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Validate IPC P50
        if let Some(ipc_p50) = metrics.ipc_p50_us {
            if ipc_p50 > self.config.ipc_p50_threshold_us {
                errors.push(format!("IPC P50 too high: {}μs (threshold: {}μs)", 
                    ipc_p50, self.config.ipc_p50_threshold_us));
            } else if ipc_p50 > self.config.ipc_p50_threshold_us * 8 / 10 {
                warnings.push(format!("IPC P50 approaching threshold: {}μs (threshold: {}μs)", 
                    ipc_p50, self.config.ipc_p50_threshold_us));
            }
        } else {
            errors.push("IPC P50 metric not available".to_string());
        }
        
        // Validate IPC P95
        if let Some(ipc_p95) = metrics.ipc_p95_us {
            if ipc_p95 > self.config.ipc_p95_threshold_us {
                errors.push(format!("IPC P95 too high: {}μs (threshold: {}μs)", 
                    ipc_p95, self.config.ipc_p95_threshold_us));
            } else if ipc_p95 > self.config.ipc_p95_threshold_us * 8 / 10 {
                warnings.push(format!("IPC P95 approaching threshold: {}μs (threshold: {}μs)", 
                    ipc_p95, self.config.ipc_p95_threshold_us));
            }
        } else {
            errors.push("IPC P95 metric not available".to_string());
        }
        
        // Validate Wake-to-Run P95
        if let Some(wake_to_run) = metrics.wake_to_run_p95_ms {
            if wake_to_run > self.config.wake_to_run_p95_threshold_ms {
                errors.push(format!("Wake-to-Run P95 too high: {}ms (threshold: {}ms)", 
                    wake_to_run, self.config.wake_to_run_p95_threshold_ms));
            } else if wake_to_run > self.config.wake_to_run_p95_threshold_ms * 8 / 10 {
                warnings.push(format!("Wake-to-Run P95 approaching threshold: {}ms (threshold: {}ms)", 
                    wake_to_run, self.config.wake_to_run_p95_threshold_ms));
            }
        } else {
            errors.push("Wake-to-Run P95 metric not available".to_string());
        }
        
        // Validate boot time
        if metrics.boot_time_ms > self.config.boot_time_threshold_ms {
            errors.push(format!("Boot time too high: {}ms (threshold: {}ms)", 
                metrics.boot_time_ms, self.config.boot_time_threshold_ms));
        } else if metrics.boot_time_ms > self.config.boot_time_threshold_ms * 8 / 10 {
            warnings.push(format!("Boot time approaching threshold: {}ms (threshold: {}ms)", 
                metrics.boot_time_ms, self.config.boot_time_threshold_ms));
        }
        
        // Validate test execution time
        if metrics.test_time_ms > self.config.test_time_threshold_ms {
            errors.push(format!("Test execution time too high: {}ms (threshold: {}ms)", 
                metrics.test_time_ms, self.config.test_time_threshold_ms));
        } else if metrics.test_time_ms > self.config.test_time_threshold_ms * 8 / 10 {
            warnings.push(format!("Test execution time approaching threshold: {}ms (threshold: {}ms)", 
                metrics.test_time_ms, self.config.test_time_threshold_ms));
        }
        
        (errors, warnings)
    }

    /// Check for regressions
    fn check_regressions(&self, current_metrics: &PerformanceMetrics) -> bool {
        if !self.config.enable_regression_detection {
            return false;
        }
        
        if let Some(ref baseline) = self.baseline_metrics {
            // Check IPC P50 regression
            if let (Some(current), Some(baseline_val)) = (current_metrics.ipc_p50_us, baseline.ipc_p50_us) {
                let regression_percent = if baseline_val > 0 {
                    ((current as f32 - baseline_val as f32) / baseline_val as f32 * 100.0) as f32
                } else {
                    0.0
                };
                
                if regression_percent > self.config.regression_tolerance_percent {
                    println!("  🚨 IPC P50 regression detected: {:.1}% degradation", regression_percent);
                    return true;
                }
            }
            
            // Check IPC P95 regression
            if let (Some(current), Some(baseline_val)) = (current_metrics.ipc_p95_us, baseline.ipc_p95_us) {
                let regression_percent = if baseline_val > 0 {
                    ((current as f32 - baseline_val as f32) / baseline_val as f32 * 100.0) as f32
                } else {
                    0.0
                };
                
                if regression_percent > self.config.regression_tolerance_percent {
                    println!("  🚨 IPC P95 regression detected: {:.1}% degradation", regression_percent);
                    return true;
                }
            }
            
            // Check Wake-to-Run regression
            if let (Some(current), Some(baseline_val)) = (current_metrics.wake_to_run_p95_ms, baseline.wake_to_run_p95_ms) {
                let regression_percent = if baseline_val > 0 {
                    ((current as f32 - baseline_val as f32) / baseline_val as f32 * 100.0) as f32
                } else {
                    0.0
                };
                
                if regression_percent > self.config.regression_tolerance_percent {
                    println!("  🚨 Wake-to-Run regression detected: {:.1}% degradation", regression_percent);
                    return true;
                }
            }
            
            // Check boot time regression
            let regression_percent = if baseline.boot_time_ms > 0 {
                ((current_metrics.boot_time_ms as f32 - baseline.boot_time_ms as f32) / baseline.boot_time_ms as f32 * 100.0) as f32
            } else {
                0.0
            };
            
            if regression_percent > self.config.regression_tolerance_percent {
                println!("  🚨 Boot time regression detected: {:.1}% degradation", regression_percent);
                return true;
            }
        }
        
        false
    }

    /// Print validation results
    fn print_results(&self, result: &GateValidationResult) {
        println!("\n📋 Enhanced Phase 1 Gates Validation Results");
        println!("=============================================");
        println!("Overall Success: {}", if result.overall_success { "✅ PASS" } else { "❌ FAIL" });
        println!("Execution Time: {} ms", result.execution_time_ms);
        println!();
        
        // Performance metrics
        println!("📊 Performance Metrics:");
        if let Some(ipc_p50) = result.performance_metrics.ipc_p50_us {
            println!("  IPC P50: {}μs (threshold: {}μs)", ipc_p50, self.config.ipc_p50_threshold_us);
        } else {
            println!("  IPC P50: Not available");
        }
        
        if let Some(ipc_p95) = result.performance_metrics.ipc_p95_us {
            println!("  IPC P95: {}μs (threshold: {}μs)", ipc_p95, self.config.ipc_p95_threshold_us);
        } else {
            println!("  IPC P95: Not available");
        }
        
        if let Some(wake_to_run) = result.performance_metrics.wake_to_run_p95_ms {
            println!("  Wake-to-Run P95: {}ms (threshold: {}ms)", wake_to_run, self.config.wake_to_run_p95_threshold_ms);
        } else {
            println!("  Wake-to-Run P95: Not available");
        }
        
        println!("  Boot Time: {}ms (threshold: {}ms)", 
            result.performance_metrics.boot_time_ms, self.config.boot_time_threshold_ms);
        println!("  Test Execution Time: {}ms (threshold: {}ms)", 
            result.performance_metrics.test_time_ms, self.config.test_time_threshold_ms);
        
        // Regression detection
        if result.regression_detected {
            println!("\n🚨 Performance regression detected!");
        } else {
            println!("\n✅ No performance regressions detected");
        }
        
        // Errors and warnings
        if !result.errors.is_empty() {
            println!("\n❌ Errors:");
            for error in &result.errors {
                println!("  • {}", error);
            }
        }
        
        if !result.warnings.is_empty() {
            println!("\n⚠️  Warnings:");
            for warning in &result.warnings {
                println!("  • {}", warning);
            }
        }
        
        println!("\n🎯 Final Result: {}", 
            if result.overall_success { "ALL GATES PASSED ✅" } else { "SOME GATES FAILED ❌" });
    }
}

fn main() {
    let config = EnhancedGateConfig::default();
    let mut validator = EnhancedPhase1GateValidator::new(config);
    
    let result = validator.run_gates();
    
    if !result.overall_success {
        println!("\n❌ Enhanced Phase 1 Gates failed! Exiting with error code 1.");
        exit(1);
    } else {
        println!("\n✅ Enhanced Phase 1 Gates passed successfully!");
    }
}
