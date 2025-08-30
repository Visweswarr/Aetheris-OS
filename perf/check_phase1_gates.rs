//! Phase 1 Automated Gate Checker for Polymera OS
//! 
//! This tool validates Phase 1 requirements and blocks merges on failure:
//! - QEMU boot test with PASS banner validation
//! - Performance metrics validation (IPC p50<200μs, wake2run p95<5ms)
//! - Unit/integration/fuzz test result validation
//! - CI integration with merge blocking

use std::fs;
use std::path::Path;
use std::process;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

/// Phase 1 Gate Configuration
#[derive(Debug, Deserialize)]
struct Phase1GateConfig {
    qemu: QemuConfig,
    performance: PerformanceThresholds,
    tests: TestRequirements,
    ci: CiConfig,
}

#[derive(Debug, Deserialize)]
struct QemuConfig {
    executable: String,
    kernel_image: String,
    timeout_seconds: u64,
    serial_log: String,
    pass_banner: String,
}

#[derive(Debug, Deserialize)]
struct PerformanceThresholds {
    ipc_p50_us: f64,
    ipc_p95_us: f64,
    wake2run_p95_ms: f64,
    ipc_throughput_ops_per_sec: f64,
}

#[derive(Debug, Deserialize)]
struct TestRequirements {
    unit_tests_pass: bool,
    integration_tests_pass: bool,
    fuzz_tests_pass: bool,
    result_directories: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CiConfig {
    block_merges: bool,
    failure_exit_code: i32,
    generate_reports: bool,
    report_dir: String,
}

/// Gate validation results
#[derive(Debug, Serialize)]
struct GateValidationResult {
    status: GateStatus,
    checks: Vec<CheckResult>,
    performance_metrics: Option<PerformanceMetrics>,
    test_results: Option<TestResults>,
    timestamp: String,
    validation_time_ms: u64,
}

#[derive(Debug, Serialize)]
enum GateStatus {
    Passed,
    Failed,
    Warning,
}

#[derive(Debug, Serialize)]
struct CheckResult {
    name: String,
    status: CheckStatus,
    error: Option<String>,
    duration_ms: u64,
}

#[derive(Debug, Serialize)]
enum CheckStatus {
    Passed,
    Failed,
    Warning,
    Skipped,
}

#[derive(Debug, Serialize)]
struct PerformanceMetrics {
    ipc_p50_us: Option<f64>,
    ipc_p95_us: Option<f64>,
    wake2run_p95_ms: Option<f64>,
    ipc_throughput_ops_per_sec: Option<f64>,
}

#[derive(Debug, Serialize)]
struct TestResults {
    unit_tests: TestResultSummary,
    integration_tests: TestResultSummary,
    fuzz_tests: TestResultSummary,
}

#[derive(Debug, Serialize)]
struct TestResultSummary {
    total: u32,
    passed: u32,
    failed: u32,
    skipped: u32,
    success_rate: f64,
}

/// Phase 1 Gate Checker
struct Phase1GateChecker {
    config: Phase1GateConfig,
    results: GateValidationResult,
}

impl Phase1GateChecker {
    fn new(config: Phase1GateConfig) -> Self {
        Self {
            config,
            results: GateValidationResult {
                status: GateStatus::Passed,
                checks: Vec::new(),
                performance_metrics: None,
                test_results: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
                validation_time_ms: 0,
            },
        }
    }

    fn run_all_checks(&mut self) -> Result<()> {
        let start_time = Instant::now();
        println!("Starting Phase 1 gate validation...");

        // Check 1: QEMU boot test with PASS banner
        self.check_qemu_boot()?;

        // Check 2: Performance metrics validation
        self.check_performance_metrics()?;

        // Check 3: Test result validation
        self.check_test_results()?;

        // Check 4: Overall gate status
        self.determine_overall_status();

        self.results.validation_time_ms = start_time.elapsed().as_millis() as u64;

        if self.config.ci.generate_reports {
            self.generate_report()?;
        }

        println!("Phase 1 gate validation completed in {}ms", self.results.validation_time_ms);
        Ok(())
    }

    fn check_qemu_boot(&mut self) -> Result<()> {
        let check_name = "QEMU Boot Test with PASS Banner";
        let start_time = Instant::now();
        
        println!("Running QEMU boot test...");

        if !Path::new(&self.config.qemu.kernel_image).exists() {
            let error_msg = format!("Kernel image not found: {}", self.config.qemu.kernel_image);
            self.add_check_result(check_name, CheckStatus::Failed, Some(error_msg), start_time);
            return Err(anyhow!("Kernel image not found"));
        }

        let qemu_result = self.run_qemu_test()?;
        let pass_validation = self.validate_pass_banner(&qemu_result)?;
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        self.add_check_result(check_name, CheckStatus::Passed, None, duration_ms);

        println!("QEMU boot test completed successfully");
        Ok(())
    }

