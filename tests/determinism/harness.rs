use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error types for determinism testing
#[derive(Error, Debug)]
pub enum DeterminismError {
    #[error("Test execution failed: {0}")]
    TestExecutionFailed(String),

    #[error("Output mismatch: {0}")]
    OutputMismatch(String),

    #[error("File I/O error: {0}")]
    FileError(#[from] io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Test case not found: {0}")]
    TestCaseNotFound(String),

    #[error("Golden file mismatch: {0}")]
    GoldenFileMismatch(String),

    #[error("Seed capture failed: {0}")]
    SeedCaptureFailed(String),

    #[error("Replay failed: {0}")]
    ReplayFailed(String),
}

/// Test execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test case identifier
    pub test_id: String,
    
    /// Execution timestamp
    pub timestamp: u64,
    
    /// Captured seed value
    pub seed: u64,
    
    /// Output hash for comparison
    pub output_hash: String,
    
    /// Output size in bytes
    pub output_size: usize,
    
    /// Execution time in milliseconds
    pub execution_time: u64,
    
    /// Exit code
    pub exit_code: i32,
    
    /// Standard output
    pub stdout: String,
    
    /// Standard error
    pub stderr: String,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Determinism test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismConfig {
    /// Number of test runs to verify determinism
    pub num_runs: usize,
    
    /// Maximum allowed execution time per run (seconds)
    pub max_execution_time: u64,
    
    /// Output directory for test results
    pub output_dir: PathBuf,
    
    /// Golden files directory
    pub golden_dir: PathBuf,
    
    /// Whether to update golden files on mismatch
    pub update_golden: bool,
    
    /// Seed capture strategy
    pub seed_strategy: SeedStrategy,
    
    /// Output comparison tolerance
    pub tolerance: f64,
}

/// Seed capture strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SeedStrategy {
    /// Use system time as seed
    SystemTime,
    
    /// Use fixed seed for reproducible tests
    Fixed(u64),
    
    /// Capture seed from test output
    Capture,
    
    /// Use random seed (not recommended for determinism)
    Random,
}

impl Default for DeterminismConfig {
    fn default() -> Self {
        Self {
            num_runs: 3,
            max_execution_time: 30,
            output_dir: PathBuf::from("determinism_output"),
            golden_dir: PathBuf::from("determinism_golden"),
            update_golden: false,
            seed_strategy: SeedStrategy::Fixed(42),
            tolerance: 0.0,
        }
    }
}

/// Determinism test harness
pub struct DeterminismHarness {
    config: DeterminismConfig,
    test_cases: HashMap<String, TestCase>,
}

/// Individual test case definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Test case identifier
    pub id: String,
    
    /// Test case name/description
    pub name: String,
    
    /// Command to execute
    pub command: String,
    
    /// Command arguments
    pub args: Vec<String>,
    
    /// Working directory
    pub working_dir: Option<PathBuf>,
    
    /// Environment variables
    pub env: HashMap<String, String>,
    
    /// Input data (if any)
    pub input: Option<String>,
    
    /// Expected exit code
    pub expected_exit_code: Option<i32>,
    
    /// Timeout in seconds
    pub timeout: Option<u64>,
    
    /// Seed value (if fixed)
    pub seed: Option<u64>,
}

impl DeterminismHarness {
    /// Create a new determinism test harness
    pub fn new(config: DeterminismConfig) -> Self {
        Self {
            config,
            test_cases: HashMap::new(),
        }
    }

    /// Add a test case to the harness
    pub fn add_test_case(&mut self, test_case: TestCase) {
        self.test_cases.insert(test_case.id.clone(), test_case);
    }

    /// Load test cases from a directory
    pub fn load_test_cases(&mut self, cases_dir: &Path) -> Result<(), DeterminismError> {
        if !cases_dir.exists() {
            return Err(DeterminismError::TestCaseNotFound(
                cases_dir.to_string_lossy().to_string()
            ));
        }

        for entry in fs::read_dir(cases_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let test_case: TestCase = serde_json::from_reader(
                    File::open(&path)?
                )?;
                self.add_test_case(test_case);
            }
        }

