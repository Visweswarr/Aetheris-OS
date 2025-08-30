# NGFS Development Tools

This document provides a comprehensive guide to the NGFS development tools, including the FUSE mount system, tree browser, and verification utilities.

## Overview

The NGFS toolchain provides several key components for development and testing:

- **FUSE Mount System**: Host-side read-only POSIX filesystem interface
- **Tree Browser**: Interactive TUI for exploring mounted NGFS trees
- **Verification Tools**: Tree integrity and policy enforcement
- **CLI Wrappers**: User-friendly command-line interfaces

## Quick Start

### Prerequisites

Ensure you have the required dependencies:

```bash
# Build all NGFS targets
bazel build //c/fuse:aeth_fuse //services/ngfs:fuse //go/tools:ngfsmount //tooling/python:ngfs_ls_verify //tooling/ts:ngfs-browse

# Install system dependencies (Linux)
sudo apt-get install fuse3 libfuse3-dev

# Install system dependencies (macOS)
brew install macfuse
```

### Basic Usage

```bash
# 1. Mount an NGFS snapshot
ngfsmount --store ngfs.dat --idx ngfs.idx --at ./mnt --root $(cat root.cid)

# 2. Browse the mounted tree
ngfs-browse --mnt ./mnt

# 3. Verify tree integrity
ngfs_ls_verify.py --mnt ./mnt --fixtures tests/ngfs/fixtures

# 4. Unmount when done
ngfsumount ./mnt
```

## FUSE Mount System

### Architecture

The FUSE mount system consists of three layers:

1. **C FUSE Shim** (`libaeth_fuse`): Platform-specific FUSE protocol implementation
2. **Rust FUSE Server** (`ngfs-fuse`): NGFS logic and CAS integration
3. **Go CLI Wrapper** (`ngfsmount`): User interface and mount management

### Mount Sources

The system supports two primary mount sources:

#### Store + Index Mode
```bash
ngfsmount --store ngfs.dat --idx ngfs.idx --at ./mnt --root <CID>
```

- **`--store`**: Path to NGFS segment store file
- **`--idx`**: Path to NGFS index file
- **`--at`**: Mount point directory
- **`--root`**: Root directory CID to mount

#### Snapshot Mode
```bash
ngfsmount --snapshot snapshots/*.cbor --at ./mnt --root <CID>
```

- **`--snapshot`**: Glob pattern for snapshot files
- **`--at`**: Mount point directory
- **`--root`**: Root directory CID to mount

### Optional Parameters

```bash
# Development key directory
ngfsmount --store ngfs.dat --idx ngfs.idx --at ./mnt --root <CID> --keydir ./keys

# CI-safe fake mode (no kernel mount)
ngfs-fuse --fake-fuse --store ngfs.dat --idx ngfs.idx --at ./mnt --root <CID>
```

### Security Features

- **Read-Only**: All mutation operations return `EROFS`
- **Path Validation**: Denies dotdot escapes, enforces NFC normalization
- **No Control Characters**: Rejects filenames with control characters
- **Deterministic**: `st_ino`/`st_ctime` derived from CIDs

### Performance Features

- **Page-Aligned Reads**: 64 KiB maximum read windows
- **Inode Cache**: LRU cache bounded to 8192 entries
- **Sorted Directory Listing**: Consistent `readdir` output
- **Statistics Tracking**: Opens, reads, and p95 read times

## Tree Browser (ngfs-browse)

### Features

The TypeScript TUI provides an interactive interface for exploring mounted NGFS trees:

- **Directory Navigation**: Browse through directory hierarchies
- **File Preview**: Hex dumps and content previews for small files
- **Metadata Display**: File sizes, permissions, modification times
- **CID Information**: Content identifiers for all entries
- **Statistics**: Directory/file counts and total size

### Usage

```bash
# Basic browsing
ngfs-browse --mnt ./mnt

# Verbose output
ngfs-browse --mnt ./mnt --verbose

# JSON output mode
ngfs-browse --mnt ./mnt --json
```

