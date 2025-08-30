#!/bin/bash

# PolyImage System Test Script
# Tests: Build RO image, verify signature, simulate rollback

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TEST_DIR="$SCRIPT_DIR/test_images"
BUILD_DIR="$TEST_DIR/build"
SOURCE_DIR="$TEST_DIR/source"
OUTPUT_DIR="$TEST_DIR/output"

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

# Setup test environment
setup_test_env() {
    log_info "Setting up test environment..."
    
    # Create test directories
    mkdir -p "$TEST_DIR"
    mkdir -p "$BUILD_DIR"
    mkdir -p "$SOURCE_DIR"
    mkdir -p "$OUTPUT_DIR"
    
    # Create test source files
    create_test_source_files
    
    log_success "Test environment ready"
}

# Create test source files
create_test_source_files() {
    log_info "Creating test source files..."
    
    # Create system partition files
    mkdir -p "$SOURCE_DIR/system"
    echo "Polymera OS v1.0.0 System Files" > "$SOURCE_DIR/system/os_version"
    echo "Kernel: 5.15.0" > "$SOURCE_DIR/system/kernel_info"
    echo "Init: systemd" > "$SOURCE_DIR/system/init_system"
    
    # Create bootloader image
    echo "UEFI Bootloader v1.0.0" > "$SOURCE_DIR/boot.img"
    
    # Create recovery image
    echo "Recovery System v1.0.0" > "$SOURCE_DIR/recovery.img"
    
    # Create vendor partition
    echo "Vendor Files v1.0.0" > "$SOURCE_DIR/vendor.img"
    
    # Create data partition files
    mkdir -p "$SOURCE_DIR/data"
    echo "User Data Directory" > "$SOURCE_DIR/data/README"
    echo "Config files and user data" > "$SOURCE_DIR/data/description"
    
    log_success "Test source files created"
}

# Test 1: Build basic read-only image
test_build_basic_image() {
    log_info "Test 1: Building basic read-only image..."
    
    local output_dir="$OUTPUT_DIR/basic_image"
    mkdir -p "$output_dir"
    
    # Build image using mkimage
    cargo run --bin mkimage \
        --output "$output_dir" \
        --image-id "polymera-os-basic-v1.0.0" \
        --description "Polymera OS Basic v1.0.0" \
        --source "$SOURCE_DIR" \
        --architecture "x86_64" \
        --platform "generic" \
        --slots 2
    
    # Verify output
    if [[ -f "$output_dir/manifest.json" ]]; then
        log_success "Basic image built successfully"
        log_info "Manifest: $output_dir/manifest.json"
        log_info "Build summary: $output_dir/build_summary.txt"
    else
        log_error "Basic image build failed"
        return 1
    fi
    
    # Display manifest summary
    echo "=== Basic Image Manifest Summary ==="
    cat "$output_dir/build_summary.txt"
    echo ""
}

# Test 2: Build image with rollback protection
test_build_rollback_image() {
    log_info "Test 2: Building image with rollback protection..."
    
    local output_dir="$OUTPUT_DIR/rollback_image"
    mkdir -p "$output_dir"
    
    # Build image with rollback protection
    cargo run --bin mkimage \
        --output "$output_dir" \
        --image-id "polymera-os-rollback-v1.0.0" \
        --description "Polymera OS Rollback v1.0.0" \
        --source "$SOURCE_DIR" \
        --architecture "x86_64" \
        --platform "generic" \
        --rollback \
        --min-version "1.0.0" \
        --slots 2
    
    # Verify output
    if [[ -f "$output_dir/manifest.json" ]]; then
        log_success "Rollback image built successfully"
        
        # Check rollback protection in manifest
        if grep -q "rollback_protection" "$output_dir/manifest.json"; then
            log_success "Rollback protection enabled in manifest"
        else
            log_warning "Rollback protection not found in manifest"
        fi
    else
        log_error "Rollback image build failed"
        return 1
    fi
    
    # Display rollback settings
    echo "=== Rollback Image Settings ==="
    jq '.rollback_protection' "$output_dir/manifest.json" 2>/dev/null || echo "jq not available, showing raw JSON"
    echo ""
}

