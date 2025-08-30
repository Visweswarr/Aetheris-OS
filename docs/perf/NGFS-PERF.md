# NGFS Performance Documentation

## Overview

This document explains how to work with NGFS performance benchmarks, interpret results, manage baselines, and integrate with the existing baseline-promotion workflow.

## Performance Data Format

### Benchmark Output JSON

The NGFS benchmark produces one-line JSON output for CI parsing:

```json
{
  "test": "ngfs_bench",
  "timestamp": 1640995200,
  "duration_ms": 1500,
  "cases": {
    "resolve_path": {
      "p50": 150,
      "p95": 400,
      "p99": 600,
      "mean": 180,
      "stdev": 45,
      "n": 50,
      "errors": 0
    },
    "stat_file": {
      "p50": 80,
      "p95": 250,
      "p99": 350,
      "mean": 95,
      "stdev": 30,
      "n": 50,
      "errors": 0
    },
    "read_small": {
      "p50": 200,
      "p95": 600,
      "p99": 800,
      "mean": 220,
      "stdev": 55,
      "n": 50,
      "errors": 0
    },
    "read_large": {
      "p50": 800,
      "p95": 1500,
      "p99": 2000,
      "mean": 850,
      "stdev": 120,
      "n": 50,
      "errors": 0
    },
    "snapshot_build": {
      "p50": 25,
      "p95": 50,
      "p99": 75,
      "mean": 28,
      "stdev": 8,
      "n": 50,
      "errors": 0
    }
  },
  "errors": 0,
  "mismatch": 0
}
```

### Field Descriptions

- **test**: Always "ngfs_bench" for NGFS benchmarks
- **timestamp**: Unix timestamp when benchmark started
- **duration_ms**: Total benchmark execution time in milliseconds
- **cases**: Performance data for each benchmark case
- **errors**: Total number of errors across all cases
- **mismatch**: Number of cases with unexpected results

### Case Metrics

Each case provides:
- **p50**: 50th percentile (median) in microseconds
- **p95**: 95th percentile in microseconds
- **p99**: 99th percentile in microseconds
- **mean**: Arithmetic mean in microseconds
- **stdev**: Standard deviation in microseconds
- **n**: Number of samples collected
- **errors**: Number of errors for this case

## Performance Baselines

### Baseline File Structure

Performance baselines are stored in `perf/baselines/p3_ngfs.json`:

```json
{
  "version": 1,
  "description": "NGFS v1 Performance Baseline - Phase 3",
  "created": "2024-01-01T00:00:00Z",
  "environment": "QEMU",
  "budgets": {
    "resolve_path": {
      "p95_max": 400,
      "variance_limit": 0.35,
      "min_samples": 50,
      "description": "Path resolution performance"
    },
    "stat_file": {
      "p95_max": 400,
      "variance_limit": 0.35,
      "min_samples": 50,
      "description": "File stat operation performance"
    },
    "read_small": {
      "p95_max": 600,
      "variance_limit": 0.25,
      "min_samples": 50,
      "description": "Small read operations (≤4 KiB)"
    },
    "read_large": {
      "p95_max": 1500,
      "variance_limit": 0.25,
      "min_samples": 50,
      "description": "Large read operations (≥1 MiB)"
    },
    "snapshot_build": {
      "p95_max": 50,
      "variance_limit": 0.35,
      "min_samples": 50,
      "description": "Snapshot building performance"
    }
  },
  "variance_limits": {
    "read_operations": 0.25,
    "metadata_operations": 0.35,
    "snapshot_operations": 0.35
  },
  "sample_requirements": {
    "min_iterations": 50,
    "warmup_iterations": 5,
    "outlier_removal": {
      "enabled": true,
      "threshold": 0.35,
      "max_removed": 1
    }
  }
}
```

### Budget Fields

- **p95_max**: Maximum allowed 95th percentile in microseconds
- **variance_limit**: Maximum allowed stdev/mean ratio
- **min_samples**: Minimum required sample count
- **description**: Human-readable description of the case

### Variance Limits

- **Read Operations**: 0.25 (25%) - Tight variance for I/O operations
- **Metadata Operations**: 0.35 (35%) - Moderate variance for path resolution
- **Snapshot Operations**: 0.35 (35%) - Moderate variance for complex operations

## Reading Performance Results

### Performance Table

The TypeScript renderer produces human-readable tables:

