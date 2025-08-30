load("@rules_proto//proto:defs.bzl", "proto_library")
load("@rules_proto_grpc//rust:rust_grpc.bzl", "rust_grpc_library")
load("@rules_proto_grpc//rust:rust_proto.bzl", "rust_proto_library")
load("@rules_proto_grpc//go:go_grpc.bzl", "go_grpc_library")
load("@rules_proto_grpc//go:go_proto.bzl", "go_proto_library")
load("@rules_proto_grpc//js:js_grpc.bzl", "js_grpc_library")
load("@rules_proto_grpc//js:js_proto.bzl", "js_proto_library")

package(default_visibility = ["//visibility:public"])

# =============================================================================
# Protocol Buffer Definitions
# =============================================================================

proto_library(
    name = "kernel_proto",
    srcs = ["kernel.proto"],
    deps = [
        "@com_google_protobuf//:timestamp_proto",
        "@com_google_protobuf//:any_proto",
    ],
)

# =============================================================================
# Rust Code Generation
# =============================================================================

# Rust protobuf library (without gRPC)
rust_proto_library(
    name = "kernel_rust_proto",
    deps = [":kernel_proto"],
)

# Rust gRPC library (with gRPC service definitions)
rust_grpc_library(
    name = "kernel_rust_grpc",
    deps = [":kernel_proto"],
)

# Combined Rust library with both proto and gRPC
rust_library(
    name = "kernel_rust",
    srcs = [],
    deps = [
        ":kernel_rust_proto",
        ":kernel_rust_grpc",
    ],
)

# =============================================================================
# Go Code Generation
# =============================================================================

# Go protobuf library (without gRPC)
go_proto_library(
    name = "kernel_go_proto",
    compilers = ["@io_bazel_rules_go//proto:go_grpc"],
    importpath = "github.com/polymera-os/kernel/proto",
    proto = ":kernel_proto",
)

# Go gRPC library (with gRPC service definitions)
go_grpc_library(
    name = "kernel_go_grpc",
    compilers = ["@io_bazel_rules_go//proto:go_grpc"],
    importpath = "github.com/polymera-os/kernel/proto",
    proto = ":kernel_proto",
)

# Combined Go library
go_library(
    name = "kernel_go",
    srcs = [],
    importpath = "github.com/polymera-os/kernel/proto",
    deps = [
        ":kernel_go_proto",
        ":kernel_go_grpc",
    ],
)

# =============================================================================
# TypeScript/JavaScript Code Generation
# =============================================================================

# TypeScript protobuf library (without gRPC)
js_proto_library(
    name = "kernel_ts_proto",
    deps = [":kernel_proto"],
)

# TypeScript gRPC library (with gRPC service definitions)
js_grpc_library(
    name = "kernel_ts_grpc",
    deps = [":kernel_proto"],
)

# Combined TypeScript library
ts_project(
    name = "kernel_ts",
    srcs = [],
    declaration = True,
    deps = [
        ":kernel_ts_proto",
        ":kernel_ts_grpc",
        "@npm//@types/node",
        "@npm//typescript",
    ],
)

# =============================================================================
# Aggregated Targets
# =============================================================================

# All proto-generated code
filegroup(
    name = "all",
    srcs = [
        ":kernel_rust",
        ":kernel_go",
        ":kernel_ts",
    ],
)

# All Rust proto code
filegroup(
    name = "rust_all",
    srcs = [
        ":kernel_rust_proto",
        ":kernel_rust_grpc",
        ":kernel_rust",
    ],
)

# All Go proto code
filegroup(
    name = "go_all",
    srcs = [
        ":kernel_go_proto",
        ":kernel_go_grpc",
        ":kernel_go",
    ],
)

# All TypeScript proto code
filegroup(
    name = "ts_all",
    srcs = [
        ":kernel_ts_proto",
        ":kernel_ts_grpc",
        ":kernel_ts",
    ],
)

# =============================================================================
# Testing Targets
# =============================================================================

# Rust tests
rust_test(
    name = "kernel_rust_test",
    srcs = ["kernel_test.rs"],
    deps = [":kernel_rust"],
)

# Go tests
go_test(
    name = "kernel_go_test",
    srcs = ["kernel_test.go"],
    embed = [":kernel_go"],
)

# TypeScript tests
ts_project(
    name = "kernel_ts_test",
    srcs = ["kernel_test.ts"],
    declaration = False,
    deps = [
        ":kernel_ts",
        "@npm//@types/jest",
        "@npm//jest",
    ],
)

# All tests
filegroup(
    name = "test_all",
    srcs = [
        ":kernel_rust_test",
        ":kernel_go_test",
        ":kernel_ts_test",
    ],
)
