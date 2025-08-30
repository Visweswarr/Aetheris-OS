# NGFS Integrity System Developer Guide

## Overview

The NGFS Integrity System ensures that all polyglot implementations (Rust, Go, Python, TypeScript) produce byte-for-byte identical outputs for critical files. This guide explains how to use the system, when and how to rebaseline, and the required justifications.

## Quick Start

### Running Integrity Checks

```bash
# Check integrity of all files
./bazel-bin/go/tools/ngfs-integrity check --manifest tooling/integrity/manifest.yml

# Check with verbose output
./bazel-bin/go/tools/ngfs-integrity check --manifest tooling/integrity/manifest.yml --verbose

# Output as JSON for CI
./bazel-bin/go/tools/ngfs-integrity check --manifest tooling/integrity/manifest.yml --json
```

### Expected Output

```json
{"test":"integrity","files_checked":42,"mismatch":0}
```

## When Integrity Checks Fail

### Common Causes

1. **Schema Changes**: Updates to NGFS schemas that affect CBOR/JSON output
2. **Implementation Drift**: Different serialization behavior between languages
3. **Fixture Updates**: Changes to test fixtures or golden files
4. **Performance Baseline Changes**: Updates to performance expectations
5. **Build Environment Changes**: Different compiler versions or flags

### Diagnosing Issues

Use the Python diff reporter to understand differences:

```bash
# Compare CBOR files
python3 tooling/python/integrity_diff.py old.cbor new.cbor

# Compare JSON files
python3 tooling/python/integrity_diff.py old.json new.json

# The diff will show:
# - Type mismatches
# - Missing/extra fields
# - Value differences
# - Structural changes
```

## Rebaseline Workflow

### When to Rebaseline

**DO rebaseline when:**
- ✅ Legitimate schema changes are made
- ✅ Test fixtures are intentionally updated
- ✅ Performance baselines are adjusted
- ✅ Bug fixes change output format
- ✅ New features are added

**DON'T rebaseline when:**
- ❌ Random test failures occur
- ❌ Build environment issues exist
- ❌ Implementation bugs are present
- ❌ Unintended changes happen

### Required Justifications

Every rebaseline operation requires:

1. **Reason**: Clear explanation of why the change is needed
2. **Ticket**: Issue tracker reference (e.g., "NGFS-123")
3. **Phase Update**: Current development phase
4. **Audit Logging**: Automatic logging of the operation

### Rebaseline Commands

```bash
# Update all files (use with caution)
ngfs-integrity update \
  --manifest tooling/integrity/manifest.yml \
  --reason "P3-01-A8: Schema update for new feature" \
  --ticket "NGFS-123"

# Update specific file categories
ngfs-integrity update \
  --manifest tooling/integrity/manifest.yml \
  --reason "P3-01-A8: Performance baseline adjustment" \
  --ticket "NGFS-124" \
  --category performance

# Update with custom phase
ngfs-integrity update \
  --manifest tooling/integrity/manifest.yml \
  --reason "P3-01-A8: Schema update" \
  --ticket "NGFS-125" \
  --phase "P3-01-A8"
```

### Rebaseline Process

1. **Identify the Issue**: Understand why integrity check failed
2. **Fix the Root Cause**: Resolve the underlying issue
3. **Test the Fix**: Ensure the fix works correctly
4. **Rebaseline**: Update the manifest with proper justification
5. **Commit Changes**: Include both fix and manifest update
6. **Verify**: Run integrity check to confirm success

## File Categories

### Schemas (`schemas`)

Core NGFS schema definitions:
- `services/ngfs/src/schema.rs`
- `tooling/schema/ngfs_schema.rs`
- `abi/ngfs.yaml`

**Critical**: Yes - Changes affect all implementations

### CBOR Fixtures (`cbor_fixtures`)

Test fixtures and golden files:
- `tests/ngfs/fixtures/dir_simple.cbor`
- `tests/ngfs/fixtures/file_1chunk.cbor`
- `tests/ngfs/fixtures/ipfs/*.cbor`

**Critical**: Yes - Changes affect test validation

### Performance (`performance`)

Performance baselines and results:
- `perf/baselines/p3_ngfs.json`
- `perf/history/p3_ngfs.jsonl`

**Critical**: No - Changes don't affect core functionality

### IPFS Exports (`ipfs_exports`)

IPFS export fixtures and maps:
- `tests/ngfs/fixtures/ipfs/dir_simple.cbor`
- `tests/ngfs/fixtures/ipfs/file_1chunk.cbor`

**Critical**: Yes - Changes affect IPFS compatibility

### Test Vectors (`test_vectors`)

Test data and validation files:
- `tests/ngfs/fixtures/*.test`
- `tests/ngfs/vectors/*.vec`

**Critical**: No - Changes don't affect core functionality

### Generated Outputs (`generated_outputs`)

Build artifacts and generated files:
- `target/ngfs/schema_hash.json`
- `bazel-bin/ngfs/schema.out`

**Critical**: No - Changes don't affect source integrity

### Configs (`configs`)

Configuration files and settings:
- `.github/workflows/phase-3-gates.yml`
- `bazel/ngfs.bzl`

**Critical**: No - Changes don't affect core functionality

## CI Integration

### Pre-Test Gate

The integrity sentinel runs as a pre-test gate in CI:

