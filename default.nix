{ pkgs ? import <nixpkgs> {} }:

let
  # Rust toolchain with specific version
  rustToolchain = pkgs.rust-bin.stable."1.75.0".default.override {
    targets = [ "x86_64-unknown-linux-gnu" "wasm32-wasi" ];
  };

  # LLVM toolchain with C++17 support
  llvm = pkgs.llvmPackages_17;

  # Development dependencies
  devDependencies = with pkgs; [
    # Build tools
    bazel
    cmake
    ninja
    pkg-config

    # Rust tools
    rustToolchain
    cargo-audit
    cargo-tarpaulin
    cargo-watch

    # Protocol buffers
    protobuf
    grpc-tools

    # WASI tools
    wasmtime
    wasm-pack
    wasmedge

    # Security tools
    sigstore-cli
    cyclonedx-cli
    grype

    # Performance tools
    criterion

    # QEMU for testing
    qemu

    # Graphics and XR
    vulkan-loader
    vulkan-tools
    vulkan-headers
    vulkan-validation-layers
    mesa
    libx11
    libxrandr
    libxcb

    # OpenXR SDK
    openxr-loader
    openxr-headers

    # Documentation
    mdbook
    graphviz

    # Development utilities
    git
    vim
    htop
    ripgrep
    fd
    bat
    exa

    # Python 3.11 for tooling
    python311
    python311Packages.pip
    python311Packages.virtualenv
    python311Packages.setuptools

    # Node.js LTS for UI development
    nodejs_20
    yarn

    # Go toolchain
    go_1_21
    gopls
    delve

    # LLVM toolchain with C++17 support
    llvm.libclang
    llvm.clang
    llvm.lld
    llvm.libcxx
    llvm.libcxxabi

    # Foundry for Ethereum development
    foundry

    # Noir for zero-knowledge proofs
    noir

    # liboqs for post-quantum cryptography
    liboqs
  ];

  # Runtime dependencies
  runtimeDependencies = with pkgs; [
    # System libraries
    glibc
    libc
    openssl
    zlib

    # Graphics libraries
    vulkan-loader
    mesa
    libx11
    libxrandr
    libxcb

    # Audio libraries
    alsa-lib
    pulseaudio

    # Network libraries
    libpcap
    libnetfilter_conntrack
  ];

