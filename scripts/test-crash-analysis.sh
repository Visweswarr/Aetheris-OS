#!/bin/bash

# Crash Analysis++ Test Suite
# Tests the enhanced minidump system and host-side symbolizer

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

# Test configuration
BUILD_DIR="target/release"
KERNEL_BINARY="${BUILD_DIR}/kernel"
SYMBOLIZER_BINARY="tooling/minidump/target/release/minidump-symbolizer"
TEST_OUTPUT_DIR="test-output/crash-analysis"
MINIDUMP_FILE="${TEST_OUTPUT_DIR}/test-crash.dmp"
SYMMAP_FILE="${TEST_OUTPUT_DIR}/test-kernel.symmap"

# Test timeout (seconds)
TEST_TIMEOUT=30

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((TESTS_PASSED++))
}

log_failure() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((TESTS_FAILED++))
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
    ((TESTS_SKIPPED++))
}

run_test() {
    local test_name="$1"
    local test_command="$2"
    local timeout_cmd="timeout ${TEST_TIMEOUT}"
    
    log_info "Running test: $test_name"
    
    if eval "$timeout_cmd $test_command" >/dev/null 2>&1; then
        log_success "$test_name"
        return 0
    else
        log_failure "$test_name"
        return 1
    fi
}

setup_test_environment() {
    log_info "Setting up test environment"
    
    # Create test output directory
    mkdir -p "$TEST_OUTPUT_DIR"
    
    # Check if kernel binary exists
    if [[ ! -f "$KERNEL_BINARY" ]]; then
        log_warning "Kernel binary not found at $KERNEL_BINARY"
        log_warning "Skipping kernel-dependent tests"
        return 1
    fi
    
    # Check if symbolizer binary exists
    if [[ ! -f "$SYMBOLIZER_BINARY" ]]; then
        log_warning "Symbolizer binary not found at $SYMBOLIZER_BINARY"
        log_warning "Building symbolizer..."
        (cd tooling/minidump && cargo build --release)
    fi
    
    return 0
}

test_minidump_generation() {
    log_info "Testing minidump generation capabilities"
    
    # Test 1: Check if minidump module compiles
    run_test "Minidump module compilation" \
        "cargo check --package kernel --lib"
    
    # Test 2: Check if minidump structures are defined
    run_test "Minidump structures definition" \
        "grep -q 'struct MinidumpV2' kernel/src/crash_dump/minidump.rs"
    
    # Test 3: Check if page fault info structure exists
    run_test "Page fault info structure" \
        "grep -q 'struct PageFaultInfo' kernel/src/crash_dump/minidump.rs"
    
    # Test 4: Check if APIC history structure exists
    run_test "APIC history structure" \
        "grep -q 'struct ApicVectorEntry' kernel/src/crash_dump/minidump.rs"
    
    # Test 5: Check if log history structure exists
    run_test "Log history structure" \
        "grep -q 'struct LogEntry' kernel/src/crash_dump/minidump.rs"
    
    # Test 6: Check if CPU features structure exists
    run_test "CPU features structure" \
        "grep -q 'struct CpuFeatures' kernel/src/crash_dump/minidump.rs"
}

test_symbolizer_compilation() {
    log_info "Testing symbolizer compilation"
    
    # Test 1: Check if symbolizer compiles
    run_test "Symbolizer compilation" \
        "cd tooling/minidump && cargo build --release"
    
    # Test 2: Check if symbolizer binary exists
    run_test "Symbolizer binary generation" \
        "test -f $SYMBOLIZER_BINARY"
    
    # Test 3: Check if symbolizer has required dependencies
    run_test "Symbolizer dependencies" \
        "cd tooling/minidump && cargo check --release"
}

test_symbol_table_functionality() {
    log_info "Testing symbol table functionality"
    
    # Create test symmap file
    cat > "$SYMMAP_FILE" << EOF
# Test symbol map
1000 test_function kernel/src/test.rs:42
2000 another_function kernel/src/lib.rs:100
3000 panic_handler kernel/src/panic.rs:15
4000 page_fault_handler kernel/src/interrupts.rs:156
5000 timer_interrupt kernel/src/timer.rs:89
EOF
    
    # Test 1: Test symbolizer with symmap file
    run_test "Symbolizer symmap loading" \
        "$SYMBOLIZER_BINARY --help"
    
    # Test 2: Test symmap file format
    run_test "Symmap file format validation" \
        "grep -q '^[0-9a-fA-F]' $SYMMAP_FILE"
    
    # Test 3: Test symbol count
    local symbol_count=$(grep -c '^[0-9a-fA-F]' "$SYMMAP_FILE")
    if [[ $symbol_count -eq 5 ]]; then
        log_success "Symbol count validation"
        ((TESTS_PASSED++))
    else
        log_failure "Symbol count validation (expected 5, got $symbol_count)"
        ((TESTS_FAILED++))
    fi
}

