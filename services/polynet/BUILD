load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test", "rust_binary")

package(default_visibility = ["//visibility:public"])

rust_library(
    name = "polymera_polynet",
    srcs = [
        "lib.rs",
        "quarantine.rs",
    ],
    deps = [
        "@crate_index//:anyhow",
        "@crate_index//:chrono",
        "@crate_index//:ed25519-dalek",
        "@crate_index//:hex",
        "@crate_index//:rand",
        "@crate_index//:serde",
        "@crate_index//:serde_json", 
        "@crate_index//:sha2",
        "@crate_index//:thiserror",
        "@crate_index//:tokio",
        "@crate_index//:tracing",
        "@crate_index//:tracing-subscriber",
        "@crate_index//:uuid",
    ],
    proc_macro_deps = [
        "@crate_index//:serde",
        "@crate_index//:thiserror",
        "@crate_index//:tokio",
        "@crate_index//:tracing",
    ],
)

rust_test(
    name = "quarantine_tests",
    srcs = ["quarantine.rs"],
    deps = [
        ":polymera_polynet",
        "@crate_index//:tempfile",
        "@crate_index//:tokio-test",
    ],
)

rust_test(
    name = "lib_tests", 
    srcs = ["lib.rs"],
    deps = [":polymera_polynet"],
)

# Integration tests
rust_test(
    name = "integration_tests",
    srcs = ["tests/integration_test.rs"],
    deps = [
        ":polymera_polynet",
        "@crate_index//:tempfile",
        "@crate_index//:tokio-test",
    ],
)

# Test suite
test_suite(
    name = "polynet_tests",
    tests = [
        ":quarantine_tests",
        ":lib_tests", 
        ":integration_tests",
    ],
)

# Example binaries
rust_binary(
    name = "quarantine_example",
    srcs = ["examples/basic_quarantine.rs"],
    deps = [
        ":polymera_polynet",
        "@crate_index//:tokio",
        "@crate_index//:tracing",
        "@crate_index//:tracing-subscriber",
    ],
)

# Performance benchmarks
rust_binary(
    name = "quarantine_bench",
    srcs = ["benches/quarantine_bench.rs"],
    deps = [
        ":polymera_polynet",
        "@crate_index//:criterion",
        "@crate_index//:tokio",
    ],
)