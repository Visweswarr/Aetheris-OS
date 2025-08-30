use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use determinism_harness::{
    DeterminismHarness, DeterminismConfig, SeedStrategy, TestCase, TestResult,
    create_simple_test_case, create_capability_token_test_case
};

#[test]
fn test_harness_creation() {
    let config = DeterminismConfig::default();
    let harness = DeterminismHarness::new(config);
    
    assert_eq!(harness.test_cases.len(), 0);
}

#[test]
fn test_add_test_case() {
    let mut harness = DeterminismHarness::new(DeterminismConfig::default());
    
    let test_case = create_simple_test_case(
        "test-1",
        "Simple Test",
        "echo",
        vec!["hello".to_string()]
    );
    
    harness.add_test_case(test_case);
    assert_eq!(harness.test_cases.len(), 1);
    
    let added_case = harness.test_cases.get("test-1").unwrap();
    assert_eq!(added_case.id, "test-1");
    assert_eq!(added_case.name, "Simple Test");
    assert_eq!(added_case.command, "echo");
}

#[test]
fn test_output_hash_generation() {
    let config = DeterminismConfig::default();
    let harness = DeterminismHarness::new(config);
    
    // Same inputs should produce same hash
    let hash1 = harness.generate_output_hash("hello", "world", 0);
    let hash2 = harness.generate_output_hash("hello", "world", 0);
    assert_eq!(hash1, hash2);
    
    // Different inputs should produce different hashes
    let hash3 = harness.generate_output_hash("hello", "world", 1);
    assert_ne!(hash1, hash3);
    
    let hash4 = harness.generate_output_hash("goodbye", "world", 0);
    assert_ne!(hash1, hash4);
}

#[test]
fn test_seed_strategies() {
    let mut config = DeterminismConfig::default();
    let harness = DeterminismHarness::new(config.clone());
    
    let test_case = create_simple_test_case("test", "Test", "echo", vec![]);
    
    // Test fixed seed strategy
    config.seed_strategy = SeedStrategy::Fixed(12345);
    let harness_fixed = DeterminismHarness::new(config.clone());
    let seed1 = harness_fixed.get_seed(&test_case, 0).unwrap();
    let seed2 = harness_fixed.get_seed(&test_case, 1).unwrap();
    assert_eq!(seed1, 12345);
    assert_eq!(seed2, 12345);
    
    // Test system time strategy (should be different for different calls)
    config.seed_strategy = SeedStrategy::SystemTime;
    let harness_time = DeterminismHarness::new(config.clone());
    let seed3 = harness_time.get_seed(&test_case, 0).unwrap();
    let seed4 = harness_time.get_seed(&test_case, 0).unwrap();
    // Note: These might be the same if called very quickly, but that's acceptable
    
    // Test random strategy (should be different for different calls)
    config.seed_strategy = SeedStrategy::Random;
    let harness_random = DeterminismHarness::new(config);
    let seed5 = harness_random.get_seed(&test_case, 0).unwrap();
    let seed6 = harness_random.get_seed(&test_case, 1).unwrap();
    assert_ne!(seed5, seed6);
}

#[test]
fn test_test_case_creation_utilities() {
    let simple_case = create_simple_test_case(
        "simple",
        "Simple Test",
        "ls",
        vec!["-la".to_string()]
    );
    
    assert_eq!(simple_case.id, "simple");
    assert_eq!(simple_case.name, "Simple Test");
    assert_eq!(simple_case.command, "ls");
    assert_eq!(simple_case.args, vec!["-la"]);
    assert_eq!(simple_case.expected_exit_code, Some(0));
    assert_eq!(simple_case.timeout, Some(30));
    assert_eq!(simple_case.seed, Some(42));
    
    let capability_case = create_capability_token_test_case(
        "cap-test",
        "sign"
    );
    
    assert_eq!(capability_case.id, "cap-test");
    assert_eq!(capability_case.name, "Capability Token sign");
    assert_eq!(capability_case.command, "cargo");
    assert_eq!(capability_case.working_dir, Some(PathBuf::from("security/caps")));
    assert_eq!(capability_case.timeout, Some(60));
    assert_eq!(capability_case.seed, Some(12345));
}

