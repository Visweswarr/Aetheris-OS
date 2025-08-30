# Polymera OS Bazel Tooling

This directory contains the Bazel build system configuration and tooling for Polymera OS, including all necessary rules for multi-language development, protocol buffer code generation, and containerization.

## 🚀 Quick Start

```bash
# Build all proto-generated code
bazel build //proto:all

# Build specific language bindings
bazel build //proto:rust_all
bazel build //proto:go_all
bazel build //proto:ts_all

# Run all tests
bazel test //...

# Build and run specific components
bazel run //kernel:kernel_bin
bazel run //services:service_bin
```

## 📋 Available Bazel Rules

### Core Rules

| Rule | Purpose | Version |
|------|---------|---------|
| **rules_rust** | Rust toolchain and build rules | 0.44.0 |
| **rules_cc** | C++ build rules | 0.0.9 |
| **rules_go** | Go build rules | 0.39.1 |
| **rules_node** | Node.js/TypeScript rules | 6.0.2 |
| **rules_python** | Python build rules | 0.21.0 |
| **rules_proto** | Protocol Buffer rules | 5.3.0-21.7 |
| **rules_oci** | Docker/OCI container rules | 0.25.0 |

### Language-Specific Rules

| Language | Rules | Features |
|----------|-------|----------|
| **Rust** | `rust_library`, `rust_binary`, `rust_test` | Cargo integration, edition 2021 |
| **Go** | `go_library`, `go_binary`, `go_test` | Go modules, version 1.21 |
| **TypeScript** | `ts_project`, `js_library` | Node.js 20.11.0, Yarn |
| **Python** | `py_library`, `py_binary`, `py_test` | Python 3.11 |
| **C++** | `cc_library`, `cc_binary`, `cc_test` | LLVM 17, C++17 |

## 🏗️ Protocol Buffer Code Generation

### Multi-Language Support

The Bazel configuration automatically generates code from `.proto` files to:

- **Rust**: Using `prost` and `tonic`
- **Go**: Using `protobuf` and `grpc-go`
- **TypeScript**: Using `protobufjs` and `grpc-web`

### Example BUILD File

```python
# kernel/proto/BUILD
load("@rules_proto_grpc//rust:rust_grpc.bzl", "rust_grpc_library")
load("@rules_proto_grpc//go:go_grpc.bzl", "go_grpc_library")
load("@rules_proto_grpc//js:js_grpc.bzl", "js_grpc_library")

proto_library(
    name = "kernel_proto",
    srcs = ["kernel.proto"],
)

# Rust code generation
rust_grpc_library(
    name = "kernel_rust_grpc",
    deps = [":kernel_proto"],
)

# Go code generation
go_grpc_library(
    name = "kernel_go_grpc",
    proto = ":kernel_proto",
)

# TypeScript code generation
js_grpc_library(
    name = "kernel_ts_grpc",
    deps = [":kernel_proto"],
)
```

### Main Targets

- `//proto:all` - Generates all language bindings
- `//proto:rust_all` - Rust proto code only
- `//proto:go_all` - Go proto code only
- `//proto:ts_all` - TypeScript proto code only

## 🔧 Configuration

### WORKSPACE Setup

The `WORKSPACE` file includes:

1. **Rule Dependencies**: All necessary Bazel rules
2. **Toolchain Registration**: Rust, Go, Node.js, Python
3. **Protocol Buffer Setup**: gRPC and code generation
4. **Custom Repositories**: Polymera OS specific dependencies

### Key Configuration Sections

```python
# Rust toolchain
rust_register_toolchains(
    edition = "2021",
    versions = ["1.75.0"],
)

# Go toolchain
go_register_toolchains(version = "1.21.0")

# Node.js toolchain
node_repositories(
    node_version = "20.11.0",
    yarn_version = "1.22.19",
)

# Python toolchain
python_register_toolchains(
    name = "python3_11",
    python_version = "3.11",
)
```

## 📁 Project Structure

```
polymera-os/
├── WORKSPACE                    # Main Bazel workspace
├── proto/                       # Protocol buffer definitions
│   ├── BUILD                   # Proto aggregation targets
│   └── kernel/                 # Kernel-specific protos
│       ├── BUILD               # Kernel proto targets
│       └── kernel.proto        # Kernel service definitions
├── kernel/                      # Kernel implementation
│   ├── BUILD                   # Kernel build targets
│   └── src/                    # Kernel source code
├── services/                    # System services
│   ├── BUILD                   # Service build targets
│   └── src/                    # Service source code
├── tooling/                     # Development tools
│   └── bazel/                  # Bazel configuration
│       ├── README.md           # This file
│       └── repositories.bzl    # Custom repository rules
└── docs/                        # Documentation
```

## 🎯 Build Targets

### Core Targets