```markdown
## NGFS Performance Benchmarks

**Test:** ngfs_bench | **Duration:** 1500ms | **Errors:** 0 | **Mismatch:** 0

| Case | P50 (μs) | P95 (μs) | P99 (μs) | Mean (μs) | Stdev | N | Errors |
|------|-----------|-----------|-----------|-----------|-------|----|--------|
| resolve_path | 150 | 400 | 600 | 180 | 45 | 50 | 0 |
| stat_file | 80 | 250 | 350 | 95 | 30 | 50 | 0 |
| read_small | 200 | 600 | 800 | 220 | 55 | 50 | 0 |
| read_large | 800 | 1500 | 2000 | 850 | 120 | 50 | 0 |
| snapshot_build | 25 | 50 | 75 | 28 | 8 | 50 | 0 |

### Performance Summary

✅ **All cases within budget**

### Variance Analysis

✅ **resolve_path**: stdev/mean = 0.25
✅ **stat_file**: stdev/mean = 0.32
✅ **read_small**: stdev/mean = 0.25
✅ **read_large**: stdev/mean = 0.14
✅ **snapshot_build**: stdev/mean = 0.29
```

### Interpreting Results

#### ✅ Pass Cases
- **P95 within budget**: Performance meets requirements
- **Variance acceptable**: Statistical stability confirmed
- **Sufficient samples**: N ≥ 50 for statistical validity
- **No errors**: All operations completed successfully

#### ❌ Fail Cases
- **P95 exceeds budget**: Performance below requirements
- **High variance**: Statistical instability detected
- **Insufficient samples**: N < 50 for statistical validity
- **Errors detected**: Some operations failed

#### ⚠️ Warning Cases
- **High variance**: Approaching variance limits
- **Low samples**: N close to minimum requirement
- **Performance degradation**: P95 approaching budget limits

## Baseline Management

### Adjusting Baselines

#### Performance Improvements
When performance improves significantly:

1. **Update budgets**: Lower p95_max values
2. **Tighten variance**: Reduce variance_limit values
3. **Document changes**: Update description and created date
4. **Version increment**: Increment version number

```json
{
  "version": 2,
  "created": "2024-02-01T00:00:00Z",
  "budgets": {
    "resolve_path": {
      "p95_max": 300,  // Reduced from 400
      "variance_limit": 0.30,  // Tightened from 0.35
      "description": "Path resolution performance (improved)"
    }
  }
}
```

#### Performance Regressions
When performance degrades:

1. **Investigate cause**: Identify root cause of regression
2. **Temporary adjustment**: Increase budgets temporarily
3. **Performance investigation**: Profile and optimize code
4. **Baseline restoration**: Restore original budgets after fix

### Baseline Promotion

#### Integration with P2.5 Tool

The NGFS performance baselines integrate with the existing baseline-promotion workflow:

1. **Baseline validation**: P2.5 tool validates baseline structure
2. **Performance gates**: CI ensures baselines are met
3. **Promotion criteria**: Performance must be stable and within budgets
4. **Version tracking**: Baselines versioned with code releases

#### Promotion Process

```bash
# Validate current baseline
python3 tooling/python/ngfs_perf_analyze.py --input bench.json --baseline perf/baselines/p3_ngfs.json

# Check baseline promotion criteria
bazel run //perf:check_baseline_promotion -- perf/baselines/p3_ngfs.json

# Promote baseline if criteria met
bazel run //perf:promote_baseline -- perf/baselines/p3_ngfs.json
```

### Environment-Specific Baselines

#### QEMU CI Environment
- **Current baseline**: Optimized for QEMU virtualization
- **Resource constraints**: Limited CPU and memory
- **Timing characteristics**: Virtualized timing with overhead

#### Local Development
- **Hardware variation**: Different CPU architectures and speeds
- **OS differences**: Linux, macOS, Windows variations
- **Resource availability**: Full hardware resources

#### Production Deployment
- **Real hardware**: Native performance characteristics
- **Network effects**: Real network latency and bandwidth
- **Load conditions**: Production workload patterns

## Performance History

### History File Format

Performance history is stored in `perf/history/p3_ngfs.jsonl` (JSON Lines format):

```json
{"timestamp": "2024-01-01T12:00:00Z", "test": "ngfs_bench", "duration_ms": 1500, "cases": {...}, "errors": 0, "mismatch": 0}
{"timestamp": "2024-01-02T12:00:00Z", "test": "ngfs_bench", "duration_ms": 1450, "cases": {...}, "errors": 0, "mismatch": 0}
{"timestamp": "2024-01-03T12:00:00Z", "test": "ngfs_bench", "duration_ms": 1550, "cases": {...}, "errors": 0, "mismatch": 0}
```

