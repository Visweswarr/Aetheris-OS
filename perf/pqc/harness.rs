use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// PQC performance targets for QEMU baseline
const DILITHIUM2_SIGN_TARGET_P50_MS: f64 = 1.2;
const DILITHIUM2_VERIFY_TARGET_P50_MS: f64 = 1.4;
const KYBER768_ENCAP_TARGET_P50_MS: f64 = 0.9;
const KYBER768_DECAP_TARGET_P50_MS: f64 = 1.1;

// Benchmark configuration
const WARMUP_ITERATIONS: usize = 1000;
const BENCHMARK_ITERATIONS: usize = 10000;
const BENCHMARK_DURATION_MS: u64 = 30000; // 30 seconds
const SAMPLE_SIZE: usize = 1000;

/// Performance measurement result for a single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub operation: String,
    pub input_type: String, // "fixed" or "random"
    pub cycles: u64,
    pub duration_ns: u64,
    pub timestamp: u64,
}

/// Statistical summary for performance measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub operation: String,
    pub input_type: String,
    pub count: usize,
    pub min_cycles: u64,
    pub max_cycles: u64,
    pub mean_cycles: f64,
    pub median_cycles: u64,
    pub p50_cycles: u64,
    pub p95_cycles: u64,
    pub p99_cycles: u64,
    pub std_dev_cycles: f64,
    pub min_duration_ns: u64,
    pub max_duration_ns: u64,
    pub mean_duration_ns: f64,
    pub median_duration_ns: u64,
    pub p50_duration_ns: u64,
    pub p95_duration_ns: u64,
    pub p99_duration_ns: u64,
    pub std_dev_duration_ns: f64,
}

/// PQC algorithm performance results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqcPerformanceResults {
    pub timestamp: String,
    pub commit_hash: String,
    pub benchmark_duration_ms: u64,
    pub total_operations: usize,
    pub algorithm_results: HashMap<String, HashMap<String, PerformanceStats>>,
    pub performance_targets: HashMap<String, f64>,
    pub targets_met: HashMap<String, bool>,
    pub overall_status: String,
}

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub warmup_iterations: usize,
    pub benchmark_iterations: usize,
    pub benchmark_duration_ms: u64,
    pub sample_size: usize,
    pub enable_cycle_counting: bool,
    pub enable_timing_analysis: bool,
    pub output_file: Option<String>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            warmup_iterations: WARMUP_ITERATIONS,
            benchmark_iterations: BENCHMARK_ITERATIONS,
            benchmark_duration_ms: BENCHMARK_DURATION_MS,
            sample_size: SAMPLE_SIZE,
            enable_cycle_counting: true,
            enable_timing_analysis: true,
            output_file: None,
        }
    }
}

/// PQC Performance Benchmark Harness
pub struct PqcBenchmarkHarness {
    config: BenchmarkConfig,
    results: Vec<OperationResult>,
    algorithm_results: HashMap<String, HashMap<String, PerformanceStats>>,
}

