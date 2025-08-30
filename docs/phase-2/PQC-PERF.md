# PQC Performance & Constant-Time Validation System

## Overview

The PQC Performance & Constant-Time Validation System provides comprehensive micro-benchmarking and statistical analysis for post-quantum cryptography algorithms. It ensures both performance targets are met and timing attacks are prevented through rigorous constant-time validation.

## System Architecture

### Core Components

1. **Performance Benchmark Harness** (`perf/pqc/harness.rs`)
   - Micro-benchmark suite for Dilithium2 and Kyber768 operations
   - Precise cycle counting using RDTSC/RDTSCP instructions
   - Statistical analysis and performance target validation
   - JSON export for CI integration and trend analysis

2. **Constant-Time Validation Engine** (`perf/pqc/ct_check.rs`)
   - Statistical validation using Welch's t-test
   - 10,000 sample collection per operation
   - Jitter analysis and timing channel detection
   - Confidence level calculation and reporting

3. **CI Integration Workflows**
   - `phase-2-pqc-perf`: Performance benchmarking and target validation
   - `phase-2-ct`: Constant-time validation and timing channel detection

## Performance Targets

### QEMU Baseline Targets

| Operation | Algorithm | Target (P50) | Description |
|-----------|-----------|--------------|-------------|
| **Sign** | Dilithium2 | ≤1.2ms | Digital signature generation |
| **Verify** | Dilithium2 | ≤1.4ms | Digital signature verification |
| **Encapsulate** | Kyber768 | ≤0.9ms | Key encapsulation mechanism |
| **Decapsulate** | Kyber768 | ≤1.1ms | Key decapsulation mechanism |

### Performance Validation Criteria

- **P50 Latency**: Median operation time must be within target
- **Statistical Significance**: 95% confidence interval
- **Sample Size**: Minimum 10,000 samples per operation
- **Environment**: QEMU baseline with controlled conditions

## Constant-Time Validation

### Statistical Methodology

The system uses **Welch's t-test** to detect potential timing channels:

1. **Sample Collection**: 10,000 samples each for fixed and random inputs
2. **Statistical Analysis**: Welch's t-test with p < 0.01 significance level
3. **Effect Size**: Cohen's d calculation for practical significance
4. **Confidence Level**: Overall confidence based on p-values

### Validation Criteria

- **Significance Level**: p < 0.01 indicates potential timing channel
- **Effect Size**: Cohen's d interpretation for practical impact
- **Sample Quality**: Minimum 100 samples for valid t-test
- **Jitter Analysis**: 1μs threshold for timing variation

### Constant-Time Requirements

- **No Data-Dependent Branches**: Execution path independent of input
- **No Data-Dependent Memory Access**: Cache access patterns consistent
- **No Data-Dependent Arithmetic**: Operation count independent of input
- **No Early Termination**: All operations complete regardless of input

## Usage Instructions

### Performance Benchmarking

#### Standalone Execution

```bash
# Run complete performance benchmark suite
cd perf/pqc
cargo run --bin harness

# Run with custom configuration
cargo run --bin harness -- --sample-size 5000 --duration 60000
```

#### Programmatic Usage

```rust
use perf::pqc::harness::{PqcBenchmarkHarness, BenchmarkConfig};

let config = BenchmarkConfig {
    sample_size: 10000,
    benchmark_duration_ms: 30000,
    enable_cycle_counting: true,
    output_file: Some("results.json".to_string()),
    ..Default::default()
};

let mut harness = PqcBenchmarkHarness::new(config);
let results = harness.run_benchmark_suite();

// Check if all targets were met
let all_targets_met = results.targets_met.values().all(|&met| met);
```

### Constant-Time Validation

#### Standalone Execution

```bash
# Run complete constant-time validation suite
cd perf/pqc
cargo run --bin ct_check

# Run with custom configuration
cargo run --bin ct_check -- --sample-size 15000 --significance 0.005
```

#### Programmatic Usage

```rust
use perf::pqc::ct_check::{CtValidationEngine, CtValidationConfig};

let config = CtValidationConfig {
    sample_size: 15000,
    significance_level: 0.005,
    enable_jitter_analysis: true,
    output_file: Some("ct_results.json".to_string()),
    ..Default::default()
};

let mut engine = CtValidationEngine::new(config);
let summary = engine.run_validation_suite();

// Check if all operations are constant-time
let all_constant_time = summary.constant_time_operations == summary.total_operations;
```

## Configuration Options

### Performance Benchmark Configuration