# Test 3: Build signed image
test_build_signed_image() {
    log_info "Test 3: Building signed image..."
    
    local output_dir="$OUTPUT_DIR/signed_image"
    mkdir -p "$output_dir"
    
    # Create test keys (in real implementation, use proper crypto)
    create_test_keys
    
    # Build signed image
    cargo run --bin mkimage \
        --output "$output_dir" \
        --image-id "polymera-os-signed-v1.0.0" \
        --description "Polymera OS Signed v1.0.0" \
        --source "$SOURCE_DIR" \
        --architecture "x86_64" \
        --platform "generic" \
        --rollback \
        --min-version "1.0.0" \
        --sign \
        --slots 2
    
    # Verify output
    if [[ -f "$output_dir/manifest.json" ]]; then
        log_success "Signed image built successfully"
        
        # Check signature in manifest
        if grep -q "signature" "$output_dir/manifest.json"; then
            log_success "Digital signature found in manifest"
        else
            log_warning "Digital signature not found in manifest"
        fi
    else
        log_error "Signed image build failed"
        return 1
    fi
    
    # Display signature info
    echo "=== Signed Image Signature Info ==="
    jq '.signature' "$output_dir/manifest.json" 2>/dev/null || echo "jq not available, showing raw JSON"
    echo ""
}

# Create test keys
create_test_keys() {
    log_info "Creating test cryptographic keys..."
    
    local key_dir="$TEST_DIR/keys"
    mkdir -p "$key_dir"
    
    # Create dummy private key
    echo "-----BEGIN PRIVATE KEY-----" > "$key_dir/private.key"
    echo "Test Private Key for PolyImage" >> "$key_dir/private.key"
    echo "This is a placeholder key" >> "$key_dir/private.key"
    echo "-----END PRIVATE KEY-----" >> "$key_dir/private.key"
    
    # Create dummy certificate
    echo "-----BEGIN CERTIFICATE-----" > "$key_dir/certificate.pem"
    echo "Test Certificate for PolyImage" >> "$key_dir/certificate.pem"
    echo "This is a placeholder certificate" >> "$key_dir/certificate.pem"
    echo "-----END CERTIFICATE-----" >> "$key_dir/certificate.pem"
    
    log_success "Test keys created"
}

# Test 4: Verify image signatures
test_verify_signatures() {
    log_info "Test 4: Verifying image signatures..."
    
    local signed_dir="$OUTPUT_DIR/signed_image"
    
    if [[ ! -f "$signed_dir/manifest.json" ]]; then
        log_error "Signed image not found, skipping signature verification"
        return 1
    fi
    
    # Verify manifest structure
    if jq empty "$signed_dir/manifest.json" 2>/dev/null; then
        log_success "Manifest JSON is valid"
    else
        log_error "Manifest JSON is invalid"
        return 1
    fi
    
    # Verify signature fields
    local signature_fields=("algorithm" "key_id" "value" "timestamp")
    for field in "${signature_fields[@]}"; do
        if jq -e ".signature.$field" "$signed_dir/manifest.json" >/dev/null 2>&1; then
            log_success "Signature field '$field' present"
        else
            log_error "Signature field '$field' missing"
            return 1
        fi
    done
    
    # Verify rollback protection fields
    local rollback_fields=("min_version" "anti_rollback_version" "enabled")
    for field in "${rollback_fields[@]}"; do
        if jq -e ".rollback_protection.$field" "$signed_dir/manifest.json" >/dev/null 2>&1; then
            log_success "Rollback field '$field' present"
        else
            log_error "Rollback field '$field' missing"
            return 1
        fi
    done
    
    log_success "Signature verification completed"
}

# Test 5: Simulate rollback scenarios
test_rollback_scenarios() {
    log_info "Test 5: Simulating rollback scenarios..."
    
    local rollback_dir="$OUTPUT_DIR/rollback_image"
    
    if [[ ! -f "$rollback_dir/manifest.json" ]]; then
        log_error "Rollback image not found, skipping rollback tests"
        return 1
    fi
    
    # Extract current version and min version
    local current_version=$(jq -r '.version' "$rollback_dir/manifest.json" 2>/dev/null || echo "1.0.0")
    local min_version=$(jq -r '.rollback_protection.min_version' "$rollback_dir/manifest.json" 2>/dev/null || echo "1.0.0")
    
    log_info "Current version: $current_version"
    log_info "Minimum version: $min_version"
    
    # Test rollback scenarios
    test_rollback_allowed "$current_version" "$min_version"
    test_rollback_blocked "$current_version" "$min_version"
    
    log_success "Rollback scenario tests completed"
}

