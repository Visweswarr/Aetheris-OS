# 🔒 Polymera OS Determinism Test Suite

A comprehensive testing framework to ensure that Polymera OS components produce identical outputs given the same inputs and seeds. This is crucial for security, reproducibility, and compliance requirements.

## 🎯 Overview

The Determinism Test Suite provides:

- **🔍 Seed/Time Capture**: Captures and controls all sources of randomness
- **🔄 Replay Testing**: Runs tests multiple times to verify identical outputs
- **📊 Golden File Comparison**: Compares results against known-good baselines
- **🚨 Violation Detection**: Flags any non-deterministic behavior
- **📈 Comprehensive Reporting**: Detailed analysis of test results

## 🏗️ Architecture

```
tests/determinism/
├── harness.rs              # Core determinism testing harness
├── main.rs                 # CLI application for running tests
├── Cargo.toml              # Rust dependencies and build configuration
├── cases/                  # Test case definitions (JSON)
│   ├── capability_token_format.json
│   ├── capability_token_sign.json
│   ├── capability_token_verify.json
│   ├── json_parser.json
│   └── base64_encoding.json
├── tests/                  # Unit tests for the harness
│   └── harness_tests.rs
├── run_tests.sh            # Shell script for easy test execution
└── README.md               # This documentation
```

## 🚀 Quick Start

### Prerequisites

- **Rust**: `cargo`, `rustc` (stable channel)
- **System**: `bash`, `timeout` command

### Installation

1. **Navigate to the determinism test directory**:
   ```bash
   cd tests/determinism
   ```

2. **Build the determinism runner**:
   ```bash
   cargo build --release
   ```

3. **Make the test runner executable**:
   ```bash
   chmod +x run_tests.sh
   ```

4. **Create sample test data**:
   ```bash
   ./run_tests.sh setup
   ```

5. **Run all determinism tests**:
   ```bash
   ./run_tests.sh run
   ```

## 📋 Usage

### Command Line Interface

The determinism test suite provides a comprehensive CLI:

```bash
# Run all tests
./run_tests.sh run

# Run specific test case
./run_tests.sh test capability_token_format

# List available test cases
./run_tests.sh list

# Generate report from existing results
./run_tests.sh report

# Show test results summary
./run_tests.sh summary

# Create sample test data
./run_tests.sh setup
```

### Direct Binary Usage

```bash
# Build the runner
cargo build --release

# Run all tests
./target/release/determinism-runner run

# Run specific test
./target/release/determinism-runner test capability_token_format

# List test cases
./target/release/determinism-runner list

# Create new test case
./target/release/determinism-runner create-case my-test "My Test" "echo" "hello world"
```

### Configuration Options

```bash
# Custom number of runs
./run_tests.sh -n 5 run

# Custom timeout
./run_tests.sh -t 600 run

# Use different seed strategy
./target/release/determinism-runner --seed-strategy fixed --fixed-seed 12345 run

# Update golden files on mismatch
./target/release/determinism-runner --update-golden run
```

## 🎯 Test Cases

### Capability Token Tests

#### Format Determinism
- **ID**: `capability_token_format`
- **Purpose**: Tests JSON formatting consistency
- **Command**: `cargo run --bin capability_token_format_test -- format`
- **Input**: Sample capability token JSON
- **Expected**: Identical formatted output across runs

#### Signing Determinism
- **ID**: `capability_token_sign`
- **Purpose**: Tests cryptographic signing consistency
- **Command**: `cargo run --bin capability_token_sign_test -- sign`
- **Input**: Unsigned capability token
- **Expected**: Identical signature across runs with same seed

#### Verification Determinism
- **ID**: `capability_token_verify`
- **Purpose**: Tests verification logic consistency
- **Command**: `cargo run --bin capability_token_verify_test -- verify`
- **Input**: Signed capability token
- **Expected**: Identical verification results across runs

### Parser Tests

#### JSON Parser
- **ID**: `json_parser`
- **Purpose**: Tests JSON parsing consistency
- **Command**: `cargo run --bin json_parser_test -- parse`
- **Input**: Complex JSON with various data types
- **Expected**: Identical parsed output across runs

#### Base64 Encoding
- **ID**: `base64_encoding`
- **Purpose**: Tests base64 encoding consistency
- **Command**: `cargo run --bin base64_test -- encode`
- **Input**: Binary data
- **Expected**: Identical encoded output across runs

