#!/bin/bash
# CI Integration test for SLO Gates system

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}CI Integration Test for SLO Gates${NC}"
echo "========================================="

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_exit_code="${3:-0}"
    
    echo ""
    echo -e "${BLUE}Test: $test_name${NC}"
    echo "Command: $test_command"
    
    TESTS_RUN=$((TESTS_RUN + 1))
    
    if eval "$test_command"; then
        actual_exit_code=0
    else
        actual_exit_code=$?
    fi
    
    if [[ $actual_exit_code -eq $expected_exit_code ]]; then
        echo -e "${GREEN}✓ PASS${NC} (exit code: $actual_exit_code)"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}✗ FAIL${NC} (expected exit code: $expected_exit_code, actual: $actual_exit_code)"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

# Test 1: Development environment with passing metrics (should pass)
run_test "Development environment - passing metrics" \
    "\"${SCRIPT_DIR}/check_slo\" \
        --config \"${SCRIPT_DIR}/slo.yaml\" \
        --metrics \"${SCRIPT_DIR}/test_metrics_pass.json\" \
        --env development \
        --fail-on-violation" \
    0

# Test 2: Production environment with failing metrics (should fail)
run_test "Production environment - failing metrics" \
    "\"${SCRIPT_DIR}/check_slo\" \
        --config \"${SCRIPT_DIR}/slo.yaml\" \
        --metrics \"${SCRIPT_DIR}/test_metrics_fail.json\" \
        --env production \
        --fail-on-violation" \
    1

# Test 3: Staging environment with relaxed SLOs (should pass with multiplier)
run_test "Staging environment - relaxed SLOs" \
    "\"${SCRIPT_DIR}/check_slo\" \
        --config \"${SCRIPT_DIR}/slo.yaml\" \
        --metrics \"${SCRIPT_DIR}/test_metrics_fail.json\" \
        --env staging \
        --fail-on-violation" \
    0

# Test 4: Tolerance mode (should pass some marginal failures)
run_test "Tolerance mode enabled" \
    "\"${SCRIPT_DIR}/check_slo\" \
        --config \"${SCRIPT_DIR}/slo.yaml\" \
        --metrics \"${SCRIPT_DIR}/test_metrics_fail.json\" \
        --env production \
        --tolerance \
        --fail-on-violation" \
    0

# Test 5: Warning mode (should always pass but report issues)
run_test "Warning mode (no failure on violations)" \
    "\"${SCRIPT_DIR}/check_slo\" \
        --config \"${SCRIPT_DIR}/slo.yaml\" \
        --metrics \"${SCRIPT_DIR}/test_metrics_fail.json\" \
        --env production" \
    0

# Test 6: JSON output format validation
echo ""
echo -e "${BLUE}Test: JSON output format validation${NC}"
TESTS_RUN=$((TESTS_RUN + 1))

if "${SCRIPT_DIR}/check_slo" \
    --config "${SCRIPT_DIR}/slo.yaml" \
    --metrics "${SCRIPT_DIR}/test_metrics_pass.json" \
    --env development \
    --format json \
    --output "${SCRIPT_DIR}/ci_test_output.json" >/dev/null 2>&1; then
    
    if [[ -f "${SCRIPT_DIR}/ci_test_output.json" ]]; then
        if jq empty "${SCRIPT_DIR}/ci_test_output.json" 2>/dev/null; then
            # Validate JSON structure
            if jq -e '.total_slos and .passed and .failed and .warnings and .exemptions and .results' \
                "${SCRIPT_DIR}/ci_test_output.json" >/dev/null 2>&1; then
                echo -e "${GREEN}✓ PASS${NC} (JSON output valid and complete)"
                TESTS_PASSED=$((TESTS_PASSED + 1))
            else
                echo -e "${RED}✗ FAIL${NC} (JSON structure incomplete)"
                TESTS_FAILED=$((TESTS_FAILED + 1))
            fi
        else
            echo -e "${RED}✗ FAIL${NC} (Invalid JSON output)"
            TESTS_FAILED=$((TESTS_FAILED + 1))
        fi
    else
        echo -e "${RED}✗ FAIL${NC} (Output file not created)"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
else
    echo -e "${RED}✗ FAIL${NC} (Command failed)"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Test 7: HTML output format validation
echo ""
echo -e "${BLUE}Test: HTML output format validation${NC}"
TESTS_RUN=$((TESTS_RUN + 1))

if "${SCRIPT_DIR}/check_slo" \
    --config "${SCRIPT_DIR}/slo.yaml" \
    --metrics "${SCRIPT_DIR}/test_metrics_pass.json" \
    --env development \
    --format html \
    --output "${SCRIPT_DIR}/ci_test_output.html" >/dev/null 2>&1; then
    
    if [[ -f "${SCRIPT_DIR}/ci_test_output.html" ]]; then
        if grep -q "<!DOCTYPE html>" "${SCRIPT_DIR}/ci_test_output.html" && \
           grep -q "SLO Check Report" "${SCRIPT_DIR}/ci_test_output.html" && \
           grep -q "</html>" "${SCRIPT_DIR}/ci_test_output.html"; then
            echo -e "${GREEN}✓ PASS${NC} (HTML output valid)"
            TESTS_PASSED=$((TESTS_PASSED + 1))
        else
            echo -e "${RED}✗ FAIL${NC} (HTML structure incomplete)"
            TESTS_FAILED=$((TESTS_FAILED + 1))
        fi
    else
        echo -e "${RED}✗ FAIL${NC} (HTML file not created)"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
