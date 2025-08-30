#!/bin/bash

# Rust Fuzzing Script for Polymera OS
# Runs cargo-fuzz on all fuzz targets and manages corpora

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
FUZZ_DURATION=${FUZZ_DURATION:-60}  # Default 60 seconds per target
MAX_CRASHES=${MAX_CRASHES:-10}      # Maximum crashes to collect per target
CORPUS_DIR="corpus"
CRASHES_DIR="crashes"
ARTIFACTS_DIR="artifacts"

# Fuzz targets
TARGETS=(
    "capability_token_format_fuzzer"
    "capability_token_sign_fuzzer"
    "capability_token_verify_fuzzer"
    "json_parser_fuzzer"
    "base64_parser_fuzzer"
    "uuid_parser_fuzzer"
)

# Create necessary directories
mkdir -p "$CORPUS_DIR" "$CRASHES_DIR" "$ARTIFACTS_DIR"

echo -e "${BLUE}🚀 Starting Rust Fuzzing Suite for Polymera OS${NC}"
echo -e "${BLUE}Duration per target: ${FUZZ_DURATION}s${NC}"
echo -e "${BLUE}Max crashes per target: ${MAX_CRASHES}${NC}"
echo ""

# Check if cargo-fuzz is installed
if ! command -v cargo-fuzz &> /dev/null; then
    echo -e "${YELLOW}⚠️  cargo-fuzz not found. Installing...${NC}"
    cargo install cargo-fuzz
fi

