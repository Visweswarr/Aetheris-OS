load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test")
load("@rules_proto//proto:defs.bzl", "proto_library")
load("@rules_proto_grpc//rust:defs.bzl", "rust_grpc_library")

package(default_visibility = ["//visibility:public"])

# Protocol buffer definitions
proto_library(
    name = "hello_proto",
    srcs = glob(["proto/**/*.proto"]),
    deps = [
        "@com_google_protobuf//:timestamp_proto",
        "@com_google_protobuf//:empty_proto",
    ],
)

# Rust gRPC library for hello service
rust_grpc_library(
    name = "hello_grpc",
    protos = [":hello_proto"],
    deps = [":hello_proto"],
)

# Hello service library
rust_library(
    name = "hello_lib",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        ":hello_grpc",
        "@crate_index//:tonic",
        "@crate_index//:tonic_health",
        "@crate_index//:tonic_reflection",
        "@crate_index//:tokio",
        "@crate_index//:tokio_stream",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
        "@crate_index//:opentelemetry",
        "@crate_index//:opentelemetry_sdk",
        "@crate_index//:opentelemetry_prometheus",
        "@crate_index//:prometheus",
        "@crate_index//:warp",
        "@crate_index//:uuid",
        "@crate_index//:prost_types",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:async_trait",
    ],
    edition = "2021",
    features = ["full"],
)

# Hello service binary
rust_binary(
    name = "hello_service",
    srcs = ["src/main.rs"],
    deps = [
        ":hello_lib",
        ":hello_grpc",
    ],
    edition = "2021",
    features = ["full"],
)

# Hello service tests
rust_test(
    name = "hello_service_tests",
    srcs = glob(["tests/**/*.rs"]),
    deps = [
        ":hello_lib",
        ":hello_grpc",
        "@crate_index//:tokio_test",
        "@crate_index//:tonic",
        "@crate_index//:tonic_health",
        "@crate_index//:serde_test",
        "@crate_index//:thiserror",
        "@crate_index//:tracing_test",
    ],
    edition = "2021",
    features = ["full"],
)

# Integration tests
rust_test(
    name = "hello_integration_tests",
    srcs = glob(["tests/integration/**/*.rs"]),
    deps = [
        ":hello_service",
        ":hello_grpc",
        "@crate_index//:tokio_test",
        "@crate_index//:tonic",
        "@crate_index//:serde_test",
        "@crate_index//:thiserror",
        "@crate_index//:tracing_test",
    ],
    edition = "2021",
    features = ["full"],
)

# Performance tests
rust_test(
    name = "hello_performance_tests",
    srcs = glob(["tests/performance/**/*.rs"]),
    deps = [
        ":hello_lib",
        ":hello_grpc",
        "@crate_index//:criterion",
        "@crate_index//:tokio_test",
        "@crate_index//:serde_test",
        "@crate_index//:thiserror",
        "@crate_index//:tracing_test",
    ],
    edition = "2021",
    features = ["full"],
)

# gRPC client for testing
rust_binary(
    name = "hello_client",
    srcs = ["src/client.rs"],
    deps = [
        ":hello_grpc",
        "@crate_index//:tonic",
        "@crate_index//:tokio",
        "@crate_index//:clap",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
    ],
    edition = "2021",
    features = ["full"],
)

# Health check client
rust_binary(
    name = "health_check_client",
    srcs = ["src/health_client.rs"],
    deps = [
        "@crate_index//:tonic",
        "@crate_index//:tonic_health",
        "@crate_index//:tokio",
        "@crate_index//:clap",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
    ],
    edition = "2021",
    features = ["full"],
)

# Metrics client
rust_binary(
    name = "metrics_client",
    srcs = ["src/metrics_client.rs"],
    deps = [
        "@crate_index//:reqwest",
        "@crate_index//:tokio",
        "@crate_index//:clap",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
    ],
    edition = "2021",
    features = ["full"],
)

# Docker image
container_image(
    name = "hello_service_image",
    base = "@distroless//cc:nonroot",
    files = [":hello_service"],
    cmd = ["/hello_service"],
    ports = ["50051", "9090"],
    labels = {
        "org.opencontainers.image.title": "Hello Service",
        "org.opencontainers.image.description": "Polymera OS Hello Service",
        "org.opencontainers.image.version": "0.1.0",
        "org.opencontainers.image.source": "https://github.com/polymera-os/polymera-os",
    },
)

# Kubernetes deployment
k8s_object(
    name = "hello_service_deployment",
    template = "k8s/deployment.yaml",
    images = [":hello_service_image"],
)

# Kubernetes service
k8s_object(
    name = "hello_service_service",
    template = "k8s/service.yaml",
)

# Kubernetes ingress
k8s_object(
    name = "hello_service_ingress",
    template = "k8s/ingress.yaml",
)

# All tests target
filegroup(
    name = "all_tests",
    srcs = [
        ":hello_service_tests",
        ":hello_integration_tests",
        ":hello_performance_tests",
    ],
)

# All binaries target
filegroup(
    name = "all_binaries",
    srcs = [
        ":hello_service",
        ":hello_client",
        ":health_check_client",
        ":metrics_client",
    ],
)

# All targets
filegroup(
    name = "all",
    srcs = [
        ":hello_proto",
        ":hello_grpc",
        ":hello_lib",
        ":hello_service",
        ":hello_client",
        ":health_check_client",
        ":metrics_client",
        ":hello_service_image",
        ":hello_service_deployment",
        ":hello_service_service",
        ":hello_service_ingress",
        ":all_tests",
    ],
)
