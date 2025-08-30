{
  description = "Polymera OS - Next-generation quantum-ready operating system";

  inputs = {
    # Nixpkgs
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    # Rust toolchains
    rust-overlay.url = "github:oxalica/rust-overlay";

    # Bazel
    bazel.url = "github:bazelbuild/bazel";

    # WASI SDK
    wasi-sdk.url = "github:WebAssembly/wasi-sdk";

    # QEMU for testing
    qemu.url = "github:qemu/qemu";

    # Security tools
    sigstore.url = "github:sigstore/sigstore";
    grype.url = "github:anchore/grype";
    cyclonedx.url = "github:CycloneDX/cyclonedx-cli";

    # Performance tools
    criterion.url = "github:bheisler/criterion.rs";

    # Development tools
    devshell.url = "github:numtide/devshell";

    # Flake utils
    flake-utils.url = "github:numtide/flake-utils";

    # Go toolchain
    gomod2nix.url = "github:tweag/gomod2nix";

    # Node.js LTS
    nodejs.url = "github:nodejs/node/v20.11.0";

    # Foundry for Ethereum development
    foundry.url = "github:foundry-rs/foundry";

    # Noir for zero-knowledge proofs
    noir.url = "github:noir-lang/noir";

    # liboqs for post-quantum cryptography
    liboqs.url = "github:open-quantum-safe/liboqs";

    # WasmEdge for WebAssembly runtime
    wasmedge.url = "github:WasmEdge/WasmEdge";
  };

  outputs = { self, nixpkgs, rust-overlay, bazel, wasi-sdk, qemu, sigstore, grype, cyclonedx, criterion, devshell, flake-utils, gomod2nix, nodejs, foundry, noir, liboqs, wasmedge }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
            gomod2nix.overlays.default
            (final: prev: {
              # Custom package overrides
              polymera-os = final.callPackage ./nix/packages.nix { };
              
              # Pin specific versions
              python311 = final.python311;
              nodejs-lts = final.nodejs_20;
              go = final.go_1_21;
              llvm = final.llvmPackages_17;
            })
          ];
        };

        # Rust toolchains with specific versions
        rustStable = pkgs.rust-bin.stable."1.75.0".default.override {
          targets = [ "x86_64-unknown-linux-gnu" "wasm32-wasi" "aarch64-unknown-linux-gnu" ];
        };

        rustNightly = pkgs.rust-bin.nightly."2024-01-15".default.override {
          targets = [ "x86_64-unknown-linux-gnu" "wasm32-wasi" "aarch64-unknown-linux-gnu" ];
        };

        # LLVM toolchain with C++17 support
        llvm = pkgs.llvmPackages_17;

        # Development dependencies
        devDeps = with pkgs; [
          # Build tools
          bazel
          cmake
          ninja
          pkg-config

          # Rust tools
          rustStable
          rustNightly
          cargo-audit
          cargo-tarpaulin
          cargo-watch
          cargo-fuzz
          rust-src

          # Protocol buffers
          protobuf
          grpc-tools

          # WASI tools
          wasmtime
          wasm-pack
          wasmedge

          # Security tools
          sigstore
          grype
          cyclonedx

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

          # Audio
          alsa-lib
          pulseaudio

          # Network
          libpcap
          libnetfilter_conntrack

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
          nodejs-lts
          yarn

          # Go toolchain
          go
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
        runtimeDeps = with pkgs; [
          glibc
          libc
          openssl
          zlib
        ];

      in {
        # Development shell
        devShells.default = pkgs.mkShell {
          name = "polymera-os-dev";

          buildInputs = devDeps ++ runtimeDeps;

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
            export WASI_SDK_PATH="${wasi-sdk}"
            export WASMTIME_HOME="${pkgs.wasmtime}"
            export WASMEDGE_HOME="${pkgs.wasmedge}"

            # Security configuration
            export SIGSTORE_CT_LOG_PUBLIC_KEY_FILE="${sigstore}/share/sigstore/ctfe.pub"

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
            export NODE_PATH="${pkgs.nodejs-lts}/lib/node_modules"
            export NODE_VERSION="20.11.0"

            # Go configuration
            export GOPATH="$HOME/go"
            export GOROOT="${pkgs.go}"
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
            echo "📦 Rust Stable: $(rustc --version)"
            echo "📦 Rust Nightly: $(rustc +nightly --version)"
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
            echo "  cargo +nightly test   - Run tests with nightly Rust"
            echo "  cargo audit           - Security audit"
            echo "  cargo tarpaulin       - Code coverage"
            echo "  cargo fuzz run        - Run fuzz tests"
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
        };

        # Packages
        packages = {
          default = pkgs.polymera-os;

          # Individual components
          kernel = pkgs.callPackage ./nix/kernel.nix { };
          services = pkgs.callPackage ./nix/services.nix { };
          runtime = pkgs.callPackage ./nix/runtime.nix { };
          ui = pkgs.callPackage ./nix/ui.nix { };
          tooling = pkgs.callPackage ./nix/tooling.nix { };
        };

        # Apps
        apps = {
          default = pkgs.writeShellScriptBin "polymera-os" ''
            echo "Polymera OS - Next-generation quantum-ready operating system"
            echo "Use 'nix develop' to enter the development environment"
          '';

          build = pkgs.writeShellScriptBin "build" ''
            bazel build //...
          '';

          test = pkgs.writeShellScriptBin "test" ''
            bazel test //...
          '';

          fuzz = pkgs.writeShellScriptBin "fuzz" ''
            cargo fuzz run
          '';

          forge = pkgs.writeShellScriptBin "forge" ''
            ${pkgs.foundry}/bin/forge "$@"
          '';

          nargo = pkgs.writeShellScriptBin "nargo" ''
            ${pkgs.noir}/bin/nargo "$@"
          '';
        };

        # Checks
        checks = {
          # Build checks
          build = self.packages.${system}.default;

          # Test checks
          test = pkgs.runCommand "test-check" {} ''
            echo "Running tests..."
            # TODO: Implement test running
            touch $out
          '';

          # Format checks
          format = pkgs.runCommand "format-check" {} ''
            echo "Checking formatting..."
            # TODO: Implement format checking
            touch $out
          '';

          # Lint checks
          lint = pkgs.runCommand "lint-check" {} ''
            echo "Running lints..."
            # TODO: Implement lint checking
            touch $out
          '';
        };
      }
    );
}