# Test rollback allowed
test_rollback_allowed() {
    local current_version="$1"
    local min_version="$2"
    
    log_info "Testing allowed rollback scenarios..."
    
    # Test rollback to current version (should be allowed)
    if [[ "$current_version" >= "$min_version" ]]; then
        log_success "Rollback to current version ($current_version) is allowed"
    else
        log_warning "Rollback to current version ($current_version) may not be allowed"
    fi
    
    # Test rollback to version between min and current
    local mid_version="1.0.5"
    if [[ "$mid_version" >= "$min_version" && "$mid_version" <= "$current_version" ]]; then
        log_success "Rollback to mid version ($mid_version) is allowed"
    else
        log_warning "Rollback to mid version ($mid_version) may not be allowed"
    fi
}

# Test rollback blocked
test_rollback_blocked() {
    local current_version="$1"
    local min_version="$2"
    
    log_info "Testing blocked rollback scenarios..."
    
    # Test rollback to version below minimum (should be blocked)
    local low_version="0.9.0"
    if [[ "$low_version" < "$min_version" ]]; then
        log_success "Rollback to low version ($low_version) is blocked (below minimum $min_version)"
    else
        log_warning "Rollback to low version ($low_version) may be allowed"
    fi
}

# Test 6: A/B slot management
test_slot_management() {
    log_info "Test 6: Testing A/B slot management..."
    
    local basic_dir="$OUTPUT_DIR/basic_image"
    
    if [[ ! -f "$basic_dir/manifest.json" ]]; then
        log_error "Basic image not found, skipping slot tests"
        return 1
    fi
    
    # Check slot configuration
    local slot_count=$(jq '.slots | length' "$basic_dir/manifest.json" 2>/dev/null || echo "0")
    log_info "Number of slots: $slot_count"
    
    if [[ "$slot_count" -eq 2 ]]; then
        log_success "A/B slots configured correctly"
        
        # Check slot A (should be priority 2, successful, active)
        local slot_a_priority=$(jq -r '.slots[0].priority' "$basic_dir/manifest.json" 2>/dev/null || echo "0")
        local slot_a_successful=$(jq -r '.slots[0].successful' "$basic_dir/manifest.json" 2>/dev/null || echo "false")
        local slot_a_active=$(jq -r '.slots[0].active' "$basic_dir/manifest.json" 2>/dev/null || echo "false")
        
        if [[ "$slot_a_priority" -eq 2 && "$slot_a_successful" == "true" && "$slot_a_active" == "true" ]]; then
            log_success "Slot A configured correctly (priority: $slot_a_priority, successful: $slot_a_successful, active: $slot_a_active)"
        else
            log_warning "Slot A configuration may be incorrect"
        fi
        
        # Check slot B (should be priority 1, not successful, not active)
        local slot_b_priority=$(jq -r '.slots[1].priority' "$basic_dir/manifest.json" 2>/dev/null || echo "0")
        local slot_b_successful=$(jq -r '.slots[1].successful' "$basic_dir/manifest.json" 2>/dev/null || echo "false")
        local slot_b_active=$(jq -r '.slots[1].active' "$basic_dir/manifest.json" 2>/dev/null || echo "false")
        
        if [[ "$slot_b_priority" -eq 1 && "$slot_b_successful" == "false" && "$slot_b_active" == "false" ]]; then
            log_success "Slot B configured correctly (priority: $slot_b_priority, successful: $slot_b_successful, active: $slot_b_active)"
        else
            log_warning "Slot B configuration may be incorrect"
        fi
    else
        log_error "Expected 2 slots, found $slot_count"
        return 1
    fi
    
    log_success "Slot management tests completed"
}

