# Polymera OS: Code Improvement Plan (Phase 1-5)

## Executive Summary
This document outlines the roadmap for elevating the Polymera OS codebase from "Prototype" to "Production Grade". The focus is on eliminating safety violations, implementing robust error handling, and adopting architectural patterns from reference operating systems (Redox, Fuchsia).

## 1. Critical Flaws Identified

### Safety Violations
- **Unchecked Unwraps**: Widespread use of `unwrap()` and `expect()` without recovery logic.
- **Lock Panics**: Mutex locks are unwrapped, which can poison the system if a thread panics.
- **Hardcoded Values**: Magic numbers and paths (e.g., `/tmp`) embedded in code.

### Architecture Smells
- **Blocking I/O**: Usage of `std::fs` in async contexts (though `tokio::fs` is present, consistency is key).
- **Monolithic Functions**: Some logic is tightly coupled and needs better separation.
- **Missing Abstractions**: Direct filesystem access instead of a VFS or Capability-based approach.


## 2. Detailed Improvement Roadmap

### Component: AI Core Service (`services/ai_core`)

| File | Flaw | Reference OS Pattern | Recommended Fix | Status |
|------|------|----------------------|-----------------|--------|
| `src/orchestrator.rs` | Basic error logging, potential panic points. | **Fuchsia Component Framework**: Lifecycle management. | Implement proper lifecycle states, graceful shutdown, and structured error envelopes. | **DONE** |
| `src/router.rs` | `SystemTime` unwrap, potential blocking. | **Mycroft Intent Parser**: Robust intent routing. | Remove panics, use `chrono` safely, ensure all I/O is non-blocking. | **DONE** |
| `src/tools/tts.rs` | `unwrap()` on configuration, `std::fs` usage. | **Redox Schemes**: VFS abstraction. | Replace `std::fs` with `tokio::fs`, return `Result` for all config lookups. | **DONE** |
| `src/tools/stt.rs` | `unwrap()` on models, `std::fs` usage. | **Redox Schemes**: VFS abstraction. | Replace `std::fs` with `tokio::fs`, handle model loading errors gracefully. | **DONE** |

### Component: Policy Engine (`crates/policy_engine`)

| File | Flaw | Reference OS Pattern | Recommended Fix | Status |
|------|------|----------------------|-----------------|--------|
| `src/cap.rs` | `unwrap()` on Mutex locks, `unwrap()` on `NonZeroUsize`. | **Fuchsia Capabilities**: Handle-based access. | Use `map_err` for lock acquisition, handle cache initialization errors, return `SystemError`. | **DONE** |
| `src/audit.rs` | `unwrap()` on SystemTime. | **Fuchsia Inspect**: Structured diagnostics. | Use safe time handling, structured audit logs. | **DONE** |

## 3. Top 3 Critical Refactors (Immediate Action)

1.  **[DONE] AI Orchestrator (`orchestrator.rs`)**: The central nervous system. Needs to be bulletproof.
2.  **[DONE] Capability Verifier (`cap.rs`)**: The security gatekeeper. Cannot panic or fail insecurely.
3.  **[DONE] Intent Router (`router.rs`)**: The brain's decision maker. Needs deterministic and safe execution.

## 4. Reference Benchmarks

- **Redox**: Used for Scheme/VFS patterns to replace direct `std::fs` usage.
- **Fuchsia**: Used for Capability patterns and Component lifecycle management.
- **Mycroft**: Used for Intent parsing and fallback strategies.

## 5. Phase 5: Global Expansion (Go, C, C++)

The "Grand Refactor" has been expanded to include the entire codebase.

### Critical Flaws Identified

| File | Language | Flaw Type | Description | Status |
| :--- | :--- | :--- | :--- | :--- |
| `go/aicore.go` | Go | Architecture Smell | `cmdSubmit` uses a fixed 64KB buffer for C output. If the plan exceeds this, it may truncate or crash. | **DONE** |
| `go/aicore.go` | Go | Architecture Smell | `os.Exit` used extensively in `main`, making testing difficult. | **DONE** |
| `c/ngfs/ngfs_merkle.c` | C | Hollow Logic | `aeth_ngfs_dir_entry_count` and `aeth_ngfs_file_chunk_count` silently return 0 and `NGFS_OK` without implementation. | **DONE** |
| `c/libc_aetheris/ai_orchestrator.c` | C | Hollow Logic | Entire file is a stub implementation ("Stub: Initialize orchestrator"). | **DONE** |

### Recommended Fixes

#### [DONE] `go/aicore.go`
- **Fix**: Refactor `cmdSubmit` to use a dynamic buffer or handle truncation gracefully.
- **Fix**: Replace `os.Exit` with returning errors from `main` logic functions.

#### [DONE] `c/ngfs/ngfs_merkle.c`
- **Fix**: Change hollow functions to return `NGFS_ERROR_NOT_IMPLEMENTED` instead of silently succeeding.
- **Fix**: Implement basic CBOR validation for entry counting if possible.

#### [DONE] `c/libc_aetheris/ai_orchestrator.c`
- **Fix**: Add `TODO` comments to all stub functions indicating they need to be linked to the Rust backend.

## 6. Next Steps

1.  **Verify Compilation**: Wait for `cargo check` to complete.
2.  **Full Test Suite**: Run `cargo test` across the workspace.
