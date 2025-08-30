#!/bin/bash
set -e

echo "🚀 Setting up Polymera OS Development Environment..."

# Install additional system packages
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    cmake \
    ninja-build \
    pkg-config \
    libssl-dev \
    libclang-dev \
    clang \
    lld \
    libvulkan-dev \
    vulkan-tools \
    mesa-vulkan-drivers \
    libx11-dev \
    libxrandr-dev \
    libxcb1-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libasound2-dev \
    libpulse-dev \
    libpcap-dev \
    libnetfilter-conntrack-dev \
    qemu-system-x86 \
    graphviz \
    curl \
    wget \
    unzip \
    git \
    vim \
    htop \
    ripgrep \
    fd-find \
    bat \
    exa

# Install Bazel
curl -fsSL https://bazel.build/bazel-release.pub.gpg | sudo gpg --dearmor > bazel.gpg
sudo mv bazel.gpg /etc/apt/trusted.gpg.d/
echo "deb [arch=amd64] https://storage.googleapis.com/bazel-apt stable jdk1.8" | sudo tee /etc/apt/sources.list.d/bazel.list
sudo apt-get update
sudo apt-get install -y bazel

# Install Nix
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
. /home/vscode/.nix-profile/etc/profile.d/nix.sh

# Install Rust nightly
rustup toolchain install nightly
rustup target add wasm32-wasi --toolchain nightly
rustup target add aarch64-unknown-linux-gnu --toolchain nightly

# Install additional Rust tools
cargo install cargo-audit
cargo install cargo-tarpaulin
cargo install cargo-watch
cargo install cargo-fuzz
cargo install wasm-pack

# Install Python packages
pip install --user \
    black \
    isort \
    flake8 \
    mypy \
    pytest \
    pytest-cov \
    pytest-asyncio \
    pre-commit

# Install Node.js packages
npm install -g \
    typescript \
    ts-node \
    nodemon \
    yarn \
    pnpm \
    @types/node \
    prettier \
    eslint \
    jest

# Setup environment variables
echo 'export RUST_BACKTRACE=1' >> ~/.bashrc
echo 'export RUST_LOG=debug' >> ~/.bashrc
echo 'export CARGO_INCREMENTAL=1' >> ~/.bashrc
echo 'export RUSTFLAGS="-C target-cpu=native -C target-feature=+crt-static"' >> ~/.bashrc
echo 'export BAZEL_USE_CPP_ONLY_TOOLCHAIN=1' >> ~/.bashrc
echo 'export BAZEL_BUILD_OPTS="--enable_platform_specific_config"' >> ~/.bashrc
echo 'export LIBCLANG_PATH="/usr/lib/llvm-16/lib"' >> ~/.bashrc
echo 'export CC="/usr/bin/clang-16"' >> ~/.bashrc
echo 'export CXX="/usr/bin/clang++-16"' >> ~/.bashrc
echo 'export VULKAN_SDK="/usr"' >> ~/.bashrc
echo 'export VK_LAYER_PATH="/usr/share/vulkan/explicit_layer.d"' >> ~/.bashrc

# Create cache directory
mkdir -p ~/.cache/polymera-os

echo "✅ Polymera OS Development Environment setup complete!"
echo ""
echo "Available tools:"
echo "  Rust (stable + nightly): $(rustc --version)"
echo "  Bazel: $(bazel --version)"
echo "  Nix: $(nix --version)"
echo "  Python: $(python3 --version)"
echo "  Node.js: $(node --version)"
echo "  LLVM: $(clang-16 --version | head -n1)"
echo "  Vulkan: $(vulkaninfo --version 2>/dev/null || echo 'Available')"
echo ""
echo "Next steps:"
echo "  1. Run 'nix develop' to enter the Nix development environment"
echo "  2. Run 'bazel build //...' to build all targets"
echo "  3. Run 'cargo test' to run Rust tests"
echo "  4. Run 'cargo fuzz run' to run fuzz tests"
