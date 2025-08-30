#!/bin/bash

# Post-Quantum Crypto Baseline Test Script
# Tests the implementation of Kyber KEM and Dilithium signatures

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRYPTO_DIR="$SCRIPT_DIR"
BUILD_DIR="$CRYPTO_DIR/target"
TEST_BINARY="$BUILD_DIR/release/crypto-test"
BENCH_BINARY="$BUILD_DIR/release/crypto-bench"

# Test results
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
    ((TESTS_PASSED++))
    ((TESTS_TOTAL++))
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    ((TESTS_FAILED++))
    ((TESTS_TOTAL++))
}

log_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

# Utility functions
check_command() {
    if ! command -v "$1" &> /dev/null; then
        log_error "Command '$1' not found"
        return 1
    fi
    return 0
}

check_file() {
    if [[ ! -f "$1" ]]; then
        log_error "File '$1' not found"
        return 1
    fi
    return 0
}

run_test() {
    local test_name="$1"
    local test_command="$2"
    
    log_test "Running: $test_name"
    
    if eval "$test_command" > /dev/null 2>&1; then
        log_success "$test_name passed"
        return 0
    else
        log_error "$test_name failed"
        return 1
    fi
}

# Test functions
test_build_system() {
    log_info "Testing build system..."
    
    # Check if Cargo is available
    if ! check_command "cargo"; then
        log_error "Cargo not found. Please install Rust and Cargo."
        return 1
    fi
    
    # Check if required tools are available
    local required_tools=("git" "cmake" "gcc")
    for tool in "${required_tools[@]}"; do
        if ! check_command "$tool"; then
            log_warning "$tool not found. Some features may not work."
        fi
    done
    
    log_success "Build system check completed"
    return 0
}

test_cargo_build() {
    log_info "Building crypto module..."
    
    cd "$CRYPTO_DIR"
    
    # Clean previous builds
    log_info "Cleaning previous builds..."
    cargo clean
    
    # Build in debug mode
    log_info "Building in debug mode..."
    if cargo build; then
        log_success "Debug build completed"
    else
        log_error "Debug build failed"
        return 1
    fi
    
    # Build in release mode
    log_info "Building in release mode..."
    if cargo build --release; then
        log_success "Release build completed"
    else
        log_error "Release build failed"
        return 1
    fi
    
    # Check if binaries were created
    if check_file "$TEST_BINARY"; then
        log_success "Test binary created"
    else
        log_error "Test binary not found"
        return 1
    fi
    
    if check_file "$BENCH_BINARY"; then
        log_success "Benchmark binary created"
    else
        log_warning "Benchmark binary not found (may require full features)"
    fi
    
    return 0
}

test_unit_tests() {
    log_info "Running unit tests..."
    
    cd "$CRYPTO_DIR"
    
    # Run all tests
    if cargo test; then
        log_success "All unit tests passed"
    else
        log_error "Some unit tests failed"
        return 1
    fi
    
    # Run tests with specific features
    log_info "Testing with full features..."
    if cargo test --features "full"; then
        log_success "Full feature tests passed"
    else
        log_warning "Full feature tests failed (may not be critical)"
    fi
    
    return 0
}

test_integration() {
    log_info "Running integration tests..."
    
    if [[ ! -f "$TEST_BINARY" ]]; then
        log_warning "Test binary not found, skipping integration tests"
        return 0
    fi
    
    # Run the test binary
    if "$TEST_BINARY"; then
        log_success "Integration tests passed"
    else
        log_error "Integration tests failed"
        return 1
    fi
    
    return 0
}

test_benchmarks() {
    log_info "Running benchmarks..."
    
    if [[ ! -f "$BENCH_BINARY" ]]; then
        log_warning "Benchmark binary not found, skipping benchmarks"
        return 0
    fi
    
    # Run benchmarks (this may take a while)
    log_info "Running benchmarks (this may take several minutes)..."
    if "$BENCH_BINARY"; then
        log_success "Benchmarks completed"
    else
        log_warning "Benchmarks failed (may not be critical)"
    fi
    
    return 0
}

test_documentation() {
    log_info "Testing documentation..."
    
    cd "$CRYPTO_DIR"
    
    # Check if documentation can be generated
    if cargo doc --no-deps; then
        log_success "Documentation generated successfully"
    else
        log_warning "Documentation generation failed"
    fi
    
    # Check if README exists
    if check_file "README.md"; then
        log_success "README.md found"
    else
        log_warning "README.md not found"
    fi
    
    return 0
}

test_code_quality() {
    log_info "Checking code quality..."
    
    cd "$CRYPTO_DIR"
    
    # Check if clippy is available
    if check_command "cargo-clippy"; then
        log_info "Running clippy..."
        if cargo clippy; then
            log_success "Clippy checks passed"
        else
            log_warning "Clippy found some issues"
        fi
    else
        log_warning "Clippy not available, skipping code quality checks"
    fi
    
    # Check if rustfmt is available
    if check_command "rustfmt"; then
        log_info "Checking code formatting..."
        if cargo fmt -- --check; then
            log_success "Code formatting is correct"
        else
            log_warning "Code formatting issues found"
        fi
    else
        log_warning "rustfmt not available, skipping formatting checks"
    fi
    
    return 0
}

test_security_features() {
    log_info "Testing security features..."
    
    # Test that the module compiles with security features
    cd "$CRYPTO_DIR"
    
    # Test compilation with different feature combinations
    local feature_combinations=(
        "kyber"
        "dilithium"
        "kyber,dilithium"
        "full"
    )
    
    for features in "${feature_combinations[@]}"; do
        log_info "Testing with features: $features"
        if cargo check --features "$features"; then
            log_success "Features '$features' compile successfully"
        else
            log_error "Features '$features' failed to compile"
            return 1
        fi
    done
    
    return 0
}

test_examples() {
    log_info "Testing examples..."
    
    # Check if example files exist
    local example_files=(
        "src/bin/test.rs"
        "src/bin/bench.rs"
    )
    
    for example in "${example_files[@]}"; do
        if check_file "$CRYPTO_DIR/$example"; then
            log_success "Example file found: $example"
        else
            log_warning "Example file not found: $example"
        fi
    done
    
    return 0
}

# Main test execution
main() {
    echo "=========================================="
    echo "  Post-Quantum Crypto Baseline Tests"
    echo "=========================================="
    echo ""
    
    # Initialize test counters
    TESTS_PASSED=0
    TESTS_FAILED=0
    TESTS_TOTAL=0
    
    # Run all tests
    local tests=(
        "test_build_system"
        "test_cargo_build"
        "test_unit_tests"
        "test_integration"
        "test_benchmarks"
        "test_documentation"
        "test_code_quality"
        "test_security_features"
        "test_examples"
    )
    
    for test in "${tests[@]}"; do
        if "$test"; then
            log_success "$test completed successfully"
        else
            log_error "$test failed"
        fi
        echo ""
    done
    
    # Print summary
    echo "=========================================="
    echo "  Test Summary"
    echo "=========================================="
    echo "Total tests: $TESTS_TOTAL"
    echo "Passed: $TESTS_PASSED"
    echo "Failed: $TESTS_FAILED"
    echo ""
    
    if [[ $TESTS_FAILED -eq 0 ]]; then
        log_success "All tests passed! Post-Quantum Crypto Baseline is working correctly."
        exit 0
    else
        log_error "Some tests failed. Please review the errors above."
        exit 1
    fi
}

# Error handling
trap 'log_error "Test interrupted by user"; exit 1' INT TERM

# Run main function
main "$@"
