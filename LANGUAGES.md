# Polymera OS Language Charter

**Status:** v0.1 — initial commit. Subject to review before Phase 1 milestone (see
`docs/polyglot/POLYGLOT-ROADMAP-TRACKING.md`).

This document is the single source of truth for **which language is allowed for which layer**
of Polymera OS, **why**, and **how cross-language boundaries are crossed**.

If a new feature needs a language not listed here, *change this document first*, then write
the code. "Drift first, document later" is how we ended up with three networking stacks (see
`DEPRECATIONS.md`).

---

## 1. The four rules

1. **Each layer has exactly one canonical language.** Bindings/shims in other languages are fine;
   parallel reimplementations are not.
2. **Cross-language calls go through a contract**, not raw FFI. Contracts are either WIT
   (Component Model boundary) or Protobuf-over-PolyBus (service mesh). Raw `extern "C"` is allowed
   only where the ABI is forced on us by the platform (kernel entry, FUSE, `-sys` crates for
   external C libraries like liboqs).
3. **New service code MAY NOT introduce raw FFI** after Phase 1 milestone 1 lands.
   Audit it through code review against `docs/polyglot/IPC-AUDIT.md`.
4. **A language gets added to this charter only if it owns something** no existing language is
   already good at. "I like writing X" is not a reason.

---

## 2. Canonical assignments

### 2.1 Rust — microkernel, core services, drivers

- **Owns:** `kernel/`, `services/*` (except those marked otherwise below), `drivers/`,
  `crates/`, `rust/`, most of `crypto/`, `userland-stubs/`.
- **Why:** memory safety without GC; zero-cost abstractions; no runtime to bring up before
  hardware-init; Polymera's existing largest investment is here.
- **Must NOT be used for:** GUI compositor hot path (use C++), high-level network protocols
  better served by Go's stdlib (use Go), AI/ML model glue (use Python), throwaway scripts
  (use Python).
- **FFI gateways:**
  - Rust → C external libs: via `-sys` crate (e.g. `rust/bindings/polycrypto-sys`,
    `crypto/.../liboqs/bindings.rs`).
  - Rust → other Polymera services: Protobuf-over-PolyBus (current) or WIT (target).
  - Rust → C inside Polymera: **discouraged** — prefer rewriting in Rust or wrapping behind WIT.

### 2.2 C — libc, hardware ABI floor, vendored C primitives

- **Owns:** `c/libc_aetheris/`, `c/fuse/` (FUSE forces C ABI), `c/time/`,
  `c/crypto/` *only* where it wraps verified upstream code (HACL\*, liboqs).
- **Why:** universal ABI; required by FUSE, kernel entry, and many crypto libraries.
- **Must NOT be used for:** new business logic, new services, new network protocols. Existing
  C code in `c/aicore.c`, `c/ngfs/`, `c/zkvm/`, `c/ipfs/`, `c/wallet/` is **legacy** —
  see `DEPRECATIONS.md` for migration targets.
- **FFI gateway:** plain C ABI consumed by Rust `-sys` crates or by Zig (Phase 2).
- **Phase 2 plan:** Zig replaces this column for new code (see § 3.1).

### 2.3 Go — network-heavy services, devtools, CLIs

- **Owns:** `services/net_go/`, `services/fs_go/` (network-FS only — not local FS),
  `services/posix/` *iff* it remains a POSIX-shim service (audit-pending — see
  `DEPRECATIONS.md`), `go/tooling/`, `go/tools/`, `go/cli/`.
- **Why:** stdlib's net stack is mature; fast iteration; great for ops/devtools/CLI;
  garbage collection is acceptable above the kernel line.
- **Must NOT be used for:** kernel-adjacent code, hard-real-time paths, cryptography
  (use Rust + HACL\*), AI/ML model glue (use Python).
- **FFI gateway:** Protobuf-over-PolyBus (current) or WIT via TinyGo + wit-bindgen-go (target).

### 2.4 C++ — graphics compositor only

- **Owns:** `services/window_server_cpp/` (CMake-built).
- **Why:** Vulkan/Skia ecosystem maturity; existing window-server vendoring.
- **Must NOT be used for:** anything else. No new C++ services. No C++ in `services/` that
  isn't part of the compositor.
- **FFI gateway:** Protobuf-over-PolyBus toward the rest of the system; raw C ABI internally.

### 2.5 C# — .NET app runtime

- **Owns:** `services/app_runtime_cs/`.
- **Why:** hosts existing .NET applications targeting Polymera.
- **Must NOT be used for:** any service that isn't the .NET runtime itself.
- **FFI gateway:** the .NET app runtime exposes a WIT-compatible component interface (target —
  Initiative 3); internally uses managed code.

### 2.6 Python — AI/ML glue, automation scripting, policy DSL

- **Owns:** `tooling/python/`, `services/python_runtime/` (trusted runtime scaffold),
  `bindings/python/polymera/` (SDK).
  AI/ML pipeline glue, dev scripts, policy rule DSL.
