#!/bin/bash

# Determinism Test Runner Script for Polymera OS
# Runs determinism tests and generates comprehensive reports

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CASES_DIR="$SCRIPT_DIR/cases"
OUTPUT_DIR="$SCRIPT_DIR/output"
GOLDEN_DIR="$SCRIPT_DIR/golden"
NUM_RUNS=3
TIMEOUT=300

# Create necessary directories
mkdir -p "$OUTPUT_DIR" "$GOLDEN_DIR"

echo -e "${PURPLE}🔒 Polymera OS Determinism Test Runner${NC}"
echo -e "${PURPLE}========================================${NC}"
echo -e "${BLUE}Cases directory: ${CASES_DIR}${NC}"
echo -e "${BLUE}Output directory: ${OUTPUT_DIR}${NC}"
echo -e "${BLUE}Golden directory: ${GOLDEN_DIR}${NC}"
echo -e "${BLUE}Number of runs: ${NUM_RUNS}${NC}"
echo ""

# Function to check prerequisites
check_prerequisites() {
    echo -e "${BLUE}🔍 Checking prerequisites...${NC}"
    
    # Check if Rust is available
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Cargo not found. Please install Rust.${NC}"
        exit 1
    fi
    
    # Check if determinism runner is built
    if [ ! -f "$SCRIPT_DIR/target/release/determinism-runner" ] && [ ! -f "$SCRIPT_DIR/target/debug/determinism-runner" ]; then
        echo -e "${YELLOW}⚠️  Determinism runner not built. Building...${NC}"
        build_runner
    fi
    
    echo -e "${GREEN}✅ Prerequisites satisfied${NC}"
    echo ""
}

# Function to build the determinism runner
build_runner() {
    echo -e "${BLUE}🔨 Building determinism runner...${NC}"
    
    cd "$SCRIPT_DIR"
    
    # Try release build first
    if cargo build --release; then
        echo -e "${GREEN}✅ Release build successful${NC}"
    else
        echo -e "${YELLOW}⚠️  Release build failed, trying debug build...${NC}"
        if cargo build; then
            echo -e "${GREEN}✅ Debug build successful${NC}"
        else
            echo -e "${RED}❌ Build failed${NC}"
            exit 1
        fi
    fi
    
    cd - > /dev/null
}

# Function to get runner path
get_runner_path() {
    if [ -f "$SCRIPT_DIR/target/release/determinism-runner" ]; then
        echo "$SCRIPT_DIR/target/release/determinism-runner"
    elif [ -f "$SCRIPT_DIR/target/debug/determinism-runner" ]; then
        echo "$SCRIPT_DIR/target/debug/determinism-runner"
    else
        echo ""
    fi
}

# Function to run determinism tests
run_determinism_tests() {
    local runner_path=$(get_runner_path)
    
    if [ -z "$runner_path" ]; then
        echo -e "${RED}❌ Determinism runner not found${NC}"
        exit 1
    fi
    
    echo -e "${CYAN}🚀 Running determinism tests...${NC}"
    echo ""
    
    # Run all tests
    "$runner_path" \
        --cases-dir "$CASES_DIR" \
        --output-dir "$OUTPUT_DIR" \
        --golden-dir "$GOLDEN_DIR" \
        --num-runs "$NUM_RUNS" \
        --seed-strategy fixed \
        --fixed-seed 42 \
        run
    
    echo ""
    echo -e "${GREEN}✅ Determinism tests completed${NC}"
}

# Function to run individual test case
run_single_test() {
    local test_id=$1
    local runner_path=$(get_runner_path)
    
    if [ -z "$runner_path" ]; then
        echo -e "${RED}❌ Determinism runner not found${NC}"
        exit 1
    fi
    
    echo -e "${CYAN}🔍 Running test case: ${test_id}${NC}"
    echo ""
    
    "$runner_path" \
        --cases-dir "$CASES_DIR" \
        --output-dir "$OUTPUT_DIR" \
        --golden-dir "$GOLDEN_DIR" \
        --num-runs "$NUM_RUNS" \
        --seed-strategy fixed \
        --fixed-seed 42 \
        test "$test_id"
}

