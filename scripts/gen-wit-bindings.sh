#!/usr/bin/env bash
# gen-wit-bindings.sh — generate per-language bindings from wit/ packages.
#
# This script is the canonical entry point for Initiative 2 binding generation.
# Run from repo root. Requires the toolchains listed in wit/README.md.
#
# Outputs:
#   bindings/rust/<package>/         — wit-bindgen-rust output
#   bindings/go/<package>/           — wit-bindgen-go output (TinyGo)
#   bindings/ts/<package>/           — jco transpile output
#   bindings/python/<package>/       — componentize-py types output
#
# Each binding directory is overwritten in place; do not hand-edit generated files.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WIT_ROOT="$ROOT/wit"
OUT_ROOT="$ROOT/bindings"
TOOL_ROOT="${POLYMERA_TOOL_ROOT:-$ROOT/.polymera-tools}"

if [[ ! -d "$WIT_ROOT" ]]; then
    echo "fatal: $WIT_ROOT not found" >&2
    exit 1
fi

tool_path() {
    local name="$1"
    local candidates=(
        "$TOOL_ROOT/bin/$name"
        "$TOOL_ROOT/node/node_modules/.bin/$name"
        "$TOOL_ROOT/python/bin/$name"
        "$TOOL_ROOT/python/Scripts/$name.exe"
    )
    for candidate in "${candidates[@]}"; do
        if [[ -x "$candidate" || -f "$candidate" ]]; then
            echo "$candidate"
            return 0
        fi
    done
    command -v "$name" 2>/dev/null || true
}

require() {
    local resolved
    resolved="$(tool_path "$1")"
    if [[ -z "$resolved" ]]; then
        echo "skip: $1 not installed — $2 bindings will not be generated" >&2
        return 1
    fi
    echo "$resolved"
    return 0
}

generate_rust() {
    local wit_bindgen
    wit_bindgen="$(require wit-bindgen "Rust")" || return 0
    for pkg_dir in "$WIT_ROOT"/polymera/*/0.1.0; do
        [[ -d "$pkg_dir" ]] || continue
        local pkg
        pkg="$(basename "$(dirname "$pkg_dir")")"
        local out="$OUT_ROOT/rust/polymera-$pkg"
        mkdir -p "$out/src"
        "$wit_bindgen" rust "$pkg_dir" --out-dir "$out/src"
        echo "rust: $out"
    done
}

generate_go() {
    local wit_bindgen_go
    local wit_bindgen
    wit_bindgen_go="$(tool_path wit-bindgen-go)"
    wit_bindgen="$(tool_path wit-bindgen)"
    if [[ -z "$wit_bindgen_go" && -z "$wit_bindgen" ]]; then
        echo "skip: wit-bindgen-go/wit-bindgen not installed — Go bindings will not be generated" >&2
        return 0
    fi
    for pkg_dir in "$WIT_ROOT"/polymera/*/0.1.0; do
        [[ -d "$pkg_dir" ]] || continue
        local pkg
        pkg="$(basename "$(dirname "$pkg_dir")")"
        local out="$OUT_ROOT/go/polymera_$pkg"
        mkdir -p "$out"
        if [[ -n "$wit_bindgen_go" ]]; then
            "$wit_bindgen_go" generate "$pkg_dir" --out "$out"
        else
            "$wit_bindgen" tiny-go "$pkg_dir" --out-dir "$out"
        fi
        echo "go: $out"
    done
}

generate_ts() {
    local jco
    jco="$(require jco "TypeScript")" || return 0
    for pkg_dir in "$WIT_ROOT"/polymera/*/0.1.0; do
        [[ -d "$pkg_dir" ]] || continue
        local pkg
        pkg="$(basename "$(dirname "$pkg_dir")")"
        local out="$OUT_ROOT/ts/polymera-$pkg"
        mkdir -p "$out"
        "$jco" types "$pkg_dir" --out-dir "$out"
        echo "ts: $out"
    done
}

generate_python() {
    local componentize_py
    componentize_py="$(require componentize-py "Python")" || return 0
    for pkg_dir in "$WIT_ROOT"/polymera/*/0.1.0; do
        [[ -d "$pkg_dir" ]] || continue
        local pkg
        pkg="$(basename "$(dirname "$pkg_dir")")"
        local out="$OUT_ROOT/python/polymera_$pkg"
        mkdir -p "$out"
        "$componentize_py" --wit-path "$pkg_dir" bindings "$out"
        echo "python: $out"
    done
}

validate() {
    local wasm_tools
    if wasm_tools="$(require wasm-tools "WIT validation")"; then
        for pkg_dir in "$WIT_ROOT"/polymera/*/0.1.0; do
            [[ -d "$pkg_dir" ]] || continue
            "$wasm_tools" component wit "$pkg_dir" > /dev/null
            echo "validated: $pkg_dir"
        done
    fi
}

main() {
    validate
    generate_rust
    generate_go
    generate_ts
    generate_python
    echo "done. bindings in $OUT_ROOT"
}

main "$@"
