# Performance Baselines

This document describes the performance baseline system for Polymera OS, including how baselines are managed, promoted, and enforced.

## Overview

Performance baselines serve as the foundation for performance regression detection and quality assurance. They represent stable, validated performance metrics that have been achieved across multiple test runs and configurations.

## Baseline Structure

### File Location
- **Current Baseline**: `perf/baselines/p2.json`
- **History Data**: `perf/history/*.json`
- **Proposals**: `perf/baseline_proposal/`

### Baseline Format
```json
{
  "version": "0.2.0-phase2",
  "generated_at": "2024-01-15T10:00:00Z",
  "baselines": {
    "config_id": {
      "name": "Configuration Name",
      "promoted_at": "2024-01-15T10:00:00Z",
      "metrics": {
        "latency_p50": {
          "value": 150.0,
          "mean": 148.5,
          "std_dev": 2.5,
          "trend": "stable",
          "status": "stable"
        }
      }
    }
  }
}
```

## Baseline Promotion Process

### 1. Analysis Phase
The baseline promotion tool analyzes performance history to determine if configurations are ready for promotion:

- **Stability Threshold**: Minimum 5 stable runs required
- **Regression Check**: No metrics worse than mean + 1σ
- **Improvement Detection**: 5% improvement threshold
- **Trend Analysis**: Linear regression for stability assessment

### 2. Promotion Criteria
A configuration is ready for promotion when:

- ✅ **Stability**: 80% of metrics are stable or improved
- ✅ **No Regressions**: All metrics within acceptable bounds
- ✅ **Sufficient Data**: At least 5 recent stable runs
- ✅ **Statistical Significance**: Improvements are meaningful

### 3. Approval Workflow
Baseline promotions require human approval:

1. **Automated Analysis**: Tool generates promotion proposal
2. **PR Creation**: GitHub Actions creates approval PR
3. **Code Review**: CODEOWNERS must approve changes
4. **Merge**: New baseline becomes active
5. **Verification**: CI confirms baseline activation

## Configuration Coverage

### Matrix Configurations
The baseline system covers the full Phase 2 matrix:

- **CPU Models**: qemu32, qemu64, qemu64+apic
- **SMP Cores**: 2, 4 cores
- **Memory Sizes**: 256MB, 512MB, 1024MB
- **Features**: APIC, HPET, Jitter, Auth, Cap Policy

### Performance Metrics
Each configuration tracks:

- **Latency**: P50, P95, P99 percentiles
- **Throughput**: Operations per second
- **Resource Usage**: Memory and CPU consumption
- **Stability**: Variance and trend analysis

## Baseline Management

### Manual Promotion
```bash
# Run baseline promotion analysis
python tooling/perf/promote_baseline.py \
  --history-dir perf/history \
  --baseline-file perf/baselines/p2.json \
  --output-dir perf/baseline_proposal \
  --verbose
```

### Automated Workflow
1. **Trigger**: Manual workflow dispatch
2. **Analysis**: Python tool processes history data
3. **Proposal**: Generates promotion artifacts
4. **PR Creation**: Automated PR with approval checklist
5. **Team Notification**: Issue created for review

### Artifacts Generated
- `baseline_proposal.json` - Complete analysis results
- `proposed_baseline.json` - New baseline file
- `impact_heatmap.txt` - Visual impact assessment
- `summary_report.md` - Human-readable summary
- `approval_checklist.md` - Review checklist

## Quality Gates

### Regression Prevention
- **Threshold**: mean + 1σ for any metric
- **Override**: `allow-regression` label (with justification)
- **Monitoring**: Continuous CI validation

### Stability Requirements
- **Minimum Runs**: 5 stable runs per configuration
- **Variance Control**: Standard deviation limits
- **Trend Analysis**: Linear regression stability

### Improvement Validation
- **Statistical Significance**: 5% improvement threshold
- **Consistency**: Multiple runs confirm improvement
- **Documentation**: Changelog entry required

## Integration Points

### CI/CD Pipeline
- **Matrix Testing**: Validates against current baseline
- **Regression Detection**: Fails builds on violations
- **Baseline Updates**: Automatic on tag releases

### Dashboard Integration
- **Performance Trends**: Visualize baseline evolution
- **Impact Assessment**: Compare current vs. baseline
- **Historical Analysis**: Track performance over time

### Documentation Sync
- **Automatic Updates**: Baseline changes update docs
- **Changelog Management**: Versioned change tracking
- **Team Communication**: Automated notifications

## Monitoring & Maintenance

### Baseline Health
- **Freshness**: Regular updates (monthly recommended)
- **Coverage**: All matrix configurations represented
- **Accuracy**: Statistical validation of metrics

### Performance Trends
- **Long-term Tracking**: Historical baseline evolution
- **Improvement Patterns**: Identify optimization opportunities
- **Regression Prevention**: Early warning systems

### Team Workflow
- **Review Process**: Structured approval checklist
- **Documentation**: Comprehensive change tracking
- **Communication**: Automated team notifications

## Troubleshooting

### Common Issues

#### Insufficient Data
- **Symptom**: "Insufficient data for analysis"
- **Solution**: Ensure ≥5 runs per configuration
- **Prevention**: Regular CI execution

#### Regression Detection
- **Symptom**: Build failures on performance tests
- **Investigation**: Check metric trends and variance
- **Resolution**: Address performance issues or adjust thresholds

#### Baseline Staleness
- **Symptom**: Outdated baseline values
- **Solution**: Run promotion analysis
- **Prevention**: Scheduled baseline updates

### Debug Commands
```bash
# Check baseline status
python tooling/perf/promote_baseline.py --verbose

# Validate history data
ls -la perf/history/
cat perf/baselines/p2.json | jq '.'

# Test promotion logic
python tooling/perf/promote_baseline.py --output-dir test_proposal
```

## Future Enhancements

### Planned Features
- **Machine Learning**: Automated trend prediction
- **Dynamic Thresholds**: Adaptive regression detection
- **Performance Budgets**: Resource allocation limits
- **Integration APIs**: External monitoring systems

### Roadmap
- **Phase 2.5**: Enhanced statistical analysis
- **Phase 3**: ML-powered optimization
- **Phase 4**: Real-time performance monitoring

## References

- [Performance Dashboard](../site/perf/)
- [CI Matrix Configuration](../../../.github/workflows/phase-2-matrix.yml)
- [Baseline Promotion Workflow](../../../.github/workflows/baseline-promote.yml)
- [Performance History Format](../history/README.md)
