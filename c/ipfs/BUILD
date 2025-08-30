load("@rules_cc//cc:defs.bzl", "cc_library")

cc_library(
    name = "aeth_ipfs",
    srcs = [
        "cidv1.c",
        "multihash.c",
        "sha256.c",
        "varint.c",
        "multibase_b32.c",
    ],
    hdrs = ["libaeth_ipfs.h"],
    copts = [
        "-O2",
        "-fstack-protector-strong",
        "-Werror",
        "-Wall",
        "-Wextra",
        "-std=c11",
    ],
    visibility = ["//visibility:public"],
)
