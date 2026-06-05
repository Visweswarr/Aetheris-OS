load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library")

package(default_visibility = ["//visibility:public"])

rust_library(
    name = "posixnet",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        "//crates/aetheris-cap",
        "//services/policy",
        "@crates_io//:tokio",
        "@crates_io//:serde",
        "@crates_io//:serde_json",
        "@crates_io//:anyhow",
        "@crates_io//:thiserror",
        "@crates_io//:tracing",
        "@crates_io//:tracing-subscriber",
        "@crates_io//:async-trait",
        "@crates_io//:futures",
        "@crates_io//:bytes",
        "@crates_io//:uuid",
        "@crates_io//:dashmap",
        "@crates_io//:parking_lot",
        "@crates_io//:once_cell",
        "@crates_io//:regex",
        "@crates_io//:cidr-utils",
        "@crates_io//:dns-lookup",
        "@crates_io//:libc",
        "@crates_io//:nix",
        "@crates_io//:wasmtime",
        "@crates_io//:regorus",
        "@crates_io//:rustls",
        "@crates_io//:rustls-pemfile",
        "@crates_io//:webpki",
        "@crates_io//:webpki-roots",
        "@crates_io//:quinn",
        "@crates_io//:quinn-proto",
        "@crates_io//:libp2p",
        "@crates_io//:libp2p-quic",
        "@crates_io//:libp2p-noise",
        "@crates_io//:libp2p-yamux",
        "@crates_io//:libp2p-mplex",
        "@crates_io//:libp2p-identify",
        "@crates_io//:libp2p-ping",
        "@crates_io//:libp2p-kad",
        "@crates_io//:libp2p-gossipsub",
        "@crates_io//:chrono",
    ],
    proc_macro_deps = [
        "@crates_io//:serde_derive",
    ],
)

rust_binary(
    name = "posixnetd",
    srcs = ["src/main.rs"],
    deps = [
        ":posixnet",
        "@crates_io//:tokio",
        "@crates_io//:tracing",
        "@crates_io//:tracing-subscriber",
    ],
)

rust_binary(
    name = "posixnet-bench",
    srcs = ["benches/socket_performance.rs"],
    deps = [
        ":posixnet",
        "@crates_io//:criterion",
        "@crates_io//:tokio",
    ],
)

rust_test(
    name = "posixnet_tests",
    srcs = glob(["src/**/*_test.rs", "tests/**/*.rs"]),
    deps = [
        ":posixnet",
        "@crates_io//:tokio-test",
        "@crates_io//:tokio",
    ],
)
