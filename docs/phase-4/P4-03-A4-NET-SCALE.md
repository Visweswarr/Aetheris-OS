# Phase 4.3 - Advanced Networking Scale & Performance

## Overview

This document describes the **Scale & Performance** implementation for Aetheris OS networking, providing high-scale benchmarking capabilities for testing network performance at scale (10k-50k concurrent connections) with both plain TCP and TLS/mTLS protocols.

## Architecture

### Core Components

1. **Rust Benchmark Harness** (`services/net/posixnet/src/bench.rs`)
   - High-scale concurrent connection testing
   - TLS/mTLS integration with PQC algorithms
   - Zero-copy I/O optimization
   - Comprehensive metrics collection

2. **Async Tuning Utilities** (`services/net/posixnet/src/tune.rs`)
   - Epoll/kqueue readiness tuning
   - Batched I/O operations
   - Zero-copy sendfile implementation
   - Back-pressure management

3. **Go CLI Extensions** (`go/tooling/netctl/bench.go`)
   - Concurrent benchmark commands
   - Soak test capabilities
   - JSON result reporting
   - Baseline recording

4. **TypeScript Bridge** (`tooling/ts/net-bench.ts`)
   - Node.js benchmarking support
   - Mini mode for CI environments
   - Performance metrics collection

5. **Python Reporting** (`tooling/python/net_report.py`)
   - Result aggregation and analysis
   - Baseline comparison
   - Regression detection
   - Performance reporting

## Features

### High-Scale Benchmarking

- **Concurrent Connections**: Support for 10k-50k concurrent connections
- **Protocol Support**: Both plain TCP and TLS/mTLS with PQC algorithms
- **Performance Metrics**: Comprehensive latency, throughput, and resource usage tracking
- **Deterministic Results**: CPU affinity, warm-up phases, and fixed RNG seeds

### Async I/O Optimization

- **Readiness Tuning**: Platform-specific epoll/kqueue optimization
- **Batched Operations**: Configurable batching for improved throughput
- **Zero-Copy I/O**: Memory-mapped buffers and sendfile optimization
- **Back-Pressure**: Automatic flow control to prevent resource exhaustion

### Performance Monitoring

- **Real-time Metrics**: RPS, latency percentiles, CPU, memory, GC stats
- **System Call Tracking**: Detailed syscall statistics
- **Error Monitoring**: Comprehensive error tracking and reporting
- **Resource Usage**: CPU, memory, and network resource monitoring

## Usage

### Command Line Interface

#### Basic Benchmarking

```bash
# Build components
cargo build -p posixnet
go build -o netctl go/tooling/netctl

# Run basic TCP benchmark
./netctl bench tcp --clients 1000 --duration 30s --json results.json

# Run TLS benchmark with PQC
./netctl bench tls --clients 1000 --duration 30s --profile pqc_hybrid --json results.json
```

#### High-Scale Concurrent Testing

```bash
# Run concurrent benchmark (10k connections)
./netctl concurrent --clients 10000 --duration 30m --tls on --profile pqc_hybrid --json runs/concurrent.json

# Run soak test (2 hours)
./netctl soak --clients 5000 --duration 2h --tls on --json runs/soak.json
```

#### CI Mini Mode

```bash
# Mini benchmark for CI (2-core runners)
./netctl concurrent --clients 2000 --duration 3m --tls off --json runs/mini.json
```

### TypeScript/Node.js

```bash
# Run TypeScript benchmark
cd tooling/ts
npm install
npx tsc net-bench.ts
node net-bench.js --clients 1000 --duration 30s --json results.json
```

### Python Reporting

```bash
# Aggregate multiple runs
python tooling/python/net_report.py aggregate runs/*.json --output aggregated.json

# Compare against baseline
python tooling/python/net_report.py compare runs/current.json clients_1000_tls_true_duration_30000

# Generate performance report
python tooling/python/net_report.py report runs/*.json --baseline clients_1000_tls_true_duration_30000 --output report.txt

# Check for regressions
python tooling/python/net_report.py check-regressions runs/*.json clients_1000_tls_true_duration_30000 --threshold 10.0
```

## Configuration

### Benchmark Configuration

```rust
let config = BenchConfig {
    clients: 10000,                    // Number of concurrent connections
    duration: Duration::from_secs(30), // Benchmark duration
    tls_enabled: true,                 // Enable TLS
    tls_profile: TLSProfile::PQCHybrid, // TLS profile
    pqc_algorithms: vec![              // PQC algorithms
        PQCAlgorithm::Kyber768,
        PQCAlgorithm::Dilithium3,
    ],
    server_addr: "127.0.0.1:8080".parse().unwrap(),
    payload_size: 1024,                // Payload size in bytes
    zero_copy: true,                   // Enable zero-copy I/O
    back_pressure: true,               // Enable back-pressure
    cpu_affinity: Some(vec![0, 1]),    // CPU affinity
    rng_seed: Some(42),                // Fixed RNG seed
    warmup_duration: Duration::from_secs(5),
    enable_profiling: false,           // Enable flamegraph profiling
};
```

### Async Tuning Configuration

```rust
let tuner = AsyncIOTuner::new(
    10000,                              // Max connections
    64,                                 // Batch size
    Duration::from_micros(100),         // Batch timeout
    (true, 1000, 100, Duration::from_millis(1)), // Back-pressure config
)?;
```

## Performance Baselines

### Baseline Storage

Baselines are stored in `perf/baselines/p4_03_net_adv.json` with the following structure:

