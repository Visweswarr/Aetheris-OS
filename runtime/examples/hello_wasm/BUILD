load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test")
load("@rules_proto//proto:defs.bzl", "proto_library")

package(default_visibility = ["//visibility:public"])

# Hello WASM runtime library
rust_library(
    name = "hello_wasm_lib",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        "@crate_index//:wasi_cap_std_sync",
        "@crate_index//:wasi_common",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:tempfile",
        "@crate_index//:thiserror",
    ],
    edition = "2021",
    features = ["full"],
)

# Hello WASM binary
rust_binary(
    name = "hello_wasm",
    srcs = ["src/main.rs"],
    deps = [
        ":hello_wasm_lib",
    ],
    edition = "2021",
    features = ["full"],
)

# Host test harness
rust_binary(
    name = "host_harness",
    srcs = ["src/host_harness.rs"],
    deps = [
        ":hello_wasm_lib",
    ],
    edition = "2021",
    features = ["full"],
)

# Unit tests
rust_test(
    name = "hello_wasm_unit_tests",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        ":hello_wasm_lib",
        "@crate_index//:wasi_cap_std_sync",
        "@crate_index//:wasi_common",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:tempfile",
        "@crate_index//:thiserror",
    ],
    edition = "2021",
    features = ["full"],
)

# Integration tests
rust_test(
    name = "hello_wasm_integration_tests",
    srcs = glob(["tests/**/*.rs"]),
    deps = [
        ":hello_wasm_lib",
        "@crate_index//:wasi_cap_std_sync",
        "@crate_index//:wasi_common",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:tempfile",
        "@crate_index//:thiserror",
    ],
    edition = "2021",
    features = ["full"],
)

# Capability tests
rust_test(
    name = "capability_tests",
    srcs = ["src/main.rs"],
    deps = [
        ":hello_wasm_lib",
    ],
    edition = "2021",
    features = ["full"],
    args = ["test"],
)

# Security policy tests
rust_test(
    name = "security_policy_tests",
    srcs = ["src/main.rs"],
    deps = [
        ":hello_wasm_lib",
    ],
    edition = "2021",
    features = ["full"],
    args = ["capabilities"],
)

# Manifest tests
rust_test(
    name = "manifest_tests",
    srcs = ["src/main.rs"],
    deps = [
        ":hello_wasm_lib",
    ],
    edition = "2021",
    features = ["full"],
    args = ["manifest", "test_manifest.json"],
)

# WASI-2 compliance tests
rust_test(
    name = "wasi2_compliance_tests",
    srcs = glob(["tests/wasi2/**/*.rs"]),
    deps = [
        ":hello_wasm_lib",
        "@crate_index//:wasi_cap_std_sync",
        "@crate_index//:wasi_common",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:tempfile",
        "@crate_index//:thiserror",
    ],
    edition = "2021",
    features = ["full"],
)

# Performance tests
rust_test(
    name = "performance_tests",
    srcs = glob(["tests/performance/**/*.rs"]),
    deps = [
        ":hello_wasm_lib",
        "@crate_index//:criterion",
        "@crate_index//:wasi_cap_std_sync",
        "@crate_index//:wasi_common",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:tempfile",
        "@crate_index//:thiserror",
    ],
    edition = "2021",
    features = ["full"],
)

# Security tests
rust_test(
    name = "security_tests",
    srcs = glob(["tests/security/**/*.rs"]),
    deps = [
        ":hello_wasm_lib",
        "@crate_index//:wasi_cap_std_sync",
        "@crate_index//:wasi_common",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:tempfile",
        "@crate_index//:thiserror",
    ],
    edition = "2021",
    features = ["full"],
)

# WASM compilation target
rust_binary(
    name = "hello_wasm_wasm",
    srcs = ["src/main.rs"],
    deps = [
        ":hello_wasm_lib",
    ],
    edition = "2021",
    features = ["full"],
    target_compatible_with = ["@platforms//cpu:wasm32"],
)

# WASI target
rust_binary(
    name = "hello_wasm_wasi",
    srcs = ["src/main.rs"],
    deps = [
        ":hello_wasm_lib",
    ],
    edition = "2021",
    features = ["full"],
    target_compatible_with = ["@platforms//os:unknown"],
    target_compatible_with = ["@platforms//cpu:wasm32"],
)

# Docker image for testing
container_image(
    name = "hello_wasm_image",
    base = "@distroless//cc:nonroot",
    files = [":hello_wasm"],
    cmd = ["/hello_wasm"],
    ports = ["8080"],
    labels = {
        "org.opencontainers.image.title": "Hello WASM Runtime",
        "org.opencontainers.image.description": "WASI-2 Module with Capability-Based Security",
        "org.opencontainers.image.version": "0.1.0",
        "org.opencontainers.image.source": "https://github.com/polymera-os/polymera-os",
    },
)

# Kubernetes deployment
k8s_object(
    name = "hello_wasm_deployment",
    template = "k8s/deployment.yaml",
    images = [":hello_wasm_image"],
)

# Kubernetes service
k8s_object(
    name = "hello_wasm_service",
    template = "k8s/service.yaml",
)

# All tests target
filegroup(
    name = "all_tests",
    srcs = [
        ":hello_wasm_unit_tests",
        ":hello_wasm_integration_tests",
        ":capability_tests",
        ":security_policy_tests",
        ":manifest_tests",
        ":wasi2_compliance_tests",
        ":performance_tests",
        ":security_tests",
    ],
)

# All binaries target
filegroup(
    name = "all_binaries",
    srcs = [
        ":hello_wasm",
        ":host_harness",
        ":hello_wasm_wasm",
        ":hello_wasm_wasi",
    ],
)

# All targets
filegroup(
    name = "all",
    srcs = [
        ":hello_wasm_lib",
        ":hello_wasm",
        ":host_harness",
        ":hello_wasm_wasm",
        ":hello_wasm_wasi",
        ":hello_wasm_image",
        ":hello_wasm_deployment",
        ":hello_wasm_service",
        ":all_tests",
    ],
)
