load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test")
load("@rules_proto//proto:defs.bzl", "proto_library")
load("@rules_proto_grpc//rust:defs.bzl", "rust_grpc_library")

package(default_visibility = ["//visibility:public"])

# Health service proto library
proto_library(
    name = "health_proto",
    srcs = glob(["proto/**/*.proto"]),
    deps = [
        "@com_google_protobuf//:timestamp_proto",
        "@com_google_protobuf//:empty_proto",
        "@com_google_protobuf//:any_proto",
    ],
)

# Health service gRPC library
rust_grpc_library(
    name = "health_grpc",
    protos = [":health_proto"],
    deps = [":health_proto"],
)

# Health service library
rust_library(
    name = "health_lib",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        ":health_grpc",
        "@crate_index//:tokio",
        "@crate_index//:tonic",
        "@crate_index//:tonic_health",
        "@crate_index//:tonic_reflection",
        "@crate_index//:uuid",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
    ],
    edition = "2021",
    features = ["full"],
)

# Health service binary
rust_binary(
    name = "health_service",
    srcs = ["src/main.rs"],
    deps = [
        ":health_lib",
        ":health_grpc",
    ],
    edition = "2021",
    features = ["full"],
)

# Health service client
rust_binary(
    name = "health_client",
    srcs = ["src/client.rs"],
    deps = [
        ":health_grpc",
        "@crate_index//:tokio",
        "@crate_index//:tonic",
        "@crate_index//:clap",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
    ],
    edition = "2021",
    features = ["full"],
)

# Health monitoring client
rust_binary(
    name = "health_monitor",
    srcs = ["src/monitor.rs"],
    deps = [
        ":health_lib",
        ":health_grpc",
        "@crate_index//:tokio",
        "@crate_index//:tonic",
        "@crate_index//:clap",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
        "@crate_index//:chrono",
    ],
    edition = "2021",
    features = ["full"],
)

# Health dashboard client
rust_binary(
    name = "health_dashboard",
    srcs = ["src/dashboard.rs"],
    deps = [
        ":health_lib",
        ":health_grpc",
        "@crate_index//:tokio",
        "@crate_index//:tonic",
        "@crate_index//:clap",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
        "@crate_index//:chrono",
        "@crate_index//:term_table",
    ],
    edition = "2021",
    features = ["full"],
)

# Unit tests
rust_test(
    name = "health_unit_tests",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        ":health_lib",
        ":health_grpc",
        "@crate_index//:tokio",
        "@crate_index//:tonic",
        "@crate_index//:tonic_health",
        "@crate_index//:tonic_reflection",
        "@crate_index//:uuid",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
    ],
    edition = "2021",
    features = ["full"],
)

# Integration tests
rust_test(
    name = "health_integration_tests",
    srcs = glob(["tests/**/*.rs"]),
    deps = [
        ":health_lib",
        ":health_grpc",
        "@crate_index//:tokio",
        "@crate_index//:tonic",
        "@crate_index//:tonic_health",
        "@crate_index//:tonic_reflection",
        "@crate_index//:uuid",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
    ],
    edition = "2021",
    features = ["full"],
)

# Health check tests
rust_test(
    name = "health_check_tests",
    srcs = ["src/main.rs"],
    deps = [
        ":health_lib",
        ":health_grpc",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "health_check"],
)

# Dependency DAG tests
rust_test(
    name = "dependency_dag_tests",
    srcs = ["src/main.rs"],
    deps = [
        ":health_lib",
        ":health_grpc",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "dependency_dag"],
)

# Health score tests
rust_test(
    name = "health_score_tests",
    srcs = ["src/main.rs"],
    deps = [
        ":health_lib",
        ":health_grpc",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "health_score"],
)

# Alert emission tests
rust_test(
    name = "alert_emission_tests",
    srcs = ["src/main.rs"],
    deps = [
        ":health_lib",
        ":health_grpc",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "alert_emission"],
)

# Performance tests
rust_test(
    name = "health_performance_tests",
    srcs = glob(["tests/performance/**/*.rs"]),
    deps = [
        ":health_lib",
        ":health_grpc",
        "@crate_index//:criterion",
        "@crate_index//:tokio",
        "@crate_index//:tonic",
        "@crate_index//:tonic_health",
        "@crate_index//:tonic_reflection",
        "@crate_index//:uuid",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
    ],
    edition = "2021",
    features = ["full"],
)

# Load testing
rust_test(
    name = "health_load_tests",
    srcs = glob(["tests/load/**/*.rs"]),
    deps = [
        ":health_lib",
        ":health_grpc",
        "@crate_index//:tokio",
        "@crate_index//:tonic",
        "@crate_index//:tonic_health",
        "@crate_index//:tonic_reflection",
        "@crate_index//:uuid",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:criterion",
    ],
    edition = "2021",
    features = ["full"],
)

# Security tests
rust_test(
    name = "health_security_tests",
    srcs = glob(["tests/security/**/*.rs"]),
    deps = [
        ":health_lib",
        ":health_grpc",
        "@crate_index//:tokio",
        "@crate_index//:tonic",
        "@crate_index//:tonic_health",
        "@crate_index//:tonic_reflection",
        "@crate_index//:uuid",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
    ],
    edition = "2021",
    features = ["full"],
)

# Docker image for testing
container_image(
    name = "health_service_image",
    base = "@distroless//cc:nonroot",
    files = [":health_service"],
    cmd = ["/health_service"],
    ports = ["50052"],
    labels = {
        "org.opencontainers.image.title": "Health Service",
        "org.opencontainers.image.description": "Health monitoring service for Polymera OS",
        "org.opencontainers.image.version": "0.1.0",
        "org.opencontainers.image.source": "https://github.com/polymera-os/polymera-os",
    },
)

# Kubernetes deployment
k8s_object(
    name = "health_service_deployment",
    template = "k8s/deployment.yaml",
    images = [":health_service_image"],
)

# Kubernetes service
k8s_object(
    name = "health_service_service",
    template = "k8s/service.yaml",
)

# Kubernetes ingress
k8s_object(
    name = "health_service_ingress",
    template = "k8s/ingress.yaml",
)

# Health check configuration
k8s_object(
    name = "health_service_configmap",
    template = "k8s/configmap.yaml",
)

# Health service secret
k8s_object(
    name = "health_service_secret",
    template = "k8s/secret.yaml",
)

# All tests target
filegroup(
    name = "all_tests",
    srcs = [
        ":health_unit_tests",
        ":health_integration_tests",
        ":health_check_tests",
        ":dependency_dag_tests",
        ":health_score_tests",
        ":alert_emission_tests",
        ":health_performance_tests",
        ":health_load_tests",
        ":health_security_tests",
    ],
)

# All binaries target
filegroup(
    name = "all_binaries",
    srcs = [
        ":health_service",
        ":health_client",
        ":health_monitor",
        ":health_dashboard",
    ],
)

# All targets
filegroup(
    name = "all",
    srcs = [
        ":health_proto",
        ":health_grpc",
        ":health_lib",
        ":health_service",
        ":health_client",
        ":health_monitor",
        ":health_dashboard",
        ":health_service_image",
        ":health_service_deployment",
        ":health_service_service",
        ":health_service_ingress",
        ":health_service_configmap",
        ":health_service_secret",
        ":all_tests",
    ],
)
