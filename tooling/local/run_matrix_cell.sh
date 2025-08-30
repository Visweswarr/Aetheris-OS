#!/bin/bash

# Local Matrix Runner for Polymera OS
# This script runs a chosen matrix cell locally with the same flags as CI
# enabling fast iteration and debugging of matrix configurations.

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
MAX_LOG_SIZE_MB=10
LOCAL_ARTIFACTS_DIR=".local_artifacts"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Logging functions
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

log_step() {
    echo -e "${CYAN}[STEP]${NC} $1"
}

log_matrix() {
    echo -e "${MAGENTA}[MATRIX]${NC} $1"
}

# Help function
show_help() {
    cat << EOF
Usage: $0 [OPTIONS]

Local Matrix Runner for Polymera OS

This script runs a chosen matrix cell locally with the same flags as CI,
enabling fast iteration and debugging of matrix configurations.

ENVIRONMENT VARIABLES:
    TIMER          Timer type: APIC|HPET (default: APIC)
    JITTER         Jitter injection: on|off (default: off)
    AUTH           Authentication: on|off (default: off)
    POLICY         Capability policy: open|closed (default: open)

OPTIONS:
    -h, --help              Show this help message
    -v, --verbose           Enable verbose output
    -d, --debug             Enable debug mode
    -c, --clean             Clean previous artifacts before running
    -k, --keep-artifacts    Keep artifacts after run (default: cleanup)
    -t, --timeout MINUTES   Set timeout for tests (default: 30)
    -l, --log-level LEVEL   Set log level: debug|info|warn|error (default: info)

EXAMPLES:
    # Run with default configuration
    $0

    # Run specific matrix cell
    TIMER=HPET JITTER=on AUTH=on POLICY=closed $0

    # Run with custom timeout and keep artifacts
    TIMER=APIC JITTER=off AUTH=on POLICY=open $0 --timeout 60 --keep-artifacts

    # Clean run with verbose output
    $0 --clean --verbose

    # Debug mode with custom log level
    $0 --debug --log-level debug

MATRIX CONFIGURATIONS:
    The script supports the same matrix configurations as CI:
    - TIMER: APIC (Local APIC timer) or HPET (High Precision Event Timer)
    - JITTER: on (jitter injection enabled) or off (no jitter)
    - AUTH: on (authentication enabled) or off (no authentication)
    - POLICY: open (fail-open capability policy) or closed (fail-closed)

OUTPUT:
    - Test results with clear PASS/FAIL banners
    - Local artifacts stored in .local_artifacts/<config_id>/
    - Nonzero exit code on test failure
    - Concise logs with automatic rotation after ${MAX_LOG_SIZE_MB}MB

EXIT CODES:
    0: All tests passed
    1: Test failures or errors
    2: Configuration errors
    3: Build failures
EOF
}

# Parse command line arguments
VERBOSE=false
DEBUG=false
CLEAN_ARTIFACTS=false
KEEP_ARTIFACTS=false
TIMEOUT_MINUTES=30
LOG_LEVEL="info"

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -d|--debug)
            DEBUG=true
            VERBOSE=true
            shift
            ;;
        -c|--clean)
            CLEAN_ARTIFACTS=true
            shift
            ;;
        -k|--keep-artifacts)
            KEEP_ARTIFACTS=true
            shift
            ;;
        -t|--timeout)
            TIMEOUT_MINUTES="$2"
            shift 2
            ;;
        -l|--log-level)
            LOG_LEVEL="$2"
            shift 2
            ;;
        -*)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
        *)
            break
            ;;
    esac
    shift
done

# Set debug mode
if [[ "$DEBUG" == true ]]; then
    set -x
fi

