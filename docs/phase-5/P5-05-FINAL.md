# Phase 5 — Cognitive Core v1.0 (FINAL)

## Overview
Deterministic Cognitive Core (reasoning/WM/LTM), Intent Client (unix/pipe/to-file), NGFS KV + snapshot/replay, CapToken v2 enforcement at syscalls, CBOR transports, strict determinism CI, and perf gates.

## Quickstart
```bash
export NGFS_ROOT=./data/ngfs
export INTENT_SOCKET=unix:///tmp/aetheris-intent.sock   # or pipe://\\.\pipe\aetheris-intent
export AICORE_CAPTOKEN=policy/captoken_v2/dev/test.token
cargo build -p aetheris-ai --features phase5 --verbose
go build -o aicore go/aicore.go
./aicore init
# Human-friendly (prints snapshot_id=<id>)
./aicore submit "hello" --ngfs-root $NGFS_ROOT
# Machine (prints strict JSON): {"snapshot_id":"<id>","plan_path":"$NGFS_ROOT/plans/<id>.json"}
./aicore submit "hello" --ngfs-root $NGFS_ROOT --json | jq -r .snapshot_id
./aicore replay --replay <id> --ngfs-root $NGFS_ROOT
```

## Determinism
- Snapshot bundle = CBOR {cfg, wm, metrics}
- id = blake3(bundle), enforced by CI:
  - tools/ngfs_hashcheck compares file hash with id
  - replay → resnapshot equality enforced

## Transports
- Sockets: u32 BE length prefix + CBOR
- File fallback: CBORL (one CBOR per line)

## CapToken v2 (syscall envelope)
- Envelope: {v:1, captoken:bstr, payload:bstr}; require intent.emit
- NGFS guards: ngfs.snapshot.read/write
- Audit: $NGFS_ROOT/audit/captoken/YYYYMMDD.cborl

## CI Matrix
- p5-02 intent transports, p5-03 ngfs replay (strict), p5-04x captoken, p5-05 cbor transports, p5-06 perf gates

## Acceptance Checklist
- [ ] Submit → snapshot id printed
- [ ] `./aicore submit ... --json` prints a single valid JSON object and nothing else
- [ ] Existing flags `--dump-plan`, `--replay`, `--ngfs-root` continue to work
- [ ] Hash of snapshot equals id (hashcheck OK)
- [ ] Replay succeeds; resnapshot equals original id
- [ ] Intent preview with valid token = ok; missing scope = deny
- [ ] Perf gate passes (submit≤250ms p95; snapshot write≤100ms; read≤60ms)
- [ ] Agentic Browser: Plan viewer UI renders deterministically given the same plan JSON
- [ ] Agentic actions require explicit approval; undo available; action log persisted as CBORL
- [ ] Windows named pipe transport tolerates brief server creation races (retry)