```bash
# Build entire project
bazel build //...

# Build specific components
bazel build //kernel:all
bazel build //services:all
bazel build //proto:all

# Build and run binaries
bazel run //kernel:kernel_bin
bazel run //services:service_bin
```

### Testing Targets

```bash
# Run all tests
bazel test //...

# Run specific test suites
bazel test //kernel/...
bazel test //services/...
bazel test //proto/...

# Run tests with coverage
bazel coverage //...
```

### Development Targets

```bash
# Generate proto code
bazel build //proto:generate_all

# Generate documentation
bazel build //proto:proto_docs

# Format code
bazel run //:format

# Lint code
bazel run //:lint
```

## 🐳 Containerization

### OCI Rules Integration

The Bazel configuration includes `rules_docker` for containerization:

```python
# Example container target
container_image(
    name = "polymera_kernel",
    base = "@ubuntu//image",
    files = ["//kernel:kernel_bin"],
    cmd = ["/app/kernel_bin"],
)
```

### Container Build Commands

```bash
# Build container image
bazel build //:polymera_kernel

# Load into Docker
bazel run //:polymera_kernel

# Push to registry
bazel run //:polymera_kernel.push
```

## 🔍 Debugging and Troubleshooting

### Common Issues

#### 1. Toolchain Not Found

```bash
# Check available toolchains
bazel query --output=location @rust_linux_x86_64//:rustc
bazel query --output=location @go_sdk//:go

# Verify toolchain registration
bazel info --show_make_env
```

#### 2. Proto Generation Fails

```bash
# Check proto dependencies
bazel query --output=location //proto:kernel_proto

# Verify proto toolchain
bazel query --output=location @rules_proto_grpc//rust:rust_grpc
```

#### 3. Build Cache Issues

```bash
# Clean build cache
bazel clean --expunge

# Rebuild from scratch
bazel build --noincremental //...
```

### Debug Commands

```bash
# Show build graph
bazel query --output=graph //proto:all

# Show dependencies
bazel query --output=location --deps //proto:all

# Show build actions
bazel aquery //proto:all
```

## 📚 Advanced Usage

### Custom Build Rules

Create custom build rules in `tooling/bazel/rules/`:

```python
# tooling/bazel/rules/quantum.bzl
def quantum_library(name, srcs, deps = []):
    native.filegroup(
        name = name,
        srcs = srcs,
        visibility = ["//visibility:public"],
    )
```

### Workspace Extensions

Extend the workspace with custom repositories:

```python
# WORKSPACE
load("//tooling/bazel:repositories.bzl", "polymera_repositories")
polymera_repositories()
```

### Platform-Specific Builds

```bash
# Build for specific platform
bazel build --platforms=@io_bazel_rules_go//go/toolchain:linux_amd64 //...

# Cross-compilation
bazel build --platforms=@io_bazel_rules_go//go/toolchain:linux_arm64 //...
```

## 🚀 Performance Optimization

### Build Optimization

```bash
# Enable build cache
bazel build --disk_cache=/path/to/cache //...

# Parallel builds
bazel build --jobs=8 //...

# Incremental builds
bazel build --incremental //...
```

### Remote Execution

```bash
# Enable remote execution
bazel build --remote_executor=grpc://remote-host:8080 //...

# Remote caching
bazel build --remote_cache=grpc://cache-host:8080 //...
```

## 🔐 Security Features

### Sigstore Integration

```bash
# Verify signatures
bazel run @sigstore//:verify //...

# Sign artifacts
bazel run @sigstore//:sign //...
```

### Vulnerability Scanning

```bash
# Run Grype scan
bazel run @grype//:scan //...

# Generate SBOM
bazel run @cyclonedx//:generate //...
```

## 📖 Additional Resources

### Documentation

- [Bazel Official Docs](https://bazel.build/docs)
- [Rules Rust](https://github.com/bazelbuild/rules_rust)
- [Rules Go](https://github.com/bazelbuild/rules_go)
- [Rules Node.js](https://github.com/bazelbuild/rules_nodejs)
- [Rules Python](https://github.com/bazelbuild/rules_python)
- [Rules Proto](https://github.com/bazelbuild/rules_proto)
- [Rules Docker](https://github.com/bazelbuild/rules_docker)

### Community

- [Bazel Slack](https://bazelbuild.slack.com/)
- [Bazel GitHub Discussions](https://github.com/bazelbuild/bazel/discussions)
- [Polymera OS Community](https://github.com/polymera-os)

## 🤝 Contributing

### Adding New Rules

1. Add rule dependency to `WORKSPACE`
2. Load and initialize in appropriate section
3. Update this documentation
4. Add example BUILD targets

### Testing Changes

```bash
# Test specific rules
bazel test //tooling/bazel/...

# Test proto generation
bazel test //proto/...

# Test all tooling
bazel test //tooling/...
```

---

**Happy building! 🎉**

For questions or issues, please refer to the troubleshooting section or create an issue in the repository.
