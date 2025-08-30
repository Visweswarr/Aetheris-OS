# PR Annotations for Regressions & Failing Specs

## Overview

The PR Annotations system provides immediate, inline feedback on Pull Requests when performance budgets or ABI conformance fail in the Phase 2 matrix. This system automatically analyzes performance and test results, compares them against baselines, and emits GitHub Actions annotations that appear directly on the PR diff.

## Features

- **Immediate Feedback**: Inline annotations on PR diffs for performance regressions and test failures
- **Baseline Comparison**: Automatic comparison against established performance baselines
- **Smart Coalescing**: Intelligent grouping of annotations to stay under GitHub's limits
- **Comprehensive Coverage**: Performance metrics, test results, and conformance validation
- **Actionable Information**: Clear error messages with specific metrics and deltas

## Annotation Types

### Performance Annotations

#### Error Level (Performance Regressions)
- **IPC Latency p50**: >15% increase vs baseline
- **IPC Latency p99**: >25% increase vs baseline
- **Wake-to-Run p50**: >12% increase vs baseline
- **Dilithium Signature**: >10% increase vs baseline

#### Warning Level (Performance Warnings)
- **IPC Latency p95**: >20% increase vs baseline
- **Kyber Encapsulation**: >8% increase vs baseline
- **Memory Usage**: >10% increase vs baseline

#### Notice Level (Performance Improvements)
- **IPC Latency p50**: >15% decrease vs baseline
- **Wake-to-Run p50**: >12% decrease vs baseline

### Test Conformance Annotations

#### Error Level (Test Failures)
- **Test Suite Failures**: Complete test suite failures
- **Individual Test Failures**: Specific test failures with error details
- **Configuration Failures**: Failures in specific matrix configurations

#### Warning Level (Test Warnings)
- **Test Suite Warnings**: Test suites with warning status
- **Performance Warnings**: Tests that pass but have performance concerns

## Annotation Format

### GitHub Actions Annotation Syntax

```bash
::error file=path/to/file,line=1,title=Title::Message
::warning file=path/to/file,line=1,title=Title::Message
::notice file=path/to/file,line=1,title=Title::Message
```

### Annotation Parameters

- **level**: `error`, `warning`, or `notice`
- **file**: Path to the relevant file (optional)
- **line**: Line number for the annotation (optional)
- **endLine**: End line for multi-line annotations (optional)
- **title**: Short title for the annotation (optional)
- **message**: Detailed message describing the issue

### Example Annotations

```bash
# Performance regression
::error file=perf/ipc_latency.json,line=1,title=Performance Regression::IPC latency p50 regression: 180ns (+20.0% vs baseline 150ns)

# Test failure
::error file=tests/core_tests.log,line=1,title=Test Failure::Test suite 'core_tests' failed in configuration apic_hpet_jitter_auth_open

# Performance warning
::warning file=perf/memory_usage.json,line=1,title=Memory Usage Warning::Kernel heap usage increased: 13.2MB (+5.6% vs baseline 12.5MB)

# Performance improvement
::notice file=perf/ipc_latency.json,line=1,title=Performance Improvement::IPC latency p50 improvement: 130ns (-13.3% vs baseline 150ns)
```

## Usage

### Command Line Interface

```bash
# Basic usage
node tooling/ci/annotate.js [perf_file] [tests_file] [baseline_file] [config_id]

# Examples
node tooling/ci/annotate.js perf/out.json tests/out.json perf/baselines/p2.json apic_hpet_jitter_auth_open
node tooling/ci/annotate.js perf/ipc_latency.json tests/core_tests.json
node tooling/ci/annotate.js perf/results/matrix/config/metrics.json tests/test_results_summary.json perf/baselines/p2.json config_id
```

### Environment Variables

- **GITHUB_OUTPUT**: Set to enable GitHub Actions outputs
- **MAX_ANNOTATIONS**: Override default limit (default: 50)
- **ANNOTATION_COALESCING**: Enable/disable coalescing (default: true)

### GitHub Actions Integration

```yaml
- name: Annotate PR with performance and test results
  id: annotate_pr
  run: |
    node tooling/ci/annotate.js \
      "perf/results/matrix/${{ matrix.config.config_id }}/metrics.json" \
      "tests/test_results_summary.json" \
      "perf/baselines/p2.json" \
      "${{ matrix.config.config_id }}"
```