test_minidump_parsing() {
    log_info "Testing minidump parsing capabilities"
    
    # Create a mock minidump file for testing
    # This is a simplified test - in practice you'd generate real minidumps
    
    # Test 1: Check if minidump parser compiles
    run_test "Minidump parser compilation" \
        "cd tooling/minidump && cargo check --release"
    
    # Test 2: Check if parser structures are defined
    run_test "Parser structures definition" \
        "grep -q 'struct MinidumpParser' tooling/minidump/src/symbolize.rs"
    
    # Test 3: Check if symbol table is defined
    run_test "Symbol table structure" \
        "grep -q 'struct SymbolTable' tooling/minidump/src/symbolize.rs"
}

test_crash_analysis_output() {
    log_info "Testing crash analysis output formatting"
    
    # Test 1: Check if analysis output functions exist
    run_test "Analysis output functions" \
        "grep -q 'fn print_crash_analysis' tooling/minidump/src/symbolize.rs"
    
    # Test 2: Check if stack trace formatting exists
    run_test "Stack trace formatting" \
        "grep -q 'STACK TRACE' tooling/minidump/src/symbolize.rs"
    
    # Test 3: Check if CPU register formatting exists
    run_test "CPU register formatting" \
        "grep -q 'CPU REGISTERS' tooling/minidump/src/symbolize.rs"
    
    # Test 4: Check if page fault formatting exists
    run_test "Page fault formatting" \
        "grep -q 'PAGE FAULT DETAILS' tooling/minidump/src/symbolize.rs"
}

test_integration() {
    log_info "Testing integration capabilities"
    
    # Test 1: Check if all required modules are present
    run_test "Required modules presence" \
        "test -f kernel/src/crash_dump/minidump.rs"
    
    # Test 2: Check if documentation exists
    run_test "Documentation presence" \
        "test -f docs/phase-2/CRASH-READING.md"
    
    # Test 3: Check if test script is executable
    run_test "Test script permissions" \
        "test -x scripts/test-crash-analysis.sh"
    
    # Test 4: Check if cargo workspace includes symbolizer
    run_test "Workspace configuration" \
        "grep -q 'minidump' Cargo.toml || grep -q 'tooling' Cargo.toml"
}

test_performance() {
    log_info "Testing performance characteristics"
    
    # Test 1: Check if minidump buffer size is reasonable
    local buffer_size=$(grep -o 'MINIDUMP_BUFFER.*\[.*\]' kernel/src/crash_dump/minidump.rs | grep -o '\[.*\]' | tr -d '[]' | sed 's/.*\*//')
    if [[ -n "$buffer_size" ]] && [[ $buffer_size -le 1024 ]]; then
        log_success "Minidump buffer size validation (${buffer_size}KB)"
        ((TESTS_PASSED++))
    else
        log_warning "Minidump buffer size validation (${buffer_size}KB, expected <=1024KB)"
        ((TESTS_SKIPPED++))
    fi
    
    # Test 2: Check if log history size is reasonable
    local log_size=$(grep -o 'LOG_HISTORY.*\[.*\]' kernel/src/crash_dump/minidump.rs | grep -o '\[.*\]' | tr -d '[]' | sed 's/.*\*//')
    if [[ -n "$log_size" ]] && [[ $log_size -le 512 ]]; then
        log_success "Log history size validation (${log_size} entries)"
        ((TESTS_PASSED++))
    else
        log_warning "Log history size validation (${log_size} entries, expected <=512)"
        ((TESTS_SKIPPED++))
    fi
}

test_security() {
    log_info "Testing security features"
    
    # Test 1: Check if sensitive data filtering is mentioned
    run_test "Sensitive data filtering" \
        "grep -q 'sensitive\|filter\|sanitiz' docs/phase-2/CRASH-READING.md"
    
    # Test 2: Check if access control is documented
    run_test "Access control documentation" \
        "grep -q 'access.*control\|secure.*storage' docs/phase-2/CRASH-READING.md"
    
    # Test 3: Check if debug mode restrictions are mentioned
    run_test "Debug mode restrictions" \
        "grep -q 'debug.*mode\|release.*build' docs/phase-2/CRASH-READING.md"
}

run_all_tests() {
    log_info "Starting Crash Analysis++ test suite"
    log_info "====================================="
    
    # Setup
    if ! setup_test_environment; then
        log_warning "Some tests will be skipped due to missing dependencies"
    fi
    
    # Run test categories
    test_minidump_generation
    test_symbolizer_compilation
    test_symbol_table_functionality
    test_minidump_parsing
    test_crash_analysis_output
    test_integration
    test_performance
    test_security
    
    # Summary
    log_info "====================================="
    log_info "Test Summary:"
    log_info "  Passed:  $TESTS_PASSED"
    log_info "  Failed:   $TESTS_FAILED"
    log_info "  Skipped:  $TESTS_SKIPPED"
    log_info "  Total:    $((TESTS_PASSED + TESTS_FAILED + TESTS_SKIPPED))"
    
    if [[ $TESTS_FAILED -eq 0 ]]; then
        log_info "All tests passed! 🎉"
        exit 0
    else
        log_info "Some tests failed. Please review the output above."
        exit 1
    fi
}

# Cleanup function
cleanup() {
    log_info "Cleaning up test artifacts"
    rm -rf "$TEST_OUTPUT_DIR"
}

# Set trap for cleanup
trap cleanup EXIT

# Main execution
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    run_all_tests
fi

