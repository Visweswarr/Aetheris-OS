load("@build_bazel_rules_nodejs//:index.bzl", "nodejs_binary", "npm_package")
load("@npm//@bazel/typescript:index.bzl", "ts_project")
load("@npm//@bazel/jest:index.bzl", "jest_test")

package(default_visibility = ["//visibility:public"])

# =============================================================================
# TypeScript Libraries
# =============================================================================

# Core UI library
ts_project(
    name = "ui_core",
    srcs = glob([
        "src/core/**/*.ts",
        "src/core/**/*.tsx",
    ]),
    declaration = True,
    deps = [
        "//kernel/proto:kernel_ts",
        "@npm//@types/node",
        "@npm//@types/react",
        "@npm//@types/react-dom",
        "@npm//react",
        "@npm//react-dom",
        "@npm//typescript",
    ],
)

# Component library
ts_project(
    name = "ui_components",
    srcs = glob([
        "src/components/**/*.ts",
        "src/components/**/*.tsx",
    ]),
    declaration = True,
    deps = [
        ":ui_core",
        "@npm//@types/node",
        "@npm//@types/react",
        "@npm//@types/react-dom",
        "@npm//react",
        "@npm//react-dom",
        "@npm//typescript",
        "@npm//styled-components",
        "@npm//@types/styled-components",
    ],
)

# Service integration library
ts_project(
    name = "ui_services",
    srcs = glob([
        "src/services/**/*.ts",
    ]),
    declaration = True,
    deps = [
        ":ui_core",
        "//kernel/proto:kernel_ts",
        "@npm//@types/node",
        "@npm//grpc-web",
        "@npm//google-protobuf",
        "@npm//axios",
    ],
)

# State management library
ts_project(
    name = "ui_state",
    srcs = glob([
        "src/state/**/*.ts",
    ]),
    declaration = True,
    deps = [
        ":ui_core",
        "@npm//@types/node",
        "@npm//zustand",
        "@npm//immer",
    ],
)

# =============================================================================
# TypeScript Applications
# =============================================================================

# Main web application
ts_project(
    name = "web_app",
    srcs = glob([
        "src/app/**/*.ts",
        "src/app/**/*.tsx",
    ]),
    declaration = True,
    deps = [
        ":ui_core",
        ":ui_components",
        ":ui_services",
        ":ui_state",
        "@npm//@types/node",
        "@npm//@types/react",
        "@npm//@types/react-dom",
        "@npm//react",
        "@npm//react-dom",
        "@npm//react-router-dom",
        "@npm//@types/react-router-dom",
        "@npm//typescript",
    ],
)

# Admin dashboard
ts_project(
    name = "admin_dashboard",
    srcs = glob([
        "src/admin/**/*.ts",
        "src/admin/**/*.tsx",
    ]),
    declaration = True,
    deps = [
        ":ui_core",
        ":ui_components",
        ":ui_services",
        ":ui_state",
        "@npm//@types/node",
        "@npm//@types/react",
        "@npm//@types/react-dom",
        "@npm//react",
        "@npm//react-dom",
        "@npm//react-router-dom",
        "@npm//@types/react-router-dom",
        "@npm//typescript",
        "@npm//@mui/material",
        "@npm//@mui/icons-material",
        "@npm//@emotion/react",
        "@npm//@emotion/styled",
    ],
)

# =============================================================================
# Node.js Binaries
# =============================================================================

# Development server
nodejs_binary(
    name = "dev_server",
    data = [
        ":web_app",
        ":admin_dashboard",
        "@npm//vite",
        "@npm//@vitejs/plugin-react",
    ],
    entry_point = "src/dev-server.js",
)

# Build tool
nodejs_binary(
    name = "build_tool",
    data = [
        ":web_app",
        ":admin_dashboard",
        "@npm//vite",
        "@npm//@vitejs/plugin-react",
        "@npm//typescript",
    ],
    entry_point = "src/build.js",
)

# =============================================================================
# NPM Packages
# =============================================================================

# Core UI package
npm_package(
    name = "ui_core_package",
    package = "ui-core",
    srcs = [":ui_core"],
    deps = [":ui_core"],
)

# Component package
npm_package(
    name = "ui_components_package",
    package = "ui-components",
    srcs = [":ui_components"],
    deps = [":ui_components"],
)

# =============================================================================
# Tests
# =============================================================================

# Unit tests for core
jest_test(
    name = "ui_core_test",
    srcs = glob(["src/core/**/*.test.ts", "src/core/**/*.test.tsx"]),
    deps = [":ui_core"],
    config = "jest.config.js",
)

