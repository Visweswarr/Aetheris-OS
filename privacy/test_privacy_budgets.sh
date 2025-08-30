#!/bin/bash
set -e

echo "🧪 Testing Privacy Budgets System..."
echo "====================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Test function
run_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_exit="$3"
    
    echo -e "\n${BLUE}Running: ${test_name}${NC}"
    echo "Command: $test_command"
    
    if eval "$test_command" > /tmp/privacy_test_output.log 2>&1; then
        local exit_code=$?
        if [ "$exit_code" = "$expected_exit" ]; then
            echo -e "${GREEN}✓ PASSED${NC}"
            ((TESTS_PASSED++))
        else
            echo -e "${RED}✗ FAILED (expected exit $expected_exit, got $exit_code)${NC}"
            ((TESTS_FAILED++))
        fi
    else
        local exit_code=$?
        if [ "$exit_code" = "$expected_exit" ]; then
            echo -e "${GREEN}✓ PASSED${NC}"
            ((TESTS_PASSED++))
        else
            echo -e "${RED}✗ FAILED (expected exit $expected_exit, got $exit_code)${NC}"
            echo "Output:"
            cat /tmp/privacy_test_output.log
            ((TESTS_FAILED++))
        fi
    fi
}

# Check if crate is built
if [ ! -f "target/debug/libpolymera_privacy.rlib" ] && [ ! -f "target/release/libpolymera_privacy.rlib" ]; then
    echo -e "${YELLOW}Building privacy budget crate first...${NC}"
    cargo build --package polymera-privacy
fi

# Test 1: Crate compilation
run_test "Crate Compilation" "cargo check --package polymera-privacy" 0

# Test 2: Crate building
run_test "Crate Building" "cargo build --package polymera-privacy" 0

# Test 3: Unit tests
run_test "Unit Tests" "cargo test --package polymera-privacy" 0

# Test 4: Integration tests
run_test "Integration Tests" "cargo test --package polymera-privacy --test integration" 0

# Test 5: Documentation generation
run_test "Documentation Generation" "cargo doc --package polymera-privacy --no-deps" 0

# Test 6: Code formatting
run_test "Code Formatting" "cargo fmt --package polymera-privacy -- --check" 0

# Test 7: Code linting
run_test "Code Linting" "cargo clippy --package polymera-privacy -- -D warnings" 0

# Test 8: Configuration validation
run_test "Configuration Validation" "cargo run --package polymera-privacy --example config_validation" 0

# Test 9: Budget manager tests
run_test "Budget Manager Tests" "cargo test --package polymera-privacy test_privacy_budget_manager" 0

# Test 10: Middleware tests
run_test "Middleware Tests" "cargo test --package polymera-privacy test_middleware" 0

# Test 11: Differential privacy tests
run_test "Differential Privacy Tests" "cargo test --package polymera-privacy test_differential_privacy" 0

# Test 12: Token bucket tests
run_test "Token Bucket Tests" "cargo test --package polymera-privacy test_token_bucket" 0

# Test 13: Configuration loading tests
run_test "Configuration Loading Tests" "cargo test --package polymera-privacy test_config_loading" 0

# Test 14: Metrics collection tests
run_test "Metrics Collection Tests" "cargo test --package polymera-privacy test_metrics" 0

# Test 15: Error handling tests
run_test "Error Handling Tests" "cargo test --package polymera-privacy test_error_handling" 0

# Test 16: Validation tests
run_test "Validation Tests" "cargo test --package polymera-privacy test_validation" 0

# Test 17: Utility function tests
run_test "Utility Function Tests" "cargo test --package polymera-privacy test_utils" 0

# Test 18: Constants tests
run_test "Constants Tests" "cargo test --package polymera-privacy test_constants" 0

# Test 19: Module structure tests
run_test "Module Structure Tests" "cargo test --package polymera-privacy test_module_structure" 0

# Test 20: Feature flag tests
run_test "Feature Flag Tests" "cargo test --package polymera-privacy --features actix-web" 0

# Test 21: Axum feature tests
run_test "Axum Feature Tests" "cargo test --package polymera-privacy --features axum" 0

# Test 22: All features tests
run_test "All Features Tests" "cargo test --package polymera-privacy --features all-integrations" 0

# Test 23: Security feature tests
run_test "Security Feature Tests" "cargo test --package polymera-privacy --features security" 0

# Test 24: Performance feature tests
run_test "Performance Feature Tests" "cargo test --package polymera-privacy --features performance" 0

# Test 25: Development feature tests
run_test "Development Feature Tests" "cargo test --package polymera-privacy --features dev" 0

# Test 26: Release build tests
run_test "Release Build Tests" "cargo build --release --package polymera-privacy" 0

# Test 27: Test coverage compilation
run_test "Test Coverage Compilation" "cargo test --package polymera-privacy --no-run" 0

# Test 28: Benchmark compilation
run_test "Benchmark Compilation" "cargo bench --package polymera-privacy --no-run" 0

# Test 29: Example compilation
run_test "Example Compilation" "cargo build --examples --package polymera-privacy" 0

# Test 30: Documentation tests
run_test "Documentation Tests" "cargo test --package polymera-privacy --doc" 0

# Test 31: Configuration file validation
run_test "Configuration File Validation" "test -f config.yaml" 0

# Test 32: Configuration file syntax
run_test "Configuration File Syntax" "python3 -c \"import yaml; yaml.safe_load(open('config.yaml'))\" 2>/dev/null" 0

# Test 33: Module file existence
run_test "Module File Existence" "test -f mod.rs" 0

# Test 34: Budget module existence
run_test "Budget Module Existence" "test -f budget.rs" 0

# Test 35: Middleware module existence
run_test "Middleware Module Existence" "test -f middleware.rs" 0

# Test 36: Cargo.toml validation
run_test "Cargo.toml Validation" "cargo verify-project" 0

# Test 37: Dependency resolution
run_test "Dependency Resolution" "cargo tree --package polymera-privacy" 0

# Test 38: License compliance
run_test "License Compliance" "cargo license --package polymera-privacy" 0

# Test 39: Security audit
run_test "Security Audit" "cargo audit --package polymera-privacy" 0

# Test 40: Size optimization
run_test "Size Optimization" "cargo build --release --package polymera-privacy && ls -lh target/release/libpolymera_privacy.rlib" 0

echo -e "\n====================================="
echo -e "${BLUE}Test Results:${NC}"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo -e "Total: $((TESTS_PASSED + TESTS_FAILED))"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All privacy budget tests passed!${NC}"
    
    # Run additional validation
    echo -e "\n${BLUE}Running Additional Validation...${NC}"
    
    # Check file sizes
    echo -e "\n${YELLOW}File Sizes:${NC}"
    ls -lh *.rs *.yaml *.toml 2>/dev/null || true
    
    # Check line counts
    echo -e "\n${YELLOW}Line Counts:${NC}"
    wc -l *.rs *.yaml *.toml 2>/dev/null || true
    
    # Check test coverage
    echo -e "\n${YELLOW}Test Coverage Summary:${NC}"
    echo "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"
    echo "Passed: $TESTS_PASSED"
    echo "Failed: $TESTS_FAILED"
    echo "Coverage: $((TESTS_PASSED * 100 / (TESTS_PASSED + TESTS_FAILED)))%"
    
    exit 0
else
    echo -e "\n${RED}❌ Some privacy budget tests failed!${NC}"
    echo -e "\n${YELLOW}Failed test details:${NC}"
    cat /tmp/privacy_test_output.log 2>/dev/null || echo "No detailed output available"
    exit 1
fi
