load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

package(default_visibility = ["//visibility:public"])

rust_library(
    name = "ngfs",
    srcs = [
        "schema.rs",
        "src/cid.rs",
        "src/cas.rs",
        "src/enc.rs",
        "src/manifest.rs",
        "src/mount.rs",
        "src/resolve.rs",
        "src/read.rs",
        "src/snapshot.rs",
        "src/ipfs.rs",
        "src/lib.rs",
    ],
    edition = "2021",
    deps = [
        "@crate_index//:serde",
        "@crate_index//:serde_cbor",
        "@crate_index//:alloc",
        "@crate_index//:blake3",
        "@crate_index//:hex",
        "@crate_index//:zeroize",
        "@crate_index//:thiserror",
        "@crate_index//:rand",
        "@crate_index//:bs58",
        "//rust/bindings/polycrypto-sys",
    ],
    visibility = ["//visibility:public"],
)

rust_test(
    name = "ngfs_schema_test",
    srcs = ["schema.rs"],
    edition = "2021",
    deps = [":ngfs", "@crate_index//:hex"],
)

rust_test(
    name = "ngfs_encryption_test",
    srcs = ["src/enc.rs"],
    edition = "2021",
    deps = [":ngfs", "@crate_index//:tempfile"],
)

filegroup(
    name = "ngfs_srcs",
    srcs = ["schema.rs"],
)

filegroup(
    name = "ngfs_all",
    srcs = [
        ":ngfs",
        ":ngfs_schema_test",
    ],
)