# Unit tests for components
jest_test(
    name = "ui_components_test",
    srcs = glob(["src/components/**/*.test.ts", "src/components/**/*.test.tsx"]),
    deps = [":ui_components"],
    config = "jest.config.js",
)

# Unit tests for services
jest_test(
    name = "ui_services_test",
    srcs = glob(["src/services/**/*.test.ts"]),
    deps = [":ui_services"],
    config = "jest.config.js",
)

# Unit tests for state
jest_test(
    name = "ui_state_test",
    srcs = glob(["src/state/**/*.test.ts"]),
    deps = [":ui_state"],
    config = "jest.config.js",
)

# Integration tests
jest_test(
    name = "ui_integration_test",
    srcs = glob(["src/**/*.integration.test.ts", "src/**/*.integration.test.tsx"]),
    deps = [
        ":ui_core",
        ":ui_components",
        ":ui_services",
        ":ui_state",
    ],
    config = "jest.config.js",
)

# E2E tests
jest_test(
    name = "ui_e2e_test",
    srcs = glob(["src/**/*.e2e.test.ts", "src/**/*.e2e.test.tsx"]),
    deps = [
        ":web_app",
        ":admin_dashboard",
    ],
    config = "jest.config.js",
)

# =============================================================================
# Development Tools
# =============================================================================

# TypeScript configuration
genrule(
    name = "tsconfig",
    srcs = ["tsconfig.json"],
    outs = ["tsconfig_generated.json"],
    cmd = """
        cp $(location tsconfig.json) $(location tsconfig_generated.json)
    """,
)

# ESLint configuration
genrule(
    name = "eslint_config",
    srcs = [".eslintrc.js"],
    outs = ["eslint_config_generated.js"],
    cmd = """
        cp $(location .eslintrc.js) $(location eslint_config_generated.js)
    """,
)

# Prettier configuration
genrule(
    name = "prettier_config",
    srcs = [".prettierrc"],
    outs = ["prettier_config_generated.json"],
    cmd = """
        cp $(location .prettierrc) $(location prettier_config_generated.json)
    """,
)

# =============================================================================
# Documentation
# =============================================================================

# Generate UI documentation
genrule(
    name = "ui_docs",
    srcs = [
        ":ui_core",
        ":ui_components",
        ":ui_services",
        ":ui_state",
        ":web_app",
        ":admin_dashboard",
    ],
    outs = ["ui_documentation.md"],
    cmd = """
        echo "# Polymera OS UI Documentation" > $(location ui_documentation.md)
        echo "" >> $(location ui_documentation.md)
        echo "Generated from TypeScript source code." >> $(location ui_documentation.md)
        echo "" >> $(location ui_documentation.md)
        echo "## Available Components" >> $(location ui_documentation.md)
        echo "" >> $(location ui_documentation.md)
        echo "- Core UI Library" >> $(location ui_documentation.md)
        echo "- UI Components" >> $(location ui_documentation.md)
        echo "- Service Integration" >> $(location ui_documentation.md)
        echo "- State Management" >> $(location ui_documentation.md)
        echo "- Web Application" >> $(location ui_documentation.md)
        echo "- Admin Dashboard" >> $(location ui_documentation.md)
    """,
)

# =============================================================================
# Aggregated Targets
# =============================================================================

# All TypeScript libraries
filegroup(
    name = "ts_libs",
    srcs = [
        ":ui_core",
        ":ui_components",
        ":ui_services",
        ":ui_state",
    ],
)

# All TypeScript applications
filegroup(
    name = "ts_apps",
    srcs = [
        ":web_app",
        ":admin_dashboard",
    ],
)

# All Node.js binaries
filegroup(
    name = "node_bins",
    srcs = [
        ":dev_server",
        ":build_tool",
    ],
)

# All NPM packages
filegroup(
    name = "npm_packages",
    srcs = [
        ":ui_core_package",
        ":ui_components_package",
    ],
)

# All tests
filegroup(
    name = "ui_tests",
    srcs = [
        ":ui_core_test",
        ":ui_components_test",
        ":ui_services_test",
        ":ui_state_test",
        ":ui_integration_test",
        ":ui_e2e_test",
    ],
)

# All UI artifacts
filegroup(
    name = "all",
    srcs = [
        ":ts_libs",
        ":ts_apps",
        ":node_bins",
        ":npm_packages",
        ":ui_tests",
        ":ui_docs",
    ],
)