## Performance Analysis

### Metrics Analyzed

#### IPC Latency
- **p50**: 50th percentile latency
- **p95**: 95th percentile latency
- **p99**: 99th percentile latency
- **Mean**: Average latency
- **Standard Deviation**: Latency variability

#### Wake-to-Run Latency
- **p50**: 50th percentile wake-to-run time
- **p95**: 95th percentile wake-to-run time
- **p99**: 99th percentile wake-to-run time

#### PQC Overhead
- **Kyber Encapsulation**: Key encapsulation time
- **Dilithium Sign**: Digital signature generation time
- **Dilithium Verify**: Digital signature verification time
- **MAC Generation**: Message authentication code generation
- **MAC Verification**: Message authentication code verification

#### Memory Usage
- **Kernel Heap**: Kernel memory allocation
- **User Heap**: User space memory allocation
- **Stack Usage**: Stack memory consumption

### Thresholds

#### Error Thresholds (Regressions)
- **IPC Latency p50**: >15% increase
- **IPC Latency p99**: >25% increase
- **Wake-to-Run p50**: >12% increase
- **Dilithium Signature**: >10% increase

#### Warning Thresholds
- **IPC Latency p95**: >20% increase
- **Kyber Encapsulation**: >8% increase
- **Memory Usage**: >10% increase

#### Improvement Thresholds
- **IPC Latency p50**: >15% decrease
- **Wake-to-Run p50**: >12% decrease

## Test Conformance Analysis

### Test Status Mapping

- **PASS**: Test passed successfully
- **FAIL**: Test failed (generates error annotation)
- **WARN**: Test has warnings (generates warning annotation)
- **UNKNOWN**: Test status unknown (no annotation)

### Test Categories

- **Core Tests**: Basic functionality tests
- **Timer Tests**: APIC and HPET timer tests
- **Auth Tests**: Authentication system tests
- **Policy Tests**: Capability policy tests
- **QEMU Integration**: Full system integration tests

## Annotation Coalescing

### Purpose

GitHub Actions has a limit of 50 annotations per job. The coalescing system intelligently groups related annotations to stay under this limit while preserving the most important information.

### Coalescing Strategy

1. **Priority Order**: Errors > Warnings > Notices
2. **File Grouping**: Group annotations by file and level
3. **Smart Summarization**: Combine multiple similar annotations
4. **Preserve Context**: Maintain file and line information

### Example Coalescing

```bash
# Before coalescing (3 separate annotations)
::error file=perf/ipc_latency.json,line=1::IPC latency p50 regression: +15.2%
::error file=perf/ipc_latency.json,line=1::IPC latency p95 regression: +18.7%
::error file=perf/ipc_latency.json,line=1::IPC latency p99 regression: +22.1%

# After coalescing (1 combined annotation)
::error file=perf/ipc_latency.json,line=1,title=Multiple Errors (3)::3 errors in ipc_latency.json: IPC latency p50 regression, IPC latency p95 regression, IPC latency p99 regression...
```

## Output and Reporting

### Console Output

The annotation script provides detailed console output including:

- **Data Loading Status**: Confirmation of loaded files
- **Analysis Results**: Summary of detected issues
- **Annotation Count**: Number of annotations emitted
- **Performance Summary**: Regression and improvement counts
- **Test Summary**: Failure and warning counts

### GitHub Actions Outputs

When run in GitHub Actions, the script sets the following outputs:

- **perf_regressions**: Number of performance regressions
- **perf_improvements**: Number of performance improvements
- **perf_warnings**: Number of performance warnings
- **test_failures**: Number of test failures
- **test_warnings**: Number of test warnings
- **total_annotations**: Total number of annotations emitted
- **has_regressions**: Boolean indicating if any regressions were detected

### Example Output

