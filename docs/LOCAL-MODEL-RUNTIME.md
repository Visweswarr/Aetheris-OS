# Polymera Local Model Runtime

This release slice makes model-backed runtime execution explicit and local-only.
The default backend remains deterministic and daemon-free; model backends are
operator-selected and fail closed when no local service is reachable.

## Deterministic mode

Use this for CI, demos without model weights, and release gates:

```powershell
cargo run --manifest-path C:\polymera-os\services\ai_core\Cargo.toml --bin ai_core_cli -- runtime-run --prompt "Explain Polymera" --backend deterministic
```

The response includes a HEG plan id, placement summary, input/output hashes,
backend metadata, and `remote_execution=false`.

## Reviewed model registry

Tracked model metadata lives in:

```text
C:\polymera-os\configs\ai\model-registry.toml
```

The registry contains metadata only. Polymera does not commit model weights.
Operators must download weights explicitly and verify hashes before production
use. Entries marked `reviewed` are permitted by the local runtime gate; unknown,
blocked, or unreviewed registry ids fail closed.

## llama.cpp server mode

Start a local OpenAI-compatible llama.cpp server using a reviewed GGUF model,
then run:

```powershell
cargo run --manifest-path C:\polymera-os\services\ai_core\Cargo.toml --bin ai_core_cli -- runtime-run `
  --prompt "Explain Polymera in one paragraph" `
  --backend llama-cpp `
  --model smollm2-135m-instruct-q4 `
  --endpoint http://127.0.0.1:8080/v1 `
  --dashboard-metrics C:\polymera-os\ui\dashboard\ai_metrics.js
```

Only `http://localhost`, `http://127.0.0.1`, and `http://[::1]` endpoints are
accepted. Any remote host or HTTPS API URL is rejected before execution.

## Ollama mode

Start Ollama locally with a reviewed or explicitly named local model, then run:

```powershell
cargo run --manifest-path C:\polymera-os\services\ai_core\Cargo.toml --bin ai_core_cli -- runtime-run `
  --prompt "Explain Polymera in one paragraph" `
  --backend ollama `
  --model hf.co/QuantFactory/SmolLM2-135M-Instruct-GGUF:Q4_K_M `
  --endpoint http://127.0.0.1:11434 `
  --dashboard-metrics C:\polymera-os\ui\dashboard\ai_metrics.js
```

If the daemon is unavailable or the model is not loaded, the command exits
non-zero and increments the backend/model failure counters.

## Live test gate

Daemon-free CI must use the normal test suite. To run the optional live model
test manually:

```powershell
$env:POLYMERA_RUN_LOCAL_MODEL_TESTS='1'
$env:POLYMERA_LIVE_MODEL_BACKEND='ollama'
$env:POLYMERA_LIVE_MODEL_ENDPOINT='http://127.0.0.1:11434'
$env:POLYMERA_LIVE_MODEL_ID='hf.co/QuantFactory/SmolLM2-135M-Instruct-GGUF:Q4_K_M'
cargo test --manifest-path C:\polymera-os\services\ai_core\Cargo.toml --test runtime_tests live_local_model -- --ignored
```

Use `POLYMERA_LIVE_MODEL_BACKEND=llama-cpp` and a matching endpoint for
llama.cpp server tests.

## Dashboard metrics

`runtime-run --dashboard-metrics <path>` writes real AI runtime counters:

- `ai_runtime_backend_executions_total`
- `ai_runtime_backend_errors_total`
- `ai_runtime_model_attempts_total`
- `ai_runtime_model_success_total`
- `ai_runtime_model_failures_total`
- `ai_runtime_model_tokens_total`
- `ai_runtime_model_last_latency_ms`

The global dashboard still remains labeled simulated. Only AI runtime cards
backed by `ui/dashboard/ai_metrics.js` should be treated as real.

## Troubleshooting

- `runtime backend endpoint must be local-only`: use `localhost`, `127.0.0.1`,
  or `[::1]`; remote inference APIs are intentionally blocked.
- `model registry entry ... not found`: use a reviewed registry id or an
  explicit backend-native model name accepted by the selected local daemon.
- `local llama.cpp server unavailable` / `local Ollama server unavailable`:
  start the daemon and load the model before selecting the backend.
- Metrics file did not change: pass `--dashboard-metrics <path>` or inspect the
  default `C:\polymera-os\ui\dashboard\ai_metrics.js`.