## ⚙️ Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DETERMINISM_SEED` | Global seed for all tests | `42` |
| `RUST_LOG` | Logging level | `info` |
| `CAPABILITY_TOKEN_SEED` | Seed for capability token tests | `12345` |
| `CRYPTO_SEED` | Seed for cryptographic operations | `67890` |
| `VERIFICATION_SEED` | Seed for verification tests | `11111` |

### Seed Strategies

#### Fixed Seed (Default)
- **Strategy**: `--seed-strategy fixed --fixed-seed 42`
- **Use Case**: Reproducible testing, CI/CD
- **Advantage**: Consistent results across environments

#### System Time
- **Strategy**: `--seed-strategy system-time`
- **Use Case**: Testing time-dependent behavior
- **Advantage**: Catches time-related non-determinism

#### Capture
- **Strategy**: `--seed-strategy capture`
- **Use Case**: Tests that generate their own seeds
- **Advantage**: Tests actual seed generation logic

#### Random
- **Strategy**: `--seed-strategy random`
- **Use Case**: Stress testing (not recommended for determinism)
- **Advantage**: Catches seed-dependent bugs

## 📊 Understanding Results

### Output Structure

```
tests/determinism/
├── output/                 # Test execution results
│   ├── capability_token_format_0_1234567890.json
│   ├── capability_token_format_1_1234567890.json
│   └── capability_token_format_2_1234567890.json
├── golden/                 # Golden file baselines
│   ├── capability_token_format.golden
│   ├── capability_token_sign.golden
│   └── capability_token_verify.golden
└── determinism_report.md   # Generated report
```

### Result Interpretation

#### ✅ Deterministic Results
```
✅ **DETERMINISTIC**: All 3 runs produced identical output
Runs: 3
Output hash: abc123def456
Average execution time: 45ms
```

#### ❌ Non-Deterministic Results
```
❌ **NON-DETERMINISTIC**: Output varies between runs
Run 1: hash=abc123, exit_code=0, time=45ms
Run 2: hash=def456, exit_code=0, time=46ms
Run 3: hash=ghi789, exit_code=0, time=44ms
```

### Golden File Management

Golden files serve as baselines for expected behavior:

```bash
# Create golden files from first run
./run_tests.sh run

# Update golden files on mismatch
./target/release/determinism-runner --update-golden run

# Compare against golden files
./target/release/determinism-runner --golden-dir golden report
```

## 🔧 Troubleshooting

### Common Issues

#### Build Failures
```bash
# Check Rust toolchain
rustup show

# Update dependencies
cargo update

# Clean and rebuild
cargo clean && cargo build --release
```

#### Test Execution Failures
```bash
# Check test data exists
ls -la test_data/

# Verify test cases
./run_tests.sh list

# Run with verbose output
RUST_LOG=debug ./run_tests.sh test capability_token_format
```

#### Non-Deterministic Results

1. **Check for time dependencies**:
   ```bash
   # Use fixed seed strategy
   ./target/release/determinism-runner --seed-strategy fixed --fixed-seed 42 run
   ```

2. **Verify environment consistency**:
   ```bash
   # Check environment variables
   env | grep -i seed
   env | grep -i time
   ```

3. **Increase test runs**:
   ```bash
   # Run more iterations
   ./run_tests.sh -n 10 run
   ```

### Performance Optimization

#### Parallel Execution
```bash
# Run multiple tests in parallel
cargo test --release --jobs 4

# Use parallel test runner
./run_tests.sh -n 5 -t 300 run &
./run_tests.sh -n 5 -t 300 run &
wait
```

#### Resource Limits
```bash
# Limit memory usage
ulimit -v 2097152  # 2GB

# Limit CPU time
ulimit -t 3600     # 1 hour

# Run with resource monitoring
time ./run_tests.sh run
```

## 🔒 Security Considerations

### Determinism in Security Contexts

- **Cryptographic Operations**: Must be deterministic for key derivation
- **Token Generation**: Consistent output prevents timing attacks
- **Audit Trails**: Reproducible results enable forensic analysis
- **Compliance**: Regulatory requirements often mandate determinism

### Best Practices

