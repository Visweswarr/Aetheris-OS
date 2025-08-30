load("@rules_rust//rust:defs.bzl", "rust_binary")

package(default_visibility = ["//visibility:public"])

# IPC Performance Check Script
rust_binary(
    name = "check_ipc",
    srcs = ["check_ipc.rs"],
    edition = "2021",
    deps = [],
    data = [],
    tags = [
        "manual",  # Don't build by default
        "perf",    # Performance testing
    ],
)

# Test target for the performance script
rust_binary(
    name = "check_ipc_test",
    srcs = ["check_ipc.rs"],
    edition = "2021",
    deps = [],
    data = [],
    testonly = 1,
    tags = [
        "test",
        "perf",
    ],
)

# Wake-to-Run Latency Performance Check Script
rust_binary(
    name = "check_wake_to_run",
    srcs = ["check_wake_to_run.rs"],
    edition = "2021",
    deps = [],
    data = [],
    tags = [
        "manual",  # Don't build by default
        "perf",    # Performance testing
    ],
)

# Test target for wake-to-run performance script
rust_binary(
    name = "check_wake_to_run_test",
    srcs = ["check_wake_to_run.rs"],
    edition = "2021",
    deps = [],
    data = [],
    testonly = 1,
    tags = [
        "test",
        "perf",
    ],
)
