#!/bin/bash
# Development Keyvault Integration Test Script for Polymera OS
# 
# This script tests the development keyvault integration by:
# 1. Building the kernel with dev-keyvault feature enabled
# 2. Running the kernel to verify integration initialization
# 3. Checking that issuer anchors and session metadata are loaded
# 4. Verifying the integration statistics are reported correctly

set -e

echo "[DEV-KEYVAULT] Testing development keyvault integration..."

# Check if we're in the right directory
if [ ! -f "kernel/Cargo.toml" ]; then
    echo "Error: Must run from polymera-os root directory"
    exit 1
fi

# Build kernel with dev-keyvault feature
echo "[DEV-KEYVAULT] Building kernel with dev-keyvault feature..."
cd kernel
cargo build --features dev-keyvault --release

if [ $? -ne 0 ]; then
    echo "Error: Kernel build failed"
    exit 1
fi

echo "[DEV-KEYVAULT] Kernel built successfully"

# Run kernel to test integration
echo "[DEV-KEYVAULT] Running kernel to test integration..."
cd ..

# Use QEMU to run the kernel and capture serial output
echo "[DEV-KEYVAULT] Starting QEMU for integration test..."
timeout 30s qemu-system-x86_64 \
    -kernel kernel/target/x86_64-unknown-none/release/polymera-kernel \
    -serial stdio \
    -display none \
    -m 512M \
    -smp 1 \
    -no-reboot \
    -no-shutdown \
    -nographic \
    -append "console=ttyS0" 2>/dev/null | tee /tmp/dev-keyvault-test.log

# Check if the integration was initialized successfully
if grep -q "Development keyvault integration initialized successfully" /tmp/dev-keyvault-test.log; then
    echo "✓ Development keyvault integration initialized successfully"
else
    echo "✗ Development keyvault integration failed to initialize"
    exit 1
fi

# Check if issuer anchors were loaded
if grep -q "Loaded [0-9]* issuer anchors" /tmp/dev-keyvault-test.log; then
    echo "✓ Issuer anchors loaded successfully"
else
    echo "✗ No issuer anchors loaded"
    exit 1
fi

# Check if session metadata was loaded
if grep -q "Loaded [0-9]* session metadata entries" /tmp/dev-keyvault-test.log; then
    echo "✓ Session metadata loaded successfully"
else
    echo "✗ No session metadata loaded"
    exit 1
fi

# Check if statistics are reported
if grep -q "DEVELOPMENT KEYVAULT INTEGRATION STATISTICS" /tmp/dev-keyvault-test.log; then
    echo "✓ Integration statistics reported correctly"
else
    echo "✗ Integration statistics not reported"
    exit 1
fi

# Clean up
rm -f /tmp/dev-keyvault-test.log

echo "[DEV-KEYVAULT] All tests passed successfully!"
echo "[DEV-KEYVAULT] Development keyvault integration is working correctly"

