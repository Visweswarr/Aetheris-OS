#!/bin/bash
# Test script for SLO checker with passing metrics

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "Testing SLO checker with metrics that should PASS..."

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Run SLO checker with passing metrics
echo "Running SLO check..."
if "${SCRIPT_DIR}/check_slo" \
    --config "${SCRIPT_DIR}/slo.yaml" \
    --metrics "${SCRIPT_DIR}/test_metrics_pass.json" \
    --env development \
    --fail-on-violation \
    --format json \
    --output "${SCRIPT_DIR}/test_pass_results.json"; then
    
    echo -e "${GREEN}✓ SLO check passed as expected${NC}"
    
    # Verify output file was created
    if [[ -f "${SCRIPT_DIR}/test_pass_results.json" ]]; then
        echo -e "${GREEN}✓ Output file created${NC}"
        
        # Check that JSON is valid
        if jq empty "${SCRIPT_DIR}/test_pass_results.json" 2>/dev/null; then
            echo -e "${GREEN}✓ Output is valid JSON${NC}"
            
            # Check for expected content
            passed_count=$(jq '.passed' "${SCRIPT_DIR}/test_pass_results.json")
            failed_count=$(jq '.failed' "${SCRIPT_DIR}/test_pass_results.json")
            
            if [[ $passed_count -gt 0 ]] && [[ $failed_count -eq 0 ]]; then
                echo -e "${GREEN}✓ Expected pass/fail counts: passed=$passed_count, failed=$failed_count${NC}"
            else
                echo -e "${RED}✗ Unexpected pass/fail counts: passed=$passed_count, failed=$failed_count${NC}"
                exit 1
            fi
        else
            echo -e "${RED}✗ Output is not valid JSON${NC}"
            exit 1
        fi
        
        # Clean up
        rm -f "${SCRIPT_DIR}/test_pass_results.json"
    else
        echo -e "${RED}✗ Output file not created${NC}"
        exit 1
    fi
else
    echo -e "${RED}✗ SLO check failed unexpectedly${NC}"
    exit 1
fi

echo -e "${GREEN}All tests passed!${NC}"
