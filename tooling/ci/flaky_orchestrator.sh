#!/bin/bash

# Flaky Test Orchestrator for Polymera OS Phase 2 Matrix
# This script handles test retries, flaky detection, and issue creation

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
CONFIG_ID=""
MATRIX_NAME=""
FAILED_TESTS_FILE=""
MAX_RETRIES=2
RESULTS_DIR=""
RUN_ID=""
VERBOSE=false

# Help function
show_help() {
    cat << EOF
Usage: $0 [OPTIONS]

Flaky Test Orchestrator for Polymera OS Phase 2 Matrix

OPTIONS:
    --config-id ID          Matrix configuration ID
    --matrix-name NAME      Matrix configuration name
    --failed-tests FILE     JSON file containing failed test names
    --max-retries N         Maximum number of retries (default: 2)
    --results-dir DIR       Directory for test results
    --run-id ID             GitHub Actions run ID
    --verbose               Enable verbose output
    --help                  Show this help message

EXAMPLE:
    $0 \\
        --config-id "apic_hpet_jitter_auth_open" \\
        --matrix-name "APIC+HPET, Jitter On, Auth On, Cap Policy Open" \\
        --failed-tests failed_tests.json \\
        --max-retries 2 \\
        --results-dir "test_results" \\
        --run-id "123456789"

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --config-id)
            CONFIG_ID="$2"
            shift 2
            ;;
        --matrix-name)
            MATRIX_NAME="$2"
            shift 2
            ;;
        --failed-tests)
            FAILED_TESTS_FILE="$2"
            shift 2
            ;;
        --max-retries)
            MAX_RETRIES="$2"
            shift 2
            ;;
        --results-dir)
            RESULTS_DIR="$2"
            shift 2
            ;;
        --run-id)
            RUN_ID="$2"
            shift 2
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Validate required arguments
if [[ -z "$CONFIG_ID" || -z "$MATRIX_NAME" || -z "$FAILED_TESTS_FILE" || -z "$RESULTS_DIR" || -z "$RUN_ID" ]]; then
    echo -e "${RED}Error: Missing required arguments${NC}"
    show_help
    exit 1
fi

# Log function
log() {
    local level="$1"
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    case "$level" in
        "INFO")
            echo -e "${BLUE}[$timestamp] INFO:${NC} $message"
            ;;
        "WARN")
            echo -e "${YELLOW}[$timestamp] WARN:${NC} $message"
            ;;
        "ERROR")
            echo -e "${RED}[$timestamp] ERROR:${NC} $message"
            ;;
        "SUCCESS")
            echo -e "${GREEN}[$timestamp] SUCCESS:${NC} $message"
            ;;
        *)
            echo -e "[$timestamp] $message"
            ;;
    esac
}

# Verbose logging
vlog() {
    if [[ "$VERBOSE" == "true" ]]; then
        log "INFO" "$*"
    fi
}

