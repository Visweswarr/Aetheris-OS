# Polymera OS Development Setup

This document provides comprehensive instructions for setting up the Polymera OS development environment using Nix.

## 🚀 Quick Start

```bash
# Clone the repository
git clone https://github.com/your-org/polymera-os.git
cd polymera-os

# Enter the development environment
nix develop

# Verify toolchain installation
rustc --version
cargo --version
go version
node --version
python3.11 --version
```

## 📋 Prerequisites

- **Nix**: Version 2.18+ with flakes enabled
- **Git**: For version control
- **System**: Linux, macOS, or Windows with WSL2

### Installing Nix

```bash
# Linux/macOS
sh <(curl -L https://nixos.org/nix/install) --daemon

# Windows (WSL2)
sh <(curl -L https://nixos.org/nix/install) --daemon

# Enable flakes
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
```

## 🛠️ Toolchain Versions

### Core Development Tools

| Tool | Version | Purpose |
|------|---------|---------|
| **Rust Stable** | 1.75.0 | Primary Rust toolchain |
| **Rust Nightly** | 2024-01-15 | Nightly features and testing |
| **LLVM** | 17.x | C++17 compiler and toolchain |
| **Bazel** | Latest | Build system |
| **CMake** | Latest | C++ build system |
| **Ninja** | Latest | Fast build system |

### Language Runtimes

| Language | Version | Purpose |
|----------|---------|---------|
| **Python** | 3.11 | Scripting and tooling |
| **Node.js** | 20.11.0 LTS | UI development |
| **Go** | 1.21 | System services |
| **Protocol Buffers** | Latest | Data serialization |

### Graphics and XR

| Tool | Version | Purpose |
|------|---------|---------|
| **Vulkan SDK** | Latest | Graphics API |
| **OpenXR SDK** | Latest | XR development |
| **Mesa** | Latest | OpenGL implementation |

### WebAssembly

| Tool | Version | Purpose |
|------|---------|---------|
| **Wasmtime** | Latest | WebAssembly runtime |
| **WasmEdge** | Latest | Edge WebAssembly runtime |
| **WASI SDK** | Latest | WebAssembly System Interface |

### Blockchain and ZK

| Tool | Version | Purpose |
|------|---------|---------|
| **Foundry** | Latest | Ethereum development |
| **Noir** | Latest | Zero-knowledge proofs |
| **liboqs** | Latest | Post-quantum cryptography |

### Security Tools

| Tool | Version | Purpose |
|------|---------|---------|
| **Sigstore** | Latest | Software signing |
| **Grype** | Latest | Vulnerability scanning |
| **CycloneDX** | Latest | SBOM generation |

## 🔧 Environment Configuration

### Environment Variables

The development shell automatically sets these environment variables:

```bash
# Rust configuration
export RUST_BACKTRACE=1
export RUST_LOG=debug
export CARGO_INCREMENTAL=1
export RUSTFLAGS="-C target-cpu=native -C target-feature=+crt-static"

# LLVM configuration
export CC="clang"
export CXX="clang++"
export LD="ld.lld"
export CXXFLAGS="-std=c++17 -stdlib=libc++"
export LDFLAGS="-fuse-ld=lld"

# Vulkan configuration
export VULKAN_SDK="/nix/store/.../vulkan-headers"
export VK_LAYER_PATH="/nix/store/.../vulkan-validation-layers/share/vulkan/explicit_layer.d"

# OpenXR configuration
export OPENXR_LOADER_PATH="/nix/store/.../openxr-loader"

# Python configuration
export PYTHONPATH="/nix/store/.../python3.11/bin/python3.11"
export PYTHON_VERSION="3.11"

# Node.js configuration
export NODE_PATH="/nix/store/.../nodejs-20/lib/node_modules"
export NODE_VERSION="20.11.0"

# Go configuration
export GOPATH="$HOME/go"
export GOROOT="/nix/store/.../go-1.21"
export PATH="$GOPATH/bin:$GOROOT/bin:$PATH"

# Foundry configuration
export FOUNDRY_HOME="/nix/store/.../foundry"
export PATH="/nix/store/.../foundry/bin:$PATH"

# Noir configuration
export NOIR_HOME="/nix/store/.../noir"
export PATH="/nix/store/.../noir/bin:$PATH"

# liboqs configuration
export LIBOQS_HOME="/nix/store/.../liboqs"
export PKG_CONFIG_PATH="/nix/store/.../liboqs/lib/pkgconfig:$PKG_CONFIG_PATH"
```

### Build Configuration

```bash
# Cargo configuration
export CARGO_TARGET_DIR="./target"
export RUST_TARGET_PATH="./rust-target"

# Bazel configuration
export BAZEL_USE_CPP_ONLY_TOOLCHAIN=1
export BAZEL_BUILD_OPTS="--enable_platform_specific_config"

# Security configuration
export RUSTSEC_DB_URL="https://github.com/rustsec/advisory-db.git"
export GRYPE_DB_URL="https://github.com/anchore/grype-db.git"
```

