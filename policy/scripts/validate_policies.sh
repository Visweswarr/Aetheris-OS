#!/bin/bash
# Policy validation script for OPA/Rego policies

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Validating Polymera OS Policy Samples${NC}"
echo "======================================"

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
POLICY_DIR="$(dirname "$SCRIPT_DIR")"

# Validation counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

echo "Policy directory: $POLICY_DIR"
echo ""

# Function to validate a single policy
validate_policy() {
    local policy_file="$1"
    local policy_name="$(basename "$policy_file" .rego)"
    
    echo -e "${BLUE}Validating Policy: $policy_name${NC}"
    echo "File: $policy_file"
    
    TESTS_RUN=$((TESTS_RUN + 1))
    
    # Check if file exists
    if [[ ! -f "$policy_file" ]]; then
        echo -e "${RED}✗ FAIL: Policy file not found${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    # Syntax validation
    echo -n "  Syntax validation... "
    if opa fmt --list "$policy_file" >/dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
    else
        echo -e "${RED}✗ FAIL${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    # Structure validation
    echo -n "  Structure validation... "
    if opa test "$policy_file" --explain=notes >/dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
    else
        echo -e "${YELLOW}⚠ WARNING: No tests found or test failures${NC}"
    fi
    
    # Package validation
    echo -n "  Package validation... "
    package_line=$(grep "^package " "$policy_file" | head -1)
    if [[ -n "$package_line" ]]; then
        package_name=$(echo "$package_line" | cut -d' ' -f2)
        echo -e "${GREEN}✓ PASS (package: $package_name)${NC}"
    else
        echo -e "${RED}✗ FAIL: No package declaration found${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    # Entrypoint validation
    echo -n "  Entrypoint validation... "
    if grep -q "^allow\s*:=" "$policy_file" || grep -q "^allow\s*if" "$policy_file"; then
        echo -e "${GREEN}✓ PASS (allow rule found)${NC}"
    else
        echo -e "${RED}✗ FAIL: No 'allow' entrypoint found${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    # Default deny validation
    echo -n "  Default deny validation... "
    if grep -q "default allow := false" "$policy_file"; then
        echo -e "${GREEN}✓ PASS (default deny present)${NC}"
    else
        echo -e "${YELLOW}⚠ WARNING: No explicit default deny found${NC}"
    fi
    
    # Rationale function validation
    echo -n "  Rationale function validation... "
    if grep -q "^rationale\s*:=" "$policy_file"; then
        echo -e "${GREEN}✓ PASS (rationale function found)${NC}"
    else
        echo -e "${YELLOW}⚠ WARNING: No rationale function found${NC}"
    fi
    
    # Input validation
    echo -n "  Input validation... "
    if grep -q "valid.*input" "$policy_file"; then
        echo -e "${GREEN}✓ PASS (input validation found)${NC}"
    else
        echo -e "${YELLOW}⚠ WARNING: No input validation found${NC}"
    fi
    
    TESTS_PASSED=$((TESTS_PASSED + 1))
    echo -e "${GREEN}✓ OVERALL: Policy validation passed${NC}"
    echo ""
    return 0
}

# Function to run unit tests
run_unit_tests() {
    local policy_file="$1"
    local test_file="$2"
    local policy_name="$(basename "$policy_file" .rego)"
    
    echo -e "${BLUE}Running Unit Tests: $policy_name${NC}"
    
    if [[ ! -f "$test_file" ]]; then
        echo -e "${YELLOW}⚠ WARNING: No test file found for $policy_name${NC}"
        return 0
    fi
    
    echo "Test file: $test_file"
    
    # Run OPA tests
    echo -n "  Running OPA unit tests... "
    if opa test "$policy_file" "$test_file" --verbose >/dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
        
        # Get detailed test results
        test_output=$(opa test "$policy_file" "$test_file" --format=json 2>/dev/null)
        if [[ -n "$test_output" ]]; then
            test_count=$(echo "$test_output" | jq -r 'length')
            passed_count=$(echo "$test_output" | jq -r '[.[] | select(.fail == false)] | length')
            echo "    Tests run: $test_count, Passed: $passed_count"
        fi
    else
        echo -e "${RED}✗ FAIL${NC}"
        echo "    Test output:"
        opa test "$policy_file" "$test_file" --verbose 2>&1 | sed 's/^/    /'
        return 1
    fi
    
    echo ""
    return 0
}