```json
{
  "metadata": {
    "version": "1.0.0",
    "created_at": "2024-01-15T10:00:00Z",
    "environment": {
      "os": "Linux",
      "arch": "x86_64",
      "cpu_cores": 8,
      "memory_gb": 16.0
    }
  },
  "baselines": {
    "clients_1000_tls_false_duration_30000": {
      "config": { /* benchmark configuration */ },
      "metrics": { /* performance metrics */ },
      "statistics": { /* statistical analysis */ }
    }
  }
}
```

### Baseline Keys

Baseline keys are generated from configuration:
- Format: `clients_{N}_tls_{true|false}_duration_{ms}`
- Example: `clients_10000_tls_true_duration_1800000`

### Performance Targets

| Metric | TCP Target | TLS Target | PQC Target |
|--------|------------|------------|------------|
| RPS | 50,000 | 25,000 | 20,000 |
| Latency P95 | ≤ 1.2ms | ≤ 2.4ms | ≤ 10ms |
| Latency P99 | ≤ 2.0ms | ≤ 4.0ms | ≤ 20ms |
| CPU Usage | ≤ 25% | ≤ 45% | ≤ 90% |
| Memory | ≤ 128MB | ≤ 256MB | ≤ 2GB |
| Concurrent Connections | 10k+ | 10k+ | 10k+ |

## CI Integration

### Workflow Configuration

The CI workflow (`.github/workflows/p4-03-net-scale.yml`) includes:

1. **Mini Benchmark Tests**: Fast tests for 2-core CI runners
2. **Performance Regression Tests**: Comprehensive regression detection
3. **Flamegraph Analysis**: Automatic profiling on regressions
4. **Baseline Updates**: Automatic baseline updates on release tags

### Regression Detection

- **Threshold**: 10% performance regression triggers failure
- **Metrics**: RPS, latency percentiles, CPU, memory usage
- **Actions**: Automatic flamegraph generation on regressions
- **Reporting**: Detailed performance reports with baseline comparison

### Artifact Collection

- **Benchmark Results**: JSON files with detailed metrics
- **Flamegraphs**: SVG files for performance analysis
- **Reports**: Human-readable performance reports
- **Baselines**: Updated baseline files for release tags

## Monitoring and Observability

### Metrics Collection

- **Request Rate**: Requests per second (RPS)
- **Latency**: P50, P95, P99 percentiles
- **Throughput**: Bytes transmitted/received
- **System Resources**: CPU, memory, RSS usage
- **Garbage Collection**: GC count, time, heap usage
- **System Calls**: Detailed syscall statistics
- **Errors**: Error count and types

### Real-time Monitoring

```bash
# Monitor benchmark progress
./netctl soak --clients 5000 --duration 2h --report-interval 5m

# Output:
# [14:30:00] Soak test running... Clients: 5000, RPS: 25000.0, Errors: 0
# [14:35:00] Soak test running... Clients: 5000, RPS: 24800.0, Errors: 0
```

### Performance Analysis

```bash
# Generate comprehensive report
python tooling/python/net_report.py report runs/*.json --output analysis.txt

# Check for regressions
python tooling/python/net_report.py check-regressions runs/*.json baseline_key --threshold 10.0
```

## Troubleshooting

### Common Issues

1. **High Memory Usage**
   - Reduce concurrent connections
   - Enable back-pressure
   - Increase GC frequency

2. **High CPU Usage**
   - Enable zero-copy I/O
   - Optimize batch sizes
   - Use CPU affinity

3. **Connection Failures**
   - Check system limits (ulimit -n)
   - Increase listen backlog
   - Enable connection pooling

4. **Performance Regressions**
   - Check system load
   - Verify baseline environment
   - Analyze flamegraphs

### Debugging

```bash
# Enable debug logging
RUST_LOG=debug ./netctl concurrent --clients 1000 --duration 30s

# Enable profiling
RUST_LOG=info PERF=on ./netctl concurrent --clients 1000 --duration 30s

# Check system resources
htop
iostat -x 1
netstat -i
```

## Best Practices

### Benchmark Design

1. **Warm-up Phase**: Always include warm-up to stabilize performance
2. **Deterministic Results**: Use fixed RNG seeds and CPU affinity
3. **Multiple Runs**: Aggregate results from multiple benchmark runs
4. **Baseline Comparison**: Always compare against established baselines

### Performance Optimization

1. **Zero-Copy I/O**: Enable for high-throughput scenarios
2. **Batching**: Use appropriate batch sizes for your workload
3. **Back-Pressure**: Enable to prevent resource exhaustion
4. **Connection Pooling**: Use connection pools for high concurrency

### CI/CD Integration

1. **Mini Mode**: Use mini benchmarks for fast CI feedback
2. **Regression Detection**: Set appropriate thresholds (10% default)
3. **Artifact Collection**: Collect and analyze performance artifacts
4. **Baseline Updates**: Automatically update baselines on releases

## Future Enhancements

### Planned Features

1. **Distributed Benchmarking**: Multi-node benchmark coordination
2. **Real-time Dashboards**: Web-based performance monitoring
3. **Machine Learning**: Automated performance optimization
4. **Cloud Integration**: Cloud-native benchmarking support

### Performance Improvements

1. **Kernel Bypass**: DPDK/SPDK integration
2. **GPU Acceleration**: CUDA/OpenCL support for crypto operations
3. **Hardware Offload**: NIC-based TLS acceleration
4. **Memory Optimization**: Advanced memory management techniques

## References

- [Fuchsia Netstack3](https://fuchsia.dev/fuchsia-src/development/network/netstack3)
- [Genode Network Services](https://genode.org/documentation/genode-foundations-22-05.pdf)
- [Redox Networking](https://doc.redox-os.org/book/networking/)
- [Linux epoll Documentation](https://man7.org/linux/man-pages/man7/epoll.7.html)
- [Zero-Copy Networking](https://www.kernel.org/doc/Documentation/networking/msg_zerocopy.rst)
