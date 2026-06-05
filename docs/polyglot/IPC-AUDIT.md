# Polymera OS — IPC Surface Audit

**Status:** Phase 1 (foundation) — initial snapshot, audit-only. No code moved.
**Last updated:** 2026-05-19

This document inventories the current cross-language and cross-service communication patterns
in the Polymera OS workspace. It exists to inform the WIT/Component-Model migration ordering
described in `POLYGLOT-ROADMAP-TRACKING.md` § Initiative 2.

OS-REF/ (third-party reference operating systems vendored for study) is excluded from all
counts below — it is not Polymera code.

---

## 1. Interface Definition Mechanisms in Use

| Mechanism | Canonical location | Generators | Languages targeted |
|---|---|---|---|
| Protobuf (Buf-managed) | `proto/` (workspace root) + per-service `proto/` | `buf generate` → Go, Rust, TypeScript, Python | Go, Rust, TS, Python |
| Per-service Protobuf | `services/ai_core/proto/ai_core.proto`, `services/hello/proto/hello.proto` | `build.rs` (prost) + `scripts/gen-proto.{sh,bat}` | Rust + Go + TS + Py |
| Rust `extern "C"` FFI | scattered across kernel, services, crypto, drivers | hand-written | Rust ↔ C |
| WIT / Component Model | **none yet** | — | — |

**Implication:** There is no WIT in the tree yet. Adding `wit/` (Initiative 2 / 3) is greenfield —
it will not conflict with existing IDL.

---

## 2. Protobuf footprint

Root proto package (`proto/`):

- `dtn.proto`         — delay-tolerant networking
- `health.proto`      — health probes
- `intent.proto`      — intent bus
- `session.proto`     — session ABI
- `test.proto`        — fixture for breaking-change tests

Per-service proto:

- `services/ai_core/proto/ai_core.proto`
- `services/hello/proto/hello.proto`

Buf configuration is present and active:

- `proto/buf.yaml`, `proto/buf.gen.yaml`, `proto/buf.work.yaml`
- Generates Go (gRPC), Rust (gRPC), TypeScript (gRPC), Python (gRPC).
- CI: `buf lint` and `buf breaking` are wired (see `proto/README.md`).

Files importing Protobuf machinery (Polymera code only, OS-REF excluded):

- `services/ai_core/{build.rs, src/generated/ai_core.rs, src/error.rs, examples/client.rs, tests/ipc_roundtrip.rs}`
- `services/polynet/dtn/{Cargo.toml, build.rs, src/lib.rs, src/envelope.rs}`
- `services/hello/{src/main.rs, src/client.rs, src/health_client.rs, src/metrics_client.rs, BUILD}`
- `services/health/{src/main.rs, BUILD}`
- `services/twin/BUILD`
- `services/xr/scene/{src/rpc.rs, Cargo.toml}`
- `services/ngfs/src/bench.rs`
- `go/tooling/ai_core/ai_core.pb.go` (generated)
- `tooling/python/ai_core/ai_core_pb2.py` (generated)
- `tooling/ts/ai_core/src/ai_core_pb.ts` (generated)
- `go/tooling/devctl/cmd/ai_*.go` (5 files — devctl CLI clients)

**Verdict:** Protobuf is the de-facto cross-language contract today, with Buf governance already in place.
The migration target is **not** "rip out Protobuf" — it is **WIT as IDL for component-shaped surfaces,
Protobuf-over-PolyBus for the long-running service mesh** (per roadmap § 2).

---

## 3. Raw `extern "C"` FFI footprint (Polymera code only)

Total: **16 occurrences across 9 files** in `kernel/`, **3 files** in `services/`.

### Kernel (expected — these are the FFI floor and stay C-ABI):

| File | Count | Justification (audit only — no change yet) |
|---|---|---|
| `kernel/src/main.rs` | 8 | boot/entry symbols — must stay C-ABI |
| `kernel/src/syscall/mod.rs` | 1 | syscall entry — C-ABI |
| `kernel/src/syscall/handlers.rs` | 1 | syscall dispatch |
| `kernel/src/sched/context.rs` | 1 | context-switch asm trampoline |
| `kernel/tests/{ipc,integration_ipc,vm_api,phys_buddy_alloc,slab_allocator}_tests.rs` | 5 | test harness FFI shims |

### Services (these are the candidates the roadmap targets for WIT migration):

| File | Count | Notes |
|---|---|---|
| `services/ai/src/aicore.rs` | 4 | aicore C bridge (also has `c/aicore.c`, `go/aicore.go` — see § 5) |
| `services/wallet/src/ffi.rs` | 11 | wallet FFI surface |
| `services/ngfs/fuse/main.rs` | 20 | FUSE callbacks — fuse_lowlevel forces C-ABI |

### Drivers / userland / crypto:

