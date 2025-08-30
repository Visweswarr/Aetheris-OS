load("@rules_rust//rust:defs.bzl", "rust_library", "rust_binary", "rust_test")
load("@rules_cc//cc:defs.bzl", "cc_library")

package(default_visibility = ["//visibility:public"])

# =============================================================================
# Rust Libraries
# =============================================================================

# Core kernel library
rust_library(
    name = "kernel_lib",
    srcs = glob([
        "*.rs",
        "core/*.rs",
        "memory/*.rs",
        "process/*.rs",
        "ipc/*.rs",
        "security/*.rs",
        "world/*.rs",
        "skills/*.rs",
        "event/*.rs",
        "policy/*.rs",
    ]),
    edition = "2021",
    deps = [
        "//kernel/proto:kernel_rust",
        "@crate_index//:tokio",
        "@crate_index//:tonic",
        "@crate_index//:prost",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:log",
        "@crate_index//:env_logger",
    ],
)

# Memory management library
rust_library(
    name = "memory_lib",
    srcs = glob(["memory/*.rs"]),
    edition = "2021",
    deps = [
        ":kernel_lib",
        "@crate_index//:bitvec",
        "@crate_index//:parking_lot",
    ],
)

# Process management library
rust_library(
    name = "process_lib",
    srcs = glob(["process/*.rs"]),
    edition = "2021",
    deps = [
        ":kernel_lib",
        "@crate_index//:uuid",
        "@crate_index//:chrono",
    ],
)

# IPC library
rust_library(
    name = "ipc_lib",
    srcs = glob(["ipc/*.rs"]),
    edition = "2021",
    deps = [
        ":kernel_lib",
        "@crate_index//:crossbeam-channel",
        "@crate_index//:serde",
    ],
)

# Security library
rust_library(
    name = "security_lib",
    srcs = glob(["security/*.rs"]),
    edition = "2021",
    deps = [
        ":kernel_lib",
        "@crate_index//:ring",
        "@crate_index//:ed25519-dalek",
        "@crate_index//:x25519-dalek",
    ],
)

# =============================================================================
# Rust Binaries
# =============================================================================

# Main kernel binary
rust_binary(
    name = "kernel_bin",
    srcs = ["main.rs"],
    edition = "2021",
    deps = [
        ":kernel_lib",
        ":memory_lib",
        ":process_lib",
        ":ipc_lib",
        ":security_lib",
    ],
)

# Kernel service binary
rust_binary(
    name = "kernel_service",
    srcs = ["service.rs"],
    edition = "2021",
    deps = [
        ":kernel_lib",
        "//kernel/proto:kernel_rust_grpc",
    ],
)

# =============================================================================
# C++ Libraries (for FFI)
# =============================================================================

# C++ FFI library for kernel operations
cc_library(
    name = "kernel_ffi",
    srcs = glob(["ffi/*.cpp"]),
    hdrs = glob(["ffi/*.h"]),
    deps = [
        ":kernel_lib",
    ],
    copts = [
        "-std=c++17",
        "-Wall",
        "-Wextra",
        "-O2",
    ],
    linkopts = [
        "-ldl",
        "-lpthread",
    ],
)

# =============================================================================
# Tests
# =============================================================================

# Unit tests for core kernel
rust_test(
    name = "kernel_tests",
    srcs = glob(["tests/*.rs"]),
    deps = [
        ":kernel_lib",
        "@crate_index//:tokio-test",
        "@crate_index//:mockall",
    ],
)

# Integration tests
rust_test(
    name = "kernel_integration_tests",
    srcs = glob(["integration_tests/*.rs"]),
    deps = [
        ":kernel_lib",
        ":memory_lib",
        ":process_lib",
        ":ipc_lib",
        ":security_lib",
        "@crate_index//:tokio-test",
        "@crate_index//:tempfile",
    ],
)

# Performance tests
rust_test(
    name = "kernel_benchmarks",
    srcs = glob(["benches/*.rs"]),
    deps = [
        ":kernel_lib",
        "@crate_index//:criterion",
    ],
)

# =============================================================================
# Documentation
# =============================================================================

# Generate documentation
genrule(
    name = "kernel_docs",
    srcs = [":kernel_lib"],
    outs = ["kernel_documentation.md"],
    cmd = """
        echo "# Polymera OS Kernel Documentation" > $(location kernel_documentation.md)
        echo "" >> $(location kernel_documentation.md)
        echo "Generated from Rust source code." >> $(location kernel_documentation.md)
        echo "" >> $(location kernel_documentation.md)
        echo "## Available Modules" >> $(location kernel_documentation.md)
        echo "" >> $(location kernel_documentation.md)
        echo "- Core Kernel" >> $(location kernel_documentation.md)
        echo "- Memory Management" >> $(location kernel_documentation.md)
        echo "- Process Management" >> $(location kernel_documentation.md)
        echo "- IPC System" >> $(location kernel_documentation.md)
        echo "- Security Framework" >> $(location kernel_documentation.md)
    """,
)

# =============================================================================
# Aggregated Targets
# =============================================================================

# All kernel libraries
filegroup(
    name = "kernel_libs",
    srcs = [
        ":kernel_lib",
        ":memory_lib",
        ":process_lib",
        ":ipc_lib",
        ":security_lib",
    ],
)

# All kernel binaries
filegroup(
    name = "kernel_bins",
    srcs = [
        ":kernel_bin",
        ":kernel_service",
    ],
)

# All kernel tests
filegroup(
    name = "kernel_tests",
    srcs = [
        ":kernel_tests",
        ":kernel_integration_tests",
        ":kernel_benchmarks",
    ],
)

# All kernel artifacts
filegroup(
    name = "all",
    srcs = [
        ":kernel_libs",
        ":kernel_bins",
        ":kernel_tests",
        ":kernel_docs",
    ],
)
