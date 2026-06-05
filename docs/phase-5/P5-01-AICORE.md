# Phase 5 — Cognitive Core (AICORE)

This document describes the Aetheris OS Cognitive Core implementation added in Phase 5.

Status: v1.0 (initial)

Goals
- Provide a deterministic Cognitive Core integrating reasoning, memory, planning, and intent orchestration.
- Maintain ≤10% performance regression from Phase 4.
- Secure operations via CapToken v2 scope checks (enforced at policy layer for now).

Components
- Reasoning Kernel
  - A directed acyclic reasoning graph of pluggable nodes.
  - Deterministic execution order using seeded tiebreakers and sorted edges.
- Working Memory
  - Bounded, deterministic short-term cache storing key/value context with 90kHz timestamps.
  - Snapshot/restore for NGFS replay.
- Long-Term Memory (LTM)
  - NGFS-anchored interface with file-backed NDJSON stub at data/aicore_ltm.ndjson.
- Planner Bridge
  - Uses existing planner::StubTaskPlanner under feature "phase5".
  - Generates plans deterministically (seeded via config.planner.random_seed) and persists summaries to LTM.
- Intent Integration
  - IntentBus trait with a StubIntentBus adapter.
  - Emits planning submissions and events; ready to swap for Event Fabric v0 / Intent Kernel v0 client.
- Meta-learning Hooks
  - Deterministic metrics (plans_generated, steps_executed, success/failure) to inform future strategy selection.

Determinism
- All stateful collections use BTreeMap or sorted vectors.
- Random tie-breaks use a seeded scheme based on blake3(seed||node_id).
- End-to-end snapshot/restore encoded in CBOR for replay via NGFS.

Security (CapToken v2)
- Capability checks applied through AiPolicy (ai:plan.generate, ai:infer.*).
- Audit logging hooks are available in policy.rs; integrate with CapToken v2 agent when available.

Integration Points
- Intent Kernel v0 / Event Fabric v0
  - StubIntentBus publishes events and simulates submissions.
  - Replace StubIntentBus with a kernel syscall client that serializes intent YAML/CBOR defined in intent/schema/intent.yaml.
  - New Intent Client: services/ai/src/intent_client.rs supports CBOR v1
    envelopes over INTENT_SOCKET with optional CapToken; includes a
    convenience send_intent(name, cbor_params) that is non-blocking when
    used via special goals (e.g., `intent:system.notification {"text":"hi"}`).
- NGFS
  - LongTermMemory currently file-backed (NDJSON). Swap append/read methods to NGFS APIs when exposed.

APIs
- Rust: services/ai/src/aicore.rs
  - CognitiveCore::new(cfg)
  - CognitiveCore::submit_goal(goal)
  - CognitiveCore::reason_once(input)
  - CognitiveCore::snapshot()/restore()
  - WorkingMemory/LongTermMemory helpers
- C FFI: c/aicore.h, c/aicore.c
  - aicore_init_default()
  - aicore_submit_goal_json(goal_json, out_buf, out_len)
- Go CLI: go/aicore.go
  - aicore init | submit "goal"
- TS Bridge: ui/aicore_bridge.ts
  - submitGoal(goal: string)
- Python Validator: tools/aicore_validator.py
  - Validates determinism by calling CLI twice for the same goal.

CLI Features
- Go CLI (go/aicore.go)
  - --interactive: start a REPL (single init; multiple goals, sequentially)
  - --ngfs-root PATH: set NGFS root for snapshots/LTM
  - --captoken FILE: attach CapToken when emitting intents
  - --dump-plan: print full plan JSON for a submit
  - --json: print strict single JSON object for machine use

Build & CI
- Workflow at .github/workflows/p5-01-aicore.yml builds and tests the Rust AI crate with feature "phase5" on Windows and Linux.

Performance
- Reasoning graph execution is O(N+E) with deterministic ordering.
- Working memory operations are O(n) for put/get in the worst case; capacity is small and bounded (default 256).
- Planner bridge reuses existing planner with deterministic settings.
- Benchmarking: Criterion benches under services/ai/benches/ measure goal
  planning, snapshot write/read. Target regression ≤10% vs previous baselines.
  Run locally with `cargo bench --all-features` (or project-specific `-p`).

Future Work
- Replace StubIntentBus with real Intent Kernel v0 client via syscalls (kernel/syscall/handlers/intent.rs).
- Swap LTM stub for NGFS API once exposed.
- Add meta-learning adaptation loops gated behind deterministic modes.
- Add more built-in reasoning nodes and a registry of system intents/tools.
