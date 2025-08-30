load("@rules_rust//rust:defs.bzl", "rust_library", "rust_binary", "rust_test", "rust_benchmark")

package(default_visibility = ["//visibility:public"])

# Main DID identity library
rust_library(
    name = "polymera_identity",
    srcs = glob([
        "src/**/*.rs",
    ]),
    deps = [
        "@crate_index//:tokio",
        "@crate_index//:futures",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:thiserror",
        "@crate_index//:tracing",
        "@crate_index//:tracing_subscriber",
        "@crate_index//:uuid",
        "@crate_index//:chrono",
        "@crate_index//:dilithium",
        "@crate_index//:kyber",
        "@crate_index//:ed25519_dalek",
        "@crate_index//:sha2",
        "@crate_index//:sha3",
        "@crate_index//:rand",
        "@crate_index//:did_parser",
        "@crate_index//:did_resolver",
        "@crate_index//:did_doc",
        "@crate_index//:sled",
        "@crate_index//:dashmap",
        "@crate_index//:anyhow",
        "@crate_index//:clap",
        "@crate_index//:config",
        "@crate_index//:dirs",
        "@crate_index//:base64",
        "@crate_index//:hex",
    ],
    proc_macro_deps = [
        "@crate_index//:serde_derive",
    ],
)

# DID identity binary
rust_binary(
    name = "polymera_identity_bin",
    srcs = ["src/main.rs"],
    deps = [
        ":polymera_identity",
        "@crate_index//:clap",
        "@crate_index//:tracing_subscriber",
    ],
)

# Unit tests
rust_test(
    name = "polymera_identity_tests",
    srcs = ["src/tests.rs"],
    deps = [":polymera_identity"],
)

rust_test(
    name = "did_tests",
    srcs = ["src/did.rs"],
    deps = [":polymera_identity"],
)

rust_test(
    name = "dstore_tests",
    srcs = ["src/dstore.rs"],
    deps = [":polymera_identity"],
)

rust_test(
    name = "lib_tests",
    srcs = ["src/lib.rs"],
    deps = [":polymera_identity"],
)

# Integration tests
rust_test(
    name = "integration_tests",
    srcs = ["src/tests.rs"],
    deps = [":polymera_identity"],
    size = "large",
    timeout = "long",
)

# Performance benchmarks
rust_benchmark(
    name = "identity_benchmarks",
    srcs = ["src/tests.rs"],
    deps = [":polymera_identity"],
)

# Test suite
test_suite(
    name = "polymera_identity_test_suite",
    tests = [
        ":polymera_identity_tests",
        ":did_tests",
        ":dstore_tests",
        ":lib_tests",
        ":integration_tests",
    ],
)

# Documentation
filegroup(
    name = "identity_docs",
    srcs = [
        "README.md",
        "docs/",
    ],
    visibility = ["//visibility:public"],
)

# Configuration files
filegroup(
    name = "identity_config",
    srcs = [
        "config/",
        "*.toml",
        "*.yaml",
    ],
    visibility = ["//visibility:public"],
)

# All identity components
filegroup(
    name = "polymera_identity_all",
    srcs = [
        ":polymera_identity",
        ":polymera_identity_bin",
        ":identity_docs",
        ":identity_config",
    ],
    visibility = ["//visibility:public"],
)