### Navigation Commands

- **Arrow Keys**: Navigate through entries
- **Enter**: Open directories or preview files
- **Backspace**: Go to parent directory
- **q**: Quit the browser
- **h**: Show help

### Display Features

#### Directory Listing
```
📁 dir1/                    [755] 2024-01-01 12:00:00
📁 dir2/                    [755] 2024-01-01 12:00:00
📄 file1.txt               [644] 1.0 KB  2024-01-01 12:00:00
📄 file2.txt               [644] 2.0 KB  2024-01-01 12:00:00
```

#### File Preview
```
File: /path/to/file.txt
Size: 1.0 KB
Mode: -rw-r--r--
CID: b94a8fe5ccb19ba61c4c0873d391e987982fbbd3

Hex Dump (first 64 bytes):
00000000: 48 65 6c 6c 6f 2c 20 57 6f 72 6c 64 21 0a        Hello, World!.

ASCII:
Hello, World!
```

## Verification Tools

### Tree Verification (ngfs_ls_verify.py)

The Python verification tool validates mounted trees against expected fixtures:

```bash
# Basic verification
ngfs_ls_verify.py --mnt ./mnt --fixtures tests/ngfs/fixtures

# JSON output
ngfs_ls_verify.py --mnt ./mnt --fixtures tests/ngfs/fixtures --json

# Verbose output
ngfs_ls_verify.py --mnt ./mnt --fixtures tests/ngfs/fixtures --verbose
```

#### Verification Features

- **File Content**: Size and SHA256 hash verification
- **Directory Structure**: Expected vs. actual file/directory listings
- **NFC Policy**: Unicode normalization enforcement
- **Path Validation**: Control character and escape sequence detection

#### Fixture Format

```json
{
  "file1.txt": {
    "size": 1024,
    "sha256": "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3"
  },
  "dir1": {
    "type": "directory",
    "children": ["nested.txt"]
  },
  "nested.txt": {
    "size": 512,
    "sha256": "b94a8fe5ccb19ba61c4c0873d391e987982fbbd4"
  }
}
```

### Policy Enforcement

The verification tool enforces several security policies:

#### Filename Constraints
- **Length**: 1-255 bytes
- **Control Characters**: No characters < 0x20 or = 0x7F
- **Dotdot Sequences**: No `..` patterns
- **Leading/Trailing Dots**: No `.` at start or end
- **Empty Names**: No empty or whitespace-only names

#### Unicode Normalization
- **NFC Form**: Filenames must be in Normalization Form Canonical Composition
- **Combining Characters**: Automatic normalization with warnings
- **Consistency**: Same content always produces same normalized form

## Development Workflow

### Local Development

```bash
# 1. Build all targets
bazel build //c/fuse:aeth_fuse //services/ngfs:fuse //go/tools:ngfsmount

# 2. Create test data
mkdir -p test_data
echo "test content" > test_data/test.txt

# 3. Mount and test
ngfsmount --store test_data/store.dat --idx test_data/index.idx --at ./mnt --root test_cid

# 4. Browse and verify
ngfs-browse --mnt ./mnt
ngfs_ls_verify.py --mnt ./mnt --fixtures test_data/fixtures.json

# 5. Unmount
ngfsumount ./mnt

## NGFS Diff Tools

### Overview

The NGFS diff system provides tools for comparing snapshots, analyzing changes, and building timelines. It follows the polyglot discipline with Rust core logic, Go CLI, Python analysis, and TypeScript visualization.

### Core Tools

#### ngfs-diff (Go CLI)

The primary diff generation tool:

```bash
# Basic diff between two snapshots
ngfs-diff --old <cid1> --new <cid2> --out diff.cbor

# Verbose output with JSON summary
ngfs-diff --old <cid1> --new <cid2> --out diff.cbor --verbose --json

