use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::{tempdir, TempDir};
use serde::{Deserialize, Serialize};

use crate::lib::{WasmModule, WasmConfig, WasmManifest, create_module_from_manifest};

/// Test result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test name
    pub name: String,
    /// Test description
    pub description: String,
    /// Test passed
    pub passed: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time: u64,
    /// Capabilities used
    pub capabilities: Vec<String>,
    /// Paths accessed
    pub paths_accessed: Vec<String>,
}

/// Test suite configuration
#[derive(Debug, Clone)]
pub struct TestSuite {
    /// Test directory
    test_dir: TempDir,
    /// Test results
    results: Vec<TestResult>,
    /// Verbose output
    verbose: bool,
}

impl TestSuite {
    /// Create a new test suite
    pub fn new(verbose: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let test_dir = tempdir()?;
        
        // Create test files
        Self::create_test_files(&test_dir)?;
        
        Ok(Self {
            test_dir,
            results: Vec::new(),
            verbose,
        })
    }
    
    /// Create test files for testing
    fn create_test_files(test_dir: &TempDir) -> Result<(), Box<dyn std::error::Error>> {
        let test_path = test_dir.path();
        
        // Create test file
        let test_file = test_path.join("test.txt");
        let content = "Hello, WASM World!\nThis is a test file for capability testing.\n";
        fs::write(&test_file, content)?;
        
        // Create subdirectory
        let subdir = test_path.join("subdir");
        fs::create_dir(&subdir)?;
        
        // Create file in subdirectory
        let subfile = subdir.join("subfile.txt");
        fs::write(&subfile, "Subdirectory file content\n")?;
        
        // Create restricted file (should be denied)
        let restricted_file = test_path.join("restricted.txt");
        fs::write(&restricted_file, "This file should not be accessible\n")?;
        
        Ok(())
    }
    
    /// Run all tests
    pub fn run_all_tests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Running WASM Capability Tests");
        println!("=================================");
        
        // Test 1: No capabilities
        self.run_no_capabilities_test()?;
        
        // Test 2: Read capability only
        self.run_read_capability_test()?;
        
        // Test 3: Write capability only
        self.run_write_capability_test()?;
        
        // Test 4: Read and write capabilities
        self.run_read_write_capabilities_test()?;
        
        // Test 5: Path restrictions
        self.run_path_restrictions_test()?;
        
        // Test 6: Manifest-based configuration
        self.run_manifest_test()?;
        
        // Test 7: Deny by default vs allow by default
        self.run_policy_comparison_test()?;
        
        // Test 8: Capability escalation prevention
        self.run_capability_escalation_test()?;
        
        // Print summary
        self.print_summary();
        
