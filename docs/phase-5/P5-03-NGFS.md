# Phase 5 — NGFS KV + Deterministic Replay

Status: Draft (Phase 5-C)

## KV Backend Layout
- `$NGFS_ROOT/kv/<key>.cbor` — arbitrary CBOR values (e.g., plans/<id>)
- `$NGFS_ROOT/snapshots/<id>.cbor` — full snapshot bundle
- `$NGFS_ROOT/plans/<id>.json` — convenience JSON dump from CLI

## Snapshot Schema (CBOR)
A CBOR map containing:
- `cfg` — CognitiveCoreConfig
- `wm` — WorkingMemoryState
- `metrics` — MetaMetrics
- Deterministic hash: `id = blake3(canonical_bytes(cfg, wm, metrics))`

## CLI Usage

Record
```bash
export NGFS_ROOT=./data/ngfs
cargo build -p aetheris-ai --features phase5 --verbose
go build -o aicore bindings/go/aicore.go
./aicore init
./aicore submit "record goal"
# prints snapshot_id or use plans/<id>.json for plan dump
```

Dump plan
```bash
cat ./data/ngfs/plans/<id>.json
```

Replay
```bash
./aicore replay --replay <id> --ngfs-root ./data/ngfs
```

## CI Determinism Tests
- Build aicore and run one submit to produce a snapshot id.
- Replay the id and verify success; compare `blake3` of the snapshot bundle where applicable.
- Jobs: linux-determinism (record+replay+artifact), macos-build, windows-build.

## Future Work
- Unify Go/TS framing to CBOR and parse transport responses.
- Add NGFS attestation and CapToken scope verification on snapshot access.
- Add performance gates on record/replay duration and I/O sizes.
