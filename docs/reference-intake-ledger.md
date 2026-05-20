# Reference Intake Ledger

Polymera imports reference-OS ideas only when the source is repo-owned or
permissive-license compatible. GPL/proprietary/vendor OS code is research input
only and must not be copied into Polymera source.

| Date | Target | Source | License | Intake mode | Notes |
| --- | --- | --- | --- | --- | --- |
| 2026-05-20 | `services/ai_core/src/browser_assist.rs` | `reference-os/services/ai/src/browser_assist.rs` | Repo-owned MIT-compatible | Copied and extended | Deterministic local summarization/classification imported into AI Core built-in tools. Extended with reasons and tests. |
| 2026-05-20 | `services/ai_core/src/cognitive.rs` | `services/ai/src/aicore.rs`, `services/ai/src/learning.rs` | Repo-owned MIT-compatible | Integrated via path dependency | AI Core calls the existing deterministic cognitive planner and LTM instead of duplicating it. |
| 2026-05-20 | `services/ai_core/src/audit.rs` | `reference-os/services/security/src/audit.rs` | Repo-owned MIT-compatible | Reimplemented behavior | Tamper-evident hash-chain concept reimplemented without copying the HMAC file logger. |
| 2026-05-20 | `services/ai_core/src/privacy.rs` | `reference-os/services/privacy/src/dp.rs` | Repo-owned MIT-compatible | Reimplemented behavior | Seeded Laplace DP helper added for off-device metric export tests. |
| 2026-05-20 | Design input only | `OS-REF/Reference OS/` | Mixed/unknown/vendor | Not copied | Treat as research/vendor library. Direct code import requires separate license review. |
