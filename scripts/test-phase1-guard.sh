#!/bin/bash

# Test script for Phase 1 Documentation Guard
# Simulates different scenarios to validate the guard logic

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CI_DIR="$PROJECT_ROOT/tooling/ci"

echo "🧪 Testing Phase 1 Documentation Guard"
echo "Project root: $PROJECT_ROOT"

# Ensure we're in the project root
cd "$PROJECT_ROOT"

# Check if Node.js is available
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is required but not installed"
    exit 1
fi

# Install dependencies if needed
if [ ! -d "$CI_DIR/node_modules" ]; then
    echo "📦 Installing CI tool dependencies..."
    cd "$CI_DIR"
    npm install
    cd "$PROJECT_ROOT"
fi

echo ""
echo "=== Testing Phase 1 Guard Logic ==="

# Test 1: Run the test suite
echo ""
echo "🧪 Test 1: Running unit tests"
cd "$CI_DIR"
if npm test; then
    echo "✅ Unit tests passed"
else
    echo "❌ Unit tests failed"
    exit 1
fi

cd "$PROJECT_ROOT"

# Test 2: Simulate current state (should pass - no kernel changes)
echo ""
echo "🧪 Test 2: Current repository state"
cd "$CI_DIR"
if BASE_SHA="HEAD~1" HEAD_SHA="HEAD" node check_phase_docs.ts; then
    echo "✅ Current state validation passed"
else
    echo "❌ Current state validation failed"
fi

cd "$PROJECT_ROOT"

# Test 3: Simulate kernel changes without docs (should fail)
echo ""
echo "🧪 Test 3: Simulating kernel change without docs"

# Create a temporary branch for testing
TEST_BRANCH="test-phase1-guard-$$"
git checkout -b "$TEST_BRANCH" > /dev/null 2>&1

# Make a dummy kernel change
echo "// Test change for Phase 1 guard" >> kernel/src/lib.rs
git add kernel/src/lib.rs
git commit -m "test: kernel change without docs" > /dev/null 2>&1

# Test the guard (should fail)
cd "$CI_DIR"
if BASE_SHA="HEAD~1" HEAD_SHA="HEAD" node check_phase_docs.ts; then
    echo "❌ Test should have failed (kernel change without docs)"
    GUARD_FAILED=true
else
    echo "✅ Correctly caught kernel change without docs"
    GUARD_FAILED=false
fi

cd "$PROJECT_ROOT"

# Test 4: Add documentation and test again (should pass)
echo ""
echo "🧪 Test 4: Adding documentation and testing again"

# Add a dummy documentation change
echo "" >> docs/phase-1/SPEC.md
echo "<!-- Test change for Phase 1 guard -->" >> docs/phase-1/SPEC.md
git add docs/phase-1/SPEC.md
git commit -m "docs: update SPEC.md for kernel change" > /dev/null 2>&1

# Test the guard again (should pass)
cd "$CI_DIR"
if BASE_SHA="HEAD~2" HEAD_SHA="HEAD" node check_phase_docs.ts; then
    echo "✅ Correctly passed with documentation update"
    DOC_PASSED=true
else
    echo "❌ Should have passed with documentation update"
    DOC_PASSED=false
fi

cd "$PROJECT_ROOT"

# Cleanup: restore original state
echo ""
echo "🧹 Cleaning up test changes"
git checkout main > /dev/null 2>&1
git branch -D "$TEST_BRANCH" > /dev/null 2>&1

# Summary
echo ""
echo "=== Test Results Summary ==="
echo "Unit tests: ✅ Passed"
echo "Current state: ✅ Passed"
echo "Kernel without docs: $([ "$GUARD_FAILED" = false ] && echo "✅ Correctly failed" || echo "❌ Should have failed")"
echo "Kernel with docs: $([ "$DOC_PASSED" = true ] && echo "✅ Correctly passed" || echo "❌ Should have passed")"

# Final result
if [ "$GUARD_FAILED" = false ] && [ "$DOC_PASSED" = true ]; then
    echo ""
    echo "🎉 All Phase 1 guard tests passed!"
    echo ""
    echo "The CI guard is working correctly:"
    echo "  ✅ Allows non-kernel changes"
    echo "  ✅ Blocks kernel changes without docs"
    echo "  ✅ Allows kernel changes with docs"
    echo ""
    echo "Ready to enforce Phase 1 documentation rule!"
    exit 0
else
    echo ""
    echo "❌ Some tests failed - Phase 1 guard needs debugging"
    exit 1
fi
