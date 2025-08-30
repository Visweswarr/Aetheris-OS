# Statistical Guardrails (Rolling Mean + 3σ)

## Overview

The Statistical Guardrails system provides robust performance regression detection for Polymera OS by using rolling statistical analysis instead of static thresholds. This approach reduces false alarms while maintaining performance quality standards through adaptive thresholds based on historical performance data.

## Features

- **Rolling Statistics**: Dynamic thresholds based on recent performance history
- **Statistical Robustness**: Mean + 3σ thresholds for reliable regression detection
- **Absolute Budgets**: Hard limits to prevent performance degradation beyond acceptable bounds
- **Warning Levels**: Multiple severity levels (pass, warning, fail) for nuanced feedback
- **History Management**: Automatic history updates on successful runs only
- **Race Condition Protection**: Simple locking mechanism for concurrent updates

## System Architecture

### Core Components

```
Performance Guardrails System
├── Guardrails Checker (Python)
│   ├── Statistical Analysis Engine
│   ├── Policy Management
│   └── Result Evaluation
├── History Management
│   ├── Rolling Window (20 entries)
│   ├── Automatic Updates
│   └── Git Integration
└── Integration Layer
    ├── Matrix Workflow Integration
    ├── PR Annotation Integration
    └── Override Mechanisms
```

### Data Flow

```
Matrix Results → Performance Metrics → Guardrails Check → History Update → Git Commit
     ↓              ↓                    ↓              ↓              ↓
Test Output → Metrics Collection → Statistical Analysis → Success Only → Repository
```

## Statistical Methodology

### Rolling Window Approach

#### **Window Size**
- **Default**: 20 performance entries per configuration
- **Configurable**: Adjustable via `--window-size` parameter
- **Purpose**: Balance between statistical significance and responsiveness

#### **Statistical Measures**
- **Mean**: Rolling average of recent performance values
- **Standard Deviation (σ)**: Measure of performance variability
- **Thresholds**: Mean + N×σ for regression detection

### Threshold Policies

#### **Warning Thresholds (2σ)**
- **Purpose**: Early warning of potential regressions
- **Action**: Mark as warning, continue execution
- **Example**: IPC latency p50 > mean + 2σ

#### **Fail Thresholds (3σ)**
- **Purpose**: Significant regression detection
- **Action**: Mark as failure, block progression
- **Example**: IPC latency p50 > mean + 3σ

#### **Absolute Budgets**
- **Purpose**: Hard performance limits
- **Action**: Immediate failure regardless of statistics
- **Example**: IPC latency p50 > 500ns

### Policy Configuration

```python
policies = {
    'ipc_latency_p50': {
        'warning_threshold': 2.0,    # 2σ
        'fail_threshold': 3.0,       # 3σ
        'absolute_budget': 500       # 500ns
    },
    'wake_to_run_p50': {
        'warning_threshold': 2.0,    # 2σ
        'fail_threshold': 3.0,       # 3σ
        'absolute_budget': 300       # 300ns
    }
    # ... more metrics
}
```

## Performance Metrics

### IPC Latency Metrics

#### **Percentile Measurements**
- **p50**: 50th percentile (median) latency
- **p95**: 95th percentile latency
- **p99**: 99th percentile latency
- **Mean**: Average latency across all measurements

#### **Thresholds**
- **Warning**: > mean + 2σ
- **Fail**: > mean + 3σ
- **Absolute**: > 500ns (p50), > 1000ns (p95), > 2000ns (p99)

### Wake-to-Run Metrics

#### **Task Scheduling Performance**
- **p50**: Median wake-to-run time
- **p95**: 95th percentile wake-to-run time
- **p99**: 99th percentile wake-to-run time

#### **Thresholds**
- **Warning**: > mean + 2σ
- **Fail**: > mean + 3σ
- **Absolute**: > 300ns (p50), > 600ns (p95), > 1200ns (p99)

### PQC Overhead Metrics

#### **Cryptographic Performance**
- **Kyber Encapsulation**: Key encapsulation time
- **Dilithium Sign**: Digital signature generation
- **Dilithium Verify**: Digital signature verification
- **MAC Generation**: Message authentication code generation
- **MAC Verification**: Message authentication code verification

