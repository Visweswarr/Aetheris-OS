load("@rules_rust//rust:defs.bzl", "rust_test_suite")

rust_test_suite(
    name = "policy",
    srcs = glob(["*.rs"]),
    deps = [
        "//kernel:kernel_lib",
        "//tests:test_common",
    ],
    data = [
        "//policy:policy_rules",
    ],
)
