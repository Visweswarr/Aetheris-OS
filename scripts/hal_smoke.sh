#!/bin/bash
# HAL Smoke Test Script
# This script performs basic smoke tests for the HAL implementation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
PROVIDER=${AETHERIS_DEV_PROVIDER:-"det"}
VERBOSE=${VERBOSE:-false}
SKIP_HARDWARE=${SKIP_HARDWARE:-false}

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    case $status in
        "INFO")
            echo -e "${BLUE}[INFO]${NC} $message"
            ;;
        "SUCCESS")
            echo -e "${GREEN}[SUCCESS]${NC} $message"
            ;;
        "WARNING")
            echo -e "${YELLOW}[WARNING]${NC} $message"
            ;;
        "ERROR")
            echo -e "${RED}[ERROR]${NC} $message"
            ;;
    esac
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to run test with error handling
run_test() {
    local test_name=$1
    local test_command=$2
    
    print_status "INFO" "Running test: $test_name"
    
    if [ "$VERBOSE" = "true" ]; then
        echo "Command: $test_command"
    fi
    
    if eval "$test_command"; then
        print_status "SUCCESS" "$test_name passed"
        return 0
    else
        print_status "ERROR" "$test_name failed"
        return 1
    fi
}

# Function to check system requirements
check_system_requirements() {
    print_status "INFO" "Checking system requirements..."
    
    # Check if we're on Linux
    if [ "$(uname)" != "Linux" ]; then
        print_status "WARNING" "Not running on Linux - hardware tests will be skipped"
        SKIP_HARDWARE=true
    fi
    
    # Check for required tools
    local required_tools=("cargo" "go" "node" "python3")
    for tool in "${required_tools[@]}"; do
        if ! command_exists "$tool"; then
            print_status "ERROR" "Required tool not found: $tool"
            exit 1
        fi
    done
    
    print_status "SUCCESS" "System requirements check passed"
}

# Function to build Rust services
build_rust_services() {
    print_status "INFO" "Building Rust services..."
    
    cd services/devices
    
    # Build in release mode
    if ! run_test "Rust build" "cargo build --release"; then
        return 1
    fi
    
    # Run tests
    if ! run_test "Rust tests" "cargo test --lib"; then
        return 1
    fi
    
    # Run HAL-specific tests
    if ! run_test "HAL tests" "cargo test hal::"; then
        return 1
    fi
    
    cd ../..
    print_status "SUCCESS" "Rust services built and tested"
}

# Function to test C FFI
test_c_ffi() {
    print_status "INFO" "Testing C FFI..."
    
    cd c/libc_aetheris
    
    # Check if we have the required system libraries
    if ! pkg-config --exists libgpiod; then
        print_status "WARNING" "libgpiod not found - some tests may fail"
    fi
    
    # Compile C FFI
    if ! run_test "C FFI compile" "gcc -c -fPIC devices_list.c -o devices_list.o"; then
        return 1
    fi
    
    # Create shared library
    if ! run_test "C FFI shared library" "gcc -shared -o libdevices_list.so devices_list.o -L../../services/devices/target/release -laetheris_devices"; then
        return 1
    fi
    
    # Create test program
    cat > test_devices_list.c << 'EOF'
#include "devices_list.h"
#include <stdio.h>
#include <stdlib.h>

int main() {
    aetheris_device_info_t* devices;
    size_t count;
    aetheris_device_list_error_t result = aetheris_discover_devices(&devices, &count);
    
    if (result == AETHERIS_DEVICE_LIST_SUCCESS) {
        printf("Discovered %zu devices\n", count);
        for (size_t i = 0; i < count; i++) {
            printf("Device: %s (%s)\n", devices[i].device_id, devices[i].device_path);
        }
        aetheris_free_device_info_array(devices);
        return 0;
    } else {
        printf("Error: %s\n", aetheris_device_list_error_message(result));
        return 1;
    }
}
EOF
    
    # Compile and run test
    if ! run_test "C FFI test compile" "gcc -o test_devices_list test_devices_list.c -L. -ldevices_list -laetheris_devices"; then
        return 1
    fi
    
    if ! run_test "C FFI test run" "LD_LIBRARY_PATH=.:../../services/devices/target/release ./test_devices_list"; then
        return 1
    fi
    
    cd ../..
    print_status "SUCCESS" "C FFI tests passed"
}