impl PqcBenchmarkHarness {
    /// Create a new benchmark harness
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            algorithm_results: HashMap::new(),
        }
    }

    /// Run the complete PQC performance benchmark suite
    pub fn run_benchmark_suite(&mut self) -> PqcPerformanceResults {
        println!("🚀 Starting PQC Performance Benchmark Suite");
        println!("==========================================");
        println!("Configuration:");
        println!("  Warmup iterations: {}", self.config.warmup_iterations);
        println!("  Benchmark iterations: {}", self.config.benchmark_iterations);
        println!("  Duration: {}ms", self.config.benchmark_duration_ms);
        println!("  Sample size: {}", self.config.sample_size);
        println!("  Cycle counting: {}", self.config.enable_cycle_counting);
        println!("  Timing analysis: {}", self.config.enable_timing_analysis);
        println!();

        let start_time = Instant::now();
        let commit_hash = get_commit_hash().unwrap_or_else(|| "unknown".to_string());

        // Run Dilithium2 benchmarks
        println!("🔐 Benchmarking Dilithium2...");
        self.benchmark_dilithium2();

        // Run Kyber768 benchmarks
        println!("🔑 Benchmarking Kyber768...");
        self.benchmark_kyber768();

        let benchmark_duration = start_time.elapsed();
        let total_operations = self.results.len();

        // Calculate performance statistics
        self.calculate_performance_stats();

        // Check performance targets
        let targets_met = self.check_performance_targets();
        let overall_status = self.determine_overall_status(&targets_met);

        // Create results summary
        let results = PqcPerformanceResults {
            timestamp: chrono::Utc::now().to_rfc3339(),
            commit_hash,
            benchmark_duration_ms: benchmark_duration.as_millis() as u64,
            total_operations,
            algorithm_results: self.algorithm_results.clone(),
            performance_targets: self.get_performance_targets(),
            targets_met,
            overall_status,
        };

        // Export results if file specified
        if let Some(ref output_file) = self.config.output_file {
            if let Err(e) = self.export_results(&results, output_file) {
                eprintln!("Warning: Failed to export results to {}: {}", output_file, e);
            }
        }

        // Print summary
        self.print_summary(&results);

        results
    }

    /// Benchmark Dilithium2 sign and verify operations
    fn benchmark_dilithium2(&mut self) {
        println!("  📝 Dilithium2 Sign Operations...");
        
        // Generate keypair once for all tests
        let (public_key, secret_key) = self.generate_dilithium2_keypair();
        let message = b"Benchmark message for Dilithium2 signing performance testing";

        // Fixed input signing (for constant-time validation)
        let fixed_results = self.benchmark_dilithium2_sign(
            &secret_key,
            message,
            "fixed",
            self.config.benchmark_iterations,
        );
        self.results.extend(fixed_results);

        // Random input signing (for constant-time validation)
        let random_results = self.benchmark_dilithium2_sign_random(
            &secret_key,
            "random",
            self.config.benchmark_iterations,
        );
        self.results.extend(random_results);

        println!("  ✅ Dilithium2 Sign Operations completed");

        println!("  🔍 Dilithium2 Verify Operations...");
        
        // Create signature for verification
        let signature = self.sign_dilithium2(&secret_key, message);

        // Fixed input verification
        let fixed_verify_results = self.benchmark_dilithium2_verify(
            &public_key,
            message,
            &signature,
            "fixed",
            self.config.benchmark_iterations,
        );
        self.results.extend(fixed_verify_results);

        // Random input verification
        let random_verify_results = self.benchmark_dilithium2_verify_random(
            &public_key,
            &signature,
            "random",
            self.config.benchmark_iterations,
        );
        self.results.extend(random_verify_results);

        println!("  ✅ Dilithium2 Verify Operations completed");
    }

    /// Benchmark Kyber768 encapsulate and decapsulate operations
    fn benchmark_kyber768(&mut self) {
        println!("  🔐 Kyber768 Encapsulate Operations...");
        
        // Generate keypair once for all tests
        let (public_key, secret_key) = self.generate_kyber768_keypair();

        // Fixed input encapsulation
        let fixed_encap_results = self.benchmark_kyber768_encapsulate(
            &public_key,
            "fixed",
            self.config.benchmark_iterations,
        );
        self.results.extend(fixed_encap_results);

        // Random input encapsulation
        let random_encap_results = self.benchmark_kyber768_encapsulate_random(
            &public_key,
            "random",
            self.config.benchmark_iterations,
        );
        self.results.extend(random_encap_results);

        println!("  ✅ Kyber768 Encapsulate Operations completed");

        println!("  🔓 Kyber768 Decapsulate Operations...");
        
        // Create ciphertext for decapsulation
        let (ciphertext, _shared_secret) = self.encapsulate_kyber768(&public_key);

        // Fixed input decapsulation
        let fixed_decap_results = self.benchmark_kyber768_decapsulate(
            &secret_key,
            &ciphertext,
            "fixed",
            self.config.benchmark_iterations,
        );
        self.results.extend(fixed_decap_results);

        // Random input decapsulation
        let random_decap_results = self.benchmark_kyber768_decapsulate_random(
            &secret_key,
            "random",
            self.config.benchmark_iterations,
        );
        self.results.extend(random_decap_results);

        println!("  ✅ Kyber768 Decapsulate Operations completed");
    }

    /// Benchmark Dilithium2 signing with fixed input
    fn benchmark_dilithium2_sign(
        &self,
        secret_key: &[u8],
        message: &[u8],
        input_type: &str,
        iterations: usize,
    ) -> Vec<OperationResult> {
        let mut results = Vec::new();
        
        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let _ = self.sign_dilithium2(secret_key, message);
        }

        // Actual benchmark
        for i in 0..iterations {
            let start_cycles = if self.config.enable_cycle_counting {
                rdtsc()
            } else {
                0
            };
            
            let start_time = Instant::now();
            let _signature = self.sign_dilithium2(secret_key, message);
            let duration = start_time.elapsed();
            
            let end_cycles = if self.config.enable_cycle_counting {
                rdtsc()
            } else {
                0
            };

            let cycles = if self.config.enable_cycle_counting {
                end_cycles.saturating_sub(start_cycles)
            } else {
                0
            };

            results.push(OperationResult {
                operation: "dilithium2_sign".to_string(),
                input_type: input_type.to_string(),
                cycles,
                duration_ns: duration.as_nanos() as u64,
                timestamp: chrono::Utc::now().timestamp_nanos() as u64,
            });

            // Progress indicator
            if (i + 1) % 1000 == 0 {
                print!("    {} iterations completed\r", i + 1);
            }
        }
        println!();

        results
    }

    /// Benchmark Dilithium2 signing with random inputs
    fn benchmark_dilithium2_sign_random(
        &self,
        secret_key: &[u8],
        input_type: &str,
        iterations: usize,
    ) -> Vec<OperationResult> {
        let mut results = Vec::new();
        
        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let random_message = self.generate_random_message(64);
            let _ = self.sign_dilithium2(secret_key, &random_message);
        }

        // Actual benchmark
        for i in 0..iterations {
            let random_message = self.generate_random_message(64);
            
            let start_cycles = if self.config.enable_cycle_counting {
                rdtsc()
            } else {
                0
            };
            
            let start_time = Instant::now();
            let _signature = self.sign_dilithium2(secret_key, &random_message);
            let duration = start_time.elapsed();
            
            let end_cycles = if self.config.enable_cycle_counting {
                rdtsc()
            } else {
                0
            };

            let cycles = if self.config.enable_cycle_counting {
                end_cycles.saturating_sub(start_cycles)
            } else {
                0
            };

            results.push(OperationResult {
                operation: "dilithium2_sign".to_string(),
                input_type: input_type.to_string(),
                cycles,
                duration_ns: duration.as_nanos() as u64,
                duration_ns,
                timestamp: chrono::Utc::now().timestamp_nanos() as u64,
            });

            // Progress indicator
            if (i + 1) % 1000 == 0 {
                print!("    {} iterations completed\r", i + 1);
            }
        }
        println!();

        results
    }

    /// Benchmark Dilithium2 verification with fixed input
    fn benchmark_dilithium2_verify(
        &self,
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
        input_type: &str,
        iterations: usize,
    ) -> Vec<OperationResult> {
        let mut results = Vec::new();
        
        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let _ = self.verify_dilithium2(public_key, message, signature);
        }

        // Actual benchmark
        for i in 0..iterations {
            let start_cycles = if self.config.enable_cycle_counting {
                rdtsc()
            } else {
                0
            };
            
            let start_time = Instant::now();
            let _valid = self.verify_dilithium2(public_key, message, signature);
            let duration = start_time.elapsed();
            
            let end_cycles = if self.config.enable_cycle_counting {
                rdtsc()
            } else {
                0
            };

            let cycles = if self.config.enable_cycle_counting {
                end_cycles.saturating_sub(start_cycles)
            } else {
                0
            };

            results.push(OperationResult {
                operation: "dilithium2_verify".to_string(),
                input_type: input_type.to_string(),
                cycles,
                duration_ns: duration.as_nanos() as u64,
                timestamp: chrono::Utc::now().timestamp_nanos() as u64,
            });

            // Progress indicator
            if (i + 1) % 1000 == 0 {
                print!("    {} iterations completed\r", i + 1);
            }
        }
        println!();

        results
    }

    /// Benchmark Dilithium2 verification with random inputs
    fn benchmark_dilithium2_verify_random(
        &self,
        public_key: &[u8],
        signature: &[u8],
        input_type: &str,
        iterations: usize,
    ) -> Vec<OperationResult> {
        let mut results = Vec::new();
        
        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let random_message = self.generate_random_message(64);
            let _ = self.verify_dilithium2(public_key, &random_message, signature);
        }

        // Actual benchmark
        for i in 0..iterations {
            let random_message = self.generate_random_message(64);
            
            let start_cycles = if self.config.enable_cycle_counting {
                rdtsc()
            } else {
                0
            };
            
            let start_time = Instant::now();
            let _valid = self.verify_dilithium2(public_key, &random_message, signature);
            let duration = start_time.elapsed();
            
            let end_cycles = if self.config.enable_cycle_counting {
                rdtsc()
            } else {
                0
            };

            let cycles = if self.config.enable_cycle_counting {
                end_cycles.saturating_sub(start_cycles)
            } else {
                0
            };

            results.push(OperationResult {
                operation: "dilithium2_verify".to_string(),
                input_type: input_type.to_string(),
                cycles,
                duration_ns: duration.as_nanos() as u64,
                timestamp: chrono::Utc::now().timestamp_nanos() as u64,
            });

            // Progress indicator
            if (i + 1) % 1000 == 0 {
                print!("    {} iterations completed\r", i + 1);
            }
        }
        println!();

        results
    }

    /// Benchmark Kyber768 encapsulation with fixed input
    fn benchmark_kyber768_encapsulate(
        &self,
        public_key: &[u8],
        input_type: &str,
        iterations: usize,
    ) -> Vec<OperationResult> {
        let mut results = Vec::new();
        
        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let _ = self.encapsulate_kyber768(public_key);
        }

        // Actual benchmark
        for i in 0..iterations {
            let start_cycles = if self.config.enable_cycle_counting {
                rdtsc()
            } else {
                0
            };
            
            let start_time = Instant::now();
            let _ = self.encapsulate_kyber768(public_key);
            let duration = start_time.elapsed();
            
            let end_cycles = if self.config.enable_cycle_counting {
                rdtsc()
            } else {
                0
            };

            let cycles = if self.config.enable_cycle_counting {
                end_cycles.saturating_sub(start_cycles)
            } else {
                0
            };

            results.push(OperationResult {
                operation: "kyber768_encapsulate".to_string(),
                input_type: input_type.to_string(),
                cycles,
                duration_ns: duration.as_nanos() as u64,
                timestamp: chrono::Utc::now().timestamp_nanos() as u64,
            });

            // Progress indicator
            if (i + 1) % 1000 == 0 {
                print!("    {} iterations completed\r", i + 1);
            }
        }
        println!();

        results
    }

    /// Benchmark Kyber768 encapsulation with random inputs
    fn benchmark_kyber768_encapsulate_random(
        &self,
        public_key: &[u8],
        input_type: &str,
        iterations: usize,
    ) -> Vec<OperationResult> {
        let mut results = Vec::new();
        
        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let _ = self.encapsulate_kyber768(public_key);
        }

        // Actual benchmark
        for i in 0..iterations {
            let start_cycles = if self.config.enable_cycle_counting {
                rdtsc()
            } else {
                0
            };
            
            let start_time = Instant::now();
            let _ = self.encapsulate_kyber768(public_key);
            let duration = start_time.elapsed();
            
            let end_cycles = if self.config.enable_cycle_counting {
                rdtsc()
            } else {
                0
            };

            let cycles = if self.config.enable_cycle_counting {
                end_cycles.saturating_sub(start_cycles)
            } else {
                0
            };

            results.push(OperationResult {
                operation: "kyber768_encapsulate".to_string(),
                input_type: input_type.to_string(),
                cycles,
                duration_ns: duration.as_nanos() as u64,
                timestamp: chrono::Utc::now().timestamp_nanos() as u64,
            });

            // Progress indicator
            if (i + 1) % 1000 == 0 {
                print!("    {} iterations completed\r", i + 1);
            }
        }
        println!();

        results
    }

    /// Benchmark Kyber768 decapsulation with fixed input
    fn benchmark_kyber768_decapsulate(
        &self,
        secret_key: &[u8],
        ciphertext: &[u8],
        input_type: &str,
        iterations: usize,
    ) -> Vec<OperationResult> {
        let mut results = Vec::new();
        
        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let _ = self.decapsulate_kyber768(secret_key, ciphertext);
        }

        // Actual benchmark
        for i in 0..iterations {
            let start_cycles = if self.config.enable_cycle_counting {
                rdtsc()
            } else {
                0
            };
            
            let start_time = Instant::now();
            let _ = self.decapsulate_kyber768(secret_key, ciphertext);
            let duration = start_time.elapsed();
            
            let end_cycles = if self.config.enable_cycle_counting {
                rdtsc()
            } else {
                0
            };

            let cycles = if self.config.enable_cycle_counting {
                end_cycles.saturating_sub(start_cycles)
            } else {
                0
            };

            results.push(OperationResult {
                operation: "kyber768_decapsulate".to_string(),
                input_type: input_type.to_string(),
                cycles,
                duration_ns: duration.as_nanos() as u64,
                timestamp: chrono::Utc::now().timestamp_nanos() as u64,
            });

            // Progress indicator
            if (i + 1) % 1000 == 0 {
                print!("    {} iterations completed\r", i + 1);
            }
        }
        println!();

        results
    }

    /// Benchmark Kyber768 decapsulation with random inputs
    fn benchmark_kyber768_decapsulate_random(
        &self,
        secret_key: &[u8],
        input_type: &str,
        iterations: usize,
    ) -> Vec<OperationResult> {
        let mut results = Vec::new();
        
        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let random_ciphertext = self.generate_random_ciphertext();
            let _ = self.decapsulate_kyber768(secret_key, &random_ciphertext);
        }

        // Actual benchmark
        for i in 0..iterations {
            let random_ciphertext = self.generate_random_ciphertext();
            
            let start_cycles = if self.config.enable_cycle_counting {
                rdtsc()
            } else {
                0
            };
            
            let start_time = Instant::now();
            let _ = self.decapsulate_kyber768(secret_key, &random_ciphertext);
            let duration = start_time.elapsed();
            
            let end_cycles = if self.config.enable_cycle_counting {
                rdtsc()
            } else {
                0
            };

            let cycles = if self.config.enable_cycle_counting {
                end_cycles.saturating_sub(start_cycles)
            } else {
                0
            };

            results.push(OperationResult {
                operation: "kyber768_decapsulate".to_string(),
                input_type: input_type.to_string(),
                cycles,
                duration_ns: duration.as_nanos() as u64,
                timestamp: chrono::Utc::now().timestamp_nanos() as u64,
            });

            // Progress indicator
            if (i + 1) % 1000 == 0 {
                print!("    {} iterations completed\r", i + 1);
            }
        }
        println!();

        results
    }

    /// Calculate performance statistics for all operations
    fn calculate_performance_stats(&mut self) {
        println!("📊 Calculating performance statistics...");

        // Group results by operation and input type
        let mut grouped_results: HashMap<String, HashMap<String, Vec<&OperationResult>>> = HashMap::new();

        for result in &self.results {
            grouped_results
                .entry(result.operation.clone())
                .or_default()
                .entry(result.input_type.clone())
                .or_default()
                .push(result);
        }

        // Calculate statistics for each group
        for (operation, input_types) in grouped_results {
            self.algorithm_results.insert(operation.clone(), HashMap::new());
            
            for (input_type, results) in input_types {
                let stats = self.calculate_stats_for_group(&operation, &input_type, results);
                self.algorithm_results
                    .get_mut(&operation)
                    .unwrap()
                    .insert(input_type, stats);
            }
        }

        println!("✅ Performance statistics calculated");
    }

    /// Calculate statistics for a specific group of results
    fn calculate_stats_for_group(
        &self,
        operation: &str,
        input_type: &str,
        results: Vec<&OperationResult>,
    ) -> PerformanceStats {
        let mut cycles: Vec<u64> = results.iter().map(|r| r.cycles).collect();
        let mut durations: Vec<u64> = results.iter().map(|r| r.duration_ns).collect();

        // Sort for percentile calculations
        cycles.sort_unstable();
        durations.sort_unstable();

        let count = results.len();
        let min_cycles = cycles.first().copied().unwrap_or(0);
        let max_cycles = cycles.last().copied().unwrap_or(0);
        let mean_cycles = cycles.iter().map(|&x| x as f64).sum::<f64>() / count as f64;
        let median_cycles = cycles[count / 2];
        let p50_cycles = cycles[(count * 50) / 100];
        let p95_cycles = cycles[(count * 95) / 100];
        let p99_cycles = cycles[(count * 99) / 100];

        let min_duration = durations.first().copied().unwrap_or(0);
        let max_duration = durations.last().copied().unwrap_or(0);
        let mean_duration = durations.iter().map(|&x| x as f64).sum::<f64>() / count as f64;
        let median_duration = durations[count / 2];
        let p50_duration = durations[(count * 50) / 100];
        let p95_duration = durations[(count * 95) / 100];
        let p99_duration = durations[(count * 99) / 100];

        // Calculate standard deviations
        let variance_cycles = cycles
            .iter()
            .map(|&x| (x as f64 - mean_cycles).powi(2))
            .sum::<f64>()
            / count as f64;
        let std_dev_cycles = variance_cycles.sqrt();

        let variance_duration = durations
            .iter()
            .map(|&x| (x as f64 - mean_duration).powi(2))
            .sum::<f64>()
            / count as f64;
        let std_dev_duration = variance_duration.sqrt();

        PerformanceStats {
            operation: operation.to_string(),
            input_type: input_type.to_string(),
            count,
            min_cycles,
            max_cycles,
            mean_cycles,
            median_cycles,
            p50_cycles,
            p95_cycles,
            p99_cycles,
            std_dev_cycles,
            min_duration_ns: min_duration,
            max_duration_ns: max_duration,
            mean_duration_ns: mean_duration,
            median_duration_ns: median_duration,
            p50_duration_ns: p50_duration,
            p95_duration_ns: p95_duration,
            p99_duration_ns: p99_duration,
            std_dev_duration_ns: std_dev_duration,
        }
    }

    /// Check if performance targets are met
    fn check_performance_targets(&self) -> HashMap<String, bool> {
        let mut targets_met = HashMap::new();

        // Check Dilithium2 sign target
        if let Some(stats) = self.algorithm_results
            .get("dilithium2_sign")
            .and_then(|m| m.get("fixed"))
        {
            let p50_ms = stats.p50_duration_ns as f64 / 1_000_000.0;
            targets_met.insert(
                "dilithium2_sign_p50".to_string(),
                p50_ms < DILITHIUM2_SIGN_TARGET_P50_MS,
            );
        }

        // Check Dilithium2 verify target
        if let Some(stats) = self.algorithm_results
            .get("dilithium2_verify")
            .and_then(|m| m.get("fixed"))
        {
            let p50_ms = stats.p50_duration_ns as f64 / 1_000_000.0;
            targets_met.insert(
                "dilithium2_verify_p50".to_string(),
                p50_ms < DILITHIUM2_VERIFY_TARGET_P50_MS,
            );
        }

        // Check Kyber768 encapsulate target
        if let Some(stats) = self.algorithm_results
            .get("kyber768_encapsulate")
            .and_then(|m| m.get("fixed"))
        {
            let p50_ms = stats.p50_duration_ns as f64 / 1_000_000.0;
            targets_met.insert(
                "kyber768_encapsulate_p50".to_string(),
                p50_ms < KYBER768_ENCAP_TARGET_P50_MS,
            );
        }

        // Check Kyber768 decapsulate target
        if let Some(stats) = self.algorithm_results
            .get("kyber768_decapsulate")
            .and_then(|m| m.get("fixed"))
        {
            let p50_ms = stats.p50_duration_ns as f64 / 1_000_000.0;
            let p50_ms = stats.p50_duration_ns as f64 / 1_000_000.0;
            targets_met.insert(
                "kyber768_decapsulate_p50".to_string(),
                p50_ms < KYBER768_DECAP_TARGET_P50_MS,
            );
        }

        targets_met
    }

    /// Determine overall benchmark status
    fn determine_overall_status(&self, targets_met: &HashMap<String, bool>) -> String {
        let all_targets_met = targets_met.values().all(|&met| met);
        
        if all_targets_met {
            "✅ ALL_TARGETS_MET".to_string()
        } else {
            let failed_targets: Vec<&String> = targets_met
                .iter()
                .filter(|(_, &met)| !met)
                .map(|(target, _)| target)
                .collect();
            format!("❌ TARGETS_FAILED: {}", failed_targets.join(", "))
        }
    }

    /// Get performance targets for reference
    fn get_performance_targets(&self) -> HashMap<String, f64> {
        let mut targets = HashMap::new();
        targets.insert("dilithium2_sign_p50_ms".to_string(), DILITHIUM2_SIGN_TARGET_P50_MS);
        targets.insert("dilithium2_verify_p50_ms".to_string(), DILITHIUM2_VERIFY_TARGET_P50_MS);
        targets.insert("kyber768_encapsulate_p50_ms".to_string(), KYBER768_ENCAP_TARGET_P50_MS);
        targets.insert("kyber768_decapsulate_p50_ms".to_string(), KYBER768_DECAP_TARGET_P50_MS);
        targets
    }

    /// Export results to JSON file
    fn export_results(&self, results: &PqcPerformanceResults, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(results)?;
        fs::write(filename, json)?;
        println!("📁 Results exported to: {}", filename);
        Ok(())
    }

    /// Print benchmark summary
    fn print_summary(&self, results: &PqcPerformanceResults) {
        println!();
        println!("📊 PQC Performance Benchmark Summary");
        println!("====================================");
        println!("Timestamp: {}", results.timestamp);
        println!("Commit: {}", results.commit_hash);
        println!("Duration: {}ms", results.benchmark_duration_ms);
        println!("Total Operations: {}", results.total_operations);
        println!("Overall Status: {}", results.overall_status);
        println!();

        // Print performance targets
        println!("🎯 Performance Targets:");
        for (target, value) in &results.performance_targets {
            let met = results.targets_met.get(target).unwrap_or(&false);
            let status = if *met { "✅" } else { "❌" };
            println!("  {} {}: {:.1}ms", status, target, value);
        }
        println!();

        // Print detailed results for each algorithm
        for (algorithm, input_types) in &results.algorithm_results {
            println!("🔐 {}:", algorithm);
            for (input_type, stats) in input_types {
                println!("  {} Input:", input_type);
                println!("    Count: {}", stats.count);
                println!("    P50 Duration: {:.2}ms", stats.p50_duration_ns as f64 / 1_000_000.0);
                println!("    P95 Duration: {:.2}ms", stats.p95_duration_ns as f64 / 1_000_000.0);
                if stats.cycles > 0 {
                    println!("    P50 Cycles: {}", stats.p50_cycles);
                    println!("    P95 Cycles: {}", stats.p95_cycles);
                }
                println!("    Std Dev: {:.2}ms", stats.std_dev_duration_ns as f64 / 1_000_000.0);
            }
            println!();
        }
    }

    // Stub implementations for PQC operations (these would be replaced with actual implementations)
    
    fn generate_dilithium2_keypair(&self) -> (Vec<u8>, Vec<u8>) {
        // Stub: generate 32-byte public key and 64-byte secret key
        (vec![0xAA; 32], vec![0xBB; 64])
    }

    fn sign_dilithium2(&self, _secret_key: &[u8], _message: &[u8]) -> Vec<u8> {
        // Stub: simulate signing operation with some delay
        std::thread::sleep(Duration::from_micros(1200)); // 1.2ms target
        vec![0xCC; 64]
    }

    fn verify_dilithium2(&self, _public_key: &[u8], _message: &[u8], _signature: &[u8]) -> bool {
        // Stub: simulate verification operation with some delay
        std::thread::sleep(Duration::from_micros(1400)); // 1.4ms target
        true
    }

    fn generate_kyber768_keypair(&self) -> (Vec<u8>, Vec<u8>) {
        // Stub: generate 32-byte public key and 32-byte secret key
        (vec![0xDD; 32], vec![0xEE; 32])
    }

    fn encapsulate_kyber768(&self, _public_key: &[u8]) -> (Vec<u8>, Vec<u8>) {
        // Stub: simulate encapsulation operation with some delay
        std::thread::sleep(Duration::from_micros(900)); // 0.9ms target
        (vec![0xFF; 64], vec![0x11; 32])
    }

    fn decapsulate_kyber768(&self, _secret_key: &[u8], _ciphertext: &[u8]) -> Vec<u8> {
        // Stub: simulate decapsulation operation with some delay
        std::thread::sleep(Duration::from_micros(1100)); // 1.1ms target
        vec![0x22; 32]
    }

    fn generate_random_message(&self, length: usize) -> Vec<u8> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        chrono::Utc::now().timestamp_nanos().hash(&mut hasher);
        length.hash(&mut hasher);
        
        let hash = hasher.finish();
        (0..length).map(|i| ((hash >> (i % 8 * 8)) & 0xFF) as u8).collect()
    }

    fn generate_random_ciphertext(&self) -> Vec<u8> {
        self.generate_random_message(64)
    }
}

