load("@rules_cc//cc:defs.bzl", "cc_library")

cc_library(
    name = "polycrypto",
    srcs = [
        "xchacha.c",
        "memzero.c",
    ],
    hdrs = ["polycrypto.h"],
    copts = [
        "-O2",
        "-fno-omit-frame-pointer",
        "-fstack-protector-strong",
        "-fPIC",
        "-Werror",
        "-Wall",
        "-Wextra",
        "-Wpedantic",
    ],
    defines = ["AETH_CT=1"],
    deps = [],
    visibility = ["//visibility:public"],
)
