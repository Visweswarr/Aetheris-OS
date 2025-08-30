# 🔄 Polymera OS Reproducible Builds

Complete reproducible build system for Polymera OS, ensuring deterministic outputs that produce identical hashes across different environments and time periods.

## 🎯 Overview

The Reproducible Builds system provides:

- **🔒 Deterministic Outputs**: Identical artifacts regardless of build environment
- **⏰ Timestamp Control**: Fixed timestamps for complete reproducibility
- **📊 Hash Verification**: Automated verification of build reproducibility
- **🏗️ Multi-Build Support**: Bazel and Nix build system integration
- **🔍 Comprehensive Testing**: Verification that builds are truly reproducible

## 🏗️ Architecture

```
tooling/repro/
├── rules.bzl                    # Bazel rules for reproducible builds
├── default.nix                  # Nix configuration for reproducible builds
├── verify_reproducibility.py    # Reproducibility verification script
└── docs.md                      # This documentation
```

## 🚀 Quick Start

### Prerequisites

- **Bazel**: Build system with reproducible build support
- **Nix**: Package manager for reproducible builds
- **Python**: 3.7+ for verification scripts
- **Build Tools**: Rust, C/C++, Go, Python toolchains

### Basic Usage

1. **Use reproducible Bazel rules**:
   ```python
   load("//tooling/repro:rules.bzl", "reproducible_rust_binary")
   
   reproducible_rust_binary(
       name = "my-binary",
       srcs = ["src/main.rs"],
       deps = ["//common:utils"],
   )
   ```

2. **Use reproducible Nix builds**:
   ```bash
   nix-build tooling/repro/default.nix
   ```

3. **Verify reproducibility**:
   ```bash
   python3 tooling/repro/verify_reproducibility.py --build-type bazel --runs 3
   ```

## 🔧 Bazel Integration

### Reproducible Build Rules

The `rules.bzl` provides drop-in replacements for standard Bazel rules:

```python
# Standard rules
rust_binary(name = "binary", srcs = ["src/main.rs"])

# Reproducible rules
reproducible_rust_binary(name = "binary", srcs = ["src/main.rs"])
```

### Available Rules

- **`reproducible_rust_binary`**: Reproducible Rust executables
- **`reproducible_rust_library`**: Reproducible Rust libraries
- **`reproducible_cc_binary`**: Reproducible C/C++ executables
- **`reproducible_cc_library`**: Reproducible C/C++ libraries

### Configuration

```python
# In your BUILD file
load("//tooling/repro:rules.bzl", "reproducible_build_config")

reproducible_build_config()

# Use reproducible rules
reproducible_rust_binary(
    name = "kernel",
    srcs = glob(["src/**/*.rs"]),
    deps = ["//common:utils"],
    visibility = ["//visibility:public"],
)
```

### Build Flags

The reproducible rules automatically apply:

```bash
# Rust flags
RUSTFLAGS="-C target-cpu=native -C codegen-units=1 -C lto=fat"
CARGO_INCREMENTAL=0
CARGO_PROFILE_RELEASE_STRIP=true
CARGO_PROFILE_RELEASE_LTO=true

# C/C++ flags
CFLAGS="-O3 -DNDEBUG -fno-ident -fno-stack-protector"
LDFLAGS="-Wl,--strip-all -Wl,--build-id=none"
```

## 🐧 Nix Integration

### Reproducible Nix Builds

The `default.nix` provides a complete reproducible build environment:

```bash
# Build with Nix
nix-build tooling/repro/default.nix

# Build specific components
nix-build tooling/repro/default.nix -A reproducibleBuild

# Verify reproducibility
nix-build tooling/repro/default.nix -A verifyReproducibility
```

### Build Environment

The Nix configuration ensures:

- **Fixed Timestamps**: All files use epoch timestamp (1970-01-01)
- **Deterministic Environment**: Identical build environment across systems
- **Stripped Binaries**: Debug symbols and build IDs removed
- **Hash Recording**: SHA256 hashes generated for all artifacts

### Build Phases

1. **Unpack**: Source code extraction
2. **Patch**: Timestamp normalization
3. **Configure**: Toolchain setup
4. **Build**: Component compilation
5. **Install**: Artifact installation
6. **Fixup**: Final reproducibility adjustments

## 🔍 Reproducibility Verification

### Automated Verification

The verification script runs multiple builds and compares outputs:

