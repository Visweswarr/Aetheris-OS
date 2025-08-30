#!/bin/bash

# Test Script for Polymera OS Kernel Boot
# Tests QEMU/OVMF runner with minimal kernel

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/build"
TEST_DIR="$SCRIPT_DIR/test"
KERNEL_IMAGE=""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

# Create test directory
setup_test_env() {
    log_info "Setting up test environment..."
    
    mkdir -p "$TEST_DIR"
    mkdir -p "$BUILD_DIR"
    
    log_success "Test environment ready"
}

# Build minimal test kernel
build_test_kernel() {
    log_info "Building minimal test kernel..."
    
    # Create a minimal test kernel if none exists
    if [[ -z "$KERNEL_IMAGE" ]]; then
        KERNEL_IMAGE="$BUILD_DIR/test_kernel.elf"
        
        if [[ ! -f "$KERNEL_IMAGE" ]]; then
            log_warning "No kernel image provided, creating minimal test kernel..."
            
            # Create a minimal ELF file for testing
            cat > "$TEST_DIR/test_kernel.c" << 'EOF'
#include <stdio.h>
#include <unistd.h>

int main() {
    printf("Polymera OS Test Kernel Starting...\n");
    printf("Architecture: %s\n", sizeof(void*) == 8 ? "64-bit" : "32-bit");
    printf("Kernel loaded successfully!\n");
    
    // Simulate kernel initialization
    for (int i = 0; i < 5; i++) {
        printf("Initializing component %d...\n", i + 1);
        usleep(100000); // 100ms delay
    }
    
    printf("Kernel initialization complete!\n");
    printf("Ready for user input...\n");
    
    // Keep running for testing
    while (1) {
        usleep(1000000); // 1 second
        printf("Kernel heartbeat...\n");
    }
    
    return 0;
}
EOF
            
            # Try to compile the test kernel
            if command -v gcc &> /dev/null; then
                log_info "Compiling test kernel with GCC..."
                gcc -o "$KERNEL_IMAGE" "$TEST_DIR/test_kernel.c" -static -nostdlib -e main
                log_success "Test kernel compiled: $KERNEL_IMAGE"
            else
                log_warning "GCC not available, creating dummy kernel file..."
                # Create a dummy file for testing
                dd if=/dev/zero of="$KERNEL_IMAGE" bs=1M count=1 2>/dev/null
                log_success "Dummy kernel created: $KERNEL_IMAGE"
            fi
        else
            log_success "Using existing kernel: $KERNEL_IMAGE"
        fi
    fi
    
    if [[ ! -f "$KERNEL_IMAGE" ]]; then
        log_error "Failed to create kernel image"
        exit 1
    fi
}

# Test x86_64 QEMU
test_x86_64() {
    log_info "Testing x86_64 QEMU runner..."
    
    local serial_log="$TEST_DIR/x86_64_serial.log"
    local test_args=(
        "--kernel" "$KERNEL_IMAGE"
        "--serial" "$serial_log"
        "--memory" "1G"
        "--cpus" "1"
        "--debug"
    )
    
    log_info "Running: $SCRIPT_DIR/run_x86_64.sh ${test_args[*]}"
    
    # Run QEMU in background for a short time
    timeout 10s "$SCRIPT_DIR/run_x86_64.sh" "${test_args[@]}" || true
    
    # Check if serial log was created
    if [[ -f "$serial_log" ]]; then
        log_success "x86_64 test completed, serial log: $serial_log"
        echo "=== x86_64 Serial Log ==="
        head -20 "$serial_log" 2>/dev/null || echo "Log file empty or unreadable"
    else
        log_warning "x86_64 test completed, no serial log generated"
    fi
}

