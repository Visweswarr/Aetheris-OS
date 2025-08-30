# ABI Policy for Polymera OS

*Version: 0.2.0*  
*Last Updated: {{ now }}*

This document defines the Application Binary Interface (ABI) policy for Polymera OS, including rules for maintaining backward compatibility, managing breaking changes, and versioning.

## Overview

The ABI policy ensures that applications built for one version of Polymera OS continue to work with future versions, while allowing the system to evolve and improve. This policy applies to:

- System call interfaces
- Error codes
- Feature flags
- Data structures
- Binary formats

## Core Principles

### 1. Backward Compatibility

**All ABI changes must maintain backward compatibility.** This means:

- Existing applications continue to work without modification
- System call signatures remain unchanged
- Error codes maintain their meanings
- Data structures preserve their layout
- Binary formats remain compatible

### 2. Stability Guarantees

**Stable ABI elements are guaranteed to remain unchanged:**

- System call IDs are immutable
- Error code values are immutable
- Feature flag bits are immutable
- Function signatures are immutable
- Data structure layouts are immutable

### 3. Evolution Through Addition

**New functionality is added through:**

- New system calls with new IDs
- New error codes with new values
- New feature flags with new bits
- New data structures alongside existing ones
- New binary format versions

## Versioning Strategy

### Semantic Versioning

Polymera OS uses semantic versioning (SemVer) for ABI versions:

- **Major Version (X.0.0)**: Breaking changes that require new ABI version
- **Minor Version (0.X.0)**: New functionality added, backward compatible
- **Patch Version (0.0.X)**: Bug fixes and improvements, backward compatible

### ABI Version Components

```
ABI_VERSION = MAJOR.MINOR.PATCH
```

- **MAJOR**: Incremented for breaking changes
- **MINOR**: Incremented for new features
- **PATCH**: Incremented for bug fixes

## Breaking Change Policy

### What Constitutes a Breaking Change

A breaking change is any modification that:

1. **Removes existing functionality**
   - Deletes a system call
   - Removes an error code
   - Eliminates a feature flag
   - Deletes a data structure

2. **Changes existing functionality**
   - Modifies system call signature
   - Changes error code meaning
   - Alters feature flag behavior
   - Modifies data structure layout

3. **Breaks binary compatibility**
   - Changes calling conventions
   - Modifies register usage
   - Alters memory layout
   - Changes binary format

### Breaking Change Process

When a breaking change is necessary:

1. **Proposal**: Document the proposed change and justification
2. **Review**: Technical review by maintainers
3. **Implementation**: Implement with new ABI version
4. **Testing**: Comprehensive testing of both versions
5. **Documentation**: Update all relevant documentation
6. **Migration**: Provide migration guide for users
7. **Deprecation**: Mark old version as deprecated

### Breaking Change Examples

#### ❌ Breaking Change (Not Allowed)

```rust
// OLD: sys_send(dst: u32, buf: &[u8]) -> Result<usize, Error>
// NEW: sys_send(dst: u32, buf: &[u8], flags: u32) -> Result<usize, Error>
```

This changes the function signature and breaks existing code.

#### ✅ Non-Breaking Change (Allowed)

```rust
// OLD: sys_send(dst: u32, buf: &[u8]) -> Result<usize, Error>
// NEW: sys_send_v2(dst: u32, buf: &[u8], flags: u32) -> Result<usize, Error>
```

This adds a new system call while preserving the old one.

## System Call Policy

### ID Assignment

System call IDs are assigned in ranges:

- **0x0001-0x00FF**: Task management
- **0x0100-0x01FF**: Inter-process communication
- **0x0200-0x02FF**: Capability management
- **0x0300-0x03FF**: System information
- **0x0400-0x04FF**: Memory management
- **0x0500-0x0FFF**: Reserved for future use
- **0x1000-0xFFFF**: Reserved for future use

### ID Stability

- **System call IDs are immutable** once assigned
- **Breaking changes require new IDs** in appropriate ranges
- **Old IDs remain functional** for backward compatibility
- **Deprecated IDs are marked** but not removed

### Signature Stability

- **Argument types cannot change** without new ID
- **Argument order cannot change** without new ID
- **Return types cannot change** without new ID
- **Error handling cannot change** without new ID

## Error Code Policy

### Code Assignment

Error codes are assigned sequentially:

- **0**: Success (ESUCCESS)
- **1-99**: Standard POSIX errors
- **100-199**: Extended POSIX errors
- **200-299**: Polymera-specific errors
- **300+**: Reserved for future use

### Code Stability

- **Error code values are immutable** once assigned
- **Error code meanings cannot change** without new code
- **New error codes are additive** only
- **Deprecated codes remain functional**

## Feature Flag Policy

### Bit Assignment

Feature flags use bit positions:

- **Bit 0**: PQC_CAPS_V2
- **Bit 1**: APIC_TIMER
- **Bit 2**: HPET_FALLBACK
- **Bit 3**: CAP_REPLAY_WIN
- **Bit 4**: DID_ANCHORS
- **Bit 5**: AUDIT_CODES
- **Bits 6-63**: Reserved for future use

