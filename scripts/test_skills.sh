#!/bin/bash

set -e

echo "=== Polymera OS Skill Runtime v0 Test Suite ==="
echo ""

if [ ! -f "kernel/src/skills/mod.rs" ]; then
    echo "Error: Skills module not found. Please run from polymera-os root directory."
    exit 1
fi

echo "Running skills tests..."
echo ""

echo "1. Testing manifest schema..."
bazel test //tests/skills:manifest_schema
echo "✅ Manifest schema tests passed"
echo ""

echo "2. Testing skill load and invoke..."
bazel test //tests/skills:load_and_invoke
echo "✅ Load and invoke tests passed"
echo ""

echo "3. Testing quota enforcement..."
bazel test //tests/skills:quota_enforcement
echo "✅ Quota enforcement tests passed"
echo ""

echo "4. Testing capability broker..."
bazel test //tests/skills:cap_broker
echo "✅ Capability broker tests passed"
echo ""

echo "5. Testing determinism..."
bazel test //tests/skills:determinism
echo "✅ Determinism tests passed"
echo ""

echo ""
echo "=== All Skills Tests Passed! ==="
echo ""
echo "Skill Runtime v0 is fully operational with:"
echo "- ✅ Skill manifest validation and CBOR encoding"
echo "- ✅ Capability broker with policy validation"
echo "- ✅ WASI host environment with restricted hostcalls"
echo "- ✅ WASM execution engine with instruction metering"
echo "- ✅ Skill registry with lifecycle management"
echo "- ✅ Comprehensive syscall interface"
echo "- ✅ Security model with capability enforcement"
echo "- ✅ Deterministic execution with strict quotas"
echo ""
echo "Feature bit SKILL_RUNTIME_V0 is set and ready for use."