1. **Seed Management**: Use cryptographically secure seeds
2. **Environment Isolation**: Ensure consistent test environment
3. **Result Validation**: Verify outputs against known baselines
4. **Continuous Testing**: Integrate into CI/CD pipeline

## 📈 Advanced Usage

### Custom Test Cases

#### Creating New Test Cases
```json
{
  "id": "my_custom_test",
  "name": "My Custom Determinism Test",
  "command": "cargo",
  "args": ["run", "--bin", "my_test", "--", "test-command"],
  "working_dir": "../../my_component",
  "env": {
    "CUSTOM_SEED": "98765",
    "TEST_MODE": "deterministic"
  },
  "expected_exit_code": 0,
  "timeout": 60,
  "seed": 98765
}
```

#### Programmatic Test Creation
```rust
use determinism_harness::{DeterminismHarness, TestCase};

let mut harness = DeterminismHarness::new(config);

let test_case = TestCase {
    id: "programmatic-test".to_string(),
    name: "Programmatic Test".to_string(),
    command: "echo".to_string(),
    args: vec!["test".to_string()],
    // ... other fields
};

harness.add_test_case(test_case);
```

### Integration with CI/CD

#### GitHub Actions Example
```yaml
- name: Run Determinism Tests
  run: |
    cd tests/determinism
    ./run_tests.sh -n 5 run
    
- name: Upload Test Results
  uses: actions/upload-artifact@v3
  with:
    name: determinism-results
    path: tests/determinism/output/
    
- name: Check for Violations
  run: |
    cd tests/determinism
    ./run_tests.sh summary
    # Exit with error if violations found
```

#### Jenkins Pipeline Example
```groovy
stage('Determinism Tests') {
    steps {
        sh '''
            cd tests/determinism
            ./run_tests.sh -n 3 run
        '''
    }
    post {
        always {
            archiveArtifacts artifacts: 'tests/determinism/output/**/*'
        }
        failure {
            sh '''
                cd tests/determinism
                ./run_tests.sh summary
            '''
        }
    }
}
```

## 📚 API Reference

### Core Types

#### `DeterminismHarness`
```rust
pub struct DeterminismHarness {
    pub config: DeterminismConfig,
    pub test_cases: HashMap<String, TestCase>,
}

impl DeterminismHarness {
    pub fn new(config: DeterminismConfig) -> Self;
    pub fn add_test_case(&mut self, test_case: TestCase);
    pub fn run_all_tests(&self) -> Result<Vec<TestResult>, DeterminismError>;
    pub fn run_test_case_deterministic(&self, test_case: &TestCase) -> Result<Vec<TestResult>, DeterminismError>;
}
```

#### `TestCase`
```rust
pub struct TestCase {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
    pub env: HashMap<String, String>,
    pub input: Option<String>,
    pub expected_exit_code: Option<i32>,
    pub timeout: Option<u64>,
    pub seed: Option<u64>,
}
```

#### `TestResult`
```rust
pub struct TestResult {
    pub test_id: String,
    pub timestamp: u64,
    pub seed: u64,
    pub output_hash: String,
    pub output_size: usize,
    pub execution_time: u64,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub metadata: HashMap<String, String>,
}
```

### Error Types

```rust
pub enum DeterminismError {
    TestExecutionFailed(String),
    OutputMismatch(String),
    FileError(io::Error),
    SerializationError(serde_json::Error),
    TestCaseNotFound(String),
    GoldenFileMismatch(String),
    SeedCaptureFailed(String),
    ReplayFailed(String),
}
```

## 🤝 Contributing

### Adding New Test Cases

1. **Create test case JSON** in `cases/` directory
2. **Add test data** in `test_data/` directory
3. **Update test runner** if needed
4. **Add documentation** for new test case
5. **Verify determinism** with multiple runs

### Extending the Harness

1. **Add new seed strategies** to `SeedStrategy` enum
2. **Implement new comparison methods** for specialized outputs
3. **Add support for new test types** (e.g., network, database)
4. **Enhance reporting** with additional metrics

### Testing the Harness

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test harness_tests

# Run with coverage
cargo tarpaulin --out Html
```

## 📄 License

This determinism test suite is part of Polymera OS and follows the same licensing terms.

---

**🔒 Happy Determinism Testing!** 🔒

For questions, issues, or contributions, please refer to the main Polymera OS documentation or submit an issue in the repository.

