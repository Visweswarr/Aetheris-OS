# Interface Locks & Golden Surfaces

## Overview

The Interface Locks system prevents accidental changes to critical Polymera OS interfaces while allowing intentional modifications through proper approval workflows. This system is essential for maintaining API stability and preventing silent regressions in contracts, generated files, and critical configurations.

## What is Guarded

The following surfaces are protected by the interface lock system:

### ABI Interfaces
- **System Calls**: `kernel/src/syscall/ids.rs`, `include/abi/polymera_syscalls.h`, `docs/abi/SYSCALLS.md`
- **Error Codes**: `docs/abi/ERRNO.md`, `include/abi/polymera_errno.h`
- **Features**: `docs/abi/FEATURES.md`, `include/abi/polymera_features.h`

### Security Interfaces
- **Audit Codes**: `kernel/src/secman/audit_codes.rs` (enum IDs and definitions)
- **Capability Schema**: `kernel/src/security/cap.rs` (CapToken header schema)

### System Configuration
- **Timer Selection**: `docs/phase-2/APIC.md` (boot banner format)
- **Performance Thresholds**: `perf/baselines/p2.json` (SLO gates and limits)
- **Policy Engine**: `policy/out/p2_boot.wasm` (compiled policy binary)
- **Policy Guards**: `docs/phase-2/POLICY-GUARDS.md` (default modes)

### CI Workflows
- **Phase 2 Gates**: `.github/workflows/phase-2-gates.yml` (critical validation steps)

## How It Works

### 1. Golden Surface Tracking
Each protected surface has a "golden hash" stored in `SURFACE.lock.json`. This hash represents the known-good state of the interface.

### 2. Deterministic Hashing
- **SHA-256** hashing for all surfaces
- **Normalized content**: LF line endings, collapsed whitespace, stripped timestamps
- **Consistent across platforms** and CI environments

### 3. Pre-commit Protection
The `.githooks/pre-commit` hook automatically:
1. Regenerates golden surfaces (ABI, policy WASM)
2. Checks surface integrity
3. Blocks commits with unapproved drift

### 4. CI Integration
The `interface-lock` job in phase-2-gates:
1. Runs before all other validation
2. Fails CI on unapproved interface changes
3. Generates HTML diff reports for review

## Usage

### Local Development

#### Install Git Hook
```bash
chmod +x .githooks/pre-commit
git config core.hooksPath .githooks
```

#### Check Surface Status
```bash
# Regenerate and check all surfaces
bazel run //tooling/guard:surface_check

# Generate HTML report
bazel run //tooling/guard:surface_check -- --html-report report.html

# Regenerate golden surfaces only
bazel run //tooling/guard:surface_check -- --generate
```

#### Manual Surface Regeneration
```bash
# ABI interfaces
bazel run //tooling/abi:gen

# Policy WASM
cd policy && ./compile.sh
```

### Intentional Changes

When you need to modify a protected interface:

#### 1. Add Allowlist Entry
Edit `SURFACE.allow.yaml`:
```yaml
allowances:
  - path_glob: "include/abi/polymera_syscalls.h"
    reason: "Add sys_foo for DeviceKit in Phase 3"
    author: "@abi-owners"
    expires: "2025-12-31T23:59:59Z"
```

#### 2. Make Your Changes
```bash
# Edit the protected files
vim include/abi/polymera_syscalls.h

# Regenerate if needed
bazel run //tooling/abi:gen
```

#### 3. Verify Allowance
```bash
bazel run //tooling/guard:surface_check
```

#### 4. Commit
```bash
git add .
git commit -m "Add sys_foo syscall with approval"
```

### Unintentional Changes

If you accidentally modified a protected interface:

#### 1. Check What Changed
```bash
bazel run //tooling/guard:surface_check
```

#### 2. Restore from Git
```bash
git checkout -- SURFACE.lock.json
git checkout -- include/abi/polymera_syscalls.h
```

#### 3. Regenerate Properly
```bash
bazel run //tooling/abi:gen
bazel run //tooling/guard:surface_check
```

## File Structure

```
SURFACE.lock.json          # Golden surface definitions and hashes
SURFACE.allow.yaml         # Intentional change approvals
.githooks/pre-commit       # Git hook for local protection
tooling/guard/             # Surface guard tool
├── Cargo.toml
├── src/
│   ├── main.rs           # CLI interface
│   ├── surface.rs        # Surface management
│   ├── normalize.rs      # Content normalization
│   └── diff.rs          # Diff generation
```

## Configuration

### Adding New Protected Surfaces

1. **Edit SURFACE.lock.json**:
```json
{
  "id": "new_surface",
  "paths": ["path/to/file1", "path/to/file2"],
  "kind": "static",
  "owner": "@team-owners",
  "notes": "Description of what this protects",
  "golden_hash": "0000000000000000000000000000000000000000000000000000000000000000",
  "generator": null
}
```

2. **Set Initial Hash**:
```bash
bazel run //tooling/guard:surface_check -- --generate
```

3. **Commit the Lock File**:
```bash
git add SURFACE.lock.json
git commit -m "Add new_surface to interface locks"
```

### Surface Types

- **`generated`**: Files created by tools (ABI headers, policy WASM)
- **`static`**: Manually maintained files (source code, docs)

### Generator Commands

For generated surfaces, specify the Bazel target:
```json
"generator": "//tooling/abi:gen"
```

## Troubleshooting

### Common Issues

#### Generator Failed
```
❌ Generator //tooling/abi:gen failed: exit status 1
```
**Solution**: Fix the generator tool or its dependencies

#### File Not Found
```
❌ Surface abi_public: ERROR - One or more files do not exist
```
**Solution**: Ensure all paths in SURFACE.lock.json exist

#### Hash Mismatch
```
⚠️  abi_public: DRIFT DETECTED
```
**Solution**: Check if change is intentional, add to allowlist, or restore from git

#### Pre-commit Hook Failing
```
❌ Surface drift detected! Please review and fix before committing.
```
**Solution**: Run `bazel run //tooling/guard:surface_check` to see details

### Performance

- **Local check**: <3 seconds typical
- **CI check**: <90 seconds maximum
- **Pre-commit**: <5 seconds typical

### Debug Mode

Enable verbose logging:
```bash
bazel run //tooling/guard:surface_check -- --verbose
```

## Best Practices

### 1. Regular Updates
- Update golden hashes after intentional changes
- Review allowlist entries monthly
- Remove expired allowances

### 2. Team Coordination
- Coordinate interface changes with surface owners
- Use descriptive reasons in allowlist entries
- Set reasonable expiration dates

### 3. CI Integration
- Always run interface-lock before other jobs
- Review diff reports for unexpected changes
- Fail fast on interface drift

### 4. Documentation
- Keep surface descriptions current
- Document breaking changes in allowlist
- Update this guide when adding new surfaces

## Security Considerations

- **No network access**: All operations are local
- **Deterministic hashing**: Consistent across environments
- **Audit trail**: All changes tracked in git
- **Owner approval**: Changes require CODEOWNERS approval
- **Expiration dates**: Temporary allowances auto-expire

## Future Enhancements

- **Automated allowlist management**: Web UI for approvals
- **Integration with CODEOWNERS**: Automatic owner validation
- **Change impact analysis**: Dependency graph for interfaces
- **Rollback automation**: Quick restoration of golden states
- **Metrics dashboard**: Drift frequency and resolution time
