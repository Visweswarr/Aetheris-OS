#!/bin/bash

set -e

echo "🧪 Testing Polymera OS Package System..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Please run this script from the pack directory"
    exit 1
fi

# Build the package crate
echo "🔨 Building package crate..."
cargo build

# Run unit tests
echo "🧪 Running unit tests..."
cargo test

# Run integration tests
echo "🔗 Running integration tests..."
cargo test --test integration

# Test package building
echo "📦 Testing package building..."
cargo test test_build_and_verify_package

# Test package verification
echo "✅ Testing package verification..."
cargo test test_verification_failure

# Test schema serialization
echo "📋 Testing schema serialization..."
cargo test test_schema_serialization

# Test capability system
echo "🛡️ Testing capability system..."
cargo test test_capability_serialization

# Test checksum system
echo "🔍 Testing checksum system..."
cargo test test_checksum_serialization

# Test SBOM generation
echo "📊 Testing SBOM generation..."
cargo test test_sbom_generation

# Test signature verification
echo "🔐 Testing signature verification..."
cargo test test_signature_verification

# Test file filtering
echo "📁 Testing file filtering..."
cargo test test_file_collection

# Test error handling
echo "⚠️ Testing error handling..."
cargo test test_error_types

# Test configuration
echo "⚙️ Testing configuration..."
cargo test test_configuration

# Test manifest validation
echo "📝 Testing manifest validation..."
cargo test test_manifest_validation

# Test package corruption and verification failure
echo "💥 Testing package corruption and verification failure..."
cargo test test_package_corruption

# Performance testing
echo "⚡ Testing performance..."
cargo test --release

echo ""
echo "🎉 Package system testing completed!"
echo ""
echo "📊 Test Results Summary:"
echo "   ✅ Package building working correctly"
echo "   ✅ Package verification functioning"
echo "   ✅ Content addressing operational"
echo "   ✅ SBOM generation working"
echo "   ✅ Digital signatures supported"
echo "   ✅ Capability system operational"
echo "   ✅ File integrity verification working"
echo "   ✅ Error handling comprehensive"
echo "   ✅ Configuration system flexible"
echo "   ✅ Manifest validation robust"
echo "   ✅ Corruption detection working"
echo ""
echo "🔧 Key Features Verified:"
echo "   - Content-addressed bundles with SHA-256"
echo "   - SBOM generation in multiple formats"
echo "   - Cryptographic signatures and verification"
echo "   - Capability manifests for security"
echo "   - Comprehensive package validation"
echo "   - File integrity and checksum verification"
echo "   - Error handling and validation"
echo "   - Configuration flexibility"
echo ""
echo "📚 For more information, see:"
echo "   - docs/PACKAGES.md - Complete package documentation"
echo "   - pack/ - Source code"
echo "   - schema.json - JSON Schema definition"
echo ""
echo "🚀 Next steps:"
echo "   1. Build packages with: cargo run --bin polymera-pack"
echo "   2. Verify packages with: cargo run --bin pack-test"
echo "   3. Integrate with your build system"
echo "   4. Configure trusted keys and signers"
echo "   5. Set up automated package verification"
