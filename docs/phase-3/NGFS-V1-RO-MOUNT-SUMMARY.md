# P3-01-A5: NGFS v1 — Snapshot Builder & RO Mount Plumbing

## Overview

This document summarizes the implementation of **P3-01-A5: NGFS v1 — Snapshot Builder & RO Mount Plumbing**, which completes the end-to-end read-only filesystem functionality for NGFS v1.

## Implementation Summary

### 1. Mount Registry (`services/ngfs/src/mount.rs`)

**Core Components:**
- `MountTable`: In-memory registry of mounted filesystems
- `Mount`: Individual mount with root CID, salt, and virtual clock
- `MountError`: Comprehensive error handling for mount operations

**Key Features:**
- Path validation: Only `/ro/*` paths allowed, no traversal
- Overlap detection: Prevents nested mount conflicts
- Policy enforcement: Absolute paths, component length limits
- Unicode support: NFC normalization, control character rejection

**Mount Policy:**
```rust
// Valid mount points
/ro/test
/ro/demo
/ro/nested/path

// Invalid mount points  
/tmp/test          // Not under /ro/
/ro/../test        // Path traversal
/ro/test\x00       // Control characters
```

### 2. Path Resolver (`services/ngfs/src/resolve.rs`)

**Core Components:**
- `PathResolver`: Handles path traversal over manifests
- `NodeRef`: Represents resolved filesystem nodes
- `ResolveError`: Path resolution error types

**Path Resolution:**
- Absolute path validation
- Component normalization (NFC, length limits)
- Manifest loading and validation
- Recursive directory traversal
- Error handling for missing/invalid nodes

**Node Types:**
```rust
pub enum NodeRef {
    File { cid: Cid, size: u64 },
    Dir { cid: Cid },
    Symlink { target: String },
}
```

### 3. File Reader (`services/ngfs/src/read.rs`)

**Core Components:**
- `FileReader`: Handles file reading and chunk decryption
- `ReadError`: File operation error types

**Reading Operations:**
- Offset/len support for partial reads
- Chunk-based reading with manifest traversal
- Automatic decryption via envelope encryption
- Associated data validation
- EOF handling and truncation

**Read API:**
```rust
impl FileReader {
    pub fn read_file(&self, file_cid: &Cid, offset: u64, len: usize) -> Result<Vec<u8>, ReadError>
    pub fn read_file_range(&self, file_cid: &Cid, start: u64, end: u64) -> Result<Vec<u8>, ReadError>
    pub fn get_file_info(&self, file_cid: &Cid) -> Result<(u64, Vec<(Cid, u32)>), ReadError>
}
```

### 4. Snapshot Builder (`services/ngfs/src/snapshot.rs`)

**Core Components:**
- `SnapshotBuilder`: Creates and verifies filesystem snapshots
- `SnapshotError`: Snapshot operation error types

**Snapshot Features:**
- Deterministic ID generation via Blake3 hash
- Virtual clock timestamp for ordering
- DID-based signer identification
- CBOR serialization for cross-language compatibility

**Snapshot Schema:**
```rust
pub struct SnapshotV1 {
    pub snap_id: String,           // Blake3 hash of snapshot
    pub root_cid: Cid,             // Root directory CID
    pub created_vclock: u64,       // Creation timestamp
    pub signer_did: String,        // Signer DID
}
```

**ID Generation:**
```
snap_id = blake3(cbor(SnapshotV1 { snap_id: "", ... }))
```

### 5. NGFS Service Integration (`services/ngfs/src/lib.rs`)

**Service Methods:**
```rust
impl NgfsService {
    pub fn mount_ro(&mut self, mount_point: &str, options: MountOptionsV1) -> Result<(), MountError>
    pub fn unmount(&mut self, mount_point: &str) -> Option<Mount>
    pub fn list_mounts(&self) -> Vec<(&String, &Mount)>
    pub fn resolve_path(&self, mount_point: &str, path: &str) -> Result<NodeRef, ResolveError>
    pub fn read_file(&self, file_cid: &Cid, offset: u64, len: usize) -> Result<Vec<u8>, ReadError>
    pub fn create_snapshot(&self, mount_point: &str, signer_did: &str) -> Result<SnapshotV1, SnapshotError>
    pub fn verify_snapshot(&self, snapshot: &SnapshotV1) -> Result<bool, SnapshotError>
}
```

## Polyglot Tools

### 1. Go CLI (`go/tools/ngfsctl/main.go`)

**Commands:**
- `mount`: Mount filesystem with root CID
- `snapshot`: Create snapshot of mounted filesystem
- `stat`: Get file/directory information
- `read`: Read file data with offset/len support

**Features:**
- Path validation and error handling
- JSON metrics output for CI parsing
- Performance measurement and reporting
- Comprehensive error reporting

### 2. Python Verification (`tooling/python/ngfs_read_verify.py`)

**Capabilities:**
- Tree walking and verification
- Golden data comparison
- Performance metrics collection
- CI integration with JSON output

**Usage:**
```bash
python3 ngfs_read_verify.py --root /ro/demo --fixtures ./fixtures --json
```

### 3. TypeScript Path Checker (`tooling/ts/path_check.ts`)

**Features:**
- Path policy validation
- CBOR manifest parsing
- Directory listing display
- Unicode normalization checks

