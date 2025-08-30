load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test")

package(default_visibility = ["//visibility:public"])

# Twin service library
rust_library(
    name = "twin_lib",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
)

# Twin service binary
rust_binary(
    name = "twin_service",
    srcs = ["src/main.rs"],
    deps = [
        ":twin_lib",
        "@crate_index//:tokio",
        "@crate_index//:tonic",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
        "@crate_index//:clap",
    ],
    edition = "2021",
    features = ["full"],
)

# Twin snapshot tool
rust_binary(
    name = "twin_snapshot",
    srcs = ["src/snapshot_tool.rs"],
    deps = [
        ":twin_lib",
        "@crate_index//:clap",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
    ],
    edition = "2021",
    features = ["full"],
)

# Twin restore tool
rust_binary(
    name = "twin_restore",
    srcs = ["src/restore_tool.rs"],
    deps = [
        ":twin_lib",
        "@crate_index//:clap",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
    ],
    edition = "2021",
    features = ["full"],
)

# Twin validation tool
rust_binary(
    name = "twin_validate",
    srcs = ["src/validation_tool.rs"],
    deps = [
        ":twin_lib",
        "@crate_index//:clap",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
    ],
    edition = "2021",
    features = ["full"],
)

# Unit tests
rust_test(
    name = "twin_unit_tests",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        ":twin_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
)

# Snapshot tests
rust_test(
    name = "snapshot_tests",
    srcs = ["src/snapshot.rs"],
    deps = [
        ":twin_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
)

# Signing tests
rust_test(
    name = "signing_tests",
    srcs = ["src/sign.rs"],
    deps = [
        ":twin_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
)

# Integration tests
rust_test(
    name = "twin_integration_tests",
    srcs = glob(["tests/**/*.rs"]),
    deps = [
        ":twin_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
)

# Snapshot creation tests
rust_test(
    name = "snapshot_creation_tests",
    srcs = ["src/snapshot.rs"],
    deps = [
        ":twin_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "snapshot_creation"],
)

# Snapshot validation tests
rust_test(
    name = "snapshot_validation_tests",
    srcs = ["src/snapshot.rs"],
    deps = [
        ":twin_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "snapshot_validation"],
)

# Snapshot restoration tests
rust_test(
    name = "snapshot_restoration_tests",
    srcs = ["src/snapshot.rs"],
    deps = [
        ":twin_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "snapshot_restoration"],
)

# Dilithium signing tests
rust_test(
    name = "dilithium_signing_tests",
    srcs = ["src/sign.rs"],
    deps = [
        ":twin_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "dilithium_signing"],
)

# Signature verification tests
rust_test(
    name = "signature_verification_tests",
    srcs = ["src/sign.rs"],
    deps = [
        ":twin_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "signature_verification"],
)

# Health score consistency tests
rust_test(
    name = "health_score_consistency_tests",
    srcs = ["src/snapshot.rs"],
    deps = [
        ":twin_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "health_score_consistency"],
)

# JSON schema tests
rust_test(
    name = "json_schema_tests",
    srcs = ["src/snapshot.rs"],
    deps = [
        ":twin_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "json_schema"],
)

# Performance tests
rust_test(
    name = "twin_performance_tests",
    srcs = glob(["tests/performance/**/*.rs"]),
    deps = [
        ":twin_lib",
        "@crate_index//:criterion",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
)

# Security tests
rust_test(
    name = "twin_security_tests",
    srcs = glob(["tests/security/**/*.rs"]),
    deps = [
        ":twin_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
)

# Docker image for testing
container_image(
    name = "twin_service_image",
    base = "@distroless//cc:nonroot",
    files = [":twin_service"],
    cmd = ["/twin_service"],
    ports = ["50053"],
    labels = {
        "org.opencontainers.image.title": "Twin Service",
        "org.opencontainers.image.description": "Service state snapshot system for Polymera OS",
        "org.opencontainers.image.version": "0.1.0",
        "org.opencontainers.image.source": "https://github.com/polymera-os/polymera-os",
    },
)

# Kubernetes deployment
k8s_object(
    name = "twin_service_deployment",
    template = "k8s/deployment.yaml",
    images = [":twin_service_image"],
)

# Kubernetes service
k8s_object(
    name = "twin_service_service",
    template = "k8s/service.yaml",
)

# All tests target
filegroup(
    name = "all_tests",
    srcs = [
        ":twin_unit_tests",
        ":snapshot_tests",
        ":signing_tests",
        ":twin_integration_tests",
        ":snapshot_creation_tests",
        ":snapshot_validation_tests",
        ":snapshot_restoration_tests",
        ":dilithium_signing_tests",
        ":signature_verification_tests",
        ":health_score_consistency_tests",
        ":json_schema_tests",
        ":twin_performance_tests",
        ":twin_security_tests",
    ],
)

# All binaries target
filegroup(
    name = "all_binaries",
    srcs = [
        ":twin_service",
        ":twin_snapshot",
        ":twin_restore",
        ":twin_validate",
    ],
)

# All targets
filegroup(
    name = "all",
    srcs = [
        ":twin_lib",
        ":twin_service",
        ":twin_snapshot",
        ":twin_restore",
        ":twin_validate",
        ":twin_service_image",
        ":twin_service_deployment",
        ":twin_service_service",
        ":all_tests",
    ],
)
