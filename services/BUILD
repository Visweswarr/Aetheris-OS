load("@io_bazel_rules_go//go:def.bzl", "go_library", "go_binary", "go_test")
load("@rules_python//python:defs.bzl", "py_library", "py_binary", "py_test")

package(default_visibility = ["//visibility:public"])

# =============================================================================
# Go Services
# =============================================================================

# Core service library
go_library(
    name = "core_service",
    srcs = glob(["go/core/*.go"]),
    importpath = "github.com/polymera-os/services/core",
    deps = [
        "//kernel/proto:kernel_go",
        "@com_github_gorilla_mux//:mux",
        "@com_github_gorilla_websocket//:websocket",
        "@org_golang_google_grpc//:grpc",
        "@org_golang_google_grpc//:grpclog",
    ],
)

# Process manager service
go_library(
    name = "process_manager",
    srcs = glob(["go/process/*.go"]),
    importpath = "github.com/polymera-os/services/process",
    deps = [
        ":core_service",
        "//kernel/proto:kernel_go",
        "@com_github_sirupsen_logrus//:logrus",
        "@org_golang_x_sys//:sys",
    ],
)

# Memory manager service
go_library(
    name = "memory_manager",
    srcs = glob(["go/memory/*.go"]),
    importpath = "github.com/polymera-os/services/memory",
    deps = [
        ":core_service",
        "//kernel/proto:kernel_go",
        "@com_github_prometheus_client_golang//:prometheus",
        "@com_github_prometheus_client_golang//:promauto",
    ],
)

# IPC service
go_library(
    name = "ipc_service",
    srcs = glob(["go/ipc/*.go"]),
    importpath = "github.com/polymera-os/services/ipc",
    deps = [
        ":core_service",
        "//kernel/proto:kernel_go",
        "@com_github_nats_io_nats_go//:nats",
        "@com_github_nats_io_nats_go//:stan",
    ],
)

# Security service
go_library(
    name = "security_service",
    srcs = glob(["go/security/*.go"]),
    importpath = "github.com/polymera-os/services/security",
    deps = [
        ":core_service",
        "//kernel/proto:kernel_go",
        "@com_github_golang_jwt_jwt_v4//:jwt",
        "@golang_org_x_crypto//:bcrypt",
        "@golang_org_x_crypto//:ssh",
    ],
)

# =============================================================================
# Go Binaries
# =============================================================================

# Main service orchestrator
go_binary(
    name = "service_orchestrator",
    srcs = ["go/main.go"],
    embed = [":core_service"],
    deps = [
        ":process_manager",
        ":memory_manager",
        ":ipc_service",
        ":security_service",
    ],
)

# Individual service binaries
go_binary(
    name = "process_service_bin",
    srcs = ["go/process/main.go"],
    embed = [":process_manager"],
)

go_binary(
    name = "memory_service_bin",
    srcs = ["go/memory/main.go"],
    embed = [":memory_manager"],
)

go_binary(
    name = "ipc_service_bin",
    srcs = ["go/ipc/main.go"],
    embed = [":ipc_service"],
)

go_binary(
    name = "security_service_bin",
    srcs = ["go/security/main.go"],
    embed = [":security_service"],
)

# =============================================================================
# Python Services
# =============================================================================

# Python service library
py_library(
    name = "python_service_lib",
    srcs = glob(["python/**/*.py"]),
    deps = [
        "//kernel/proto:kernel_ts",  # Using TS for Python integration
        "@pip//:grpcio",
        "@pip//:grpcio-tools",
        "@pip//:protobuf",
        "@pip//:asyncio",
        "@pip//:aiohttp",
    ],
)

# Python service binary
py_binary(
    name = "python_service",
    srcs = ["python/main.py"],
    deps = [":python_service_lib"],
)

# =============================================================================
# Tests
# =============================================================================

# Go unit tests
go_test(
    name = "core_service_test",
    srcs = glob(["go/core/*_test.go"]),
    embed = [":core_service"],
    deps = [
        "@org_golang_x_net//:context",
        "@com_github_stretchr_testify//:assert",
        "@com_github_stretchr_testify//:require",
    ],
)

