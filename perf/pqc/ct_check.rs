use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// Constant-time validation configuration
const SAMPLE_SIZE: usize = 10000; // N=10k samples per operation
const SIGNIFICANCE_LEVEL: f64 = 0.01; // p < 0.01 for statistical significance
const MIN_SAMPLE_SIZE: usize = 100; // Minimum samples for valid t-test
const JITTER_THRESHOLD_NS: u64 = 1000; // 1μs jitter threshold

/// Constant-time validation result for a single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtValidationResult {
    pub operation: String,
    pub fixed_input_samples: Vec<u64>, // Duration samples for fixed input
    pub random_input_samples: Vec<u64>, // Duration samples for random input
    pub fixed_input_stats: SampleStats,
    pub random_input_stats: SampleStats,
    pub t_test_result: TTestResult,
    pub is_constant_time: bool,
    pub confidence_level: f64,
    pub timestamp: String,
}

/// Statistical summary for a sample set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleStats {
    pub count: usize,
    pub mean_ns: f64,
    pub median_ns: u64,
    pub std_dev_ns: f64,
    pub min_ns: u64,
    pub max_ns: u64,
    pub p50_ns: u64,
    pub p95_ns: u64,
    pub p99_ns: u64,
}

/// Welch's t-test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTestResult {
    pub t_statistic: f64,
    pub degrees_of_freedom: f64,
    pub p_value: f64,
    pub is_significant: bool,
    pub effect_size: f64, // Cohen's d
    pub interpretation: String,
}

/// Overall constant-time validation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtValidationSummary {
    pub timestamp: String,
    pub commit_hash: String,
    pub total_operations: usize,
    pub constant_time_operations: usize,
    pub potential_timing_channels: usize,
    pub operation_results: HashMap<String, CtValidationResult>,
    pub overall_status: String,
    pub confidence_level: f64,
}

/// Constant-time validation configuration
#[derive(Debug, Clone)]
pub struct CtValidationConfig {
    pub sample_size: usize,
    pub significance_level: f64,
    pub min_sample_size: usize,
    pub jitter_threshold_ns: u64,
    pub enable_jitter_analysis: bool,
    pub output_file: Option<String>,
    pub detailed_reporting: bool,
}

impl Default for CtValidationConfig {
    fn default() -> Self {
        Self {
            sample_size: SAMPLE_SIZE,
            significance_level: SIGNIFICANCE_LEVEL,
            min_sample_size: MIN_SAMPLE_SIZE,
            jitter_threshold_ns: JITTER_THRESHOLD_NS,
            enable_jitter_analysis: true,
            output_file: None,
            detailed_reporting: true,
        }
    }
}

/// Constant-Time Validation Engine
pub struct CtValidationEngine {
    config: CtValidationConfig,
    results: HashMap<String, CtValidationResult>,
}

impl CtValidationEngine {
    /// Create a new constant-time validation engine
    pub fn new(config: CtValidationConfig) -> Self {
        Self {
            config,
            results: HashMap::new(),
        }
    }

    /// Run constant-time validation for all PQC operations
    pub fn run_validation_suite(&mut self) -> CtValidationSummary {
        println!("🔒 Starting Constant-Time Validation Suite");
        println!("=========================================");
        println!("Configuration:");
        println!("  Sample size: {}", self.config.sample_size);
        println!("  Significance level: p < {}", self.config.significance_level);
        println!("  Min sample size: {}", self.config.min_sample_size);
        println!("  Jitter threshold: {}ns", self.config.jitter_threshold_ns);
        println!("  Jitter analysis: {}", self.config.enable_jitter_analysis);
        println!("  Detailed reporting: {}", self.config.detailed_reporting);
        println!();

        let start_time = Instant::now();
        let commit_hash = get_commit_hash().unwrap_or_else(|| "unknown".to_string());

        // Validate Dilithium2 operations
        println!("🔐 Validating Dilithium2 constant-time behavior...");
        self.validate_dilithium2_operations();

        // Validate Kyber768 operations
        println!("🔑 Validating Kyber768 constant-time behavior...");
        self.validate_kyber768_operations();

        let validation_duration = start_time.elapsed();
        let total_operations = self.results.len();
        let constant_time_operations = self.results.values().filter(|r| r.is_constant_time).count();
        let potential_timing_channels = total_operations - constant_time_operations;

        // Calculate overall confidence level
        let confidence_level = self.calculate_overall_confidence();
        let overall_status = self.determine_overall_status(constant_time_operations, total_operations);

        // Create summary
        let summary = CtValidationSummary {
            timestamp: chrono::Utc::now().to_rfc3339(),
            commit_hash,
            total_operations,
            constant_time_operations,
            potential_timing_channels,
            operation_results: self.results.clone(),
            overall_status,
            confidence_level,
        };

        // Export results if file specified
        if let Some(ref output_file) = self.config.output_file {
            if let Err(e) = self.export_results(&summary, output_file) {
                eprintln!("Warning: Failed to export results to {}: {}", output_file, e);
            }
        }

        // Print summary
        self.print_summary(&summary);

        summary
    }

