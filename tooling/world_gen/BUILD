package(default_visibility = ["//visibility:public"])

rust_binary(
    name = "gen",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        "@crates//:serde",
        "@crates//:serde_yaml",
        "@crates//:tera",
        "@crates//:sha2",
        "@crates//:hex",
        "@crates//:clap",
        "@crates//:anyhow",
    ],
    data = [
        "//world/schema:world.yaml",
        "templates/schema.rs.tera",
        "templates/polymera_world.h.tera",
        "templates/SCHEMA.md.tera",
    ],
)
