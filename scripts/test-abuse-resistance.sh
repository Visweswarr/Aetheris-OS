#!/bin/bash

# Test script for Abuse Resistance System
# Tests table-driven syscall conformance and libFuzzer targets

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FUZZ_DIR="$PROJECT_ROOT/fuzz/rust"
TESTS_DIR="$PROJECT_ROOT/tests"
BUILD_DIR="$PROJECT_ROOT/target"
CORPUS_DIR="$FUZZ_DIR/corpus"
ARTIFACTS_DIR="$FUZZ_DIR/artifacts"

# Test configuration
CONFORMANCE_TIMEOUT=300      # 5 minutes for conformance tests
FUZZ_TIMEOUT=60              # 60 seconds for fuzz runs
SMOKE_TIMEOUT=30             # 30 seconds for smoke tests
NIGHTLY_TIMEOUT=3600         # 1 hour for nightly runs

# Test results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
CRASHES_FOUND=0

# Logging
LOG_FILE="$PROJECT_ROOT/test-abuse-resistance.log"
exec > >(tee -a "$LOG_FILE")
exec 2>&1

echo "=========================================="
echo "🧪 Polymera OS Abuse Resistance Testing"
echo "=========================================="
echo "Timestamp: $(date)"
echo "Project: $PROJECT_ROOT"
echo "Log file: $LOG_FILE"
echo ""

#=============================================================================
# UTILITY FUNCTIONS
#=============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

run_test() {
    local test_name="$1"
    local test_cmd="$2"
    local timeout="${3:-60}"
    
    log_info "Running: $test_name"
    log_info "Command: $test_cmd"
    log_info "Timeout: ${timeout}s"
    
    local start_time=$(date +%s)
    
    if timeout "$timeout" bash -c "$test_cmd" > /dev/null 2>&1; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        log_success "$test_name PASSED (${duration}s)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        log_error "$test_name FAILED (${duration}s)"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check for Rust
    if ! command -v cargo &> /dev/null; then
        log_error "Rust/cargo not found. Please install Rust first."
        return 1
    fi
    
    # Check for libFuzzer
    if ! cargo install --list | grep -q "cargo-fuzz"; then
        log_warning "cargo-fuzz not found. Installing..."
        cargo install cargo-fuzz
    fi
    
    # Check for required directories
    if [[ ! -d "$FUZZ_DIR" ]]; then
        log_error "Fuzz directory not found: $FUZZ_DIR"
        return 1
    fi
    
    if [[ ! -d "$TESTS_DIR" ]]; then
        log_error "Tests directory not found: $TESTS_DIR"
        return 1
    fi
    
    log_success "Prerequisites check passed"
    return 0
}

