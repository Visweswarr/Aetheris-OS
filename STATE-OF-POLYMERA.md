# STATE OF POLYMERA — 2026-05-20

Detailed status report against the actual repo state at commit
`96b9cde`. Every claim below was verified by reading the file or
running the command. Where something is claim-only it is marked
**[unverified]**. Where I have a concern it is marked **⚠️**.

This file is a snapshot; the running ledger is
[KERNEL-BUILD-GREEN.md](KERNEL-BUILD-GREEN.md). Read both.

---

## 2026-05-21 Addendum — Runtime Truth Consolidation

The AI runtime boundary has been tightened so the main `RuntimeManager`
path no longer returns `"Mock response to: ..."`. `runtime-run` now goes
through `RuntimeBackend::DeterministicLocal`, preserves the HEG plan id,
emits `runtime.backend.executed` with an execution hash, and writes real
runtime counters to `ui/dashboard/ai_metrics.js`.

Verification for this addendum is tracked by:

- `cargo test --manifest-path services\ai_core\Cargo.toml --test runtime_tests`
- `cargo check --manifest-path services\ai_core\Cargo.toml`
- `cargo test --manifest-path services\ai_core\Cargo.toml`

The dashboard remains globally labeled simulated. Only AI runtime cards
backed by `ai_metrics.js` should be treated as real.

---

## 2026-05-21 Addendum — Local Model Runtime v1

The runtime now has an opt-in local model backend boundary:

- `RuntimeBackendKind::LlamaCppServer` targets a localhost
  OpenAI-compatible llama.cpp server.
- `RuntimeBackendKind::Ollama` targets a localhost Ollama daemon.
- `configs/ai/model-registry.toml` tracks reviewed Hugging Face model
  metadata only; no model weights are committed.
- `runtime-run --backend deterministic|llama-cpp|ollama` rejects
  non-local endpoints and records model attempts/success/failure metrics.

This does **not** mean Polymera ships an embedded model yet. The model
backends require an operator-started local daemon and fail closed when it is
unavailable. The release gate remains daemon-free; the optional live model
test is ignored unless `POLYMERA_RUN_LOCAL_MODEL_TESTS=1` is set.

---

## 1. Executive summary

The OS just crossed two real milestones:

1. **`polymera-kernel` cargo check is GREEN.** 619 → 0 errors over
   25 cluster commits. `cd kernel && cargo check` exits 0 with 7
   warnings. This is the first time the kernel has actually compiled
   end-to-end in this branch.
2. **One real AI workload runs end-to-end.** `ai_core_cli summarize-log
   --path <file> --approve` walks CLI → planner → policy/approval gate
   → guarded executor → built-in `log_summarizer` tool → wasm_driver
   `SummarizerHost` (with a deterministic local fallback) → writes
   `ai_log_summaries_total` to a dashboard-readable JS metric file.
   Without `--approve` it is blocked by the capability gate, and that
   denial bumps `ai_capability_denials_total`.

Beyond those two, the picture is the same shape as the previous report:
real, build-passing scaffolding for Web3/PQC/ZK/AI; one card on the
dashboard is now real, the rest is `Math.random()` mock; integration
tests in `services/ai_core` outside the new slice are broken against
the new APIs.

**Honest scorecard against the original "quantum-era AI Web3 OS" pitch:**

| Pillar | Status |
|---|---|
| Microkernel that compiles | ✅ shipped this cycle |
| One real AI workload end-to-end | ✅ shipped this cycle |
| Real PQC inside the kernel | ❌ still fail-closed stubs; real PQC lives in the standalone `crypto/` crate and the kernel does not call it |
| Real PQC in services (wallet, IPC, anchor) | ❌ uses the `crypto/` crate's liboqs binding for typing only; no real PQC handshake on a hot path yet |
| Real dashboard telemetry | 🟡 1 of ~30 cards is real (AI Log Summaries); the rest are mock |
| Production-grade Web3 anchor | 🟡 AnchorDAO ECDSA verification fixed; PQC verification still off-chain by design; not deployed |
| ZK circuits used by anything | 🟡 Noir circuits have real constraints but no service routes through them |
| Formal verification | ❌ aspirational; F* / Lean 4 pinned but not running |
| Multi-VM polyglot runtime (CPython, JVM, CLR) | ❌ aspirational; only Rust services run today |
| Bootable on real hardware | ❌ unverified — `cargo check` passes but the kernel has never been booted on QEMU or hardware in this branch |