# Help and options
ngfs-diff --help
```

**Output Format**: One-line JSON summary
```json
{"test":"ngfs_diff","added":5,"removed":2,"modified":1,"success":true}
```

**Options**:
- `--old <cid>`: Old snapshot CID (required)
- `--new <cid>`: New snapshot CID (required)
- `--out <file>`: Output CBOR file (required)
- `--verbose`: Show detailed progress
- `--json`: Output JSON summary to stdout
- `--help`: Show usage information

#### ngfs_diff_stats.py (Python Analyzer)

Comprehensive diff analysis and timeline building:

```bash
# Single diff analysis
ngfs_diff_stats.py --diff diff.cbor

# Multiple diff analysis
ngfs_diff_stats.py --diff diff1.cbor --diff diff2.cbor --diff diff3.cbor

# Timeline building
ngfs_diff_stats.py --timeline

# Export to different formats
ngfs_diff_stats.py --diff diff.cbor --merge merged.cbor --format cbor
ngfs_diff_stats.py --diff diff.cbor --merge merged.json --format json
ngfs_diff_stats.py --diff diff.cbor --export timeline.csv
```

**Features**:
- **Statistics**: File counts, byte deltas, type analysis
- **Timeline**: Chronological change tracking
- **Merging**: Combine multiple diffs
- **Export**: CBOR, JSON, CSV formats
- **Validation**: Structure and consistency checking

#### ngfs-diff-view (TypeScript Viewer)

Interactive diff visualization with colored tree output:

```bash
# Basic viewing
ngfs-diff-view diff.cbor

# Custom options
ngfs-diff-view --max-depth 3 --compact --no-colors diff.cbor

# Help
ngfs-diff-view --help
```

**Display Features**:
- **Color Coding**: Green (+) added, red (-) removed, yellow (~) modified
- **Tree Structure**: Hierarchical path display
- **Depth Control**: Configurable nesting levels
- **Compact Mode**: Condensed output for large diffs
- **Color Control**: Optional ANSI color codes

### Diff Workflow

#### 1. Generate Diff

```bash
# Create diff between snapshots
ngfs-diff --old $(cat snap1.cid) --new $(cat snap2.cid) --out changes.cbor

# Verify output
ls -la changes.cbor
```

#### 2. Analyze Changes

```bash
# Get detailed statistics
ngfs_diff_stats.py --diff changes.cbor

# Output includes:
# - File counts by type
# - Byte deltas
# - Path analysis
# - Summary table
```

#### 3. Visualize Changes

```bash
# View colored tree
ngfs-diff-view changes.cbor

# Customize display
ngfs-diff-view --max-depth 2 --compact changes.cbor
```

#### 4. Build Timeline

```bash
# Load multiple diffs
ngfs_diff_stats.py --diff diff1.cbor --diff diff2.cbor --diff diff3.cbor

# Build chronological timeline
ngfs_diff_stats.py --timeline

# Export for external analysis
ngfs_diff_stats.py --export timeline.csv
```

### Advanced Usage

#### Batch Processing

```bash
# Process multiple snapshot pairs
for i in {1..10}; do
  ngfs-diff --old snap${i}.cid --new snap$((i+1)).cid --out diff${i}.cbor
done

# Analyze all diffs
ngfs_diff_stats.py --diff diff*.cbor --timeline
```

#### Integration with Other Tools

```bash
# Generate diff and mount for inspection
ngfs-diff --old snap1.cid --new snap2.cid --out changes.cbor
ngfsmount --store store.dat --idx index.idx --at ./mnt --root $(cat snap2.cid)

# Browse changes
ngfs-browse --mnt ./mnt
ngfsumount ./mnt
```

#### Custom Analysis Scripts

```python
#!/usr/bin/env python3
from ngfs_diff_stats import NgfsDiffAnalyzer

# Load and analyze diff
analyzer = NgfsDiffAnalyzer()
analyzer.load_diff('changes.cbor')
stats = analyzer.analyze_diff(analyzer.diffs[0])

# Custom processing
if stats.net_bytes_change > 1024 * 1024:  # >1MB
    print("Large change detected!")
    print(f"Net change: {analyzer.format_bytes(stats.net_bytes_change)}")

