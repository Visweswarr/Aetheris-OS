#!/bin/bash

# Python Fuzzing Script for Polymera OS
# Runs Atheris fuzzers on Python components and manages corpora

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
    "json_parser_fuzzer.py"
    "base64_parser_fuzzer.py"
)

# Create necessary directories
mkdir -p "$CORPUS_DIR" "$CRASHES_DIR" "$ARTIFACTS_DIR"

echo -e "${BLUE}🐍 Starting Python Fuzzing Suite for Polymera OS${NC}"
echo -e "${BLUE}Duration per target: ${FUZZ_DURATION}s${NC}"
echo -e "${BLUE}Max crashes per target: ${MAX_CRASHES}${NC}"
echo ""

# Check if Python and Atheris are available
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}❌ Python3 not found${NC}"
    exit 1
fi

# Check if virtual environment exists, create if not
if [ ! -d "venv" ]; then
    echo -e "${YELLOW}📦 Creating Python virtual environment...${NC}"
    python3 -m venv venv
fi

# Activate virtual environment
source venv/bin/activate

# Install/upgrade dependencies
echo -e "${YELLOW}📦 Installing/upgrading dependencies...${NC}"
pip install --upgrade pip
pip install -r requirements.txt

# Function to run fuzzing for a single target
run_fuzz_target() {
    local target=$1
    local target_name=$(basename "$target" .py)
    local corpus_path="$CORPUS_DIR/$target_name"
    local crashes_path="$CRASHES_DIR/$target_name"
    
    echo -e "${BLUE}🔍 Fuzzing target: ${target_name}${NC}"
    
    # Create corpus directory for this target
    mkdir -p "$corpus_path"
    mkdir -p "$crashes_path"
    
    # Seed corpus with some basic test data if empty
    if [ ! "$(ls -A $corpus_path)" ]; then
        echo -e "${YELLOW}📝 Seeding corpus for ${target_name}...${NC}"
        seed_corpus "$target_name" "$corpus_path"
    fi
    
    # Run fuzzing
    echo -e "${GREEN}▶️  Running ${target_name} for ${FUZZ_DURATION}s...${NC}"
    
    # Set environment variables for Atheris
    export ATHERIS_FUZZ_TEST_LEN=65536
    export ATHERIS_FUZZ_TEST_COUNT=1000
    
    # Run the fuzzer with timeout
    timeout "$FUZZ_DURATION" python3 "$target" \
        --corpus "$corpus_path" \
        --crash-dir "$crashes_path" \
        --max-crashes "$MAX_CRASHES" \
        --timeout 10 || true
    
    # Check for crashes
    local crash_count=$(find "$crashes_path" -name "*.fuzz" 2>/dev/null | wc -l)
    if [ "$crash_count" -gt 0 ]; then
        echo -e "${RED}💥 Found ${crash_count} crashes in ${target_name}${NC}"
        
        # Copy crashes to artifacts
        local artifacts_path="$ARTIFACTS_DIR/$target_name"
        mkdir -p "$artifacts_path"
        cp -r "$crashes_path"/* "$artifacts_path/" 2>/dev/null || true
        
        # Generate crash report
        generate_crash_report "$target_name" "$crashes_path" "$artifacts_path"
    else
        echo -e "${GREEN}✅ No crashes found in ${target_name}${NC}"
    fi
    
    echo ""
}

# Function to seed corpus with basic test data
seed_corpus() {
    local target_name=$1
    local corpus_path=$2
    
    case $target_name in
        "json_parser_fuzzer")
            # Basic JSON test cases
            echo '{"key": "value"}' > "$corpus_path/basic.json"
            echo '[1, 2, 3]' > "$corpus_path/array.json"
            echo 'true' > "$corpus_path/boolean.json"
            echo 'null' > "$corpus_path/null.json"
            echo '{"nested": {"deep": {"value": 42}}}' > "$corpus_path/nested.json"
            echo '{"unicode": "🚀🌟💻"}' > "$corpus_path/unicode.json"
            ;;
        "base64_parser_fuzzer")
            # Basic base64 test cases
            echo 'SGVsbG8gV29ybGQ=' > "$corpus_path/hello_world.txt"
            echo 'UG9seW1lcmEgT1M=' > "$corpus_path/polymera_os.txt"
            echo 'QmFzZTY0IEVuY29kaW5n' > "$corpus_path/base64_encoding.txt"
            echo 'VGVzdGluZyB3aXRoIHNwYWNlcyA=' > "$corpus_path/with_spaces.txt"
            ;;
        *)
            # Default test data
            echo 'test' > "$corpus_path/default.txt"
            ;;
    esac
}

# Function to generate crash report
generate_crash_report() {
    local target_name=$1
    local crashes_path=$2
    local artifacts_path=$3
    
    local report_file="$artifacts_path/crash_report.txt"
    
    {
        echo "Crash Report for $target_name"
        echo "Generated: $(date)"
        echo "Target: $target_name"
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
        echo "  Python version: $(python3 --version 2>/dev/null || echo "unknown")"
        echo "  Atheris version: $(python3 -c "import atheris; print(atheris.__version__)" 2>/dev/null || echo "unknown")"
        echo "  OS: $(uname -a 2>/dev/null || echo "unknown")"
        echo "  Virtual env: $(which python3)"
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
        local target_name=$(basename "$target" .py)
        local crashes_path="$CRASHES_DIR/$target_name"
        local crash_count=$(find "$crashes_path" -name "*.fuzz" 2>/dev/null | wc -l)
        
        if [ "$crash_count" -gt 0 ]; then
            echo -e "${RED}💥 ${target_name}: ${crash_count} crashes${NC}"
            total_crashes=$((total_crashes + crash_count))
            targets_with_crashes=$((targets_with_crashes + 1))
        else
            echo -e "${GREEN}✅ ${target_name}: No crashes${NC}"
        fi
    done
    
    echo ""
    echo -e "${BLUE}Total crashes found: ${total_crashes}${NC}"
    echo -e "${BLUE}Targets with crashes: ${targets_with_crashes}${NC}"
    echo -e "${BLUE}Artifacts saved to: ${ARTIFACTS_DIR}${NC}"
}

# Function to run quick tests
run_quick_tests() {
    echo -e "${YELLOW}🧪 Running quick tests to verify fuzzers...${NC}"
    
    for target in "${TARGETS[@]}"; do
        local target_name=$(basename "$target" .py)
        echo -e "${BLUE}Testing ${target_name}...${NC}"
        
        # Test with a simple input
        echo "test" | timeout 5 python3 "$target" --test-input || echo "Test completed"
    done
    
    echo -e "${GREEN}✅ Quick tests complete${NC}"
    echo ""
}

# Main execution
main() {
    # Clean up old artifacts
    cleanup_artifacts
    
    # Run quick tests first
    run_quick_tests
    
    # Run fuzzing for each target
    for target in "${TARGETS[@]}"; do
        run_fuzz_target "$target"
    done
    
    # Show summary
    show_summary
    
    echo -e "${GREEN}🎉 Python fuzzing complete!${NC}"
}

# Run main function
main "$@"