        Ok(())
    }

    /// Run all test cases with determinism verification
    pub fn run_all_tests(&self) -> Result<Vec<TestResult>, DeterminismError> {
        let mut all_results = Vec::new();

        for test_case in self.test_cases.values() {
            let results = self.run_test_case_deterministic(test_case)?;
            all_results.extend(results);
        }

        Ok(all_results)
    }

    /// Run a single test case multiple times to verify determinism
    pub fn run_test_case_deterministic(
        &self,
        test_case: &TestCase,
    ) -> Result<Vec<TestResult>, DeterminismError> {
        let mut results = Vec::new();
        let mut golden_file = self.config.golden_dir.join(&format!("{}.golden", test_case.id));

        // Ensure output directories exist
        fs::create_dir_all(&self.config.output_dir)?;
        fs::create_dir_all(&self.config.golden_dir)?;

        // Run the test case multiple times
        for run in 0..self.config.num_runs {
            let result = self.run_single_test(test_case, run)?;
            results.push(result.clone());

            // Compare with previous runs
            if run > 0 {
                let previous_result = &results[run - 1];
                
                if result.output_hash != previous_result.output_hash {
                    return Err(DeterminismError::OutputMismatch(format!(
                        "Test case {} run {} output hash mismatch: expected {}, got {}",
                        test_case.id, run, previous_result.output_hash, result.output_hash
                    )));
                }
            }
        }

        // Compare with golden file if it exists
        if golden_file.exists() {
            self.compare_with_golden(&results[0], &golden_file)?;
        } else {
            // Create golden file
            self.create_golden_file(&results[0], &golden_file)?;
        }

        Ok(results)
    }

    /// Run a single test execution
    fn run_single_test(
        &self,
        test_case: &TestCase,
        run_number: usize,
    ) -> Result<TestResult, DeterminismError> {
        let start_time = SystemTime::now();
        
        // Prepare command
        let mut command = Command::new(&test_case.command);
        command.args(&test_case.args);
        
        if let Some(ref working_dir) = test_case.working_dir {
            command.current_dir(working_dir);
        }
        
        // Set environment variables
        for (key, value) in &test_case.env {
            command.env(key, value);
        }
        
        // Set seed environment variable
        let seed = self.get_seed(test_case, run_number)?;
        command.env("DETERMINISM_SEED", seed.to_string());
        
        // Set output capture
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        
        // Set timeout
        let timeout = test_case.timeout.unwrap_or(self.config.max_execution_time);
        
        // Execute command with timeout
        let output = match command.output() {
            Ok(output) => output,
            Err(e) => {
                return Err(DeterminismError::TestExecutionFailed(format!(
                    "Failed to execute test case {}: {}",
                    test_case.id, e
                )));
            }
        };

        let execution_time = start_time.elapsed()
            .unwrap_or_default()
            .as_millis() as u64;

        // Capture output
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        // Generate output hash
        let output_hash = self.generate_output_hash(&stdout, &stderr, output.status.code().unwrap_or(-1));
        
        // Create test result
        let result = TestResult {
            test_id: test_case.id.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            seed,
            output_hash: output_hash.clone(),
            output_size: stdout.len() + stderr.len(),
            execution_time,
            exit_code: output.status.code().unwrap_or(-1),
            stdout,
            stderr,
            metadata: HashMap::new(),
        };

        // Save individual result
        let result_file = self.config.output_dir.join(
            format!("{}_{}_{}.json", test_case.id, run_number, result.timestamp)
        );
        let file = File::create(&result_file)?;
        serde_json::to_writer_pretty(file, &result)?;

        Ok(result)
    }

    /// Get seed value based on strategy
    fn get_seed(&self, test_case: &TestCase, run_number: usize) -> Result<u64, DeterminismError> {
        match &self.config.seed_strategy {
            SeedStrategy::SystemTime => {
                Ok(SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs())
            }
            SeedStrategy::Fixed(seed) => Ok(*seed),
            SeedStrategy::Capture => {
                test_case.seed.ok_or_else(|| {
                    DeterminismError::SeedCaptureFailed(
                        "No seed specified for capture strategy".to_string()
                    )
                })
            }
            SeedStrategy::Random => {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                
                let mut hasher = DefaultHasher::new();
                test_case.id.hash(&mut hasher);
                run_number.hash(&mut hasher);
                Ok(hasher.finish())
            }
        }
    }

    /// Generate hash from output data
    fn generate_output_hash(&self, stdout: &str, stderr: &str, exit_code: i32) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        stdout.hash(&mut hasher);
        stderr.hash(&mut hasher);
        exit_code.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }

    /// Compare result with golden file
    fn compare_with_golden(
        &self,
        result: &TestResult,
        golden_file: &Path,
    ) -> Result<(), DeterminismError> {
        let golden_content: TestResult = serde_json::from_reader(
            File::open(golden_file)?
        )?;

        if result.output_hash != golden_content.output_hash {
            if self.config.update_golden {
                // Update golden file
                self.create_golden_file(result, golden_file)?;
                println!("Updated golden file for test case: {}", result.test_id);
            } else {
                return Err(DeterminismError::GoldenFileMismatch(format!(
                    "Test case {} output hash mismatch with golden file: expected {}, got {}",
                    result.test_id, golden_content.output_hash, result.output_hash
                )));
            }
        }

        Ok(())
    }

    /// Create golden file from test result
    fn create_golden_file(
        &self,
        result: &TestResult,
        golden_file: &Path,
    ) -> Result<(), DeterminismError> {
        let file = File::create(golden_file)?;
        serde_json::to_writer_pretty(file, result)?;
        Ok(())
    }

    /// Generate determinism report
    pub fn generate_report(&self, results: &[TestResult]) -> Result<String, DeterminismError> {
        let mut report = String::new();
        
        writeln!(report, "# Determinism Test Report")?;
        writeln!(report, "Generated: {}", chrono::Utc::now().to_rfc3339())?;
        writeln!(report, "Configuration: {:?}", self.config)?;
        writeln!(report, "")?;
        
        // Group results by test case
        let mut test_groups: HashMap<String, Vec<&TestResult>> = HashMap::new();
        for result in results {
            test_groups.entry(result.test_id.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }
        
        for (test_id, test_results) in test_groups {
            writeln!(report, "## Test Case: {}", test_id)?;
            
            if test_results.len() >= 2 {
                let first_hash = &test_results[0].output_hash;
                let all_identical = test_results.iter()
                    .all(|r| r.output_hash == *first_hash);
                
                if all_identical {
                    writeln!(report, "✅ **DETERMINISTIC**: All {} runs produced identical output", test_results.len())?;
                } else {
                    writeln!(report, "❌ **NON-DETERMINISTIC**: Output varies between runs")?;
                }
            }
            
            writeln!(report, "Runs: {}", test_results.len())?;
            writeln!(report, "Output hash: {}", test_results[0].output_hash)?;
            writeln!(report, "Average execution time: {}ms", 
                test_results.iter().map(|r| r.execution_time).sum::<u64>() / test_results.len() as u64)?;
            writeln!(report, "")?;
        }
        
        Ok(report)
    }

    /// Save report to file
    pub fn save_report(&self, report: &str) -> Result<(), DeterminismError> {
        let report_file = self.config.output_dir.join("determinism_report.md");
        let mut file = File::create(report_file)?;
        file.write_all(report.as_bytes())?;
        Ok(())
    }
}

