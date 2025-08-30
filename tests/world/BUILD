package(default_visibility = ["//visibility:public"])

rust_test_suite(
    name = "world",
    srcs = glob(["*.rs"]),
    deps = [
        "//kernel:kernel_lib",
        "//tests:test_common",
    ],
    data = [
        "//world/schema:world.yaml",
    ],
    test_suite = True,
)
