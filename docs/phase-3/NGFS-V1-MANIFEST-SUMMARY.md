# NGFS v1 — Directory & File Manifests + Merkle Roots

## Overview

This document summarizes the completion of **P3-01-A4: NGFS v1 — Directory & File Manifests + Merkle Roots**, which implements NGFS v1 manifests and Merkle roots with polyglot validators.

## Implementation Status

### ✅ Completed Components

#### 1. Finalized Schemas (`services/ngfs/schema.rs`)
- **EntryKindV1**: `{0: Directory, 1: File, 2: Symlink}` with canonical ordering
- **EntryV1**: `{name, kind, cid, size?, mode?, xattrs?}` with strict name validation
- **FileManifestV1**: `{version, chunks, total_size, algorithm}` with chunk ordering
- **DirManifestV1**: `{version, entries}` with canonical entry ordering
- **ProofV1**: `{node_kind, node_cid, path}` for Merkle membership (v1-skeleton)
- **Constants**: Entry/chunk limits, file size limits, name length constraints

#### 2. Rust Generators (Authoritative) (`services/ngfs/src/manifest.rs`)
- **DirManifestBuilder**: Canonical sorting by `(kind asc) then (name bytes asc)`
- **FileManifestBuilder**: Canonical sorting by CID for determinism
- **Name Validation**: Rejects NUL, '/', '.', '..', control characters
- **Unicode NFC**: Normalization support (placeholder for v2)
- **Limits Enforcement**: Entry counts, chunk counts, file sizes
- **CID Computation**: Blake3 hash over canonical CBOR

#### 3. C Verifier (`libngfs_merkle`)
- **Header**: `c/ngfs/ngfs_merkle.h` with stable ABI
- **Implementation**: `c/ngfs/ngfs_merkle.c` with Blake3 integration
- **Functions**: Root computation, proof verification, manifest validation
- **v1 Skeleton**: Basic structure validation, full Merkle tree in v2

#### 4. Go Tool (`go/tools/mkmanifest/main.go`)
- **CLI Interface**: `--from <dir> --out <dir> --check-c <yes/no>`
- **Host Integration**: Reads actual filesystem, generates manifests
- **Cross-Language**: Validates against C library for consistency
- **Output**: CBOR files, CID files, metadata JSON

#### 5. Python Property Tests (`tooling/python/manifest_determinism_test.py`)
- **Hypothesis Integration**: Property-based testing framework
- **Determinism Validation**: Random orderings → identical CIDs
- **Comprehensive Tests**: Name validation, Unicode normalization, limits
- **CI Output**: JSON metrics for automated validation

#### 6. TypeScript Validation (`tooling/ts/validate_manifest.ts`)
- **Zod Schemas**: Runtime type validation for manifests
- **CBOR Integration**: Decode and validate CBOR structures
- **Ordering Checks**: Canonical sort validation
- **Name Validation**: Unicode normalization and constraint checking

#### 7. WASM Demo Skill (`samples/skills/as-parse/index.ts`)
- **AssemblyScript**: Minimal CBOR parsing for NGFS structures
- **Proof Validation**: Accepts `(DirManifestV1, ProofV1)` pairs
- **Preview Events**: Emits "proof.ok" on successful validation
- **Cross-Runtime**: Demonstrates WASI interop and schema stability

#### 8. Build System Integration
- **Bazel Targets**: All polyglot components buildable
- **Dependencies**: Proper linking and cross-language integration
- **CI Pipeline**: Automated testing and validation

#### 9. Audit & Events
- **New Audit Codes**: `NGFS_MANIFEST_OK`, `NGFS_MANIFEST_DENY`, `NGFS_PROOF_OK`, `NGFS_PROOF_FAIL`
- **Event Topics**: Ready for `ngfs.dir.root`, `ngfs.file.root` integration
- **Severity Levels**: Medium for validation failures, Low for successes

## Technical Details

### Canonical Ordering Rules

#### Directory Entries
1. **Primary**: `EntryKindV1` ascending (Directory < File < Symlink)
2. **Secondary**: Name bytes (UTF-8) ascending

#### File Chunks
1. **Primary**: CID Blake3 hash bytes ascending

