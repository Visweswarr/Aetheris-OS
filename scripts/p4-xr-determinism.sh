#!/bin/bash
# P4 XR Determinism Test
# Tests deterministic behavior of XR scene operations and persistence

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${CYAN}📋 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to cleanup on exit
cleanup() {
    print_info "Cleaning up temporary files..."
    rm -rf artifacts/xr/
    print_info "Cleanup complete"
}

# Set up cleanup trap
trap cleanup EXIT

# Function to run deterministic XR sequence
run_deterministic_sequence() {
    local iteration=$1
    local snapshot_file=$2
    
    print_status "Running deterministic sequence iteration $iteration..."
    
    # Start XR demo session
    print_info "Starting XR demo session..."
    if xrctl demo run --headless --spawn cube --bind-did did:aeth:demo; then
        print_success "XR demo session started"
    else
        print_error "Failed to start XR demo session"
        return 1
    fi
    
    # Wait for session to stabilize
    sleep 2
    
    # Spawn initial cube
    print_info "Spawning initial cube..."
    if xrctl scene spawn --room room_demo --mesh cube --pos 0,1,0 --color red; then
        print_success "Initial cube spawned"
    else
        print_error "Failed to spawn initial cube"
        return 1
    fi
    
    # Move cube to position 1
    print_info "Moving cube to position 1..."
    if xrctl scene move --room room_demo --object cube --pos 1,1,0; then
        print_success "Cube moved to position 1"
    else
        print_error "Failed to move cube to position 1"
        return 1
    fi
    
    # Spawn second cube
    print_info "Spawning second cube..."
    if xrctl scene spawn --room room_demo --mesh cube --pos 2,1,0 --color blue; then
        print_success "Second cube spawned"
    else
        print_error "Failed to spawn second cube"
        return 1
    fi
    
    # Move second cube
    print_info "Moving second cube..."
    if xrctl scene move --room room_demo --object cube_2 --pos 2,2,0; then
        print_success "Second cube moved"
    else
        print_error "Failed to move second cube"
        return 1
    fi
    
    # Spawn sphere
    print_info "Spawning sphere..."
    if xrctl scene spawn --room room_demo --mesh sphere --pos 0,2,0 --color green; then
        print_success "Sphere spawned"
    else
        print_error "Failed to spawn sphere"
        return 1
    fi
    
    # Move sphere
    print_info "Moving sphere..."
    if xrctl scene move --room room_demo --object sphere --pos 1,2,0; then
        print_success "Sphere moved"
    else
        print_error "Failed to move sphere"
        return 1
    fi
    
    # Wait for physics to stabilize
    sleep 1
    
    # Save snapshot
    print_info "Saving snapshot to $snapshot_file..."
    if xrctl persist snapshot save --room room_demo --out "$snapshot_file"; then
        print_success "Snapshot saved successfully"
    else
        print_error "Failed to save snapshot"
        return 1
    fi
    
    # Stop demo session
    print_info "Stopping XR demo session..."
    if xrctl demo stop; then
        print_success "XR demo session stopped"
    else
        print_warning "Failed to stop XR demo session (may not be running)"
    fi
    
    return 0
}

# Main determinism test function
main() {
    echo -e "${GREEN}🚀 P4 XR Determinism Test${NC}"
    echo -e "${GREEN}==========================${NC}"
    echo ""
    
    # Check prerequisites
    print_status "Checking prerequisites..."
    
    if ! command_exists xrctl; then
        print_error "xrctl not found. Please build it first:"
        echo "  cd go/tooling/xrctl && go build -o xrctl ."
        exit 1
    fi
    
    if ! command_exists sha256sum; then
        print_error "sha256sum not found. Please install coreutils."
        exit 1
    fi
    
    print_success "Prerequisites check passed"
    
    # Create necessary directories
    print_status "Setting up test environment..."
    mkdir -p artifacts/xr
    
    print_success "Test environment ready"
    
    # Set deterministic environment
    print_status "Setting deterministic environment..."
    export AETHERIS_DETERMINISTIC=true
    export AETHERIS_SEED=42
    export AETHERIS_XR_PHYSICS_TICK=60
    export AETHERIS_XR_TARGET_FPS=90
    
    print_success "Deterministic environment configured"
    
    # Run deterministic sequence 3 times
    print_status "Running deterministic sequence 3 times..."
    
    local snapshots=()
    local success=true
    
    for i in {1..3}; do
        local snapshot_file="artifacts/xr/snap${i}.cbor"
        snapshots+=("$snapshot_file")
        
        print_info "=== Iteration $i ==="
        
        if run_deterministic_sequence "$i" "$snapshot_file"; then
            print_success "Iteration $i completed successfully"
        else
            print_error "Iteration $i failed"
            success=false
            break
        fi
        
        # Small delay between iterations
        sleep 1
    done
    
    if [ "$success" = false ]; then
        print_error "❌ P4 XR Determinism Test FAILED - Sequence execution failed"
        exit 1
    fi
    
    # Compare snapshots
    print_status "Comparing snapshots for determinism..."
    
    # Check if all snapshot files exist
    for snapshot in "${snapshots[@]}"; do
        if [ ! -f "$snapshot" ]; then
            print_error "Snapshot file not found: $snapshot"
            exit 1
        fi
    done
    
    # Calculate SHA256 hashes
    print_info "Calculating SHA256 hashes..."
    local hashes=()
    for snapshot in "${snapshots[@]}"; do
        local hash=$(sha256sum "$snapshot" | cut -d' ' -f1)
        hashes+=("$hash")
        print_info "Hash for $(basename "$snapshot"): $hash"
    done
    
    # Compare hashes
    print_info "Comparing hashes..."
    local first_hash="${hashes[0]}"
    local all_match=true
    
    for i in "${!hashes[@]}"; do
        if [ "${hashes[$i]}" != "$first_hash" ]; then
            print_error "Hash mismatch detected!"
            print_error "  snap1.cbor: $first_hash"
            print_error "  snap$((i+1)).cbor: ${hashes[$i]}"
            all_match=false
        fi
    done
    
    if [ "$all_match" = true ]; then
        print_success "All snapshots have identical hashes"
        print_success "✅ P4 XR Determinism Test PASSED"
        echo ""
        print_info "Summary:"
        echo "  ✅ XR demo sessions started successfully"
        echo "  ✅ Deterministic spawn/move sequences executed"
        echo "  ✅ Snapshots saved for all 3 iterations"
        echo "  ✅ All snapshots are byte-identical (deterministic)"
        echo ""
        print_info "Artifacts generated:"
        echo "  📄 artifacts/xr/snap1.cbor (SHA256: $first_hash)"
        echo "  📄 artifacts/xr/snap2.cbor (SHA256: $first_hash)"
        echo "  📄 artifacts/xr/snap3.cbor (SHA256: $first_hash)"
        echo ""
        print_info "Determinism verified: All snapshots are byte-identical"
        echo ""
        exit 0
    else
        print_error "❌ P4 XR Determinism Test FAILED - Snapshots are not deterministic"
        echo ""
        print_info "Debug information:"
        echo "  Snapshot file sizes:"
        for snapshot in "${snapshots[@]}"; do
            echo "    $(basename "$snapshot"): $(wc -c < "$snapshot") bytes"
        done
        echo ""
        print_info "First 100 bytes of each snapshot:"
        for snapshot in "${snapshots[@]}"; do
            echo "  $(basename "$snapshot"):"
            hexdump -C "$snapshot" | head -n 5
        done
        echo ""
        exit 1
    fi
}

# Run main function
main "$@"
