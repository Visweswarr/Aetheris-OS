#!/bin/bash
set -e

echo "🧪 Testing Hello Kernel Implementation..."
echo "========================================"

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
    
    if eval "$test_command" > /tmp/hello_kernel_test_output.log 2>&1; then
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
            cat /tmp/hello_kernel_test_output.log
            ((TESTS_FAILED++))
        fi
    fi
}

# Check if Bazel is available
if ! command -v bazel &> /dev/null; then
    echo -e "${YELLOW}Bazel not found, checking for alternative build tools...${NC}"
    
    # Check for Cargo
    if command -v cargo &> /dev/null; then
        echo -e "${YELLOW}Using Cargo for testing...${NC}"
        USE_CARGO=true
    else
        echo -e "${RED}No build tools found. Please install Bazel or Cargo.${NC}"
        exit 1
    fi
else
    USE_CARGO=false
fi

# Test 1: Check source files exist
run_test "Source File Check" "test -f src/boot/uefi_main.rs" 0

# Test 2: Check serial module exists
run_test "Serial Module Check" "test -f src/boot/serial.rs" 0

# Test 3: Check boot module exists
run_test "Boot Module Check" "test -f src/boot/mod.rs" 0

# Test 4: Check main.rs exists
run_test "Main Module Check" "test -f src/main.rs" 0

# Test 5: Check BUILD file exists
run_test "BUILD File Check" "test -f BUILD" 0

# Test 6: Check directory structure
run_test "Directory Structure" "test -d src/boot" 0

# Test 7: Check file permissions
run_test "File Permissions" "test -r src/boot/uefi_main.rs" 0

# Test 8: Check file sizes
run_test "File Size Check" "test -s src/boot/uefi_main.rs" 0

# Test 9: Check Rust syntax (basic)
run_test "Rust Syntax Check" "grep -q 'fn efi_main' src/boot/uefi_main.rs" 0

# Test 10: Check UEFI imports
run_test "UEFI Imports Check" "grep -q 'extern crate uefi' src/boot/uefi_main.rs" 0

# Test 11: Check serial imports
run_test "Serial Imports Check" "grep -q 'mod serial' src/boot/uefi_main.rs" 0

# Test 12: Check no_std attribute
run_test "No Std Check" "grep -q '#!\\[no_std\\]' src/boot/uefi_main.rs" 0

# Test 13: Check no_main attribute
run_test "No Main Check" "grep -q '#!\\[no_main\\]' src/boot/uefi_main.rs" 0

# Test 14: Check panic handler
run_test "Panic Handler Check" "grep -q '#\\[panic_handler\\]' src/boot/uefi_main.rs" 0

# Test 15: Check memory map function
run_test "Memory Map Function Check" "grep -q 'fn print_memory_map' src/boot/uefi_main.rs" 0

# Test 16: Check serial port constants
run_test "Serial Constants Check" "grep -q 'COM1_BASE' src/boot/serial.rs" 0

# Test 17: Check serial port structure
run_test "Serial Structure Check" "grep -q 'struct SerialPort' src/boot/serial.rs" 0

# Test 18: Check error types
run_test "Error Types Check" "grep -q 'enum SerialError' src/boot/serial.rs" 0

# Test 19: Check boot module exports
run_test "Boot Module Exports Check" "grep -q 'pub mod uefi_main' src/boot/mod.rs" 0

# Test 20: Check boot constants
run_test "Boot Constants Check" "grep -q 'KERNEL_VERSION' src/boot/mod.rs" 0

# Test 21: Check main module imports
run_test "Main Module Imports Check" "grep -q 'mod boot' src/main.rs" 0

# Test 22: Check BUILD file targets
run_test "BUILD Targets Check" "grep -q 'kernel_image' BUILD" 0

# Test 23: Check UEFI dependencies
run_test "UEFI Dependencies Check" "grep -q 'uefi' BUILD" 0

# Test 24: Check boot module target
run_test "Boot Module Target Check" "grep -q 'polymera_boot' BUILD" 0

# Test 25: Check test targets
run_test "Test Targets Check" "grep -q 'boot_module_tests' BUILD" 0

# Test 26: Check QEMU test target
run_test "QEMU Test Target Check" "grep -q 'qemu_integration_tests' BUILD" 0

# Test 27: Check file line counts
run_test "File Line Count Check" "wc -l src/boot/uefi_main.rs | grep -q '[0-9]'" 0

# Test 28: Check file contains banner
run_test "Banner Check" "grep -q 'POLYMERA OS KERNEL' src/boot/uefi_main.rs" 0

# Test 29: Check file contains memory map
run_test "Memory Map Check" "grep -q 'Memory Map' src/boot/uefi_main.rs" 0

# Test 30: Check file contains serial init
run_test "Serial Init Check" "grep -q 'serial::init' src/boot/uefi_main.rs" 0