in pkgs.mkShell {
  name = "polymera-os-dev";

  buildInputs = devDependencies ++ runtimeDependencies;

  # Environment variables
  shellHook = ''
    export RUST_BACKTRACE=1
    export RUST_LOG=debug
    export CARGO_INCREMENTAL=1
    export RUSTFLAGS="-C target-cpu=native -C target-feature=+crt-static"

    # Bazel configuration
    export BAZEL_USE_CPP_ONLY_TOOLCHAIN=1
    export BAZEL_BUILD_OPTS="--enable_platform_specific_config"

    # WASI configuration
    export WASI_SDK_PATH="${pkgs.wasi-sdk}"
    export WASMTIME_HOME="${pkgs.wasmtime}"
    export WASMEDGE_HOME="${pkgs.wasmedge}"

    # Security configuration
    export SIGSTORE_CT_LOG_PUBLIC_KEY_FILE="${pkgs.sigstore-cli}/share/sigstore/ctfe.pub"

    # LLVM configuration with C++17
    export LIBCLANG_PATH="${llvm.libclang.lib}/lib"
    export CC="${llvm.clang}/bin/clang"
    export CXX="${llvm.clang}/bin/clang++"
    export LD="${llvm.lld}/bin/ld.lld"
    export CXXFLAGS="-std=c++17 -stdlib=libc++"
    export LDFLAGS="-fuse-ld=lld"

    # Vulkan configuration
    export VULKAN_SDK="${pkgs.vulkan-headers}"
    export VK_LAYER_PATH="${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d"

    # OpenXR configuration
    export OPENXR_LOADER_PATH="${pkgs.openxr-loader}"

    # Python 3.11 configuration
    export PYTHONPATH="${pkgs.python311}/bin/python3.11"
    export PYTHON_VERSION="3.11"

    # Node.js LTS configuration
    export NODE_PATH="${pkgs.nodejs_20}/lib/node_modules"
    export NODE_VERSION="20.11.0"

    # Go configuration
    export GOPATH="$HOME/go"
    export GOROOT="${pkgs.go_1_21}"
    export PATH="$GOPATH/bin:$GOROOT/bin:$PATH"

    # Foundry configuration
    export FOUNDRY_HOME="${pkgs.foundry}"
    export PATH="${pkgs.foundry}/bin:$PATH"

    # Noir configuration
    export NOIR_HOME="${pkgs.noir}"
    export PATH="${pkgs.noir}/bin:$PATH"

    # liboqs configuration
    export LIBOQS_HOME="${pkgs.liboqs}"
    export PKG_CONFIG_PATH="${pkgs.liboqs}/lib/pkgconfig:$PKG_CONFIG_PATH"

    echo "🚀 Polymera OS Development Environment"
    echo "📦 Rust: $(rustc --version)"
    echo "🔨 Bazel: $(bazel --version)"
    echo "🐧 Nix: $(nix --version)"
    echo "🔐 Security tools: sigstore, grype, cyclonedx"
    echo "🎮 Graphics: Vulkan SDK, OpenXR SDK, LLVM 17"
    echo "🐍 Python: $(python3.11 --version)"
    echo "🟢 Node.js: $(node --version)"
    echo "🔵 Go: $(go version)"
    echo "⚡ Foundry: $(forge --version 2>/dev/null || echo 'installed')"
    echo "🔒 Noir: $(nargo --version 2>/dev/null || echo 'installed')"
    echo "🔐 liboqs: $(pkg-config --modversion liboqs 2>/dev/null || echo 'installed')"
    echo ""
    echo "Available commands:"
    echo "  bazel build //...     - Build all targets"
    echo "  bazel test //...      - Run all tests"
    echo "  cargo build           - Build Rust components"
    echo "  cargo test            - Run Rust tests"
    echo "  cargo audit           - Security audit"
    echo "  cargo tarpaulin       - Code coverage"
    echo "  go build ./...        - Build Go components"
    echo "  go test ./...         - Test Go components"
    echo "  forge build           - Build Foundry contracts"
    echo "  forge test            - Test Foundry contracts"
    echo "  nargo compile          - Compile Noir programs"
    echo "  nargo test            - Test Noir programs"
    echo ""
    echo "Happy coding! 🎉"
  '';

  # Build configuration
  CARGO_TARGET_DIR = "./target";
  RUST_TARGET_PATH = "./rust-target";

  # Security configuration
  RUSTSEC_DB_URL = "https://github.com/rustsec/advisory-db.git";
  GRYPE_DB_URL = "https://github.com/anchore/grype-db.git";

  # Performance configuration
  RUSTFLAGS = "-C target-cpu=native -C target-feature=+crt-static";
  CARGO_INCREMENTAL = "1";

  # Development tools
  EDITOR = "vim";
  PAGER = "less";

  # Network configuration for development
  RUSTUP_DIST_SERVER = "https://rsproxy.cn";
  RUSTUP_UPDATE_ROOT = "https://rsproxy.cn/rustup";

  # Cachix configuration for faster builds
  NIX_CONFIG = ''
    extra-substituters = https://cache.nixos.org https://cachix.cachix.org
    extra-trusted-public-keys = cache.nixos.org-1:6NCHdD59X431o0gWypbMrA/RkbJxLv6j+7qkXqQFz0Y= cachix.cachix.org-1:WnPlPlSOyBUU7/52CejD2Q7Y8Jz1FwG1X6v1txdQ8rs=
  '';
}