# Function to run fuzzing for a single target
run_fuzz_target() {
    local target=$1
    local corpus_path="$CORPUS_DIR/$target"
    local crashes_path="$CRASHES_DIR/$target"
    
    echo -e "${BLUE}🔍 Fuzzing target: ${target}${NC}"
    
    # Create corpus directory for this target
    mkdir -p "$corpus_path"
    mkdir -p "$crashes_path"
    
    # Seed corpus with some basic test data if empty
    if [ ! "$(ls -A $corpus_path)" ]; then
        echo -e "${YELLOW}📝 Seeding corpus for ${target}...${NC}"
        seed_corpus "$target" "$corpus_path"
    fi
    
    # Run fuzzing
    echo -e "${GREEN}▶️  Running ${target} for ${FUZZ_DURATION}s...${NC}"
    
    timeout "$FUZZ_DURATION" cargo fuzz run "$target" \
        --jobs 1 \
        --max-crashes "$MAX_CRASHES" \
        --corpus "$corpus_path" \
        --crash-dir "$crashes_path" \
        --timeout 10 \
        --max-len 65536 \
        --sanitizer address \
        --release || true
    
    # Check for crashes
    local crash_count=$(find "$crashes_path" -name "*.fuzz" 2>/dev/null | wc -l)
    if [ "$crash_count" -gt 0 ]; then
        echo -e "${RED}💥 Found ${crash_count} crashes in ${target}${NC}"
        
        # Copy crashes to artifacts
        local artifacts_path="$ARTIFACTS_DIR/$target"
        mkdir -p "$artifacts_path"
        cp -r "$crashes_path"/* "$artifacts_path/" 2>/dev/null || true
        
        # Generate crash report
        generate_crash_report "$target" "$crashes_path" "$artifacts_path"
    else
        echo -e "${GREEN}✅ No crashes found in ${target}${NC}"
    fi
    
    echo ""
}

# Function to seed corpus with basic test data
seed_corpus() {
    local target=$1
    local corpus_path=$2
    
    case $target in
        "capability_token_format_fuzzer")
            # Basic capability token JSON
            cat > "$corpus_path/valid_token.json" << 'EOF'
{
  "header": {
    "typ": "capability",
    "alg": "Dilithium3",
    "ver": "1.0.0",
    "kid": "key-123",
    "jti": "token-456",
    "iss": "service.example.com",
    "sub": "user-789",
    "aud": "app.example.com",
    "iat": 1234567890,
    "nbf": 1234567890,
    "exp": 1234567890,
    "additional": {}
  },
  "payload": {
    "purpose": "test",
    "claims": [],
    "scope": "test",
    "level": 1,
    "hierarchy": [],
    "constraints": {},
    "metadata": {},
    "additional": {}
  },
  "signature": null,
  "format_version": "1.0.0"
}
EOF
            ;;
        "json_parser_fuzzer")
            # Basic JSON test cases
            echo '{"key": "value"}' > "$corpus_path/basic.json"
            echo '[1, 2, 3]' > "$corpus_path/array.json"
            echo 'true' > "$corpus_path/boolean.json"
            echo 'null' > "$corpus_path/null.json"
            ;;
        "base64_parser_fuzzer")
            # Basic base64 test cases
            echo 'SGVsbG8gV29ybGQ=' > "$corpus_path/hello_world.txt"
            echo 'UG9seW1lcmEgT1M=' > "$corpus_path/polymera_os.txt"
            ;;
        "uuid_parser_fuzzer")
            # Basic UUID test cases
            echo '550e8400-e29b-41d4-a716-446655440000' > "$corpus_path/valid_uuid.txt"
            echo '12345678-1234-1234-1234-123456789abc' > "$corpus_path/another_uuid.txt"
            ;;
        *)
            # Default test data
            echo 'test' > "$corpus_path/default.txt"
            ;;
    esac
}

# Function to generate crash report
generate_crash_report() {
    local target=$1
    local crashes_path=$2
    local artifacts_path=$3
    
    local report_file="$artifacts_path/crash_report.txt"
    
    {
        echo "Crash Report for $target"
        echo "Generated: $(date)"
        echo "Target: $target"
        echo "Crashes found: $(find "$crashes_path" -name "*.fuzz" 2>/dev/null | wc -l)"
        echo ""
        echo "Crash files:"
        find "$crashes_path" -name "*.fuzz" 2>/dev/null | while read -r crash_file; do
            echo "  - $(basename "$crash_file")"
            echo "    Size: $(stat -c%s "$crash_file" 2>/dev/null || echo "unknown") bytes"
            echo "    Modified: $(stat -c%y "$crash_file" 2>/dev/null || echo "unknown")"
        done
        echo ""
        echo "Environment:"
        echo "  Rust version: $(rustc --version 2>/dev/null || echo "unknown")"
        echo "  Cargo version: $(cargo --version 2>/dev/null || echo "unknown")"
        echo "  OS: $(uname -a 2>/dev/null || echo "unknown")"
    } > "$report_file"
}

# Function to clean up old artifacts
cleanup_artifacts() {
    echo -e "${YELLOW}🧹 Cleaning up old artifacts...${NC}"
    
    # Keep only the latest artifacts
    if [ -d "$ARTIFACTS_DIR" ]; then
        find "$ARTIFACTS_DIR" -type f -name "*.txt" -mtime +7 -delete 2>/dev/null || true
    fi
    
    echo -e "${GREEN}✅ Cleanup complete${NC}"
}

# Function to show summary
show_summary() {
    echo -e "${BLUE}📊 Fuzzing Summary${NC}"
    echo "=================="
    
    local total_crashes=0
    local targets_with_crashes=0
    
    for target in "${TARGETS[@]}"; do
        local crashes_path="$CRASHES_DIR/$target"
        local crash_count=$(find "$crashes_path" -name "*.fuzz" 2>/dev/null | wc -l)
        
        if [ "$crash_count" -gt 0 ]; then
            echo -e "${RED}💥 ${target}: ${crash_count} crashes${NC}"
            total_crashes=$((total_crashes + crash_count))
            targets_with_crashes=$((targets_with_crashes + 1))
        else
            echo -e "${GREEN}✅ ${target}: No crashes${NC}"
        fi
    done
    
    echo ""
    echo -e "${BLUE}Total crashes found: ${total_crashes}${NC}"
    echo -e "${BLUE}Targets with crashes: ${targets_with_crashes}${NC}"
    echo -e "${BLUE}Artifacts saved to: ${ARTIFACTS_DIR}${NC}"
}

# Main execution
main() {
    # Clean up old artifacts
    cleanup_artifacts
    
    # Run fuzzing for each target
    for target in "${TARGETS[@]}"; do
        run_fuzz_target "$target"
    done
    
    # Show summary
    show_summary
    
    echo -e "${GREEN}🎉 Fuzzing complete!${NC}"
}

# Run main function
main "$@"