- **Why:** ecosystem velocity; Polymera already depends on it via `pyproject.toml`.
- **Must NOT be used for:** anything in the kernel/service hot path, anything security-sensitive
  without sandboxing (use Pyodide-in-WASM for untrusted scripts per Initiative 7).
- **FFI gateway:** PyO3 inside `services/python_runtime` (trusted); Pyodide WASM component
  (untrusted user scripts).

### 2.7 TypeScript — UI shells, web/embedded views, browser-shaped clients

- **Owns:** `apps/assistant-ui/`, `apps/shell/`, `bindings/ts/`, `tooling/ts/`, `ui/`.
- **Why:** best language for UI; required for the assistant/shell UIs.
- **Must NOT be used for:** anything that isn't a UI or an API client for a UI. No TS services.
- **FFI gateway:** Protobuf-over-PolyBus today; WASM component via jco (target — Initiative 3).

### 2.8 WASM (any source language) — sandboxed user apps & plugins

- **Owns:** `services/wasm_driver/` (Component Model host), `runtime/examples/hello_wasm/`,
  `runtime/examples/hello_component/`,
  future user-app distribution format.
- **Why:** universal portable binary; capability-based sandboxing; the Component Model lets
  apps in any source language interop through one ABI (Initiative 3).
- **Must NOT be used for:** kernel code, drivers, anything that needs raw hardware access.
- **FFI gateway:** WIT-defined interfaces only. No host-imports outside WIT.

---

## 3. Languages on the roadmap (not yet adopted)

### 3.1 Zig — Phase 2 replacement for new systems C code

- **Will own:** new code that would have been C. Cross-compilation toolchain
  (`zig cc` as `CC` replacement).
- **Why:** ABI-compatible with C (incremental adoption); built-in cross-compilation;
  no UB by default.
- **Adoption gate:** Phase 2 milestone — see `POLYGLOT-ROADMAP-TRACKING.md` Initiative 4.
  Tooling is pinned but not installed by default.

### 3.2 F\* / Lean 4 — Phase 2 formal verification for crypto

- **Will own:** specifications and proofs for PQC primitives (Kyber, Dilithium) and zkVM
  critical sections. Extracted C lives in `c/crypto/verified/`.
- **Why:** mathematical correctness proofs for the "quantum-era OS" claim.
- **Adoption gate:** Phase 2 — see Initiative 5. F\* is preferred over Lean 4 because of
  the mature HACL\* extraction path; reassess at Month-10 review.
  Tooling is pinned but not installed by default.

---

## 4. Languages we have considered and rejected

| Language | Why considered | Why rejected (for now) |
|---|---|---|
| Erlang/Elixir | Supervisor trees, hot reload | We adopt the **patterns** via Rust crates (Initiative 6). Adding BEAM is too much runtime for the value. |
| Solidity | On-chain contracts | `services/chain/` already covers chain glue. If on-chain contracts ship, prefer ink! (Rust) over Solidity to keep the Rust charter intact. |
| Java/JVM | Existing ecosystem | C# runtime already covers the managed-VM slot. Adding JVM doubles maintenance for marginal ecosystem reach. |
| Lua / JavaScript (non-UI) | Scripting | Python owns scripting in § 2.6. TS owns UI in § 2.7. No third scripting layer. |
| OCaml / Haskell | Type-system rigour | F\* covers verified code; Rust covers safe systems code. No room. |

---

## 5. Process: adding a language

1. Open a discussion citing a workload that *no* charter language serves well.
2. Update this file with a proposed § 2.x entry (owns / why / must-not / FFI gateway).
3. Tag a `LANGUAGES.md` reviewer (placeholder: see `CODEOWNERS`).
4. Land the charter update *before* any code in the new language.

## 6. Process: deprecating a language or implementation

See `DEPRECATIONS.md`. Every deprecation must reference the LANGUAGES.md decision that motivates it.

---

## 7. Enforcement

- `CODEOWNERS` enforces per-directory language boundaries (target — Phase 1 milestone).
- CI's polyglot job (`.github/workflows/polyglot-ci.yml` — planned) builds each language column
  independently so cross-language regressions surface immediately.
- New raw `extern "C"` blocks in `services/` require a `LANGUAGES.md` reference in the PR
  description after Phase 1 milestone 1.

---

## 8. Quick reference: which language for which task?

| Task | Use |
|---|---|
| Add a kernel feature | Rust |
| Add a new local service | Rust |
| Add a network-protocol service | Go |
| Add a UI screen | TypeScript |
| Write a one-off ops script | Python |
| Add an AI/ML pipeline step | Python |
| Add a graphics compositor feature | C++ (only inside `services/window_server_cpp/`) |
| Add a .NET app entry point | C# (only inside `services/app_runtime_cs/`) |
| Add a sandboxed user app | WASM (Rust, Go, Python, or JS source — Component Model) |
| Add or modify a syscall | Rust + the C-ABI floor (kernel boundary) |
| Wrap a new external C library | Rust `-sys` crate |
| Add cryptography | Rust + HACL\* C (verified). Never roll your own. |

When in doubt, the answer is **Rust**, and the burden of proof is on choosing anything else.