#[test]
fn test_golden_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let config = DeterminismConfig {
        output_dir: temp_dir.path().join("output"),
        golden_dir: temp_dir.path().join("golden"),
        ..Default::default()
    };
    
    let harness = DeterminismHarness::new(config);
    
    // Create a test result
    let mut metadata = HashMap::new();
    metadata.insert("test_key".to_string(), "test_value".to_string());
    
    let result = TestResult {
        test_id: "test-1".to_string(),
        timestamp: 1234567890,
        seed: 42,
        output_hash: "abc123".to_string(),
        output_size: 100,
        execution_time: 50,
        exit_code: 0,
        stdout: "Hello, World!".to_string(),
        stderr: "".to_string(),
        metadata,
    };
    
    // Test creating golden file
    let golden_file = temp_dir.path().join("golden").join("test-1.golden");
    harness.create_golden_file(&result, &golden_file).unwrap();
    assert!(golden_file.exists());
    
    // Test comparing with golden file
    harness.compare_with_golden(&result, &golden_file).unwrap();
    
    // Test mismatch detection
    let different_result = TestResult {
        output_hash: "def456".to_string(),
        ..result.clone()
    };
    
    let result = harness.compare_with_golden(&different_result, &golden_file);
    assert!(result.is_err());
    match result {
        Err(determinism_harness::DeterminismError::GoldenFileMismatch(_)) => {},
        _ => panic!("Expected GoldenFileMismatch error"),
    }
}

#[test]
fn test_report_generation() {
    let config = DeterminismConfig::default();
    let harness = DeterminismHarness::new(config);
    
    let mut metadata = HashMap::new();
    metadata.insert("test_key".to_string(), "test_value".to_string());
    
    let results = vec![
        TestResult {
            test_id: "test-1".to_string(),
            timestamp: 1234567890,
            seed: 42,
            output_hash: "abc123".to_string(),
            output_size: 100,
            execution_time: 50,
            exit_code: 0,
            stdout: "Hello".to_string(),
            stderr: "".to_string(),
            metadata: metadata.clone(),
        },
        TestResult {
            test_id: "test-1".to_string(),
            timestamp: 1234567891,
            seed: 42,
            output_hash: "abc123".to_string(),
            output_size: 100,
            execution_time: 51,
            exit_code: 0,
            stdout: "Hello".to_string(),
            stderr: "".to_string(),
            metadata: metadata.clone(),
        },
        TestResult {
            test_id: "test-2".to_string(),
            timestamp: 1234567892,
            seed: 43,
            output_hash: "def456".to_string(),
            output_size: 200,
            execution_time: 100,
            exit_code: 0,
            stdout: "World".to_string(),
            stderr: "".to_string(),
            metadata,
        },
    ];
    
    let report = harness.generate_report(&results).unwrap();
    
    // Check that report contains expected content
    assert!(report.contains("# Determinism Test Report"));
    assert!(report.contains("## Test Case: test-1"));
    assert!(report.contains("## Test Case: test-2"));
    assert!(report.contains("✅ **DETERMINISTIC**: All 2 runs produced identical output"));
    assert!(report.contains("Output hash: abc123"));
    assert!(report.contains("Output hash: def456"));
}

#[test]
fn test_config_defaults() {
    let config = DeterminismConfig::default();
    
    assert_eq!(config.num_runs, 3);
    assert_eq!(config.max_execution_time, 30);
    assert_eq!(config.output_dir, PathBuf::from("determinism_output"));
    assert_eq!(config.golden_dir, PathBuf::from("determinism_golden"));
    assert_eq!(config.update_golden, false);
    assert_eq!(config.tolerance, 0.0);
    
    match config.seed_strategy {
        SeedStrategy::Fixed(42) => {},
        _ => panic!("Expected Fixed(42) seed strategy"),
    }
}

