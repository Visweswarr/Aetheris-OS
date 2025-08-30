load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "policy",
    srcs = glob(["*.rs"]),
    deps = [
        "//kernel:kernel_common",
        "//kernel/src/event:event",
        "//kernel/src/intent:intent",
        "//kernel/src/world:world",
        "//kernel/src/skills:skills",
        "serde",
        "serde_cbor",
        "blake3",
        "alloc",
        "spin",
    ],
)
