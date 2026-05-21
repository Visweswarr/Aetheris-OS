# Local Model Runtime v1 Release Inventory

Date: 2026-05-21

Purpose: isolate the releasable Local Model Runtime v1 slice from the broader
dirty tree. Files outside this inventory are not part of this phase.

## Runtime/API

- `services/ai_core/Cargo.toml`
- `services/ai_core/src/runtime/backend.rs`
- `services/ai_core/src/runtime/mod.rs`
- `services/ai_core/src/runtime/model_registry.rs`
- `services/ai_core/src/runtime/gguf.rs`
- `services/ai_core/src/runtime/onnx.rs`
- `services/ai_core/src/metrics.rs`
- `services/ai_core/src/lib.rs`
- `configs/ai/model-registry.toml`

## CLI and Tests

- `services/ai_core/src/bin/ai_core_cli.rs`
- `services/ai_core/tests/runtime_tests.rs`
- `services/ai_core/tests/cli_smoke.rs`

## Dashboard and Documentation

- `ui/dashboard/ai_metrics.js`
- `ui/dashboard/app.js`
- `ui/dashboard/index.html`
- `docs/LOCAL-MODEL-RUNTIME.md`
- `docs/reference-intake-ledger.md`
- `STATE-OF-POLYMERA.md`

## Verification Commands

- `cargo check --manifest-path services\ai_core\Cargo.toml`
- `cargo test --manifest-path services\ai_core\Cargo.toml`
- `cargo test --manifest-path services\ai_core\Cargo.toml --test runtime_tests`
- `cargo build --manifest-path services\ai_core\Cargo.toml --bin ai_core_cli`
- `cd kernel && cargo check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify-web3.ps1 -SkipCargo`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify-quantum.ps1 -SkipCargo`
- Scoped `git diff --check` over this inventory.

## Boundary

- No model weights are committed.
- No external reference OS code is copied.
- No kernel files are changed by this phase.
- Local model execution is opt-in and restricted to localhost endpoints.