# Function to list available test cases
list_test_cases() {
    local runner_path=$(get_runner_path)
    
    if [ -z "$runner_path" ]; then
        echo -e "${RED}❌ Determinism runner not found${NC}"
        exit 1
    fi
    
    echo -e "${CYAN}📋 Available test cases:${NC}"
    echo ""
    
    "$runner_path" \
        --cases-dir "$CASES_DIR" \
        --output-dir "$OUTPUT_DIR" \
        --golden-dir "$GOLDEN_DIR" \
        list
}

# Function to generate report
generate_report() {
    local runner_path=$(get_runner_path)
    
    if [ -z "$runner_path" ]; then
        echo -e "${RED}❌ Determinism runner not found${NC}"
        exit 1
    fi
    
    echo -e "${CYAN}📊 Generating determinism report...${NC}"
    echo ""
    
    "$runner_path" \
        --cases-dir "$CASES_DIR" \
        --output-dir "$OUTPUT_DIR" \
        --golden-dir "$GOLDEN_DIR" \
        report
    
    echo ""
    echo -e "${GREEN}✅ Report generated${NC}"
}

# Function to show test results summary
show_results_summary() {
    echo -e "${BLUE}📊 Test Results Summary${NC}"
    echo "====================="
    
    if [ ! -d "$OUTPUT_DIR" ]; then
        echo -e "${YELLOW}No output directory found${NC}"
        return
    fi
    
    # Count test results
    local total_results=$(find "$OUTPUT_DIR" -name "*.json" | wc -l)
    local total_tests=$(find "$OUTPUT_DIR" -name "*.json" | sed 's/.*_\([0-9]*\)_.*\.json/\1/' | sort -u | wc -l)
    
    echo -e "Total results: ${total_results}"
    echo -e "Total test cases: ${total_tests}"
    
    # Check for determinism violations
    local violations=0
    for test_dir in "$OUTPUT_DIR"/*; do
        if [ -d "$test_dir" ]; then
            local test_id=$(basename "$test_dir")
            local result_files=$(find "$test_dir" -name "*.json" | wc -l)
            
            if [ $result_files -gt 1 ]; then
                # Check if all results have the same hash
                local hashes=$(find "$test_dir" -name "*.json" -exec grep -o '"output_hash":"[^"]*"' {} \; | cut -d'"' -f4 | sort -u | wc -l)
                
                if [ $hashes -gt 1 ]; then
                    echo -e "${RED}❌ ${test_id}: NON-DETERMINISTIC (${hashes} different hashes)${NC}"
                    violations=$((violations + 1))
                else
                    echo -e "${GREEN}✅ ${test_id}: Deterministic${NC}"
                fi
            fi
        fi
    done
    
    echo ""
    if [ $violations -eq 0 ]; then
        echo -e "${GREEN}🎉 All tests are deterministic!${NC}"
    else
        echo -e "${RED}⚠️  ${violations} test(s) show non-deterministic behavior${NC}"
    fi
}

# Function to create sample test data
create_sample_data() {
    echo -e "${BLUE}📝 Creating sample test data...${NC}"
    
    # Create test data directory
    local test_data_dir="$SCRIPT_DIR/test_data"
    mkdir -p "$test_data_dir"
    
    # Create sample capability token
    cat > "$test_data_dir/sample_token.json" << 'EOF'
{
  "header": {
    "typ": "capability",
    "alg": "Dilithium3",
    "ver": "1.0.0",
    "kid": "test-key-123",
    "jti": "test-token-456",
    "iss": "test.example.com",
    "sub": "test-user-789",
    "aud": "test-app.example.com",
    "iat": 1234567890,
    "nbf": 1234567890,
    "exp": 1234567890,
    "additional": {}
  },
  "payload": {
    "purpose": "test",
    "claims": [
      {
        "claim_type": "Resource",
        "data": {
          "resource_type": "file",
          "resource_id": "test-file-123",
          "path": "/test/path/file.txt",
          "permissions": ["read"]
        }
      }
    ],
    "scope": "test-scope",
    "level": 1,
    "hierarchy": ["test", "determinism"],
    "constraints": {},
    "metadata": {},
    "additional": {}
  },
  "signature": null,
  "format_version": "1.0.0"
}
EOF
    
    # Create complex JSON for parser testing
    cat > "$test_data_dir/complex_json.json" << 'EOF'
{
  "string": "Hello, World!",
  "number": 42,
  "boolean": true,
  "null": null,
  "array": [1, 2, 3, "four", false, null],
  "object": {
    "nested": {
      "deep": {
        "value": "nested value",
        "numbers": [1, 2, 3, 4, 5],
        "mixed": [true, "string", 123, null]
      }
    }
  },
  "unicode": "🚀🌟💻🔒",
  "special_chars": "!@#$%^&*()_+-=[]{}|;':\",./<>?",
  "large_number": 999999999999999999,
  "small_number": 0.000000000000000001
}
EOF
    
    # Create binary data for base64 testing
    dd if=/dev/urandom of="$test_data_dir/binary_data.bin" bs=1024 count=10 2>/dev/null || {
        # Fallback: create deterministic binary data
        python3 -c "
import struct
with open('$test_data_dir/binary_data.bin', 'wb') as f:
    for i in range(10240):
        f.write(struct.pack('B', i % 256))
" 2>/dev/null || {
            # Simple fallback
            echo "Creating simple binary data..."
            for i in {0..10239}; do
                printf "\\$(printf '%03o' $((i % 256)))" >> "$test_data_dir/binary_data.bin"
            done
        }
    }
    
    echo -e "${GREEN}✅ Sample test data created${NC}"
}

# Function to show help
show_help() {
    echo "Usage: $0 [OPTION] [COMMAND]"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -n, --runs     Number of test runs (default: $NUM_RUNS)"
    echo "  -t, --timeout  Timeout per test in seconds (default: $TIMEOUT)"
    echo ""
    echo "Commands:"
    echo "  run            Run all determinism tests (default)"
    echo "  test ID        Run specific test case"
    echo "  list           List available test cases"
    echo "  report         Generate report from existing results"
    echo "  setup          Create sample test data"
    echo "  summary        Show test results summary"
    echo ""
    echo "Examples:"
    echo "  $0                    # Run all tests"
    echo "  $0 test capability_token_format  # Run specific test"
    echo "  $0 -n 5 run          # Run all tests with 5 iterations"
    echo "  $0 setup             # Create sample test data"
}

# Main execution
main() {
    local command="run"
    local test_id=""
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -n|--runs)
                NUM_RUNS="$2"
                shift 2
                ;;
            -t|--timeout)
                TIMEOUT="$2"
                shift 2
                ;;
            run|test|list|report|setup|summary)
                command="$1"
                if [ "$1" = "test" ]; then
                    test_id="$2"
                    shift
                fi
                shift
                ;;
            *)
                echo -e "${RED}Unknown option: $1${NC}"
                show_help
                exit 1
                ;;
        esac
    done
    
    # Check prerequisites
    check_prerequisites
    
    # Execute command
    case $command in
        "run")
            run_determinism_tests
            ;;
        "test")
            if [ -z "$test_id" ]; then
                echo -e "${RED}Test ID required for 'test' command${NC}"
                exit 1
            fi
            run_single_test "$test_id"
            ;;
        "list")
            list_test_cases
            ;;
        "report")
            generate_report
            ;;
        "setup")
            create_sample_data
            ;;
        "summary")
            show_results_summary
            ;;
        *)
            echo -e "${RED}Unknown command: $command${NC}"
            show_help
            exit 1
            ;;
    esac
    
    echo ""
    echo -e "${GREEN}🎉 Determinism test execution complete!${NC}"
}

# Run main function
main "$@"