# Function to validate matrix configuration
validate_matrix_config() {
    local timer="${TIMER:-APIC}"
    local jitter="${JITTER:-off}"
    local auth="${AUTH:-off}"
    local policy="${POLICY:-open}"
    
    # Validate timer
    if [[ "$timer" != "APIC" && "$timer" != "HPET" ]]; then
        log_error "Invalid TIMER value: $timer. Must be APIC or HPET"
        exit 2
    fi
    
    # Validate jitter
    if [[ "$jitter" != "on" && "$jitter" != "off" ]]; then
        log_error "Invalid JITTER value: $jitter. Must be on or off"
        exit 2
    fi
    
    # Validate auth
    if [[ "$auth" != "on" && "$auth" != "off" ]]; then
        log_error "Invalid AUTH value: $auth. Must be on or off"
        exit 2
    fi
    
    # Validate policy
    if [[ "$policy" != "open" && "$policy" != "closed" ]]; then
        log_error "Invalid POLICY value: $policy. Must be open or closed"
        exit 2
    fi
    
    # Set validated values
    TIMER="$timer"
    JITTER="$jitter"
    AUTH="$auth"
    POLICY="$policy"
    
    log_matrix "Configuration validated: TIMER=$TIMER, JITTER=$JITTER, AUTH=$AUTH, POLICY=$POLICY"
}

# Function to generate configuration ID
generate_config_id() {
    local timer="${TIMER:-APIC}"
    local jitter="${JITTER:-off}"
    local auth="${AUTH:-off}"
    local policy="${POLICY:-open}"
    
    # Convert to lowercase for consistency with CI
    local timer_lower=$(echo "$timer" | tr '[:upper:]' '[:lower:]')
    local jitter_lower=$(echo "$jitter" | tr '[:upper:]' '[:lower:]')
    local auth_lower=$(echo "$auth" | tr '[:upper:]' '[:lower:]')
    local policy_lower=$(echo "$policy" | tr '[:upper:]' '[:lower:]')
    
    echo "${timer_lower}_${jitter_lower}_${auth_lower}_${policy_lower}"
}

# Function to generate configuration name
generate_config_name() {
    local timer="${TIMER:-APIC}"
    local jitter="${JITTER:-off}"
    local auth="${AUTH:-off}"
    local policy="${POLICY:-open}"
    
    echo "$timer Timer, Jitter $jitter, Auth $auth, Policy $policy"
}

# Function to setup log rotation
setup_log_rotation() {
    local log_file="$1"
    local max_size_bytes=$((MAX_LOG_SIZE_MB * 1024 * 1024))
    
    # Check if log file exists and is too large
    if [[ -f "$log_file" ]]; then
        local current_size=$(stat -c%s "$log_file" 2>/dev/null || stat -f%z "$log_file" 2>/dev/null || echo "0")
        
        if [[ $current_size -gt $max_size_bytes ]]; then
            log_info "Rotating log file: $log_file (${current_size} bytes > ${max_size_bytes} bytes)"
            
            # Create backup with timestamp
            local backup_file="${log_file}.$(date +%Y%m%d_%H%M%S)"
            mv "$log_file" "$backup_file"
            
            # Compress backup if available
            if command -v gzip &> /dev/null; then
                gzip "$backup_file"
                log_info "Log rotated and compressed: ${backup_file}.gz"
            else
                log_info "Log rotated: $backup_file"
            fi
        fi
    fi
}

# Function to log with rotation
log_with_rotation() {
    local log_file="$1"
    local message="$2"
    
    # Setup log rotation
    setup_log_rotation "$log_file"
    
    # Log message with timestamp
    echo "$(date -u +"%Y-%m-%dT%H:%M:%SZ") - $message" >> "$log_file"
    
    # Also output to console if verbose
    if [[ "$VERBOSE" == true ]]; then
        echo "$message"
    fi
}

# Function to setup local artifacts directory
setup_artifacts_directory() {
    local config_id="$1"
    local artifacts_dir="$LOCAL_ARTIFACTS_DIR/$config_id"
    
    # Clean previous artifacts if requested
    if [[ "$CLEAN_ARTIFACTS" == true ]]; then
        if [[ -d "$artifacts_dir" ]]; then
            log_info "Cleaning previous artifacts: $artifacts_dir"
            rm -rf "$artifacts_dir"
        fi
    fi
    
    # Create artifacts directory structure
    mkdir -p "$artifacts_dir/logs"
    mkdir -p "$artifacts_dir/tests"
    mkdir -p "$artifacts_dir/kernel"
    mkdir -p "$artifacts_dir/perf"
    
    log_info "Artifacts directory setup: $artifacts_dir"
    echo "$artifacts_dir"
}

