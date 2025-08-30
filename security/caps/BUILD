load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test")

package(default_visibility = ["//visibility:public"])

# Capability tokens library
rust_library(
    name = "caps_lib",
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

# Capability tokens service binary
rust_binary(
    name = "caps_service",
    srcs = ["src/main.rs"],
    deps = [
        ":caps_lib",
        "@crate_index//:tokio",
        "@crate_index//:tonic",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
        "@crate_index//:clap",
    ],
    edition = "2021",
    features = ["full"],
)

# Capability tokens CLI tool
rust_binary(
    name = "caps_cli",
    srcs = ["src/cli.rs"],
    deps = [
        ":caps_lib",
        "@crate_index//:clap",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
    ],
    edition = "2021",
    features = ["full"],
)

# Capability tokens generator tool
rust_binary(
    name = "caps_generate",
    srcs = ["src/generator.rs"],
    deps = [
        ":caps_lib",
        "@crate_index//:clap",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
    ],
    edition = "2021",
    features = ["full"],
)

# Capability tokens validator tool
rust_binary(
    name = "caps_validate",
    srcs = ["src/validator.rs"],
    deps = [
        ":caps_lib",
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
    name = "caps_unit_tests",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        ":caps_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
)

# Format tests
rust_test(
    name = "format_tests",
    srcs = ["src/format.rs"],
    deps = [
        ":caps_lib",
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
        ":caps_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
)

# Verification tests
rust_test(
    name = "verification_tests",
    srcs = ["src/verify.rs"],
    deps = [
        ":caps_lib",
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
    name = "caps_integration_tests",
    srcs = glob(["tests/**/*.rs"]),
    deps = [
        ":caps_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
)

# Token format tests
rust_test(
    name = "token_format_tests",
    srcs = ["src/format.rs"],
    deps = [
        ":caps_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "token_format"],
)

# Token builder tests
rust_test(
    name = "token_builder_tests",
    srcs = ["src/format.rs"],
    deps = [
        ":caps_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "token_builder"],
)

# Token formatter tests
rust_test(
    name = "token_formatter_tests",
    srcs = ["src/format.rs"],
    deps = [
        ":caps_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "token_formatter"],
)

# Dilithium signing tests
rust_test(
    name = "dilithium_signing_tests",
    srcs = ["src/sign.rs"],
    deps = [
        ":caps_lib",
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
        ":caps_lib",
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

# Mock signer tests
rust_test(
    name = "mock_signer_tests",
    srcs = ["src/sign.rs"],
    deps = [
        ":caps_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "mock_signer"],
)

# Token validation tests
rust_test(
    name = "token_validation_tests",
    srcs = ["src/verify.rs"],
    deps = [
        ":caps_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "token_validation"],
)

# Expired token tests
rust_test(
    name = "expired_token_tests",
    srcs = ["src/verify.rs"],
    deps = [
        ":caps_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "expired_token"],
)

# Forged token tests
rust_test(
    name = "forged_token_tests",
    srcs = ["src/verify.rs"],
    deps = [
        ":caps_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "forged_token"],
)

# Purpose binding tests
rust_test(
    name = "purpose_binding_tests",
    srcs = ["src/verify.rs"],
    deps = [
        ":caps_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "purpose_binding"],
)

# Time scoping tests
rust_test(
    name = "time_scoping_tests",
    srcs = ["src/verify.rs"],
    deps = [
        ":caps_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "time_scoping"],
)

# Least privilege tests
rust_test(
    name = "least_privilege_tests",
    srcs = ["src/verify.rs"],
    deps = [
        ":caps_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "least_privilege"],
)

# Service tests
rust_test(
    name = "service_tests",
    srcs = ["src/mod.rs"],
    deps = [
        ":caps_lib",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:uuid",
        "@crate_index//:base64",
    ],
    edition = "2021",
    features = ["full"],
    args = ["--test", "service"],
)

# Performance tests
rust_test(
    name = "caps_performance_tests",
    srcs = glob(["tests/performance/**/*.rs"]),
    deps = [
        ":caps_lib",
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
    name = "caps_security_tests",
    srcs = glob(["tests/security/**/*.rs"]),
    deps = [
        ":caps_lib",
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
    name = "caps_service_image",
    base = "@distroless//cc:nonroot",
    files = [":caps_service"],
    cmd = ["/caps_service"],
    ports = ["50054"],
    labels = {
        "org.opencontainers.image.title": "Capability Token Service",
        "org.opencontainers.image.description": "Purpose-bound, time-scoped, least-privilege security system for Polymera OS",
        "org.opencontainers.image.version": "0.1.0",
        "org.opencontainers.image.source": "https://github.com/polymera-os/polymera-os",
    },
)

# Kubernetes deployment
k8s_object(
    name = "caps_service_deployment",
    template = "k8s/deployment.yaml",
    images = [":caps_service_image"],
)

# Kubernetes service
k8s_object(
    name = "caps_service_service",
    template = "k8s/service.yaml",
)

# All tests target
filegroup(
    name = "all_tests",
    srcs = [
        ":caps_unit_tests",
        ":format_tests",
        ":signing_tests",
        ":verification_tests",
        ":caps_integration_tests",
        ":token_format_tests",
        ":token_builder_tests",
        ":token_formatter_tests",
        ":dilithium_signing_tests",
        ":signature_verification_tests",
        ":mock_signer_tests",
        ":token_validation_tests",
        ":expired_token_tests",
        ":forged_token_tests",
        ":purpose_binding_tests",
        ":time_scoping_tests",
        ":least_privilege_tests",
        ":service_tests",
        ":caps_performance_tests",
        ":caps_security_tests",
    ],
)

# All binaries target
filegroup(
    name = "all_binaries",
    srcs = [
        ":caps_service",
        ":caps_cli",
        ":caps_generate",
        ":caps_validate",
    ],
)

# All targets
filegroup(
    name = "all",
    srcs = [
        ":caps_lib",
        ":caps_service",
        ":caps_cli",
        ":caps_generate",
        ":caps_validate",
        ":caps_service_image",
        ":caps_service_deployment",
        ":caps_service_service",
        ":all_tests",
    ],
)
