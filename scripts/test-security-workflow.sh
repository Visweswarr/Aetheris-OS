#!/bin/bash
# Test script for Security & Side-Channel CI Integration

set -euo pipefail

echo "🔒 Testing Security & Side-Channel CI Integration"
echo "=================================================="

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
check_file ".github/workflows/phase-2-security.yml" "Security workflow file"

# Check security configuration
check_file "security/allowlist.yaml" "Security allowlist configuration"

# Check documentation
check_file "docs/security/SCANS.md" "Security scanning documentation"

# Check baseline update script
check_file "tooling/analysis/update_security_baselines.py" "Security baseline update script"

# Check baseline directory
if [ -d "security/baselines" ]; then
    echo -e "${GREEN}✅ Found: Security baselines directory${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}❌ Missing: Security baselines directory${NC}"
    ((TESTS_FAILED++))
fi

echo -e "\n${YELLOW}Phase 2: Workflow Syntax Validation${NC}"
echo "----------------------------------------"

# Check if workflow file is valid YAML
run_test "YAML Syntax" "python3 -c \"import yaml; yaml.safe_load(open('.github/workflows/phase-2-security.yml'))\""

# Check if allowlist is valid YAML
run_test "Allowlist YAML Syntax" "python3 -c \"import yaml; yaml.safe_load(open('security/allowlist.yaml'))\""

echo -e "\n${YELLOW}Phase 3: Python Script Validation${NC}"
echo "----------------------------------------"

# Check Python script syntax
run_test "Baseline Update Script Syntax" "python3 -m py_compile tooling/analysis/update_security_baselines.py"

# Check if required Python modules are available
run_test "Required Python Modules" "python3 -c \"import json, os, sys, pathlib, typing, datetime, timezone, statistics, dataclasses, logging\""

echo -e "\n${YELLOW}Phase 4: Side-Channel Scanner Validation${NC}"
echo "----------------------------------------"

# Check if side-channel scanner exists
if [ -f "tooling/analysis/sidechan_scan.py" ]; then
    echo -e "${GREEN}✅ Found: Side-channel scanner${NC}"
    ((TESTS_PASSED++))
    
    # Check scanner syntax
    run_test "Side-Channel Scanner Syntax" "python3 -m py_compile tooling/analysis/sidechan_scan.py"
    
    # Check scanner help
    run_test "Side-Channel Scanner Help" "python3 tooling/analysis/sidechan_scan.py --help > /dev/null 2>&1"
else
    echo -e "${RED}❌ Missing: Side-channel scanner${NC}"
    ((TESTS_FAILED++))
fi

echo -e "\n${YELLOW}Phase 5: Integration Validation${NC}"
echo "----------------------------------------"

# Check if security workflow integrates with main matrix
if grep -q "phase-2-security" .github/workflows/phase-2-matrix.yml 2>/dev/null; then
    echo -e "${GREEN}✅ Found: Security workflow integration in main matrix${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${YELLOW}⚠️  Note: Security workflow is standalone (this is acceptable)${NC}"
fi

# Check if security directories exist in project structure
if [ -d "kernel" ] && [ -d "crypto" ] && [ -d "services" ] && [ -d "tooling" ]; then
    echo -e "${GREEN}✅ Found: All required scopes for security analysis${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}❌ Missing: Some required scopes for security analysis${NC}"
    ((TESTS_FAILED++))
fi

echo -e "\n${YELLOW}Phase 6: Configuration Validation${NC}"
echo "----------------------------------------"

# Check allowlist configuration structure
if python3 -c "
import yaml
with open('security/allowlist.yaml') as f:
    config = yaml.safe_load(f)
    assert 'allowlist' in config
    assert 'settings' in config
    print('Allowlist structure is valid')
" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Allowlist configuration structure is valid${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}❌ Allowlist configuration structure is invalid${NC}"
    ((TESTS_FAILED++))
fi

echo -e "\n${YELLOW}Test Summary${NC}"
echo "============="
echo -e "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All security workflow tests passed!${NC}"
    echo -e "${BLUE}The Security & Side-Channel CI Integration is ready for use.${NC}"
    exit 0
else
    echo -e "\n${RED}❌ Some tests failed. Please review the issues above.${NC}"
    exit 1
fi