# Function to configure kernel
configure_kernel() {
    local config_id="$1"
    local artifacts_dir="$2"
    
    log_step "Configuring kernel for matrix cell: $config_id"
    
    # Create kernel configuration
    local kernel_config="$artifacts_dir/kernel/.config.matrix"
    mkdir -p "$(dirname "$kernel_config")"
    
    # Generate kernel configuration based on matrix values
    cat > "$kernel_config" << EOF
# Kernel configuration for matrix cell: $config_id
# Generated at: $(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Timer configuration
CONFIG_APIC_TIMER=$(if [[ "$TIMER" == "APIC" ]]; then echo "y"; else echo "n"; fi)
CONFIG_HPET_TIMER=$(if [[ "$TIMER" == "HPET" ]]; then echo "y"; else echo "n"; fi)

# Jitter injection
CONFIG_JITTER_INJECTION=$(if [[ "$JITTER" == "on" ]]; then echo "y"; else echo "n"; fi)

# Authentication
CONFIG_AUTH_ENABLED=$(if [[ "$AUTH" == "on" ]]; then echo "y"; else echo "n"; fi)

# Capability policy
CONFIG_CAP_POLICY_OPEN=$(if [[ "$POLICY" == "open" ]]; then echo "y"; else echo "n"; fi)
CONFIG_CAP_POLICY_CLOSED=$(if [[ "$POLICY" == "closed" ]]; then echo "y"; else echo "n"; fi)

# Development features
CONFIG_DEV_MODE=y
CONFIG_DEBUG=y
CONFIG_KERNEL_DEBUG=y

# Matrix cell identifier
CONFIG_MATRIX_CELL="$config_id"
CONFIG_MATRIX_TIMER="$TIMER"
CONFIG_MATRIX_JITTER="$JITTER"
CONFIG_MATRIX_AUTH="$AUTH"
CONFIG_MATRIX_POLICY="$POLICY"
EOF
    
    log_success "Kernel configuration created: $kernel_config"
    
    # Copy to kernel directory for build
    cp "$kernel_config" "$PROJECT_ROOT/kernel/.config.matrix"
    
    # Set environment variables for build
    export CONFIG_APIC_TIMER=$(if [[ "$TIMER" == "APIC" ]]; then echo "y"; else echo "n"; fi)
    export CONFIG_HPET_TIMER=$(if [[ "$TIMER" == "HPET" ]]; then echo "y"; else echo "n"; fi)
    export CONFIG_JITTER_INJECTION=$(if [[ "$JITTER" == "on" ]]; then echo "y"; else echo "n"; fi)
    export CONFIG_AUTH_ENABLED=$(if [[ "$AUTH" == "on" ]]; then echo "y"; else echo "n"; fi)
    export CONFIG_CAP_POLICY_OPEN=$(if [[ "$POLICY" == "open" ]]; then echo "y"; else echo "n"; fi)
    export CONFIG_CAP_POLICY_CLOSED=$(if [[ "$POLICY" == "closed" ]]; then echo "y"; else echo "n"; fi)
    export CONFIG_MATRIX_CELL="$config_id"
}

# Function to build kernel
build_kernel() {
    local config_id="$1"
    local artifacts_dir="$2"
    local log_file="$artifacts_dir/logs/build.log"
    
    log_step "Building kernel for matrix cell: $config_id"
    
    # Log build start
    log_with_rotation "$log_file" "Starting kernel build for matrix cell: $config_id"
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    # Build kernel with matrix configuration
    if make kernel MATRIX_CONFIG=".config.matrix" 2>&1 | tee -a "$log_file"; then
        log_success "Kernel build completed successfully"
        log_with_rotation "$log_file" "Kernel build completed successfully"
        return 0
    else
        log_error "Kernel build failed"
        log_with_rotation "$log_file" "Kernel build failed"
        return 1
    fi
}

