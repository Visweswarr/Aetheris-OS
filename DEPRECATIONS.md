# Polymera OS — Deprecation Roadmap

**Status:** v0.1 — skeleton. No code is being removed yet. This file is the **decision log**
that pairs with `LANGUAGES.md` and the polyglot roadmap.

Each entry below is **proposed**, not approved. Approval requires:

1. A DRI for the migration (see `POLYGLOT-ROADMAP-TRACKING.md` § Initiative 8).
2. Confirmation from a domain owner that the "canonical" choice covers all features of the
   deprecated one — or an explicit list of features that will be dropped.
3. A timeline approved at a Phase 1 milestone review.

The path findings below were taken from `docs/polyglot/IPC-AUDIT.md` § 5 and may differ from
the roadmap's audit table where the roadmap was inaccurate about file locations.

---

## D-1: `c/ngfs/` (C Merkle-tree code)

- **Status:** proposed
- **Duplicate of:** `services/ngfs/` (Rust)
- **Reason:** Two implementations of NGFS Merkle infrastructure. Rust version is larger
  and is the one wired into the service mesh (`services/ngfs/src/bench.rs` uses prost).
- **Canonical choice:** `services/ngfs/` (Rust). Per `LANGUAGES.md` § 2.1, new local-service
  code is Rust.
- **What `c/ngfs/` contains:** `ngfs_merkle.c`, `ngfs_merkle.h`, `BUILD`.
- **Migration:** confirm `services/ngfs/` has Merkle parity with `c/ngfs/ngfs_merkle.c`.
  If yes, retire. If no, port the missing Merkle helpers to `services/ngfs/` first.
- **Note:** The roadmap stated the Rust version lives in `kernel/.../ngfs` — that's wrong.
  No NGFS code exists in `kernel/src/`.
- **Proposed timeline:** deprecation warning at M+11, default-off at M+13, delete at M+14.

## D-2: `crypto/liboqs/` (entire directory, not just `bindings.rs`)

- **Status:** **approved for deletion — awaiting user `git rm`**
- **Evidence (gathered 2026-05-19):**
  - `crypto/build.rs` compiles `src/liboqs_stub.c` (stub), not anything in `crypto/liboqs/`.
  - `crypto/src/lib.rs` declares `mod crypto;` → `crypto/src/crypto/mod.rs` →
    `pub mod liboqs;` → `crypto/src/crypto/liboqs/mod.rs`. **None of this chain touches
    `crypto/liboqs/`.**
  - `crypto/liboqs/bindings.rs` and `crypto/src/crypto/liboqs/bindings.rs` are byte-identical
    (`diff -q` returned empty; both 209 lines).
  - `crypto/liboqs/{build.rs,wrapper.c,wrapper.h}` belong to an earlier "bundle-and-build
    real liboqs" approach that was replaced by the pqcrypto-\*-crate + stub-`liboqs_stub.c`
    approach in the active Cargo.toml. None of those files are referenced from the package
    root build.rs or src/.
  - Active crypto library uses `pqcrypto-kyber` and `pqcrypto-dilithium` Cargo deps
    (`crypto/Cargo.toml` lines 19–20), feature-gated under `kyber` / `dilithium`.
- **Canonical choice:** `crypto/src/crypto/liboqs/` stays. `crypto/liboqs/` is dead.
- **Recommended deletion command (run from repo root):**
  ```bash
  git rm -r crypto/liboqs/
  cargo build -p polymera-crypto --features full   # sanity-check
  ```
- **Why I did not run this in-session:** zero-callers analysis from grep is strong but not
  proof. A build-script glob, a packaging script, or an out-of-tree consumer (the legacy
  `EPIC_SUMMARY.md` still references this path) could break. The one-line `git rm` is
  trivially reversible — but the audit responsibility belongs to a domain owner.

## D-3: `c/aicore.c` + `go/aicore.go` (relative to `services/ai/src/aicore.rs`)

- **Status:** **investigation needed** — *not* yet proposed for deprecation.
- **Reason:** Three-way overlap. `services/ai/src/aicore.rs` uses `extern "C"` 4 times, which
  suggests it's *binding to* the C implementation, not duplicating it. `go/aicore.go` might
  be a separate Go consumer or a parallel reimplementation.
- **Required:** clarify the relationship before deciding. If the C is the real impl and the
  Rust/Go files are bindings, this is **not** a duplicate — the LANGUAGES.md § 2.1 FFI rule
  permits this as a legacy boundary.
- **Action:** add to the Phase 1 audit follow-up list. Do not deprecate yet.

## D-4: Networking surface — `services/net_go/` vs `services/polynet/` vs `services/posix/`

- **Status:** **investigation needed** — likely *not* a 1:1 duplicate.
- **Roles plausible:**
  - `services/net_go/` — Go network stack (per `LANGUAGES.md` § 2.3).
  - `services/polynet/` — Rust DTN (delay-tolerant networking) with proto contracts.
  - `services/posix/` — Rust POSIX shim, possibly different concern entirely.
- **Roadmap claim** ("`net_go` vs `services/posix`") oversimplifies what's on disk.
- **Action:** in Phase 1, each service owner writes one paragraph in this file describing
  what their service uniquely owns. If two paragraphs overlap, then deprecation can be
  discussed.

## D-5: Filesystem surface — `services/fs_go/` vs `services/ngfs/` vs `c/fuse/`

- **Status:** likely **all kept**, with clarified roles.
- **Proposed clarification:**
  - `services/fs_go/` — network filesystem (per `LANGUAGES.md` § 2.3).
  - `services/ngfs/` — Polymera-native filesystem (NGFS).
  - `c/fuse/` + `services/ngfs/fuse/` — FUSE bridge (C-ABI forced by FUSE).
- **Action:** document these roles in each service's `README.md` so the audit doesn't
  flag them again.

## D-6: `services/wasm_driver/` stub `Cargo.toml`

- **Status:** **not a deprecation** — flagged here so it does not get conflated with one.
- **Issue:** `Cargo.toml` declares no `wasmtime` or `wasmer` dependency. The file says
  "For now, we define the interface."
- **Required for Initiative 3:** add `wasmtime = { version = "...", features = ["component-model"] }`
  to this crate before any other Component Model work proceeds. **This is an addition, not
  a deprecation, and it is gated on confirmation that we want to pin a Wasmtime version.**

---

## Format for new entries

```markdown
## D-N: <path being deprecated>

- **Status:** proposed | approved | in-progress | complete
- **Duplicate of:** <canonical path>
- **Reason:** <one-line justification>
- **Canonical choice + LANGUAGES.md reference:** <which clause>
- **Migration:** <what must be ported first>
- **Timeline:** <when warnings, when default-off, when deleted>
- **DRI:** <name>
```

## Anti-patterns this file exists to prevent

- Silently deleting code without an audit reference.
- Marking something "duplicate" because the roadmap said so, when the audit shows they serve
  different purposes (see D-3, D-4, D-5).
- Migrating without first confirming the canonical implementation has feature parity.
