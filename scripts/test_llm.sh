#!/bin/bash

# LLM Adapter v0 Test Runner
# Runs all LLM-related tests and provides a summary

set -e

echo "🧠 Running LLM Adapter v0 Tests..."
echo "=================================="

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run a test and count results
run_test() {
    local test_name="$1"
    local test_target="$2"
    
    echo "Running $test_name..."
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if bazel test "$test_target" --test_output=all; then
        echo "✅ $test_name passed"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo "❌ $test_name failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    echo ""
}

# Run all LLM tests
run_test "Schema Stability" "//tests/llm:schema_stability"
run_test "Redaction Rules" "//tests/llm:redaction_rules"
run_test "Null Backend Streaming" "//tests/llm:null_stream"
run_test "Quota Enforcement" "//tests/llm:quotas"
run_test "Policy Integration" "//tests/llm:policy_integration"
run_test "Event Integration" "//tests/llm:events_integration"

# Summary
echo "=================================="
echo "🧠 LLM Adapter v0 Test Summary"
echo "=================================="
echo "Total tests: $TOTAL_TESTS"
echo "Passed: $PASSED_TESTS"
echo "Failed: $FAILED_TESTS"

if [ $FAILED_TESTS -eq 0 ]; then
    echo "🎉 All LLM tests passed!"
    exit 0
else
    echo "💥 $FAILED_TESTS test(s) failed!"
    exit 1
fi

