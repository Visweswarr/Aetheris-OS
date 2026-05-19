# KERNEL-BUILD-GREEN

Working tracker for the polymera-kernel "cargo check exits 0" milestone.

## Why this exists

Until `cd kernel && cargo check` exits cleanly, every claim about the
kernel — process management, IPC, capabilities, AI integration — is
unverifiable. This file is the single source of truth for what's been
fixed, what's left, and the recommended attack order so future sessions
can pick up the thread without re-surveying from scratch.

When the kernel is green, the next milestone is one real AI workload
end-to-end (capability-gated log summarization wired CLI → planner →
executor → wasm_driver → tool → result, with at least one real metric
on the dashboard).

## Trajectory

| Point in time              | Errors | Δ      | Notes                                  |
| -------------------------- | ------ | ------ | -------------------------------------- |
| Session start              | 619    | —      | Baseline from `cargo check` on master  |
| After ToString sweep (29 files) | 477 | −142 | `use alloc::string::ToString;`         |
| After StreamId/PredId Ord  | 470    | −7     | `#[derive(PartialOrd, Ord)]`           |
| After TaskId::value() alias | 465   | −5     | `pub const fn value(&self) -> u64`     |
| After ActionV1 + Instant fixes | 447 | −18    | with_cost_estimate, as_millis, serde   |
| After Cluster A (audit_log)| 427    | −20    | trait drop, three positional args      |
| After Cluster G (schema)   | 408    | −19    | time_estimate, total_*, value_scalar, seq |
| After Cluster I (ToString) | 404    | −4     | UFCS in audit_codes macros             |
| After Cluster F (Page<S>)  | 399    | −5     | Page::<Size4KiB>::containing_address   |
| After Cluster C (VirtAddr) | 391    | −8     | VirtAddr::new_truncate in statics      |
| After schema-init follow-up | 373   | −18    | ConstraintV1 init + duplicate impl drop |
| After Cluster B (MSR)      | 364    | −9     | hal/x86_64/msr.rs inline asm wrappers  |
| After Cluster D (RING)     | 356    | −8     | derive Copy on AuditEntry              |
| After Cluster K (PQC serde) | 352   | −4     | serde derives on insecure-toy stubs    |
| After Cluster L (spin locks) | 325  | −27    | normalize `spin::Mutex::lock()` usage  |
| After Cluster M (IDT ABI)  | 307    | −18    | x86_64 IDT field names + handler ABI   |
| After Cluster N (LLM)      | 276    | −31    | schema/session/backend consistency     |
| After Cluster O (IPC caps) | 259    | −17    | canonical `security::cap_v2` for IPC   |
| After Cluster P (IPC payload/syscall) | 237 | −22 | payload bytes, policy unwrap, trace casts |
| After Cluster Q (HPET)     | 221    | −16    | usize MMIO offsets, Copy mode, rdtsc asm |
| After Cluster R (MM/TLB)   | 213    | −8     | flush page start addr, lazy statics, as_micros |
| After Cluster S (loader)   | 199    | −14    | cap header drift, MemoryFlags, LoaderResult demo |
| After Cluster T (sched tick) | 125   | −74    | jitter budget state, lazy statics, ABI feature call sites |
| **Current**                | **125** | **−494** | **80% of baseline cleared**         |

## Landed commits (in order)

1. `b29d7cc kernel: structural type fixes for build` — ToString sweep + Ord derives + TaskId::value + Instant helpers + ActionV1 builders
2. `eea9d53 contracts: ECDSA recovery in verifyBatchSignature` — actually verifies ECDSA + binds to PQC metadata via payloadHash
3. `dbaf2d6 kernel: fail-closed insecure-toy PQC stubs + RNG-backed session keys` — verify() returns false, compile_error gate, RNG for session keys
4. `fa51fc8 dao: introduce services/dao crate with hardened proposal execution` — new crate, `value < 0` dead branch removed
5. `3151d96 chore: rebrand README, label dashboard mockup, drop lint-test junk` — cosmetic
6. `16adfc3 kernel: replace AuditLogInput trait with three positional audit_log args` — Cluster A
7. `99acee7 kernel: re-add planner schema fields (time/cost totals, value_scalar, seq)` — Cluster G
8. `12e4b21 kernel: fully-qualify ToString in audit_codes macros` — Cluster I
9. `8723978 docs: KERNEL-BUILD-GREEN tracker — clusters A, G, I done; 215 errors burned` — tracker update
10. `36f6c14 kernel: paging Page<Size4KiB> annotations + ConstraintV1 init sync` — Clusters F + C
11. `e8e44ee kernel: hal/x86_64/msr inline-asm wrappers for rdmsr/wrmsr` — Cluster B
12. `549b58b kernel: derive Copy on AuditEntry so drain paths can snapshot by value` — Cluster D
13. `1481bf5 kernel: serde derives on insecure-toy PQC stub types` — Cluster K
14. `7845b8e kernel: normalize spin mutex locking semantics` — Cluster L
15. `c79b44d kernel: repair x86_64 IDT handler ABI` — Cluster M
16. `e51316b kernel: make LLM schema session and backend consistent` — Cluster N
17. `f0f93e9 kernel: canonicalize IPC V2 capability validation` — Cluster O
18. `ef64bc2 kernel: align IPC message payload and syscall helpers` — Cluster P
19. `f1b4805 kernel: normalize HPET register arithmetic and TSC read` — Cluster Q
20. `84ad992 kernel: align TLB flush and timing boundaries` — Cluster R
21. `afb37ef kernel: align user task loader with current cap and memory APIs` — Cluster S
22. `TBD kernel: normalize scheduler jitter budget state` — Cluster T

