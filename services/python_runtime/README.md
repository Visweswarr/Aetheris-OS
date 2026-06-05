# Polymera Python Runtime

Trusted CPython boundary for Polymera AI Dev OS automation.

This service is intentionally **not** the untrusted user-script sandbox. Native
CPython runs with trusted system privileges and talks to AI Core through the
same AI Core message shapes used by Protobuf-over-PolyBus. Untrusted Python
belongs in a future Pyodide/WASM Component Model runtime.

## Status

Phase 1 scaffold:

- exposes a `polymera_native` PyO3 module;
- builds AI Core-compatible JSON envelopes for plan and tool operations;
- pairs with `bindings/python/polymera` for the public Python SDK.

The transport endpoint is selected by `POLYMERA_AI_CORE_ENDPOINT`. Until the
AI Core IPC server is made active on all platforms, the native module keeps the
transport explicit instead of silently pretending an IPC call succeeded.