    /// Validate Dilithium2 operations for constant-time behavior
    fn validate_dilithium2_operations(&mut self) {
        println!("  📝 Dilithium2 Sign Operation...");
        
        // Generate keypair once for all tests
        let (public_key, secret_key) = self.generate_dilithium2_keypair();
        let fixed_message = b"Fixed message for constant-time validation testing";
        
        // Collect fixed input samples
        let fixed_samples = self.collect_timing_samples(
            "dilithium2_sign",
            "fixed",
            || self.sign_dilithium2(&secret_key, fixed_message),
        );

        // Collect random input samples
        let random_samples = self.collect_timing_samples(
            "dilithium2_sign",
            "random",
            || {
                let random_message = self.generate_random_message(64);
                self.sign_dilithium2(&secret_key, &random_message)
            },
        );

        // Perform statistical analysis
        let result = self.analyze_constant_time_behavior(
            "dilithium2_sign",
            fixed_samples,
            random_samples,
        );

        self.results.insert("dilithium2_sign".to_string(), result);
        println!("  ✅ Dilithium2 Sign validation completed");

        println!("  🔍 Dilithium2 Verify Operation...");
        
        // Create signature for verification
        let signature = self.sign_dilithium2(&secret_key, fixed_message);

        // Collect fixed input samples
        let fixed_verify_samples = self.collect_timing_samples(
            "dilithium2_verify",
            "fixed",
            || self.verify_dilithium2(&public_key, fixed_message, &signature),
        );

        // Collect random input samples
        let random_verify_samples = self.collect_timing_samples(
            "dilithium2_verify",
            "random",
            || {
                let random_message = self.generate_random_message(64);
                self.verify_dilithium2(&public_key, &random_message, &signature)
            },
        );

        // Perform statistical analysis
        let verify_result = self.analyze_constant_time_behavior(
            "dilithium2_verify",
            fixed_verify_samples,
            random_verify_samples,
        );

        self.results.insert("dilithium2_verify".to_string(), verify_result);
        println!("  ✅ Dilithium2 Verify validation completed");
    }

    /// Validate Kyber768 operations for constant-time behavior
    fn validate_kyber768_operations(&mut self) {
        println!("  🔐 Kyber768 Encapsulate Operation...");
        
        // Generate keypair once for all tests
        let (public_key, _secret_key) = self.generate_kyber768_keypair();

        // Collect fixed input samples
        let fixed_samples = self.collect_timing_samples(
            "kyber768_encapsulate",
            "fixed",
            || self.encapsulate_kyber768(&public_key),
        );

        // Collect random input samples
        let random_samples = self.collect_timing_samples(
            "kyber768_encapsulate",
            "random",
            || self.encapsulate_kyber768(&public_key),
        );

        // Perform statistical analysis
        let result = self.analyze_constant_time_behavior(
            "kyber768_encapsulate",
            fixed_samples,
            random_samples,
        );

        self.results.insert("kyber768_encapsulate".to_string(), result);
        println!("  ✅ Kyber768 Encapsulate validation completed");

        println!("  🔓 Kyber768 Decapsulate Operation...");
        
        // Generate keypair for decapsulation
        let (decap_public_key, decap_secret_key) = self.generate_kyber768_keypair();
        let (ciphertext, _shared_secret) = self.encapsulate_kyber768(&decap_public_key);

        // Collect fixed input samples
        let fixed_decap_samples = self.collect_timing_samples(
            "kyber768_decapsulate",
            "fixed",
            || self.decapsulate_kyber768(&decap_secret_key, &ciphertext),
        );

        // Collect random input samples
        let random_decap_samples = self.collect_timing_samples(
            "kyber768_decapsulate",
            "random",
            || {
                let random_ciphertext = self.generate_random_ciphertext();
                self.decapsulate_kyber768(&decap_secret_key, &random_ciphertext)
            },
        );

        // Perform statistical analysis
        let decap_result = self.analyze_constant_time_behavior(
            "kyber768_decapsulate",
            fixed_decap_samples,
            random_decap_samples,
        );

        self.results.insert("kyber768_decapsulate".to_string(), decap_result);
        println!("  ✅ Kyber768 Decapsulate validation completed");
    }

