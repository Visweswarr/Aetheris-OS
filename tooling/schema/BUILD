load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library")

package(default_visibility = ["//visibility:public"])

rust_library(
    name = "ngfs_schema_lib",
    srcs = ["ngfs_schema.rs"],
    edition = "2021",
    deps = [
        "@crate_index//:blake3",
        "@crate_index//:hex",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:chrono",
    ],
)

rust_binary(
    name = "ngfs_schema_gen",
    srcs = ["ngfs_schema.rs"],
    edition = "2021",
    deps = [
        ":ngfs_schema_lib",
    ],
)

filegroup(
    name = "ngfs_schema_all",
    srcs = [
        ":ngfs_schema_lib",
        ":ngfs_schema_gen",
    ],
)