#### **Thresholds**
- **Warning**: > mean + 2σ
- **Fail**: > mean + 3σ
- **Absolute**: Varies by operation (1000-5000ns)

### Memory Usage Metrics

#### **Resource Consumption**
- **Kernel Heap**: Kernel memory allocation (MB)
- **User Heap**: User space memory allocation (MB)
- **Stack Usage**: Stack memory consumption (MB)

#### **Thresholds**
- **Warning**: > mean + 2σ
- **Fail**: > mean + 3σ
- **Absolute**: 50MB (kernel), 100MB (user), 20MB (stack)

## Usage

### Command Line Interface

#### **Basic Usage**
```bash
python3 tooling/perf/guardrails.py [perf_file] [history_dir] [config_id]
```

#### **Complete Example**
```bash
python3 tooling/perf/guardrails.py \
  "perf/results/matrix/apic_hpet_jitter_auth_open/metrics.json" \
  "perf/history" \
  "apic_hpet_jitter_auth_open" \
  --config-name "APIC+HPET, Jitter On, Auth On, Cap Policy Open" \
  --output-file "guardrails_result.json"
```

#### **History Update**
```bash
python3 tooling/perf/guardrails.py \
  "perf/results/matrix/apic_hpet_jitter_auth_open/metrics.json" \
  "perf/history" \
  "apic_hpet_jitter_auth_open" \
  --update-history
```

### Parameters

#### **Required Parameters**
- **perf_file**: Path to current performance JSON file
- **history_dir**: Directory containing performance history files
- **config_id**: Configuration identifier for history tracking

#### **Optional Parameters**
- **--config-name**: Human-readable configuration name
- **--update-history**: Update history on successful runs
- **--output-file**: Save results to specified file
- **--window-size**: History window size (default: 20)

## History Management

### File Structure

#### **History Directory**
```
perf/history/
├── .gitkeep                    # Ensures directory is tracked
├── apic_hpet_jitter_auth_open.json
├── apic_only_jitter_auth_closed.json
├── hpet_only_jitter_auth_open.json
└── ... (one file per configuration)
```

#### **History File Format**
```json
{
  "config_id": "apic_hpet_jitter_auth_open",
  "created": "2024-01-15T02:00:00Z",
  "last_updated": "2024-01-15T02:00:00Z",
  "window_size": 20,
  "total_entries": 15,
  "entries": [
    {
      "timestamp": "2024-01-15T02:00:00Z",
      "run_id": "123456789",
      "commit_sha": "abc123def456",
      "metrics": {
        "ipc_latency_p50": 150.5,
        "ipc_latency_p95": 280.3,
        "wake_to_run_p50": 125.7
      }
    }
  ]
}
```

### Update Strategy

#### **Success-Only Updates**
- **Rule**: History updated only on successful guardrail checks
- **Rationale**: Prevent poisoning of baseline with failed performance
- **Implementation**: Conditional execution based on guardrail results

#### **Rolling Window Maintenance**
- **Automatic**: Old entries removed when exceeding window size
- **Preservation**: Most recent N entries maintained
- **Consistency**: Window size enforced across all configurations

## Integration with CI/CD

### Matrix Workflow Integration

#### **Guardrails Check Step**
```yaml
- name: Check performance guardrails
  id: guardrails_check
  run: |
    python3 tooling/perf/guardrails.py \
      "perf/results/matrix/${{ matrix.config.config_id }}/metrics.json" \
      "perf/history" \
      "${{ matrix.config.config_id }}" \
      --config-name "${{ matrix.config.name }}" \
      --output-file "perf/results/matrix/${{ matrix.config.config_id }}/guardrails_result.json"
```

#### **History Update Step**
```yaml
- name: Update performance history on success
  if: steps.guardrails_check.outputs.guardrails_passed == 'true'
  run: |
    python3 tooling/perf/guardrails.py \
      "perf/results/matrix/${{ matrix.config.config_id }}/metrics.json" \
      "perf/history" \
      "${{ matrix.config.config_id }}" \
      --update-history
```

### Output Management

