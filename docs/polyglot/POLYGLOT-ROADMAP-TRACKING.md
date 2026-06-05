# Polymera OS — Polyglot Architecture Roadmap Tracking

**Source of plan:** `Polymera OS  Complete Implementation Roadmap for True Polyglot Architecture.md`
(repo root).  
**Scope:** 8 initiatives over 12–18 months, three phases.  
**Last updated:** 2026-05-19.

This file is the live operational tracker for turning Polymera into an AI-native developer
OS with deliberate polyglot boundaries. The roadmap document is the long-form strategy;
`docs/polyglot/IPC-AUDIT.md` is the disk-truth audit when the roadmap and repo disagree.

## Status legend

- ⬜ Not started
- 🟦 Skeleton landed
- 🟨 In progress
- 🟩 Complete for current milestone
- ⚠️ Blocked / needs decision
- 🚫 Deferred external/toolchain work

## Phase 1 — Foundations

### Initiative 1: Language Charter & Governance — 🟨

| Artifact | Status | Path |
|---|---|---|
| Language charter | 🟩 landed | `LANGUAGES.md` |
| Deprecation process | 🟩 landed | `DEPRECATIONS.md` |
| CODEOWNERS template | 🟩 landed | `docs/polyglot/CODEOWNERS.template` |
| Active `.github/CODEOWNERS` | ⚠️ blocked | needs real GitHub users/teams |
| Raw service FFI allowlist check | 🟩 landed | `scripts/verify-polyglot.*` |

Decision: do not create an active CODEOWNERS file with invented owners. Promote the template
only after real DRIs are assigned.

### Initiative 2: Standardize IPC Contract — 🟨

| Artifact | Status | Path |
|---|---|---|
| IPC audit | 🟩 landed | `docs/polyglot/IPC-AUDIT.md` |
| WIT root | 🟩 landed | `wit/` |
| Filesystem WIT | 🟩 landed | `wit/polymera/fs/0.1.0/` |
| AI Core WIT facade | 🟩 landed | `wit/polymera/ai/0.1.0/world.wit` |
| Repo-pinned WIT tooling | 🟩 landed | `configs/polyglot-toolchain.toml`, `scripts/bootstrap-polyglot-tools.*` |
| Binding generation entrypoint | 🟨 local-tool aware | `scripts/gen-wit-bindings.sh` |

Decision: Protobuf-over-PolyBus remains canonical for service mesh IPC. WIT is canonical for
Component Model app/plugin/tool boundaries.

### Initiative 3: WASM Component Model App ABI — 🟨

| Artifact | Status | Path |
|---|---|---|
| Wasmtime Component Model dependency | 🟩 pinned | `services/wasm_driver/Cargo.toml` |
| Host runtime scaffold | 🟩 compile-verified | `services/wasm_driver/src/host.rs` |
| Host integration test | 🟩 passing | `services/wasm_driver/tests/host_integration.rs` |
| Hello component example | 🟩 compile-verified | `runtime/examples/hello_component/` |
| Repo-local component tooling bootstrap | 🟩 landed | `scripts/bootstrap-polyglot-tools.*` |
| Full `hello_wasm` migration | ⬜ not started | `runtime/examples/hello_wasm/` |

Tooling policy: `cargo-component`, `wasm-tools`, `wit-bindgen`, `jco`, and `componentize-py`
are installed under `.polymera-tools/`, never assumed globally.

## Phase 2 — Capabilities

### Initiative 4: Adopt Zig as Canonical C Replacement — 🟦

- Zig migration scaffold exists under `zig/`.
- Zig version is pinned in `configs/polyglot-toolchain.toml`.
- Zig is **not installed by default**; use `--include-roadmap-tools` only when actively working
  the C-to-Zig migration.

### Initiative 5: Formal Verification Layer — 🟦

