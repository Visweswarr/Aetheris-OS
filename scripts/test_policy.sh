#!/bin/bash

# Policy Guardrail v0 Test Runner
# This script runs all policy tests and provides a summary

set -e

echo "🔒 Policy Guardrail v0 Test Runner"
echo "=================================="

# Check if we're in the right directory
if [ ! -f "kernel/src/policy/mod.rs" ]; then
    echo "❌ Error: Must run from polymera-os root directory"
    exit 1
fi

echo "📁 Current directory: $(pwd)"
echo "🚀 Running Policy Guardrail tests..."
echo ""

# Run all policy tests
echo "🧪 Running schema stability tests..."
bazel test //tests/policy:schema_stability

echo "🧪 Running engine evaluation tests..."
bazel test //tests/policy:engine_eval

echo "🧪 Running simulation and diff tests..."
bazel test //tests/policy:simulate_diff

echo "🧪 Running fail-open detector tests..."
bazel test //tests/policy:fail_open_detector

echo "🧪 Running integration tests..."
bazel test //tests/policy:integration_intent

echo ""
echo "✅ All Policy Guardrail tests completed successfully!"
echo ""
echo "📊 Test Summary:"
echo "- Schema stability: ✅"
echo "- Engine evaluation: ✅"
echo "- Simulation engine: ✅"
echo "- Fail-open detection: ✅"
echo "- Integration: ✅"
echo ""
echo "🔍 Next steps:"
echo "- Review test output for performance metrics"
echo "- Check fail-open detection results"
echo "- Verify policy integration with other subsystems"
echo "- Run end-to-end tests with Intent Kernel and Skills"
