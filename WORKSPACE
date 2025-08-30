workspace(name = "polymera_os")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:git.bzl", "git_repository")

# =============================================================================
# Core Bazel Rules
# =============================================================================

# Rust toolchain and rules
http_archive(
    name = "rules_rust",
    sha256 = "9d9e82c47926b71980c8c5656d88096128f084e3c732c861b5bcc52094e0e519",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/download/v0.44.0/rules_rust-v0.44.0.tar.gz"],
)

# C++ rules
http_archive(
    name = "rules_cc",
    sha256 = "3d9e271e2876baa6d7c9c7878bb5de71ca3362d0f2b1a7bda0b5591d8c7e1b7d",
    strip_prefix = "rules_cc-0.0.9",
    urls = ["https://github.com/bazelbuild/rules_cc/archive/0.0.9.tar.gz"],
)

# Go rules
http_archive(
    name = "io_bazel_rules_go",
    sha256 = "6dc2da7ab4cf5d7bfc7c949776b1b7c733f05e56edc4bcd9022bb249d2e2a996",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/rules_go/releases/download/v0.39.1/rules_go-v0.39.1.zip",
        "https://github.com/bazelbuild/rules_go/releases/download/v0.39.1/rules_go-v0.39.1.zip",
    ],
)

# Node.js rules
http_archive(
    name = "build_bazel_rules_nodejs",
    sha256 = "c07780a7ecf3868c3b5554db363a8e5fbb6f76f9d11069b87bacd9c12fae56e6",
    urls = ["https://github.com/bazelbuild/rules_nodejs/releases/download/6.0.2/rules_nodejs-6.0.2.tar.gz"],
)

# Python rules
http_archive(
    name = "rules_python",
    sha256 = "94750828b18044530e221a8c8c3c2b5b8c5b8c5b8c5b8c5b8c5b8c5b8c5b8c5",
    strip_prefix = "rules_python-0.21.0",
    urls = ["https://github.com/bazelbuild/rules_python/archive/0.21.0.tar.gz"],
)

# Protocol Buffers rules
http_archive(
    name = "rules_proto",
    sha256 = "dc3fb206a2cb94cbaa5c5f21c9ce23eaa2ae34d0722b4073fba7812e38134dfd",
    strip_prefix = "rules_proto-5.3.0-21.7",
    urls = [
        "https://github.com/bazelbuild/rules_proto/archive/refs/tags/5.3.0-21.7.tar.gz",
    ],
)

# gRPC rules
http_archive(
    name = "rules_proto_grpc",
    sha256 = "b8cc21775ec2f3eed4c2ca59fff4c9d2e8f81c92a47329f5b3c86c4e58c3e4c0",
    strip_prefix = "rules_proto_grpc-4.2.0",
    urls = [
        "https://github.com/rules-proto-grpc/rules_proto_grpc/archive/4.2.0.tar.gz",
    ],
)

# OCI (Docker) rules
http_archive(
    name = "io_bazel_rules_docker",
    sha256 = "b1e80761a71a824fa39efe5f44f742dc4a334c9a3db120fcc1b8d18d6402b54e",
    strip_prefix = "rules_docker-0.25.0",
    urls = ["https://github.com/bazelbuild/rules_docker/archive/v0.25.0.tar.gz"],
)

# =============================================================================
# Additional Dependencies
# =============================================================================

# WASI SDK
http_archive(
    name = "wasi_sdk",
    sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
    urls = ["https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-20/wasi-sdk-20.0-linux.tar.gz"],
)

# QEMU for testing
http_archive(
    name = "qemu",
    sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
    urls = ["https://download.qemu.org/qemu-8.0.0.tar.xz"],
)

# Nix integration
http_archive(
    name = "io_tweag_rules_nixpkgs",
    sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
    strip_prefix = "rules_nixpkgs-0.10.0",
    urls = ["https://github.com/tweag/rules_nixpkgs/archive/v0.10.0.tar.gz"],
)

# Sigstore for cryptographic verification
http_archive(
    name = "sigstore",
    sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
    urls = ["https://github.com/sigstore/sigstore/releases/download/v1.0.0/sigstore-v1.0.0.tar.gz"],
)

# SBOM generation tools
http_archive(
    name = "cyclonedx",
    sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
    urls = ["https://github.com/CycloneDX/cyclonedx-cli/releases/download/v0.1.0/cyclonedx-cli-v0.1.0.tar.gz"],
)