## Remaining error clusters

Counts from a fresh `cargo check` against HEAD. Each row is a cluster
that should be addressed as one unit (one design decision, then the fix
cascades). The "Approach" column is the recommended first move, not a
binding plan.

### ✅ Cluster A — `audit_log` arity mismatch (CLEARED in 16adfc3)

The `audit_log<T: AuditLogInput>(input: T)` API at
[kernel/src/secman/audit.rs:531](kernel/src/secman/audit.rs#L531) takes a single
tuple, with trait impls for `(u16, AuditLevel, &str)` and
`(u16, AuditLevel, &String)`. Call sites pass **three positional
arguments** instead of a tuple:

- [kernel/src/syscall/validate.rs:409](kernel/src/syscall/validate.rs#L409)
- [kernel/src/syscall/copy.rs:104, :119, :129, :146](kernel/src/syscall/copy.rs#L104)
- [kernel/src/secman/audit.rs:230, :267, :294, :380, :418, :448, :471](kernel/src/secman/audit.rs#L230)

**Approach**: change `audit_log` to take three positional args directly
(`pub fn audit_log(code: u16, level: AuditLevel, msg: &str)`). Drop the
trait. ~10-20 callers, one function, no behavioural change. The trait
impls become dead code and can be removed in the same commit.

### ✅ Cluster B — `__rdmsr` / `__wrmsr` not in scope (CLEARED in e8e44ee)

`core::arch::x86_64` does not export MSR intrinsics on the toolchain in
use. Affected sites all in [kernel/src/hal/x86_64/apic.rs:283-417](kernel/src/hal/x86_64/apic.rs#L283).

**Approach**: add a small `kernel/src/hal/x86_64/msr.rs` module with
`#[inline] unsafe fn rdmsr(reg: u32) -> u64` and `wrmsr(reg, val)`
implemented via inline asm (`rdmsr` / `wrmsr` instructions). Import
those into `apic.rs` in place of the missing intrinsics.

### ✅ Cluster C — `VirtAddr::new` in `static` initializers (CLEARED in 36f6c14)

The `x86_64` crate's `VirtAddr::new` is not `const fn`. Affected sites all
in [kernel/src/mm/paging.rs:385-407](kernel/src/mm/paging.rs#L385).

**Approach**: switch the statics to `VirtAddr::new_truncate` (which is
`const`) if the address is known to be canonical, OR move the statics
behind `lazy_static!` / `spin::Lazy`. Inspect each one first — some may
genuinely need full `VirtAddr::new` validation at first access rather
than build time, which favours `Lazy`.

### ✅ Cluster D — Move out of `static RING[_]` (CLEARED in 549b58b)

The audit ring buffer is a `static RING: [_; N]` and the consumer code
moves elements out instead of borrowing. Affected in
[kernel/src/secman/audit.rs](kernel/src/secman/audit.rs) (drain loop).

**Approach**: either (a) clone before consuming, (b) restructure the
ring to use `[Mutex<Option<Entry>>; N]` so `take()` is valid, or (c)
use an `arrayvec`-style ring with proper drain semantics. Option (a)
is the smallest change; pick that unless profiling argues otherwise.

### Cluster E — Closure arity in `Page::range` / similar (5 × E0593)

Callers pass single-arg closures where the iterator expects two-arg
closures.

**Approach**: inspect each call site individually. Often a `for_each`
vs `fold` vs `map` mix-up.

### ✅ Cluster F — `Page<_>` type annotations (CLEARED in 36f6c14)

The `x86_64` `Page<S>` is generic over page size and the compiler can't
infer S. Annotate with `Page<Size4KiB>` or `Page<Size2MiB>` as
appropriate.

### ✅ Cluster G — Schema drift: ActionV1 / PlanV1 / ConstraintV1 fields (CLEARED in 99acee7)

The intent schema lost fields that callers still reference:

- `ActionV1::time_estimate` (4 errors)
- `PlanV1::total_time` / `total_cost` / `constraints_applied` (9 errors)
- `ConstraintV1::value_scalar` (3 errors)
- `CompletionChunkV1::seq` (3 errors)

**Approach**: re-add the missing fields to
[kernel/src/intent/schema.rs](kernel/src/intent/schema.rs) (and policy/schema.rs
if mirrored), with `#[serde(default)]` so existing serialized data
still loads.

### ✅ Cluster H — `multiple applicable items in scope` (CLEARED in 36f6c14, was a downstream of the duplicate ActionV1 impl)

Two traits both provide a method with the same name on the same type
(common pattern: `format::Format` and `core::fmt::Display`).

**Approach**: spot-check, then disambiguate with fully qualified
`Trait::method(&value)` syntax at each call site.

### ✅ Cluster I — `to_string` on `u64` and stray primitives (CLEARED in 12e4b21)

`(some_u64).to_string()` requires `ToString` on the primitive, which
needs the same `use alloc::string::ToString;` that already landed for
the 29 sweep files but is missing from a few more.

**Approach**: identical to the original ToString sweep — find the
remaining files via `cargo check 2>&1 | grep "to_string"` and apply
the same one-line import.

### Cluster J — Long tail (~80+ × E0308 mismatched types, plus assorted ≤4-error errors)

The remaining ~98 E0308 errors and the long tail of E0061 / E0599 /
E0610 / E0593 / E0277 errors that don't fit a single named cluster are
individual API drifts. Each needs its own decision — there is no
mechanical sweep that resolves them.

**Approach**: address after A–I, when the cascading effects from the
big clusters have settled. Many will turn out to be downstream of A–I
fixes and disappear without per-site work.

### Cluster K — serde on PQC stub types (CLEARED in 1481bf5)

(Added late and cleared in the same session: InsecureDilithium* /
InsecureKyber* needed serde::Serialize/Deserialize because CapTokenV2
carries a DilithiumSignature field and itself derives serde. Closed
4 errors.)

### ✅ Cluster L — `spin::Mutex::lock()` treated as `Result` (CLEARED)

Several kernel subsystems used `spin::Mutex` as though `lock()` returned
`Result<MutexGuard, _>`, copying the shape of `std::sync::Mutex`. In
`spin`, locks are non-poisoning and return the guard directly. Cleared
across HAL timer/APIC/HPET jitter, scheduler jitter, secman policy,
assertion macros, intent counters, and WASI hostcall context snapshotting.

### ✅ Cluster M — x86_64 IDT field names and handler ABI (CLEARED)

The IDT setup used old/nonexistent field names (`divide_by_zero`,
`floating_point_exception`, `simd_floating_point_exception`,
`virtualization_exception`) and assigned Rust closures where the
`x86_64` crate requires `extern "x86-interrupt"` handler function
pointers. The table now uses the actual 0.14.13 field names and named
handlers with the exact no-error, with-error, page-fault, and diverging
machine-check signatures. IDT loading uses `load_unsafe()` instead of
trying to synthesize a descriptor pointer from a `MutexGuard`.

### ✅ Cluster N — LLM schema/session/backend consistency (CLEARED)

The LLM facade was split between several incompatible designs:
`SessionHandle` was a `u64` alias while call sites used `.0`,
`LlmSession` had no active state or completion ring, backend registry
had no availability API, and the global service tried to return a
reference out of a `Mutex<Option<_>>` guard. The LLM surface now has a
tuple `SessionHandle`, a session object with `send_prompt` /
`receive_chunks` / `close` / `get_info`, completion chunks with
token/tool/finish fields, backend availability checks, and a `spin::Once`
global service.

### ✅ Cluster O — IPC V2 capability validation split (CLEARED)

`ipc/auth.rs` was validating `security::cap_v2::CapTokenV2` by calling
the legacy `secman::cap_store` API, which returned incompatible
`secman::cap_v2` result types. IPC authentication now calls
`CapTokenV2::validate()` directly and maps `Success` / `Failure` into
the IPC auth result. Legacy-to-V2 conversion now uses the canonical
`security::cap_v2` constructor shapes, and the V2 header exposes a stable
byte representation for IPC capability IDs.

## Recommended attack order

Clusters A, B, C, D, F, G, H, I, K are all CLEARED (see Trajectory).
Remaining order:

1. **E** (closure arity, 5 errors) — per-call-site fixes for `for_each`
   vs `fold` confusions.
2. The **7+7 E0061** clusters ("function takes 2/3 args but 1/4 supplied")
   — each subgroup is likely a single API rename that needs trace-back to
   find the canonical signature, then a sweep across callers.
3. The **3 E0499** mutable-borrow conflicts — these usually want either
   a `let` to drop the first borrow or a `RefCell`/`Mutex` split. Look
   at site individually; risk of subtle regressions if rewritten quickly.
4. The **3 E0277 `?` couldn't convert error to &str** — add `From<X>`
   impls for the wrapping error type or rewrite call sites to use
   `.map_err(|e| ...)`.
5. **J** (98 × E0308 + long tail) — final mile, individual fixes.

After each cluster, re-run `cargo check 2>&1 | tail -3` to recount.
Update the "Trajectory" table in this file as part of every commit
that meaningfully moves the number.

## Hard rules (per the project defaults)

- **No new features while the kernel is red.** If a fix can wait until
  after green, it should.
- **No introducing more `unsafe` than the cluster's existing footprint
  requires.** Especially for B (MSRs) and C (VirtAddr).
- **No silencing errors with `#[allow]` or `unimplemented!()`.** A
  cluster is only closed when its root cause is fixed.
- **Commits stay narrow.** One cluster per commit, mention this file
  in the body so the trajectory stays auditable.