| File | Count | Notes |
|---|---|---|
| `drivers/usb.rs` | 2 | USB driver shim |
| `userland-stubs/src/lib.rs` | 2 | syscall stub |
| `userland-stubs/examples/syscall_example.rs` | 2 | example |
| `crypto/src/crypto/liboqs/bindings.rs` | 1 | liboqs PQC bindings |
| `crypto/liboqs/bindings.rs` | 1 | duplicate — see § 5 |
| `rust/bindings/polycrypto-sys/src/lib.rs` | 1 | -sys crate |

**Verdict for Initiative 2:** The FFI surface that is **eligible** for WIT migration is small and
specific — `services/ai/src/aicore.rs`, `services/wallet/src/ffi.rs`, and any new service FFI.
The kernel boundary, FUSE callbacks, and liboqs `-sys` crates are *not* eligible — they live below
the WIT line by necessity.

---

## 4. Service-mesh transport (PolyBus / tonic / prost)

Files using tonic/prost/grpc (Polymera code only, 22 files):

- All `services/hello/*`, `services/ai_core/*`, `services/health/*`, `services/twin/*`,
  `services/xr/scene/*`, `services/polynet/dtn/*`, `services/ngfs/src/bench.rs`.
- BUILD files for those services.

**Verdict:** The Rust service mesh uses Protobuf+tonic today. No raw socket-level service IPC
was found outside of generated stubs. This matches the roadmap's "Protobuf-over-PolyBus" target —
the work is to keep generated bindings under one IDL, not to swap transports.

---

## 5. Duplicate-implementation candidates surfaced by the audit

These differ from the roadmap's audit table in § 8 — captured here so the deprecation roadmap
in `DEPRECATIONS.md` is grounded in actual file paths, not the roadmap's approximate map.

| Component | Actual locations on disk | Roadmap's claim | Audit note |
|---|---|---|---|
| NGFS | `c/ngfs/{ngfs_merkle.c,.h}` + `services/ngfs/` (Rust) | Roadmap said `kernel/.../ngfs` (Rust) vs `c/ngfs` | **Roadmap is wrong about the path.** Rust NGFS lives in `services/ngfs/`, not in `kernel/`. Decision below in DEPRECATIONS.md. |
| aicore | `c/aicore.{c,h}` + `go/aicore.go` + `services/ai/src/aicore.rs` (calls C via FFI) | not listed | **Three-way overlap.** Likely the C is the real impl and Rust/Go are bindings — needs confirmation before deprecation. |
| liboqs bindings | `crypto/src/crypto/liboqs/bindings.rs` + `crypto/liboqs/bindings.rs` | not listed | Two `bindings.rs` files for the same C library — likely an accidental duplicate. |
| Networking | `services/net_go/` (Go) + `services/polynet/` (Rust, with `dtn/`) + `services/posix/` (Rust) | Roadmap said `net_go` vs `services/posix` | Three implementations, not two. `polynet/dtn` is the only one with proto contracts. |
| Filesystem | `services/fs_go/` (Go) + `services/ngfs/` (Rust) + `c/fuse/` + `services/ngfs/fuse/` | Roadmap said `fs_go` vs "Rust alternatives" | Four locations touch filesystem concerns. Different roles plausible; needs domain-expert sign-off. |
| WASM driver | `services/wasm_driver/` (Rust, stub) | not listed | `Cargo.toml` declares no wasmtime/wasmer dependency yet — it's an interface stub. Component Model adoption (Initiative 3) must add the actual runtime. |

---

## 6. Audit conclusions feeding the roadmap

1. **Protobuf is not the enemy.** Buf governance is already in place. The WIT migration (Initiative 2)
   should target the *raw-FFI* and *Component Model app boundary* surfaces, not Protobuf services.

2. **The eligible-for-WIT FFI surface in service code is exactly 3 files** (`ai/aicore.rs`,
   `wallet/ffi.rs`, `ngfs/fuse/main.rs`). Of those, only `ai/aicore.rs` and `wallet/ffi.rs`
   are realistic candidates (FUSE forces C-ABI). This is much smaller than the roadmap implied.

3. **`services/wasm_driver` is a stub.** Initiative 3 (Component Model) needs to add wasmtime
   dependency before any of the rest of that initiative can proceed.

4. **The duplicate map in the roadmap is inaccurate in two places** (NGFS location; networking
   has three implementations not two). `DEPRECATIONS.md` uses the disk-truth from this audit.

5. **No WIT files exist anywhere.** Greenfield for Initiative 2.

---

## 7. Audit commands used (reproducible)

```bash
# Raw FFI footprint (Polymera code, not OS-REF)
rg "extern\s+\"C\"" --type rust kernel/ services/ drivers/ crypto/ rust/ userland-stubs/

# Protobuf footprint
rg "protobuf|prost|tonic|grpc" --type rust --type go --type python --type ts services/ go/ tooling/

# WIT footprint (currently empty)
fd -e wit .
```