# Export custom format
analyzer.export_csv('custom_analysis.csv')
```

### Testing

```bash
# Run all diff tests
bazel test //tests/ngfs:diff_smoke_test
bazel test //tooling/python:tests/test_diff_stats
bazel test //tooling/ts:tests/test_diff_view

# Run specific test suites
bazel test //tests/ngfs:diff_smoke_test
bazel test //tooling/python:tests/test_diff_stats
bazel test //tooling/ts:tests/test_diff_view
```

### CI Integration

The diff tools integrate with CI through the `ngfs-diff` job:

```yaml
# .github/workflows/phase-3-gates.yml
ngfs-diff:
  name: NGFS Snapshot Diff
  needs: [ngfs-schema, ngfs-features, ngfs-syscalls, ngfs-integrity, ngfs-fuse]
  steps:
    - Build diff targets (Rust, Go, Python, TypeScript)
    - Run unit tests for all components
    - Generate test diffs from fixture snapshots
    - Validate against golden CBOR files
    - Run analyzers and viewers
    - Upload results as artifacts
```

### Troubleshooting

#### Common Issues

**Diff Generation Fails**
```bash
# Check CID format
echo "CID format: $(cat snap1.cid | wc -c) bytes"

# Verify snapshots exist
ls -la snap1.cid snap2.cbor

# Check permissions
ls -la store.dat index.idx
```

**Analysis Errors**
```bash
# Validate CBOR file
file diff.cbor
hexdump -C diff.cbor | head -20

# Check Python dependencies
pip list | grep cbor2
```

**Viewer Issues**
```bash
# Test with simple diff
echo '{"version":1,"timestamp":"2024-01-01T00:00:00Z"}' > test.json
ngfs-diff-view test.json

# Check Node.js version
node --version
npm list -g typescript
```

#### Performance Issues

**Large Diffs**
```bash
# Use compact mode for large diffs
ngfs-diff-view --compact --max-depth 2 large_diff.cbor

# Limit analysis scope
ngfs_diff_stats.py --diff large_diff.cbor --max-entries 10000
```

**Memory Usage**
```bash
# Monitor memory during processing
/usr/bin/time -v ngfs_diff_stats.py --diff large_diff.cbor

# Use streaming for very large diffs
ngfs_diff_stats.py --diff large_diff.cbor --stream
```
```

### Testing

```bash
# Run all tests
bazel test //tests/fuse:all //tooling/python:tests/test_ls_verify //tooling/ts:tests/test_browse

# Run specific test suites
bazel test //tests/fuse:fake_mount_smoke_test
bazel test //tooling/python:tests/test_ls_verify
bazel test //tooling/ts:tests/test_browse

# Run with coverage
bazel coverage //tests/fuse:all
```

### CI Integration

The tools integrate with CI through the `ngfs-fuse` job:

```yaml
# .github/workflows/phase-3-gates.yml
ngfs-fuse:
  name: NGFS FUSE Mount
  needs: [ngfs-schema, ngfs-features, ngfs-syscalls, ngfs-integrity]
  steps:
    - name: Build FUSE targets
      run: bazel build //c/fuse:aeth_fuse //services/ngfs:fuse
    - name: Run fake FUSE tests
      run: ./bazel-bin/services/ngfs/fuse --fake-fuse --root <fixtureCID> --at /tmp/mnt
    - name: Verify FUSE output
      run: # Check for mount/stats events and performance thresholds
```

## Troubleshooting

### Common Issues

#### Mount Failures
```bash
# Check if FUSE is available
lsmod | grep fuse  # Linux
kextstat | grep macfuse  # macOS

# Check mount point permissions
ls -la ./mnt

# Check for existing mounts
mount | grep fuse
```

#### Permission Errors
```bash
# Add user to fuse group (Linux)
sudo usermod -a -G fuse $USER

# Check FUSE device permissions
ls -la /dev/fuse

# Restart user session after group changes
```

