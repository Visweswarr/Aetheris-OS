# Surface Guard Tool

The Surface Guard tool protects critical Polymera OS interfaces from accidental changes while allowing intentional modifications through proper approval workflows.

## Features

- **Interface Locking**: Prevents unapproved changes to critical surfaces
- **Golden Surface Tracking**: SHA-256 hashing of protected interfaces
- **Content Normalization**: Consistent hashing across platforms
- **Generator Integration**: Automatic regeneration of derived files
- **Allowlist Management**: Controlled approval of intentional changes
- **CI Integration**: Fails builds on interface drift
- **HTML Reports**: Detailed diff reports for review

## Usage

### Basic Commands

```bash
# Check all surfaces for drift
bazel run //tooling/guard:surface_check

# Generate HTML report
bazel run //tooling/guard:surface_check -- --html-report report.html

# Regenerate golden surfaces
bazel run //tooling/guard:surface_check -- --generate

# Verbose output
bazel run //tooling/guard:surface_check -- --verbose
```

### Command Line Options

- `--lock-file <path>`: Path to SURFACE.lock.json (default: SURFACE.lock.json)
- `--allow-file <path>`: Path to SURFACE.allow.yaml (default: SURFACE.allow.yaml)
- `--generate`: Regenerate golden surfaces before checking
- `--html-report <path>`: Generate HTML diff report
- `--verbose`: Enable debug logging

## Architecture

### Core Components

- **`main.rs`**: CLI interface and orchestration
- **`surface.rs`**: Surface definitions and status checking
- **`normalize.rs`**: Content normalization for consistent hashing
- **`diff.rs`**: Diff generation and reporting

### Data Flow

1. **Load Configuration**: Read SURFACE.lock.json and SURFACE.allow.yaml
2. **Regenerate Surfaces**: Run generators for derived files
3. **Check Status**: Hash normalized content and compare with golden hashes
4. **Generate Reports**: Create HTML diff reports for drift analysis
5. **Exit Status**: Return non-zero on unapproved drift

### Normalization Process

1. **Line Endings**: Convert to LF (Unix style)
2. **Whitespace**: Collapse multiple spaces, trim trailing
3. **Timestamps**: Remove generated timestamps and dates
4. **Markdown Tables**: Normalize table formatting
5. **Content Semantics**: Preserve meaningful content

## Configuration

### Surface Lock File (SURFACE.lock.json)

```json
{
  "version": "1.0.0",
  "generated_at": "2024-12-19T00:00:00Z",
  "surfaces": [
    {
      "id": "abi_public",
      "paths": ["kernel/src/syscall/ids.rs"],
      "kind": "generated",
      "owner": "@abi-owners",
      "notes": "ABI numbers are immutable once released",
      "golden_hash": "abc123...",
      "generator": "//tooling/abi:gen"
    }
  ]
}
```

### Allowlist File (SURFACE.allow.yaml)

```yaml
allowances:
  - path_glob: "include/abi/polymera_syscalls.h"
    reason: "Add new syscall for Phase 3"
    author: "@abi-owners"
    expires: "2025-12-31T23:59:59Z"
```

## Integration

### Git Hooks

Install the pre-commit hook:
```bash
chmod +x .githooks/pre-commit
git config core.hooksPath .githooks
```

### CI/CD

Add to your workflow:
```yaml
- name: Check surface integrity
  run: |
    bazel run //tooling/abi:gen
    bazel run //tooling/guard:surface_check -- --html-report interface-diff.html
```

### Bazel

Build and run:
```bash
# Build
bazel build //tooling/guard:surface_check

# Run
bazel run //tooling/guard:surface_check
```

## Performance

- **Local Check**: <3 seconds typical
- **CI Check**: <90 seconds maximum
- **Pre-commit**: <5 seconds typical
- **Memory Usage**: <50MB typical

## Security

- **No Network Access**: All operations are local
- **Deterministic Hashing**: SHA-256 with consistent normalization
- **Audit Trail**: All changes tracked in git
- **Owner Approval**: Changes require CODEOWNERS approval
- **Expiration Dates**: Temporary allowances auto-expire

## Troubleshooting

### Common Issues

1. **Generator Failed**: Check tool dependencies and configuration
2. **File Not Found**: Verify paths in SURFACE.lock.json
3. **Hash Mismatch**: Review for intentional vs. accidental changes
4. **Permission Denied**: Ensure proper file access rights

### Debug Mode

Enable verbose logging:
```bash
bazel run //tooling/guard:surface_check -- --verbose
```

### Manual Hash Calculation

For debugging, manually calculate hashes:
```bash
# Normalize and hash a file
cat file.txt | tr -d '\r' | sed 's/[[:space:]]*$//' | sha256sum
```

## Development

### Building from Source

```bash
cd tooling/guard
cargo build --release
```

### Running Tests

```bash
cargo test
bazel test //tooling/guard/...
```

### Adding New Features

1. **Extend Surface Types**: Add new surface kinds in `surface.rs`
2. **Enhance Normalization**: Add normalization rules in `normalize.rs`
3. **Improve Diffing**: Enhance diff algorithms in `diff.rs`
4. **Add CLI Options**: Extend argument parsing in `main.rs`

## Contributing

1. **Fork** the repository
2. **Create** a feature branch
3. **Implement** your changes
4. **Add tests** for new functionality
5. **Submit** a pull request

## License

MIT OR Apache-2.0 - see LICENSE file for details.