```rust
pub struct BenchmarkConfig {
    pub warmup_iterations: usize,        // Default: 1000
    pub benchmark_iterations: usize,     // Default: 10000
    pub benchmark_duration_ms: u64,      // Default: 30000 (30s)
    pub sample_size: usize,              // Default: 1000
    pub enable_cycle_counting: bool,     // Default: true
    pub enable_timing_analysis: bool,    // Default: true
    pub output_file: Option<String>,     // Default: None
}
```

### Constant-Time Validation Configuration

```rust
pub struct CtValidationConfig {
    pub sample_size: usize,              // Default: 10000
    pub significance_level: f64,         // Default: 0.01
    pub min_sample_size: usize,          // Default: 100
    pub jitter_threshold_ns: u64,        // Default: 1000 (1μs)
    pub enable_jitter_analysis: bool,    // Default: true
    pub output_file: Option<String>,     // Default: None
    pub detailed_reporting: bool,        // Default: true
}
```

## CI Integration

### GitHub Actions Workflows

#### Phase 2 PQC Performance Job

```yaml
phase-2-pqc-perf:
  name: "PQC Performance Validation"
  runs-on: ubuntu-latest
  steps:
    - name: "Build PQC performance tools"
      run: |
        cd perf/pqc
        cargo build --release --bin harness
        
    - name: "Run performance benchmarks"
      run: |
        cd perf/pqc
        timeout 30s cargo run --release --bin harness
        
    - name: "Validate performance targets"
      run: |
        # Check if all targets were met
        if grep -q "ALL_TARGETS_MET" pqc_performance_results.json; then
          echo "✅ All performance targets met"
        else
          echo "❌ Some performance targets not met"
          exit 1
        fi
```

#### Phase 2 Constant-Time Job

```yaml
phase-2-ct:
  name: "Constant-Time Validation"
  runs-on: ubuntu-latest
  steps:
    - name: "Build CT validation tools"
      run: |
        cd perf/pqc
        cargo build --release --bin ct_check
        
    - name: "Run constant-time validation"
      run: |
        cd perf/pqc
        timeout 60s cargo run --release --bin ct_check
        
    - name: "Check for timing channels"
      run: |
        # Check if any timing channels were detected
        if grep -q "TIMING_CHANNELS_DETECTED" ct_validation_results.json; then
          echo "❌ Potential timing channels detected"
          exit 1
        else
          echo "✅ All operations appear constant-time"
        fi
```

### CI Gates and Validation

#### Performance Gates

- **Target Validation**: All P50 targets must be met
- **Statistical Significance**: 95% confidence in results
- **Sample Quality**: Minimum sample size requirements
- **Environment Consistency**: QEMU baseline validation

#### Constant-Time Gates

- **Statistical Validation**: p < 0.01 significance threshold
- **Effect Size Analysis**: Cohen's d interpretation
- **Jitter Analysis**: 1μs timing variation threshold
- **Confidence Level**: Overall confidence calculation

## Output Formats

