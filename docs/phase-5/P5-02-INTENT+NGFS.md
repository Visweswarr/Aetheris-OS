# Phase 5 — Intent Client + NGFS Integration (Design Notes)

Status: Draft (initial)

Scope
- Replace StubIntentBus with a userland client that sends CBOR envelopes `{intent, args, captoken, nonce}` to `INTENT_SOCKET`.
- Swap LTM NDJSON with NGFS KV/segment API and implement snapshot replay in CLI.

1) Intent Client v0
- Envelope: fields as above, canonical JSON → CBOR. Nonce prevents replay.
- Transports:
  - to-file://<path>: append line-delimited CBOR (dev/testing)
  - unix://<socket>: AF_UNIX stream (Linux/macOS)
  - pipe://\\.\pipe\name: Named pipe (Windows)
- Responses: `{"kind":"preview|commit|error","payload":<cbor>}`
- Security: CapToken v2 passed as bytes (JWT/CBOR); kernel validates scopes.

2) NGFS Snapshot & Replay
- Layout: `$NGFS_ROOT/snapshots/<id>/`
  - `plan.cbor`, `wm.cbor`, `metrics.cbor`, `events.cbor`, `rng.seed`, `models.lock`
- ID: `blake3(canonical_bundle)`; timestamps in 90kHz.
- CLI flags: `--ngfs-root`, `--replay <id>`, `--dump-plan`.
- Determinism test: record → replay → compare `events.cbor` and `plan.cbor` hashes.

3) CapToken v2 Verification
- JWKS/CBOR verifier in userland optional; kernel is source of truth.
- Enforce scopes: `ai.memory.read/write`, `intent.emit/read`, `ngfs.snapshot.read/write`.
- Audits: append to NGFS log (append-only, fsync).

4) CI additions
- Add macOS runner; Go/TS/Python smoke tests; Criterion perf gates.

Transports

CBOR Transport
- Sockets (unix://, pipe://): u32 BE length prefix followed by CBOR bytes (request/response), one message per request.
- File fallback (to-file://): CBOR bytes per line (CBORL), suitable for debugging only.
- Envelope schema (CBOR): { v: 1, captoken: bstr, payload: bstr }
- Response schema (CBOR): { status: "ok"|"deny"|"error", preview_id?: tstr, commit_id?: tstr, message?: tstr }
- Echo server
  - UNIX: cargo build --manifest-path tools/intent_echo_server/Cargo.toml && ./target/debug/intent-echo-server --unix /tmp/aetheris-intent.sock
  - Windows: cargo build --manifest-path tools/intent_echo_server/Cargo.toml; .\target\debug\intent-echo-server.exe --pipe \\.\pipe\aetheris-intent
- Client usage
  - export INTENT_SOCKET=unix:///tmp/aetheris-intent.sock && aicore submit "unix socket test"
  - $env:INTENT_SOCKET="pipe://\\.\pipe\aetheris-intent"; .\aicore submit "named pipe test"
- Framing: sockets use length-prefixed (u32 BE) CBOR (Rust), while Go/TS temporarily send length-prefixed JSON (to be unified to CBOR).
- File fallback: to-file://... remains line-delimited and is suitable for local debugging.

Next Steps
- Unify Go/TS framing to CBOR (fxamacker/cbor for Go; node-cbor for TS) and parse responses.
- Add NGFS KV module and wire LTM swap.
- Extend Go CLI flags and surface replay/dump-plan.