- Verified-crypto scaffold exists under `c/crypto/verified/`.
- F* version is pinned in `configs/polyglot-toolchain.toml`.
- No HACL* upstream code or F* proofs are vendored yet.

### Initiative 6: Erlang-Style Supervision Trees — 🟩

| Artifact | Status | Path |
|---|---|---|
| Rust supervisor crate | 🟩 landed | `services/supervisor/` |
| Observable supervisor events | 🟩 landed | `SupervisorEvent` |
| Deterministic tests | 🟩 passing | `services/supervisor/tests/one_for_one.rs` |

Covered behavior: permanent restart, temporary non-restart, and restart-rate limit for
flapping children.

### Initiative 7: Python First-Class Scripting Boundary — 🟨

| Artifact | Status | Path |
|---|---|---|
| Trusted PyO3 runtime scaffold | 🟩 compile-verified | `services/python_runtime/` |
| Python SDK package | 🟩 landed | `bindings/python/polymera/` |
| Native AI Core transport | 🟦 explicit placeholder | `POLYMERA_AI_CORE_ENDPOINT` |
| Untrusted Python sandbox | ⬜ deferred | future Pyodide/WASM path |

Decision: native CPython is trusted only. Untrusted user scripts must wait for the Component
Model/Pyodide route.

## Phase 3 — Consolidation

### Initiative 8: Eliminate Duplicate Implementations — 🟨

| Artifact | Status | Path |
|---|---|---|
| Deprecation log | 🟩 landed | `DEPRECATIONS.md` |
| D-2 liboqs duplicate investigation | 🟩 resolved as dead directory | `DEPRECATIONS.md` |
| Code deletion | ⚠️ not done | requires domain-owner approval |
| Networking/filesystem role docs | ⬜ not started | D-4/D-5 follow-up |

No duplicate implementation was deleted in this pass.

## Current Implementation Pass

Landed:

- repo-local toolchain pins and bootstrap scripts;
- polyglot verification scripts and CI workflow;
- AI WIT facade over existing AI Core protobuf concepts;
- deterministic AI planner validation and guarded executor path;
- AI Core CLI `plan-smoke` flow;
- Wasmtime host integration test and hello component compile fix;
- supervisor event stream and deterministic restart tests;
- trusted Python runtime scaffold and Python SDK package;
- Go AI tooling `go.sum` repair so polyglot verification can run cleanly.

Verification run on Windows:

- `cargo check --manifest-path services/ai_core/Cargo.toml`
- `cargo check --manifest-path services/wasm_driver/Cargo.toml`
- `cargo check --manifest-path services/supervisor/Cargo.toml`
- `cargo check --manifest-path runtime/examples/hello_component/Cargo.toml`
- `cargo check --manifest-path services/python_runtime/Cargo.toml`
- `cargo test --manifest-path services/supervisor/Cargo.toml`
- `cargo test --manifest-path services/wasm_driver/Cargo.toml --test host_integration`
- `cargo test --manifest-path services/ai_core/Cargo.toml --lib agent::`
- `cargo test --manifest-path services/ai_core/Cargo.toml --lib replay`
- `cargo install wasm-tools --version 1.249.0 --locked --root .polymera-tools`
- `powershell -ExecutionPolicy Bypass -File scripts/verify-polyglot.ps1`

Known verification gaps:

- TypeScript type-checks were skipped locally because package-local `node_modules/` directories
  are not installed; CI installs them before running `scripts/verify-polyglot.sh`.
- Bash verification could not run on this Windows host because WSL failed to attach its disk.

## Next Best Work

1. Run `scripts/bootstrap-polyglot-tools.ps1` and rerun `scripts/verify-polyglot.ps1` without
   `-SkipExternalTools`.
2. Build a real `hello_component.wasm` through repo-local `cargo-component` and extend the host
   integration test to call its exported `greet` function.
3. Activate real AI Core IPC transport for `services/python_runtime`.
4. Add per-service role docs for D-4/D-5 before deleting any duplicate implementations.
