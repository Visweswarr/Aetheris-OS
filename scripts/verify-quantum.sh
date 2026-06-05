#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

step() {
  printf '==> %s\n' "$1"
}

if [[ "${SKIP_CARGO:-0}" != "1" ]]; then
  step "cargo check polymera-crypto"
  cargo check --manifest-path crypto/Cargo.toml --features full
  step "cargo test polymera-crypto KAT/smoke tests"
  cargo test --manifest-path crypto/Cargo.toml --features full --lib
fi

step "kernel IPC fixed-pattern session-key guard"
if rg --line-number '0xAA|Simple pattern for demonstration|Keys should be deterministic' kernel/src/ipc/pqc_helpers.rs; then
  echo "deterministic fake session key remains" >&2
  exit 1
fi

step "ZK forced-true circuit guard"
if [[ -d zk ]] && find zk -name '*.nr' -type f -print0 | xargs -0 -r rg --line-number '=\s*true\s*;'; then
  echo "forced-true Noir constraint remains" >&2
  exit 1
fi

step "verified-crypto scaffold guard"
test -f c/crypto/verified/README.md

echo "Quantum/PQC verification completed"