# Function to validate against test scenarios
validate_scenarios() {
    local policy_name="$1"
    local scenario_file="$POLICY_DIR/test_data/${policy_name}_test_scenarios.json"
    
    echo -e "${BLUE}Validating Test Scenarios: $policy_name${NC}"
    
    if [[ ! -f "$scenario_file" ]]; then
        echo -e "${YELLOW}⚠ WARNING: No scenario file found for $policy_name${NC}"
        return 0
    fi
    
    echo "Scenario file: $scenario_file"
    
    # Validate JSON syntax
    echo -n "  JSON syntax validation... "
    if jq empty "$scenario_file" >/dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
    else
        echo -e "${RED}✗ FAIL: Invalid JSON syntax${NC}"
        return 1
    fi
    
    # Count scenarios
    scenario_count=$(jq -r ".test_scenarios.${policy_name} | length" "$scenario_file" 2>/dev/null || echo "0")
    echo "    Scenarios found: $scenario_count"
    
    # Validate scenario structure
    echo -n "  Scenario structure validation... "
    valid_scenarios=0
    for ((i=0; i<scenario_count; i++)); do
        scenario=$(jq -r ".test_scenarios.${policy_name}[$i]" "$scenario_file")
        
        # Check required fields
        if echo "$scenario" | jq -e '.name and .input and .expected' >/dev/null 2>&1; then
            valid_scenarios=$((valid_scenarios + 1))
        fi
    done
    
    if [[ $valid_scenarios -eq $scenario_count ]] && [[ $scenario_count -gt 0 ]]; then
        echo -e "${GREEN}✓ PASS ($valid_scenarios/$scenario_count scenarios valid)${NC}"
    else
        echo -e "${RED}✗ FAIL ($valid_scenarios/$scenario_count scenarios valid)${NC}"
        return 1
    fi
    
    echo ""
    return 0
}

# Function to check policy security
check_security() {
    local policy_file="$1"
    local policy_name="$(basename "$policy_file" .rego)"
    
    echo -e "${BLUE}Security Check: $policy_name${NC}"
    
    local security_issues=0
    
    # Check for hardcoded secrets
    echo -n "  Hardcoded secrets check... "
    if grep -i -E "(password|secret|key|token)\s*:=\s*\"[^\"]+\"" "$policy_file" >/dev/null 2>&1; then
        echo -e "${RED}✗ FAIL: Potential hardcoded secrets found${NC}"
        grep -n -i -E "(password|secret|key|token)\s*:=\s*\"[^\"]+\"" "$policy_file" | sed 's/^/    /'
        security_issues=$((security_issues + 1))
    else
        echo -e "${GREEN}✓ PASS${NC}"
    fi
    
    # Check for dangerous operations
    echo -n "  Dangerous operations check... "
    if grep -E "(shell|exec|system|eval)" "$policy_file" >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠ WARNING: Potentially dangerous operations found${NC}"
        grep -n -E "(shell|exec|system|eval)" "$policy_file" | sed 's/^/    /'
    else
        echo -e "${GREEN}✓ PASS${NC}"
    fi
    
    # Check for proper input validation
    echo -n "  Input validation coverage... "
    if grep -E "input\." "$policy_file" | grep -q -E "(valid|check|validate)"; then
        echo -e "${GREEN}✓ PASS${NC}"
    else
        echo -e "${YELLOW}⚠ WARNING: Limited input validation detected${NC}"
    fi
    
    # Check for authorization bypass patterns
    echo -n "  Authorization bypass check... "
    if grep -E "allow\s*:=\s*true\s*$" "$policy_file" >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠ WARNING: Unconditional allow rules found${NC}"
        grep -n -E "allow\s*:=\s*true\s*$" "$policy_file" | sed 's/^/    /'
    else
        echo -e "${GREEN}✓ PASS${NC}"
    fi
    
    if [[ $security_issues -eq 0 ]]; then
        echo -e "${GREEN}✓ OVERALL: Security check passed${NC}"
    else
        echo -e "${RED}✗ OVERALL: Security issues found${NC}"
    fi
    
    echo ""
    return $security_issues
}

# Main validation execution
echo "Starting policy validation..."
echo ""

# Policy files to validate
POLICIES=(
    "$POLICY_DIR/samples/wallet_spend_limits.rego"
    "$POLICY_DIR/samples/network_rate_limits.rego"
    "$POLICY_DIR/samples/fs_access_scopes.rego"
)

# Validate each policy
for policy_file in "${POLICIES[@]}"; do
    if [[ -f "$policy_file" ]]; then
        policy_name="$(basename "$policy_file" .rego)"
        
        # Basic validation
        validate_policy "$policy_file"
        
        # Unit tests
        test_file="$POLICY_DIR/samples/${policy_name}_test.rego"
        run_unit_tests "$policy_file" "$test_file"
        
        # Scenario validation
        validate_scenarios "$policy_name"
        
        # Security check
        check_security "$policy_file"
        
        echo "----------------------------------------"
    else
        echo -e "${RED}Policy file not found: $policy_file${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
done

# Summary
echo ""
echo "======================================"
echo -e "${BLUE}Validation Summary${NC}"
echo "======================================"
echo "Total policies validated: $TESTS_RUN"
echo -e "Policies passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Policies failed: ${RED}$TESTS_FAILED${NC}"

if [[ $TESTS_FAILED -eq 0 ]]; then
    echo ""
    echo -e "${GREEN}🎉 All policy validations passed!${NC}"
    echo "Policies are ready for WASM compilation."
    exit 0
else
    echo ""
    echo -e "${RED}❌ Some policy validations failed.${NC}"
    echo "Please review and fix the issues above."
    exit 1
fi
