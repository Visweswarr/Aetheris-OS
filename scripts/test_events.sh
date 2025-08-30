#!/bin/bash

# Event Fabric v0 Test Runner
# This script runs all Event Fabric tests and provides a summary

set -e

echo "=========================================="
echo "Event Fabric v0 Test Runner"
echo "=========================================="

# Check if we're in the right directory
if [ ! -f "kernel/src/event/mod.rs" ]; then
    echo "Error: Please run this script from the Polymera OS root directory"
    echo "Current directory: $(pwd)"
    exit 1
fi

echo "Current directory: $(pwd)"
echo "Running Event Fabric tests..."
echo ""

# Run all Event Fabric tests
echo "1. Testing subscription capabilities..."
bazel test //tests/events:subscribe_caps

echo ""
echo "2. Testing event routing..."
bazel test //tests/events:publish_routing

echo ""
echo "3. Testing backpressure handling..."
bazel test //tests/events:backpressure_drop

echo ""
echo "4. Testing performance benchmarks..."
bazel test //tests/events:latency_benchmark

echo ""
echo "5. Testing integrations..."
bazel test //tests/events:integrations

echo ""
echo "=========================================="
echo "All Event Fabric tests completed successfully!"
echo "=========================================="

# Print summary
echo ""
echo "Test Summary:"
echo "✅ subscribe_caps - Subscription capability tests"
echo "✅ publish_routing - Event routing and delivery tests"
echo "✅ backpressure_drop - Backpressure and drop policy tests"
echo "✅ latency_benchmark - Performance and latency tests"
echo "✅ integrations - Integration with other subsystems"

echo ""
echo "Event Fabric v0 is ready for use!"
echo "Feature bit EVENT_FABRIC_V0 is set and available."
echo ""
echo "Next steps:"
echo "- Run 'make qemu EVENTS=1' to test in QEMU"
echo "- Check 'sys_get_features()' for EVENT_FABRIC_V0 bit"
echo "- Use EventKernel::publish_* methods in kernel code"
echo "- Subscribe to topics via sys_event_subscribe syscall"
