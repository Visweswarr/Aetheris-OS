load("@rules_cc//cc:defs.bzl", "cc_library")

cc_library(
    name = "libngfs_merkle",
    srcs = ["ngfs_merkle.c"],
    hdrs = ["ngfs_merkle.h"],
    deps = [
        "@blake3//:blake3",
    ],
    copts = [
        "-O2",
        "-fno-omit-frame-pointer",
        "-fstack-protector-strong",
        "-fPIC",
        "-Werror",
        "-Wall",
        "-Wextra",
    ],
    defines = [
        "NGFS_TEST=1",
    ],
    visibility = ["//visibility:public"],
)
