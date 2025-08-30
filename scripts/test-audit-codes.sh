#!/bin/bash
# Audit Codes System Test Script for Polymera OS
# 
# This script tests the audit codes system by:
# 1. Building the kernel with audit codes enabled
# 2. Running the kernel to verify audit event emission
# 3. Checking that syscall entry/exit events are recorded
# 4. Verifying exec load events are captured
# 5. Testing rate limiting functionality

set -e

echo "[AUDIT-CODES] Testing audit codes system..."

# Check if we're in the right directory
if [ ! -f "kernel/Cargo.toml" ]; then
    echo "Error: Must run from polymera-os root directory"
    exit 1
fi

# Build kernel
echo "[AUDIT-CODES] Building kernel..."
cd kernel
cargo build --release

if [ $? -ne 0 ]; then
    echo "Error: Kernel build failed"
    exit 1
fi

echo "[AUDIT-CODES] Kernel built successfully"

# Run kernel to test audit codes system
echo "[AUDIT-CODES] Running kernel to test audit codes system..."
cd ..

# Use QEMU to run the kernel and capture serial output
echo "[AUDIT-CODES] Starting QEMU for audit codes test..."
timeout 30s qemu-system-x86_64 \
    -kernel kernel/target/x86_64-unknown-none/release/polymera-kernel \
    -serial stdio \
    -display none \
    -m 512M \
    -smp 1 \
    -no-reboot \
    -no-shutdown \
    -nographic \
    -append "console=ttyS0" 2>/dev/null | tee /tmp/audit-codes-test.log

# Check if the audit codes system was initialized successfully
if grep -q "Audit codes system initialized successfully" /tmp/audit-codes-test.log; then
    echo "✓ Audit codes system initialized successfully"
else
    echo "✗ Audit codes system failed to initialize"
    exit 1
fi

# Check if syscall entry audit events are being emitted
if grep -q "SYSCALL_ENTRY" /tmp/audit-codes-test.log; then
    echo "✓ Syscall entry audit events are being emitted"
else
    echo "✗ No syscall entry audit events found"
    exit 1
fi

# Check if syscall exit audit events are being emitted
if grep -q "SYSCALL_EXIT" /tmp/audit-codes-test.log; then
    echo "✓ Syscall exit audit events are being emitted"
else
    echo "✗ No syscall exit audit events found"
    exit 1
fi

# Check if exec load audit events are being emitted
if grep -q "EXEC_LOAD" /tmp/audit-codes-test.log; then
    echo "✓ Exec load audit events are being emitted"
else
    echo "✗ No exec load audit events found"
    exit 1
fi

# Check if rate limiter is working
if grep -q "Rate limiter initialized" /tmp/audit-codes-test.log; then
    echo "✓ Rate limiter initialized successfully"
else
    echo "✗ Rate limiter initialization failed"
    exit 1
fi

# Check if rate limiting is working
if grep -q "Rate limiting working" /tmp/audit-codes-test.log; then
    echo "✓ Rate limiting functionality is working"
else
    echo "✗ Rate limiting functionality failed"
    exit 1
fi

# Check if audit event statistics are reported
if grep -q "AUDIT CODES SYSTEM STATISTICS" /tmp/audit-codes-test.log; then
    echo "✓ Audit event statistics are being reported"
else
    echo "✗ Audit event statistics not reported"
    exit 1
fi

# Check if events are being encoded correctly
if grep -q "Event encoding:" /tmp/audit-codes-test.log; then
    echo "✓ Event encoding is working correctly"
else
    echo "✗ Event encoding failed"
    exit 1
fi

# Check if payload encoding is working
if grep -q "Payload encoding:" /tmp/audit-codes-test.log; then
    echo "✓ Payload encoding is working correctly"
else
    echo "✗ Payload encoding failed"
    exit 1
fi

# Check if reason codes are working correctly
if grep -q "Reason code: SYSCALL_ENTRY" /tmp/audit-codes-test.log; then
    echo "✓ Reason codes are working correctly"
else
    echo "✗ Reason codes failed"
    exit 1
fi

# Check if categories are working correctly
if grep -q "Category: BOUNDARY" /tmp/audit-codes-test.log; then
    echo "✓ Event categories are working correctly"
else
    echo "✗ Event categories failed"
    exit 1
fi

# Check if severity levels are working correctly
if grep -q "Severity: LOW" /tmp/audit-codes-test.log; then
    echo "✓ Severity levels are working correctly"
else
    echo "✗ Severity levels failed"
    exit 1
fi

# Check if boundary transition detection is working
if grep -q "Is boundary transition: true" /tmp/audit-codes-test.log; then
    echo "✓ Boundary transition detection is working"
else
    echo "✗ Boundary transition detection failed"
    exit 1
fi

# Check if audit event emission is working
if grep -q "Audit event emitted successfully" /tmp/audit-codes-test.log; then
    echo "✓ Audit event emission is working correctly"
else
    echo "✗ Audit event emission failed"
    exit 1
fi

# Clean up
rm -f /tmp/audit-codes-test.log

echo "[AUDIT-CODES] All tests passed successfully!"
echo "[AUDIT-CODES] Audit codes system is working correctly"

# Print summary of what was tested
echo ""
echo "=== AUDIT CODES SYSTEM TEST SUMMARY ==="
echo "✓ System initialization and configuration"
echo "✓ Syscall entry/exit audit event emission"
echo "✓ Exec load audit event emission"
echo "✓ Rate limiter initialization and operation"
echo "✓ Event encoding and payload handling"
echo "✓ Reason code categorization and severity"
echo "✓ Boundary transition detection"
echo "✓ Statistics reporting and monitoring"
echo "=== TEST SUMMARY COMPLETE ==="
echo ""