    fn run_qemu_test(&self) -> Result<String> {
        let qemu_cmd = format!(
            "{} -kernel {} -serial stdio -display none -no-reboot -no-shutdown -m 512",
            self.config.qemu.executable,
            self.config.qemu.kernel_image
        );

        println!("Executing QEMU command: {}", qemu_cmd);

        let output = std::process::Command::new("timeout")
            .arg(format!("{}", self.config.qemu.timeout_seconds))
            .arg("bash")
            .arg("-c")
            .arg(&qemu_cmd)
            .output()
            .context("Failed to execute QEMU test")?;

        fs::write(&self.config.qemu.serial_log, &output.stdout)
            .context("Failed to write serial log")?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        println!("QEMU test completed, captured {} bytes of output", output_str.len());

        Ok(output_str)
    }

    fn validate_pass_banner(&self, output: &str) -> Result<()> {
        println!("Validating PASS banner in QEMU output...");

        if output.contains(&self.config.qemu.pass_banner) {
            println!("✅ PASS banner found: {}", self.config.qemu.pass_banner);
            Ok(())
        } else {
            let error_msg = format!(
                "PASS banner '{}' not found in QEMU output. Output preview:\n{}",
                self.config.qemu.pass_banner,
                output.lines().take(20).collect::<Vec<_>>().join("\n")
            );
            Err(anyhow!(error_msg))
        }
    }

    fn check_performance_metrics(&mut self) -> Result<()> {
        let check_name = "Performance Metrics Validation";
        let start_time = Instant::now();

        println!("Validating performance metrics...");

        let metrics = self.parse_sys_stats()?;
        let validation_result = self.validate_performance_thresholds(&metrics)?;
        
        self.results.performance_metrics = Some(metrics);

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let status = if validation_result.is_empty() {
            CheckStatus::Passed
        } else {
            CheckStatus::Failed
        };

        let error_msg = if !validation_result.is_empty() {
            Some(validation_result.join("; "))
        } else {
            None
        };

        self.add_check_result(check_name, status, error_msg, duration_ms);

        if status == CheckStatus::Passed {
            println!("✅ Performance metrics validation passed");
        } else {
            println!("⚠️ Performance metrics validation failed: {}", validation_result.join("; "));
        }

        Ok(())
    }

    fn parse_sys_stats(&self) -> Result<PerformanceMetrics> {
        let serial_log = fs::read_to_string(&self.config.qemu.serial_log)
            .context("Failed to read serial log")?;

        let mut metrics = PerformanceMetrics {
            ipc_p50_us: None,
            ipc_p95_us: None,
            wake2run_p95_ms: None,
            ipc_throughput_ops_per_sec: None,
        };

        for line in serial_log.lines() {
            if line.contains("ipc_median_latency") {
                if let Some(value) = self.extract_numeric_value(line) {
                    metrics.ipc_p50_us = Some(value);
                }
            } else if line.contains("ipc_p95_latency") {
                if let Some(value) = self.extract_numeric_value(line) {
                    metrics.ipc_p95_us = Some(value);
                }
            } else if line.contains("wake2run_p95") {
                if let Some(value) = self.extract_numeric_value(line) {
                    metrics.wake2run_p95_ms = Some(value);
                }
            } else if line.contains("ipc_throughput") {
                if let Some(value) = self.extract_numeric_value(line) {
                    metrics.ipc_throughput_ops_per_sec = Some(value);
                }
            }
        }

        println!("Parsed performance metrics: {:?}", metrics);
        Ok(metrics)
    }

    fn extract_numeric_value(&self, line: &str) -> Option<f64> {
        line.split(':')
            .nth(1)?
            .trim()
            .parse::<f64>()
            .ok()
    }

    fn validate_performance_thresholds(&self, metrics: &PerformanceMetrics) -> Result<Vec<String>> {
        let mut violations = Vec::new();

        if let Some(ipc_p50) = metrics.ipc_p50_us {
            if ipc_p50 >= self.config.performance.ipc_p50_us {
                violations.push(format!(
                    "IPC p50 latency {}μs >= threshold {}μs",
                    ipc_p50, self.config.performance.ipc_p50_us
                ));
            }
        }

        if let Some(ipc_p95) = metrics.ipc_p95_us {
            if ipc_p95 >= self.config.performance.ipc_p95_us {
                violations.push(format!(
                    "IPC p95 latency {}μs >= threshold {}μs",
                    ipc_p95, self.config.performance.ipc_p95_us
                ));
            }
        }

        if let Some(wake2run_p95) = metrics.wake2run_p95_ms {
            if wake2run_p95 >= self.config.performance.wake2run_p95_ms {
                violations.push(format!(
                    "Wake-to-run p95 latency {}ms >= threshold {}ms",
                    wake2run_p95, self.config.performance.wake2run_p95_ms
                ));
            }
        }

        if let Some(throughput) = metrics.ipc_throughput_ops_per_sec {
            if throughput < self.config.performance.ipc_throughput_ops_per_sec {
                violations.push(format!(
                    "IPC throughput {} ops/sec < threshold {} ops/sec",
                    throughput, self.config.performance.ipc_throughput_ops_per_sec
                ));
            }
        }

        Ok(violations)
    }

