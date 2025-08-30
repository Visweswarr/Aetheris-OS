load("@rules_rust//rust:defs.bzl", "rust_library", "rust_binary", "rust_test", "rust_benchmark")

package(default_visibility = ["//visibility:public"])

# Main keystore library
rust_library(
    name = "polymera_wallet",
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
        "@crate_index//:aes",
        "@crate_index//:aes_gcm",
        "@crate_index//:chacha20poly1305",
        "@crate_index//:argon2",
        "@crate_index//:zeroize",
        "@crate_index//:secrecy",
        "@crate_index//:sled",
        "@crate_index//:dashmap",
        "@crate_index//:anyhow",
        "@crate_index//:clap",
        "@crate_index//:config",
        "@crate_index//:dirs",
        "@crate_index//:base64",
        "@crate_index//:hex",
        "@crate_index//:async_trait",
    ],
    proc_macro_deps = [
        "@crate_index//:serde_derive",
    ],
)

# Wallet binary
rust_binary(
    name = "polymera_wallet_bin",
    srcs = ["src/main.rs"],
    deps = [
        ":polymera_wallet",
        "@crate_index//:clap",
        "@crate_index//:tracing_subscriber",
        "@crate_index//:serde_yaml",
        "@crate_index//:base64",
        "@crate_index//:hex",
    ],
)

# Unit tests
rust_test(
    name = "keystore_tests",
    srcs = ["src/keystore.rs"],
    deps = [":polymera_wallet"],
)

rust_test(
    name = "backend_tests",
    srcs = ["src/backend/mod.rs"],
    deps = [":polymera_wallet"],
)

rust_test(
    name = "software_backend_tests",
    srcs = ["src/backend/software.rs"],
    deps = [":polymera_wallet"],
)

rust_test(
    name = "keyvault_backend_tests",
    srcs = ["src/backend/keyvault.rs"],
    deps = [":polymera_wallet"],
)

rust_test(
    name = "lib_tests",
    srcs = ["src/lib.rs"],
    deps = [":polymera_wallet"],
)

rust_test(
    name = "session_tests",
    srcs = ["src/session.rs"],
    deps = [":polymera_wallet"],
)

# Integration tests
rust_test(
    name = "integration_tests",
    srcs = ["src/tests.rs"],
    deps = [":polymera_wallet"],
    size = "large",
)

# Performance benchmarks
rust_benchmark(
    name = "keystore_benchmarks",
    srcs = ["src/benches/keystore_benchmarks.rs"],
    deps = [":polymera_wallet"],
)

# Test suite
test_suite(
    name = "wallet_tests",
    tests = [
        ":keystore_tests",
        ":backend_tests",
        ":software_backend_tests",
        ":keyvault_backend_tests",
        ":lib_tests",
        ":session_tests",
        ":integration_tests",
    ],
)

# Documentation
filegroup(
    name = "wallet_docs",
    srcs = [
        "README.md",
        "docs/",
    ],
    visibility = ["//visibility:public"],
)

# Configuration files
filegroup(
    name = "wallet_config",
    srcs = [
        "config/",
        "*.yaml",
        "*.yml",
        "*.toml",
    ],
    visibility = ["//visibility:public"],
)

# All wallet components
filegroup(
    name = "wallet_all",
    srcs = [
        ":polymera_wallet",
        ":polymera_wallet_bin",
        ":wallet_docs",
        ":wallet_config",
    ],
    visibility = ["//visibility:public"],
)