/// Read timestamp counter (RDTSC instruction)
#[inline(always)]
fn rdtsc() -> u64 {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let low: u32;
        let high: u32;
        std::arch::asm!(
            "rdtsc",
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags)
        );
        ((high as u64) << 32) | (low as u64)
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    {
        // Fallback for non-x86_64 architectures
        std::time::Instant::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}

/// Get current git commit hash
fn get_commit_hash() -> Option<String> {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
}

/// Main function for standalone execution
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = BenchmarkConfig {
        output_file: Some("pqc_performance_results.json".to_string()),
        ..Default::default()
    };

    let mut harness = PqcBenchmarkHarness::new(config);
    let results = harness.run_benchmark_suite();

    // Check if all targets were met
    let all_targets_met = results.targets_met.values().all(|&met| met);
    
    if all_targets_met {
        println!("🎉 All performance targets met!");
        Ok(())
    } else {
        println!("⚠️  Some performance targets not met");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_config_default() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.warmup_iterations, WARMUP_ITERATIONS);
        assert_eq!(config.benchmark_iterations, BENCHMARK_ITERATIONS);
        assert_eq!(config.benchmark_duration_ms, BENCHMARK_DURATION_MS);
        assert_eq!(config.sample_size, SAMPLE_SIZE);
        assert!(config.enable_cycle_counting);
        assert!(config.enable_timing_analysis);
    }

    #[test]
    fn test_performance_stats_calculation() {
        let mut harness = PqcBenchmarkHarness::new(BenchmarkConfig::default());
        
        // Add some test results
        harness.results.push(OperationResult {
            operation: "test_op".to_string(),
            input_type: "fixed".to_string(),
            cycles: 1000,
            duration_ns: 1000000,
            timestamp: 1234567890,
        });
        
        harness.results.push(OperationResult {
            operation: "test_op".to_string(),
            input_type: "fixed".to_string(),
            cycles: 2000,
            duration_ns: 2000000,
            timestamp: 1234567891,
        });

        harness.calculate_performance_stats();
        
        assert!(harness.algorithm_results.contains_key("test_op"));
        let stats = &harness.algorithm_results["test_op"]["fixed"];
        assert_eq!(stats.count, 2);
        assert_eq!(stats.mean_cycles, 1500.0);
        assert_eq!(stats.mean_duration_ns, 1500000.0);
    }

    #[test]
    fn test_performance_targets() {
        let mut harness = PqcBenchmarkHarness::new(BenchmarkConfig::default());
        
        // Mock some results that meet targets
        harness.results.push(OperationResult {
            operation: "dilithium2_sign".to_string(),
            input_type: "fixed".to_string(),
            cycles: 1000,
            duration_ns: 1000000, // 1ms < 1.2ms target
            timestamp: 1234567890,
        });

        harness.calculate_performance_stats();
        let targets_met = harness.check_performance_targets();
        
        // Should have at least one target checked
        assert!(!targets_met.is_empty());
    }
}