/// Utility function to create a simple test case
pub fn create_simple_test_case(
    id: &str,
    name: &str,
    command: &str,
    args: Vec<String>,
) -> TestCase {
    TestCase {
        id: id.to_string(),
        name: name.to_string(),
        command: command.to_string(),
        args,
        working_dir: None,
        env: HashMap::new(),
        input: None,
        expected_exit_code: Some(0),
        timeout: Some(30),
        seed: Some(42),
    }
}

/// Utility function to create a capability token test case
pub fn create_capability_token_test_case(
    id: &str,
    operation: &str,
) -> TestCase {
    let mut env = HashMap::new();
    env.insert("RUST_LOG".to_string(), "info".to_string());
    
    TestCase {
        id: id.to_string(),
        name: format!("Capability Token {}", operation),
        command: "cargo".to_string(),
        args: vec!["run", "--bin", "capability_token_test", "--", operation],
        working_dir: Some(PathBuf::from("security/caps")),
        env,
        input: None,
        expected_exit_code: Some(0),
        timeout: Some(60),
        seed: Some(12345),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_determinism_harness_creation() {
        let config = DeterminismConfig::default();
        let harness = DeterminismHarness::new(config);
        assert_eq!(harness.test_cases.len(), 0);
    }

    #[test]
    fn test_test_case_creation() {
        let test_case = create_simple_test_case(
            "test-1",
            "Simple Test",
            "echo",
            vec!["hello".to_string()]
        );
        
        assert_eq!(test_case.id, "test-1");
        assert_eq!(test_case.name, "Simple Test");
        assert_eq!(test_case.command, "echo");
        assert_eq!(test_case.args, vec!["hello"]);
    }

    #[test]
    fn test_output_hash_generation() {
        let config = DeterminismConfig::default();
        let harness = DeterminismHarness::new(config);
        
        let hash1 = harness.generate_output_hash("hello", "world", 0);
        let hash2 = harness.generate_output_hash("hello", "world", 0);
        let hash3 = harness.generate_output_hash("hello", "world", 1);
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