### Performance Results JSON

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "commit_hash": "abc1234",
  "benchmark_duration_ms": 30000,
  "total_operations": 40000,
  "algorithm_results": {
    "dilithium2_sign": {
      "fixed": {
        "count": 10000,
        "mean_duration_ns": 1200000,
        "p50_duration_ns": 1200000,
        "p95_duration_ns": 1250000,
        "std_dev_duration_ns": 50000
      }
    }
  },
  "performance_targets": {
    "dilithium2_sign_p50_ms": 1.2,
    "dilithium2_verify_p50_ms": 1.4,
    "kyber768_encapsulate_p50_ms": 0.9,
    "kyber768_decapsulate_p50_ms": 1.1
  },
  "targets_met": {
    "dilithium2_sign_p50": true,
    "dilithium2_verify_p50": true,
    "kyber768_encapsulate_p50": true,
    "kyber768_decapsulate_p50": true
  },
  "overall_status": "✅ ALL_TARGETS_MET"
}
```

### Constant-Time Validation Results JSON

```json
{
  "timestamp": "2024-01-15T10:35:00Z",
  "commit_hash": "abc1234",
  "total_operations": 4,
  "constant_time_operations": 4,
  "potential_timing_channels": 0,
  "operation_results": {
    "dilithium2_sign": {
      "operation": "dilithium2_sign",
      "is_constant_time": true,
      "confidence_level": 0.9995,
      "t_test_result": {
        "p_value": 0.1234,
        "is_significant": false,
        "effect_size": 0.15,
        "interpretation": "Negligible effect size"
      }
    }
  },
  "overall_status": "✅ ALL_OPERATIONS_CONSTANT_TIME",
  "confidence_level": 0.9995
}
```

## Statistical Analysis Details

### Welch's t-test Implementation

The system implements Welch's t-test for comparing timing distributions:

1. **Test Statistic**: t = (μ₁ - μ₂) / √(s₁²/n₁ + s₂²/n₂)
2. **Degrees of Freedom**: Welch-Satterthwaite equation
3. **P-value Calculation**: Two-tailed test approximation
4. **Effect Size**: Cohen's d = |μ₁ - μ₂| / σ_pooled

### Sample Size Requirements

- **Minimum**: 100 samples for valid t-test
- **Recommended**: 10,000 samples for statistical power
- **Statistical Power**: 95% confidence level
- **Effect Size Detection**: 0.2 Cohen's d minimum

### Jitter Analysis

- **Threshold**: 1μs maximum jitter
- **Measurement**: Consecutive sample differences
- **Analysis**: Mean and maximum jitter calculation
- **Reporting**: High jitter warnings and recommendations

## Performance Optimization

### Cycle Counting Optimization

- **RDTSC Instruction**: High-precision timestamp counter
- **Memory Fencing**: MFENCE/LFENCE for accurate timing
- **CPU Affinity**: Pin to single core for consistency
- **Interrupt Disabling**: Minimize system interference

### Statistical Analysis Optimization

- **Parallel Processing**: Concurrent sample collection
- **Memory Management**: Efficient sample storage
- **Algorithm Optimization**: Optimized statistical calculations
- **Caching**: Result caching for repeated analysis

## Security Considerations

### Timing Attack Prevention

- **Constant-Time Implementation**: No data-dependent branches
- **Memory Access Patterns**: Consistent cache behavior
- **Arithmetic Operations**: Fixed operation count
- **Branch Prediction**: No predictable execution paths

### Validation Security

- **Statistical Rigor**: Proper significance testing
- **Sample Independence**: Uncorrelated timing samples
- **Environment Control**: Consistent testing conditions
- **Result Verification**: Multiple validation approaches

## Troubleshooting

### Common Issues

#### Performance Target Failures

1. **Check Environment**: Ensure QEMU baseline conditions
2. **Verify Implementation**: Check algorithm implementation
3. **Review Configuration**: Validate benchmark parameters
4. **Analyze Results**: Examine detailed performance data

#### Constant-Time Validation Failures

1. **Review Implementation**: Check for data-dependent code
2. **Analyze Statistics**: Examine p-values and effect sizes
3. **Check Environment**: Ensure consistent testing conditions
4. **Validate Samples**: Verify sample quality and independence

#### Build Failures

1. **Dependencies**: Ensure all required crates are available
2. **Toolchain**: Verify Rust toolchain compatibility
3. **Architecture**: Check target architecture support
4. **Permissions**: Verify file system access permissions

### Debug Mode

Enable detailed debugging:

```rust
let config = BenchmarkConfig {
    detailed_reporting: true,
    enable_timing_analysis: true,
    ..Default::default()
};
```

## Future Enhancements

### Planned Features

1. **Advanced Statistical Analysis**
   - Multiple comparison corrections
   - Non-parametric tests
   - Bayesian analysis methods

2. **Performance Monitoring**
   - Real-time performance tracking
   - Regression detection
   - Automated alerting

3. **Enhanced Validation**
   - Power analysis attacks
   - Cache timing analysis
   - Side-channel resistance

4. **CI Integration**
   - Automated trend analysis
   - Performance regression gates
   - Historical data management

### Integration Points

1. **Performance Baselines**: Integration with CI performance gates
2. **Security Validation**: Integration with security scanning tools
3. **Documentation**: Automated report generation
4. **Monitoring**: Real-time performance dashboards

## Conclusion

The PQC Performance & Constant-Time Validation System provides comprehensive validation of both performance characteristics and security properties for post-quantum cryptography implementations. Through rigorous statistical analysis and performance benchmarking, it ensures that PQC algorithms meet both performance targets and constant-time security requirements.

**Key Benefits**:
- **Performance Assurance**: Validates performance targets are met
- **Security Validation**: Detects potential timing channels
- **Statistical Rigor**: Proper statistical analysis methodology
- **CI Integration**: Automated validation in development pipeline
- **Comprehensive Reporting**: Detailed results and recommendations

**Next Steps**:
1. **Integration**: Integrate with existing CI/CD pipelines
2. **Validation**: Run validation on actual PQC implementations
3. **Monitoring**: Establish continuous performance monitoring
4. **Enhancement**: Implement additional validation methods

The system provides a solid foundation for ensuring both performance and security requirements are met in post-quantum cryptography implementations, enabling confident deployment of PQC algorithms in production environments.