```bash
🔍 PR Annotations for Regressions & Failing Specs

✅ Loaded performance data from: perf/results/matrix/config/metrics.json
✅ Loaded test data from: tests/test_results_summary.json
✅ Loaded baseline data from: perf/baselines/p2.json

📊 Emitting 5 annotations:

::error file=perf/ipc_latency.json,line=1,title=Performance Regression::IPC latency p50 regression: 180ns (+20.0% vs baseline 150ns)
::error file=tests/core_tests.log,line=1,title=Test Failure::Test suite 'core_tests' failed in configuration apic_hpet_jitter_auth_open
::warning file=perf/memory_usage.json,line=1,title=Memory Usage Warning::Kernel heap usage increased: 13.2MB (+5.6% vs baseline 12.5MB)

📈 Analysis Summary:
Performance:
  - Regressions: 1
  - Improvements: 0
  - Warnings: 1
  - Total Metrics: 16
Tests:
  - Failures: 1
  - Warnings: 0
  - Total Tests: 6
Annotations:
  - Emitted: 5
  - Limit: 50
  - Coalescing: Enabled

❌ Regressions detected!
```

## Integration with Phase 2 Matrix

### Workflow Integration

The annotation system is integrated into the Phase 2 matrix workflow:

1. **Test Execution**: Tests run and results are collected
2. **Performance Collection**: Performance metrics are gathered
3. **Annotation Analysis**: Results are analyzed against baselines
4. **PR Annotation**: Annotations are emitted for immediate feedback
5. **Flaky Detection**: Flaky test detection runs if needed

### Artifact Integration

Annotations reference specific artifacts and files:

- **Performance Metrics**: `perf/results/matrix/{config_id}/metrics.json`
- **Test Results**: `tests/test_results_summary.json`
- **Baseline Data**: `perf/baselines/p2.json`
- **Log Files**: Individual test and performance log files

## Best Practices

### Annotation Content

- **Be Specific**: Include exact metrics and deltas
- **Provide Context**: Reference baseline values and thresholds
- **Actionable**: Suggest what needs to be investigated
- **Concise**: Keep messages under 200 characters when possible

### Threshold Management

- **Conservative Thresholds**: Start with conservative thresholds and adjust based on data
- **Context-Aware**: Consider different thresholds for different metrics
- **Baseline Stability**: Ensure baselines are stable before setting strict thresholds
- **Regular Review**: Periodically review and adjust thresholds

### Performance Monitoring

- **Trend Analysis**: Monitor performance trends over time
- **Regression Detection**: Set up alerts for performance regressions
- **Improvement Tracking**: Track and celebrate performance improvements
- **Baseline Updates**: Regularly update baselines with new performance data

## Troubleshooting

### Common Issues

#### No Annotations Generated
- Check if input files exist and are valid JSON
- Verify baseline data contains the expected configuration
- Check console output for error messages

#### Too Many Annotations
- Enable annotation coalescing
- Increase MAX_ANNOTATIONS limit
- Review annotation thresholds

#### Missing Baseline Data
- Ensure baseline file exists and is accessible
- Verify configuration ID matches baseline data
- Check baseline file format and structure

#### Performance Data Issues
- Validate performance data format
- Check metric names and units
- Verify data collection is working correctly

### Debug Mode

Enable debug output by setting environment variables:

```bash
export DEBUG=true
export VERBOSE=true
node tooling/ci/annotate.js [files...]
```

### Validation

Test the annotation system locally:

```bash
# Test with sample data
node tooling/ci/annotate.js test_data/perf.json test_data/tests.json test_data/baseline.json test_config

# Validate annotation format
node tooling/ci/annotate.js --validate-only test_data/perf.json
```

## Future Enhancements

### Planned Features

1. **Machine Learning Integration**: ML-based regression detection
2. **Advanced Coalescing**: Smarter annotation grouping algorithms
3. **Custom Thresholds**: Per-repository threshold configuration
4. **Historical Analysis**: Long-term performance trend analysis
5. **Automated Fixes**: Suggestions for performance improvements

### Integration Improvements

1. **IDE Integration**: Local annotation preview
2. **Real-time Monitoring**: Live performance monitoring
3. **Advanced Reporting**: Enhanced visualization and reporting
4. **Team Dashboards**: Team-specific performance dashboards

## Conclusion

The PR Annotations system provides immediate, actionable feedback on performance regressions and test failures, enabling developers to quickly identify and address issues before they impact the codebase. By integrating seamlessly with the Phase 2 matrix workflow and providing comprehensive analysis, the system improves code quality and maintains performance standards across all configurations.