```yaml
# .github/workflows/phase-3-gates.yml
ngfs-integrity:
  name: NGFS Integrity Sentinel
  needs: [ngfs-schema, ngfs-features, ngfs-syscalls]
  # ... integrity check steps ...

ngfs-perf:
  name: NGFS Performance Benchmarks
  needs: [ngfs-schema, ngfs-features, ngfs-syscalls, ngfs-integrity]
  # ... performance steps ...
```

### Failure Handling

- **Build Blocked**: No PR can merge if integrity fails
- **Artifacts Stored**: Results and manifest diffs saved
- **Notifications**: Team notified of integrity failures
- **Audit Trail**: All operations logged for review

## Best Practices

### Development Workflow

1. **Local Testing**: Run integrity checks before committing
2. **Incremental Changes**: Make small, focused changes
3. **Documentation**: Update docs when schemas change
4. **Testing**: Add tests for new functionality
5. **Review**: Get code review for significant changes

### Schema Evolution

1. **Backward Compatibility**: Maintain compatibility when possible
2. **Versioning**: Use semantic versioning for schemas
3. **Migration**: Provide migration paths for breaking changes
4. **Documentation**: Document all schema changes
5. **Testing**: Test all language implementations

### Performance Baselines

1. **Statistical Rigor**: Use sufficient sample sizes
2. **Environment Consistency**: Maintain consistent test environments
3. **Regression Detection**: Monitor for performance regressions
4. **Budget Management**: Adjust budgets based on improvements
5. **History Tracking**: Maintain performance history

## Troubleshooting

### Common Issues

#### "File not found" Errors

```bash
# Check if file exists
ls -la tooling/integrity/manifest.yml

# Verify file paths in manifest
cat tooling/integrity/manifest.yml | grep -A 5 "path:"
```

#### "Hash mismatch" Errors

```bash
# Compare files manually
diff old.cbor new.cbor

# Use diff reporter
python3 tooling/python/integrity_diff.py old.cbor new.cbor

# Check file permissions
ls -la old.cbor new.cbor
```

#### "Schema validation" Errors

```bash
# Validate JSON manually
python3 -m json.tool file.json

# Check TypeScript schemas
node bazel-bin/tooling/ts/integrity-check.js manifest.yml

# Verify schema compatibility
bazel test //tooling/ts:tests/test_integrity
```

### Debug Mode

Enable verbose output for debugging:

```bash
# Go CLI verbose mode
ngfs-integrity check --manifest manifest.yml --verbose

# Python diff reporter debug
python3 tooling/python/integrity_diff.py old.cbor new.cbor --debug

# TypeScript validator debug
node bazel-bin/tooling/ts/integrity-check.js manifest.yml --debug
```

## Security Considerations

### Access Control

- **Manifest Updates**: Require proper justification and tickets
- **Audit Logging**: All operations logged with timestamps
- **Review Process**: Changes require code review
- **CI Enforcement**: Automated checks prevent bypass

### Data Protection

- **File Size Limits**: Maximum 10 MB per file
- **Hash Validation**: Blake3-256 for secure hashing
- **Offline Operation**: No network access during checks
- **Input Validation**: Comprehensive validation of all inputs

### Threat Model

The integrity system protects against:

- **Accidental Changes**: Unintended file modifications
- **Implementation Drift**: Inconsistent behavior across languages
- **Schema Evolution**: Uncontrolled schema changes
- **Build Reproducibility**: Non-deterministic builds

## Future Enhancements

### Planned Features

1. **Automated Rebaseline**: CI-triggered rebaseline for known changes
2. **Change Detection**: Git-based change detection and validation
3. **Schema Evolution**: Automated schema compatibility checking
4. **Performance Monitoring**: Continuous performance regression detection
5. **Cross-Platform**: Windows and macOS support

### Integration Points

1. **Git Hooks**: Pre-commit integrity checks
2. **IDE Integration**: Real-time validation in editors
3. **Monitoring**: Dashboard for integrity status
4. **Alerts**: Automated notifications for failures
5. **Metrics**: Integrity check performance metrics

## Support and Resources

### Documentation

- [NGFS v1 Overview](../phase-3/NGFS-V1.md)
- [Performance Harness Guide](../perf/NGFS-PERF.md)
- [Schema Reference](../schema/NGFS-SCHEMA.md)

### Tools

- **Integrity Sentinel**: `tooling/integrity/sentinel.rs`
- **Go CLI**: `go/tools/ngfs-integrity`
- **Python Diff**: `tooling/python/integrity_diff.py`
- **TypeScript Validator**: `tooling/ts/integrity_check.ts`

### Tests

- **Rust Tests**: `tests/integrity/sentinel_smoke.rs`
- **Python Tests**: `tooling/python/tests/test_diff.py`
- **TypeScript Tests**: `tooling/ts/tests/test_integrity.ts`

### CI Jobs

- **Integrity Check**: `ngfs-integrity` job in phase-3-gates.yml
- **Performance**: `ngfs-perf` job (depends on integrity)
- **Schema Validation**: `ngfs-schema` job

## Conclusion

The NGFS Integrity System provides a robust foundation for maintaining consistency across polyglot implementations. By following the rebaseline workflow and using proper justifications, developers can ensure that the system remains reliable while allowing legitimate evolution.

Remember: **Integrity is not optional** - it's a core requirement for the NGFS system. When in doubt, ask for help rather than bypassing the system.
