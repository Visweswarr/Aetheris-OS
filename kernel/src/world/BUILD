package(default_visibility = ["//visibility:public"])

rust_library(
    name = "world",
    srcs = glob(["*.rs"]),
    deps = [
        "//kernel:kernel_common",
        "@crates//:serde",
        "@crates//:serde_cbor",
        "@crates//:alloc",
        "@crates//:spin",
    ],
)
