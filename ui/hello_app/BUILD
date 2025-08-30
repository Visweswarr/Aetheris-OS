load("@npm//@bazel/typescript:index.bzl", "ts_project")
load("@npm//@bazel/rollup:index.bzl", "rollup_bundle")
load("@npm//@bazel/jest:index.bzl", "jest_test")
load("@rules_cc//cc:defs.bzl", "cc_binary", "cc_library")
load("@io_bazel_rules_go//go:def.bzl", "go_binary", "go_library")

package(default_visibility = ["//visibility:public"])

# TypeScript project configuration
ts_project(
    name = "hello_app_ts",
    srcs = glob(["src/**/*.ts", "src/**/*.tsx"]),
    declaration = True,
    tsconfig = "//ui/hello_app:tsconfig.json",
    deps = [
        "@npm//react",
        "@npm//react-dom",
        "@npm//@types/react",
        "@npm//@types/react-dom",
    ],
)

# Rollup bundle for production
rollup_bundle(
    name = "hello_app_bundle",
    entry_point = "src/index.tsx",
    deps = [
        ":hello_app_ts",
        "@npm//rollup",
        "@npm//@rollup/plugin-commonjs",
        "@npm//@rollup/plugin-node-resolve",
        "@npm//@rollup/plugin-typescript",
        "@npm//rollup-plugin-terser",
    ],
    config_file = "rollup.config.js",
    sourcemap = "hidden",
)

# Main application target
filegroup(
    name = "hello_app",
    srcs = [
        ":hello_app_bundle",
        "//ui/hello_app:index.html",
    ],
    data = [
        ":hello_app_bundle",
        "//ui/hello_app:index.html",
    ],
)

# Development server
filegroup(
    name = "hello_app_dev",
    srcs = [
        ":hello_app_ts",
        "//ui/hello_app:index.html",
    ],
    data = [
        ":hello_app_ts",
        "//ui/hello_app:index.html",
    ],
)

# Unit tests
jest_test(
    name = "hello_app_unit_tests",
    srcs = glob(["tests/**/*.test.ts", "tests/**/*.test.tsx"]),
    deps = [
        ":hello_app_ts",
        "@npm//jest",
        "@npm//@types/jest",
        "@npm//ts-jest",
    ],
    config = "jest.config.js",
    data = [
        "//ui/hello_app:package.json",
        "//ui/hello_app:tsconfig.json",
    ],
)

# Integration tests
jest_test(
    name = "hello_app_integration_tests",
    srcs = glob(["tests/integration/**/*.test.ts", "tests/integration/**/*.test.tsx"]),
    deps = [
        ":hello_app_ts",
        "@npm//jest",
        "@npm//@types/jest",
        "@npm//ts-jest",
        "@npm//@testing-library/react",
        "@npm//@testing-library/jest-dom",
    ],
    config = "jest.config.js",
    data = [
        "//ui/hello_app:package.json",
        "//ui/hello_app:tsconfig.json",
    ],
)

# E2E tests
filegroup(
    name = "hello_app_e2e_tests",
    srcs = glob(["tests/e2e/**/*.ts", "tests/e2e/**/*.js"]),
    data = [
        ":hello_app",
        "//ui/hello_app:package.json",
    ],
)

# Performance tests
jest_test(
    name = "hello_app_performance_tests",
    srcs = glob(["tests/performance/**/*.test.ts", "tests/performance/**/*.test.tsx"]),
    deps = [
        ":hello_app_ts",
        "@npm//jest",
        "@npm//@types/jest",
        "@npm//ts-jest",
        "@npm//@testing-library/react",
    ],
    config = "jest.config.js",
    data = [
        "//ui/hello_app:package.json",
        "//ui/hello_app:tsconfig.json",
    ],
)

# Accessibility tests
jest_test(
    name = "hello_app_accessibility_tests",
    srcs = glob(["tests/accessibility/**/*.test.ts", "tests/accessibility/**/*.test.tsx"]),
    deps = [
        ":hello_app_ts",
        "@npm//jest",
        "@npm//@types/jest",
        "@npm//ts-jest",
        "@npm//@testing-library/react",
        "@npm//jest-axe",
    ],
    config = "jest.config.js",
    data = [
        "//ui/hello_app:package.json",
        "//ui/hello_app:tsconfig.json",
    ],
)

# Build verification tests
filegroup(
    name = "hello_app_build_tests",
    srcs = [
        ":hello_app_bundle",
        "//ui/hello_app:index.html",
    ],
    data = [
        ":hello_app_bundle",
        "//ui/hello_app:index.html",
    ],
)

# Docker image for testing
container_image(
    name = "hello_app_image",
    base = "@distroless//cc:nonroot",
    files = [":hello_app"],
    cmd = ["/hello_app"],
    ports = ["3000"],
    labels = {
        "org.opencontainers.image.title": "Hello Polymera App",
        "org.opencontainers.image.description": "Minimal TypeScript React app for Polymera OS UI",
        "org.opencontainers.image.version": "0.1.0",
        "org.opencontainers.image.source": "https://github.com/polymera-os/polymera-os",
    },
)

# Kubernetes deployment
k8s_object(
    name = "hello_app_deployment",
    template = "k8s/deployment.yaml",
    images = [":hello_app_image"],
)

# Kubernetes service
k8s_object(
    name = "hello_app_service",
    template = "k8s/service.yaml",
)

# Kubernetes ingress
k8s_object(
    name = "hello_app_ingress",
    template = "k8s/ingress.yaml",
)

# All tests target
filegroup(
    name = "all_tests",
    srcs = [
        ":hello_app_unit_tests",
        ":hello_app_integration_tests",
        ":hello_app_e2e_tests",
        ":hello_app_performance_tests",
        ":hello_app_accessibility_tests",
        ":hello_app_build_tests",
    ],
)

# All binaries target
filegroup(
    name = "all_binaries",
    srcs = [
        ":hello_app",
        ":hello_app_dev",
        ":hello_app_bundle",
    ],
)

# All targets
filegroup(
    name = "all",
    srcs = [
        ":hello_app_ts",
        ":hello_app",
        ":hello_app_dev",
        ":hello_app_bundle",
        ":hello_app_image",
        ":hello_app_deployment",
        ":hello_app_service",
        ":hello_app_ingress",
        ":all_tests",
    ],
)