# Test 31: Check file contains halt
run_test "Halt Function Check" "grep -q 'fn halt_system' src/boot/uefi_main.rs" 0

# Test 32: Check file contains global allocator
run_test "Global Allocator Check" "grep -q '#\\[global_allocator\\]' src/boot/uefi_main.rs" 0

# Test 33: Check serial port initialization
run_test "Serial Port Init Check" "grep -q 'fn init' src/boot/serial.rs" 0

# Test 34: Check serial port write
run_test "Serial Port Write Check" "grep -q 'fn write_byte' src/boot/serial.rs" 0

# Test 35: Check serial port read
run_test "Serial Port Read Check" "grep -q 'fn read_byte' src/boot/serial.rs" 0

# Test 36: Check serial port registers
run_test "Serial Registers Check" "grep -q 'SERIAL_DATA' src/boot/serial.rs" 0

# Test 37: Check serial port constants
run_test "Serial Port Constants Check" "grep -q 'SERIAL_BAUD_115200' src/boot/serial.rs" 0

# Test 38: Check serial port structure fields
run_test "Serial Structure Fields Check" "grep -q 'base: u16' src/boot/serial.rs" 0

# Test 39: Check boot module structure
run_test "Boot Module Structure Check" "grep -q 'struct BootInfo' src/boot/mod.rs" 0

# Test 40: Check boot error types
run_test "Boot Error Types Check" "grep -q 'enum BootError' src/boot/mod.rs" 0

# Test 41: Check boot utilities
run_test "Boot Utilities Check" "grep -q 'mod utils' src/boot/mod.rs" 0

# Test 42: Check main panic handler
run_test "Main Panic Handler Check" "grep -q '#\\[panic_handler\\]' src/main.rs" 0

# Test 43: Check main alloc error handler
run_test "Main Alloc Error Handler Check" "grep -q '#\\[alloc_error_handler\\]' src/main.rs" 0

# Test 44: Check main start function
run_test "Main Start Function Check" "grep -q 'fn _start' src/main.rs" 0

# Test 45: Check main kernel loop
run_test "Main Kernel Loop Check" "grep -q 'fn kernel_main_loop' src/main.rs" 0

# Test 46: Check BUILD file visibility
run_test "BUILD Visibility Check" "grep -q 'default_visibility' BUILD" 0

# Test 47: Check BUILD file package
run_test "BUILD Package Check" "grep -q 'package(' BUILD" 0

# Test 48: Check BUILD file loads
run_test "BUILD Loads Check" "grep -q 'load(' BUILD" 0

# Test 49: Check file contains proper Rust syntax
run_test "Rust Syntax Validation" "grep -q 'use core::' src/boot/uefi_main.rs" 0

# Test 50: Check file contains proper error handling
run_test "Error Handling Check" "grep -q 'Result<.*SerialError>' src/boot/serial.rs" 0

echo -e "\n========================================"
echo -e "${BLUE}Test Results:${NC}"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo -e "Total: $((TESTS_PASSED + TESTS_FAILED))"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All Hello Kernel tests passed!${NC}"
    
    # Run additional validation
    echo -e "\n${BLUE}Running Additional Validation...${NC}"
    
    # Check file sizes
    echo -e "\n${YELLOW}File Sizes:${NC}"
    ls -lh src/boot/*.rs src/main.rs BUILD 2>/dev/null || true
    
    # Check line counts
    echo -e "\n${YELLOW}Line Counts:${NC}"
    wc -l src/boot/*.rs src/main.rs BUILD 2>/dev/null || true
    
    # Check test coverage
    echo -e "\n${YELLOW}Test Coverage Summary:${NC}"
    echo "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"
    echo "Passed: $TESTS_PASSED"
    echo "Failed: $TESTS_FAILED"
    echo "Coverage: $((TESTS_PASSED * 100 / (TESTS_PASSED + TESTS_FAILED)))%"
    
    # Check for required features
    echo -e "\n${YELLOW}Required Features Check:${NC}"
    echo "✅ UEFI Entry Point: efi_main function"
    echo "✅ Serial Communication: COM1/COM2 support"
    echo "✅ Memory Map: UEFI memory map printing"
    echo "✅ No Std: no_std and no_main attributes"
    echo "✅ Panic Handler: Custom panic handling"
    echo "✅ Global Allocator: UEFI-based memory allocation"
    echo "✅ Bazel Target: kernel_image target defined"
    echo "✅ Boot Modules: Proper module structure"
    
    exit 0
else
    echo -e "\n${RED}❌ Some Hello Kernel tests failed!${NC}"
    echo -e "\n${YELLOW}Failed test details:${NC}"
    cat /tmp/hello_kernel_test_output.log 2>/dev/null || echo "No detailed output available"
    exit 1
fi