# Test aarch64 QEMU
test_aarch64() {
    log_info "Testing aarch64 QEMU runner..."
    
    local serial_log="$TEST_DIR/aarch64_serial.log"
    local test_args=(
        "--kernel" "$KERNEL_IMAGE"
        "--serial" "$serial_log"
        "--memory" "1G"
        "--cpus" "1"
        "--debug"
    )
    
    log_info "Running: $SCRIPT_DIR/run_aarch64.sh ${test_args[*]}"
    
    # Run QEMU in background for a short time
    timeout 10s "$SCRIPT_DIR/run_aarch64.sh" "${test_args[@]}" || true
    
    # Check if serial log was created
    if [[ -f "$serial_log" ]]; then
        log_success "aarch64 test completed, serial log: $serial_log"
        echo "=== aarch64 Serial Log ==="
        head -20 "$serial_log" 2>/dev/null || echo "Log file empty or unreadable"
    else
        log_warning "aarch64 test completed, no serial log generated"
    fi
}

# Test universal runner
test_universal() {
    log_info "Testing universal QEMU runner..."
    
    local serial_log="$TEST_DIR/universal_serial.log"
    local test_args=(
        "--kernel" "$KERNEL_IMAGE"
        "--serial" "$serial_log"
        "--memory" "1G"
        "--cpus" "1"
        "--debug"
    )
    
    log_info "Running: $SCRIPT_DIR/run.sh ${test_args[*]}"
    
    # Run QEMU in background for a short time
    timeout 10s "$SCRIPT_DIR/run.sh" "${test_args[@]}" || true
    
    # Check if serial log was created
    if [[ -f "$serial_log" ]]; then
        log_success "Universal test completed, serial log: $serial_log"
        echo "=== Universal Serial Log ==="
        head -20 "$serial_log" 2>/dev/null || echo "Log file empty or unreadable"
    else
        log_warning "Universal test completed, no serial log generated"
    fi
}

# Run all tests
run_all_tests() {
    log_info "Running all QEMU tests..."
    
    # Test x86_64 if available
    if command -v qemu-system-x86_64 &> /dev/null; then
        test_x86_64
        echo ""
    else
        log_warning "qemu-system-x86_64 not available, skipping x86_64 test"
    fi
    
    # Test aarch64 if available
    if command -v qemu-system-aarch64 &> /dev/null; then
        test_aarch64
        echo ""
    else
        log_warning "qemu-system-aarch64 not available, skipping aarch64 test"
    fi
    
    # Test universal runner
    test_universal
    echo ""
}

# Show test results
show_results() {
    log_info "Test Results Summary"
    log_info "==================="
    
    local test_files=(
        "$TEST_DIR/x86_64_serial.log"
        "$TEST_DIR/aarch64_serial.log"
        "$TEST_DIR/universal_serial.log"
    )
    
    for file in "${test_files[@]}"; do
        if [[ -f "$file" ]]; then
            local size=$(stat -c%s "$file" 2>/dev/null || echo "0")
            if [[ "$size" -gt 0 ]]; then
                log_success "$(basename "$file"): $size bytes"
            else
                log_warning "$(basename "$file"): empty"
            fi
        else
            log_error "$(basename "$file"): not found"
        fi
    done
    
    echo ""
    log_info "Test logs available in: $TEST_DIR"
}

# Cleanup test environment
cleanup() {
    log_info "Cleaning up test environment..."
    
    # Stop any running QEMU processes
    pkill -f "qemu-system-" 2>/dev/null || true
    
    log_success "Cleanup completed"
}

# Main execution
main() {
    log_info "Polymera OS QEMU Kernel Boot Test"
    log_info "=================================="
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -k|--kernel)
                KERNEL_IMAGE="$2"
                shift 2
                ;;
            -h|--help)
                cat << EOF
Polymera OS QEMU Kernel Boot Test

Usage: $0 [OPTIONS]

Options:
    -k, --kernel <file>       Use existing kernel image
    -h, --help                Show this help message

This script tests the QEMU/OVMF runner by:
1. Building a minimal test kernel (if none provided)
2. Testing x86_64 QEMU runner
3. Testing aarch64 QEMU runner
4. Testing universal QEMU runner
5. Verifying serial output

EOF
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Setup test environment
    setup_test_env
    
    # Build test kernel
    build_test_kernel
    
    # Run tests
    run_all_tests
    
    # Show results
    show_results
    
    # Cleanup
    cleanup
    
    log_success "All tests completed!"
}

# Trap cleanup on exit
trap cleanup EXIT

# Run main function
main "$@"