### Trend Analysis

#### Performance Trends
- **Improvement**: Decreasing P95 values over time
- **Stability**: Consistent performance within variance limits
- **Regressions**: Sudden performance degradation
- **Seasonal effects**: Performance variations by time of day/week

#### Statistical Analysis
```python
import json
import pandas as pd
from pathlib import Path

# Load history data
history_file = Path("perf/history/p3_ngfs.jsonl")
history_data = []

with open(history_file, 'r') as f:
    for line in f:
        history_data.append(json.loads(line.strip()))

# Convert to DataFrame
df = pd.DataFrame(history_data)

# Analyze trends
for case in ['resolve_path', 'read_small', 'read_large']:
    case_data = df['cases'].apply(lambda x: x.get(case, {}).get('p95', None))
    print(f"{case} P95 trend: {case_data.mean():.0f}μs ± {case_data.std():.0f}μs")
```

## Troubleshooting

### Common Issues

#### High Variance
**Symptoms**: stdev/mean > variance_limit
**Causes**: 
- System load variations
- JIT compilation effects
- Memory pressure
- Network latency (for network operations)

**Solutions**:
- Increase warm-up iterations
- Pin thread affinity
- Reduce system load during testing
- Check for memory leaks

#### Performance Regressions
**Symptoms**: P95 exceeds budget
**Causes**:
- Code changes affecting performance
- Dependency updates
- System configuration changes
- Hardware degradation

**Solutions**:
- Profile code for bottlenecks
- Revert recent changes
- Check system configuration
- Validate hardware health

#### Insufficient Samples
**Symptoms**: N < 50
**Causes**:
- Benchmark timeouts
- Resource exhaustion
- Test failures
- Configuration errors

**Solutions**:
- Increase timeout values
- Check resource limits
- Fix failing tests
- Validate configuration

### Debugging Commands

#### Performance Analysis
```bash
# Run benchmarks with verbose output
./bazel-bin/go/tools/ngfs-bench --root <CID> --mount /ro/bench --fixtures tests/ngfs/fixtures --json --verbose

# Analyze specific case
python3 tooling/python/ngfs_perf_analyze.py --input bench.json --baseline perf/baselines/p3_ngfs.json --case resolve_path

# Generate detailed report
node bazel-bin/tooling/ts/render-perf.js bench.json --detailed
```

#### System Investigation
```bash
# Check system load
top -n 1
iostat -x 1 5

# Monitor memory usage
free -h
cat /proc/meminfo

# Check CPU frequency
cat /proc/cpuinfo | grep MHz
```

## Future Enhancements

### Phase 3+ Features

#### Advanced Metrics
- **Memory profiling**: Heap usage and allocation patterns
- **CPU profiling**: Instruction-level performance analysis
- **I/O profiling**: Disk and network I/O patterns
- **Power profiling**: Energy consumption metrics

#### Machine Learning
- **Regression detection**: Automated performance regression identification
- **Anomaly detection**: Statistical outlier identification
- **Performance prediction**: ML-based performance forecasting
- **Optimization suggestions**: AI-powered optimization recommendations

#### Cross-Platform
- **Windows support**: Windows performance validation
- **macOS support**: macOS performance validation
- **ARM support**: ARM64 performance validation
- **Container support**: Docker and Kubernetes performance

### Integration Improvements

#### CI/CD Integration
- **Performance gates**: Automated performance regression prevention
- **Baseline automation**: Automatic baseline updates
- **Performance dashboards**: Real-time performance monitoring
- **Alert systems**: Performance degradation notifications

#### Developer Experience
- **IDE integration**: Performance analysis in development environments
- **Git hooks**: Pre-commit performance validation
- **Performance linting**: Code-level performance warnings
- **Documentation generation**: Automatic performance documentation

## Conclusion

The NGFS performance harness provides comprehensive, statistically sound performance testing with CI integration and baseline management. The polyglot tooling ensures consistency across languages while the virtual clock provides deterministic timing.

By following the guidelines in this document, teams can effectively manage performance baselines, interpret benchmark results, and integrate with existing promotion workflows. The system is designed to evolve with future performance requirements while maintaining backward compatibility and statistical rigor.
