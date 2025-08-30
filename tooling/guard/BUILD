load("@rules_rust//rust:defs.bzl", "rust_binary")

rust_binary(
    name = "surface_check",
    srcs = glob(["src/**/*.rs"]),
    edition = "2021",
    deps = [
        "@crate_index//:serde",
        "@crate_index//:serde_yaml",
        "@crate_index//:serde_json",
        "@crate_index//:sha2",
        "@crate_index//:walkdir",
        "@crate_index//:anyhow",
        "@crate_index//:thiserror",
        "@crate_index//:clap",
        "@crate_index//:colored",
        "@crate_index//:chrono",
        "@crate_index//:glob",
        "@crate_index//:tempfile",
        "@crate_index//:log",
        "@crate_index//:env_logger",
        "@crate_index//:tokio",
    ],
    visibility = ["//visibility:public"],
)
