# Runtime Truth Working Tree Inventory

Date: 2026-05-21

Purpose: record the dirty-tree shape before the runtime truth consolidation
slice, without reverting or folding unrelated OS-wide work into this phase.

## Summary

`git status --short` reported 592 dirty entries before this slice's final
verification pass. The tree is intentionally broad and includes prior kernel,
polyglot, Web3, dashboard, CI, and reference-OS work.

## Top-Level Groups

| Group | Dirty entries | Handling in this slice |
| --- | ---: | --- |
| `services/` | 125 | Touch only `services/ai_core/src/runtime/*`, `services/ai_core/tests/runtime_tests.rs`, and existing AI Core CLI metrics path if needed. |
| `kernel/` | 124 | Do not edit; run verification only. |
| `.github/` | 80 | Do not edit. |
| `tooling/` | 54 | Do not edit. |
| `docs/` | 35 | Touch only `docs/reference-intake-ledger.md` and release-state wording. |
| `scripts/` | 29 | Do not edit; run requested verification scripts only. |
| `go/` | 20 | Do not edit. |
| `crypto/` | 12 | Do not edit. |
| `ui/` | 10 | Touch only dashboard AI metric truth labels/contracts if needed. |
| other top-level paths | 103 | Preserve as unrelated dirty work. |

## Scoped Files For Runtime Truth

- `services/ai_core/src/runtime/backend.rs`
- `services/ai_core/src/runtime/mod.rs`
- `services/ai_core/src/runtime/gguf.rs`
- `services/ai_core/src/runtime/onnx.rs`
- `services/ai_core/tests/runtime_tests.rs`
- `docs/reference-intake-ledger.md`
- `STATE-OF-POLYMERA.md`
- `ui/dashboard/ai_metrics.js`
- `artifacts/ai_core/runtime-truth-working-tree-inventory.md`

No reference OS source is copied in this slice.
