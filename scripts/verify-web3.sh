#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

step() {
  printf '==> %s\n' "$1"
}

step "Web3 manifest dependency guard"
if rg --line-number '\.\./\.\./polymera-crypto|\.\./\.\./intent|\.\./cap-tokens|\.\./intent-kernel' \
  services/wallet/Cargo.toml \
  services/chain/Cargo.toml \
  services/contracts/Cargo.toml \
  services/dao/Cargo.toml; then
  echo "stale Web3 path dependency found" >&2
  exit 1
fi

if [[ "${SKIP_CARGO:-0}" != "1" ]]; then
  step "cargo check wallet"
  cargo check --manifest-path services/wallet/Cargo.toml
  step "cargo check chain"
  cargo check --manifest-path services/chain/Cargo.toml
  step "cargo check dao"
  cargo check --manifest-path services/dao/Cargo.toml
  step "cargo check contracts"
  cargo check --manifest-path services/contracts/Cargo.toml
fi

step "AnchorDAO hybrid signature policy guard"
if [[ -f contracts/AnchorDAO.sol ]] && rg --line-number 'pqcSignature\.length\s*>\s*0|just check that signatures are not empty' contracts/AnchorDAO.sol; then
  echo "weak AnchorDAO PQC policy remains" >&2
  exit 1
fi

step "WIT validation when repo-pinned wasm-tools exists"
WASM_TOOLS="$ROOT/.polymera-tools/bin/wasm-tools"
if [[ -x "$WASM_TOOLS" ]]; then
  while IFS= read -r wit_dir; do
    "$WASM_TOOLS" component wit "$wit_dir" >/dev/null
  done < <(find wit/polymera -name '*.wit' -type f -printf '%h\n' | sort -u)
else
  echo "repo-pinned wasm-tools not installed; run scripts/bootstrap-polyglot-tools.sh first"
fi

echo "Web3 verification completed"
