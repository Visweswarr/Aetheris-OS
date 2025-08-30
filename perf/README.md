# Polymera OS Performance Testing Tools

This directory contains performance testing and monitoring tools for Polymera OS, with a focus on IPC (Inter-Process Communication) latency measurement and SLO validation.

## IPC Performance Check Script

### Overview
The `check_ipc.rs` script is a standalone performance testing tool that validates IPC latency performance against Service Level Objectives (SLOs).

### SLO Targets
- **P50 Latency**: < 200μs (QEMU environment)
- **P95 Latency**: < 500μs (QEMU environment)
- **Test Duration**: 1 second
- **Message Count**: 100 messages

### Features
- **Latency Histogram**: Tracks latency distribution across 16 buckets
- **Percentile Calculation**: P50, P95, P99 latency measurements
- **Statistical Analysis**: Mean, min, max, and sample count
- **SLO Validation**: Automatic pass/fail determination
- **Detailed Reporting**: Comprehensive performance metrics output

### Usage

#### Building with Bazel
```bash
# Build the performance script
bazel build //perf:check_ipc

# Run the performance test
bazel run //perf:check_ipc
```

#### Building with Cargo
```bash
# Build in release mode for best performance
cargo build --release

# Run the performance test
cargo run --release
```

#### Running Tests
```bash
# Run unit tests
cargo test

# Run with Bazel
bazel test //perf:check_ipc_test
```

### Output Format
The script provides detailed performance metrics:

```
=== IPC PERFORMANCE TEST RESULTS ===
Test Duration: 1000ms
Messages: 100 sent, 100 received

Latency Statistics (100 samples):
  P50 (median): 150μs
  P95: 400μs
  P99: 600μs
  Mean: 180μs
  Min: 50μs
  Max: 800μs

SLO Targets:
  P50 < 200μs: ✅ PASS
  P95 < 500μs: ✅ PASS

🎉 ALL SLO TARGETS MET! IPC performance is excellent.
```

### Integration with Kernel

The IPC latency tracking is integrated into the kernel's tracing system:

1. **Send Timestamp**: Set when messages are sent via `MessageHeader::set_send_timestamp()`
2. **Latency Calculation**: Computed when messages are received
3. **Histogram Recording**: Stored in the global `IPC_LATENCY_HISTOGRAM`
4. **Statistics Exposure**: Available via `sys_stats()` syscall

### Kernel Integration Points

- **Message Header**: Added `send_timestamp` field for latency tracking
- **IPC Queues**: Modified `send_message()` and `receive_message()` functions
- **Tracing System**: Enhanced with latency histogram and statistics
- **System Stats**: Extended `SystemStats` with IPC latency metrics

### Performance Characteristics

#### Memory Usage
- **Histogram Buckets**: 16 fixed-size buckets (64 bytes total)
- **Sample Storage**: Efficient circular buffer implementation
- **Memory Overhead**: < 1KB per IPC operation

#### CPU Overhead
- **Send Operation**: +2-5μs for timestamp setting
- **Receive Operation**: +5-10μs for latency calculation
- **Histogram Update**: +1-2μs for bucket indexing

#### Accuracy
- **Timestamp Resolution**: Microsecond precision
- **Latency Range**: 0μs to 500,000μs (500ms)
- **Bucket Granularity**: Logarithmic distribution for wide range coverage

### Troubleshooting

#### Common Issues

1. **High Latency Measurements**
   - Check QEMU configuration (KVM acceleration enabled)
   - Verify system load and resource availability
   - Review IPC queue sizes and scheduling

2. **Missing Latency Data**
   - Ensure IPC tracing is initialized
   - Verify message send/receive flow
   - Check for timestamp overflow issues

3. **SLO Target Failures**
   - Review system configuration
   - Check for resource contention
   - Verify measurement methodology

#### Debug Commands

```bash
# Check kernel IPC statistics
sys_debug 2 0  # Print scheduler/IPC counters

# View IPC latency histogram
# (Available via kernel tracing system)

# Monitor IPC performance in real-time
# (Use kernel tracing and monitoring tools)
```

### Future Enhancements

- **Real-time Monitoring**: Live latency tracking and alerting
- **Advanced Analytics**: Trend analysis and anomaly detection
- **Performance Profiling**: Detailed IPC operation breakdown
- **Load Testing**: Stress testing under various conditions
- **Cross-platform Support**: Performance validation on different architectures

### Contributing

When adding new performance tests or modifying existing ones:

1. **Follow SLO Standards**: Maintain consistent latency targets
2. **Document Changes**: Update this README and related documentation
3. **Add Tests**: Include unit tests for new functionality
4. **Performance Impact**: Minimize overhead of measurement tools
5. **Validation**: Ensure accuracy across different environments

### References

- [Phase 1 SPEC](../docs/phase-1/SPEC.md) - Performance requirements and SLOs
- [IPC Design](../docs/phase-1/ipc.md) - IPC system architecture
- [Tracing System](../docs/phase-1/tracing.md) - Kernel tracing and monitoring
- [Developer Guide](../docs/phase-1/DEV_GUIDE.md) - Building and testing procedures