# Function to run tests
run_tests() {
    local config_id="$1"
    local artifacts_dir="$2"
    local log_file="$artifacts_dir/logs/tests.log"
    
    log_step "Running tests for matrix cell: $config_id"
    
    # Log test start
    log_with_rotation "$log_file" "Starting tests for matrix cell: $config_id"
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    # Run core tests
    log_info "Running core tests..."
    if make test-core MATRIX_CONFIG=".config.matrix" 2>&1 | tee -a "$log_file"; then
        log_success "Core tests completed successfully"
        log_with_rotation "$log_file" "Core tests completed successfully"
    else
        log_error "Core tests failed"
        log_with_rotation "$log_file" "Core tests failed"
        return 1
    fi
    
    # Run QEMU tests if available
    if command -v qemu-system-x86_64 &> /dev/null; then
        log_info "Running QEMU tests..."
        if make test-qemu MATRIX_CONFIG=".config.matrix" 2>&1 | tee -a "$log_file"; then
            log_success "QEMU tests completed successfully"
            log_with_rotation "$log_file" "QEMU tests completed successfully"
        else
            log_error "QEMU tests failed"
            log_with_rotation "$log_file" "QEMU tests failed"
            return 1
        fi
    else
        log_warning "QEMU not available, skipping QEMU tests"
        log_with_rotation "$log_file" "QEMU not available, skipping QEMU tests"
    fi
    
    return 0
}

