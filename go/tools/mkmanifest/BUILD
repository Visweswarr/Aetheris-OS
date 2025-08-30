load("@io_bazel_rules_go//go:def.bzl", "go_binary")

go_binary(
    name = "aeth-ngfs-mkmanifest",
    srcs = ["main.go"],
    cgo = True,
    deps = [
        "//go:go_default_library",
    ],
    data = [
        "//c/ngfs:libngfs_merkle",
    ],
    linkopts = [
        "-L$(BINDIR)/c/ngfs",
        "-lngfs_merkle",
        "-lblake3",
    ],
    visibility = ["//visibility:public"],
)
