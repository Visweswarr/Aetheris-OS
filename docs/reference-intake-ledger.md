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
| 2026-05-21 | `services/ai_core/src/runtime/heg.rs` | `C:\Users\reddy\Downloads\OS Evolution_ AI, Web3, Metaverse, Quantum.pdf` | User-provided design document | Reimplemented behavior | Heterogeneous Execution Graph idea used as design input only. No PDF text or external OS code copied into Polymera source. |
| 2026-05-21 | `services/ai_core/src/runtime/backend.rs` | `nebulet/nebulet` | MIT | Design input only | Ring-0 Wasm/user-mode isolation concept. Strictly no code copied in this phase; informed the design for the RuntimeBackend trait boundary. |
| 2026-05-21 | `services/ai_core/src/runtime/backend.rs` | `substratum-labs/mini-castor` | Apache-2.0 | Design/reimplemented only | Syscall proxy pipeline, checkpoint/replay, budgets, and HITL patterns. Reimplemented structural logic matching Mini-Castor design constraints; strictly no code copied in this phase. |
| 2026-05-21 | Design input only | `viralcode/tensoragentos` | BUSL-1.1 (non-production) | Not copied (BUSL restriction) | WebMCP/browser/agent-shell ideas reviewed for high-level desktop-OS context. Strictly no code copied in this phase. |
| 2026-05-21 | Design input only | `agiresearch/AIOS` | Mixed / Unverified | Design input (unless local license verified) | Microkernel/SDK boundary, scheduling queues, and computer-use abstractions analyzed. Strictly no code copied in this phase. |
| 2026-05-21 | `configs/ai/model-registry.toml` | `HuggingFaceTB/SmolLM2-135M-Instruct` | Apache-2.0 | Metadata reference only | Model card used to record a reviewed small local model candidate. No model weights, source code, or generated artifacts copied into Polymera. |
| 2026-05-21 | `configs/ai/model-registry.toml` | `QuantFactory/SmolLM2-135M-Instruct-GGUF` | Apache-2.0 | Metadata reference only | GGUF model card used to record a local llama.cpp/Ollama candidate. No model weights copied; operators must explicitly download and verify artifacts. |

> [!NOTE]
> **Consolidation Record**: During this release consolidation phase, all reference materials were utilized strictly as high-level architectural inputs. No source code or assets from any external repository were copied, cloned, or direct-imported into the Polymera OS codebase.
