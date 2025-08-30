package(default_visibility = ["//visibility:public"])

# Rust library target for the kernel
rust_library(
    name = "kernel_lib",
    srcs = glob(["src/**/*.rs"]),
    crate_root = "src/lib.rs",
    edition = "2021",
    features = ["asm_const"],
    rustc_flags = [
        "--target=x86_64-unknown-none",
        "--cfg=target_os=\"none\"",
        "-C", "target-cpu=native",
        "-C", "target-feature=+sse,+sse2,+sse3,+ssse3,+sse4.1,+sse4.2,+avx,+avx2",
        "-C", "link-arg=--strip-all",
        "-C", "link-arg=--gc-sections",
        "-C", "link-arg=--as-needed",
        "-C", "link-arg=--entry=_start",
        "-C", "link-arg=--no-rosegment",
    ],
    deps = [],
    proc_macro_deps = [],
    compile_data = [],
    data = [],
    aliases = {},
    rustc_env = {
        "RUSTFLAGS": "--cfg=target_os=\"none\"",
        "CARGO_INCREMENTAL": "0",
        "CARGO_PROFILE_RELEASE_DEBUG": "0",
        "CARGO_PROFILE_RELEASE_STRIP": "true",
    },
    tags = ["manual"],
)

# Kernel binary target using Cargo
genrule(
    name = "kernel_binary",
    srcs = [
        ":kernel_lib",
        "link.ld",
        ".cargo/config.toml",
        "Cargo.toml",
        "Cargo.lock",
    ],
    outs = ["polymera-kernel"],
    cmd = """
    # Set environment variables for reproducible builds
    export RUSTFLAGS="--cfg=target_os=\\"none\\""
    export CARGO_INCREMENTAL=0
    export CARGO_PROFILE_RELEASE_DEBUG=0
    export CARGO_PROFILE_RELEASE_STRIP=true
    
    # Build kernel using Cargo with release profile
    cargo build --target x86_64-unknown-none --release --manifest-path $(location Cargo.toml)
    
    # Copy the built binary
    cp target/x86_64-unknown-none/release/polymera-kernel $@
    
    # Verify the binary was created
    if [ ! -f "$@" ]; then
        echo "Error: Kernel binary was not created"
        exit 1
    fi
    
    # Show binary information
    echo "Kernel binary created: $@"
    ls -la "$@"
    """,
    tags = ["manual"],
)

# Kernel image target (binary format)
genrule(
    name = "kernel_image",
    srcs = [":kernel_binary"],
    outs = ["polymera-kernel.bin"],
    cmd = """
    # Convert ELF to binary format if needed
    # For now, just copy the binary
    cp $(location :kernel_binary) $@
    
    # Show image information
    echo "Kernel image created: $@"
    ls -la "$@"
    """,
    tags = ["manual"],
)

# Debug kernel target
genrule(
    name = "kernel_debug",
    srcs = [
        ":kernel_lib",
        "link.ld",
        ".cargo/config.toml",
        "Cargo.toml",
        "Cargo.lock",
    ],
    outs = ["polymera-kernel-debug"],
    cmd = """
    # Set environment variables for debug builds
    export RUSTFLAGS="--cfg=target_os=\\"none\\""
    export CARGO_INCREMENTAL=1
    
    # Build kernel using Cargo with debug profile
    cargo build --target x86_64-unknown-none --debug --manifest-path $(location Cargo.toml)
    
    # Copy the built binary
    cp target/x86_64-unknown-none/debug/polymera-kernel $@
    
    # Verify the binary was created
    if [ ! -f "$@" ]; then
        echo "Error: Debug kernel binary was not created"
        exit 1
    fi
    
    # Show binary information
    echo "Debug kernel binary created: $@"
    ls -la "$@"
    """,
    tags = ["manual"],
)

# Test target for the kernel
rust_binary(
    name = "kernel_test",
    srcs = ["src/lib.rs"],
    edition = "2021",
    features = ["asm_const"],
    rustc_flags = [
        "--target=x86_64-unknown-none",
        "--cfg=target_os=\"none\"",
        "--test",
    ],
    tags = ["manual"],
)