## 🚀 Development Workflow

### 1. Enter Development Environment

```bash
nix develop
```

This will:
- Load all toolchains and dependencies
- Set up environment variables
- Display available commands
- Show toolchain versions

### 2. Build and Test

```bash
# Build all targets with Bazel
bazel build //...

# Run all tests
bazel test //...

# Build Rust components
cargo build

# Run Rust tests
cargo test

# Run nightly Rust tests
cargo +nightly test
```

### 3. Development Tools

```bash
# Code formatting
cargo fmt
rustfmt src/**/*.rs

# Linting
cargo clippy
cargo clippy -- -D warnings

# Security audit
cargo audit

# Code coverage
cargo tarpaulin

# Fuzz testing
cargo fuzz run

# Go development
go build ./...
go test ./...
go mod tidy

# Foundry development
forge build
forge test
forge script

# Noir development
nargo compile
nargo test
nargo prove
```

### 4. Performance and Security

```bash
# Performance profiling
cargo bench
cargo criterion

# Security scanning
grype .
cyclonedx generate

# Dependency analysis
cargo tree
cargo outdated
```

## 🔍 Troubleshooting

### Common Issues

#### 1. Nix Flake Not Found

```bash
# Ensure flakes are enabled
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf

# Restart Nix daemon
sudo systemctl restart nix-daemon
```

#### 2. Toolchain Not Found

```bash
# Check if tool is available
which rustc
which go
which node

# Verify environment variables
echo $PATH
echo $GOROOT
echo $NODE_PATH
```

#### 3. Build Failures

```bash
# Clean build artifacts
cargo clean
bazel clean --expunge

# Update dependencies
cargo update
go mod download

# Check toolchain versions
rustc --version
go version
clang --version
```

#### 4. Permission Issues

```bash
# Fix Nix store permissions
sudo chown -R $USER:$USER /nix/store
sudo chmod -R 755 /nix/store

# Check file ownership
ls -la ~/.nix-profile
```

### Performance Optimization

#### 1. Enable Cachix

```bash
# Install cachix
nix-env -iA nixpkgs.cachix

# Add binary cache
cachix use cachix

# Or manually configure in ~/.config/nix/nix.conf
extra-substituters = https://cache.nixos.org https://cachix.cachix.org
extra-trusted-public-keys = cache.nixos.org-1:6NCHdD59X431o0gWypbMrA/RkbJxLv6j+7qkXqQFz0Y= cachix.cachix.org-1:WnPlPlSOyBUU7/52CejD2Q7Y8Jz1FwG1X6v1txdQ8rs=
```

#### 2. Parallel Builds

```bash
# Set number of parallel jobs
export NIX_BUILD_CORES=8
export CARGO_BUILD_JOBS=8
export MAKEFLAGS="-j8"
```

#### 3. Incremental Builds

```bash
# Enable incremental compilation
export CARGO_INCREMENTAL=1
export RUSTFLAGS="-C incremental=target/incremental"
```

## 📚 Additional Resources

### Documentation

- [Nix User Guide](https://nixos.org/guides/nix-pills/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Go Documentation](https://golang.org/doc/)
- [Vulkan Guide](https://vulkan.lunarg.com/doc/sdk/1.3.216.0/windows/getting_started.html)
- [OpenXR Specification](https://www.khronos.org/openxr/)

### Community

- [NixOS Discourse](https://discourse.nixos.org/)
- [Rust Community](https://www.rust-lang.org/community)
- [Go Community](https://golang.org/community/)
- [Vulkan Community](https://vulkan.lunarg.com/community/)

### Support

For issues specific to Polymera OS:

1. Check this documentation
2. Search existing issues
3. Create a new issue with:
   - System information
   - Error messages
   - Steps to reproduce
   - Expected vs actual behavior

## 🔄 Updates and Maintenance

### Updating Toolchains

```bash
# Update flake inputs
nix flake update

# Update specific input
nix flake lock --update-input rust-overlay

# Rebuild development environment
nix develop --rebuild
```

### Version Pinning

Toolchain versions are pinned in `flake.nix` for reproducibility:

```nix
# Rust toolchains with specific versions
rustStable = pkgs.rust-bin.stable."1.75.0".default.override {
  targets = [ "x86_64-unknown-linux-gnu" "wasm32-wasi" "aarch64-unknown-linux-gnu" ];
};

rustNightly = pkgs.rust-bin.nightly."2024-01-15".default.override {
  targets = [ "x86_64-unknown-linux-gnu" "wasm32-wasi" "aarch64-unknown-linux-gnu" ];
};
```

### Security Updates

```bash
# Check for security vulnerabilities
cargo audit
grype .
nix audit

# Update vulnerable packages
nix flake update
nix develop --rebuild
```

---

**Happy coding! 🎉**

For questions or issues, please refer to the troubleshooting section or create an issue in the repository.

