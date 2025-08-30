# Root BUILD file for Polymera OS
# This file defines lint targets and build rules

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test")
load("@rules_proto//proto:defs.bzl", "proto_library")
load("@rules_proto_grpc//rust:rust_grpc.bzl", "rust_grpc_library")
load("@rules_proto_grpc//rust:rust_proto.bzl", "rust_proto_library")
load("@rules_proto_grpc//go:go_grpc.bzl", "go_grpc_library")
load("@rules_proto_grpc//go:go_proto.bzl", "go_proto_library")
load("@rules_proto_grpc//js:js_grpc.bzl", "js_grpc_library")
load("@rules_proto_grpc//js:js_proto.bzl", "js_proto_library")

package(default_visibility = ["//visibility:public"])

# =============================================================================
# Project Overview
# =============================================================================
# Polymera OS - Next-generation quantum-ready operating system
# This BUILD file provides the main entry points for building the entire project

# =============================================================================
# Protocol Buffer Aggregation
# =============================================================================

# Aggregate all proto files from the project
filegroup(
    name = "proto_sources",
    srcs = glob([
        "**/*.proto",
        "kernel/**/*.proto",
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
    ],
)

rust_grpc_library(
    name = "proto_rust_grpc",
    deps = [
        "//kernel/proto:kernel_proto",
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
    proto = "//kernel/proto:kernel_proto",
)

# TypeScript code generation for all protos
js_proto_library(
    name = "proto_ts",
    deps = [
        "//kernel/proto:kernel_proto",
    ],
)

js_grpc_library(
    name = "proto_ts_grpc",
    deps = [
        "//kernel/proto:kernel_proto",
    ],
)

# =============================================================================
# Main Target: //proto:all
# =============================================================================

# This target generates all language bindings from protos
filegroup(
    name = "proto_all",
    srcs = [
        ":proto_rust",
        ":proto_rust_grpc",
        ":proto_go",
        ":proto_go_grpc",
        ":proto_ts",
        ":proto_ts_grpc",
        "//proto:all",
    ],
    visibility = ["//visibility:public"],
)

# =============================================================================
# Language-Specific Aggregates
# =============================================================================

# All Rust proto code
filegroup(
    name = "proto_rust_all",
    srcs = [
        ":proto_rust",
        ":proto_rust_grpc",
        "//proto:rust_all",
    ],
)

# All Go proto code
filegroup(
    name = "proto_go_all",
    srcs = [
        ":proto_go",
        ":proto_go_grpc",
        "//proto:go_all",
    ],
)

# All TypeScript proto code
filegroup(
    name = "proto_ts_all",
    srcs = [
        ":proto_ts",
        ":proto_ts_grpc",
        "//proto:ts_all",
    ],
)

# =============================================================================
# Component Aggregation
# =============================================================================

# All kernel components
filegroup(
    name = "kernel_all",
    srcs = [
        "//kernel:all",
        "//kernel/proto:all",
    ],
)

# All service components
filegroup(
    name = "services_all",
    srcs = [
        "//services:all",
    ],
)

# All UI components
filegroup(
    name = "ui_all",
    srcs = [
        "//ui:all",
    ],
)

# All tooling components
filegroup(
    name = "tooling_all",
    srcs = [
        "//tooling/bazel:all",
    ],
)

# =============================================================================
# Development and Testing
# =============================================================================

# Generate all proto code for development
genrule(
    name = "generate_all_proto",
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
        "//proto:generate_all",
    ],
)

# Generate project documentation
genrule(
    name = "project_docs",
    srcs = [
        "//kernel:kernel_docs",
        "//services:services_docs",
        "//ui:ui_docs",
        "//proto:proto_docs",
    ],
    outs = ["project_documentation.md"],
    cmd = """
        echo "# Polymera OS Project Documentation" > $(location project_documentation.md)
        echo "" >> $(location project_documentation.md)
        echo "Generated from source code and configuration." >> $(location project_documentation.md)
        echo "" >> $(location project_documentation.md)
        echo "## Available Components" >> $(location project_documentation.md)
        echo "" >> $(location project_documentation.md)
        echo "- Kernel System" >> $(location project_documentation.md)
        echo "- System Services" >> $(location project_documentation.md)
        echo "- User Interface" >> $(location project_documentation.md)
        echo "- Protocol Buffers" >> $(location project_documentation.md)
        echo "- Development Tooling" >> $(location project_documentation.md)
    """,
)

# =============================================================================
# Build and Test Targets
# =============================================================================

# Build entire project
filegroup(
    name = "build_all",
    srcs = [
        ":kernel_all",
        ":services_all",
        ":ui_all",
        ":tooling_all",
        ":proto_all",
    ],
)

# Test entire project
filegroup(
    name = "test_all",
    srcs = [
        "//kernel:kernel_tests",
        "//services:service_tests",
        "//ui:ui_tests",
        "//proto:test_all",
    ],
)

# =============================================================================
# Convenience Targets
# =============================================================================

# Quick build targets
filegroup(
    name = "quick_build",
    srcs = [
        "//kernel:kernel_bin",
        "//services:service_orchestrator",
        "//ui:web_app",
    ],
)

# Development targets
filegroup(
    name = "dev_targets",
    srcs = [
        "//kernel:kernel_libs",
        "//services:go_libs",
        "//ui:ts_libs",
    ],
)

# Production targets
filegroup(
    name = "prod_targets",
    srcs = [
        "//kernel:kernel_bins",
        "//services:go_bins",
        "//ui:ts_apps",
    ],
)

# =============================================================================
# Documentation Aggregation
# =============================================================================

# All project documentation
filegroup(
    name = "docs_all",
    srcs = [
        ":project_docs",
        "//kernel:kernel_docs",
        "//services:services_docs",
        "//ui:ui_docs",
        "//proto:proto_docs",
        "//tooling/bazel:README.md",
        "docs/DEV_SETUP.md",
        "README.md",
        "DESIGN.md",
        "SPEC.md",
    ],
)

# =============================================================================
# Main Project Target
# =============================================================================

# Complete project target
filegroup(
    name = "all",
    srcs = [
        ":build_all",
        ":test_all",
        ":docs_all",
        ":generate_all_proto",
    ],
    visibility = ["//visibility:public"],
)
