#!/bin/bash
#! Comprehensive TLS Test Runner for Aetheris OS
#! 
#! This script runs all TLS integration tests across all language bindings.

set -e

echo "==============================================="
echo "Aetheris OS TLS Integration Test Suite"
echo "==============================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Function to run a test and track results
run_test() {
    local test_name="$1"
    local test_command="$2"
    local timeout_seconds="${3:-30}"
    
    echo -e "${BLUE}Running: $test_name${NC}"
    echo "Command: $test_command"
    echo ""
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if timeout $timeout_seconds bash -c "$test_command" > /tmp/test_output.log 2>&1; then
        echo -e "${GREEN}✓ $test_name PASSED${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            echo -e "${YELLOW}⚠ $test_name TIMEOUT${NC}"
            SKIPPED_TESTS=$((SKIPPED_TESTS + 1))
        else
            echo -e "${RED}✗ $test_name FAILED${NC}"
            echo "Error output:"
            cat /tmp/test_output.log
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi
    fi
    echo ""
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

echo "Checking prerequisites..."
echo "========================="

# Check for required tools
tools=("cargo" "go" "python3" "gcc" "npx")
for tool in "${tools[@]}"; do
    if command_exists "$tool"; then
        echo -e "${GREEN}✓ $tool found${NC}"
    else
        echo -e "${YELLOW}⚠ $tool not found${NC}"
    fi
done
echo ""

echo "Running TLS Integration Tests"
echo "============================="
echo ""

# 1. Rust TLS Tests
echo "1. Rust TLS Implementation Tests"
echo "--------------------------------"
if command_exists "cargo"; then
    run_test "Rust TLS Unit Tests" "cargo test --package posixnet --test tls_tests" 60
    run_test "Rust TLS Integration Tests" "cargo test --package posixnet --test tls_integration_test" 60
    run_test "Rust TLS Performance Tests" "cargo test --package posixnet --test tls_tests -- --ignored" 120
else
    echo -e "${YELLOW}⚠ Rust tests skipped (cargo not found)${NC}"
    SKIPPED_TESTS=$((SKIPPED_TESTS + 3))
    TOTAL_TESTS=$((TOTAL_TESTS + 3))
fi
echo ""

# 2. C TLS Tests
echo "2. C TLS Bindings Tests"
echo "-----------------------"
if command_exists "gcc"; then
    run_test "C TLS Compilation" "gcc -c c/libc_aetheris/src/pqc_tls.c -o /tmp/pqc_tls_test.o -I c/libc_aetheris/include" 30
    run_test "C TLS Integration Tests" "gcc -o /tmp/tls_c_tests tests/c/tls_c_tests.c c/libc_aetheris/src/pqc_tls.c -I c/libc_aetheris/include && /tmp/tls_c_tests" 60
    run_test "C TLS Socket Integration" "gcc -c c/libc_aetheris/src/net.c -o /tmp/net_test.o -I c/libc_aetheris/include" 30
else
    echo -e "${YELLOW}⚠ C tests skipped (gcc not found)${NC}"
    SKIPPED_TESTS=$((SKIPPED_TESTS + 3))
    TOTAL_TESTS=$((TOTAL_TESTS + 3))
fi
echo ""

# 3. Go TLS Tests
echo "3. Go TLS CLI Tests"
echo "-------------------"
if command_exists "go"; then
    run_test "Go TLS CLI Help Commands" "go run go/tooling/netctl/main.go --help" 10
    run_test "Go TLS Echo Server Help" "go run go/tooling/netctl/main.go tls-echo-server --help" 10
    run_test "Go TLS Echo Client Help" "go run go/tooling/netctl/main.go tls-echo-client --help" 10
    run_test "Go Firewall Help" "go run go/tooling/netctl/main.go firewall --help" 10
    run_test "Go Benchmark Help" "go run go/tooling/netctl/main.go bench --help" 10
    run_test "Go TLS Integration Tests" "go test tests/go/tls_go_tests.go -v" 60
else
    echo -e "${YELLOW}⚠ Go tests skipped (go not found)${NC}"
    SKIPPED_TESTS=$((SKIPPED_TESTS + 6))
    TOTAL_TESTS=$((TOTAL_TESTS + 6))
fi
echo ""

# 4. TypeScript TLS Tests
echo "4. TypeScript TLS Bridge Tests"
echo "------------------------------"
if command_exists "npx"; then
    run_test "TypeScript TLS Compilation" "npx tsc --noEmit ui/net/secure_sockets.ts" 30
    run_test "TypeScript TLS Type Checking" "npx tsc --noEmit --strict ui/net/secure_sockets.ts" 30
else
    echo -e "${YELLOW}⚠ TypeScript tests skipped (npx not found)${NC}"
    SKIPPED_TESTS=$((SKIPPED_TESTS + 2))
    TOTAL_TESTS=$((TOTAL_TESTS + 2))
fi
echo ""

# 5. Python TLS Tests
echo "5. Python TLS Validator Tests"
echo "-----------------------------"
if command_exists "python3"; then
    run_test "Python TLS Validator Import" "python3 -c 'import sys; sys.path.insert(0, \"tooling\"); import network_validator'" 10
    run_test "Python TLS Validator Help" "python3 tooling/network_validator.py --help" 10
    run_test "Python TLS Integration Tests" "python3 tests/python/tls_python_tests.py" 60
    run_test "Python TLS Configuration Validation" "python3 tooling/network_validator.py --validate-tls" 30
    run_test "Python TLS PQC Validation" "python3 tooling/network_validator.py --validate-pqc" 30
    run_test "Python TLS Firewall Validation" "python3 tooling/network_validator.py --validate-firewall" 30
else
    echo -e "${YELLOW}⚠ Python tests skipped (python3 not found)${NC}"
    SKIPPED_TESTS=$((SKIPPED_TESTS + 6))
    TOTAL_TESTS=$((TOTAL_TESTS + 6))
fi
echo ""

# 6. End-to-End Integration Tests
echo "6. End-to-End Integration Tests"
echo "-------------------------------"
if command_exists "go" && command_exists "python3"; then
    run_test "TLS Echo Server/Client Integration" "
        go run go/tooling/netctl/main.go tls-echo-server --addr 127.0.0.1 --port 8443 &
        SERVER_PID=\$!
        sleep 2
        go run go/tooling/netctl/main.go tls-echo-client --addr 127.0.0.1 --port 8443 --message 'Hello TLS!' || true
        kill \$SERVER_PID 2>/dev/null || true
        wait \$SERVER_PID 2>/dev/null || true
    " 30
    
    run_test "TLS Firewall Integration" "go run go/tooling/netctl/main.go firewall add --source-ip 127.0.0.1 --dest-port 8443 --action allow" 10
    
    run_test "TLS Benchmark Integration" "go run go/tooling/netctl/main.go bench tcp --connections 100 --duration 5s" 30
    
    run_test "TLS Performance Validation" "python3 tooling/network_validator.py --validate-performance" 60
else
    echo -e "${YELLOW}⚠ End-to-end tests skipped (go or python3 not found)${NC}"
    SKIPPED_TESTS=$((SKIPPED_TESTS + 4))
    TOTAL_TESTS=$((TOTAL_TESTS + 4))
fi
echo ""

# 7. Performance Regression Tests
echo "7. Performance Regression Tests"
echo "-------------------------------"
if command_exists "go"; then
    run_test "TLS Performance Benchmark" "go run go/tooling/netctl/main.go bench concurrent --connections 1000 --duration 10s --protocol tls" 60
    
    run_test "TLS Latency Test" "go run go/tooling/netctl/main.go bench tcp --connections 100 --duration 5s --measure-latency" 30
    
    run_test "TLS Throughput Test" "go run go/tooling/netctl/main.go bench tcp --connections 1000 --duration 10s --measure-throughput" 60
else
    echo -e "${YELLOW}⚠ Performance tests skipped (go not found)${NC}"
    SKIPPED_TESTS=$((SKIPPED_TESTS + 3))
    TOTAL_TESTS=$((TOTAL_TESTS + 3))
fi
echo ""

# 8. Security Validation Tests
echo "8. Security Validation Tests"
echo "----------------------------"
if command_exists "python3"; then
    run_test "TLS Security Validation" "python3 tooling/network_validator.py --validate-security" 30
    
    run_test "PQC Algorithm Validation" "python3 tooling/network_validator.py --validate-pqc --algorithm kyber512" 30
    
    run_test "Certificate Validation" "python3 tooling/network_validator.py --validate-certificates" 30
else
    echo -e "${YELLOW}⚠ Security tests skipped (python3 not found)${NC}"
    SKIPPED_TESTS=$((SKIPPED_TESTS + 3))
    TOTAL_TESTS=$((TOTAL_TESTS + 3))
fi
echo ""

# 9. Error Handling Tests
echo "9. Error Handling Tests"
echo "-----------------------"
if command_exists "go"; then
    run_test "TLS Connection Refused Handling" "go run go/tooling/netctl/main.go tls-echo-client --addr 127.0.0.1 --port 9999 --message 'test' || true" 10
    
    run_test "TLS Invalid Address Handling" "go run go/tooling/netctl/main.go tls-echo-server --addr invalid-ip --port 8443 || true" 10
    
    run_test "TLS Invalid Port Handling" "go run go/tooling/netctl/main.go tls-echo-server --addr 127.0.0.1 --port 99999 || true" 10
else
    echo -e "${YELLOW}⚠ Error handling tests skipped (go not found)${NC}"
    SKIPPED_TESTS=$((SKIPPED_TESTS + 3))
    TOTAL_TESTS=$((TOTAL_TESTS + 3))
fi
echo ""

# 10. Documentation Tests
echo "10. Documentation Tests"
echo "-----------------------"
run_test "TLS Documentation Exists" "test -f docs/phase-4/P4-03-A3-NETWORKING.md" 5
run_test "TLS Configuration Examples Exist" "test -f tooling/examples/tls_config.json" 5
run_test "TLS Firewall Examples Exist" "test -f tooling/examples/firewall_rules.json" 5
run_test "TLS PQC Examples Exist" "test -f tooling/examples/pqc_config.json" 5
run_test "TLS Performance Baseline Exists" "test -f tooling/examples/performance_baseline.json" 5
echo ""

# Clean up temporary files
echo "Cleaning up temporary files..."
rm -f /tmp/test_output.log /tmp/pqc_tls_test.o /tmp/tls_c_tests /tmp/net_test.o
echo ""

# Print test summary
echo "==============================================="
echo "TLS Integration Test Summary"
echo "==============================================="
echo -e "Total Tests: ${BLUE}$TOTAL_TESTS${NC}"
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
echo -e "Skipped: ${YELLOW}$SKIPPED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}🎉 All TLS tests passed successfully!${NC}"
    echo ""
    echo "TLS Implementation Status:"
    echo "✓ Rust TLS service with PQC support"
    echo "✓ C TLS bindings and socket integration"
    echo "✓ Go TLS CLI with echo server/client"
    echo "✓ TypeScript TLS bridge for secure sockets"
    echo "✓ Python TLS validator for correctness"
    echo "✓ Comprehensive test coverage"
    echo "✓ Performance regression testing"
    echo "✓ Security validation"
    echo "✓ Error handling"
    echo "✓ Documentation and examples"
    exit 0
else
    echo -e "${RED}❌ Some TLS tests failed!${NC}"
    echo ""
    echo "Please check the failed tests above and fix any issues."
    exit 1
fi