    fn check_test_results(&mut self) -> Result<()> {
        let check_name = "Test Results Validation";
        let start_time = Instant::now();

        println!("Validating test results...");

        let test_results = self.collect_test_results()?;
        self.results.test_results = Some(test_results.clone());

        let all_tests_passed = test_results.unit_tests.failed == 0
            && test_results.integration_tests.failed == 0
            && test_results.fuzz_tests.failed == 0;

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let status = if all_tests_passed {
            CheckStatus::Passed
        } else {
            CheckStatus::Failed
        };

        let error_msg = if !all_tests_passed {
            Some(format!(
                "Test failures: Unit={}, Integration={}, Fuzz={}",
                test_results.unit_tests.failed,
                test_results.integration_tests.failed,
                test_results.fuzz_tests.failed
            ))
        } else {
            None
        };

        self.add_check_result(check_name, status, error_msg, duration_ms);

        if status == CheckStatus::Passed {
            println!("✅ Test results validation passed");
        } else {
            println!("⚠️ Test results validation failed: {}", error_msg.as_ref().unwrap());
        }

        Ok(())
    }

    fn collect_test_results(&self) -> Result<TestResults> {
        let mut test_results = TestResults {
            unit_tests: TestResultSummary {
                total: 0,
                passed: 0,
                failed: 0,
                skipped: 0,
                success_rate: 0.0,
            },
            integration_tests: TestResultSummary {
                total: 0,
                passed: 0,
                failed: 0,
                skipped: 0,
                success_rate: 0.0,
            },
            fuzz_tests: TestResultSummary {
                total: 0,
                passed: 0,
                failed: 0,
                skipped: 0,
                success_rate: 0.0,
            },
        };

        for result_dir in &self.config.tests.result_directories {
            if let Ok(entries) = fs::read_dir(result_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(extension) = path.extension() {
                        match extension.to_str().unwrap() {
                            "xml" | "json" => {
                                self.parse_test_result_file(&path, &mut test_results)?;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        test_results.unit_tests.calculate_success_rate();
        test_results.integration_tests.calculate_success_rate();
        test_results.fuzz_tests.calculate_success_rate();

        println!("Collected test results: {:?}", test_results);
        Ok(test_results)
    }

    fn parse_test_result_file(&self, path: &Path, results: &mut TestResults) -> Result<()> {
        let content = fs::read_to_string(path)
            .context(format!("Failed to read test result file: {:?}", path))?;

        if content.contains("unit_test") || path.to_string_lossy().contains("unit") {
            self.update_test_summary(&mut results.unit_tests, &content);
        } else if content.contains("integration_test") || path.to_string_lossy().contains("integration") {
            self.update_test_summary(&mut results.integration_tests, &content);
        } else if content.contains("fuzz_test") || path.to_string_lossy().contains("fuzz") {
            self.update_test_summary(&mut results.fuzz_tests, &content);
        }

        Ok(())
    }

    fn update_test_summary(&self, summary: &mut TestResultSummary, content: &str) {
        let lines: Vec<&str> = content.lines().collect();
        
        for line in lines {
            if line.contains("passed") || line.contains("PASS") {
                summary.passed += 1;
            } else if line.contains("failed") || line.contains("FAIL") {
                summary.failed += 1;
            } else if line.contains("skipped") || line.contains("SKIP") {
                summary.skipped += 1;
            }
        }
        
        summary.total = summary.passed + summary.failed + summary.skipped;
    }

    fn add_check_result(&mut self, name: &str, status: CheckStatus, error: Option<String>, duration_ms: u64) {
        let check_result = CheckResult {
            name: name.to_string(),
            status: status.clone(),
            error,
            duration_ms,
        };

        self.results.checks.push(check_result);

        if matches!(status, CheckStatus::Failed) {
            self.results.status = GateStatus::Failed;
        }
    }

    fn determine_overall_status(&mut self) {
        let failed_checks = self.results.checks.iter()
            .filter(|check| matches!(check.status, CheckStatus::Failed))
            .count();

        let warning_checks = self.results.checks.iter()
            .filter(|check| matches!(check.status, CheckStatus::Warning))
            .count();

        self.results.status = if failed_checks > 0 {
            GateStatus::Failed
        } else if warning_checks > 0 {
            GateStatus::Warning
        } else {
            GateStatus::Passed
        };

        println!("Gate status determined: {:?}", self.results.status);
    }

    fn generate_report(&self) -> Result<()> {
        let report_path = Path::new(&self.config.ci.report_dir).join("phase1_gate_report.json");
        
        if let Some(parent) = report_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create report directory")?;
        }

        let report_json = serde_json::to_string_pretty(&self.results)
            .context("Failed to serialize gate results")?;

        fs::write(&report_path, report_json)
            .context("Failed to write gate report")?;

        println!("Gate report generated: {:?}", report_path);
        Ok(())
    }

    fn get_exit_code(&self) -> i32 {
        match self.results.status {
            GateStatus::Passed => 0,
            GateStatus::Warning => 0,
            GateStatus::Failed => self.config.ci.failure_exit_code,
        }
    }

    fn print_summary(&self) {
        println!("\n🎯 Phase 1 Gate Validation Summary");
        println!("==================================");
        println!("Status: {:?}", self.results.status);
        println!("Timestamp: {}", self.results.timestamp);
        println!("Validation Time: {}ms", self.results.validation_time_ms);
        println!();

        println!("Check Results:");
        for check in &self.results.checks {
            let status_icon = match check.status {
                CheckStatus::Passed => "✅",
                CheckStatus::Failed => "❌",
                CheckStatus::Warning => "⚠️",
                CheckStatus::Skipped => "⏭️",
            };
            println!("  {} {} ({}ms)", status_icon, check.name, check.duration_ms);
            if let Some(error) = &check.error {
                println!("    Error: {}", error);
            }
        }

        println!();
        if let Some(metrics) = &self.results.performance_metrics {
            println!("Performance Metrics:");
            if let Some(ipc_p50) = metrics.ipc_p50_us {
                println!("  IPC p50: {}μs", ipc_p50);
            }
            if let Some(ipc_p95) = metrics.ipc_p95_us {
                println!("  IPC p95: {}μs", ipc_p95);
            }
            if let Some(wake2run_p95) = metrics.wake2run_p95_ms {
                println!("  Wake-to-run p95: {}ms", wake2run_p95);
            }
            if let Some(throughput) = metrics.ipc_throughput_ops_per_sec {
                println!("  IPC Throughput: {} ops/sec", throughput);
            }
        }

        println!();
        if let Some(test_results) = &self.results.test_results {
            println!("Test Results:");
            println!("  Unit Tests: {}/{} passed ({:.1}%)", 
                test_results.unit_tests.passed, 
                test_results.unit_tests.total,
                test_results.unit_tests.success_rate);
            println!("  Integration Tests: {}/{} passed ({:.1}%)", 
                test_results.integration_tests.passed, 
                test_results.integration_tests.total,
                test_results.integration_tests.success_rate);
            println!("  Fuzz Tests: {}/{} passed ({:.1}%)", 
                test_results.fuzz_tests.passed, 
                test_results.fuzz_tests.total,
                test_results.fuzz_tests.success_rate);
        }

        println!();
        match self.results.status {
            GateStatus::Passed => {
                println!("🎉 Phase 1 Gate Validation PASSED - Ready for merge!");
            }
            GateStatus::Warning => {
                println!("⚠️ Phase 1 Gate Validation PASSED with warnings");
            }
            GateStatus::Failed => {
                println!("❌ Phase 1 Gate Validation FAILED - Merge blocked!");
                if self.config.ci.block_merges {
                    println!("   Merges will be blocked until issues are resolved.");
                }
            }
        }
    }
}

impl TestResultSummary {
    fn calculate_success_rate(&mut self) {
        if self.total > 0 {
            self.success_rate = (self.passed as f64 / self.total as f64) * 100.0;
        }
    }
}

fn load_config(config_path: &str) -> Result<Phase1GateConfig> {
    let config_content = fs::read_to_string(config_path)
        .context(format!("Failed to read config file: {}", config_path))?;
    
    let config: Phase1GateConfig = serde_yaml::from_str(&config_content)
        .context("Failed to parse config file")?;
    
    Ok(config)
}

fn main() -> Result<()> {
    let config_path = "perf/phase1_gates.yaml";
    println!("Starting Phase 1 gate validation with config: {}", config_path);

    let config = load_config(config_path)?;
    println!("Configuration loaded successfully");

    let mut checker = Phase1GateChecker::new(config);
    checker.run_all_checks()?;

    checker.print_summary();

    let exit_code = checker.get_exit_code();
    if exit_code != 0 {
        eprintln!("Phase 1 gate validation failed with exit code: {}", exit_code);
    }

    std::process::exit(exit_code);
}