**Commands:**
```bash
# Validate paths
node path_check.js validate /ro/test /ro/demo/file.txt

# Parse manifests
node path_check.js parse ./dir_manifest.cbor
```

## Testing

### Test Coverage

**Mount and Stat Tests (`tests/ngfs/mount_and_stat.rs`):**
- Valid/invalid mount point validation
- Overlap detection and prevention
- Mount/unmount operations
- Path resolution integration

**Read Roundtrip Tests (`tests/ngfs/read_roundtrip.rs`):**
- File reading with offset/len
- Chunk boundary handling
- EOF truncation
- Error path validation

**Snapshot Builder Tests (`tests/ngfs/snapshot_build.rs`):**
- Snapshot creation and verification
- ID determinism and stability
- Virtual clock handling
- Signer DID validation

**Error Path Tests (`tests/ngfs/error_paths.rs`):**
- Invalid path handling
- Control character rejection
- Unicode edge cases
- Concurrent operation handling

**Performance Tests (`tests/ngfs/perf_ro.rs`):**
- Batch mount operations
- Snapshot creation performance
- Read operation latency
- Stress testing and metrics

### CI Integration

**Test Execution:**
```bash
# Run all NGFS tests
bazel test //tests/ngfs:all

# Run specific test suites
bazel test //tests/ngfs:ngfs_mount_and_stat_test
bazel test //tests/ngfs:ngfs_read_roundtrip_test
bazel test //tests/ngfs:ngfs_snapshot_build_test
bazel test //tests/ngfs:ngfs_error_paths_test
bazel test //tests/ngfs:ngfs_perf_ro_test
```

**Performance Metrics:**
```json
{"test":"ngfs_ro","reads":200,"read_p95_us":1200,"snap_ms":45,"mounts":100,"errors":0}
```

**CI Thresholds:**
- `read_p95_us` ≤ 1500 (QEMU)
- `snap_ms` ≤ 50
- `errors` == 0 in nominal runs

## Build System

### Dependencies

**Rust Dependencies:**
```toml
[dependencies]
bs58 = "0.5"           # Base58 encoding for CIDs
tempfile = "3.8"       # Temporary file handling for tests
```

**Bazel Targets:**
```python
rust_library(
    name = "ngfs",
    srcs = [
        "schema.rs",
        "src/cid.rs",
        "src/cas.rs", 
        "src/enc.rs",
        "src/manifest.rs",
        "src/mount.rs",        # New
        "src/resolve.rs",      # New
        "src/read.rs",         # New
        "src/snapshot.rs",     # New
        "src/lib.rs",
    ],
    deps = [
        "@crate_index//:bs58",  # New
        # ... existing deps
    ],
)
```

### Tool Builds

**Go Tool:**
```python
go_binary(
    name = "ngfsctl",
    srcs = ["main.go"],
    importpath = "github.com/polymera-os/polymera-os/go/tools/ngfsctl",
)
```

**Python Tool:**
```python
py_binary(
    name = "ngfs_read_verify",
    srcs = ["ngfs_read_verify.py"],
)
```

**TypeScript Tool:**
```python
ts_project(
    name = "ngfs_path_check_ts",
    srcs = ["path_check.ts"],
)

js_binary(
    name = "ngfs_path_check",
    data = [":ngfs_path_check_ts"],
    entry_point = "path_check.js",
)
```

## Security Considerations

### Path Validation
- Absolute path enforcement
- No path traversal (`..`, `.`)
- Component length limits (255 bytes)
- Control character rejection
- Unicode normalization (NFC)

### Mount Isolation
- No overlapping mount points
- Per-mount salt for nonce generation
- Virtual clock isolation
- Capability-based access control

### Read-Only Enforcement
- No write operations in v1
- Manifest-based access control
- Chunk-level encryption
- Associated data validation

## Performance Characteristics

### Targets
- **Mount operations**: < 50ms for 100 mounts
- **Snapshot creation**: < 100ms for 50 snapshots
- **Read operations**: p95 < 1500μs in QEMU
- **Memory usage**: < 8 MiB per mount
- **Storage overhead**: < 5% for metadata

### Optimizations
- In-memory mount table with O(1) lookups
- Lazy manifest loading
- Chunk-based reading with minimal copying
- Deterministic CBOR serialization
- Efficient path component validation

## Future Enhancements

### Phase 3+ Features
- Write support with manifest updates
- Dynamic mount point management
- Advanced snapshot policies
- Cross-mount path resolution
- Performance monitoring and alerting

### Integration Opportunities
- Policy Guardrail integration
- Event Fabric topic publishing
- Skill Runtime filesystem access
- LLM Adapter file operations
- World Model filesystem facts

## Conclusion

P3-01-A5 successfully implements the complete RO mount plumbing for NGFS v1, providing:

1. **End-to-end filesystem functionality** from mount to read
2. **Deterministic snapshot creation** with DID signatures
3. **Comprehensive path resolution** with policy enforcement
4. **Polyglot tooling** for administration and validation
5. **Performance testing** with CI integration
6. **Security hardening** through path validation and isolation

The implementation follows Polymera OS principles of determinism, capability-based security, and polyglot tooling, providing a solid foundation for Phase 3 storage requirements.