# All kernel targets
filegroup(
    name = "kernel_all",
    srcs = [
        ":kernel_lib",
        ":kernel_binary",
        ":kernel_image",
        ":kernel_debug",
        ":kernel_test",
    ],
    visibility = ["//visibility:public"],
)
        "-C", "link-arg=--no-rosegment",
    ],
    deps = [],
    proc_macro_deps = [],
    compile_data = [],
    data = [],
    aliases = {},
    rustc_env = {
        "RUSTFLAGS": "--cfg=target_os=\"none\"",
        "CARGO_INCREMENTAL": "0",
        "CARGO_PROFILE_RELEASE_DEBUG": "0",
        "CARGO_PROFILE_RELEASE_STRIP": "true",
    },
    tags = ["manual"],
)

# Kernel binary target using Cargo
genrule(
    name = "kernel_binary",
    srcs = [
        ":kernel_lib",
        "link.ld",
        ".cargo/config.toml",
        "Cargo.toml",
        "Cargo.lock",
    ],
    outs = ["polymera-kernel"],
    cmd = """
    # Set environment variables for reproducible builds
    export RUSTFLAGS="--cfg=target_os=\\"none\\""
    export CARGO_INCREMENTAL=0
    export CARGO_PROFILE_RELEASE_DEBUG=0
    export CARGO_PROFILE_RELEASE_STRIP=true
    
    # Build kernel using Cargo with release profile
    cargo build --target x86_64-unknown-none --release --manifest-path $(location Cargo.toml)
    
    # Copy the built binary
    cp target/x86_64-unknown-none/release/polymera-kernel $@
    
    # Verify the binary was created
    if [ ! -f "$@" ]; then
        echo "Error: Kernel binary was not created"
        exit 1
    fi
    
    # Show binary information
    echo "Kernel binary created: $@"
    ls -la "$@"
    """,
    tags = ["manual"],
)

# Kernel image target (binary format)
genrule(
    name = "kernel_image",
    srcs = [":kernel_binary"],
    outs = ["polymera-kernel.bin"],
    cmd = """
    # Convert ELF to binary format if needed
    # For now, just copy the binary
    cp $(location :kernel_binary) $@
    
    # Show image information
    echo "Kernel image created: $@"
    ls -la "$@"
    """,
    tags = ["manual"],
)

# Debug kernel target
genrule(
    name = "kernel_debug",
    srcs = [
        ":kernel_lib",
        "link.ld",
        ".cargo/config.toml",
        "Cargo.toml",
        "Cargo.lock",
    ],
    outs = ["polymera-kernel-debug"],
    cmd = """
    # Set environment variables for debug builds
    export RUSTFLAGS="--cfg=target_os=\\"none\\""
    export CARGO_INCREMENTAL=1
    
    # Build kernel using Cargo with debug profile
    cargo build --target x86_64-unknown-none --debug --manifest-path $(location Cargo.toml)
    
    # Copy the built binary
    cp target/x86_64-unknown-none/debug/polymera-kernel $@
    
    # Verify the binary was created
    if [ ! -f "$@" ]; then
        echo "Error: Debug kernel binary was not created"
        exit 1
    fi
    
    # Show binary information
    echo "Debug kernel binary created: $@"
    ls -la "$@"
    """,
    tags = ["manual"],
)

# Test target for the kernel
rust_binary(
    name = "kernel_test",
    srcs = ["src/lib.rs"],
    edition = "2021",
    features = ["asm_const"],
    rustc_flags = [
        "--target=x86_64-unknown-none",
        "--cfg=target_os=\"none\"",
        "--test",
    ],
    tags = ["manual"],
)

# All kernel targets
filegroup(
    name = "kernel_all",
    srcs = [
        ":kernel_lib",
        ":kernel_binary",
        ":kernel_image",
        ":kernel_debug",
        ":kernel_test",
    ],
    visibility = ["//visibility:public"],
)