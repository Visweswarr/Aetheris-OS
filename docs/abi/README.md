# Polymera OS Application Binary Interface (ABI)

*Version: 0.2.0*  
*Last Updated: {{ now }}*

This directory contains the complete Application Binary Interface (ABI) system for Polymera OS, providing a stable, versioned interface for applications to interact with the kernel.

## Overview

The Polymera OS ABI system provides:

- **Stable System Call Interface**: Well-defined system calls with immutable IDs
- **Comprehensive Error Handling**: Standardized error codes and handling patterns
- **Feature Detection**: Runtime feature availability checking
- **Multi-Architecture Support**: x86_64 and aarch64 calling conventions
- **Backward Compatibility**: Guaranteed compatibility across minor versions

## Architecture

```
abi/
├── syscalls.yaml          # System call definitions
├── errno.yaml            # Error code definitions
├── features.yaml         # Feature flag definitions
└── README.md            # This file

tooling/abi/
├── src/
│   ├── main.rs          # ABI generator main
│   ├── model.rs         # Data structures
│   └── abi_lint.rs      # ABI linting tool
├── templates/            # Code generation templates
│   ├── kernel_*.tera    # Kernel code templates
│   ├── rust_*.tera      # Rust stub templates
│   ├── c_*.tera         # C header templates
│   └── *_md.md.tera     # Documentation templates
└── Cargo.toml           # Rust package definition

generated/                # Generated files (not committed)
├── kernel/              # Kernel system call tables
├── userland/            # Userland stub libraries
├── include/             # C header files
└── docs/                # Generated documentation
```

## Components

### 1. **System Calls** (`syscalls.yaml`)

Defines the complete system call interface:

- **Task Management**: Process creation, termination, scheduling
- **Inter-Process Communication**: Message passing, channels
- **Capability Management**: Security tokens, verification
- **System Information**: Feature detection, time queries
- **Memory Management**: Mapping, protection, cleanup

**Key Features:**
- Immutable numeric IDs (0x0001-0xFFFF)
- Architecture-specific calling conventions
- Comprehensive safety documentation
- Argument validation rules

### 2. **Error Codes** (`errno.yaml`)

Standardized error handling:

- **POSIX Compatibility**: Standard error codes (EPERM, ENOENT, etc.)
- **Extended Errors**: Polymera-specific error conditions
- **Categorized**: Permission, resource, validation errors
- **Stable Values**: Immutable error code assignments

### 3. **Feature Flags** (`features.yaml`)

Runtime feature detection:

- **Security Features**: PQC capabilities, replay protection
- **Hardware Support**: APIC timer, HPET fallback
- **System Capabilities**: Audit logging, DID anchors
- **Development Tools**: Testing, debugging, analysis

## Generated Outputs

### Kernel Integration

- **`kernel/src/syscall/table.rs`**: System call dispatch table
- **`kernel/src/syscall/ids.rs`**: Stable numeric constants

### Userland Libraries

- **`userland-stubs/src/lib.rs`**: Rust system call wrappers
- **`include/abi/*.h`**: C header files for applications

### Documentation

- **`docs/abi/SYSCALLS.md`**: Complete system call reference
- **`docs/abi/ERRNO.md`**: Error code reference
- **`docs/abi/FEATURES.md`**: Feature flag reference
- **`docs/abi/EXAMPLES.md`**: Working code examples
- **`docs/abi/ABI-POLICY.md`**: Compatibility policy

## Usage

### For Application Developers

```c
#include <polymera/polymera_syscalls.h>
#include <polymera/polymera_errno.h>
#include <polymera/polymera_features.h>

int main() {
    // Check feature availability
    unsigned long long features = sys_get_features();
    if (features & PQC_CAPS_V2) {
        printf("PQC capabilities available\n");
    }
    
    // Make system calls
    int result = sys_yield();
    if (result < 0) {
        printf("Yield failed: %s\n", strerror(-result));
    }
    
    return 0;
}
```

### For Rust Developers

```rust
use polymera_os::syscalls;
use polymera_os::errno::*;
use polymera_os::features::*;

fn main() -> Result<(), ErrorCode> {
    // Check features
    let features = syscalls::sys_get_features()?;
    if features & PQC_CAPS_V2 != 0 {
        println!("PQC capabilities available");
    }
    
    // Make system calls
    syscalls::sys_yield()?;
    Ok(())
}
```

### For System Integrators

```bash
# Install headers
sudo cp include/*.h /usr/local/include/polymera/

# Compile applications
gcc -std=c11 -I/usr/local/include/polymera -o app app.c

# Link with libraries
gcc -o app app.c -lpolymera
```

## Development

### Building the ABI Generator

```bash
cd tooling/abi
cargo build --release
```

### Generating ABI Files

```bash
# Generate all files
./target/release/abi-gen generate --input-dir ../../abi --output-dir ../../generated

# Validate schemas only
./target/release/abi-gen validate --input-dir ../../abi

# Show information
./target/release/abi-gen info --input-dir ../../abi
```

### Running ABI Lint

```bash
# Check for breaking changes
./target/release/abi-lint check --current-dir ../../abi --strict

# Generate compatibility report
./target/release/abi-lint report --current-dir ../../abi --output report.md
```

### Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --package tests --test conformance

