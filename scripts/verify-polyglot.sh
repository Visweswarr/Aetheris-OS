#!/usr/bin/env bash
set -euo pipefail

SKIP_EXTERNAL_TOOLS=0
if [[ "${1:-}" == "--skip-external-tools" ]]; then
  SKIP_EXTERNAL_TOOLS=1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOL_ROOT="${POLYMERA_TOOL_ROOT:-$ROOT/.polymera-tools}"
FAILURES=0

tool_path() {
  local name="$1"
  local candidates=(
    "$TOOL_ROOT/bin/$name"
    "$TOOL_ROOT/node/node_modules/.bin/$name"
    "$TOOL_ROOT/python/bin/$name"
  )
  for candidate in "${candidates[@]}"; do
    [[ -x "$candidate" ]] && { echo "$candidate"; return 0; }
  done
  command -v "$name" 2>/dev/null || true
}

run_step() {
  local name="$1"
  shift
  echo "==> $name"
  if "$@"; then
    echo "ok: $name"
  else
    echo "failed: $name" >&2
    FAILURES=$((FAILURES + 1))
  fi
}

require_tool() {
  local name="$1"
  local resolved
  resolved="$(tool_path "$name")"
  if [[ -z "$resolved" ]]; then
    if [[ "$SKIP_EXTERNAL_TOOLS" == "1" ]]; then
      echo "warning: $name not found; skipped because --skip-external-tools is set" >&2
      return 1
    fi
    echo "$name not found. Run scripts/bootstrap-polyglot-tools.sh first." >&2
    return 2
  fi
  echo "$resolved"
}

check_wit() {
  local wasm_tools
  if ! wasm_tools="$(require_tool wasm-tools)"; then
    [[ "$SKIP_EXTERNAL_TOOLS" == "1" ]] && return 0
    return 1
  fi
  while IFS= read -r -d '' pkg; do
    "$wasm_tools" component wit "$pkg" >/dev/null
  done < <(find "$ROOT/wit/polymera" -type d -regex '.*/[0-9]+\.[0-9]+\.[0-9]+' -print0)
}

check_rust() {
  local manifests=(
    services/ai_core/Cargo.toml
    services/wasm_driver/Cargo.toml
    services/supervisor/Cargo.toml
    runtime/examples/hello_component/Cargo.toml
  )
  for manifest in "${manifests[@]}"; do
    cargo check --locked --manifest-path "$ROOT/$manifest"
  done
}

check_go() {
  local modules=(services/fs_go services/net_go go/tooling/ai_core)
  for module in "${modules[@]}"; do
    if [[ -f "$ROOT/$module/go.mod" ]]; then
      (cd "$ROOT/$module" && go test ./...)
    fi
  done
}

check_ts() {
  local packages=(tooling/ts apps/assistant-ui)
  for package in "${packages[@]}"; do
    if [[ -f "$ROOT/$package/package.json" && -d "$ROOT/$package/node_modules" ]]; then
      npm --prefix "$ROOT/$package" run type-check
    else
      echo "warning: skipping $package type-check; package.json or node_modules missing" >&2
    fi
  done
}

check_raw_ffi() {
  command -v rg >/dev/null 2>&1 || { echo "rg is required" >&2; return 1; }
  local allowed='services/ai/src/aicore.rs|services/wallet/src/ffi.rs|services/ngfs/fuse/main.rs'
  local matches
  matches="$(cd "$ROOT" && rg -n 'extern\s+"C"' services -g '!**/target/**' -g '!**/Cargo.lock' || true)"
  if [[ -n "$matches" ]]; then
    local unexpected
    unexpected="$(echo "$matches" | grep -Ev "^($allowed):" || true)"
    if [[ -n "$unexpected" ]]; then
      echo "$unexpected" >&2
      return 1
    fi
  fi
}

run_step "WIT validation" check_wit
run_step "Rust checks" check_rust
run_step "Go checks" check_go
run_step "TypeScript checks" check_ts
run_step "Raw service FFI allowlist" check_raw_ffi

if [[ "$FAILURES" -gt 0 ]]; then
  echo "$FAILURES polyglot verification step(s) failed" >&2
  exit 1
fi

echo "polyglot verification complete"