```bash
# Verify Bazel builds
python3 tooling/repro/verify_reproducibility.py --build-type bazel --runs 3

# Verify Nix builds
python3 tooling/repro/verify_reproducibility.py --build-type nix --runs 3

# Custom verification
python3 tooling/repro/verify_reproducibility.py \
    --build-type bazel \
    --runs 5 \
    --build-dir custom-build \
    --keep-artifacts
```

### Verification Process

1. **Multiple Builds**: Run specified number of builds
2. **Artifact Collection**: Gather all build outputs
3. **Hash Calculation**: Generate SHA256 hashes
4. **Comparison**: Compare hashes across builds
5. **Report Generation**: Create detailed verification report

### Expected Results

For reproducible builds:
- ✅ **All artifact hashes match** across builds
- ✅ **Identical file sets** produced
- ✅ **Deterministic outputs** regardless of environment

## ⚙️ Configuration

### Environment Variables

Set these for reproducible builds:

```bash
# General reproducibility
export SOURCE_DATE_EPOCH=0
export BUILD_DATE="1970-01-01T00:00:00Z"
export BUILD_TIMESTAMP="1970-01-01T00:00:00Z"

# Rust reproducibility
export RUSTFLAGS="-C target-cpu=native -C codegen-units=1 -C lto=fat"
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_RELEASE_STRIP=true

# C/C++ reproducibility
export CFLAGS="-O3 -DNDEBUG -fno-ident -fno-stack-protector"
export CXXFLAGS="-O3 -DNDEBUG -fno-ident -fno-stack-protector"
export LDFLAGS="-Wl,--strip-all -Wl,--build-id=none"

# Go reproducibility
export CGO_ENABLED=0
export GOOS=linux
export GOARCH=amd64
export GOMAXPROCS=1

# Python reproducibility
export PYTHONHASHSEED=0
export PYTHONDONTWRITEBYTECODE=1
```

### Bazel Configuration

Add to your `.bazelrc`:

```bash
# Reproducible build configuration
build:reproducible_build --define=reproducible_build=true
build:reproducible_build --define=build_timestamp=1970-01-01T00:00:00Z
build:reproducible_build --define=source_hash=reproducible

# Use reproducible configuration
build --config=reproducible_build
```

### Nix Configuration

Customize the Nix build:

```nix
# In your shell.nix or default.nix
{ pkgs ? import <nixpkgs> {} }:

let
  reproducibleBuild = pkgs.callPackage ./tooling/repro/default.nix {
    buildInputs = with pkgs; [ additional-tools ];
    nativeBuildInputs = with pkgs; [ additional-build-tools ];
  };
in
  reproducibleBuild
```

## 🧪 Testing and Validation

### Running Tests

```bash
# Test Bazel rules
bazel test //tooling/repro:all

# Test Nix builds
nix-build tooling/repro/default.nix -A verifyReproducibility

# Test verification script
python3 tooling/repro/verify_reproducibility.py --build-type bazel --runs 2
```

### Test Coverage

The system includes tests for:

- **Rule Functionality**: Bazel rule behavior
- **Build Reproducibility**: Hash consistency across builds
- **Environment Control**: Timestamp and variable handling
- **Artifact Generation**: Output file creation and hashing

### Validation Scripts

```bash
# Validate Bazel builds
bazel build --config=reproducible_build //...

# Validate Nix builds
nix-build --no-out-link tooling/repro/default.nix

# Validate reproducibility
python3 tooling/repro/verify_reproducibility.py --verbose
```

## 📊 Reporting and Analysis

### Build Reports

Each build generates:

- **Build Info**: Metadata about the build process
- **Component Hashes**: SHA256 hashes for all artifacts
- **Reproducibility Report**: Markdown summary of build results

### Verification Reports

The verification script produces:

- **JSON Report**: Machine-readable verification data
- **Markdown Report**: Human-readable analysis
- **Hash Comparisons**: Detailed hash matching results
- **Reproducibility Analysis**: Success/failure assessment

### Report Examples

```json
{
  "verification_timestamp": "2024-01-01T00:00:00Z",
  "build_type": "bazel",
  "num_runs": 3,
  "reproducible": true,
  "summary": {
    "total_builds": 3,
    "successful_builds": 3,
    "failed_builds": 0,
    "reproducible": true
  }
}
```

## 🔧 Advanced Configuration

### Custom Build Rules

Extend the reproducible rules:

```python
def custom_reproducible_rule(name, srcs, **kwargs):
    """Custom reproducible rule with additional features."""
    
    # Get base reproducible configuration
    env = _get_reproducible_env("rust")
    build_flags = _get_reproducible_flags("rust")
    
    # Add custom configuration
    env.update({
        "CUSTOM_FLAG": "custom_value",
        "ADDITIONAL_OPTION": "enabled"
    })
    
    # Create the rule
    rust_binary(
        name = name,
        srcs = srcs,
        env = env,
        rustc_flags = build_flags + ["--custom-flag"],
        **kwargs
    )
```