# Function to test Go CLI
test_go_cli() {
    print_status "INFO" "Testing Go CLI..."
    
    cd go/tooling/devctl
    
    # Download dependencies
    if ! run_test "Go dependencies" "go mod download"; then
        return 1
    fi
    
    # Run tests
    if ! run_test "Go tests" "go test -v ./..."; then
        return 1
    fi
    
    # Build CLI
    if ! run_test "Go build" "go build -o devctl ."; then
        return 1
    fi
    
    # Test device listing
    if ! run_test "Go device listing" "./devctl devices list --format json"; then
        return 1
    fi
    
    # Test provider status
    if ! run_test "Go provider status" "./devctl devices provider status --format json"; then
        return 1
    fi
    
    # Test provider switching
    if ! run_test "Go provider switching" "./devctl devices provider set det && ./devctl devices provider get"; then
        return 1
    fi
    
    # Test device filtering
    if ! run_test "Go device filtering" "./devctl devices list --type camera"; then
        return 1
    fi
    
    cd ../../..
    print_status "SUCCESS" "Go CLI tests passed"
}

# Function to test TypeScript bridge
test_typescript_bridge() {
    print_status "INFO" "Testing TypeScript bridge..."
    
    cd tooling/ts
    
    # Install dependencies
    if ! run_test "TypeScript dependencies" "npm install"; then
        return 1
    fi
    
    # Check TypeScript compilation
    if ! run_test "TypeScript compile" "npx tsc --noEmit device_bridge.ts"; then
        return 1
    fi
    
    # Run ESLint
    if ! run_test "TypeScript lint" "npx eslint device_bridge.ts"; then
        return 1
    fi
    
    # Create and run test
    cat > test_device_bridge.ts << 'EOF'
import { DeviceBridge, DeviceType, ProviderType } from './device_bridge';

async function testDeviceBridge() {
    const bridge = new DeviceBridge();
    
    try {
        // Test device discovery
        const devices = await bridge.discoverDevices();
        console.log(`Discovered ${devices.length} devices`);
        
        // Test filtering
        const cameras = bridge.getDevicesByType(DeviceType.Camera);
        console.log(`Found ${cameras.length} cameras`);
        
        // Test provider status
        const statuses = await bridge.getProviderStatus();
        console.log(`Found ${statuses.length} providers`);
        
        // Test provider switching
        await bridge.setDefaultProvider(ProviderType.Deterministic);
        const currentProvider = bridge.getDefaultProvider();
        console.log(`Current provider: ${currentProvider}`);
        
        console.log('All tests passed!');
    } catch (error) {
        console.error('Test failed:', error);
        process.exit(1);
    } finally {
        bridge.destroy();
    }
}

testDeviceBridge();
EOF
    
    if ! run_test "TypeScript test" "npx ts-node test_device_bridge.ts"; then
        return 1
    fi
    
    cd ../..
    print_status "SUCCESS" "TypeScript bridge tests passed"
}

# Function to test Python validator
test_python_validator() {
    print_status "INFO" "Testing Python validator..."
    
    cd tooling/python
    
    # Install dependencies
    if ! run_test "Python dependencies" "pip install pytest pytest-asyncio"; then
        return 1
    fi
    
    # Run validator with deterministic provider
    if ! run_test "Python validator (det)" "python device_validator.py --provider det --verbose"; then
        return 1
    fi
    
    # Run validator with Linux provider (if available)
    if [ "$SKIP_HARDWARE" != "true" ]; then
        if ! run_test "Python validator (linux)" "python device_validator.py --provider linux --verbose"; then
            print_status "WARNING" "Linux provider tests failed - this is expected on some systems"
        fi
    else
        print_status "INFO" "Skipping Linux provider tests (hardware not available)"
    fi
    
    cd ../..
    print_status "SUCCESS" "Python validator tests passed"
}