#[test]
fn test_test_case_serialization() {
    let test_case = create_simple_test_case(
        "serialization-test",
        "Serialization Test",
        "echo",
        vec!["test".to_string()]
    );
    
    // Test serialization
    let serialized = serde_json::to_string(&test_case).unwrap();
    assert!(serialized.contains("serialization-test"));
    assert!(serialized.contains("Serialization Test"));
    assert!(serialized.contains("echo"));
    
    // Test deserialization
    let deserialized: TestCase = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.id, test_case.id);
    assert_eq!(deserialized.name, test_case.name);
    assert_eq!(deserialized.command, test_case.command);
    assert_eq!(deserialized.args, test_case.args);
}

#[test]
fn test_test_result_serialization() {
    let mut metadata = HashMap::new();
    metadata.insert("key1".to_string(), "value1".to_string());
    metadata.insert("key2".to_string(), "value2".to_string());
    
    let result = TestResult {
        test_id: "test-1".to_string(),
        timestamp: 1234567890,
        seed: 42,
        output_hash: "abc123".to_string(),
        output_size: 100,
        execution_time: 50,
        exit_code: 0,
        stdout: "Hello, World!".to_string(),
        stderr: "Error message".to_string(),
        metadata,
    };
    
    // Test serialization
    let serialized = serde_json::to_string(&result).unwrap();
    assert!(serialized.contains("test-1"));
    assert!(serialized.contains("abc123"));
    assert!(serialized.contains("Hello, World!"));
    assert!(serialized.contains("Error message"));
    
    // Test deserialization
    let deserialized: TestResult = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.test_id, result.test_id);
    assert_eq!(deserialized.output_hash, result.output_hash);
    assert_eq!(deserialized.stdout, result.stdout);
    assert_eq!(deserialized.stderr, result.stderr);
    assert_eq!(deserialized.metadata.len(), result.metadata.len());
}

#[test]
fn test_error_types() {
    use determinism_harness::DeterminismError;
    
    // Test error creation and display
    let test_error = DeterminismError::TestExecutionFailed("Test failed".to_string());
    assert!(test_error.to_string().contains("Test execution failed: Test failed"));
    
    let output_error = DeterminismError::OutputMismatch("Hash mismatch".to_string());
    assert!(output_error.to_string().contains("Output mismatch: Hash mismatch"));
    
    let file_error = DeterminismError::FileError(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "File not found"
    ));
    assert!(file_error.to_string().contains("File I/O error"));
}

#[test]
fn test_harness_integration() {
    let temp_dir = TempDir::new().unwrap();
    let config = DeterminismConfig {
        output_dir: temp_dir.path().join("output"),
        golden_dir: temp_dir.path().join("golden"),
        num_runs: 2,
        ..Default::default()
    };
    
    let mut harness = DeterminismHarness::new(config);
    
    // Add test cases
    let test_case1 = create_simple_test_case(
        "integration-test-1",
        "Integration Test 1",
        "echo",
        vec!["test1".to_string()]
    );
    
    let test_case2 = create_simple_test_case(
        "integration-test-2",
        "Integration Test 2",
        "echo",
        vec!["test2".to_string()]
    );
    
    harness.add_test_case(test_case1);
    harness.add_test_case(test_case2);
    
    assert_eq!(harness.test_cases.len(), 2);
    
    // Test that we can access test cases
    let case1 = harness.test_cases.get("integration-test-1").unwrap();
    assert_eq!(case1.name, "Integration Test 1");
    
    let case2 = harness.test_cases.get("integration-test-2").unwrap();
    assert_eq!(case2.name, "Integration Test 2");
}