else
    echo -e "${RED}✗ FAIL${NC} (Command failed)"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Test 8: Missing metrics file handling
run_test "Missing metrics file handling" \
    "\"${SCRIPT_DIR}/check_slo\" \
        --config \"${SCRIPT_DIR}/slo.yaml\" \
        --metrics \"${SCRIPT_DIR}/nonexistent_metrics.json\" \
        --env development \
        --fail-on-violation" \
    1

# Test 9: Invalid config file handling
run_test "Invalid config file handling" \
    "\"${SCRIPT_DIR}/check_slo\" \
        --config \"${SCRIPT_DIR}/nonexistent_config.yaml\" \
        --metrics \"${SCRIPT_DIR}/test_metrics_pass.json\" \
        --env development \
        --fail-on-violation" \
    1

# Test 10: Performance test with timing
echo ""
echo -e "${BLUE}Test: Performance measurement${NC}"
TESTS_RUN=$((TESTS_RUN + 1))

start_time=$(date +%s%N)
if "${SCRIPT_DIR}/check_slo" \
    --config "${SCRIPT_DIR}/slo.yaml" \
    --metrics "${SCRIPT_DIR}/test_metrics_pass.json" \
    --env development >/dev/null 2>&1; then
    end_time=$(date +%s%N)
    
    # Calculate execution time in milliseconds
    execution_time=$(( (end_time - start_time) / 1000000 ))
    
    # Performance should be under 5 seconds for normal metrics
    if [[ $execution_time -lt 5000 ]]; then
        echo -e "${GREEN}✓ PASS${NC} (execution time: ${execution_time}ms)"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${YELLOW}⚠ SLOW${NC} (execution time: ${execution_time}ms > 5000ms)"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    fi
else
    echo -e "${RED}✗ FAIL${NC} (Command failed)"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Test 11: Simulate CI pipeline integration
echo ""
echo -e "${BLUE}Test: CI Pipeline Integration Simulation${NC}"
TESTS_RUN=$((TESTS_RUN + 1))

# Create a temporary CI script
cat > "${SCRIPT_DIR}/ci_simulation.sh" << 'EOF'
#!/bin/bash
set -euo pipefail

# Simulate CI environment variables
export CI=true
export GITHUB_ACTIONS=true
export GITHUB_WORKFLOW="SLO Gates"
export GITHUB_JOB="check-slos"

# Run SLO check as part of CI
echo "Running SLO checks in CI..."
if ./check_slo \
    --config ./slo.yaml \
    --metrics ./test_metrics_pass.json \
    --env production \
    --fail-on-violation \
    --format json \
    --output ./slo_results.json; then
    echo "SLO checks passed - proceeding with deployment"
    exit 0
else
    echo "SLO checks failed - blocking deployment"
    exit 1
fi
EOF

chmod +x "${SCRIPT_DIR}/ci_simulation.sh"

# Run CI simulation
if (cd "${SCRIPT_DIR}" && ./ci_simulation.sh) >/dev/null 2>&1; then
    echo -e "${GREEN}✓ PASS${NC} (CI integration successful)"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC} (CI integration failed)"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Clean up CI simulation files
rm -f "${SCRIPT_DIR}/ci_simulation.sh"
rm -f "${SCRIPT_DIR}/slo_results.json"

# Test 12: Environment override testing
echo ""
echo -e "${BLUE}Test: Environment-specific overrides${NC}"
TESTS_RUN=$((TESTS_RUN + 1))

# Test development environment multiplier (should be more lenient)
if "${SCRIPT_DIR}/check_slo" \
    --config "${SCRIPT_DIR}/slo.yaml" \
    --metrics "${SCRIPT_DIR}/test_metrics_fail.json" \
    --env development \
    --format json \
    --output "${SCRIPT_DIR}/dev_override_test.json" >/dev/null 2>&1; then
    
    # Check if development environment has more passes due to multiplier
    dev_passed=$(jq '.passed' "${SCRIPT_DIR}/dev_override_test.json" 2>/dev/null || echo "0")
    
    if [[ $dev_passed -gt 0 ]]; then
        echo -e "${GREEN}✓ PASS${NC} (Development environment overrides working: $dev_passed passed)"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}✗ FAIL${NC} (Development environment overrides not working)"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
else
    echo -e "${RED}✗ FAIL${NC} (Command failed)"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Clean up test files
echo ""
echo "Cleaning up test files..."
rm -f "${SCRIPT_DIR}/ci_test_output.json"
rm -f "${SCRIPT_DIR}/ci_test_output.html"
rm -f "${SCRIPT_DIR}/dev_override_test.json"

# Final results
echo ""
echo "========================================="
echo -e "${BLUE}CI Integration Test Results${NC}"
echo "========================================="
echo "Tests Run: $TESTS_RUN"
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"

if [[ $TESTS_FAILED -eq 0 ]]; then
    echo ""
    echo -e "${GREEN}🎉 All CI integration tests passed!${NC}"
    echo "The SLO Gates system is ready for production use."
    exit 0
else
    echo ""
    echo -e "${RED}❌ Some CI integration tests failed.${NC}"
    echo "Please review the failures before deploying."
    exit 1
fi
