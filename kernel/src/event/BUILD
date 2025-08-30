load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "event",
    srcs = glob(["*.rs"]),
    deps = [
        "//kernel:kernel_common",
        "//kernel/src/intent:intent",
        "//kernel/src/world:world",
        "//kernel/src/skills:skills",
        "serde",
        "serde_cbor",
        "alloc",
        "spin",
    ],
)
