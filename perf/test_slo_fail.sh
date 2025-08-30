#!/bin/bash
# Test script for SLO checker with failing metrics

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Testing SLO checker with metrics that should FAIL..."

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Test 1: Run SLO checker with failing metrics and expect failure
echo "Test 1: Running SLO check with --fail-on-violation (should fail)..."
if "${SCRIPT_DIR}/check_slo" \
    --config "${SCRIPT_DIR}/slo.yaml" \
    --metrics "${SCRIPT_DIR}/test_metrics_fail.json" \
    --env production \
    --fail-on-violation \
    --format json \
    --output "${SCRIPT_DIR}/test_fail_results.json" 2>/dev/null; then
    
    echo -e "${RED}✗ SLO check passed unexpectedly${NC}"
    exit 1
else
    echo -e "${GREEN}✓ SLO check failed as expected (exit code: $?)${NC}"
    
    # Verify output file was created even on failure
    if [[ -f "${SCRIPT_DIR}/test_fail_results.json" ]]; then
        echo -e "${GREEN}✓ Output file created${NC}"
        
        # Check that JSON is valid
        if jq empty "${SCRIPT_DIR}/test_fail_results.json" 2>/dev/null; then
            echo -e "${GREEN}✓ Output is valid JSON${NC}"
            
            # Check for expected content
            passed_count=$(jq '.passed' "${SCRIPT_DIR}/test_fail_results.json")
            failed_count=$(jq '.failed' "${SCRIPT_DIR}/test_fail_results.json")
            
            if [[ $failed_count -gt 0 ]]; then
                echo -e "${GREEN}✓ Expected failure count: failed=$failed_count${NC}"
                
                # Check specific SLO failures
                identity_failed=$(jq -r '.results[] | select(.slo_name == "identity_issuance") | .status' "${SCRIPT_DIR}/test_fail_results.json")
                xr_failed=$(jq -r '.results[] | select(.slo_name == "xr_mean_time_to_present") | .status' "${SCRIPT_DIR}/test_fail_results.json")
                
                if [[ "$identity_failed" == "Fail" ]]; then
                    echo -e "${GREEN}✓ Identity issuance SLO failed as expected${NC}"
                else
                    echo -e "${YELLOW}⚠ Identity issuance SLO status: $identity_failed${NC}"
                fi
                
                if [[ "$xr_failed" == "Fail" ]]; then
                    echo -e "${GREEN}✓ XR MTP SLO failed as expected${NC}"
                else
                    echo -e "${YELLOW}⚠ XR MTP SLO status: $xr_failed${NC}"
                fi
            else
                echo -e "${RED}✗ Expected failures but got: passed=$passed_count, failed=$failed_count${NC}"
                exit 1
            fi
        else
            echo -e "${RED}✗ Output is not valid JSON${NC}"
            exit 1
        fi
    else
        echo -e "${RED}✗ Output file not created${NC}"
        exit 1
    fi
fi

# Test 2: Run SLO checker without --fail-on-violation (should succeed but report failures)
echo ""
echo "Test 2: Running SLO check without --fail-on-violation (should succeed)..."
if "${SCRIPT_DIR}/check_slo" \
    --config "${SCRIPT_DIR}/slo.yaml" \
    --metrics "${SCRIPT_DIR}/test_metrics_fail.json" \
    --env production \
    --format json \
    --output "${SCRIPT_DIR}/test_warn_results.json"; then
    
    echo -e "${GREEN}✓ SLO check succeeded (warnings only)${NC}"
    
    # Verify warnings were generated
    if [[ -f "${SCRIPT_DIR}/test_warn_results.json" ]]; then
        failed_count=$(jq '.failed' "${SCRIPT_DIR}/test_warn_results.json")
        if [[ $failed_count -gt 0 ]]; then
            echo -e "${GREEN}✓ Failures reported but process succeeded: failed=$failed_count${NC}"
        else
            echo -e "${RED}✗ Expected failures to be reported${NC}"
            exit 1
        fi
    fi
else
    echo -e "${RED}✗ SLO check failed unexpectedly in warning mode${NC}"
    exit 1
fi

# Test 3: Test tolerance mode
echo ""
echo "Test 3: Running SLO check with tolerance mode..."
if "${SCRIPT_DIR}/check_slo" \
    --config "${SCRIPT_DIR}/slo.yaml" \
    --metrics "${SCRIPT_DIR}/test_metrics_fail.json" \
    --env production \
    --tolerance \
    --format json \
    --output "${SCRIPT_DIR}/test_tolerance_results.json"; then
    
    echo -e "${GREEN}✓ SLO check with tolerance completed${NC}"
    
    # Check if tolerance reduced failures
    if [[ -f "${SCRIPT_DIR}/test_tolerance_results.json" ]]; then
        tolerance_failed=$(jq '.failed' "${SCRIPT_DIR}/test_tolerance_results.json")
        tolerance_passed=$(jq '.passed' "${SCRIPT_DIR}/test_tolerance_results.json")
        echo -e "${GREEN}✓ With tolerance: passed=$tolerance_passed, failed=$tolerance_failed${NC}"
        
        # Tolerance should help some SLOs pass
        if [[ $tolerance_passed -gt 0 ]]; then
            echo -e "${GREEN}✓ Tolerance mode helped some SLOs pass${NC}"
        fi
    fi
else
    echo -e "${RED}✗ SLO check with tolerance failed${NC}"
    exit 1
fi

# Test 4: Test HTML output format
echo ""
echo "Test 4: Testing HTML output format..."
if "${SCRIPT_DIR}/check_slo" \
    --config "${SCRIPT_DIR}/slo.yaml" \
    --metrics "${SCRIPT_DIR}/test_metrics_fail.json" \
    --env production \
    --format html \
    --output "${SCRIPT_DIR}/test_report.html"; then
    
    echo -e "${GREEN}✓ HTML report generated${NC}"
    
    # Verify HTML file was created and contains expected content
    if [[ -f "${SCRIPT_DIR}/test_report.html" ]]; then
        if grep -q "SLO Check Report" "${SCRIPT_DIR}/test_report.html"; then
            echo -e "${GREEN}✓ HTML report contains expected content${NC}"
        else
            echo -e "${RED}✗ HTML report missing expected content${NC}"
            exit 1
        fi
    else
        echo -e "${RED}✗ HTML report file not created${NC}"
        exit 1
    fi
else
    echo -e "${RED}✗ HTML report generation failed${NC}"
    exit 1
fi

# Clean up test files
echo ""
echo "Cleaning up test files..."
rm -f "${SCRIPT_DIR}/test_fail_results.json"
rm -f "${SCRIPT_DIR}/test_warn_results.json"
rm -f "${SCRIPT_DIR}/test_tolerance_results.json"
rm -f "${SCRIPT_DIR}/test_report.html"

echo -e "${GREEN}All failure tests passed!${NC}"
