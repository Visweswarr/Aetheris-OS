load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "polycrypto-sys",
    srcs = ["src/lib.rs"],
    edition = "2021",
    deps = [
        "//c/crypto:polycrypto",
        "@crate_index//:cc",
    ],
    features = ["ffi"],
    visibility = ["//visibility:public"],
)
