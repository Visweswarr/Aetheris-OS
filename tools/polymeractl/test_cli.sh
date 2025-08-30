#!/bin/bash
set -e

echo "🧪 Testing Polymera OS CLI (polymeractl)..."
echo "=============================================="

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
    
    if eval "$test_command" > /tmp/cli_test_output.log 2>&1; then
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
            cat /tmp/cli_test_output.log
            ((TESTS_FAILED++))
        fi
    fi
}

# Check if CLI is built
if [ ! -f "target/release/polymeractl" ] && [ ! -f "target/debug/polymeractl" ]; then
    echo -e "${YELLOW}Building CLI first...${NC}"
    cargo build --package polymeractl
fi

# Find CLI binary
CLI_BIN=""
if [ -f "target/release/polymeractl" ]; then
    CLI_BIN="target/release/polymeractl"
elif [ -f "target/debug/polymeractl" ]; then
    CLI_BIN="target/debug/polymeractl"
else
    echo -e "${RED}CLI binary not found!${NC}"
    exit 1
fi

echo -e "${GREEN}Using CLI: $CLI_BIN${NC}"

# Test 1: Help command
run_test "Help Command" "$CLI_BIN --help" 0

# Test 2: Version command
run_test "Version Command" "$CLI_BIN --version" 0

# Test 3: Info command
run_test "Info Command" "$CLI_BIN info" 0

# Test 4: Info with detailed flag
run_test "Detailed Info Command" "$CLI_BIN info --detailed" 0

# Test 5: Build command help
run_test "Build Help" "$CLI_BIN build --help" 0

# Test 6: Build command with invalid target
run_test "Build Invalid Target" "$CLI_BIN build --target invalid" 1

# Test 7: Run-QEMU command help
run_test "Run-QEMU Help" "$CLI_BIN run-qemu --help" 0

# Test 8: Run-QEMU with invalid architecture
run_test "Run-QEMU Invalid Arch" "$CLI_BIN run-qemu --arch invalid" 1

# Test 9: Mkimage command help
run_test "Mkimage Help" "$CLI_BIN mkimage --help" 0

# Test 10: Mkimage with invalid format
run_test "Mkimage Invalid Format" "$CLI_BIN mkimage --format invalid" 1

# Test 11: Verify command help
run_test "Verify Help" "$CLI_BIN verify --help" 0

# Test 12: Verify with invalid target
run_test "Verify Invalid Target" "$CLI_BIN verify --target invalid" 1

# Test 13: SBOM command help
run_test "SBOM Help" "$CLI_BIN sbom --help" 0

# Test 14: SBOM with invalid format
run_test "SBOM Invalid Format" "$CLI_BIN sbom --format invalid" 1

# Test 15: Sign command help
run_test "Sign Help" "$CLI_BIN sign --help" 0

# Test 16: Sign with invalid algorithm
run_test "Sign Invalid Algorithm" "$CLI_BIN sign --algorithm invalid" 1

# Test 17: Init command help
run_test "Init Help" "$CLI_BIN init --help" 0

# Test 18: Update command help
run_test "Update Help" "$CLI_BIN update --help" 0

# Test 19: Clean command help
run_test "Clean Help" "$CLI_BIN clean --help" 0

# Test 20: No color flag
run_test "No Color Flag" "$CLI_BIN --no-color info" 0

# Test 21: Verbose flag
run_test "Verbose Flag" "$CLI_BIN --verbose info" 0

# Test 22: Debug flag
run_test "Debug Flag" "$CLI_BIN --debug info" 0

# Test 23: Build with specific target
run_test "Build Kernel Target" "$CLI_BIN build --target kernel --no-test" 0

# Test 24: Build with profile
run_test "Build Release Profile" "$CLI_BIN build --profile release --no-test" 0

# Test 25: Build with output directory
run_test "Build Custom Output" "$CLI_BIN build --output test_dist --no-test" 0

# Test 26: Run-QEMU with memory and CPU
run_test "Run-QEMU Custom Config" "$CLI_BIN run-qemu --memory 256 --cpus 1" 0

# Test 27: Mkimage with custom size
run_test "Mkimage Custom Size" "$CLI_BIN mkimage --size 5" 0

# Test 28: Verify with custom output format
run_test "Verify JSON Output" "$CLI_BIN verify --output json" 0

# Test 29: SBOM generation
run_test "SBOM Generation" "$CLI_BIN sbom --action generate --format spdx" 0

# Test 30: Sign with custom algorithm
run_test "Sign Custom Algorithm" "$CLI_BIN sign --algorithm rsa" 0

# Test 31: Init new project
run_test "Init New Project" "$CLI_BIN init test-project --template basic" 0

# Test 32: Update toolchains
run_test "Update Toolchains" "$CLI_BIN update --toolchains" 0

# Test 33: Clean build artifacts
run_test "Clean Build" "$CLI_BIN clean --build" 0

# Test 34: Cleanup test files
run_test "Cleanup Test Files" "rm -rf test-project test_dist polymera.img" 0

# Test 35: CLI compilation check
run_test "CLI Compilation" "cargo check --package polymeractl" 0

# Test 36: CLI linting
run_test "CLI Linting" "cargo clippy --package polymeractl -- -D warnings" 0

# Test 37: CLI formatting
run_test "CLI Formatting" "cargo fmt --package polymeractl -- --check" 0

# Test 38: CLI tests compilation
run_test "CLI Tests Compilation" "cargo test --package polymeractl --no-run" 0

# Test 39: CLI documentation generation
run_test "CLI Documentation" "cargo doc --package polymeractl --no-deps" 0

# Test 40: CLI binary size check
run_test "CLI Binary Size" "[ -s $CLI_BIN ]" 0

echo -e "\n=============================================="
echo -e "${BLUE}Test Results:${NC}"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo -e "Total: $((TESTS_PASSED + TESTS_FAILED))"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "\n${RED}❌ Some tests failed!${NC}"
    exit 1
fi
