#!/bin/bash

# Main Fuzz Suite Runner for Polymera OS
# Orchestrates Rust, Python, and Go fuzzing and generates comprehensive reports

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
FUZZ_DURATION=${FUZZ_DURATION:-300}  # Default 5 minutes per target
MAX_CRASHES=${MAX_CRASHES:-20}       # Maximum crashes to collect per target
TOTAL_TIMEOUT=${TOTAL_TIMEOUT:-3600} # Total timeout for entire suite (1 hour)
ARTIFACTS_DIR="artifacts"
REPORTS_DIR="reports"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Create necessary directories
mkdir -p "$ARTIFACTS_DIR" "$REPORTS_DIR"

echo -e "${PURPLE}🚀 Polymera OS Fuzz Suite Runner${NC}"
echo -e "${PURPLE}================================${NC}"
echo -e "${BLUE}Duration per target: ${FUZZ_DURATION}s${NC}"
echo -e "${BLUE}Max crashes per target: ${MAX_CRASHES}${NC}"
echo -e "${BLUE}Total timeout: ${TOTAL_TIMEOUT}s${NC}"
echo -e "${BLUE}Timestamp: ${TIMESTAMP}${NC}"
echo ""

# Function to check prerequisites
check_prerequisites() {
    echo -e "${BLUE}🔍 Checking prerequisites...${NC}"
    
    local missing_deps=()
    
    # Check for Rust
    if ! command -v cargo &> /dev/null; then
        missing_deps+=("Rust (cargo)")
    else
        echo -e "${GREEN}✅ Rust found: $(cargo --version)${NC}"
    fi
    
    # Check for Python
    if ! command -v python3 &> /dev/null; then
        missing_deps+=("Python3")
    else
        echo -e "${GREEN}✅ Python3 found: $(python3 --version)${NC}"
    fi
    
    # Check for Go
    if ! command -v go &> /dev/null; then
        missing_deps+=("Go")
    else
        echo -e "${GREEN}✅ Go found: $(go version)${NC}"
    fi
    
    # Check for timeout command
    if ! command -v timeout &> /dev/null; then
        missing_deps+=("timeout command")
    else
        echo -e "${GREEN}✅ timeout command found${NC}"
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        echo -e "${RED}❌ Missing dependencies:${NC}"
        for dep in "${missing_deps[@]}"; do
            echo -e "${RED}   - ${dep}${NC}"
        done
        echo -e "${YELLOW}Please install missing dependencies and try again.${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ All prerequisites satisfied${NC}"
    echo ""
}

