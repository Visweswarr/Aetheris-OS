# Polymera OS CLI (polymeractl)

A comprehensive command-line interface for Polymera OS development tasks, providing easy access to building, testing, image creation, verification, SBOM generation, and signing capabilities.

## 🚀 Features

- **Build System**: Compile kernel, services, crypto components, and UI
- **QEMU Integration**: Run virtual machines with OVMF firmware
- **Image Management**: Create system images with A/B slots
- **Package Verification**: Validate packages and images
- **SBOM Generation**: Generate Software Bill of Materials in multiple formats
- **Digital Signing**: Sign packages and images with cryptographic keys
- **Project Management**: Initialize new projects and manage development environment

## 📦 Installation

### Prerequisites

- Rust 1.70+ and Cargo
- QEMU (for virtualization features)
- OpenSSL (for cryptographic operations)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/polymera-os/polymera-os.git
cd polymera-os

# Build the CLI
cargo build --release --package polymeractl

# Install globally (optional)
cargo install --path tools/polymeractl
```

## 🎯 Quick Start

### Basic Usage

```bash
# Show help
polymeractl --help

# Show available commands
polymeractl info

# Show detailed system information
polymeractl info --detailed
```

### Building Components

```bash
# Build all components
polymeractl build

# Build specific target
polymeractl build --target kernel

# Build with release profile
polymeractl build --profile release

# Build with parallel compilation
polymeractl build --parallel
```

### Running QEMU

```bash
# Run QEMU with default settings
polymeractl run-qemu

# Run with specific architecture
polymeractl run-qemu --arch aarch64

# Run with custom memory and CPU settings
polymeractl run-qemu --memory 1024 --cpus 4

# Enable networking
polymeractl run-qemu --network
```

### Creating System Images

```bash
# Create basic system image
polymeractl mkimage

# Create image with A/B slots
polymeractl mkimage --ab-slots

# Create image in specific format
polymeractl mkimage --format qcow2

# Create image with verification
polymeractl mkimage --verify
```

### Verifying Components

```bash
# Verify all components
polymeractl verify

# Verify specific target
polymeractl verify --target package

# Generate verification report
polymeractl verify --report verification.json

# Show detailed output
polymeractl verify --detailed
```

### Generating SBOMs

```bash
# Generate SPDX SBOM
polymeractl sbom --format spdx

# Generate CycloneDX SBOM
polymeractl sbom --format cyclonedx

# Include dependencies
polymeractl sbom --dependencies

# Validate generated SBOM
polymeractl sbom --validate
```

### Signing Components

```bash
# Sign all components
polymeractl sign

# Sign with specific algorithm
polymeractl sign --algorithm ed25519

# Generate new signing key
polymeractl sign --target package

# Verify signatures after signing
polymeractl sign --verify
```

### Project Management

```bash
# Initialize new project
polymeractl init my-project

# Initialize with service template
polymeractl init my-service --template service

# Initialize with git repository
polymeractl init my-project --git

# Update development environment
polymeractl update --all

# Clean build artifacts
polymeractl clean --all
```

## 🔧 Configuration

### Environment Variables

- `POLYMERA_LOG_LEVEL`: Set logging level (trace, debug, info, warn, error)
- `POLYMERA_NO_COLOR`: Disable colored output
- `POLYMERA_CONFIG_PATH`: Path to configuration file

### Configuration File

Create `~/.polymera/config.toml` for global settings:

```toml
[cli]
default_target = "all"
default_profile = "debug"
enable_colors = true
log_level = "info"

[build]
parallel_jobs = 0
output_directory = "dist"
clean_before_build = false

[qemu]
default_arch = "x86_64"
default_memory = 512
default_cpus = 2
ovmf_path = "/usr/share/ovmf"

[signing]
default_algorithm = "ed25519"
key_directory = "~/.polymera/keys"
```

## 📚 Command Reference

### Global Options

- `-v, --verbose`: Enable verbose output
- `-d, --debug`: Enable debug output
- `--no-color`: Suppress colored output

### Build Command

```bash
polymeractl build [OPTIONS]

OPTIONS:
    -t, --target <TARGET>        Target to build (kernel, services, crypto, ui, all) [default: all]
    -p, --profile <PROFILE>      Build profile (debug, release, optimized) [default: debug]
    -o, --output <OUTPUT>        Output directory [default: dist]
    --parallel                    Enable parallel builds
    --no-test                    Skip tests
    --packages                   Build packages
    --docs                       Build documentation
    --clean                      Clean before building
    --progress                   Show build progress
```

### Run-QEMU Command

```bash
polymeractl run-qemu [OPTIONS]