### Flag Stability

- **Feature flag bits are immutable** once assigned
- **Flag meanings cannot change** without new bit
- **New flags are additive** only
- **Deprecated flags remain functional**

## Data Structure Policy

### Layout Stability

- **Memory layout is immutable** once defined
- **Field types cannot change** without new structure
- **Field order cannot change** without new structure
- **Field sizes cannot change** without new structure

### Evolution Strategies

1. **Additive Changes**: Add new fields at the end
2. **Versioned Structures**: Create new structure versions
3. **Optional Fields**: Use option types for new fields
4. **Extension Points**: Reserve space for future use

## Binary Format Policy

### Format Stability

- **Binary formats are immutable** once defined
- **Format versions cannot change** without new version
- **New formats are additive** only
- **Deprecated formats remain supported**

### Version Management

1. **Format Versioning**: Include version in binary data
2. **Backward Compatibility**: Support multiple format versions
3. **Migration Tools**: Provide tools for format conversion
4. **Documentation**: Document all format versions

## Testing Requirements

### Conformance Testing

All ABI changes must pass:

1. **Backward Compatibility Tests**: Verify old code still works
2. **Forward Compatibility Tests**: Verify new code works with old system
3. **Regression Tests**: Ensure no functionality is broken
4. **Performance Tests**: Verify no performance regression
5. **Stress Tests**: Test under heavy load conditions

### Test Coverage

- **100% System Call Coverage**: All syscalls must be tested
- **100% Error Code Coverage**: All error codes must be tested
- **100% Feature Flag Coverage**: All features must be tested
- **100% Data Structure Coverage**: All structures must be tested

## Documentation Requirements

### Required Documentation

1. **API Reference**: Complete system call documentation
2. **Error Code Reference**: Complete error code documentation
3. **Feature Reference**: Complete feature flag documentation
4. **Migration Guide**: Guide for upgrading applications
5. **Examples**: Working code examples for all functionality
6. **Changelog**: Complete list of all changes

### Documentation Standards

- **Clear and Concise**: Easy to understand
- **Complete**: Cover all functionality
- **Accurate**: Match actual implementation
- **Up-to-Date**: Reflect current state
- **Searchable**: Easy to find information

## Tooling Requirements

### Required Tools

1. **ABI Generator**: Generate code from schemas
2. **ABI Linter**: Check for breaking changes
3. **Conformance Tester**: Test ABI compliance
4. **Documentation Generator**: Generate documentation
5. **Version Checker**: Verify version compatibility

### Tool Integration

- **CI/CD Integration**: Automated ABI checking
- **Pre-commit Hooks**: Prevent breaking changes
- **Release Validation**: Verify release compatibility
- **Migration Validation**: Verify migration paths

## Enforcement

### Automated Enforcement

- **CI/CD Pipelines**: Block breaking changes
- **Linting Tools**: Detect policy violations
- **Testing Suites**: Verify compatibility
- **Documentation Checks**: Ensure completeness

### Manual Review

- **Code Reviews**: Human review of changes
- **Architecture Reviews**: Design review for major changes
- **Compatibility Reviews**: Verify backward compatibility
- **Documentation Reviews**: Ensure documentation quality

### Consequences

- **Breaking Changes**: Blocked until properly versioned
- **Policy Violations**: Require fixes before merge
- **Documentation Gaps**: Block release until resolved
- **Test Failures**: Require fixes before merge

## Migration Guidelines

### Application Migration

1. **Feature Detection**: Check feature availability
2. **Version Checking**: Verify system compatibility
3. **Graceful Degradation**: Handle missing features
4. **Error Handling**: Handle new error codes
5. **Testing**: Test with multiple system versions

### System Migration

1. **Dual Support**: Support old and new ABIs
2. **Migration Tools**: Provide upgrade utilities
3. **Rollback Support**: Allow reverting changes
4. **Monitoring**: Track migration progress
5. **Documentation**: Provide migration guides

## Future Considerations

### Long-term Stability

- **ABI Stability**: Maintain compatibility for 5+ years
- **Version Support**: Support multiple major versions
- **Migration Paths**: Clear upgrade paths for users
- **Deprecation Policy**: Clear deprecation timelines

### Evolution Planning

- **Roadmap**: Plan future ABI changes
- **User Feedback**: Gather requirements from users
- **Technology Trends**: Adapt to new technologies
- **Performance Goals**: Optimize for performance

## References

- [System Calls Reference](SYSCALLS.md)
- [Error Codes Reference](ERRNO.md)
- [Features Reference](FEATURES.md)
- [Development Guide](../../../docs/DEVELOPMENT.md)
- [Testing Guide](../../../docs/TESTING.md)
- [Release Process](../../../docs/RELEASE.md)

## Contact

For questions about this ABI policy:

- **Technical Issues**: Open an issue on GitHub
- **Policy Questions**: Contact the maintainers
- **Breaking Change Proposals**: Submit a proposal
- **Documentation Issues**: Submit a documentation PR
