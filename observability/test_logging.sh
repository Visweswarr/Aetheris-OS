#!/bin/bash

set -e

echo "🧪 Testing Polymera OS Logging System..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Please run this script from the observability directory"
    exit 1
fi

# Build the observability crate with logging features
echo "🔨 Building observability crate with logging features..."
cargo build --features full

# Run unit tests for logging
echo "🧪 Running logging unit tests..."
cargo test --features full logging

# Run redaction tests specifically
echo "🔒 Testing PII redaction functionality..."
cargo test --features full redaction

# Test the logging functionality
echo "📝 Testing logging functionality..."
cargo test --features full --test logging_integration

# Test PII redaction with sample data
echo "🛡️ Testing PII redaction with sample data..."
cargo run --bin observability-test --features full

# Verify that sensitive data is properly redacted
echo "✅ Verifying PII redaction..."
cargo test --features full test_pii_redaction

# Test correlation ID functionality
echo "🆔 Testing correlation ID functionality..."
cargo test --features full test_correlation_context

# Test structured logging
echo "📊 Testing structured logging..."
cargo test --features full test_structured_logging

# Test different redaction modes
echo "🎭 Testing different redaction modes..."
cargo test --features full test_redaction_modes

# Performance testing
echo "⚡ Testing logging performance..."
cargo run --bin observability-bench --features full

echo ""
echo "🎉 Logging system testing completed!"
echo ""
echo "📊 Test Results Summary:"
echo "   ✅ PII redaction working correctly"
echo "   ✅ Correlation IDs functioning"
echo "   ✅ Structured logging operational"
echo "   ✅ Multiple redaction modes supported"
echo "   ✅ Performance benchmarks completed"
echo ""
echo "🔧 Key Features Verified:"
echo "   - Automatic PII detection and redaction"
echo "   - Request correlation across threads"
echo "   - JSON and text logging formats"
echo "   - Configurable sampling and privacy modes"
echo "   - Rich field metadata and context"
echo ""
echo "📚 For more information, see:"
echo "   - docs/LOGGING.md - Complete logging documentation"
echo "   - observability/logging/ - Source code"
echo "   - observability/README.md - General observability guide"
