load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_test")

package(default_visibility = ["//visibility:public"])

# XR Timing Harness
rust_binary(
    name = "xr_harness",
    srcs = ["xr.rs"],
    deps = [
        "@crates//:anyhow",
        "@crates//:clap",
        "@crates//:serde",
        "@crates//:serde_json",
        "@crates//:serde_yaml", 
        "@crates//:tokio",
        "@crates//:uuid",
        "@crates//:rand",
        "@crates//:tracing",
        "@crates//:tracing-subscriber",
        "@crates//:opentelemetry",
        "@crates//:opentelemetry-otlp",
        "@crates//:opentelemetry-sdk",
    ],
    edition = "2021",
)

# Network RTT Harness
rust_binary(
    name = "network_harness", 
    srcs = ["net.rs"],
    deps = [
        "@crates//:anyhow",
        "@crates//:clap",
        "@crates//:serde",
        "@crates//:serde_json",
        "@crates//:serde_yaml",
        "@crates//:tokio",
        "@crates//:uuid", 
        "@crates//:rand",
        "@crates//:tracing",
        "@crates//:tracing-subscriber",
        "@crates//:opentelemetry",
        "@crates//:opentelemetry-otlp",
        "@crates//:opentelemetry-sdk",
    ],
    edition = "2021",
)

# Wallet Confirmation Harness
rust_binary(
    name = "wallet_harness",
    srcs = ["wallet.rs"],
    deps = [
        "@crates//:anyhow",
        "@crates//:clap", 
        "@crates//:serde",
        "@crates//:serde_json",
        "@crates//:serde_yaml",
        "@crates//:tokio",
        "@crates//:uuid",
        "@crates//:rand",
        "@crates//:tracing",
        "@crates//:tracing-subscriber",
        "@crates//:opentelemetry",
        "@crates//:opentelemetry-otlp",
        "@crates//:opentelemetry-sdk",
    ],
    edition = "2021",
)

# Configuration files
filegroup(
    name = "configs",
    srcs = [
        "xr_config.yaml",
        "network_config.yaml", 
        "wallet_config.yaml",
    ],
)

# Test harness with Prometheus metrics
sh_test(
    name = "test_xr_metrics",
    srcs = ["test_xr_metrics.sh"],
    data = [
        ":xr_harness",
        ":configs",
    ],
    deps = ["@bazel_tools//tools/bash/runfiles"],
)

# Test network harness with metrics
sh_test(
    name = "test_network_metrics",
    srcs = ["test_network_metrics.sh"],
    data = [
        ":network_harness",
        ":configs",
    ],
    deps = ["@bazel_tools//tools/bash/runfiles"],
)

# Test wallet harness with metrics
sh_test(
    name = "test_wallet_metrics",
    srcs = ["test_wallet_metrics.sh"],
    data = [
        ":wallet_harness",
        ":configs",
    ],
    deps = ["@bazel_tools//tools/bash/runfiles"],
)

# Integration test for all harnesses
sh_test(
    name = "test_all_harnesses",
    srcs = ["test_all_harnesses.sh"],
    data = [
        ":xr_harness",
        ":network_harness", 
        ":wallet_harness",
        ":configs",
    ],
    deps = ["@bazel_tools//tools/bash/runfiles"],
    size = "large",
)

# Prometheus integration test
sh_test(
    name = "test_prometheus_integration",
    srcs = ["test_prometheus_integration.sh"],
    data = [
        ":xr_harness",
        ":network_harness",
        ":wallet_harness",
    ],
    deps = ["@bazel_tools//tools/bash/runfiles"],
    tags = ["integration", "prometheus"],
)

# Unit tests for harness implementations
rust_test(
    name = "xr_harness_tests",
    crate = ":xr_harness",
)

rust_test(
    name = "network_harness_tests", 
    crate = ":network_harness",
)

rust_test(
    name = "wallet_harness_tests",
    crate = ":wallet_harness",
)

# Generate sample configuration files
genrule(
    name = "generate_configs",
    outs = [
        "xr_config.yaml",
        "network_config.yaml",
        "wallet_config.yaml",
    ],
    cmd = """
        # XR Configuration
        cat > $(location xr_config.yaml) << 'EOF'
target_fps: 90
base_latency_ms: 8.0
latency_variance_ms: 3.0
frame_drop_probability: 0.001
duration_seconds: 60
enable_degradation: true
degradation_cycle_seconds: 30
EOF

        # Network Configuration  
        cat > $(location network_config.yaml) << 'EOF'
peer_count: 15
base_rtt_ms: 25.0
rtt_variance_ms: 10.0
packet_loss_probability: 0.002
sync_interval_seconds: 30
discovery_interval_seconds: 60
duration_seconds: 120
enable_degradation: true
enable_partitioning: false
EOF

        # Wallet Configuration
        cat > $(location wallet_config.yaml) << 'EOF'
base_auth_latency_ms: 1800.0
auth_latency_variance_ms: 400.0
base_session_latency_ms: 15.0
session_latency_variance_ms: 8.0
base_keystore_latency_ms: 30.0
keystore_latency_variance_ms: 15.0
auth_interval_seconds: 45
session_interval_seconds: 5
keystore_interval_seconds: 10
duration_seconds: 180
enable_privacy_overhead: true
enable_crypto_delays: true
operation_success_rate: 0.999
EOF
    """,
)

# Docker compose for testing with Prometheus
genrule(
    name = "docker_compose",
    outs = ["docker-compose.yml"],
    cmd = """
        cat > $(location docker-compose.yml) << 'EOF'
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:latest
    container_name: polymera-prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--web.enable-lifecycle'

  otel-collector:
    image: otel/opentelemetry-collector-contrib:latest
    container_name: polymera-otel
    ports:
      - "4317:4317"   # OTLP gRPC receiver
      - "4318:4318"   # OTLP HTTP receiver
      - "8888:8888"   # Prometheus metrics
    volumes:
      - ./otel-config.yml:/etc/otel-collector-config.yml
    command: ["--config=/etc/otel-collector-config.yml"]
    depends_on:
      - prometheus

  grafana:
    image: grafana/grafana:latest
    container_name: polymera-grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana-storage:/var/lib/grafana
    depends_on:
      - prometheus

volumes:
  grafana-storage:
EOF
    """,
)

# Test suite for all harness tests
test_suite(
    name = "harness_tests",
    tests = [
        ":test_xr_metrics",
        ":test_network_metrics", 
        ":test_wallet_metrics",
        ":xr_harness_tests",
        ":network_harness_tests",
        ":wallet_harness_tests",
    ],
)

# Alias for running all harnesses
alias(
    name = "all_harnesses",
    actual = ":test_all_harnesses",
)
