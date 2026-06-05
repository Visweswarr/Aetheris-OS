#!/usr/bin/env bash
set -euo pipefail

INCLUDE_ROADMAP_TOOLS=0
if [[ "${1:-}" == "--include-roadmap-tools" ]]; then
  INCLUDE_ROADMAP_TOOLS=1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="$ROOT/configs/polyglot-toolchain.toml"

if [[ ! -f "$CONFIG" ]]; then
  echo "missing toolchain config: $CONFIG" >&2
  exit 1
fi

version() {
  local key="$1"
  awk -F'"' -v key="$key" '
    $0 == "[versions]" { in_versions=1; next }
    /^\[/ && $0 != "[versions]" { in_versions=0 }
    in_versions && $1 ~ key"[[:space:]]*=" { print $2; exit }
  ' "$CONFIG"
}

TOOL_ROOT="${POLYMERA_TOOL_ROOT:-$ROOT/.polymera-tools}"
CARGO_ROOT="$TOOL_ROOT"
NODE_PREFIX="$TOOL_ROOT/node"
PYTHON_VENV="$TOOL_ROOT/python"
BIN="$TOOL_ROOT/bin"
NODE_BIN="$NODE_PREFIX/node_modules/.bin"
PYTHON_BIN="$PYTHON_VENV/bin"

mkdir -p "$TOOL_ROOT" "$BIN" "$NODE_PREFIX"

install_cargo_tool() {
  local package="$1"
  local pinned="$2"
  echo "installing $package@$pinned into $TOOL_ROOT"
  cargo install "$package" --version "$pinned" --locked --root "$CARGO_ROOT"
}

install_cargo_tool wasm-tools "$(version wasm_tools)"
install_cargo_tool wit-bindgen-cli "$(version wit_bindgen_cli)"
install_cargo_tool cargo-component "$(version cargo_component)"

echo "installing @bytecodealliance/jco@$(version jco) into $NODE_PREFIX"
npm install --prefix "$NODE_PREFIX" "@bytecodealliance/jco@$(version jco)"

if [[ ! -d "$PYTHON_VENV" ]]; then
  python3 -m venv "$PYTHON_VENV"
fi
"$PYTHON_BIN/python" -m pip install --upgrade pip
"$PYTHON_BIN/python" -m pip install "componentize-py==$(version componentize_py)"

if [[ "$INCLUDE_ROADMAP_TOOLS" == "1" ]]; then
  echo "roadmap tools requested"
  echo "Zig $(version zig) is pinned; install from https://ziglang.org/download/ or your platform package cache."
  if command -v opam >/dev/null 2>&1; then
    opam install -y "fstar.$(version fstar)"
  else
    echo "opam not found; skipping F* $(version fstar)"
  fi
fi

cat <<EOF

Add these directories to PATH when needed:
  $BIN
  $NODE_BIN
  $PYTHON_BIN
EOF