    /// Collect timing samples for a specific operation
    fn collect_timing_samples<F, R>(
        &self,
        operation: &str,
        input_type: &str,
        operation_fn: F,
    ) -> Vec<u64>
    where
        F: Fn() -> R,
    {
        println!("    Collecting {} samples for {} ({})...", self.config.sample_size, operation, input_type);
        
        let mut samples = Vec::with_capacity(self.config.sample_size);
        
        // Warmup phase
        for _ in 0..100 {
            let _ = operation_fn();
        }

        // Collect actual samples
        for i in 0..self.config.sample_size {
            let start_time = Instant::now();
            let _result = operation_fn();
            let duration = start_time.elapsed();
            
            samples.push(duration.as_nanos() as u64);

            // Progress indicator
            if (i + 1) % 1000 == 0 {
                print!("      {} samples collected\r", i + 1);
            }
        }
        println!();

        // Apply jitter analysis if enabled
        if self.config.enable_jitter_analysis {
            self.analyze_jitter(&samples, operation, input_type);
        }

        samples
    }

    /// Analyze jitter in timing samples
    fn analyze_jitter(&self, samples: &[u64], operation: &str, input_type: &str) {
        if samples.len() < 2 {
            return;
        }

        let mut jitter_samples = Vec::new();
        for i in 1..samples.len() {
            let jitter = if samples[i] > samples[i - 1] {
                samples[i] - samples[i - 1]
            } else {
                samples[i - 1] - samples[i]
            };
            jitter_samples.push(jitter);
        }

        let mean_jitter: f64 = jitter_samples.iter().map(|&x| x as f64).sum::<f64>() / jitter_samples.len() as f64;
        let max_jitter = jitter_samples.iter().max().copied().unwrap_or(0);

        if max_jitter > self.config.jitter_threshold_ns {
            println!("    ⚠️  High jitter detected in {} ({}): max={}ns, mean={:.1}ns", 
                operation, input_type, max_jitter, mean_jitter);
        }
    }