# Test C header compilation
gcc -std=c11 -Wall -Wextra test_headers.c
```

## CI/CD Integration

### Automated Workflows

1. **ABI Conformance Testing** (`.github/workflows/abi-conformance.yml`)
   - Generates ABI files on every change
   - Checks for drift between generated and committed files
   - Runs conformance tests
   - Validates schema integrity
   - Ensures documentation completeness

2. **ABI Release Publishing** (`.github/workflows/abi-release.yml`)
   - Automatically publishes releases on version tags
   - Creates GitHub releases with all assets
   - Packages headers, documentation, and examples
   - Provides multiple download formats

### Quality Gates

- **Schema Validation**: All YAML schemas must be valid
- **Drift Detection**: Generated files must match committed versions
- **Conformance Testing**: All system calls must pass tests
- **Documentation Completeness**: All required docs must exist
- **Breaking Change Detection**: ABI lint must pass

## Versioning and Compatibility

### Version Strategy

- **Major Version (X.0.0)**: Breaking changes requiring new ABI version
- **Minor Version (0.X.0)**: New functionality, backward compatible
- **Patch Version (0.0.X)**: Bug fixes, backward compatible

### Compatibility Guarantees

- **System Call IDs**: Immutable once assigned
- **Error Code Values**: Stable meanings across versions
- **Feature Flag Bits**: Unchanged semantics
- **Function Signatures**: Preserved argument types and order
- **Data Structures**: Maintained memory layout

### Breaking Change Process

1. **Proposal**: Document change and justification
2. **Review**: Technical review by maintainers
3. **Implementation**: New ABI version with old preserved
4. **Testing**: Comprehensive compatibility testing
5. **Documentation**: Update all relevant docs
6. **Migration**: Provide migration guides
7. **Deprecation**: Mark old version as deprecated

## Security Considerations

### Capability System

- **PQC Signatures**: Post-quantum cryptographic verification
- **Replay Protection**: Nonce-based token validation
- **DID Trust**: Decentralized identifier anchors
- **Audit Logging**: Comprehensive security event tracking

### Access Control

- **Permission Checks**: Runtime capability verification
- **Resource Isolation**: Process boundary enforcement
- **Input Validation**: Comprehensive argument checking
- **Error Handling**: Secure error reporting

## Performance Characteristics

### System Call Overhead

- **Direct Calls**: Minimal overhead for simple operations
- **Batch Operations**: Efficient multi-call patterns
- **Async Support**: Non-blocking operation modes
- **Memory Management**: Optimized allocation patterns

### Resource Usage

- **Memory Footprint**: Minimal runtime overhead
- **CPU Efficiency**: Optimized calling conventions
- **Cache Locality**: Efficient data structure layout
- **Scalability**: Linear scaling with system size

## Troubleshooting

### Common Issues

1. **Permission Denied (EPERM)**
   - Check capability requirements
   - Verify system configuration
   - Ensure proper privilege level

2. **Resource Not Found (ENOENT)**
   - Verify resource existence
   - Check resource paths/IDs
   - Ensure accessibility

3. **Out of Memory (ENOMEM)**
   - Reduce buffer sizes
   - Check system memory usage
   - Use memory mapping alternatives

4. **Invalid Parameters (EINVAL)**
   - Verify parameter types and ranges
   - Check flag combinations
   - Ensure proper alignment

### Debugging Tips

1. **Enable Logging**: Use environment variables for debug output
2. **Check Return Values**: Always verify system call results
3. **Use Examples**: Start with working examples from documentation
4. **Test Incrementally**: Add complexity gradually
5. **Verify Assumptions**: Don't assume features or resources exist

## Contributing

### Development Guidelines

1. **Schema Changes**: Update YAML files, not generated code
2. **Backward Compatibility**: Maintain existing interfaces
3. **Documentation**: Update docs for all changes
4. **Testing**: Add tests for new functionality
5. **Validation**: Ensure schema validation passes

### Code Review Process

1. **Schema Review**: Verify YAML structure and content
2. **Generator Review**: Check code generation logic
3. **Template Review**: Validate output templates
4. **Test Review**: Ensure comprehensive test coverage
5. **Documentation Review**: Verify documentation accuracy

## References

### Documentation

- [System Calls Reference](SYSCALLS.md)
- [Error Codes Reference](ERRNO.md)
- [Features Reference](FEATURES.md)
- [Examples](EXAMPLES.md)
- [ABI Policy](ABI-POLICY.md)

### External Resources

- [Polymera OS Main Documentation](../../../docs/)
- [Development Guide](../../../docs/DEVELOPMENT.md)
- [Testing Guide](../../../docs/TESTING.md)
- [Release Process](../../../docs/RELEASE.md)

### Standards

- [System V ABI](https://www.uclibc.org/docs/psABI-x86_64.pdf)
- [ARM AAPCS64](https://github.com/ARM-software/abi-aa/releases)
- [POSIX Error Codes](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/errno.h.html)

## Contact

For questions about the ABI system:

- **Technical Issues**: Open an issue on GitHub
- **Feature Requests**: Submit a feature proposal
- **Breaking Changes**: Follow the breaking change process
- **Documentation**: Submit documentation PRs
- **General Questions**: Contact the maintainers

## License

This ABI system is part of Polymera OS and is licensed under the same terms as the main project.

---

*This documentation is automatically generated and maintained as part of the ABI system. For the most up-to-date information, refer to the generated files.*