setup_environment() {
    log_info "Setting up test environment..."
    
    # Create necessary directories
    mkdir -p "$BUILD_DIR"
    mkdir -p "$CORPUS_DIR"
    mkdir -p "$ARTIFACTS_DIR"
    
    # Clean previous test artifacts
    rm -rf "$ARTIFACTS_DIR"/*
    
    # Initialize corpus if empty
    if [[ ! -d "$CORPUS_DIR/cap_parser" ]]; then
        mkdir -p "$CORPUS_DIR/cap_parser"
        # Add some basic test cases
        echo "test" > "$CORPUS_DIR/cap_parser/seed"
    fi
    
    if [[ ! -d "$CORPUS_DIR/syscall_decoder" ]]; then
        mkdir -p "$CORPUS_DIR/syscall_decoder"
        echo "test" > "$CORPUS_DIR/syscall_decoder/seed"
    fi
    
    if [[ ! -d "$CORPUS_DIR/ipc_header" ]]; then
        mkdir -p "$CORPUS_DIR/ipc_header"
        echo "test" > "$CORPUS_DIR/ipc_header/seed"
    fi
    
    log_success "Environment setup completed"
}

#=============================================================================
# CONFORMANCE TESTING
#=============================================================================

run_conformance_tests() {
    log_info "Running syscall conformance tests..."
    
    cd "$PROJECT_ROOT"
    
    # Build the kernel to ensure conformance tests compile
    log_info "Building kernel for conformance tests..."
    if ! cargo build --package kernel --release; then
        log_error "Kernel build failed"
        return 1
    fi
    
    # Run conformance tests
    log_info "Running conformance tests..."
    if ! cargo test --package kernel conformance; then
        log_error "Conformance tests failed"
        return 1
    fi
    
    # Run specific conformance test
    log_info "Running abuse resistance conformance tests..."
    if ! cargo test --package kernel test_abuse_resistance_conformance; then
        log_error "Abuse resistance conformance tests failed"
        return 1
    fi
    
    log_success "Conformance tests completed"
    return 0
}

#=============================================================================
# FUZZING TESTS
#=============================================================================

run_fuzz_tests() {
    log_info "Running fuzzing tests..."
    
    cd "$FUZZ_DIR"
    
    # Build fuzz targets
    log_info "Building fuzz targets..."
    if ! cargo fuzz build; then
        log_error "Fuzz target build failed"
        return 1
    fi
    
    # Run smoke tests (30 seconds each)
    run_fuzz_smoke_tests
    
    # Run full fuzz tests (60 seconds each)
    run_fuzz_full_tests
    
    # Run nightly fuzz tests (1 hour each) if requested
    if [[ "${1:-}" == "--nightly" ]]; then
        run_fuzz_nightly_tests
    fi
    
    log_success "Fuzzing tests completed"
    return 0
}

run_fuzz_smoke_tests() {
    log_info "Running fuzz smoke tests (${SMOKE_TIMEOUT}s each)..."
    
    local fuzz_targets=("cap_parser" "syscall_decoder" "ipc_header")
    
    for target in "${fuzz_targets[@]}"; do
        log_info "Running smoke test for $target..."
        
        if timeout "$SMOKE_TIMEOUT" cargo fuzz run "$target" -- -max_total_time="$SMOKE_TIMEOUT" -max_len=1024 > /dev/null 2>&1; then
            log_success "$target smoke test PASSED"
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            log_error "$target smoke test FAILED"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            CRASHES_FOUND=$((CRASHES_FOUND + 1))
        fi
        
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
    done
}

run_fuzz_full_tests() {
    log_info "Running full fuzz tests (${FUZZ_TIMEOUT}s each)..."
    
    local fuzz_targets=("cap_parser" "syscall_decoder" "ipc_header")
    
    for target in "${fuzz_targets[@]}"; do
        log_info "Running full fuzz test for $target..."
        
        if timeout "$FUZZ_TIMEOUT" cargo fuzz run "$target" -- -max_total_time="$FUZZ_TIMEOUT" -max_len=4096 > /dev/null 2>&1; then
            log_success "$target full fuzz test PASSED"
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            log_error "$target full fuzz test FAILED"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            CRASHES_FOUND=$((CRASHES_FOUND + 1))
        fi
        
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
    done
}

run_fuzz_nightly_tests() {
    log_info "Running nightly fuzz tests (${NIGHTLY_TIMEOUT}s each)..."
    
    local fuzz_targets=("cap_parser" "syscall_decoder" "ipc_header")
    
    for target in "${fuzz_targets[@]}"; do
        log_info "Running nightly fuzz test for $target..."
        
        # Create corpus directory for this target
        local target_corpus="$CORPUS_DIR/$target"
        mkdir -p "$target_corpus"
        
        if timeout "$NIGHTLY_TIMEOUT" cargo fuzz run "$target" -- -max_total_time="$NIGHTLY_TIMEOUT" -max_len=8192 -merge=1 "$target_corpus" > /dev/null 2>&1; then
            log_success "$target nightly fuzz test PASSED"
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            log_error "$target nightly fuzz test FAILED"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            CRASHES_FOUND=$((CRASHES_FOUND + 1))
        fi
        
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
    done
}

#=============================================================================
# CORPUS MANAGEMENT
#=============================================================================

manage_corpus() {
    log_info "Managing fuzzing corpus..."
    
    cd "$FUZZ_DIR"
    
    # Update corpus for each target
    local fuzz_targets=("cap_parser" "syscall_decoder" "ipc_header")
    
    for target in "${fuzz_targets[@]}"; do
        log_info "Updating corpus for $target..."
        
        local target_corpus="$CORPUS_DIR/$target"
        local target_artifacts="$ARTIFACTS_DIR/$target"
        
        mkdir -p "$target_corpus"
        mkdir -p "$target_artifacts"
        
        # Run fuzzer to update corpus
        if timeout 300 cargo fuzz run "$target" -- -max_total_time=300 -merge=1 "$target_corpus" > /dev/null 2>&1; then
            log_success "$target corpus updated successfully"
            
            # Copy interesting inputs to artifacts
            if [[ -d "$target_corpus" ]] && [[ "$(ls -A "$target_corpus")" ]]; then
                cp -r "$target_corpus"/* "$target_artifacts/"
                log_info "$target artifacts saved to $target_artifacts"
            fi
        else
            log_warning "$target corpus update failed"
        fi
    done
    
    log_success "Corpus management completed"
}

#=============================================================================
# CRASH ANALYSIS
#=============================================================================

analyze_crashes() {
    log_info "Analyzing fuzzing crashes..."
    
    cd "$FUZZ_DIR"
    
    local crashes_found=false
    
    # Check for crashes in each target
    local fuzz_targets=("cap_parser" "syscall_decoder" "ipc_header")
    
    for target in "${fuzz_targets[@]}"; do
        local target_artifacts="$ARTIFACTS_DIR/$target"
        
        if [[ -d "$target_artifacts" ]] && [[ "$(ls -A "$target_artifacts")" ]]; then
            log_warning "Found artifacts for $target - potential crashes detected"
            crashes_found=true
            
            # List artifacts
            log_info "Artifacts for $target:"
            ls -la "$target_artifacts"
        fi
    done
    
    if [[ "$crashes_found" == false ]]; then
        log_success "No crashes detected in fuzzing runs"
    else
        log_error "Crashes detected! Review artifacts in $ARTIFACTS_DIR"
        CRASHES_FOUND=$((CRASHES_FOUND + 1))
    fi
}

#=============================================================================
# PERFORMANCE VALIDATION
#=============================================================================

validate_performance() {
    log_info "Validating performance targets..."
    
    # Check conformance test performance
    local start_time=$(date +%s)
    run_conformance_tests
    local end_time=$(date +%s)
    local conformance_duration=$((end_time - start_time))
    
    # Check fuzz test performance
    local start_time=$(date +%s)
    run_fuzz_smoke_tests
    local end_time=$(date +%s)
    local fuzz_duration=$((end_time - start_time))
    
    log_info "Performance Summary:"
    log_info "  Conformance tests: ${conformance_duration}s"
    log_info "  Fuzz smoke tests: ${fuzz_duration}s"
    
    # Validate performance targets
    if [[ $conformance_duration -le $CONFORMANCE_TIMEOUT ]]; then
        log_success "Conformance tests meet performance target (${conformance_duration}s <= ${CONFORMANCE_TIMEOUT}s)"
    else
        log_warning "Conformance tests exceed performance target (${conformance_duration}s > ${CONFORMANCE_TIMEOUT}s)"
    fi
    
    if [[ $fuzz_duration -le $FUZZ_TIMEOUT ]]; then
        log_success "Fuzz tests meet performance target (${fuzz_duration}s <= ${FUZZ_TIMEOUT}s)"
    else
        log_warning "Fuzz tests exceed performance target (${fuzz_duration}s > ${FUZZ_TIMEOUT}s)"
    fi
}

#=============================================================================
# MAIN TEST EXECUTION
#=============================================================================

main() {
    local nightly_mode=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --nightly)
                nightly_mode=true
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --nightly    Run nightly fuzz tests (1 hour each)"
                echo "  --help, -h   Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
    
    log_info "Starting abuse resistance testing..."
    log_info "Nightly mode: $nightly_mode"
    
    # Initialize test environment
    if ! check_prerequisites; then
        log_error "Prerequisites check failed"
        exit 1
    fi
    
    setup_environment
    
    # Run tests
    local test_failed=false
    
    # Run conformance tests
    if ! run_conformance_tests; then
        test_failed=true
    fi
    
    # Run fuzzing tests
    if ! run_fuzz_tests; then
        test_failed=true
    fi
    
    # Run nightly tests if requested
    if [[ "$nightly_mode" == true ]]; then
        if ! run_fuzz_nightly_tests; then
            test_failed=true
        fi
    fi
    
    # Manage corpus
    manage_corpus
    
    # Analyze crashes
    analyze_crashes
    
    # Validate performance
    validate_performance
    
    # Print summary
    print_summary
    
    # Exit with appropriate code
    if [[ "$test_failed" == true ]]; then
        log_error "Some tests failed"
        exit 1
    else
        log_success "All tests passed"
        exit 0
    fi
}

print_summary() {
    echo ""
    echo "=========================================="
    echo "📊 Test Summary"
    echo "=========================================="
    echo "Total Tests: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    echo "Crashes Found: $CRASHES_FOUND"
    
    if [[ $TOTAL_TESTS -gt 0 ]]; then
        local success_rate=$((PASSED_TESTS * 100 / TOTAL_TESTS))
        echo "Success Rate: ${success_rate}%"
        
        if [[ $success_rate -eq 100 ]]; then
            log_success "🎉 100% conformance achieved!"
        elif [[ $success_rate -ge 95 ]]; then
            log_success "✅ Excellent conformance (${success_rate}%)"
        elif [[ $success_rate -ge 90 ]]; then
            log_warning "⚠️  Good conformance (${success_rate}%)"
        else
            log_error "❌ Poor conformance (${success_rate}%)"
        fi
    fi
    
    if [[ $CRASHES_FOUND -eq 0 ]]; then
        log_success "🛡️  No crashes detected - system is abuse resistant"
    else
        log_error "💥 $CRASHES_FOUND crashes detected - review required"
    fi
    
    echo ""
    echo "Log file: $LOG_FILE"
    echo "Artifacts: $ARTIFACTS_DIR"
    echo "Corpus: $CORPUS_DIR"
    echo "=========================================="
}

#=============================================================================
# CLEANUP AND SIGNAL HANDLING
#=============================================================================

cleanup() {
    log_info "Cleaning up test environment..."
    
    # Kill any running fuzz processes
    pkill -f "cargo fuzz" 2>/dev/null || true
    
    # Clean temporary files
    rm -rf /tmp/polymera-fuzz-* 2>/dev/null || true
    
    log_info "Cleanup completed"
}

# Set up signal handlers
trap cleanup EXIT
trap 'log_error "Test interrupted by user"; exit 1' INT TERM

# Run main function
main "$@"
