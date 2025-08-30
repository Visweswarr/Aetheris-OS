#!/bin/bash
# Test script for Performance Dashboard Generation

set -euo pipefail

echo "📊 Testing Performance Dashboard Generation"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0

# Helper function to run tests
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "\n${BLUE}Testing: ${test_name}${NC}"
    echo "Command: $test_command"
    
    if eval "$test_command" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ PASSED${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}❌ FAILED${NC}"
        ((TESTS_FAILED++))
    fi
}

# Helper function to check file existence
check_file() {
    local file_path="$1"
    local description="$2"
    
    if [ -f "$file_path" ]; then
        echo -e "${GREEN}✅ Found: $description${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}❌ Missing: $description${NC}"
        ((TESTS_FAILED++))
    fi
}

echo -e "\n${YELLOW}Phase 1: File Structure Validation${NC}"
echo "----------------------------------------"

# Check workflow file
check_file ".github/workflows/perf-dashboard.yml" "Performance dashboard workflow"

# Check dashboard renderer script
check_file "tooling/perf/render_dashboard.js" "Dashboard renderer script"

# Check performance history directory
if [ -d "perf/history" ]; then
    echo -e "${GREEN}✅ Found: Performance history directory${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}❌ Missing: Performance history directory${NC}"
    ((TESTS_FAILED++))
fi

# Check sample history file
check_file "perf/history/sample_config.json" "Sample performance history file"

echo -e "\n${YELLOW}Phase 2: Script Validation${NC}"
echo "----------------------------------------"

# Check if Node.js script syntax is valid
run_test "Dashboard Script Syntax" "node -c tooling/perf/render_dashboard.js"

# Check if script has help option
run_test "Dashboard Script Help" "node tooling/perf/render_dashboard.js --help > /dev/null 2>&1"

echo -e "\n${YELLOW}Phase 3: Dashboard Generation Test${NC}"
echo "----------------------------------------"

# Create test output directory
TEST_OUTPUT_DIR="test_dashboard_output"
if [ -d "$TEST_OUTPUT_DIR" ]; then
    rm -rf "$TEST_OUTPUT_DIR"
fi

# Test dashboard generation with sample data
run_test "Dashboard Generation" "node tooling/perf/render_dashboard.js --input perf/history --output $TEST_OUTPUT_DIR --baseline perf/baselines/p2.json"

# Check generated files
if [ -d "$TEST_OUTPUT_DIR" ]; then
    echo -e "\n${BLUE}Checking generated dashboard files...${NC}"
    
    check_file "$TEST_OUTPUT_DIR/index.html" "Dashboard HTML file"
    check_file "$TEST_OUTPUT_DIR/style.css" "Dashboard CSS file"
    check_file "$TEST_OUTPUT_DIR/dashboard.js" "Dashboard JavaScript file"
    check_file "$TEST_OUTPUT_DIR/README.md" "Dashboard README"
    
    # Check data directory
    if [ -d "$TEST_OUTPUT_DIR/data" ]; then
        echo -e "${GREEN}✅ Found: Dashboard data directory${NC}"
        ((TESTS_PASSED++))
        
        # Check data files
        check_file "$TEST_OUTPUT_DIR/data/index.json" "Dashboard data index"
        check_file "$TEST_OUTPUT_DIR/data/sample_config.json" "Sample config data shard"
    else
        echo -e "${RED}❌ Missing: Dashboard data directory${NC}"
        ((TESTS_FAILED++))
    fi
else
    echo -e "${RED}❌ Dashboard generation failed - output directory not created${NC}"
    ((TESTS_FAILED++))
fi

echo -e "\n${YELLOW}Phase 4: Generated Content Validation${NC}"
echo "----------------------------------------"

# Check HTML content
if [ -f "$TEST_OUTPUT_DIR/index.html" ]; then
    if grep -q "Polymera OS Performance Dashboard" "$TEST_OUTPUT_DIR/index.html"; then
        echo -e "${GREEN}✅ Dashboard HTML content is valid${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}❌ Dashboard HTML content is invalid${NC}"
        ((TESTS_FAILED++))
    fi
fi

# Check CSS content
if [ -f "$TEST_OUTPUT_DIR/style.css" ]; then
    if grep -q "Performance Dashboard Styles" "$TEST_OUTPUT_DIR/style.css"; then
        echo -e "${GREEN}✅ Dashboard CSS content is valid${NC}"
        ((TESTS_FAILED++))
    else
        echo -e "${RED}❌ Dashboard CSS content is invalid${NC}"
        ((TESTS_FAILED++))
    fi
fi

# Check JavaScript content
if [ -f "$TEST_OUTPUT_DIR/dashboard.js" ]; then
    if grep -q "Performance Dashboard JavaScript" "$TEST_OUTPUT_DIR/dashboard.js"; then
        echo -e "${GREEN}✅ Dashboard JavaScript content is valid${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}❌ Dashboard JavaScript content is invalid${NC}"
        ((TESTS_FAILED++))
    fi
fi

# Check data index content
if [ -f "$TEST_OUTPUT_DIR/data/index.json" ]; then
    if node -e "
        const fs = require('fs');
        const data = JSON.parse(fs.readFileSync('$TEST_OUTPUT_DIR/data/index.json', 'utf8'));
        if (data.generated_at && data.total_configurations !== undefined) {
            console.log('✅ Dashboard data index is valid');
        } else {
            console.error('❌ Dashboard data index is invalid');
            process.exit(1);
        }
    " > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Dashboard data index is valid${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}❌ Dashboard data index is invalid${NC}"
        ((TESTS_FAILED++))
    fi
fi

echo -e "\n${YELLOW}Phase 5: Cleanup${NC}"
echo "----------------------------------------"

# Clean up test files
if [ -d "$TEST_OUTPUT_DIR" ]; then
    rm -rf "$TEST_OUTPUT_DIR"
    echo -e "${GREEN}✅ Test files cleaned up${NC}"
    ((TESTS_PASSED++))
fi

# Test Summary
echo -e "\n${YELLOW}Test Summary${NC}"
echo "============="
echo -e "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All performance dashboard tests passed!${NC}"
    echo -e "${BLUE}The Performance Dashboard Generation is ready for use.${NC}"
    exit 0
else
    echo -e "\n${RED}❌ Some tests failed. Please review the issues above.${NC}"
    exit 1
fi