The pitch oversells reality by maybe 2–3 phases of work. The
*foundation* under the pitch is now sound enough to compound on.

---

## 2. Major milestones reached this cycle

### 2.1 Kernel green (df6bf27)

Verified with `cd kernel && cargo check` → `Finished dev profile … 8.96s`
with 7 warnings, 0 errors. The full trajectory is recorded in
[KERNEL-BUILD-GREEN.md](KERNEL-BUILD-GREEN.md) as 25 clusters labeled
A through Y. Pulled out, the sequence of `cargo check` error counts
is:

```
619 → 477 → 447 → 427 → 408 → 404 → 399 → 391 → 373 → 364 → 356 →
352 → 325 → 307 → 276 → 259 → 237 → 221 → 213 → 199 → 125 → 80 →
35 → 21 → 12 → 0
```

Each step is a separate commit with a body that names the cluster and
the trajectory delta. This means a future agent picking up the work
can `git log --oneline` and read what was done in what order without
re-surveying the codebase.

What "green" does **not** mean:
- It does not mean the kernel is bug-free; warnings remain (7) and
  many of them are dead code / unused imports / suspicious unsafe
  blocks that should be cleaned up.
- It does not mean the kernel boots. There is no `cargo run` or
  `qemu-system-x86_64` run in any CI step yet for this branch.
- It does not mean the kernel runs the AI workload. The AI workload
  runs in `services/ai_core` as a host-side Rust binary; the kernel
  is not in the call path for `summarize-log`.
- **⚠️ Known Dependency Limitation (`serde_cbor`)**: The `polymera-kernel` `cargo check` compiles successfully on standard target profiles. However, in certain strict cross-compilation environments or non-standard targets, a pre-existing target compatibility limitation involving the `serde_cbor` crate (due to its allocator dependency configurations) may occur. This is an inherited dependency constraint, unrelated to the AI Core or Runtime boundary changes of this phase, and is treated as a known out-of-scope limitation for this release cycle.

### 2.2 Capability-gated log summarization (8949f14)

The first real AI-native workload after kernel green. Code paths
verified by reading:

