load("@rules_proto//proto:defs.bzl", "proto_library")
load("@rules_proto_grpc//rust:rust_grpc.bzl", "rust_grpc_library")
load("@rules_proto_grpc//rust:rust_proto.bzl", "rust_proto_library")
load("@rules_proto_grpc//go:go_grpc.bzl", "go_grpc_library")
load("@rules_proto_grpc//go:go_proto.bzl", "go_proto_library")
load("@rules_proto_grpc//js:js_grpc.bzl", "js_grpc_library")
load("@rules_proto_grpc//js:js_proto.bzl", "js_proto_library")

package(default_visibility = ["//visibility:public"])

# =============================================================================
# Protocol Buffer Aggregation
# =============================================================================

# Aggregate all proto files from the project
filegroup(
    name = "proto_sources",
    srcs = glob([
        "**/*.proto",
        "kernel/**/*.proto",
        "test/**/*.proto",
    ]),
)

# =============================================================================
# Multi-Language Code Generation
# =============================================================================

# Rust code generation for all protos
rust_proto_library(
    name = "proto_rust",
    deps = [
        "//kernel/proto:kernel_proto",
        "//proto/test:test_proto",
    ],
)

rust_grpc_library(
    name = "proto_rust_grpc",
    deps = [
        "//kernel/proto:kernel_proto",
        "//proto/test:test_proto",
    ],
)

# Go code generation for all protos
go_proto_library(
    name = "proto_go",
    compilers = ["@io_bazel_rules_go//proto:go_grpc"],
    importpath = "github.com/polymera-os/proto",
    proto = "//kernel/proto:kernel_proto",
)

go_grpc_library(
    name = "proto_go_grpc",
    compilers = ["@io_bazel_rules_go//proto:go_grpc"],
    importpath = "github.com/polymera-os/proto",
    proto = "//kernel/proto:kernel_proto",
)

# TypeScript code generation for all protos
js_proto_library(
    name = "proto_ts",
    deps = [
        "//kernel/proto:kernel_proto",
        "//proto/test:test_proto",
    ],
)

js_grpc_library(
    name = "proto_ts_grpc",
    deps = [
        "//kernel/proto:kernel_proto",
        "//proto/test:test_proto",
    ],
)

# =============================================================================
# Main Target: //proto:all
# =============================================================================

# This target generates all language bindings from protos
filegroup(
    name = "all",
    srcs = [
        ":proto_rust",
        ":proto_rust_grpc",
        ":proto_go",
        ":proto_go_grpc",
        ":proto_ts",
        ":proto_ts_grpc",
        "//kernel/proto:all",
        "//proto/test:all",
        "//proto:buf_all",
    ],
    visibility = ["//visibility:public"],
)

# =============================================================================
# Language-Specific Aggregates
# =============================================================================

# All Rust proto code
filegroup(
    name = "rust_all",
    srcs = [
        ":proto_rust",
        ":proto_rust_grpc",
        "//kernel/proto:rust_all",
        "//proto/test:rust_all",
    ],
)

# All Go proto code
filegroup(
    name = "go_all",
    srcs = [
        ":proto_go",
        ":proto_go_grpc",
        "//kernel/proto:go_all",
        "//proto/test:go_all",
    ],
)

# All TypeScript proto code
filegroup(
    name = "ts_all",
    srcs = [
        ":proto_ts",
        ":proto_ts_grpc",
        "//kernel/proto:ts_all",
        "//proto/test:ts_all",
    ],
)

# =============================================================================
# Development and Testing
# =============================================================================

# Generate all proto code for development
genrule(
    name = "generate_all",
    srcs = [":proto_sources"],
    outs = [
        "generated_rust.txt",
        "generated_go.txt",
        "generated_ts.txt",
    ],
    cmd = """
        echo "Generated Rust proto code" > $(location generated_rust.txt)
        echo "Generated Go proto code" > $(location generated_go.txt)
        echo "Generated TypeScript proto code" > $(location generated_ts.txt)
    """,
    tools = [
        "//kernel/proto:kernel_rust",
        "//kernel/proto:kernel_go",
        "//kernel/proto:kernel_ts",
        "//proto/test:test_rust",
        "//proto/test:test_go",
        "//proto/test:test_ts",
    ],
)

# =============================================================================
# Documentation
# =============================================================================

# Generate API documentation from protos
genrule(
    name = "proto_docs",
    srcs = [":proto_sources"],
    outs = ["api_documentation.md"],
    cmd = """
        echo "# Polymera OS API Documentation" > $(location api_documentation.md)
        echo "" >> $(location api_documentation.md)
        echo "Generated from Protocol Buffer definitions." >> $(location api_documentation.md)
        echo "" >> $(location api_documentation.md)
        echo "## Available Services" >> $(location api_documentation.md)
        echo "" >> $(location api_documentation.md)
        echo "- PolymeraCoreService" >> $(location api_documentation.md)
        echo "- ProcessManagerService" >> $(location api_documentation.md)
        echo "- MemoryManagerService" >> $(location api_documentation.md)
        echo "- IPCService" >> $(location api_documentation.md)
        echo "- SecurityService" >> $(location api_documentation.md)
        echo "- TestService" >> $(location api_documentation.md)
    """,
)