# Function to run Rust fuzzing
run_rust_fuzzing() {
    echo -e "${CYAN}🦀 Running Rust Fuzzing Suite...${NC}"
    
    if [ -d "rust" ]; then
        cd rust
        
        # Set environment variables
        export FUZZ_DURATION="$FUZZ_DURATION"
        export MAX_CRASHES="$MAX_CRASHES"
        
        # Run fuzzing
        echo -e "${BLUE}Executing Rust fuzz script...${NC}"
        timeout $((FUZZ_DURATION * 6 + 300)) bash fuzz.sh || {
            echo -e "${YELLOW}⚠️  Rust fuzzing timed out or encountered issues${NC}"
        }
        
        # Copy artifacts
        if [ -d "artifacts" ]; then
            cp -r artifacts/* "../$ARTIFACTS_DIR/rust_${TIMESTAMP}/" 2>/dev/null || true
        fi
        
        cd ..
        echo -e "${GREEN}✅ Rust fuzzing complete${NC}"
    else
        echo -e "${YELLOW}⚠️  Rust fuzzing directory not found, skipping${NC}"
    fi
    
    echo ""
}

# Function to run Python fuzzing
run_python_fuzzing() {
    echo -e "${CYAN}🐍 Running Python Fuzzing Suite...${NC}"
    
    if [ -d "python" ]; then
        cd python
        
        # Set environment variables
        export FUZZ_DURATION="$FUZZ_DURATION"
        export MAX_CRASHES="$MAX_CRASHES"
        
        # Run fuzzing
        echo -e "${BLUE}Executing Python fuzz script...${NC}"
        timeout $((FUZZ_DURATION * 2 + 300)) bash fuzz.sh || {
            echo -e "${YELLOW}⚠️  Python fuzzing timed out or encountered issues${NC}"
        }
        
        # Copy artifacts
        if [ -d "artifacts" ]; then
            cp -r artifacts/* "../$ARTIFACTS_DIR/python_${TIMESTAMP}/" 2>/dev/null || true
        fi
        
        cd ..
        echo -e "${GREEN}✅ Python fuzzing complete${NC}"
    else
        echo -e "${YELLOW}⚠️  Python fuzzing directory not found, skipping${NC}"
    fi
    
    echo ""
}

# Function to run Go fuzzing
run_go_fuzzing() {
    echo -e "${CYAN}🐹 Running Go Fuzzing Suite...${NC}"
    
    if [ -d "go" ]; then
        cd go
        
        # Set environment variables
        export FUZZ_DURATION="$FUZZ_DURATION"
        export MAX_CRASHES="$MAX_CRASHES"
        
        # Initialize Go modules
        echo -e "${BLUE}Initializing Go modules...${NC}"
        go mod tidy
        go mod download
        
        # Run fuzzing for each target
        echo -e "${BLUE}Running Go fuzzers...${NC}"
        
        # JSON parser fuzzer
        if [ -f "json_parser_fuzzer.go" ]; then
            echo -e "${BLUE}Fuzzing JSON parser...${NC}"
            timeout "$FUZZ_DURATION" go-fuzz -bin=./json_parser_fuzzer-fuzz.zip -workdir=workdir -procs=1 || {
                echo -e "${YELLOW}⚠️  JSON parser fuzzing timed out or encountered issues${NC}"
            }
        fi
        
        # Copy artifacts
        if [ -d "workdir" ]; then
            mkdir -p "../$ARTIFACTS_DIR/go_${TIMESTAMP}"
            cp -r workdir/* "../$ARTIFACTS_DIR/go_${TIMESTAMP}/" 2>/dev/null || true
        fi
        
        cd ..
        echo -e "${GREEN}✅ Go fuzzing complete${NC}"
    else
        echo -e "${YELLOW}⚠️  Go fuzzing directory not found, skipping${NC}"
    fi
    
    echo ""
}

# Function to generate comprehensive report
generate_report() {
    echo -e "${BLUE}📊 Generating comprehensive fuzz report...${NC}"
    
    local report_file="$REPORTS_DIR/fuzz_report_${TIMESTAMP}.md"
    
    {
        echo "# Polymera OS Fuzz Suite Report"
        echo ""
        echo "**Generated:** $(date)"
        echo "**Duration per target:** ${FUZZ_DURATION}s"
        echo "**Max crashes per target:** ${MAX_CRASHES}"
        echo "**Total timeout:** ${TOTAL_TIMEOUT}s"
        echo ""
        echo "## Summary"
        echo ""
        
        # Count total crashes
        local total_crashes=0
        local total_targets=0
        
        for lang in rust python go; do
            local lang_artifacts="$ARTIFACTS_DIR/${lang}_${TIMESTAMP}"
            if [ -d "$lang_artifacts" ]; then
                local lang_crashes=$(find "$lang_artifacts" -name "*.fuzz" -o -name "*.txt" | wc -l)
                total_crashes=$((total_crashes + lang_crashes))
                total_targets=$((total_targets + 1))
                
                echo "### ${lang^} Fuzzing"
                echo "- **Status:** Completed"
                echo "- **Crashes found:** $lang_crashes"
                echo "- **Artifacts:** \`$lang_artifacts\`"
                echo ""
            else
                echo "### ${lang^} Fuzzing"
                echo "- **Status:** Not run or failed"
                echo "- **Crashes found:** 0"
                echo "- **Artifacts:** None"
                echo ""
            fi
        done
        
        echo "## Overall Statistics"
        echo "- **Total crashes found:** $total_crashes"
        echo "- **Languages tested:** $total_targets"
        echo "- **Success rate:** $((total_targets * 100 / 3))%"
        echo ""
        
        echo "## Recommendations"
        if [ $total_crashes -eq 0 ]; then
            echo "✅ No crashes found. The codebase appears robust against fuzzing attacks."
            echo "✅ Consider increasing fuzz duration for more thorough testing."
        else
            echo "⚠️  Crashes found. Immediate investigation recommended."
            echo "🔍 Review crash artifacts in the artifacts directory."
            echo "🛠️  Fix identified vulnerabilities before production deployment."
        fi
        
        echo ""
        echo "## Next Steps"
        echo "1. Review crash artifacts for security vulnerabilities"
        echo "2. Fix any identified issues"
        echo "3. Re-run fuzzing to verify fixes"
        echo "4. Integrate fuzzing into CI/CD pipeline"
        echo "5. Consider continuous fuzzing for ongoing security testing"
        
    } > "$report_file"
    
    echo -e "${GREEN}✅ Report generated: ${report_file}${NC}"
}

# Function to show final summary
show_final_summary() {
    echo -e "${PURPLE}🎉 Fuzz Suite Complete!${NC}"
    echo "========================"
    
    local total_crashes=0
    local languages_run=0
    
    for lang in rust python go; do
        local lang_artifacts="$ARTIFACTS_DIR/${lang}_${TIMESTAMP}"
        if [ -d "$lang_artifacts" ]; then
            local lang_crashes=$(find "$lang_artifacts" -name "*.fuzz" -o -name "*.txt" | wc -l)
            total_crashes=$((total_crashes + lang_crashes))
            languages_run=$((languages_run + 1))
            
            echo -e "${CYAN}${lang^}: ${lang_crashes} artifacts${NC}"
        fi
    done
    
    echo ""
    echo -e "${BLUE}Total artifacts collected: ${total_crashes}${NC}"
    echo -e "${BLUE}Languages tested: ${languages_run}/3${NC}"
    echo -e "${BLUE}Report generated: reports/fuzz_report_${TIMESTAMP}.md${NC}"
    echo -e "${BLUE}Artifacts saved to: ${ARTIFACTS_DIR}${NC}"
    
    if [ $total_crashes -gt 0 ]; then
        echo ""
        echo -e "${YELLOW}⚠️  Crashes or issues found. Review artifacts for security vulnerabilities.${NC}"
    else
        echo ""
        echo -e "${GREEN}✅ No crashes found. Codebase appears robust!${NC}"
    fi
}

# Function to cleanup temporary files
cleanup() {
    echo -e "${YELLOW}🧹 Cleaning up temporary files...${NC}"
    
    # Remove temporary work directories
    find . -name "workdir" -type d -exec rm -rf {} + 2>/dev/null || true
    find . -name "corpus" -type d -exec rm -rf {} + 2>/dev/null || true
    find . -name "crashes" -type d -exec rm -rf {} + 2>/dev/null || true
    
    echo -e "${GREEN}✅ Cleanup complete${NC}"
}

# Main execution
main() {
    # Check prerequisites
    check_prerequisites
    
    # Create timestamped artifact directories
    mkdir -p "$ARTIFACTS_DIR/rust_${TIMESTAMP}"
    mkdir -p "$ARTIFACTS_DIR/python_${TIMESTAMP}"
    mkdir -p "$ARTIFACTS_DIR/go_${TIMESTAMP}"
    
    # Run fuzzing suites with timeout
    timeout "$TOTAL_TIMEOUT" bash -c '
        run_rust_fuzzing
        run_python_fuzzing
        run_go_fuzzing
    ' || {
        echo -e "${YELLOW}⚠️  Fuzz suite timed out after ${TOTAL_TIMEOUT}s${NC}"
    }
    
    # Generate report
    generate_report
    
    # Show final summary
    show_final_summary
    
    # Cleanup
    cleanup
    
    echo -e "${GREEN}🎉 Fuzz suite execution complete!${NC}"
}

# Trap cleanup on exit
trap cleanup EXIT

# Run main function
main "$@"