OPTIONS:
    -a, --arch <ARCH>            Architecture (x86_64, aarch64) [default: x86_64]
    -k, --kernel <KERNEL>        Kernel image path
    -d, --disk <DISK>            Disk image path
    -m, --memory <MEMORY>        Memory size in MB [default: 512]
    -c, --cpus <CPUS>            Number of CPU cores [default: 2]
    --graphics                    Enable graphics (disable for headless)
    --network                     Enable network
    --serial <SERIAL>            Serial output file
    --debug                       Enable debugging
    --qemu-bin <QEMU_BIN>        QEMU binary path
    --ovmf-path <OVMF_PATH>      OVMF firmware path
    --extra-args <EXTRA_ARGS>    Additional QEMU arguments
```

### Mkimage Command

```bash
polymeractl mkimage [OPTIONS]

OPTIONS:
    -o, --output <OUTPUT>        Output image path [default: polymera.img]
    -s, --source <SOURCE>        Source directory [default: dist]
    --size <SIZE>                Image size in GB [default: 10]
    --ab-slots                   Enable A/B slots
    --slot-size <SLOT_SIZE>      Slot size in MB [default: 2048]
    --compress                    Enable compression
    --encrypt                     Enable encryption
    --key-file <KEY_FILE>        Encryption key file
    --format <FORMAT>            Image format (raw, qcow2, vmdk) [default: raw]
    --verify                      Enable verification
    --progress                    Show progress
    --force                       Overwrite existing image
```

### Verify Command

```bash
polymeractl verify [OPTIONS]

OPTIONS:
    -t, --target <TARGET>        Target to verify (package, image, all) [default: all]
    -p, --path <PATH>            Path to package or image
    --content-hash               Verify content hash
    --signatures                  Verify signatures
    --file-integrity             Verify file integrity
    --capabilities                Verify capabilities
    --dependencies                Verify dependencies
    --sbom                        Verify SBOM
    --security                   Verify security
    --strict                      Strict mode (fail on warnings)
    --output <OUTPUT>            Output format (text, json, yaml) [default: text]
    --report <REPORT>            Save verification report
    --detailed                    Show detailed output
```

### SBOM Command

```bash
polymeractl sbom [OPTIONS]

OPTIONS:
    -a, --action <ACTION>        Action to perform (generate, validate, merge, diff) [default: generate]
    -s, --source <SOURCE>        Source directory or file [default: .]
    -o, --output <OUTPUT>        Output file path
    -f, --format <FORMAT>        SBOM format (spdx, cyclonedx, swid) [default: spdx]
    --version <VERSION>          SBOM version
    --dependencies                Include dependencies
    --licenses                    Include licenses
    --vulnerabilities            Include vulnerabilities
    --validate                    Validate against schema
    --detailed                    Show detailed output
    --output-format <OUTPUT_FORMAT>  Output format (json, xml, yaml, text) [default: json]
```

### Sign Command

```bash
polymeractl sign [OPTIONS]

OPTIONS:
    -t, --target <TARGET>        Target to sign (package, image, all) [default: all]
    -p, --path <PATH>            Path to package or image
    -k, --key <KEY>              Signing key file
    --algorithm <ALGORITHM>      Signing algorithm (ed25519, rsa, ecdsa) [default: ed25519]
    --passphrase <PASSPHRASE>    Key passphrase
    -o, --output <OUTPUT>        Output signature file
    --detached                    Detached signature
    --verify                      Verify after signing
    --detailed                    Show detailed output
    --force                       Overwrite existing signatures
```

## 🧪 Testing

### Run Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_build_command

# Run with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration
```

### Test Coverage

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

## 🔍 Troubleshooting

### Common Issues

1. **QEMU not found**: Install QEMU via your package manager
2. **OVMF firmware missing**: Install OVMF package or specify path with `--ovmf-path`
3. **Permission denied**: Ensure you have write permissions to output directories
4. **Build failures**: Check that all dependencies are installed and up to date

### Debug Mode

```bash
# Enable debug output
polymeractl --debug build

# Set log level
RUST_LOG=debug polymeractl build
```

### Verbose Output

```bash
# Enable verbose output
polymeractl --verbose build

# Show progress
polymeractl build --progress
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

### Development Setup

```bash
# Clone and setup
git clone https://github.com/polymera-os/polymera-os.git
cd polymera-os/tools/polymeractl

# Install development dependencies
cargo install cargo-watch

# Run tests in watch mode
cargo watch -x test

# Format code
cargo fmt

# Lint code
cargo clippy
```

## 📄 License

This project is licensed under the MIT License or Apache License 2.0 - see the [LICENSE](../LICENSE) file for details.

## 🆘 Support

- **Documentation**: [Polymera OS Docs](https://docs.polymera-os.org)
- **Issues**: [GitHub Issues](https://github.com/polymera-os/polymera-os/issues)
- **Discord**: [Polymera OS Community](https://discord.gg/polymera-os)
- **Email**: [support@polymera-os.org](mailto:support@polymera-os.org)

## 🔮 Roadmap

- [ ] Plugin system for custom commands
- [ ] Integration with CI/CD pipelines
- [ ] Remote development environment support
- [ ] Advanced debugging and profiling tools
- [ ] Package repository management
- [ ] Multi-language project support
- [ ] Cloud deployment integration
- [ ] Performance benchmarking tools