### Custom Nix Derivations

Extend the Nix build:

```nix
# Custom reproducible derivation
customReproducibleBuild = stdenv.mkDerivation {
  name = "custom-reproducible";
  version = "1.0.0";
  
  inherit reproducibleEnv;
  
  # Custom build phases
  buildPhase = ''
    echo "Custom build phase"
    # Your custom build logic here
  '';
  
  # Custom installation
  installPhase = ''
    mkdir -p $out/bin
    # Custom installation logic
  '';
};
```

### Integration with CI/CD

```yaml
# GitHub Actions example
- name: Verify Reproducible Builds
  run: |
    python3 tooling/repro/verify_reproducibility.py \
      --build-type bazel \
      --runs 3 \
      --build-dir reproducible-builds

- name: Upload Verification Report
  uses: actions/upload-artifact@v3
  with:
    name: reproducibility-report
    path: reproducible-builds/*.md
```

## 🔍 Troubleshooting

### Common Issues

#### Build Not Reproducible

```bash
# Check environment variables
env | grep -E "(SOURCE_DATE_EPOCH|BUILD_DATE|RUSTFLAGS)"

# Verify Bazel configuration
bazel query --output=location //tooling/repro:rules.bzl

# Check Nix configuration
nix-instantiate --show-trace tooling/repro/default.nix
```

#### Hash Mismatches

```bash
# Compare build artifacts
diff -r build/run_1 build/run_2

# Check file timestamps
find build -type f -exec stat -c "%y %n" {} \;

# Verify build flags
bazel query --output=build //target:name
```

#### Verification Failures

```bash
# Run with verbose output
python3 tooling/repro/verify_reproducibility.py --verbose

# Check build logs
cat build/*/bazel_root/bazel-out/*/logs/build.log

# Verify toolchain versions
rustc --version
gcc --version
go version
```

### Debug Mode

Enable debug output:

```bash
# Bazel debug
bazel build --verbose_failures --config=reproducible_build //...

# Nix debug
nix-build --show-trace tooling/repro/default.nix

# Python debug
python3 -v tooling/repro/verify_reproducibility.py
```

### Log Analysis

Review build logs for issues:

```bash
# Bazel logs
find . -name "*.log" -exec grep -l "error\|warning" {} \;

# Nix logs
nix log $(nix-build --no-out-link tooling/repro/default.nix)

# Verification logs
cat build/*_verification_report.txt
```

## 📚 API Reference

### Bazel Rules API

```python
# Core functions
reproducible_rust_binary(name, srcs, deps, **kwargs)
reproducible_rust_library(name, srcs, deps, **kwargs)
reproducible_cc_binary(name, srcs, deps, **kwargs)
reproducible_cc_library(name, srcs, deps, **kwargs)

# Configuration
reproducible_build_config()
reproducible_build_verification(name, artifacts, **kwargs)

# Constants
REPRODUCIBLE_BUILD_CONFIG
```

### Nix API

```nix
# Main derivations
reproducibleBuild
verifyReproducibility

# Configuration
reproducibleEnv
fixedTimestamp
```

### Python API

```python
class ReproducibilityVerifier:
    def __init__(self, build_dir: str, num_runs: int)
    def verify_reproducibility(self, build_type: str) -> bool
    def generate_verification_report(self) -> str
    def cleanup_builds(self)
```

## 🤝 Contributing

### Adding New Build Systems

1. **Create build function** in `verify_reproducibility.py`
2. **Add build type** to argument parser
3. **Implement artifact collection** for the new system
4. **Add tests** for the new build system
5. **Update documentation** with usage examples

### Extending Reproducible Rules

1. **Add new rule type** to `rules.bzl`
2. **Implement reproducible configuration** for the language
3. **Add build flags** and environment variables
4. **Create tests** for the new rule
5. **Update configuration** constants

### Enhancing Verification

1. **Add new verification checks** to the verifier
2. **Implement additional metrics** for reproducibility
3. **Create custom report formats** for specific needs
4. **Add integration tests** for new features
5. **Update documentation** with new capabilities

## 📄 License

This tooling is part of Polymera OS and follows the same licensing terms.

---

**🔄 Deterministic Builds, Trusted Artifacts!** 🔄

For questions, issues, or contributions, please refer to the main Polymera OS documentation or submit an issue in the repository.
