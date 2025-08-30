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
    name = "test_proto",
    srcs = ["test.proto"],
    deps = [
        "@com_google_protobuf//:timestamp_proto",
    ],
)

# =============================================================================
# Rust Code Generation
# =============================================================================

# Rust protobuf library (without gRPC)
rust_proto_library(
    name = "test_rust_proto",
    deps = [":test_proto"],
)

# Rust gRPC library (with gRPC service definitions)
rust_grpc_library(
    name = "test_rust_grpc",
    deps = [":test_proto"],
)

# Combined Rust library
rust_library(
    name = "test_rust",
    srcs = [],
    deps = [
        ":test_rust_proto",
        ":test_rust_grpc",
    ],
)

# =============================================================================
# Go Code Generation
# =============================================================================

# Go protobuf library (without gRPC)
go_proto_library(
    name = "test_go_proto",
    compilers = ["@io_bazel_rules_go//proto:go_grpc"],
    importpath = "github.com/polymera-os/proto/test",
    proto = ":test_proto",
)

# Go gRPC library (with gRPC service definitions)
go_grpc_library(
    name = "test_go_grpc",
    compilers = ["@io_bazel_rules_go//proto:go_grpc"],
    importpath = "github.com/polymera-os/proto/test",
    proto = ":test_proto",
)

# Combined Go library
go_library(
    name = "test_go",
    srcs = [],
    importpath = "github.com/polymera-os/proto/test",
    deps = [
        ":test_go_proto",
        ":test_go_grpc",
    ],
)

# =============================================================================
# TypeScript Code Generation
# =============================================================================

# TypeScript protobuf library (without gRPC)
js_proto_library(
    name = "test_ts_proto",
    deps = [":test_proto"],
)

# TypeScript gRPC library (with gRPC service definitions)
js_grpc_library(
    name = "test_ts_grpc",
    deps = [":test_proto"],
)

# Combined TypeScript library
ts_project(
    name = "test_ts",
    srcs = [],
    declaration = True,
    deps = [
        ":test_ts_proto",
        ":test_ts_grpc",
        "@npm//@types/node",
        "@npm//typescript",
    ],
)

# =============================================================================
# Aggregated Targets
# =============================================================================

# All test proto-generated code
filegroup(
    name = "all",
    srcs = [
        ":test_rust",
        ":test_go",
        ":test_ts",
    ],
    visibility = ["//visibility:public"],
)

# All Rust test proto code
filegroup(
    name = "rust_all",
    srcs = [
        ":test_rust_proto",
        ":test_rust_grpc",
        ":test_rust",
    ],
)

# All Go test proto code
filegroup(
    name = "go_all",
    srcs = [
        ":test_go_proto",
        ":test_go_grpc",
        ":test_go",
    ],
)

# All TypeScript test proto code
filegroup(
    name = "ts_all",
    srcs = [
        ":test_ts_proto",
        ":test_ts_grpc",
        ":test_ts",
    ],
)

# =============================================================================
# Testing Targets
# =============================================================================

# Rust tests
rust_test(
    name = "test_rust_test",
    srcs = ["test_rust_test.rs"],
    deps = [":test_rust"],
)

# Go tests
go_test(
    name = "test_go_test",
    srcs = ["test_go_test.go"],
    embed = [":test_go"],
)

# TypeScript tests
ts_project(
    name = "test_ts_test",
    srcs = ["test_ts_test.ts"],
    declaration = False,
    deps = [
        ":test_ts",
        "@npm//@types/jest",
        "@npm//jest",
    ],
)

# All tests
filegroup(
    name = "test_all",
    srcs = [
        ":test_rust_test",
        ":test_go_test",
        ":test_ts_test",
    ],
)