#### **Workflow Outputs**
- **guardrails_passed**: Boolean indicating overall pass/fail status
- **guardrails_summary**: Human-readable summary of results
- **guardrails_exit_code**: Exit code from guardrails script

#### **Artifact Generation**
- **guardrails_result.json**: Detailed guardrail check results
- **History files**: Updated performance history in repository

## Override Mechanisms

### Label-Based Overrides

#### **Allow Performance Regression**
- **Label**: `allow-perf-regression`
- **Purpose**: Bypass guardrail failures for specific PRs
- **Usage**: Add label to PR to allow performance regressions
- **Rationale**: Enable intentional performance changes

#### **Implementation**
```yaml
# Check for override label
if: contains(github.event.pull_request.labels.*.name, 'allow-perf-regression')
  # Skip guardrail enforcement
  echo "Performance regression override detected, skipping guardrails"
else
  # Enforce guardrails
  python3 tooling/perf/guardrails.py ...
```

### Manual Overrides

#### **Command Line Options**
- **--force-update**: Force history update regardless of results
- **--skip-check**: Skip guardrail evaluation
- **--custom-thresholds**: Override default threshold policies

#### **Environment Variables**
- **GUARDRAILS_DISABLED**: Disable guardrail system entirely
- **GUARDRAILS_WINDOW_SIZE**: Override default window size
- **GUARDRAILS_STRICT_MODE**: Enable stricter threshold enforcement

## Race Condition Protection

### Concurrent Update Handling

#### **Simple Locking Strategy**
- **Approach**: File-based locking for history updates
- **Implementation**: Atomic file operations and error handling
- **Fallback**: Retry mechanism for failed updates

#### **Git Integration**
- **Atomic Commits**: Single commit per history update
- **Conflict Resolution**: Automatic merge conflict handling
- **Rollback**: Revert to previous state on update failure

### Update Sequence

#### **Safe Update Process**
1. **Load History**: Read current history file
2. **Validate Data**: Ensure data integrity
3. **Update History**: Add new performance entry
4. **Save File**: Write updated history atomically
5. **Git Commit**: Commit changes with descriptive message
6. **Push Updates**: Push to repository (if permitted)

#### **Error Handling**
- **File Locked**: Retry with exponential backoff
- **Corrupted Data**: Restore from backup or recreate
- **Git Conflicts**: Resolve automatically or skip update

## Results and Reporting

### Guardrail Results

#### **Individual Metric Results**
```json
{
  "metric_name": "ipc_latency_p50",
  "current_value": 180.5,
  "mean": 150.2,
  "std_dev": 25.1,
  "threshold": 225.5,
  "delta": 30.3,
  "delta_percent": 20.2,
  "passed": true,
  "severity": "warning",
  "reason": "Exceeds warning threshold: 180.5 > 200.4 (mean + 2σ)"
}
```

#### **Configuration Summary**
```json
{
  "config_id": "apic_hpet_jitter_auth_open",
  "config_name": "APIC+HPET, Jitter On, Auth On, Cap Policy Open",
  "timestamp": "2024-01-15T02:00:00Z",
  "total_metrics": 15,
  "passed_metrics": 12,
  "warning_metrics": 3,
  "failed_metrics": 0,
  "overall_passed": true,
  "summary": "PASSED with warnings: 3 metrics exceeded warning thresholds"
}
```

### Console Output

#### **Summary Display**
```
🔍 Performance Guardrails Check for apic_hpet_jitter_auth_open
📊 PASSED with warnings: 3 metrics exceeded warning thresholds
📈 Metrics: 12 passed, 3 warnings, 0 failed
⏰ Timestamp: 2024-01-15T02:00:00Z

📋 Detailed Results:
  ⚠️ ipc_latency_p50: 180.50
     Mean: 150.20, σ: 25.10
     Delta: +30.30 (+20.2%)
     Threshold: 200.40
     Status: Exceeds warning threshold: 180.5 > 200.4 (mean + 2σ)
```

## Best Practices

### Configuration Management

#### **Threshold Selection**
- **Start Conservative**: Begin with 2σ warning, 3σ fail thresholds
- **Adjust Based on Data**: Refine thresholds based on historical performance
- **Consider Context**: Different thresholds for different metric types
- **Regular Review**: Periodically review and adjust threshold policies