# Function to run performance benchmarks
run_performance() {
    local config_id="$1"
    local artifacts_dir="$2"
    local log_file="$artifacts_dir/logs/performance.log"
    
    log_step "Running performance benchmarks for matrix cell: $config_id"
    
    # Log performance start
    log_with_rotation "$log_file" "Starting performance benchmarks for matrix cell: $config_id"
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    # Run performance benchmarks
    if make perf MATRIX_CONFIG=".config.matrix" 2>&1 | tee -a "$log_file"; then
        log_success "Performance benchmarks completed successfully"
        log_with_rotation "$log_file" "Performance benchmarks completed successfully"
        
        # Copy performance results to artifacts
        if [[ -d "perf/results" ]]; then
            cp -r perf/results/* "$artifacts_dir/perf/"
            log_info "Performance results copied to artifacts"
        fi
        
        return 0
    else
        log_error "Performance benchmarks failed"
        log_with_rotation "$log_file" "Performance benchmarks failed"
        return 1
    fi
}

# Function to collect artifacts
collect_artifacts() {
    local config_id="$1"
    local artifacts_dir="$2"
    
    log_step "Collecting artifacts for matrix cell: $config_id"
    
    # Copy test results
    if [[ -d "tests/test_results" ]]; then
        cp -r tests/test_results/* "$artifacts_dir/tests/"
        log_info "Test results copied to artifacts"
    fi
    
    # Copy kernel logs
    if [[ -f "kernel/kernel.log" ]]; then
        cp kernel/kernel.log "$artifacts_dir/kernel/"
        log_info "Kernel logs copied to artifacts"
    fi
    
    # Copy kernel configuration
    if [[ -f "kernel/.config.matrix" ]]; then
        cp kernel/.config.matrix "$artifacts_dir/kernel/"
        log_info "Kernel configuration copied to artifacts"
    fi
    
    # Create summary file
    local summary_file="$artifacts_dir/run_summary.md"
    cat > "$summary_file" << EOF
# Matrix Cell Run Summary

**Configuration ID**: \`$config_id\`
**Configuration Name**: $(generate_config_name)
**Run Timestamp**: $(date -u +"%Y-%m-%dT%H:%M:%SZ")
**Exit Code**: $EXIT_CODE

## Matrix Configuration
- **Timer**: $TIMER
- **Jitter**: $JITTER
- **Auth**: $AUTH
- **Policy**: $POLICY

## Results
- **Build**: $(if [[ $BUILD_EXIT_CODE -eq 0 ]]; then echo "✅ PASS"; else echo "❌ FAIL"; fi)
- **Tests**: $(if [[ $TESTS_EXIT_CODE -eq 0 ]]; then echo "✅ PASS"; else echo "❌ FAIL"; fi)
- **Performance**: $(if [[ $PERF_EXIT_CODE -eq 0 ]]; then echo "✅ PASS"; else echo "❌ FAIL"; fi)

## Artifacts
- **Logs**: logs/
- **Tests**: tests/
- **Kernel**: kernel/
- **Performance**: perf/

## Next Steps
$(if [[ $EXIT_CODE -eq 0 ]]; then echo "- All tests passed successfully"; else echo "- Review logs for failures"; echo "- Fix issues and re-run"; fi)
EOF
    
    log_success "Artifacts collected: $artifacts_dir"
}

# Function to display results
display_results() {
    local config_id="$1"
    local artifacts_dir="$2"
    
    echo
    echo "╔══════════════════════════════════════════════════════════════════════════════╗"
    echo "║                        MATRIX CELL RUN COMPLETED                            ║"
    echo "╚══════════════════════════════════════════════════════════════════════════════╝"
    echo
    
    if [[ $EXIT_CODE -eq 0 ]]; then
        echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
        echo -e "${GREEN}║                                ✅ ALL TESTS PASSED ✅                          ║${NC}"
        echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
    else
        echo -e "${RED}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
        echo -e "${RED}║                                ❌ SOME TESTS FAILED ❌                          ║${NC}"
        echo -e "${RED}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
    fi
    
    echo
    echo "📊 Configuration: $(generate_config_name)"
    echo "🆔 Config ID: $config_id"
    echo "📁 Artifacts: $artifacts_dir"
    echo "⏱️  Build: $(if [[ $BUILD_EXIT_CODE -eq 0 ]]; then echo "✅ PASS"; else echo "❌ FAIL"; fi)"
    echo "🧪 Tests: $(if [[ $TESTS_EXIT_CODE -eq 0 ]]; then echo "✅ PASS"; else echo "❌ FAIL"; fi)"
    echo "📈 Performance: $(if [[ $PERF_EXIT_CODE -eq 0 ]]; then echo "✅ PASS"; else echo "❌ FAIL"; fi)"
    echo
    
    if [[ $EXIT_CODE -eq 0 ]]; then
        echo -e "${GREEN}🎉 Matrix cell completed successfully!${NC}"
    else
        echo -e "${RED}💥 Matrix cell completed with failures. Check logs for details.${NC}"
    fi
    
    echo
}

# Function to cleanup
cleanup() {
    if [[ "$KEEP_ARTIFACTS" != true ]]; then
        log_info "Cleaning up temporary files..."
        # Add cleanup logic here if needed
    else
        log_info "Keeping artifacts as requested"
    fi
}

# Main execution
main() {
    log_info "Starting local matrix runner..."
    
    # Check if we're in the right directory
    if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        log_error "Not in Polymera OS project root. Expected Cargo.toml at: $PROJECT_ROOT"
        exit 1
    fi
    
    # Validate matrix configuration
    validate_matrix_config
    
    # Generate configuration ID and name
    CONFIG_ID=$(generate_config_id)
    CONFIG_NAME=$(generate_config_name)
    
    log_matrix "Running matrix cell: $CONFIG_NAME (ID: $CONFIG_ID)"
    
    # Setup artifacts directory
    ARTIFACTS_DIR=$(setup_artifacts_directory "$CONFIG_ID")
    
    # Initialize exit codes
    BUILD_EXIT_CODE=0
    TESTS_EXIT_CODE=0
    PERF_EXIT_CODE=0
    EXIT_CODE=0
    
    # Run matrix cell
    if configure_kernel "$CONFIG_ID" "$ARTIFACTS_DIR"; then
        if build_kernel "$CONFIG_ID" "$ARTIFACTS_DIR"; then
            if run_tests "$CONFIG_ID" "$ARTIFACTS_DIR"; then
                if run_performance "$CONFIG_ID" "$ARTIFACTS_DIR"; then
                    log_success "All matrix cell components completed successfully"
                else
                    log_error "Performance benchmarks failed"
                    PERF_EXIT_CODE=1
                    EXIT_CODE=1
                fi
            else
                log_error "Tests failed"
                TESTS_EXIT_CODE=1
                EXIT_CODE=1
            fi
        else
            log_error "Kernel build failed"
            BUILD_EXIT_CODE=1
            EXIT_CODE=1
        fi
    else
        log_error "Kernel configuration failed"
        EXIT_CODE=1
    fi
    
    # Collect artifacts
    collect_artifacts "$CONFIG_ID" "$ARTIFACTS_DIR"
    
    # Display results
    display_results "$CONFIG_ID" "$ARTIFACTS_DIR"
    
    # Cleanup
    cleanup
    
    # Exit with appropriate code
    exit $EXIT_CODE
}

# Set trap for cleanup
trap cleanup EXIT

# Run main function
main "$@"