        Ok(())
    }
    
    /// Test with no capabilities
    fn run_no_capabilities_test(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let test_name = "No Capabilities Test";
        let description = "Test file access with no capabilities granted";
        
        if self.verbose {
            println!("\n📋 {}", test_name);
            println!("   {}", description);
        }
        
        let start_time = std::time::Instant::now();
        
        let config = WasmConfig::default();
        let module = WasmModule::new(config)?;
        
        let test_file = self.test_dir.path().join("test.txt");
        let test_file_str = test_file.to_str().unwrap();
        
        let result = match module.read_file(test_file_str) {
            Ok(_) => {
                TestResult {
                    name: test_name.to_string(),
                    description: description.to_string(),
                    passed: false,
                    error: Some("Unexpected success: Should have failed with no capabilities".to_string()),
                    execution_time: start_time.elapsed().as_millis() as u64,
                    capabilities: vec![],
                    paths_accessed: vec![test_file_str.to_string()],
                }
            }
            Err(e) => {
                TestResult {
                    name: test_name.to_string(),
                    description: description.to_string(),
                    passed: true,
                    error: None,
                    execution_time: start_time.elapsed().as_millis() as u64,
                    capabilities: vec![],
                    paths_accessed: vec![test_file_str.to_string()],
                }
            }
        };
        
        self.results.push(result);
        Ok(())
    }
    
    /// Test with read capability only
    fn run_read_capability_test(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let test_name = "Read Capability Test";
        let description = "Test file access with read capability only";
        
        if self.verbose {
            println!("\n📋 {}", test_name);
            println!("   {}", description);
        }
        
        let start_time = std::time::Instant::now();
        
        let mut config = WasmConfig::default();
        config.file_caps = wasi_common::file::FileCaps::READ;
        config.allowed_paths = vec![self.test_dir.path().to_str().unwrap().to_string()];
        
        let module = WasmModule::new(config)?;
        
        let test_file = self.test_dir.path().join("test.txt");
        let test_file_str = test_file.to_str().unwrap();
        
        let result = match module.read_file(test_file_str) {
            Ok(content) => {
                if content.contains("Hello, WASM World!") {
                    TestResult {
                        name: test_name.to_string(),
                        description: description.to_string(),
                        passed: true,
                        error: None,
                        execution_time: start_time.elapsed().as_millis() as u64,
                        capabilities: vec!["read".to_string()],
                        paths_accessed: vec![test_file_str.to_string()],
                    }
                } else {
                    TestResult {
                        name: test_name.to_string(),
                        description: description.to_string(),
                        passed: false,
                        error: Some("File content mismatch".to_string()),
                        execution_time: start_time.elapsed().as_millis() as u64,
                        capabilities: vec!["read".to_string()],
                        paths_accessed: vec![test_file_str.to_string()],
                    }
                }
            }
            Err(e) => {
                TestResult {
                    name: test_name.to_string(),
                    description: description.to_string(),
                    passed: false,
                    error: Some(format!("Unexpected failure: {}", e)),
                    execution_time: start_time.elapsed().as_millis() as u64,
                    capabilities: vec!["read".to_string()],
                    paths_accessed: vec![test_file_str.to_string()],
                }
            }
        };
        
        self.results.push(result);
        Ok(())
    }
    
    /// Test with write capability only
    fn run_write_capability_test(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let test_name = "Write Capability Test";
        let description = "Test file access with write capability only";
        
        if self.verbose {
            println!("\n📋 {}", test_name);
            println!("   {}", description);
        }
        
        let start_time = std::time::Instant::now();
        
        let mut config = WasmConfig::default();
        config.file_caps = wasi_common::file::FileCaps::WRITE | wasi_common::file::FileCaps::CREATE;
        config.allowed_paths = vec![self.test_dir.path().to_str().unwrap().to_string()];
        
        let module = WasmModule::new(config)?;
        
        let test_file = self.test_dir.path().join("write_test.txt");
        let test_file_str = test_file.to_str().unwrap();
        let test_content = "Write capability test content\n";
        
        let result = match module.write_file(test_file_str, test_content) {
            Ok(_) => {
                // Verify the file was written
                if let Ok(content) = fs::read_to_string(&test_file) {
                    if content == test_content {
                        TestResult {
                            name: test_name.to_string(),
                            description: description.to_string(),
                            passed: true,
                            error: None,
                            execution_time: start_time.elapsed().as_millis() as u64,
                            capabilities: vec!["write".to_string(), "create".to_string()],
                            paths_accessed: vec![test_file_str.to_string()],
                        }
                    } else {
                        TestResult {
                            name: test_name.to_string(),
                            description: description.to_string(),
                            passed: false,
                            error: Some("File content verification failed".to_string()),
                            execution_time: start_time.elapsed().as_millis() as u64,
                            capabilities: vec!["write".to_string(), "create".to_string()],
                            paths_accessed: vec![test_file_str.to_string()],
                        }
                    }
                } else {
                    TestResult {
                        name: test_name.to_string(),
                        description: description.to_string(),
                        passed: false,
                        error: Some("Could not verify written file".to_string()),
                        execution_time: start_time.elapsed().as_millis() as u64,
                        capabilities: vec!["write".to_string(), "create".to_string()],
                        paths_accessed: vec![test_file_str.to_string()],
                    }
                }
            }
            Err(e) => {
                TestResult {
                    name: test_name.to_string(),
                    description: description.to_string(),
                    passed: false,
                    error: Some(format!("Write failed: {}", e)),
                    execution_time: start_time.elapsed().as_millis() as u64,
                    capabilities: vec!["write".to_string(), "create".to_string()],
                    paths_accessed: vec![test_file_str.to_string()],
                }
            }
        };
        
        self.results.push(result);
        Ok(())
    }
    
    /// Test with read and write capabilities
    fn run_read_write_capabilities_test(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let test_name = "Read-Write Capabilities Test";
        let description = "Test file access with both read and write capabilities";
        
        if self.verbose {
            println!("\n📋 {}", test_name);
            println!("   {}", description);
        }
        
        let start_time = std::time::Instant::now();
        
        let mut config = WasmConfig::default();
        config.file_caps = wasi_common::file::FileCaps::READ | wasi_common::file::FileCaps::WRITE | wasi_common::file::FileCaps::CREATE;
        config.allowed_paths = vec![self.test_dir.path().to_str().unwrap().to_string()];
        
        let module = WasmModule::new(config)?;
        
        let test_file = self.test_dir.path().join("rw_test.txt");
        let test_file_str = test_file.to_str().unwrap();
        let test_content = "Read-write capability test content\n";
        
        // Test write
        let write_result = module.write_file(test_file_str, test_content);
        if write_result.is_err() {
            let result = TestResult {
                name: test_name.to_string(),
                description: description.to_string(),
                passed: false,
                error: Some(format!("Write failed: {}", write_result.unwrap_err())),
                execution_time: start_time.elapsed().as_millis() as u64,
                capabilities: vec!["read".to_string(), "write".to_string(), "create".to_string()],
                paths_accessed: vec![test_file_str.to_string()],
            };
            self.results.push(result);
            return Ok(());
        }
        
        // Test read
        let read_result = module.read_file(test_file_str);
        let result = match read_result {
            Ok(content) => {
                if content == test_content {
                    TestResult {
                        name: test_name.to_string(),
                        description: description.to_string(),
                        passed: true,
                        error: None,
                        execution_time: start_time.elapsed().as_millis() as u64,
                        capabilities: vec!["read".to_string(), "write".to_string(), "create".to_string()],
                        paths_accessed: vec![test_file_str.to_string()],
                    }
                } else {
                    TestResult {
                        name: test_name.to_string(),
                        description: description.to_string(),
                        passed: false,
                        error: Some("Read content mismatch".to_string()),
                        execution_time: start_time.elapsed().as_millis() as u64,
                        capabilities: vec!["read".to_string(), "write".to_string(), "create".to_string()],
                        paths_accessed: vec![test_file_str.to_string()],
                    }
                }
            }
            Err(e) => {
                TestResult {
                    name: test_name.to_string(),
                    description: description.to_string(),
                    passed: false,
                    error: Some(format!("Read failed: {}", e)),
                    execution_time: start_time.elapsed().as_millis() as u64,
                    capabilities: vec!["read".to_string(), "write".to_string(), "create".to_string()],
                    paths_accessed: vec![test_file_str.to_string()],
                }
            }
        };
        
        self.results.push(result);
        Ok(())
    }
    
    /// Test path restrictions
    fn run_path_restrictions_test(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let test_name = "Path Restrictions Test";
        let description = "Test that restricted paths are properly denied";
        
        if self.verbose {
            println!("\n📋 {}", test_name);
            println!("   {}", description);
        }
        
        let start_time = std::time::Instant::now();
        
        let mut config = WasmConfig::default();
        config.file_caps = wasi_common::file::FileCaps::READ;
        config.allowed_paths = vec![self.test_dir.path().join("subdir").to_str().unwrap().to_string()];
        
        let module = WasmModule::new(config)?;
        
        // Test allowed path
        let allowed_file = self.test_dir.path().join("subdir").join("subfile.txt");
        let allowed_file_str = allowed_file.to_str().unwrap();
        
        let allowed_result = module.read_file(allowed_file_str);
        if allowed_result.is_err() {
            let result = TestResult {
                name: test_name.to_string(),
                description: description.to_string(),
                passed: false,
                error: Some(format!("Allowed path access failed: {}", allowed_result.unwrap_err())),
                execution_time: start_time.elapsed().as_millis() as u64,
                capabilities: vec!["read".to_string()],
                paths_accessed: vec![allowed_file_str.to_string()],
            };
            self.results.push(result);
            return Ok(());
        }
        
        // Test denied path
        let denied_file = self.test_dir.path().join("restricted.txt");
        let denied_file_str = denied_file.to_str().unwrap();
        
        let denied_result = module.read_file(denied_file_str);
        let result = match denied_result {
            Ok(_) => {
                TestResult {
                    name: test_name.to_string(),
                    description: description.to_string(),
                    passed: false,
                    error: Some("Denied path access succeeded unexpectedly".to_string()),
                    execution_time: start_time.elapsed().as_millis() as u64,
                    capabilities: vec!["read".to_string()],
                    paths_accessed: vec![allowed_file_str.to_string(), denied_file_str.to_string()],
                }
            }
            Err(_) => {
                TestResult {
                    name: test_name.to_string(),
                    description: description.to_string(),
                    passed: true,
                    error: None,
                    execution_time: start_time.elapsed().as_millis() as u64,
                    capabilities: vec!["read".to_string()],
                    paths_accessed: vec![allowed_file_str.to_string(), denied_file_str.to_string()],
                }
            }
        };
        
        self.results.push(result);
        Ok(())
    }
    
    /// Test manifest-based configuration
    fn run_manifest_test(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let test_name = "Manifest Configuration Test";
        let description = "Test module creation from manifest file";
        
        if self.verbose {
            println!("\n📋 {}", test_name);
            println!("   {}", description);
        }
        
        let start_time = std::time::Instant::now();
        
        let manifest = WasmManifest {
            name: "test-module".to_string(),
            version: "1.0.0".to_string(),
            required_caps: vec!["read".to_string(), "write".to_string()],
            allowed_paths: vec![self.test_dir.path().to_str().unwrap().to_string()],
            denied_paths: vec![],
            deny_by_default: true,
            environment: HashMap::new(),
            args: vec![],
        };
        
        let module = create_module_from_manifest(&manifest)?;
        
        // Test that capabilities were set correctly
        let has_read = module.has_capability(wasi_common::file::FileCaps::READ);
        let has_write = module.has_capability(wasi_common::file::FileCaps::WRITE);
        let has_create = module.has_capability(wasi_common::file::FileCaps::CREATE);
        
        let result = if has_read && has_write && !has_create {
            TestResult {
                name: test_name.to_string(),
                description: description.to_string(),
                passed: true,
                error: None,
                execution_time: start_time.elapsed().as_millis() as u64,
                capabilities: vec!["read".to_string(), "write".to_string()],
                paths_accessed: vec![],
            }
        } else {
            TestResult {
                name: test_name.to_string(),
                description: description.to_string(),
                passed: false,
                error: Some(format!("Capability mismatch: read={}, write={}, create={}", has_read, has_write, has_create)),
                execution_time: start_time.elapsed().as_millis() as u64,
                capabilities: vec!["read".to_string(), "write".to_string()],
                paths_accessed: vec![],
            }
        };
        
        self.results.push(result);
        Ok(())
    }
    
    /// Test policy comparison
    fn run_policy_comparison_test(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let test_name = "Policy Comparison Test";
        let description = "Test deny-by-default vs allow-by-default policies";
        
        if self.verbose {
            println!("\n📋 {}", test_name);
            println!("   {}", description);
        }
        
        let start_time = std::time::Instant::now();
        
        // Test deny-by-default
        let mut deny_config = WasmConfig::default();
        deny_config.deny_by_default = true;
        deny_config.allowed_paths = vec![self.test_dir.path().to_str().unwrap().to_string()];
        deny_config.file_caps = wasi_common::file::FileCaps::READ;
        
        let deny_module = WasmModule::new(deny_config)?;
        
        // Test allow-by-default
        let mut allow_config = WasmConfig::default();
        allow_config.deny_by_default = false;
        allow_config.allowed_paths = vec![self.test_dir.path().join("restricted.txt").to_str().unwrap().to_string()];
        allow_config.file_caps = wasi_common::file::FileCaps::READ;
        
        let allow_module = WasmModule::new(allow_config)?;
        
        let test_file = self.test_dir.path().join("test.txt");
        let test_file_str = test_file.to_str().unwrap();
        
        let deny_result = deny_module.read_file(test_file_str);
        let allow_result = allow_module.read_file(test_file_str);
        
        let result = if deny_result.is_ok() && allow_result.is_ok() {
            TestResult {
                name: test_name.to_string(),
                description: description.to_string(),
                passed: true,
                error: None,
                execution_time: start_time.elapsed().as_millis() as u64,
                capabilities: vec!["read".to_string()],
                paths_accessed: vec![test_file_str.to_string()],
            }
        } else {
            TestResult {
                name: test_name.to_string(),
                description: description.to_string(),
                passed: false,
                error: Some(format!("Policy comparison failed: deny={:?}, allow={:?}", deny_result, allow_result)),
                execution_time: start_time.elapsed().as_millis() as u64,
                capabilities: vec!["read".to_string()],
                paths_accessed: vec![test_file_str.to_string()],
            }
        };
        
        self.results.push(result);
        Ok(())
    }
    
    /// Test capability escalation prevention
    fn run_capability_escalation_test(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let test_name = "Capability Escalation Prevention Test";
        let description = "Test that modules cannot escalate their capabilities";
        
        if self.verbose {
            println!("\n📋 {}", test_name);
            println!("   {}", description);
        }
        
        let start_time = std::time::Instant::now();
        
        let mut config = WasmConfig::default();
        config.file_caps = wasi_common::file::FileCaps::READ;
        config.allowed_paths = vec![self.test_dir.path().to_str().unwrap().to_string()];
        
        let module = WasmModule::new(config)?;
        
        // Try to access a file that requires write capability
        let test_file = self.test_dir.path().join("test.txt");
        let test_file_str = test_file.to_str().unwrap();
        
        let result = match module.write_file(test_file_str, "escalation test") {
            Ok(_) => {
                TestResult {
                    name: test_name.to_string(),
                    description: description.to_string(),
                    passed: false,
                    error: Some("Capability escalation succeeded unexpectedly".to_string()),
                    execution_time: start_time.elapsed().as_millis() as u64,
                    capabilities: vec!["read".to_string()],
                    paths_accessed: vec![test_file_str.to_string()],
                }
            }
            Err(_) => {
                TestResult {
                    name: test_name.to_string(),
                    description: description.to_string(),
                    passed: true,
                    error: None,
                    execution_time: start_time.elapsed().as_millis() as u64,
                    capabilities: vec!["read".to_string()],
                    paths_accessed: vec![test_file_str.to_string()],
                }
            }
        };
        
        self.results.push(result);
        Ok(())
    }
    
    /// Print test summary
    fn print_summary(&self) {
        println!("\n📊 Test Summary");
        println!("===============");
        
        let total_tests = self.results.len();
        let passed_tests = self.results.iter().filter(|r| r.passed).count();
        let failed_tests = total_tests - passed_tests;
        
        println!("Total Tests: {}", total_tests);
        println!("Passed: {}", passed_tests);
        println!("Failed: {}", failed_tests);
        println!("Success Rate: {:.1}%", (passed_tests as f64 / total_tests as f64) * 100.0);
        
        if failed_tests > 0 {
            println!("\n❌ Failed Tests:");
            for result in &self.results {
                if !result.passed {
                    println!("  - {}: {}", result.name, result.error.as_ref().unwrap_or(&"Unknown error".to_string()));
                }
            }
        }
        
        println!("\n✅ Passed Tests:");
        for result in &self.results {
            if result.passed {
                println!("  - {} ({}ms)", result.name, result.execution_time);
            }
        }
    }
    
    /// Get test results
    pub fn get_results(&self) -> &[TestResult] {
        &self.results
    }
    
    /// Export results to JSON
    pub fn export_results(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.results)?;
        fs::write(path, json)?;
        Ok(())
    }
}

/// Run the host test harness
pub fn run_host_harness(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut test_suite = TestSuite::new(verbose)?;
    test_suite.run_all_tests()?;
    
    // Export results
    test_suite.export_results("test_results.json")?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_test_suite_creation() {
        let test_suite = TestSuite::new(false);
        assert!(test_suite.is_ok());
    }
    
    #[test]
    fn test_test_result_creation() {
        let result = TestResult {
            name: "test".to_string(),
            description: "test description".to_string(),
            passed: true,
            error: None,
            execution_time: 100,
            capabilities: vec!["read".to_string()],
            paths_accessed: vec!["/tmp/test.txt".to_string()],
        };
        
        assert_eq!(result.name, "test");
        assert!(result.passed);
        assert_eq!(result.execution_time, 100);
    }
}
