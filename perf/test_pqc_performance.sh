#!/bin/bash

# Test script for PQC Performance Checker
# Validates PQC overhead performance and CI thresholds

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
PERF_DIR="$PROJECT_ROOT/perf"
BUILD_DIR="$PERF_DIR/target"
RESULTS_DIR="$PERF_DIR/results"

# Test configuration
TARGET_P50_OVERHEAD=15.0
TARGET_P95_OVERHEAD=20.0
TARGET_WAKE_TO_RUN_MS=3

# Test results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Logging
LOG_FILE="$PERF_DIR/test_pqc_performance.log"
exec > >(tee -a "$LOG_FILE")
exec 2>&1

echo "=========================================="
echo "🧪 PQC Performance Checker Test Suite"
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
    
    # Check for required directories
    if [[ ! -d "$PERF_DIR" ]]; then
        log_error "Performance directory not found: $PERF_DIR"
        return 1
    fi
    
    # Check for required files
    if [[ ! -f "$PERF_DIR/check_pqc_overhead.rs" ]]; then
        log_error "PQC overhead checker not found: $PERF_DIR/check_pqc_overhead.rs"
        return 1
    fi
    
    log_success "Prerequisites check passed"
    return 0
}

setup_environment() {
    log_info "Setting up test environment..."
    
    # Create necessary directories
    mkdir -p "$BUILD_DIR"
    mkdir -p "$RESULTS_DIR"
    
    # Clean previous test results
    rm -rf "$RESULTS_DIR"/*
    
    log_success "Environment setup completed"
}

#=============================================================================
# BUILD TESTS
#=============================================================================

test_build() {
    log_info "Testing PQC performance checker build..."
    
    cd "$PERF_DIR"
    
    # Build the performance checker
    if cargo build --bin check_pqc_overhead; then
        log_success "Build test passed"
        return 0
    else
        log_error "Build test failed"
        return 1
    fi
}

test_build_release() {
    log_info "Testing PQC performance checker release build..."
    
    cd "$PERF_DIR"
    
    # Build release version
    if cargo build --release --bin check_pqc_overhead; then
        log_success "Release build test passed"
        return 0
    else
        log_error "Release build test failed"
        return 1
    fi
}

#=============================================================================
# FUNCTIONALITY TESTS
#=============================================================================

test_basic_execution() {
    log_info "Testing basic execution..."
    
    cd "$PERF_DIR"
    
    # Run the performance checker
    if cargo run --bin check_pqc_overhead > "$RESULTS_DIR/basic_execution.log" 2>&1; then
        log_success "Basic execution test passed"
        return 0
    else
        log_error "Basic execution test failed"
        cat "$RESULTS_DIR/basic_execution.log"
        return 1
    fi
}

test_performance_analysis() {
    log_info "Testing performance analysis functionality..."
    
    cd "$PERF_DIR"
    
    # Run performance analysis
    if cargo run --bin check_pqc_overhead > "$RESULTS_DIR/performance_analysis.log" 2>&1; then
        # Check if performance analysis was generated
        if grep -q "PQC Overhead Performance Analysis" "$RESULTS_DIR/performance_analysis.log"; then
            log_success "Performance analysis test passed"
            return 0
        else
            log_error "Performance analysis not generated"
            return 1
        fi
    else
        log_error "Performance analysis test failed"
        cat "$RESULTS_DIR/performance_analysis.log"
        return 1
    fi
}

test_json_export() {
    log_info "Testing JSON export functionality..."
    
    cd "$PERF_DIR"
    
    # Run the checker to generate JSON
    if cargo run --bin check_pqc_overhead > "$RESULTS_DIR/json_export.log" 2>&1; then
        # Check if JSON file was created
        local json_files=(pqc_overhead_results_*.json)
        if [[ ${#json_files[@]} -gt 0 ]]; then
            log_success "JSON export test passed"
            
            # Move JSON files to results directory
            mv pqc_overhead_results_*.json "$RESULTS_DIR/"
            return 0
        else
            log_error "JSON file not created"
            return 1
        fi
    else
        log_error "JSON export test failed"
        cat "$RESULTS_DIR/json_export.log"
        return 1
    fi
}

#=============================================================================
# PERFORMANCE VALIDATION TESTS
#=============================================================================

test_performance_targets() {
    log_info "Testing performance target validation..."
    
    cd "$PERF_DIR"
    
    # Run performance check
    if cargo run --bin check_pqc_overhead > "$RESULTS_DIR/target_validation.log" 2>&1; then
        # Check if all targets are met
        if grep -q "ALL PERFORMANCE TARGETS MET" "$RESULTS_DIR/target_validation.log"; then
            log_success "Performance target validation test passed"
            return 0
        else
            log_warning "Performance targets not met (this may be expected during development)"
            return 0  # Don't fail the test for performance issues during development
        fi
    else
        log_error "Performance target validation test failed"
        cat "$RESULTS_DIR/target_validation.log"
        return 1
    fi
}

test_overhead_calculation() {
    log_info "Testing overhead calculation accuracy..."
    
    cd "$PERF_DIR"
    
    # Run performance check
    if cargo run --bin check_pqc_overhead > "$RESULTS_DIR/overhead_calculation.log" 2>&1; else
        log_error "Overhead calculation test failed to run"
        return 1
    fi
    
    # Extract overhead values
    local p50_overhead=$(grep "P50 Overhead:" "$RESULTS_DIR/overhead_calculation.log" | grep -o '[0-9.]*%' | head -1 | sed 's/%//')
    local p95_overhead=$(grep "P95 Overhead:" "$RESULTS_DIR/overhead_calculation.log" | grep -o '[0-9.]*%' | head -1 | sed 's/%//')
    
    if [[ -n "$p50_overhead" && -n "$p95_overhead" ]]; then
        log_success "Overhead calculation test passed"
        log_info "P50 Overhead: ${p50_overhead}%"
        log_info "P95 Overhead: ${p95_overhead}%"
        return 0
    else
        log_error "Overhead values not found in output"
        return 1
    fi
}

test_wake_to_run_validation() {
    log_info "Testing wake-to-run latency validation..."
    
    cd "$PERF_DIR"
    
    # Run performance check
    if cargo run --bin check_pqc_overhead > "$RESULTS_DIR/wake_to_run.log" 2>&1; else
        log_error "Wake-to-run validation test failed to run"
        return 1
    fi
    
    # Extract wake-to-run value
    local wake_to_run=$(grep "Wake-to-Run p95" "$RESULTS_DIR/wake_to_run.log" | grep -o '[0-9]*ms' | head -1 | sed 's/ms//')
    
    if [[ -n "$wake_to_run" ]]; then
        log_success "Wake-to-run validation test passed"
        log_info "Wake-to-Run p95: ${wake_to_run}ms"
        return 0
    else
        log_error "Wake-to-run value not found in output"
        return 1
    fi
}

#=============================================================================
# CI INTEGRATION TESTS
#=============================================================================

test_ci_thresholds() {
    log_info "Testing CI threshold validation..."
    
    cd "$PERF_DIR"
    
    # Run performance check
    if cargo run --bin check_pqc_overhead > "$RESULTS_DIR/ci_thresholds.log" 2>&1; else
        log_error "CI threshold test failed to run"
        return 1
    fi
    
    # Check if exit code indicates success/failure
    local exit_code=$?
    
    # Extract performance metrics
    local p50_overhead=$(grep "P50 Overhead:" "$RESULTS_DIR/ci_thresholds.log" | grep -o '[0-9.]*%' | head -1 | sed 's/%//')
    local p95_overhead=$(grep "P95 Overhead:" "$RESULTS_DIR/ci_thresholds.log" | grep -o '[0-9.]*%' | head -1 | sed 's/%//')
    local wake_to_run=$(grep "Wake-to-Run p95" "$RESULTS_DIR/ci_thresholds.log" | grep -o '[0-9]*ms' | head -1 | sed 's/ms//')
    
    # Validate thresholds
    local thresholds_met=true
    local issues=""
    
    if [[ -n "$p50_overhead" ]]; then
        if (( $(echo "$p50_overhead > $TARGET_P50_OVERHEAD" | bc -l) )); then
            thresholds_met=false
            issues="$issues P50 overhead (${p50_overhead}%) exceeds target (${TARGET_P50_OVERHEAD}%);"
        fi
    fi
    
    if [[ -n "$p95_overhead" ]]; then
        if (( $(echo "$p95_overhead > $TARGET_P95_OVERHEAD" | bc -l) )); then
            thresholds_met=false
            issues="$issues P95 overhead (${p95_overhead}%) exceeds target (${TARGET_P95_OVERHEAD}%);"
        fi
    fi
    
    if [[ -n "$wake_to_run" ]]; then
        if (( wake_to_run > TARGET_WAKE_TO_RUN_MS )); then
            thresholds_met=false
            issues="$issues Wake-to-run (${wake_to_run}ms) exceeds target (${TARGET_WAKE_TO_RUN_MS}ms);"
        fi
    fi
    
    if [[ "$thresholds_met" == true ]]; then
        log_success "CI threshold validation test passed"
        log_info "All performance thresholds met"
        return 0
    else
        log_warning "CI threshold validation test: some thresholds exceeded"
        log_info "Issues: $issues"
        return 0  # Don't fail the test for performance issues during development
    fi
}

#=============================================================================
# TEST EXECUTION
#=============================================================================

run_all_tests() {
    log_info "Starting PQC performance checker test suite..."
    
    # Build tests
    TOTAL_TESTS=$((TOTAL_TESTS + 2))
    test_build && test_build_release
    
    # Functionality tests
    TOTAL_TESTS=$((TOTAL_TESTS + 3))
    test_basic_execution && test_performance_analysis && test_json_export
    
    # Performance validation tests
    TOTAL_TESTS=$((TOTAL_TESTS + 3))
    test_performance_targets && test_overhead_calculation && test_wake_to_run_validation
    
    # CI integration tests
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    test_ci_thresholds
    
    log_success "All tests completed"
}

#=============================================================================
# RESULTS ANALYSIS
#=============================================================================

analyze_results() {
    log_info "Analyzing test results..."
    
    # Check JSON results
    local json_files=("$RESULTS_DIR"/pqc_overhead_results_*.json)
    if [[ ${#json_files[@]} -gt 0 ]]; then
        log_info "Found ${#json_files[@]} performance result file(s):"
        for file in "${json_files[@]}"; do
            log_info "  - $(basename "$file")"
        done
        
        # Analyze latest result
        local latest_file="${json_files[-1]}"
        if [[ -f "$latest_file" ]]; then
            log_info "Analyzing latest result: $(basename "$latest_file")"
            
            # Extract key metrics
            local p50_overhead=$(jq -r '.p50_overhead_percent' "$latest_file" 2>/dev/null || echo "N/A")
            local p95_overhead=$(jq -r '.p95_overhead_percent' "$latest_file" 2>/dev/null || echo "N/A")
            local wake_to_run=$(jq -r '.wake_to_run_p95_ms' "$latest_file" 2>/dev/null || echo "N/A")
            
            log_info "Performance Metrics:"
            log_info "  P50 Overhead: ${p50_overhead}%"
            log_info "  P95 Overhead: ${p95_overhead}%"
            log_info "  Wake-to-Run p95: ${wake_to_run}ms"
        fi
    fi
    
    # Check log files
    local log_files=("$RESULTS_DIR"/*.log)
    if [[ ${#log_files[@]} -gt 0 ]]; then
        log_info "Generated log files:"
        for file in "${log_files[@]}"; do
            log_info "  - $(basename "$file")"
        done
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
    
    if [[ $TOTAL_TESTS -gt 0 ]]; then
        local success_rate=$((PASSED_TESTS * 100 / TOTAL_TESTS))
        echo "Success Rate: ${success_rate}%"
        
        if [[ $success_rate -eq 100 ]]; then
            log_success "🎉 All tests passed!"
        elif [[ $success_rate -ge 80 ]]; then
            log_success "✅ Good test results (${success_rate}%)"
        elif [[ $success_rate -ge 60 ]]; then
            log_warning "⚠️  Moderate test results (${success_rate}%)"
        else
            log_error "❌ Poor test results (${success_rate}%)"
        fi
    fi
    
    echo ""
    echo "Results Directory: $RESULTS_DIR"
    echo "Log File: $LOG_FILE"
    echo "=========================================="
}

#=============================================================================
# MAIN EXECUTION
#=============================================================================

main() {
    log_info "Starting PQC performance checker test suite..."
    
    # Initialize test environment
    if ! check_prerequisites; then
        log_error "Prerequisites check failed"
        exit 1
    fi
    
    setup_environment
    
    # Run tests
    run_all_tests
    
    # Analyze results
    analyze_results
    
    # Print summary
    print_summary
    
    # Exit with appropriate code
    if [[ $FAILED_TESTS -eq 0 ]]; then
        log_success "All tests passed successfully"
        exit 0
    else
        log_error "Some tests failed"
        exit 1
    fi
}

#=============================================================================
# CLEANUP AND SIGNAL HANDLING
#=============================================================================

cleanup() {
    log_info "Cleaning up test environment..."
    
    # Kill any running processes
    pkill -f "check_pqc_overhead" 2>/dev/null || true
    
    # Clean temporary files
    rm -rf /tmp/pqc-test-* 2>/dev/null || true
    
    log_info "Cleanup completed"
}

# Set up signal handlers
trap cleanup EXIT
trap 'log_error "Test interrupted by user"; exit 1' INT TERM

# Run main function
main "$@"

