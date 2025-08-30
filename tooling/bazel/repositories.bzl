"""Custom repository rules for Polymera OS."""

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:git.bzl", "git_repository")

def polymera_repositories():
    """Initialize Polymera OS specific repositories."""
    
    # Polymera OS specific dependencies
    http_archive(
        name = "polymera_common",
        sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
        strip_prefix = "polymera-common-0.1.0",
        urls = ["https://github.com/polymera-os/common/releases/download/v0.1.0/polymera-common-0.1.0.tar.gz"],
    )
    
    # Quantum computing libraries
    http_archive(
        name = "qiskit",
        sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
        strip_prefix = "qiskit-0.44.0",
        urls = ["https://github.com/Qiskit/qiskit/archive/0.44.0.tar.gz"],
    )
    
    # Zero-knowledge proof libraries
    http_archive(
        name = "arkworks",
        sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
        strip_prefix = "arkworks-0.4.0",
        urls = ["https://github.com/arkworks-rs/arkworks/archive/v0.4.0.tar.gz"],
    )
    
    # Post-quantum cryptography
    http_archive(
        name = "liboqs",
        sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
        strip_prefix = "liboqs-0.7.2",
        urls = ["https://github.com/open-quantum-safe/liboqs/archive/v0.7.2.tar.gz"],
    )
    
    # WebAssembly tools
    http_archive(
        name = "wasmtime",
        sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
        strip_prefix = "wasmtime-12.0.1",
        urls = ["https://github.com/bytecodealliance/wasmtime/releases/download/v12.0.1/wasmtime-v12.0.1-src.tar.gz"],
    )
    
    # Blockchain and smart contract tools
    http_archive(
        name = "foundry",
        sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
        strip_prefix = "foundry-1.7.0",
        urls = ["https://github.com/foundry-rs/foundry/releases/download/v1.7.0/foundry-v1.7.0.tar.gz"],
    )
    
    # Noir zero-knowledge language
    http_archive(
        name = "noir",
        sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
        strip_prefix = "noir-0.25.0",
        urls = ["https://github.com/noir-lang/noir/releases/download/v0.25.0/noir-v0.25.0.tar.gz"],
    )
    
    # Security and verification tools
    http_archive(
        name = "sigstore",
        sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
        strip_prefix = "sigstore-1.0.0",
        urls = ["https://github.com/sigstore/sigstore/releases/download/v1.0.0/sigstore-v1.0.0.tar.gz"],
    )
    
    # Performance and benchmarking tools
    http_archive(
        name = "criterion",
        sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
        strip_prefix = "criterion-0.5.1",
        urls = ["https://github.com/bheisler/criterion.rs/releases/download/0.5.1/criterion-v0.5.1.tar.gz"],
    )
    
    # Development and testing tools
    http_archive(
        name = "proptest",
        sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
        strip_prefix = "proptest-1.3.0",
        urls = ["https://github.com/AltSysrq/proptest/archive/v1.3.0.tar.gz"],
    )
    
    # Documentation and analysis tools
    http_archive(
        name = "rustdoc",
        sha256 = "c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5c9c5",
        strip_prefix = "rustdoc-1.75.0",
        urls = ["https://github.com/rust-lang/rust/archive/1.75.0.tar.gz"],
    )