go_test(
    name = "process_manager_test",
    srcs = glob(["go/process/*_test.go"]),
    embed = [":process_manager"],
    deps = [
        "@org_golang_x_net//:context",
        "@com_github_stretchr_testify//:assert",
        "@com_github_stretchr_testify//:require",
    ],
)

go_test(
    name = "memory_manager_test",
    srcs = glob(["go/memory/*_test.go"]),
    embed = [":memory_manager"],
    deps = [
        "@org_golang_x_net//:context",
        "@com_github_stretchr_testify//:assert",
        "@com_github_stretchr_testify//:require",
    ],
)

go_test(
    name = "ipc_service_test",
    srcs = glob(["go/ipc/*_test.go"]),
    embed = [":ipc_service"],
    deps = [
        "@org_golang_x_net//:context",
        "@com_github_stretchr_testify//:assert",
        "@com_github_stretchr_testify//:require",
    ],
)

go_test(
    name = "security_service_test",
    srcs = glob(["go/security/*_test.go"]),
    embed = [":security_service"],
    deps = [
        "@org_golang_x_net//:context",
        "@com_github_stretchr_testify//:assert",
        "@com_github_stretchr_testify//:require",
    ],
)

# Python tests
py_test(
    name = "python_service_test",
    srcs = glob(["python/**/*_test.py"]),
    deps = [":python_service_lib"],
)

# Integration tests
go_test(
    name = "services_integration_test",
    srcs = ["go/integration_test.go"],
    deps = [
        ":core_service",
        ":process_manager",
        ":memory_manager",
        ":ipc_service",
        ":security_service",
        "@com_github_stretchr_testify//:suite",
    ],
)

# =============================================================================
# Documentation
# =============================================================================

# Generate service documentation
genrule(
    name = "services_docs",
    srcs = [
        ":core_service",
        ":process_manager",
        ":memory_manager",
        ":ipc_service",
        ":security_service",
    ],
    outs = ["services_documentation.md"],
    cmd = """
        echo "# Polymera OS Services Documentation" > $(location services_documentation.md)
        echo "" >> $(location services_documentation.md)
        echo "Generated from Go and Python source code." >> $(location services_documentation.md)
        echo "" >> $(location services_documentation.md)
        echo "## Available Services" >> $(location services_documentation.md)
        echo "" >> $(location services_documentation.md)
        echo "- Core Service" >> $(location services_documentation.md)
        echo "- Process Manager" >> $(location services_documentation.md)
        echo "- Memory Manager" >> $(location services_documentation.md)
        echo "- IPC Service" >> $(location services_documentation.md)
        echo "- Security Service" >> $(location services_documentation.md)
        echo "- Python Service" >> $(location services_documentation.md)
    """,
)

# =============================================================================
# Aggregated Targets
# =============================================================================

# All Go libraries
filegroup(
    name = "go_libs",
    srcs = [
        ":core_service",
        ":process_manager",
        ":memory_manager",
        ":ipc_service",
        ":security_service",
    ],
)

# All Go binaries
filegroup(
    name = "go_bins",
    srcs = [
        ":service_orchestrator",
        ":process_service_bin",
        ":memory_service_bin",
        ":ipc_service_bin",
        ":security_service_bin",
    ],
)

# All Python services
filegroup(
    name = "python_services",
    srcs = [
        ":python_service_lib",
        ":python_service",
    ],
)

# All tests
filegroup(
    name = "service_tests",
    srcs = [
        ":core_service_test",
        ":process_manager_test",
        ":memory_manager_test",
        ":ipc_service_test",
        ":security_service_test",
        ":python_service_test",
        ":services_integration_test",
    ],
)

# All services
filegroup(
    name = "all",
    srcs = [
        ":go_libs",
        ":go_bins",
        ":python_services",
        ":service_tests",
        ":services_docs",
    ],
)