# Initialize analysis data structure
init_analysis() {
    cat > flaky_analysis.json << EOF
{
  "config_id": "$CONFIG_ID",
  "matrix_name": "$MATRIX_NAME",
  "run_id": "$RUN_ID",
  "max_retries": $MAX_RETRIES,
  "total_failed": 0,
  "flaky_tests": [],
  "definitive_failures": [],
  "retry_summary": "",
  "flaky_tests_found": false,
  "definitive_failures": false,
  "issue_needed": false,
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
}

# Update analysis with test results
update_analysis() {
    local test_name="$1"
    local attempt="$2"
    local result="$3"
    local log_file="$4"
    
    vlog "Updating analysis for test: $test_name (attempt $attempt, result: $result)"
    
    # Read current analysis
    local analysis=$(cat flaky_analysis.json)
    
    if [[ "$result" == "PASS" ]]; then
        # Test passed - check if it was previously failed (flaky)
        if jq -e ".definitive_failures[] | select(.name == \"$test_name\")" flaky_analysis.json > /dev/null 2>&1; then
            # Remove from definitive failures and add to flaky tests
            analysis=$(echo "$analysis" | jq --arg name "$test_name" --arg attempt "$attempt" --arg log "$log_file" '
                .definitive_failures = (.definitive_failures | map(select(.name != $name))) |
                .flaky_tests += [{
                    name: $name,
                    failed_attempts: [1, 2],
                    passed_attempt: ($attempt | tonumber),
                    log_file: $log_file
                }]
            ')
            
            log "SUCCESS" "Test '$test_name' passed on attempt $attempt - marked as flaky"
        fi
    else
        # Test failed - add to definitive failures if not already there
        if ! jq -e ".definitive_failures[] | select(.name == \"$test_name\")" flaky_analysis.json > /dev/null 2>&1; then
            analysis=$(echo "$analysis" | jq --arg name "$test_name" --arg attempt "$attempt" --arg log "$log_file" '
                .definitive_failures += [{
                    name: $name,
                    total_attempts: ($attempt | tonumber),
                    log_file: $log_file,
                    last_failure: "Attempt $attempt"
                }]
            ')
        else
            # Update attempt count for existing failure
            analysis=$(echo "$analysis" | jq --arg name "$test_name" --arg attempt "$attempt" '
                .definitive_failures |= map(
                    if .name == $name then
                        .total_attempts = ($attempt | tonumber) |
                        .last_failure = "Attempt $attempt"
                    else . end
                )
            ')
        fi
        
        log "WARN" "Test '$test_name' failed on attempt $attempt"
    fi
    
    # Update total failed count
    local total_failed=$(echo "$analysis" | jq '.definitive_failures | length')
    analysis=$(echo "$analysis" | jq --arg count "$total_failed" '.total_failed = ($count | tonumber)')
    
    # Update flags
    local flaky_count=$(echo "$analysis" | jq '.flaky_tests | length')
    local definitive_count=$(echo "$analysis" | jq '.definitive_failures | length')
    
    analysis=$(echo "$analysis" | jq --arg flaky "$flaky_count" --arg definitive "$definitive_count" '
        .flaky_tests_found = ($flaky | tonumber) > 0 |
        .definitive_failures = ($definitive | tonumber) > 0 |
        .issue_needed = ($flaky | tonumber) > 0
    ')
    
    # Write updated analysis
    echo "$analysis" > flaky_analysis.json
}

# Run a single test with retry logic
run_test_with_retry() {
    local test_name="$1"
    local max_attempts=$((MAX_RETRIES + 1))
    local attempt=1
    local result="FAIL"
    local log_file=""
    
    log "INFO" "Running test '$test_name' with up to $max_attempts attempts"
    
    while [[ $attempt -le $max_attempts ]]; do
        log "INFO" "Attempt $attempt/$max_attempts for test '$test_name'"
        
        # Create log file for this attempt
        log_file="retry_results/${test_name}_attempt_${attempt}.log"
        mkdir -p "$(dirname "$log_file")"
        
        # Run the test
        if run_single_test "$test_name" "$log_file"; then
            result="PASS"
            log "SUCCESS" "Test '$test_name' passed on attempt $attempt"
            break
        else
            result="FAIL"
            log "WARN" "Test '$test_name' failed on attempt $attempt"
            
            if [[ $attempt -lt $max_attempts ]]; then
                log "INFO" "Waiting before retry..."
                sleep 5
            fi
        fi
        
        attempt=$((attempt + 1))
    done
    
    # Update analysis with final result
    update_analysis "$test_name" "$((attempt - 1))" "$result" "$log_file"
    
    # Return final result
    if [[ "$result" == "PASS" ]]; then
        return 0
    else
        return 1
    fi
}

# Run a single test
run_single_test() {
    local test_name="$1"
    local log_file="$2"
    
    log "INFO" "Executing test: $test_name"
    
    # Determine test type and run accordingly
    if [[ "$test_name" == *"core"* ]]; then
        run_core_test "$test_name" "$log_file"
    elif [[ "$test_name" == *"timer"* ]]; then
        run_timer_test "$test_name" "$log_file"
    elif [[ "$test_name" == *"auth"* ]]; then
        run_auth_test "$test_name" "$log_file"
    elif [[ "$test_name" == *"policy"* ]]; then
        run_policy_test "$test_name" "$log_file"
    elif [[ "$test_name" == *"qemu"* ]]; then
        run_qemu_test "$test_name" "$log_file"
    else
        run_generic_test "$test_name" "$log_file"
    fi
}

# Run core functionality tests
run_core_test() {
    local test_name="$1"
    local log_file="$2"
    
    cd tests
    timeout 300 cargo test --release --test phase2_core -- "$test_name" > "$log_file" 2>&1
    local exit_code=$?
    cd ..
    
    return $exit_code
}

# Run timer-specific tests
run_timer_test() {
    local test_name="$1"
    local log_file="$2"
    
    cd tests
    
    if [[ "$test_name" == *"apic"* ]]; then
        timeout 300 cargo test --release --test apic_timer -- "$test_name" > "$log_file" 2>&1
    elif [[ "$test_name" == *"hpet"* ]]; then
        timeout 300 cargo test --release --test hpet_timer -- "$test_name" > "$log_file" 2>&1
    else
        timeout 300 cargo test --release --test timer_tests -- "$test_name" > "$log_file" 2>&1
    fi
    
    local exit_code=$?
    cd ..
    
    return $exit_code
}

# Run authentication tests
run_auth_test() {
    local test_name="$1"
    local log_file="$2"
    
    cd tests
    timeout 300 cargo test --release --test ipc_auth -- "$test_name" > "$log_file" 2>&1
    local exit_code=$?
    cd ..
    
    return $exit_code
}

# Run capability policy tests
run_policy_test() {
    local test_name="$1"
    local log_file="$2"
    
    cd tests
    timeout 300 cargo test --release --test cap_policy -- "$test_name" > "$log_file" 2>&1
    local exit_code=$?
    cd ..
    
    return $exit_code
}

# Run QEMU integration tests
run_qemu_test() {
    local test_name="$1"
    local log_file="$2"
    
    cd tests
    timeout 600 cargo test --release --test qemu_integration -- "$test_name" > "$log_file" 2>&1
    local exit_code=$?
    cd ..
    
    return $exit_code
}

# Run generic tests
run_generic_test() {
    local test_name="$1"
    local log_file="$2"
    
    cd tests
    timeout 300 cargo test --release -- "$test_name" > "$log_file" 2>&1
    local exit_code=$?
    cd ..
    
    return $exit_code
}

# Generate retry summary
generate_summary() {
    local analysis=$(cat flaky_analysis.json)
    local flaky_count=$(echo "$analysis" | jq '.flaky_tests | length')
    local definitive_count=$(echo "$analysis" | jq '.definitive_failures | length')
    local total_failed=$(echo "$analysis" | jq '.total_failed')
    
    # Create summary markdown
    cat > test_summary.md << EOF
# Flaky Test Analysis Summary

## Configuration
- **Config ID**: $CONFIG_ID
- **Matrix Name**: $MATRIX_NAME
- **Run ID**: $RUN_ID
- **Max Retries**: $MAX_RETRIES
- **Analysis Time**: $(date -u +%Y-%m-%dT%H:%M:%SZ)

## Results Overview
- **Total Failed Tests**: $total_failed
- **Flaky Tests**: $flaky_count
- **Definitive Failures**: $definitive_count

## Flaky Tests (Passed on Retry)
EOF
    
    if [[ $flaky_count -gt 0 ]]; then
        echo "$analysis" | jq -r '.flaky_tests[] | "- **\(.name)** - Failed on attempts \(.failed_attempts | join(", ")), passed on attempt \(.passed_attempt)' >> test_summary.md
    else
        echo "- None" >> test_summary.md
    fi
    
    cat >> test_summary.md << EOF

## Definitive Failures (Failed All Attempts)
EOF
    
    if [[ $definitive_count -gt 0 ]]; then
        echo "$analysis" | jq -r '.definitive_failures[] | "- **\(.name)** - Failed on all \(.total_attempts) attempts' >> test_summary.md
    else
        echo "- None" >> test_summary.md
    fi
    
    cat >> test_summary.md << EOF

## Recommendations
EOF
    
    if [[ $flaky_count -gt 0 ]]; then
        cat >> test_summary.md << EOF
- **Flaky Tests Detected**: Consider investigating environmental factors, timing issues, or race conditions
- **Issue Created**: GitHub issue has been created for tracking flaky test patterns
EOF
    fi
    
    if [[ $definitive_count -gt 0 ]]; then
        cat >> test_summary.md << EOF
- **Definitive Failures**: These tests require immediate attention and code fixes
- **Investigation Needed**: Review test logic, dependencies, and system requirements
EOF
    fi
    
    if [[ $flaky_count -eq 0 && $definitive_count -eq 0 ]]; then
        cat >> test_summary.md << EOF
- **All Tests Passed**: No issues detected
EOF
    fi
    
    # Update analysis with summary
    local summary=$(cat test_summary.md)
    analysis=$(echo "$analysis" | jq --arg summary "$summary" '.retry_summary = $summary')
    echo "$analysis" > flaky_analysis.json
    
    log "INFO" "Test summary generated: test_summary.md"
}

# Main execution
main() {
    log "INFO" "Starting Flaky Test Orchestrator"
    log "INFO" "Configuration: $CONFIG_ID"
    log "INFO" "Matrix: $MATRIX_NAME"
    log "INFO" "Max Retries: $MAX_RETRIES"
    log "INFO" "Results Directory: $RESULTS_DIR"
    log "INFO" "Run ID: $RUN_ID"
    
    # Create results directory
    mkdir -p "$RESULTS_DIR"
    mkdir -p retry_results
    
    # Initialize analysis
    init_analysis
    log "INFO" "Analysis initialized"
    
    # Read failed tests
    if [[ ! -f "$FAILED_TESTS_FILE" ]]; then
        log "ERROR" "Failed tests file not found: $FAILED_TESTS_FILE"
        exit 1
    fi
    
    local failed_tests=$(cat "$FAILED_TESTS_FILE")
    local test_count=$(echo "$failed_tests" | jq '. | length')
    
    log "INFO" "Found $test_count failed tests to retry"
    
    # Process each failed test
    local processed=0
    local flaky_count=0
    local definitive_count=0
    
    echo "$failed_tests" | jq -r '.[]' | while read -r test_name; do
        processed=$((processed + 1))
        log "INFO" "Processing test $processed/$test_count: $test_name"
        
        if run_test_with_retry "$test_name"; then
            flaky_count=$((flaky_count + 1))
        else
            definitive_count=$((definitive_count + 1))
        fi
    done
    
    # Generate final summary
    generate_summary
    
    # Final status
    local analysis=$(cat flaky_analysis.json)
    local final_flaky=$(echo "$analysis" | jq '.flaky_tests | length')
    local final_definitive=$(echo "$analysis" | jq '.definitive_failures | length')
    
    log "INFO" "Flaky test analysis completed"
    log "INFO" "Final results: $final_flaky flaky tests, $final_definitive definitive failures"
    
    if [[ $final_definitive -gt 0 ]]; then
        log "ERROR" "Definitive failures detected - job should fail"
        exit 1
    else
        log "SUCCESS" "All tests eventually passed (some were flaky)"
        exit 0
    fi
}

# Run main function
main "$@"