# Security scanning tools
http_archive(
    name = "grype",
    sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
    urls = ["https://github.com/anchore/grype/releases/download/v0.65.0/grype_0.65.0_linux_amd64.tar.gz"],
)

# Performance testing tools
http_archive(
    name = "criterion",
    sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
    urls = ["https://github.com/bheisler/criterion.rs/releases/download/v0.5.1/criterion-v0.5.1.tar.gz"],
)

# =============================================================================
# Load and Initialize Rules
# =============================================================================

# Load Rust rules
load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")
load("@rules_rust//crate_universe:repositories.bzl", "crate_universe_dependencies")

# Load C++ rules
load("@rules_cc//cc:repositories.bzl", "rules_cc_dependencies")

# Load Go rules
load("@io_bazel_rules_go//go:deps.bzl", "go_register_toolchains", "go_rules_dependencies")

# Load Node.js rules
load("@build_bazel_rules_nodejs//:index.bzl", "node_repositories", "yarn_install")

# Load Python rules
load("@rules_python//python:repositories.bzl", "python_register_toolchains")

# Load Protocol Buffer rules
load("@rules_proto//proto:repositories.bzl", "rules_proto_dependencies")

# Load gRPC rules
load("@rules_proto_grpc//repositories:repositories.bzl", "rules_proto_grpc_repos")

# Load OCI rules
load("@io_bazel_rules_docker//repositories:repositories.bzl", "repositories")

# Load Nix rules
load("@io_tweag_rules_nixpkgs//nixpkgs:nixpkgs.bzl", "nixpkgs_git_repository", "nixpkgs_package")

# =============================================================================
# Initialize All Rules
# =============================================================================

# Initialize Rust
rules_rust_dependencies()
crate_universe_dependencies()
rust_register_toolchains(
    edition = "2021",
    versions = ["1.75.0"],
)

# Initialize C++
rules_cc_dependencies()

# Initialize Go
go_rules_dependencies()
go_register_toolchains(version = "1.21.0")

# Initialize Node.js
node_repositories(
    node_version = "20.11.0",
    yarn_version = "1.22.19",
)

# Initialize Python
python_register_toolchains(
    name = "python3_11",
    python_version = "3.11",
)

# Initialize Protocol Buffers
rules_proto_dependencies()
rules_proto_grpc_repos()

# Initialize OCI
repositories()

# =============================================================================
# Protocol Buffer Code Generation Configuration
# =============================================================================

# Load gRPC and protobuf generation rules
load("@rules_proto_grpc//rust:repositories.bzl", rules_proto_grpc_rust_repos)
load("@rules_proto_grpc//go:repositories.bzl", rules_proto_grpc_go_repos)
load("@rules_proto_grpc//js:repositories.bzl", rules_proto_grpc_js_repos)

# Initialize language-specific repositories
rules_proto_grpc_rust_repos()
rules_proto_grpc_go_repos()
rules_proto_grpc_js_repos()

# Load the actual generation rules
load("@rules_proto_grpc//rust:rust_grpc.bzl", "rust_grpc_library")
load("@rules_proto_grpc//go:go_grpc.bzl", "go_grpc_library")
load("@rules_proto_grpc//js:js_grpc.bzl", "js_grpc_library")
load("@rules_proto_grpc//rust:rust_proto.bzl", "rust_proto_library")
load("@rules_proto_grpc//go:go_proto.bzl", "go_proto_library")
load("@rules_proto_grpc//js:js_proto.bzl", "js_proto_library")

# =============================================================================
# External Dependencies
# =============================================================================

# Google APIs (for protobuf well-known types)
http_archive(
    name = "googleapis",
    sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
    strip_prefix = "googleapis-20231215",
    urls = ["https://github.com/googleapis/googleapis/archive/20231215.tar.gz"],
)

# gRPC Java
http_archive(
    name = "io_grpc_grpc_java",
    sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
    strip_prefix = "grpc-java-1.59.0",
    urls = ["https://github.com/grpc/grpc-java/archive/v1.59.0.tar.gz"],
)

# =============================================================================
# Custom Repository Rules
# =============================================================================

# Load custom repository rules for Polymera OS
load("//tooling/bazel:repositories.bzl", "polymera_repositories")

# Initialize Polymera OS specific repositories
polymera_repositories()
