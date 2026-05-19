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
| **Current**                | **404** | **−215** | **35% of baseline cleared**         |

## Landed commits (in order)

1. `b29d7cc kernel: structural type fixes for build` — ToString sweep + Ord derives + TaskId::value + Instant helpers + ActionV1 builders
2. `eea9d53 contracts: ECDSA recovery in verifyBatchSignature` — actually verifies ECDSA + binds to PQC metadata via payloadHash
3. `dbaf2d6 kernel: fail-closed insecure-toy PQC stubs + RNG-backed session keys` — verify() returns false, compile_error gate, RNG for session keys
4. `fa51fc8 dao: introduce services/dao crate with hardened proposal execution` — new crate, `value < 0` dead branch removed
5. `3151d96 chore: rebrand README, label dashboard mockup, drop lint-test junk` — cosmetic
6. `16adfc3 kernel: replace AuditLogInput trait with three positional audit_log args` — Cluster A
7. `99acee7 kernel: re-add planner schema fields (time/cost totals, value_scalar, seq)` — Cluster G
8. `12e4b21 kernel: fully-qualify ToString in audit_codes macros` — Cluster I

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

### Cluster B — `__rdmsr` / `__wrmsr` not in scope (8 × E0425)

`core::arch::x86_64` does not export MSR intrinsics on the toolchain in
use. Affected sites all in [kernel/src/hal/x86_64/apic.rs:283-417](kernel/src/hal/x86_64/apic.rs#L283).

**Approach**: add a small `kernel/src/hal/x86_64/msr.rs` module with
`#[inline] unsafe fn rdmsr(reg: u32) -> u64` and `wrmsr(reg, val)`
implemented via inline asm (`rdmsr` / `wrmsr` instructions). Import
those into `apic.rs` in place of the missing intrinsics.

### Cluster C — `VirtAddr::new` in `static` initializers (8 × E0015)

The `x86_64` crate's `VirtAddr::new` is not `const fn`. Affected sites all
in [kernel/src/mm/paging.rs:385-407](kernel/src/mm/paging.rs#L385).

**Approach**: switch the statics to `VirtAddr::new_truncate` (which is
`const`) if the address is known to be canonical, OR move the statics
behind `lazy_static!` / `spin::Lazy`. Inspect each one first — some may
genuinely need full `VirtAddr::new` validation at first access rather
than build time, which favours `Lazy`.

### Cluster D — Move out of `static RING[_]` (7 × E0507)

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

### Cluster F — `Page<_>` type annotations (5 × E0283)

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

### Cluster H — `multiple applicable items in scope` (10 × E0034)

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

## Recommended attack order

1. **A** (audit_log) — biggest cluster, smallest design surface; one
   function rewrite kills ~23 errors and a trait that adds no value.
2. **G** (schema field re-additions) — pure additions, no risk; kills
   ~17 errors and unblocks visibility on the planner/executor paths.
3. **I** (remaining ToString) — mechanical, copy the pattern from
   commit `b29d7cc`.
4. **F** (`Page<S>` annotations) — local, easy.
5. **C** (VirtAddr statics) — moderate; requires per-static decision
   between `new_truncate` and `Lazy`.
6. **B** (MSR intrinsics) — adds one new file with inline asm.
7. **D** (RING statics) — moderate; the cleanest fix may touch the
   ring's data representation.
8. **E**, **H**, **J** — long tail; revisit error breakdown after the
   above and pick up whatever's left.

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