    /// Analyze constant-time behavior using statistical tests
    fn analyze_constant_time_behavior(
        &self,
        operation: &str,
        fixed_samples: Vec<u64>,
        random_samples: Vec<u64>,
    ) -> CtValidationResult {
        // Calculate sample statistics
        let fixed_stats = self.calculate_sample_stats(&fixed_samples);
        let random_stats = self.calculate_sample_stats(&random_samples);

        // Perform Welch's t-test
        let t_test_result = self.perform_welch_t_test(&fixed_samples, &random_samples);

        // Determine if operation is constant-time
        let is_constant_time = !t_test_result.is_significant || t_test_result.p_value >= self.config.significance_level;

        // Calculate confidence level
        let confidence_level = 1.0 - t_test_result.p_value;

        CtValidationResult {
            operation: operation.to_string(),
            fixed_input_samples: fixed_samples,
            random_input_samples: random_samples,
            fixed_input_stats: fixed_stats,
            random_input_stats: random_stats,
            t_test_result,
            is_constant_time,
            confidence_level,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Calculate statistical summary for a sample set
    fn calculate_sample_stats(&self, samples: &[u64]) -> SampleStats {
        if samples.is_empty() {
            return SampleStats {
                count: 0,
                mean_ns: 0.0,
                median_ns: 0,
                std_dev_ns: 0.0,
                min_ns: 0,
                max_ns: 0,
                p50_ns: 0,
                p95_ns: 0,
                p99_ns: 0,
            };
        }

        let mut sorted_samples = samples.to_vec();
        sorted_samples.sort_unstable();

        let count = samples.len();
        let min_ns = *sorted_samples.first().unwrap();
        let max_ns = *sorted_samples.last().unwrap();
        let mean_ns = samples.iter().map(|&x| x as f64).sum::<f64>() / count as f64;
        let median_ns = sorted_samples[count / 2];
        let p50_ns = sorted_samples[(count * 50) / 100];
        let p95_ns = sorted_samples[(count * 95) / 100];
        let p99_ns = sorted_samples[(count * 99) / 100];

        // Calculate standard deviation
        let variance = samples
            .iter()
            .map(|&x| (x as f64 - mean_ns).powi(2))
            .sum::<f64>()
            / count as f64;
        let std_dev_ns = variance.sqrt();

        SampleStats {
            count,
            mean_ns,
            median_ns,
            std_dev_ns,
            min_ns,
            max_ns,
            p50_ns,
            p95_ns,
            p99_ns,
        }
    }

    /// Perform Welch's t-test between two sample sets
    fn perform_welch_t_test(&self, sample1: &[u64], sample2: &[u64]) -> TTestResult {
        if sample1.len() < self.config.min_sample_size || sample2.len() < self.config.min_sample_size {
            return TTestResult {
                t_statistic: 0.0,
                degrees_of_freedom: 0.0,
                p_value: 1.0,
                is_significant: false,
                effect_size: 0.0,
                interpretation: "Insufficient samples for valid t-test".to_string(),
            };
        }

        let n1 = sample1.len() as f64;
        let n2 = sample2.len() as f64;

        let mean1 = sample1.iter().map(|&x| x as f64).sum::<f64>() / n1;
        let mean2 = sample2.iter().map(|&x| x as f64).sum::<f64>() / n2;

        let var1 = sample1
            .iter()
            .map(|&x| (x as f64 - mean1).powi(2))
            .sum::<f64>()
            / (n1 - 1.0);
        let var2 = sample2
            .iter()
            .map(|&x| (x as f64 - mean2).powi(2))
            .sum::<f64>()
            / (n2 - 1.0);

        // Welch's t-test statistic
        let t_statistic = (mean1 - mean2) / ((var1 / n1 + var2 / n2).sqrt());

        // Degrees of freedom (Welch-Satterthwaite equation)
        let degrees_of_freedom = (var1 / n1 + var2 / n2).powi(2)
            / ((var1 / n1).powi(2) / (n1 - 1.0) + (var2 / n2).powi(2) / (n2 - 1.0));

        // Calculate p-value (approximate using t-distribution)
        let p_value = self.calculate_p_value(t_statistic.abs(), degrees_of_freedom);
        let is_significant = p_value < self.config.significance_level;

        // Calculate effect size (Cohen's d)
        let pooled_std = ((var1 + var2) / 2.0).sqrt();
        let effect_size = if pooled_std > 0.0 {
            (mean1 - mean2).abs() / pooled_std
        } else {
            0.0
        };

        // Interpret effect size
        let interpretation = self.interpret_effect_size(effect_size);

        TTestResult {
            t_statistic,
            degrees_of_freedom,
            p_value,
            is_significant,
            effect_size,
            interpretation,
        }
    }

    /// Calculate approximate p-value for t-distribution
    fn calculate_p_value(&self, t_statistic: f64, degrees_of_freedom: f64) -> f64 {
        // This is a simplified approximation
        // In practice, you might want to use a proper t-distribution library
        
        if degrees_of_freedom <= 0.0 {
            return 1.0;
        }

        // For large degrees of freedom, approximate with normal distribution
        if degrees_of_freedom > 30.0 {
            // Two-tailed test: P(|T| > |t|)
            let z_score = t_statistic;
            let normal_p = 2.0 * (1.0 - self.normal_cdf(z_score.abs()));
            return normal_p;
        }

        // For smaller degrees of freedom, use approximation
        // This is a rough approximation - consider using proper t-distribution
        let z_score = t_statistic;
        let normal_p = 2.0 * (1.0 - self.normal_cdf(z_score.abs()));
        
        // Adjust for degrees of freedom (rough approximation)
        let adjustment = 1.0 + (1.0 / degrees_of_freedom.sqrt());
        (normal_p * adjustment).min(1.0)
    }

    /// Calculate normal distribution CDF (approximate)
    fn normal_cdf(&self, x: f64) -> f64 {
        // Approximation using error function
        0.5 * (1.0 + self.erf(x / 2.0_f64.sqrt()))
    }

    /// Error function approximation
    fn erf(&self, x: f64) -> f64 {
        // Simple approximation using Taylor series
        let a1 = 0.254829592;
        let a2 = -0.284496736;
        let a3 = 1.421413741;
        let a4 = -1.453152027;
        let a5 = 1.061405429;
        let p = 0.3275911;

        let sign = if x < 0.0 { -1.0 } else { 1.0 };
        let x = x.abs();

        let t = 1.0 / (1.0 + p * x);
        let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

        sign * y
    }

    /// Interpret effect size using Cohen's d
    fn interpret_effect_size(&self, effect_size: f64) -> String {
        if effect_size < 0.2 {
            "Negligible effect size".to_string()
        } else if effect_size < 0.5 {
            "Small effect size".to_string()
        } else if effect_size < 0.8 {
            "Medium effect size".to_string()
        } else {
            "Large effect size".to_string()
        }
    }

    /// Calculate overall confidence level
    fn calculate_overall_confidence(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }

        let total_confidence: f64 = self.results.values().map(|r| r.confidence_level).sum();
        total_confidence / self.results.len() as f64
    }

    /// Determine overall validation status
    fn determine_overall_status(&self, constant_time_ops: usize, total_ops: usize) -> String {
        if total_ops == 0 {
            "❌ NO_OPERATIONS_TESTED".to_string()
        } else if constant_time_ops == total_ops {
            "✅ ALL_OPERATIONS_CONSTANT_TIME".to_string()
        } else if constant_time_ops > total_ops / 2 {
            format!("⚠️  MOSTLY_CONSTANT_TIME ({}/{})", constant_time_ops, total_ops)
        } else {
            format!("❌ TIMING_CHANNELS_DETECTED ({}/{})", constant_time_ops, total_ops)
        }
    }

    /// Export results to JSON file
    fn export_results(&self, summary: &CtValidationSummary, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(summary)?;
        fs::write(filename, json)?;
        println!("📁 Constant-time validation results exported to: {}", filename);
        Ok(())
    }

    /// Print validation summary
    fn print_summary(&self, summary: &CtValidationSummary) {
        println!();
        println!("🔒 Constant-Time Validation Summary");
        println!("===================================");
        println!("Timestamp: {}", summary.timestamp);
        println!("Commit: {}", summary.commit_hash);
        println!("Total Operations: {}", summary.total_operations);
        println!("Constant-Time Operations: {}", summary.constant_time_operations);
        println!("Potential Timing Channels: {}", summary.potential_timing_channels);
        println!("Overall Status: {}", summary.overall_status);
        println!("Confidence Level: {:.2}%", summary.confidence_level * 100.0);
        println!();

        // Print detailed results for each operation
        for (operation, result) in &summary.operation_results {
            println!("🔐 {}:", operation);
            println!("  Constant-Time: {}", if result.is_constant_time { "✅ YES" } else { "❌ NO" });
            println!("  P-Value: {:.6}", result.t_test_result.p_value);
            println!("  Significance: {}", if result.t_test_result.is_significant { "YES" } else { "NO" });
            println!("  Effect Size: {:.4} ({})", result.t_test_result.effect_size, result.t_test_result.interpretation);
            println!("  Confidence: {:.2}%", result.confidence_level * 100.0);
            
            if self.config.detailed_reporting {
                println!("  Fixed Input Stats:");
                println!("    Mean: {:.1}ns, Std Dev: {:.1}ns", 
                    result.fixed_input_stats.mean_ns, result.fixed_input_stats.std_dev_ns);
                println!("    P50: {}ns, P95: {}ns", 
                    result.fixed_input_stats.p50_ns, result.fixed_input_stats.p95_ns);
                
                println!("  Random Input Stats:");
                println!("    Mean: {:.1}ns, Std Dev: {:.1}ns", 
                    result.random_input_stats.mean_ns, result.random_input_stats.std_dev_ns);
                println!("    P50: {}ns, P95: {}ns", 
                    result.random_input_stats.p50_ns, result.random_input_stats.p95_ns);
            }
            println!();
        }

        // Print recommendations
        if summary.potential_timing_channels > 0 {
            println!("⚠️  RECOMMENDATIONS:");
            println!("  - Investigate operations with p < {}", self.config.significance_level);
            println!("  - Review implementation for data-dependent branches");
            println!("  - Consider using constant-time cryptographic libraries");
            println!("  - Add additional timing attack mitigations");
        } else {
            println!("✅ All operations appear to be constant-time!");
            println!("  - No significant timing differences detected");
            println!("  - Implementation appears secure against timing attacks");
        }
    }

    // Stub implementations for PQC operations (these would be replaced with actual implementations)
    
    fn generate_dilithium2_keypair(&self) -> (Vec<u8>, Vec<u8>) {
        (vec![0xAA; 32], vec![0xBB; 64])
    }

    fn sign_dilithium2(&self, _secret_key: &[u8], _message: &[u8]) -> Vec<u8> {
        // Simulate constant-time signing with minimal variation
        std::thread::sleep(Duration::from_micros(1200));
        vec![0xCC; 64]
    }

    fn verify_dilithium2(&self, _public_key: &[u8], _message: &[u8], _signature: &[u8]) -> bool {
        // Simulate constant-time verification with minimal variation
        std::thread::sleep(Duration::from_micros(1400));
        true
    }

    fn generate_kyber768_keypair(&self) -> (Vec<u8>, Vec<u8>) {
        (vec![0xDD; 32], vec![0xEE; 32])
    }

    fn encapsulate_kyber768(&self, _public_key: &[u8]) -> (Vec<u8>, Vec<u8>) {
        // Simulate constant-time encapsulation with minimal variation
        std::thread::sleep(Duration::from_micros(900));
        (vec![0xFF; 64], vec![0x11; 32])
    }

    fn decapsulate_kyber768(&self, _secret_key: &[u8], _ciphertext: &[u8]) -> Vec<u8> {
        // Simulate constant-time decapsulation with minimal variation
        std::thread::sleep(Duration::from_micros(1100));
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
    let config = CtValidationConfig {
        output_file: Some("ct_validation_results.json".to_string()),
        ..Default::default()
    };

    let mut engine = CtValidationEngine::new(config);
    let summary = engine.run_validation_suite();

    // Check if all operations are constant-time
    let all_constant_time = summary.constant_time_operations == summary.total_operations;
    
    if all_constant_time {
        println!("🎉 All operations are constant-time!");
        Ok(())
    } else {
        println!("⚠️  Potential timing channels detected!");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_validation_config_default() {
        let config = CtValidationConfig::default();
        assert_eq!(config.sample_size, SAMPLE_SIZE);
        assert_eq!(config.significance_level, SIGNIFICANCE_LEVEL);
        assert_eq!(config.min_sample_size, MIN_SAMPLE_SIZE);
        assert_eq!(config.jitter_threshold_ns, JITTER_THRESHOLD_NS);
        assert!(config.enable_jitter_analysis);
        assert!(config.detailed_reporting);
    }

    #[test]
    fn test_sample_stats_calculation() {
        let engine = CtValidationEngine::new(CtValidationConfig::default());
        let samples = vec![1000, 2000, 3000, 4000, 5000];
        
        let stats = engine.calculate_sample_stats(&samples);
        
        assert_eq!(stats.count, 5);
        assert_eq!(stats.mean_ns, 3000.0);
        assert_eq!(stats.median_ns, 3000);
        assert_eq!(stats.min_ns, 1000);
        assert_eq!(stats.max_ns, 5000);
        assert_eq!(stats.p50_ns, 3000);
        assert_eq!(stats.p95_ns, 5000);
        assert_eq!(stats.p99_ns, 5000);
    }

    #[test]
    fn test_welch_t_test() {
        let engine = CtValidationEngine::new(CtValidationConfig::default());
        
        // Create two similar sample sets
        let sample1: Vec<u64> = (1000..1100).collect();
        let sample2: Vec<u64> = (1001..1101).collect();
        
        let result = engine.perform_welch_t_test(&sample1, &sample2);
        
        // Should not be significantly different
        assert!(!result.is_significant || result.p_value >= 0.01);
    }

    #[test]
    fn test_effect_size_interpretation() {
        let engine = CtValidationEngine::new(CtValidationConfig::default());
        
        assert_eq!(engine.interpret_effect_size(0.1), "Negligible effect size");
        assert_eq!(engine.interpret_effect_size(0.3), "Small effect size");
        assert_eq!(engine.interpret_effect_size(0.6), "Medium effect size");
        assert_eq!(engine.interpret_effect_size(1.0), "Large effect size");
    }
}