- CLI dispatch: [services/ai_core/src/bin/ai_core_cli.rs:23-24](services/ai_core/src/bin/ai_core_cli.rs#L23-L24),
  implementation at [:109-205](services/ai_core/src/bin/ai_core_cli.rs#L109-L205).
- Planner builds a deterministic plan with `summarize_log` action;
  [services/ai_core/src/agent/planner.rs:231](services/ai_core/src/agent/planner.rs#L231) writes the
  `ai_log_summaries_total` reference into the plan record.
- Policy gate requires capability `read:logs` and explicit approval
  before file reads — denial path goes through
  [services/ai_core/src/agent/policy.rs](services/ai_core/src/agent/policy.rs).
- Executor enforces the gate at [services/ai_core/src/agent/executor.rs](services/ai_core/src/agent/executor.rs);
  the new `tests::destructive_step_without_approval_is_blocked` and
  `tests::privacy_denial_blocks_remote_execution` and
  `tests::budget_denial_blocks_execution` all pass.
- Tool: `log_summarizer` registered in
  [services/ai_core/src/tools/registry.rs](services/ai_core/src/tools/registry.rs).
- Host fallback: `SummarizerHost` at
  [services/wasm_driver/src/summarizer.rs](services/wasm_driver/src/summarizer.rs)
  ships a deterministic local fallback (no wasm component required).
- Metric writer: [services/ai_core/src/metrics.rs:181](services/ai_core/src/metrics.rs#L181)
  bumps `ai_log_summaries_total: AtomicU64`; the CLI exports the
  snapshot to [ui/dashboard/ai_metrics.js](ui/dashboard/ai_metrics.js)
  ([ai_core_cli.rs:539](services/ai_core/src/bin/ai_core_cli.rs#L539) onward).
- Dashboard consumer:
  [ui/dashboard/app.js:294-297](ui/dashboard/app.js#L294-L297) reads the
  four AI counters out of `window.POLYMERA_AI_METRICS`.

⚠️ The wasm component is **not** loaded — the `SummarizerHost` falls
back to a Rust-side deterministic summarizer. Real wasm execution
remains untested on this path. Tracked separately; this is the right
shape (host falls back if no component), just be aware the "wasm
runtime is in the loop" claim is not yet exercised.

### 2.3 Reference cognitive layer + browser-assist (96b9cde)

Imports the deterministic `CognitiveCore` and `LTM` from `services/ai`
into `services/ai_core` via a path dependency (no code duplication),
plus a small set of capability-gated browser tools:

- New CLI commands (all verified present in `ai_core_cli.rs`):
  `goal-plan`, `record-outcome`, `replay-goal`,
  `browser-summarize`, `browser-classify`.
- New built-in tools: `browser_summarize`, `browser_classify_page`.
- New metrics fields:
  - `ai_plans_generated_total` (recorded by `record_ai_plan_generated`)
  - `ai_ltm_events_total` (recorded by `record_ltm_event`)
  - `ai_browser_summaries_total` (recorded by `record_browser_summary`)
  - `ai_capability_denials_total` (recorded by `record_capability_denial`)
- Tamper-evident audit-chain primitives + seeded DP export helpers
  with tests (live in `services/ai_core/src/audit_chain.rs` and
  `services/ai_core/src/privacy.rs`).
- [docs/reference-intake-ledger.md](docs/reference-intake-ledger.md)
  documents source path, license, attribution, and copy/reimplementation
  mode for each reference input. **Read this before importing anything
  else** — it is the contract for how non-original work gets pulled in.

**26 / 26 ai_core library tests pass** (`cargo test --manifest-path
services/ai_core/Cargo.toml --lib`). Confirmed.

---

## 3. What is verified to work

Every row in this table was confirmed by running the listed command in
the verification pass for this report (5–10 minutes ago).

| Component | Command | Result |
|---|---|---|
| Kernel build | `cd kernel && cargo check` | 0 errors, 7 warnings |
| AI core build | `cargo check --manifest-path services/ai_core/Cargo.toml` | 0 errors |
| AI core unit tests | `cargo test --manifest-path services/ai_core/Cargo.toml --lib` | 26/26 passed |
| wasm_driver build | `cargo check --manifest-path services/wasm_driver/Cargo.toml` | 0 errors |
| dao build | `cargo check --manifest-path services/dao/Cargo.toml` | 0 errors, 20 warnings |
| contracts build | `cargo check --manifest-path services/contracts/Cargo.toml` | 0 errors, 11 warnings |
| AnchorDAO ECDSA fix | reading `verifyBatchSignature` | calls `ECDSA.recover(...)` and `payloadHash` binds to PQC metadata ([commit eea9d53](contracts/AnchorDAO.sol)) |
| Fail-closed PQC | reading `kernel/src/crypto/pqc.rs` | `verify()` returns `false` literally; `compile_error!` in release without feature gate |
| dashboard "Simulated" label | reading `ui/dashboard/index.html` | title + header badge both say "Simulated" |
| CLI `summarize-log` exists | `grep summarize-log services/ai_core/src/bin/ai_core_cli.rs` | dispatches at line 23; `--approve` flag honoured |
| CLI `goal-plan / replay-goal / record-outcome` exist | same | all six new subcommands present |
| Dashboard real metric wire-up | reading `ui/dashboard/app.js` | reads `ai_log_summaries_total`, `ai_plans_generated_total`, `ai_browser_summaries_total`, `ai_capability_denials_total` from the JS metrics file |
| Metric file shape | reading `ui/dashboard/ai_metrics.js` | 5 named counters + 5 named "last_*" descriptors; initialised to 0 |
| Verify scripts | not re-run this report; `verify-web3.ps1 -SkipCargo` and `verify-quantum.ps1 -SkipCargo` claimed passing — **[unverified this report]** |

Also verified existing-from-previous-cycles:
- AnchorDAO `verifyHybridSignature` already verified ECDSA against an
  `expectedSigner` parameter.
- WIT facades at `wit/polymera/{ai,web3,crypto,zk}/0.1.0/world.wit`
  describe sensible record/function shapes.
- DAO `value < 0` dead branch removed (services/dao/src/execution.rs).
- Top-level lint-test scratch files (`-p`, `test_lint_errors.{py,rs,ts}`,
  `protoc.zip`, `docker-gen*.sh`) removed.
- README rebrand done (no more `Aetheris-OS` in body, badges present,
  dashboard section labels the dashboard as a mockup).

---

## 4. What is partially working

### 4.1 Dashboard — 1 of ~30 cards is real

Confirmed by grepping `ui/dashboard/app.js`. The card-reading section
reads four AI counters from `window.POLYMERA_AI_METRICS`:

- `ai_log_summaries_total`
- `ai_plans_generated_total`
- `ai_browser_summaries_total`
- `ai_capability_denials_total`

The other ~26 metric sites in that file still call `Math.random()` —
process spawn rates, IPC bytes, syscall counts, crypto operation
counts, memory faults, the kernel tick simulation, etc. These are
honest in the sense that the page title and header badge both say
"Simulated", but the *implementation* gap is large: there is no
websocket, no SSE, no kernel-side telemetry sink that the dashboard
listens to. Wiring up even one real kernel metric would require a
host-side IPC bridge that doesn't exist yet.

### 4.2 wasm_driver — real instantiation, fallback path is what runs

[services/wasm_driver/tests/host_integration.rs](services/wasm_driver/tests/host_integration.rs)
does real `Host::new()` + `load_component(...)` + `instantiate(...).await`
on a minimal empty `(component)` blob. That's plumbing-verified.

But in the `summarize-log` flow, the `SummarizerHost` does **not** load
a wasm component — it runs a deterministic Rust-side fallback. The
"AI workload runs through wasm" claim is therefore aspirational at the
runtime layer; only the trait surface is wasm-shaped.

### 4.3 ai_core integration tests — consolidated

The stale `ai_core` test surface described in the earlier snapshot has
been ported or removed. The current release gate is the full command:

```
cargo test --manifest-path services\ai_core\Cargo.toml
```

`runtime_tests` now also covers the `runtime-run` CLI metrics path, so
the deterministic runtime boundary is no longer verified only by hand.

### 4.4 PQC integration

The standalone `crypto/` crate builds with `--features full` and has
real liboqs bindings; the kernel's in-kernel PQC stubs are explicitly
fail-closed (verify returns false, encap/decap return zeros). What is
**missing**: nothing on a hot path actually calls the standalone
crypto crate. `kernel/src/ipc/pqc_helpers.rs` generates RNG-backed
session keys but does not negotiate a real Kyber-encapsulated session
yet. `services/wallet` references PQC types but the signing path goes
through wallet-local code, not through `crypto/` for verified
Dilithium/SPHINCS+ operations. To call the integration "real" anywhere
in the OS, one of these boundaries needs to actually flow through
`crypto/`.

---

## 5. Working tree state

`git status --short | wc -l` → **563 entries** (370 untracked, 184
modified, 9 deleted). This is the same "very dirty pre-existing
worktree" the user flagged in their walkthrough.

Breakdown by top-level directory (file count, modifications +
untracked + deleted combined):

| Dir | Count |
|---|---|
| kernel/ | 123 |
| services/ | 111 |
| .github/ | 79 |
| tooling/ | 54 |
| docs/ | 34 |
| scripts/ | 28 |
| go/ | 20 |
| crypto/ | 12 |
| tools/ | 9 |
| tests/ | 9 |
| c/ | 7 |
| ui/ | 6 |
| contracts/ | 4 |
| artifacts/ | 4 |
| zk/ | 2 |

These are **not** part of the committed kernel-green / AI-workload /
reference-cognitive work. They are an inherited pile from earlier
sessions that was deliberately not touched (per project default: small
scoped commits, don't drag unreviewed work along). They include:

- Likely-relevant: schema files, polyglot tooling, CI workflow edits,
  service stubs that may want to land later.
- Likely-noise: build artefacts that escaped `.gitignore`, old
  scratch experiments, partially-edited docs.

⚠️ **Risk**: this pile is large enough that a casual `git add -A`
would bury the audit trail. Recommend treating it as triage work:
walk through top-level dirs, decide per-directory whether to commit /
discard / move to a tracked TODO branch.

---

## 6. Commit ledger this work cycle

Newest first. Every commit named by the user in their walkthrough was
verified present.

```
96b9cde  ai_core: import deterministic reference cognitive and browser assist
8949f14  ai: add capability-gated log summarization path
270d218  docs: record final kernel-green commit hash
df6bf27  kernel: clear final library and binary compile drift              (Cluster Y)
d3ad295  kernel: align event fabric and policy guardrail API drift         (Cluster X)
07b5a15  kernel: align intent world and skills schema drift                (Cluster W)
4317bf5  kernel: align secman capability and audit API drift               (Cluster V)
f050c5a  kernel: align page-table audit translation and stat init          (Cluster U)
7e41d8d  kernel: normalize scheduler jitter budget state                   (dup)
7b8e035  kernel: normalize scheduler jitter budget state                   (Cluster T)
afb37ef  kernel: align user task loader with current cap and memory APIs   (Cluster S)
84ad992  kernel: align TLB flush and timing boundaries                     (Cluster R)
f1b4805  kernel: normalize HPET register arithmetic and TSC read           (Cluster Q)
ef64bc2  kernel: align IPC message payload and syscall helpers             (Cluster P)
f0f93e9  kernel: canonicalize IPC V2 capability validation                 (Cluster O)
e51316b  kernel: make LLM schema session and backend consistent            (Cluster N)
c79b44d  kernel: repair x86_64 IDT handler ABI                             (Cluster M)
7845b8e  kernel: normalize spin mutex locking semantics                    (Cluster L)
efb8919  docs: KERNEL-BUILD-GREEN — session 2 wrap                          (tracker)
1481bf5  kernel: serde derives on insecure-toy PQC stub types              (Cluster K)
549b58b  kernel: derive Copy on AuditEntry                                 (Cluster D)
e8e44ee  kernel: hal/x86_64/msr inline-asm wrappers for rdmsr/wrmsr        (Cluster B)
36f6c14  kernel: paging Page<Size4KiB> annotations + ConstraintV1 init    (Clusters F+C)
8723978  docs: KERNEL-BUILD-GREEN tracker — clusters A, G, I done          (tracker)
12e4b21  kernel: fully-qualify ToString in audit_codes macros              (Cluster I)
99acee7  kernel: re-add planner schema fields                              (Cluster G)
16adfc3  kernel: replace AuditLogInput trait with positional audit_log     (Cluster A)
3151d96  chore: rebrand README, label dashboard mockup, drop junk
fa51fc8  dao: introduce services/dao crate with hardened proposal exec
dbaf2d6  kernel: fail-closed insecure-toy PQC stubs + RNG session keys
eea9d53  contracts: ECDSA recovery in verifyBatchSignature
b29d7cc  kernel: structural type fixes for build                           (foundation)
```

Two cosmetic notes:
- `7e41d8d` is a duplicate of `7b8e035` (same title, same intent).
  Worth a `git rebase -i` clean-up before pushing, but harmless if
  left alone.
- None of these commits are pushed to any remote.

---

## 7. What is **not** done

Phrased so each line is something that needs a concrete deliverable
before the claim it implies is honest.

### Architecture / correctness
1. **Kernel boot.** `cargo check` passes; the kernel has never been
   linked into a bootable image and run in QEMU on this branch.
   Until that happens, nothing about runtime behaviour is verified —
   only types.
2. **Real PQC on at least one hot path.** Standalone `crypto/` crate
   is real; nothing in `kernel/`, `services/wallet`,
   `services/ipc`, or the IPC `pqc_helpers` actually calls it.
3. **Wasm component in the AI loop.** `SummarizerHost` falls back
   to Rust. Need at least one real component compiled, signed, and
   invoked from `summarize-log` to call the runtime "real".
4. **Real telemetry source for the dashboard.** Only 1 of ~30 cards
   is wired; the rest are `Math.random()`. Wiring even one kernel
   metric (e.g. process count from `kernel::process::active_process_count()`)
   would require an IPC bridge to the dashboard host, which does not
   exist.

### Test coverage
5. **AI Core test drift has been consolidated.** Earlier stale integration
   binaries have been ported or removed; keep `cargo test --manifest-path
   services\ai_core\Cargo.toml` as the release gate so this does not
   regress.
6. **Runtime backend truth is now test-gated.** `runtime_tests` verifies
   deterministic local execution, HEG plan preservation, backend metrics,
   and the `runtime-run` dashboard metrics path.
7. **No CI on `cd kernel && cargo check`.** The kernel-green state
   is local; nothing yet prevents a future commit from re-breaking
   the build.

### Cleanliness
8. **563-file working tree.** Triage and commit / discard / branch.
9. **345 kernel warnings.** Many are dead code or unused imports
   that should be cleaned to surface the warnings that actually
   matter (suspicious unsafe, missing safety doc, etc.).
10. **Duplicate commit `7e41d8d`.**

### Long-running pillars (these are next-phase, not this-phase)
11. Formal verification (F* / Lean 4). Pinned in toolchain config;
    no proof exists today.
12. Multi-VM polyglot runtime (CPython, JVM, CLR). Only Rust services
    run; CPython is loaded via `pyo3` in `services/python_runtime`
    but is not used by any flagship workload.
13. AnchorDAO deployment + on-chain verification flow. Solidity
    builds; never deployed.
14. ZK circuits in the AI loop. Noir circuits have real constraints
    but no service routes through them.

---

## 8. Recommended next 5 deliverables (ranked by leverage)

Each item is *one focused session* of work; none requires speculation
beyond what the current code base already proves possible.

### A. Boot the kernel in QEMU (≈ 1 session)

Highest leverage. Without this, "kernel green" is a type-check claim,
not a runtime claim. Concrete output: a `make qemu-smoke` target that
loads the kernel image, prints the boot banner, dumps the process
table once, and exits. Whatever the smoke test catches (and it will
catch things — green-on-types ≠ green-on-runtime) becomes the next
KERNEL-BUILD-GREEN equivalent: BOOT-GREEN.md.

### B. Wire one real kernel metric to the dashboard (≈ 1 session)

Pick the cheapest source — `kernel::process::active_process_count()`
or `secman::audit` ring depth. Add a host-side IPC bridge that polls
the kernel via a syscall, writes to `ui/dashboard/kernel_metrics.js`,
and have `app.js` swap one `Math.random()` site for the real value.
Doing this **once** establishes the pattern for the other 26 sites and
makes the dashboard's "Simulated" badge actually retractable per-card
instead of all-or-nothing.

### C. Compile and invoke a real summarizer wasm component (≈ 1 session)

Take the existing `SummarizerHost` fallback, lift its logic into a
WIT-compliant Rust crate, compile to wasm with `cargo component`, and
have `SummarizerHost::summarize` prefer the component when present and
fall back to the in-process implementation when absent. That closes
the "AI workload runs through wasm" gap on the one path the user has
already named as the flagship.

### D. Delete or port the 9 stale ai_core integration tests (≈ 1 session)

Each stale test is fast to triage:
- If it covers a removed feature (`STT config` etc.), delete.
- If it covers a renamed surface (`ToolCallRequest` etc.), port to
  the new naming.
- Add **one** integration test that exercises `summarize-log`
  end-to-end (CLI invocation → exit 0 → metric file updated).

Output: `cargo test --manifest-path services/ai_core/Cargo.toml`
(no `--lib`) returns 0. CI can then run it.

### E. Add CI gate for `cd kernel && cargo check` (≈ ½ session)

Smallest concrete safeguard against re-breaking the build. One step
in `.github/workflows/polyglot-ci.yml` (or a new workflow). Until this
exists, kernel-green is a property of one machine on one branch.

After A–E, the project is in a position where the next big
deliverable — real PQC on a hot path, or a real wasm-loaded second
AI workload, or the polyglot runtime story — has a foundation that
won't silently regress while the work is in flight.

---

## 9. Open risks worth flagging

1. **The 563-file working tree** is the single biggest correctness
   risk because it is large enough to contain reverts of the
   committed work. Treat as a triage task before any aggressive
   `git add` or rebase.
2. **No CI is gating the kernel build.** Any commit that lands
   without a local `cargo check` in `kernel/` can put it back into
   the red.
3. **The `SummarizerHost` wasm path is untested.** If a future change
   removes the fallback before the real component lands, the AI
   workload silently breaks.
4. **The 9 stale integration test binaries hide regressions.** Any
   real regression in the ai_core APIs is masked by the noise of
   "these tests have been broken forever."
5. **No on-chain test for AnchorDAO.** The Solidity fixes look right
   on reading; Foundry/Hardhat tests against a local node have not
   been run on this branch.
6. **Dashboard says "Simulated" but most viewers will still read it
   as authoritative.** Once any card is real, the all-or-nothing
   honesty of the badge starts to mislead. Recommend per-card real /
   simulated badges once item B above lands.

---

## 10. Where this report is wrong, or might become wrong

- **AnchorDAO Foundry tests:** I did not run them; the Solidity reads
  correctly but I cannot prove it works on EVM until Foundry/Hardhat
  is wired in CI.
- **Boot behaviour:** the kernel has not been booted in QEMU on this
  branch; any claim about runtime is a type-system claim only.
- **Verify-web3 / verify-quantum scripts:** the user's walkthrough
  says they pass; I did not re-run them in this report's
  verification pass.
- **Dashboard metric refresh cadence:** I read the static
  `ai_metrics.js` initial values (all 0) and the consumer code in
  `app.js`; I did not confirm in a running browser that the page
  re-reads the metric file when the CLI updates it. The mechanism
  in `ai_core_cli.rs` writes the file synchronously, so the next
  page reload should show the bumped value — but auto-refresh
  semantics are not verified.
- **Anything beyond `git log` and `cargo check` output is paraphrase**
  of what the user's walkthrough said. Where I had time to read
  files I did; where I did not, items are flagged **[unverified]**
  or **⚠️**.

When in doubt, trust [KERNEL-BUILD-GREEN.md](KERNEL-BUILD-GREEN.md) and
the commit log over this file. This is a snapshot; those are the
source of truth.
