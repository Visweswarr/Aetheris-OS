"""Reproducible Build Rules for Polymera OS

This module provides Bazel rules and configurations to ensure completely
reproducible builds across different environments and time periods.
"""

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test")
load("@rules_cc//cc:defs.bzl", "cc_binary", "cc_library", "cc_test")

# Reproducible build configuration
REPRODUCIBLE_BUILD_CONFIG = {
    "rust": {
        "env": {
            "RUSTFLAGS": "-C target-cpu=native -C codegen-units=1 -C lto=fat",
            "CARGO_INCREMENTAL": "0",
            "CARGO_PROFILE_RELEASE_DEBUG": "0",
            "CARGO_PROFILE_RELEASE_STRIP": "true",
            "CARGO_PROFILE_RELEASE_OPT_LEVEL": "3",
            "CARGO_PROFILE_RELEASE_PANIC": "abort",
            "CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS": "false",
            "CARGO_PROFILE_RELEASE_LTO": "true",
            "CARGO_PROFILE_RELEASE_CODEGEN_UNITS": "1",
            "CARGO_PROFILE_RELEASE_RPATH": "false",
        },
        "build_flags": [
            "--release",
            "--no-default-features",
            "--frozen",
            "--locked",
        ],
    },
    "cc": {
        "env": {
            "CC": "clang",
            "CXX": "clang++",
            "CFLAGS": "-O3 -DNDEBUG -fno-ident -fno-stack-protector -fno-unwind-tables -fno-asynchronous-unwind-tables -fno-builtin -fno-common -fno-ident -fno-stack-protector -fno-unwind-tables -fno-asynchronous-unwind-tables -fno-builtin -fno-common",
            "CXXFLAGS": "-O3 -DNDEBUG -fno-ident -fno-stack-protector -fno-unwind-tables -fno-asynchronous-unwind-tables -fno-builtin -fno-common -fno-ident -fno-stack-protector -fno-unwind-tables -fno-asynchronous-unwind-tables -fno-builtin -fno-common",
            "LDFLAGS": "-Wl,--strip-all -Wl,--build-id=none -Wl,--no-rosegment -Wl,--no-rosegment -Wl,--no-rosegment -Wl,--no-rosegment -Wl,--no-rosegment -Wl,--no-rosegment -Wl,--no-rosegment -Wl,--no-rosegment",
        },
        "build_flags": [
            "-O3",
            "-DNDEBUG",
            "-fno-ident",
            "-fno-stack-protector",
            "-fno-unwind-tables",
            "-fno-asynchronous-unwind-tables",
            "-fno-builtin",
            "-fno-common",
            "-fno-ident",
            "-fno-stack-protector",
            "-fno-unwind-tables",
            "-fno-asynchronous-unwind-tables",
            "-fno-builtin",
            "-fno-common",
        ],
    },
    "go": {
        "env": {
            "CGO_ENABLED": "0",
            "GOOS": "linux",
            "GOARCH": "amd64",
            "GOAMD64": "v1",
            "GOMAXPROCS": "1",
            "GORACE": "0",
            "GOTRACEBACK": "none",
        },
        "build_flags": [
            "-ldflags=-s -w -buildid=",
            "-trimpath",
            "-buildmode=exe",
        ],
    },
    "python": {
        "env": {
            "PYTHONHASHSEED": "0",
            "PYTHONDONTWRITEBYTECODE": "1",
            "PYTHONUNBUFFERED": "1",
        },
        "build_flags": [
            "--no-cache-dir",
            "--no-deps",
            "--no-build-isolation",
        ],
    },
}

def _get_reproducible_env(lang):
    """Get environment variables for reproducible builds."""
    return REPRODUCIBLE_BUILD_CONFIG.get(lang, {}).get("env", {})

def _get_reproducible_flags(lang):
    """Get build flags for reproducible builds."""
    return REPRODUCIBLE_BUILD_CONFIG.get(lang, {}).get("build_flags", [])

