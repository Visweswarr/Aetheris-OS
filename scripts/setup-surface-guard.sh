#!/bin/bash

set -e

echo "🔒 Polymera OS Surface Guard Setup"
echo "=================================="

if ! command -v bazel &> /dev/null; then
    echo "❌ Bazel not found. Please install Bazel first."
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust first."
    exit 1
fi

echo "🔄 Building surface guard tool..."
if ! bazel build //tooling/guard:surface_check; then
    echo "❌ Failed to build surface guard tool"
    exit 1
fi

echo "🔄 Regenerating golden surfaces..."
if ! bazel run //tooling/abi:gen; then
    echo "❌ Failed to regenerate ABI surfaces"
    exit 1
fi

if [ -f "policy/compile.sh" ]; then
    echo "🔄 Regenerating policy WASM..."
    if ! (cd policy && ./compile.sh); then
        echo "❌ Failed to regenerate policy WASM"
        exit 1
    fi
fi

echo "🔍 Calculating initial golden hashes..."
if ! bazel run //tooling/guard:surface_check -- --generate; then
    echo "❌ Failed to generate initial hashes"
    exit 1
fi

echo "✅ Surface guard setup complete!"
echo ""
echo "Next steps:"
echo "1. Review SURFACE.lock.json for generated hashes"
echo "2. Install git hook: chmod +x .githooks/pre-commit && git config core.hooksPath .githooks"
echo "3. Test: bazel run //tooling/guard:surface_check"
echo "4. Commit: git add SURFACE.lock.json && git commit -m 'Initialize surface guard'"
echo ""
echo "The system will now protect all critical interfaces from accidental changes."