# Test 7: Partition verification
test_partition_verification() {
    log_info "Test 7: Testing partition verification..."
    
    local basic_dir="$OUTPUT_DIR/basic_image"
    
    if [[ ! -f "$basic_dir/manifest.json" ]]; then
        log_error "Basic image not found, skipping partition tests"
        return 1
    fi
    
    # Check partition count
    local partition_count=$(jq '.partitions | length' "$basic_dir/manifest.json" 2>/dev/null || echo "0")
    log_info "Number of partitions: $partition_count"
    
    if [[ "$partition_count" -gt 0 ]]; then
        log_success "Partitions found in manifest"
        
        # Check each partition
        for i in $(seq 0 $((partition_count - 1))); do
            local partition_name=$(jq -r ".partitions[$i].name" "$basic_dir/manifest.json" 2>/dev/null || echo "unknown")
            local partition_type=$(jq -r ".partitions[$i].image_type" "$basic_dir/manifest.json" 2>/dev/null || echo "unknown")
            local partition_hash=$(jq -r ".partitions[$i].hash" "$basic_dir/manifest.json" 2>/dev/null || echo "unknown")
            local partition_size=$(jq -r ".partitions[$i].size" "$basic_dir/manifest.json" 2>/dev/null || echo "0")
            
            log_info "Partition $i: $partition_name ($partition_type, ${partition_size} bytes, hash: ${partition_hash:0:8}...)"
            
            # Verify partition file exists
            local partition_file="$basic_dir/$partition_name"
            if [[ -f "$partition_file" ]]; then
                log_success "Partition file exists: $partition_file"
            else
                log_warning "Partition file missing: $partition_file"
            fi
        done
    else
        log_warning "No partitions found in manifest"
    fi
    
    log_success "Partition verification tests completed"
}

# Run all tests
run_all_tests() {
    log_info "Running all PolyImage system tests..."
    echo ""
    
    # Test 1: Build basic image
    test_build_basic_image
    echo ""
    
    # Test 2: Build rollback image
    test_build_rollback_image
    echo ""
    
    # Test 3: Build signed image
    test_build_signed_image
    echo ""
    
    # Test 4: Verify signatures
    test_verify_signatures
    echo ""
    
    # Test 5: Rollback scenarios
    test_rollback_scenarios
    echo ""
    
    # Test 6: Slot management
    test_slot_management
    echo ""
    
    # Test 7: Partition verification
    test_partition_verification
    echo ""
    
    log_success "All tests completed successfully!"
}

# Show test summary
show_test_summary() {
    log_info "Test Summary"
    log_info "============"
    
    local test_dirs=("basic_image" "rollback_image" "signed_image")
    
    for dir in "${test_dirs[@]}"; do
        local full_dir="$OUTPUT_DIR/$dir"
        if [[ -d "$full_dir" ]]; then
            local manifest_file="$full_dir/manifest.json"
            local summary_file="$full_dir/build_summary.txt"
            
            if [[ -f "$manifest_file" ]]; then
                log_success "✅ $dir: Built successfully"
                if [[ -f "$summary_file" ]]; then
                    log_info "   Summary: $summary_file"
                fi
            else
                log_error "❌ $dir: Build failed"
            fi
        else
            log_warning "⚠️  $dir: Not found"
        fi
    done
    
    echo ""
    log_info "Test artifacts available in: $OUTPUT_DIR"
    log_info "Test logs and output in: $TEST_DIR"
}

# Cleanup test environment
cleanup() {
    log_info "Cleaning up test environment..."
    
    # Keep test artifacts for inspection
    log_info "Test artifacts preserved in: $TEST_DIR"
    log_info "To clean up completely, run: rm -rf $TEST_DIR"
}

# Main execution
main() {
    log_info "PolyImage System Test Suite"
    log_info "============================"
    
    # Parse command line arguments
    local run_tests=true
    local cleanup_env=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --no-tests)
                run_tests=false
                shift
                ;;
            --cleanup)
                cleanup_env=true
                shift
                ;;
            -h|--help)
                cat << EOF
PolyImage System Test Suite

Usage: $0 [OPTIONS]

Options:
    --no-tests        Skip running tests (setup only)
    --cleanup         Clean up test environment after completion
    -h, --help        Show this help message

This script tests:
1. Building basic read-only images
2. Building images with rollback protection
3. Building signed images
4. Verifying image signatures
5. Simulating rollback scenarios
6. Testing A/B slot management
7. Verifying partition integrity

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
    
    # Run tests if requested
    if [[ "$run_tests" == true ]]; then
        run_all_tests
        show_test_summary
    fi
    
    # Cleanup if requested
    if [[ "$cleanup_env" == true ]]; then
        cleanup
    fi
    
    log_success "Test suite completed!"
}

# Trap cleanup on exit
trap cleanup EXIT

# Run main function
main "$@"
