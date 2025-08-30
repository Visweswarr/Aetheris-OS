load("@rules_rust//rust:defs.bzl", "rust_test")

package(default_visibility = ["//visibility:public"])

rust_test(
    name = "ngfs_schema_hash_test",
    srcs = ["schema_hash.rs"],
    edition = "2021",
    deps = ["//services/ngfs", "@crate_index//:hex"],
)

rust_test(
    name = "ngfs_schema_roundtrip_test",
    srcs = ["schema_roundtrip.rs"],
    edition = "2021",
    deps = ["//services/ngfs", "@crate_index//:serde_cbor"],
)

rust_test(
    name = "ngfs_cas_roundtrip_test",
    srcs = ["cas_roundtrip.rs"],
    edition = "2021",
    deps = ["//services/ngfs", "@crate_index//:tempfile"],
)

rust_test(
    name = "ngfs_cas_recovery_test",
    srcs = ["cas_recovery.rs"],
    edition = "2021",
    deps = ["//services/ngfs", "@crate_index//:tempfile"],
)

rust_test(
    name = "ngfs_cas_perf_test",
    srcs = ["cas_perf.rs"],
    edition = "2021",
    deps = ["//services/ngfs", "@crate_index//:tempfile"],
)

rust_test(
    name = "ngfs_mount_and_stat_test",
    srcs = ["mount_and_stat.rs"],
    edition = "2021",
    deps = ["//services/ngfs", "@crate_index//:tempfile"],
)

rust_test(
    name = "ngfs_read_roundtrip_test",
    srcs = ["read_roundtrip.rs"],
    edition = "2021",
    deps = ["//services/ngfs", "@crate_index//:tempfile"],
)

rust_test(
    name = "ngfs_snapshot_build_test",
    srcs = ["snapshot_build.rs"],
    edition = "2021",
    deps = ["//services/ngfs", "@crate_index//:tempfile"],
)

rust_test(
    name = "ngfs_error_paths_test",
    srcs = ["error_paths.rs"],
    edition = "2021",
    deps = ["//services/ngfs", "@crate_index//:tempfile"],
)

rust_test(
    name = "ngfs_perf_ro_test",
    srcs = ["perf_ro.rs"],
    edition = "2021",
    deps = ["//services/ngfs", "@crate_index//:tempfile"],
)

rust_test(
    name = "ngfs_ipfs_map_vectors_test",
    srcs = ["ipfs_map_vectors.rs"],
    edition = "2021",
    deps = ["//services/ngfs", "@crate_index//:tempfile"],
)

rust_test(
    name = "ngfs_ipfs_car_smoke_test",
    srcs = ["ipfs_car_smoke.rs"],
    edition = "2021",
    deps = ["//services/ngfs", "@crate_index//:tempfile"],
)

filegroup(
    name = "ngfs_tests_all",
    srcs = [
        ":ngfs_schema_hash_test",
        ":ngfs_schema_roundtrip_test",
        ":ngfs_cas_roundtrip_test",
        ":ngfs_cas_recovery_test",
        ":ngfs_cas_perf_test",
                            ":ngfs_mount_and_stat_test",
                    ":ngfs_read_roundtrip_test",
                    ":ngfs_snapshot_build_test",
                    ":ngfs_error_paths_test",
                    ":ngfs_perf_ro_test",
                    ":ngfs_ipfs_map_vectors_test",
                    ":ngfs_ipfs_car_smoke_test",
    ],
)