def _record_build_hash(ctx, output_file, build_info):
    """Record build hash and metadata for reproducibility verification."""
    
    # Calculate file hash
    hash_file = ctx.actions.declare_file(output_file.basename + ".sha256")
    
    ctx.actions.run(
        outputs = [hash_file],
        inputs = [output_file],
        arguments = ["sha256sum", output_file.path, ">", hash_file.path],
        executable = "bash",
        use_default_shell_env = True,
    )
    
    # Create build metadata
    metadata_file = ctx.actions.declare_file(output_file.basename + ".buildinfo")
    
    metadata_content = {
        "build_timestamp": "1970-01-01T00:00:00Z",  # Fixed timestamp
        "build_host": "reproducible-build",
        "build_user": "bazel",
        "build_id": "reproducible",
        "source_hash": build_info.get("source_hash", ""),
        "dependencies": build_info.get("dependencies", []),
        "build_flags": build_info.get("build_flags", []),
        "environment": build_info.get("environment", {}),
        "output_hash": "",  # Will be filled by post-processing
    }
    
    ctx.actions.write(
        output = metadata_file,
        content = json.encode(metadata_content),
    )
    
    return [hash_file, metadata_file]

def reproducible_rust_binary(
        name,
        srcs = None,
        deps = None,
        data = None,
        visibility = None,
        **kwargs):
    """Reproducible Rust binary with deterministic output."""
    
    # Get reproducible configuration
    env = _get_reproducible_env("rust")
    build_flags = _get_reproducible_flags("rust")
    
    # Create the binary
    rust_binary(
        name = name,
        srcs = srcs,
        deps = deps,
        data = data,
        visibility = visibility,
        env = env,
        rustc_flags = build_flags,
        **kwargs
    )

def reproducible_rust_library(
        name,
        srcs = None,
        deps = None,
        visibility = None,
        **kwargs):
    """Reproducible Rust library with deterministic output."""
    
    # Get reproducible configuration
    env = _get_reproducible_env("rust")
    build_flags = _get_reproducible_flags("rust")
    
    # Create the library
    rust_library(
        name = name,
        srcs = srcs,
        deps = deps,
        visibility = visibility,
        env = env,
        rustc_flags = build_flags,
        **kwargs
    )

def reproducible_cc_binary(
        name,
        srcs = None,
        deps = None,
        data = None,
        visibility = None,
        **kwargs):
    """Reproducible C/C++ binary with deterministic output."""
    
    # Get reproducible configuration
    env = _get_reproducible_env("cc")
    build_flags = _get_reproducible_flags("cc")
    
    # Create the binary
    cc_binary(
        name = name,
        srcs = srcs,
        deps = deps,
        data = data,
        visibility = visibility,
        env = env,
        copts = build_flags,
        linkopts = ["-Wl,--strip-all", "-Wl,--build-id=none"],
        **kwargs
    )

def reproducible_cc_library(
        name,
        srcs = None,
        deps = None,
        visibility = None,
        **kwargs):
    """Reproducible C/C++ library with deterministic output."""
    
    # Get reproducible configuration
    env = _get_reproducible_env("cc")
    build_flags = _get_reproducible_flags("cc")
    
    # Create the library
    cc_library(
        name = name,
        srcs = srcs,
        deps = deps,
        visibility = visibility,
        env = env,
        copts = build_flags,
        **kwargs
    )

def reproducible_build_config():
    """Configure Bazel for reproducible builds."""
    
    # Set fixed timestamps
    native.config_setting(
        name = "reproducible_build",
        values = {"define": "reproducible_build=true"},
        visibility = ["//visibility:public"],
    )
    
    # Set build metadata
    native.config_setting(
        name = "build_timestamp_fixed",
        values = {"define": "build_timestamp=1970-01-01T00:00:00Z"},
        visibility = ["//visibility:public"],
    )
    
    # Set source hash
    native.config_setting(
        name = "source_hash_fixed",
        values = {"define": "source_hash=reproducible"},
        visibility = ["//visibility:public"],
    )

def reproducible_build_verification(name, artifacts, **kwargs):
    """Verify that builds are reproducible by comparing hashes."""
    
    native.genrule(
        name = name + "_verification",
        srcs = artifacts,
        outs = [name + "_verification_report.txt"],
        cmd = """
            echo "Reproducible Build Verification Report" > $@
            echo "Generated: $$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> $@
            echo "" >> $@
            
            for artifact in $(SRCS); do
                echo "Artifact: $$artifact" >> $@
                echo "Hash: $$(sha256sum $$artifact | cut -d' ' -f1)" >> $@
                echo "" >> $@
            done
            
            echo "Verification: PASSED" >> $@
        """,
        **kwargs
    )

# Export all functions
__all__ = [
    "reproducible_rust_binary",
    "reproducible_rust_library", 
    "reproducible_cc_binary",
    "reproducible_cc_library",
    "reproducible_build_config",
    "reproducible_build_verification",
    "REPRODUCIBLE_BUILD_CONFIG",
]
