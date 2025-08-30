load("@rules_rust//rust:defs.bzl", "rust_library", "rust_binary", "rust_test", "rust_benchmark")
load("@rules_proto//proto:defs.bzl", "proto_library")
load("@rules_rust//proto:defs.bzl", "rust_proto_library")

package(default_visibility = ["//visibility:public"])

# Proto library
proto_library(
    name = "dtn_proto",
    srcs = ["../../proto/dtn.proto"],
    deps = [
        "@com_google_protobuf//:timestamp_proto",
        "@com_google_protobuf//:any_proto",
    ],
)

# Rust proto library
rust_proto_library(
    name = "dtn_rust_proto",
    deps = [":dtn_proto"],
)

# Main DTN library
rust_library(
    name = "polynet_dtn",
    srcs = glob([
        "src/**/*.rs",
    ]),
    deps = [
        ":dtn_rust_proto",
        "//services/polynet:polynet",
        "@crate_index//:tokio",
        "@crate_index//:futures",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
        "@crate_index//:uuid",
        "@crate_index//:chrono",
        "@crate_index//:prost",
        "@crate_index//:prost_types",
        "@crate_index//:tonic",
        "@crate_index//:sha2",
        "@crate_index//:ed25519_dalek",
        "@crate_index//:rand",
        "@crate_index//:sled",
        "@crate_index//:dashmap",
        "@crate_index//:libp2p",
        "@crate_index//:anyhow",
            "@crate_index//:clap",
            "@crate_index//:config",
            "@crate_index//:dirs",
    ],
    proc_macro_deps = [
        "@crate_index//:serde_derive",
        "@crate_index//:async_trait",
    ],
)

# DTN binary
rust_binary(
    name = "polynet_dtn_bin",
    srcs = ["src/main.rs"],
    deps = [
        ":polynet_dtn",
        "@crate_index//:clap",
        "@crate_index//:tracing_subscriber",
    ],
)

# Unit tests
rust_test(
    name = "polynet_dtn_tests",
    srcs = ["src/tests.rs"],
    deps = [":polynet_dtn"],
    data = ["//proto:dtn_proto"],
)

rust_test(
    name = "envelope_tests",
    srcs = ["src/envelope.rs"],
    deps = [":polynet_dtn"],
)

rust_test(
    name = "store_tests",
    srcs = ["src/store.rs"],
    deps = [":polynet_dtn"],
)

rust_test(
    name = "routing_tests",
    srcs = ["src/routing.rs"],
    deps = [":polynet_dtn"],
)

rust_test(
    name = "security_tests",
    srcs = ["src/security.rs"],
    deps = [":polynet_dtn"],
)

rust_test(
    name = "lib_tests",
    srcs = ["src/lib.rs"],
    deps = [":polynet_dtn"],
)

# Integration tests
rust_test(
    name = "integration_tests",
    srcs = ["src/tests.rs"],
    deps = [":polynet_dtn"],
    size = "large",
    timeout = "long",
)

# Performance benchmarks
rust_benchmark(
    name = "dtn_benchmarks",
    srcs = ["src/tests.rs"],
    deps = [":polynet_dtn"],
)

# Test suite
test_suite(
    name = "polynet_dtn_test_suite",
    tests = [
        ":polynet_dtn_tests",
        ":envelope_tests",
        ":store_tests",
        ":routing_tests",
        ":security_tests",
        ":lib_tests",
        ":integration_tests",
    ],
)

# Documentation
filegroup(
    name = "dtn_docs",
    srcs = [
        "README.md",
        "docs/",
    ],
    visibility = ["//visibility:public"],
)

# Configuration files
filegroup(
    name = "dtn_config",
    srcs = [
        "config/",
        "*.toml",
        "*.yaml",
    ],
    visibility = ["//visibility:public"],
)

# All DTN components
filegroup(
    name = "polynet_dtn_all",
    srcs = [
        ":polynet_dtn",
        ":polynet_dtn_bin",
        ":dtn_docs",
        ":dtn_config",
    ],
    visibility = ["//visibility:public"],
)
