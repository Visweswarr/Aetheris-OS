load("@io_bazel_rules_go//go:def.bzl", "go_binary")

go_binary(
    name = "aeth-enc",
    srcs = ["main.go"],
    cgo = True,
    clinkopts = [
        "-L$(BINDIR)/c/crypto",
        "-lpolycrypto",
        "-lsodium",
    ],
    cdeps = ["//c/crypto:polycrypto"],
    visibility = ["//visibility:public"],
)