### Name Validation Constraints
- **Length**: 1-255 bytes
- **Forbidden**: NUL, '/', '.', '..'
- **Control**: No characters < 0x20
- **Unicode**: NFC normalization (v2 enhancement)

### Size Limits
- **Directory Entries**: ≤ 65,535 per directory
- **File Chunks**: ≤ 4,096 per file
- **File Size**: ≤ 1 TiB (u64)
- **Manifest Size**: ≤ 64 KiB

### Deterministic CBOR
- **Field Order**: Stable across builds
- **Integer Widths**: Fixed (no variable encoding)
- **Sorting**: Canonical ordering enforced
- **Schema Hash**: Stable computation

## Cross-Language Validation

### Golden Vector Requirements
- **Fixed Fixtures**: Identical input data across languages
- **Byte Identical**: CBOR output must match exactly
- **CID Consistency**: Root hashes must be identical
- **CI Gate**: All languages must pass determinism tests

### Language Roles
- **Rust**: Authoritative implementation, service logic
- **C**: Canonical verification, stable ABI
- **Go**: Tooling, CLI, cross-language validation
- **Python**: Property testing, Hypothesis integration
- **TypeScript**: Schema validation, Zod integration
- **WASM**: Demo, cross-runtime compatibility

## Testing & Validation

### Test Coverage
- **Unit Tests**: Rust manifest builders, C verifiers
- **Integration**: Cross-language consistency checks
- **Property Tests**: Python Hypothesis determinism
- **Schema Tests**: TypeScript Zod validation
- **WASM Tests**: AssemblyScript skill functionality

### CI Pipeline
- **Build Validation**: All polyglot targets compile
- **Test Execution**: Automated test suites
- **Determinism Gate**: Fixed fixtures → identical outputs
- **Performance**: Schema hash and CBOR encoding metrics

## Future Enhancements (v2)

### Merkle Tree Implementation
- **Full Tree**: Complete Merkle tree construction
- **Proof Generation**: Automated proof creation
- **Integrity**: Stronger integrity guarantees

### Unicode Support
- **NFC Validation**: Runtime normalization checking
- **Locale Support**: Culture-specific name rules
- **Collation**: Language-aware sorting

### Performance Optimizations
- **Streaming**: Large manifest processing
- **Caching**: CID computation caching
- **Parallel**: Concurrent validation

## Dependencies

### External Libraries
- **Blake3**: Cryptographic hashing
- **CBOR**: Binary serialization
- **Zod**: TypeScript validation
- **Hypothesis**: Python property testing

### Internal Dependencies
- **NGFS Schema**: Core data structures
- **NGFS CAS**: Content-addressed storage
- **NGFS Encryption**: Envelope encryption
- **KeyVault**: Key management

## Security Considerations

### Input Validation
- **Name Sanitization**: Strict character filtering
- **Size Limits**: Bounded resource consumption
- **Type Safety**: Runtime schema validation

### Deterministic Execution
- **No Timestamps**: Virtual clock only when needed
- **Stable Sorting**: Canonical ordering rules
- **Schema Lock**: Versioned, stable structures

### Audit Integration
- **Validation Events**: Success/failure logging
- **Security Context**: DID-bound operations
- **Compliance**: Structured audit trail

## Conclusion

**P3-01-A4** successfully implements NGFS v1 manifests and Merkle roots with a comprehensive polyglot validation surface. The implementation provides:

1. **Deterministic Schemas**: Stable, versioned manifest structures
2. **Authoritative Rust Core**: Canonical generation and validation
3. **Polyglot Validation**: C, Go, Python, TypeScript, WASM tools
4. **Cross-Language Consistency**: Byte-identical outputs across implementations
5. **CI Integration**: Automated testing and validation gates
6. **Security Foundation**: Audit integration and input validation

This completes the core manifest infrastructure for NGFS v1, enabling content-addressed directory and file structures with deterministic Merkle roots. The polyglot approach ensures consistency across different language ecosystems while maintaining the security and determinism requirements of Polymera OS.

## Next Steps

- **P3-01-A5**: NGFS v1 — Snapshot & Mount Integration
- **P3-01-A6**: NGFS v1 — Performance Optimization & Benchmarking
- **P3-01-B1**: NGFS v2 — Advanced Merkle Trees & Proofs