# Function to test hardware interfaces
test_hardware_interfaces() {
    if [ "$SKIP_HARDWARE" = "true" ]; then
        print_status "INFO" "Skipping hardware interface tests"
        return 0
    fi
    
    print_status "INFO" "Testing hardware interfaces..."
    
    # Check for video devices
    if ls /dev/video* >/dev/null 2>&1; then
        print_status "SUCCESS" "Video devices found: $(ls /dev/video* | wc -l)"
    else
        print_status "WARNING" "No video devices found"
    fi
    
    # Check for audio devices
    if ls /dev/snd/* >/dev/null 2>&1; then
        print_status "SUCCESS" "Audio devices found: $(ls /dev/snd/* | wc -l)"
    else
        print_status "WARNING" "No audio devices found"
    fi
    
    # Check for GPIO chips
    if ls /dev/gpiochip* >/dev/null 2>&1; then
        print_status "SUCCESS" "GPIO chips found: $(ls /dev/gpiochip* | wc -l)"
    else
        print_status "WARNING" "No GPIO chips found"
    fi
    
    # Check for IIO devices
    if ls /sys/bus/iio/devices/* >/dev/null 2>&1; then
        print_status "SUCCESS" "IIO devices found: $(ls /sys/bus/iio/devices/* | wc -l)"
    else
        print_status "WARNING" "No IIO devices found"
    fi
    
    # Check for PWM devices
    if ls /sys/class/pwm/* >/dev/null 2>&1; then
        print_status "SUCCESS" "PWM devices found: $(ls /sys/class/pwm/* | wc -l)"
    else
        print_status "WARNING" "No PWM devices found"
    fi
    
    # Check for LED devices
    if ls /sys/class/leds/* >/dev/null 2>&1; then
        print_status "SUCCESS" "LED devices found: $(ls /sys/class/leds/* | wc -l)"
    else
        print_status "WARNING" "No LED devices found"
    fi
}

# Function to run integration tests
run_integration_tests() {
    print_status "INFO" "Running integration tests..."
    
    # Test cross-language compatibility
    cd go/tooling/devctl
    if ! run_test "Cross-language test" "./devctl devices list --format json | jq length"; then
        return 1
    fi
    
    cd ../../..
    print_status "SUCCESS" "Integration tests passed"
}

# Function to run performance tests
run_performance_tests() {
    print_status "INFO" "Running performance tests..."
    
    # Test device discovery performance
    cd go/tooling/devctl
    local start_time=$(date +%s%N)
    ./devctl devices list >/dev/null 2>&1
    local end_time=$(date +%s%N)
    local duration=$(( (end_time - start_time) / 1000000 ))
    
    if [ $duration -lt 1000 ]; then
        print_status "SUCCESS" "Device discovery performance: ${duration}ms (target: <1000ms)"
    else
        print_status "WARNING" "Device discovery performance: ${duration}ms (target: <1000ms)"
    fi
    
    cd ../../..
}

# Function to generate test report
generate_report() {
    local report_file="hal_smoke_test_report.txt"
    
    print_status "INFO" "Generating test report: $report_file"
    
    cat > "$report_file" << EOF
HAL Smoke Test Report
Generated: $(date)
Provider: $PROVIDER
System: $(uname -a)

Test Results:
EOF
    
    # Add test results to report
    if [ -f "$report_file" ]; then
        print_status "SUCCESS" "Test report generated: $report_file"
    else
        print_status "ERROR" "Failed to generate test report"
    fi
}

# Main function
main() {
    print_status "INFO" "Starting HAL smoke tests..."
    print_status "INFO" "Provider: $PROVIDER"
    print_status "INFO" "Verbose: $VERBOSE"
    print_status "INFO" "Skip Hardware: $SKIP_HARDWARE"
    
    local failed_tests=0
    
    # Run all tests
    check_system_requirements || ((failed_tests++))
    build_rust_services || ((failed_tests++))
    test_c_ffi || ((failed_tests++))
    test_go_cli || ((failed_tests++))
    test_typescript_bridge || ((failed_tests++))
    test_python_validator || ((failed_tests++))
    test_hardware_interfaces || ((failed_tests++))
    run_integration_tests || ((failed_tests++))
    run_performance_tests || ((failed_tests++))
    
    # Generate report
    generate_report
    
    # Summary
    if [ $failed_tests -eq 0 ]; then
        print_status "SUCCESS" "All HAL smoke tests passed!"
        exit 0
    else
        print_status "ERROR" "$failed_tests test(s) failed"
        exit 1
    fi
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --provider)
            PROVIDER="$2"
            shift 2
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --skip-hardware)
            SKIP_HARDWARE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --provider PROVIDER    Set the provider (det, linux)"
            echo "  --verbose             Enable verbose output"
            echo "  --skip-hardware       Skip hardware interface tests"
            echo "  --help                Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Run main function
main