#### Performance Issues
```bash
# Check FUSE statistics
cat /proc/fs/fuse/dev  # Linux

# Monitor system resources
htop
iostat -x 1

# Check for I/O bottlenecks
iotop
```

### Debug Mode

Enable verbose logging for debugging:

```bash
# FUSE server debug
ngfs-fuse --store ngfs.dat --idx ngfs.idx --at ./mnt --root <CID> --verbose

# Browser debug
ngfs-browse --mnt ./mnt --verbose --debug

# Verification debug
ngfs_ls_verify.py --mnt ./mnt --fixtures ./fixtures --verbose --debug
```

### Log Analysis

The tools emit structured JSON logs for analysis:

```json
{"event":"mount","path":"./mnt","root":"<CID>","fake":false}
{"event":"stats","opens":42,"reads":128,"read_p95_us":1200}
{"event":"error","path":"/nonexistent","error":"ENOENT"}
```

## Advanced Usage

### Custom FUSE Options

```bash
# Custom FUSE mount options
ngfs-fuse --store ngfs.dat --idx ngfs.idx --at ./mnt --root <CID> \
  --fuse-opt allow_other \
  --fuse-opt default_permissions \
  --fuse-opt max_read=65536
```

### Integration with Other Tools

```bash
# Use with standard Unix tools
find ./mnt -type f -exec sha256sum {} \;
du -sh ./mnt
tree ./mnt

# Integration with development tools
ngfs-browse --mnt ./mnt --json | jq '.entries[] | select(.type == "file")'
ngfs_ls_verify.py --mnt ./mnt --fixtures ./fixtures --json | jq '.success'
```

### Performance Tuning

```bash
# Adjust cache sizes
export NGFS_INODE_CACHE_SIZE=16384
export NGFS_READ_WINDOW_SIZE=131072

# Monitor performance
ngfs-fuse --store ngfs.dat --idx ngfs.idx --at ./mnt --root <CID> --stats-interval 5
```

## Security Considerations

### Development vs. Production

- **Development**: Uses `--keydir` for local key fixtures
- **Production**: Integrates with KeyVault for key management
- **CI**: Uses `--fake-fuse` mode for testing without kernel mounts

### Access Control

- **Read-Only**: No write operations permitted
- **Path Validation**: Strict filename and path constraints
- **No Network**: All operations are local filesystem only

### Audit and Logging

- **Operation Logging**: All mount/unmount operations logged
- **Error Tracking**: Failed operations logged with context
- **Performance Metrics**: Read/write statistics collected

## Future Enhancements

### Planned Features

- **Write Support**: Append-only file modifications
- **Compression**: LZ4/Zstandard integration
- **Deduplication**: Content-based deduplication
- **Replication**: Multi-node synchronization

### Integration Points

- **Kubernetes**: Volume plugin support
- **Docker**: Layer format compatibility
- **Git**: Tree/hash format support
- **IPFS**: Full multihash compatibility

## Support and Resources

### Documentation

- **API Reference**: `docs/phase-3/NGFS-V1.md`
- **Schema Definitions**: `services/ngfs/src/schema.rs`
- **Test Examples**: `tests/fuse/` and `tests/ngfs/`

### Community

- **Issues**: GitHub issue tracker
- **Discussions**: GitHub discussions
- **Contributing**: CONTRIBUTING.md

### Development Setup

```bash
# Clone repository
git clone https://github.com/polymera-os/polymera-os.git
cd polymera-os

# Setup development environment
bazel build //c/fuse:aeth_fuse //services/ngfs:fuse //go/tools:ngfsmount

# Run tests
bazel test //tests/fuse:all

# Build documentation
bazel build //docs:all
```

## Conclusion

The NGFS development tools provide a comprehensive suite for working with NGFS filesystems during development. The FUSE mount system enables convenient host-side access, while the verification tools ensure consistency and policy compliance.

For production use, these tools integrate with the full NGFS service layer, providing enterprise-grade security and performance features.