#### **Window Size Optimization**
- **Statistical Significance**: Ensure sufficient data for reliable statistics
- **Responsiveness**: Balance between stability and responsiveness
- **Resource Constraints**: Consider storage and processing overhead
- **Performance Patterns**: Adjust based on observed performance variability

### History Management

#### **Data Quality**
- **Validation**: Ensure performance data integrity before history updates
- **Filtering**: Remove outliers and invalid measurements
- **Normalization**: Consistent units and measurement methods
- **Documentation**: Clear documentation of data sources and methods

#### **Maintenance**
- **Regular Cleanup**: Remove old or corrupted history entries
- **Backup Strategy**: Maintain backups of critical history data
- **Version Control**: Track history file changes in git
- **Monitoring**: Monitor history file sizes and update frequency

## Troubleshooting

### Common Issues

#### **No History Data**
- **Symptoms**: All metrics show 0 mean and std_dev
- **Causes**: New configuration or corrupted history
- **Solutions**: Run with more data, check history file integrity

#### **Excessive Warnings**
- **Symptoms**: Many metrics showing warning status
- **Causes**: High performance variability or overly strict thresholds
- **Solutions**: Adjust threshold policies, increase window size

#### **History Update Failures**
- **Symptoms**: History not updating on successful runs
- **Causes**: File permissions, git issues, or race conditions
- **Solutions**: Check permissions, resolve git conflicts, retry updates

### Debug Mode

#### **Enhanced Logging**
```bash
export GUARDRAILS_DEBUG=true
export GUARDRAILS_VERBOSE=true
python3 tooling/perf/guardrails.py ...
```

#### **Detailed Output**
- **Metric Extraction**: Log all extracted metrics
- **Statistical Calculations**: Show mean and std_dev calculations
- **Threshold Comparisons**: Display threshold evaluation logic
- **History Operations**: Log all history file operations

## Future Enhancements

### Planned Features

#### **Machine Learning Integration**
- **Pattern Recognition**: ML-based anomaly detection
- **Predictive Analysis**: Forecast performance trends
- **Adaptive Thresholds**: Dynamic threshold adjustment
- **Intelligent Filtering**: Smart outlier detection

#### **Advanced Statistics**
- **Confidence Intervals**: Statistical confidence measures
- **Trend Analysis**: Long-term performance trend detection
- **Seasonal Patterns**: Recognition of periodic performance variations
- **Correlation Analysis**: Metric relationship analysis

### Integration Improvements

#### **Real-time Monitoring**
- **Live Updates**: Real-time performance monitoring
- **Alert Systems**: Proactive performance regression alerts
- **Dashboard Integration**: Performance monitoring dashboards
- **API Endpoints**: RESTful API for guardrail queries

#### **Advanced Reporting**
- **Trend Visualization**: Interactive performance trend charts
- **Comparative Analysis**: Cross-configuration performance comparison
- **Executive Summaries**: High-level performance reports
- **Automated Insights**: AI-generated performance insights

## Conclusion

The Statistical Guardrails system provides a robust foundation for performance regression detection in Polymera OS through rolling statistical analysis. By using adaptive thresholds based on historical performance data, the system reduces false alarms while maintaining strict performance quality standards.

### Key Benefits

- **Reduced False Alarms**: Statistical approach minimizes false positives
- **Adaptive Thresholds**: Dynamic thresholds based on performance history
- **Robust Detection**: Reliable regression detection with confidence measures
- **Automated Management**: Self-maintaining history and threshold systems
- **Integration Ready**: Seamless CI/CD workflow integration

### Success Metrics

- **False Positive Rate**: <5% false regression detections
- **Detection Sensitivity**: >95% true regression detection rate
- **Response Time**: <1 minute for guardrail evaluation
- **History Accuracy**: >99% successful history updates
- **System Reliability**: >99.9% uptime for guardrail system

The statistical guardrails system ensures that Polymera OS maintains high performance standards while providing developers with reliable feedback on performance changes, enabling data-driven performance optimization and quality assurance.
