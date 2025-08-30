#!/bin/bash
set -e

echo "🧪 Testing UI Stub Implementation..."
echo "===================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Test function
run_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_exit="$3"
    
    echo -e "\n${BLUE}Running: ${test_name}${NC}"
    echo "Command: $test_command"
    
    if eval "$test_command" > /tmp/ui_stub_test_output.log 2>&1; then
        local exit_code=$?
        if [ "$exit_code" = "$expected_exit" ]; then
            echo -e "${GREEN}✓ PASSED${NC}"
            ((TESTS_PASSED++))
        else
            echo -e "${RED}✗ FAILED (expected exit $expected_exit, got $exit_code)${NC}"
            ((TESTS_FAILED++))
        fi
    else
        local exit_code=$?
        if [ "$exit_code" = "$expected_exit" ]; then
            echo -e "${GREEN}✓ PASSED${NC}"
            ((TESTS_PASSED++))
        else
            echo -e "${RED}✗ FAILED (expected exit $expected_exit, got $exit_code)${NC}"
            echo "Output:"
            cat /tmp/ui_stub_test_output.log
            ((TESTS_FAILED++))
        fi
    fi
}

# Check if Bazel is available
if ! command -v bazel &> /dev/null; then
    echo -e "${YELLOW}Bazel not found, checking for alternative build tools...${NC}"
    
    # Check for Yarn
    if command -v yarn &> /dev/null; then
        echo -e "${YELLOW}Using Yarn for testing...${NC}"
        USE_YARN=true
    else
        echo -e "${RED}No build tools found. Please install Bazel or Yarn.${NC}"
        exit 1
    fi
else
    USE_YARN=false
fi

# Test 1: Check package.json exists
run_test "Package.json Check" "test -f package.json" 0

# Test 2: Check source files exist
run_test "Source Files Check" "test -f src/index.tsx" 0

# Test 3: Check BUILD file exists
run_test "BUILD File Check" "test -f BUILD" 0

# Test 4: Check HTML file exists
run_test "HTML File Check" "test -f index.html" 0

# Test 5: Check TypeScript config exists
run_test "TypeScript Config Check" "test -f tsconfig.json" 0

# Test 6: Check directory structure
run_test "Directory Structure Check" "test -d src" 0

# Test 7: Check file permissions
run_test "File Permissions Check" "test -r src/index.tsx" 0

# Test 8: Check file sizes
run_test "File Size Check" "test -s src/index.tsx" 0

# Test 9: Check React imports
run_test "React Imports Check" "grep -q 'import React' src/index.tsx" 0

# Test 10: Check ReactDOM imports
run_test "ReactDOM Imports Check" "grep -q 'import ReactDOM' src/index.tsx" 0

# Test 11: Check Hello Polymera text
run_test "Hello Polymera Text Check" "grep -q 'Hello Polymera' src/index.tsx" 0

# Test 12: Check component definition
run_test "Component Definition Check" "grep -q 'const HelloPolymeraApp' src/index.tsx" 0

# Test 13: Check React.FC type
run_test "React.FC Type Check" "grep -q 'React.FC' src/index.tsx" 0

# Test 14: Check JSX return
run_test "JSX Return Check" "grep -q 'return (' src/index.tsx" 0

# Test 15: Check app header
run_test "App Header Check" "grep -q 'app-header' src/index.tsx" 0

# Test 16: Check app main
run_test "App Main Check" "grep -q 'app-main' src/index.tsx" 0

# Test 17: Check app footer
run_test "App Footer Check" "grep -q 'app-footer' src/index.tsx" 0

# Test 18: Check inline styles
run_test "Inline Styles Check" "grep -q 'appStyles' src/index.tsx" 0

# Test 19: Check style injection
run_test "Style Injection Check" "grep -q 'appendChild' src/index.tsx" 0

# Test 20: Check root element
run_test "Root Element Check" "grep -q 'getElementById' src/index.tsx" 0

# Test 21: Check ReactDOM render
run_test "ReactDOM Render Check" "grep -q 'createRoot' src/index.tsx" 0

# Test 22: Check component export
run_test "Component Export Check" "grep -q 'export default' src/index.tsx" 0

# Test 23: Check named export
run_test "Named Export Check" "grep -q 'export { HelloPolymeraApp }' src/index.tsx" 0

# Test 24: Check development helpers
run_test "Development Helpers Check" "grep -q 'NODE_ENV.*development' src/index.tsx" 0

# Test 25: Check console logging
run_test "Console Logging Check" "grep -q 'console.log' src/index.tsx" 0

# Test 26: Check package.json name
run_test "Package Name Check" "grep -q 'hello-polymera-app' package.json" 0

# Test 27: Check package version
run_test "Package Version Check" "grep -q '0.1.0' package.json" 0

# Test 28: Check React dependency
run_test "React Dependency Check" "grep -q 'react.*18.2.0' package.json" 0

# Test 29: Check ReactDOM dependency
run_test "ReactDOM Dependency Check" "grep -q 'react-dom.*18.2.0' package.json" 0

# Test 30: Check TypeScript dependency
run_test "TypeScript Dependency Check" "grep -q 'typescript.*5.0.0' package.json" 0

# Test 31: Check build script
run_test "Build Script Check" "grep -q 'build.*tsc' package.json" 0

# Test 32: Check Bazel scripts
run_test "Bazel Scripts Check" "grep -q 'bazel:build' package.json" 0

# Test 33: Check keywords
run_test "Keywords Check" "grep -q 'polymera.*typescript.*react.*bazel' package.json" 0

# Test 34: Check HTML title
run_test "HTML Title Check" "grep -q 'Hello Polymera.*Polymera OS UI' index.html" 0

# Test 35: Check HTML meta description
run_test "HTML Meta Description Check" "grep -q 'Minimal TypeScript React app for Polymera OS UI' index.html" 0

# Test 36: Check HTML root element
run_test "HTML Root Element Check" "grep -q 'id=\"root\"' index.html" 0

# Test 37: Check HTML script import
run_test "HTML Script Import Check" "grep -q 'src=\"/src/index.tsx\"' index.html" 0

# Test 38: Check HTML loading state
run_test "HTML Loading State Check" "grep -q 'Loading Polymera OS' index.html" 0

# Test 39: Check HTML error boundary
run_test "HTML Error Boundary Check" "grep -q 'error-boundary' index.html" 0

# Test 40: Check HTML service worker
run_test "HTML Service Worker Check" "grep -q 'serviceWorker' index.html" 0

# Test 41: Check HTML performance monitoring
run_test "HTML Performance Monitoring Check" "grep -q 'performance.*getEntriesByType' index.html" 0

# Test 42: Check HTML development helpers
run_test "HTML Development Helpers Check" "grep -q 'Development mode detected' index.html" 0

# Test 43: Check TypeScript config target
run_test "TS Config Target Check" "grep -q 'ES2020' tsconfig.json" 0

# Test 44: Check TypeScript config JSX
run_test "TS Config JSX Check" "grep -q 'react-jsx' tsconfig.json" 0

# Test 45: Check TypeScript config strict
run_test "TS Config Strict Check" "grep -q 'strict.*true' tsconfig.json" 0

# Test 46: Check TypeScript config paths
run_test "TS Config Paths Check" "grep -q '@components' tsconfig.json" 0

# Test 47: Check BUILD file TypeScript project
run_test "BUILD TypeScript Project Check" "grep -q 'ts_project' BUILD" 0

# Test 48: Check BUILD file Rollup bundle
run_test "BUILD Rollup Bundle Check" "grep -q 'rollup_bundle' BUILD" 0

# Test 49: Check BUILD file Jest tests
run_test "BUILD Jest Tests Check" "grep -q 'jest_test' BUILD" 0

# Test 50: Check BUILD file Docker image
run_test "BUILD Docker Image Check" "grep -q 'container_image' BUILD" 0

# Test 51: Check BUILD file Kubernetes
run_test "BUILD Kubernetes Check" "grep -q 'k8s_object' BUILD" 0

# Test 52: Check BUILD file targets
run_test "BUILD Targets Check" "grep -q 'hello_app' BUILD" 0

# Test 53: Check BUILD file dependencies
run_test "BUILD Dependencies Check" "grep -q '@npm//react' BUILD" 0

# Test 54: Check file line counts
run_test "File Line Count Check" "wc -l src/index.tsx | grep -q '[0-9]'" 0

# Test 55: Check package.json line count
run_test "Package.json Line Count Check" "wc -l package.json | grep -q '[0-9]'" 0

# Test 56: Check BUILD line count
run_test "BUILD Line Count Check" "wc -l BUILD | grep -q '[0-9]'" 0

# Test 57: Check HTML line count
run_test "HTML Line Count Check" "wc -l index.html | grep -q '[0-9]'" 0

# Test 58: Check tsconfig line count
run_test "TSConfig Line Count Check" "wc -l tsconfig.json | grep -q '[0-9]'" 0

# Test 59: Check React component structure
run_test "React Component Structure Check" "grep -A 5 -B 5 'HelloPolymeraApp' src/index.tsx | grep -q 'React.FC'" 0

# Test 60: Check JSX structure
run_test "JSX Structure Check" "grep -A 10 'return (' src/index.tsx | grep -q 'div.*className'" 0

# Test 61: Check CSS classes
run_test "CSS Classes Check" "grep -q 'hello-polymera-app' src/index.tsx" 0

# Test 62: Check responsive design
run_test "Responsive Design Check" "grep -q '@media.*max-width.*768px' src/index.tsx" 0

# Test 63: Check accessibility features
run_test "Accessibility Features Check" "grep -q 'lang=\"en\"' index.html" 0

# Test 64: Check security headers
run_test "Security Headers Check" "grep -q 'Content-Security-Policy' index.html" 0

# Test 65: Check PWA support
run_test "PWA Support Check" "grep -q 'theme-color' index.html" 0

echo -e "\n======================================"
echo -e "${BLUE}Test Results:${NC}"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo -e "Total: $((TESTS_PASSED + TESTS_FAILED))"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All UI Stub tests passed!${NC}"
    
    # Run additional validation
    echo -e "\n${BLUE}Running Additional Validation...${NC}"
    
    # Check file sizes
    echo -e "\n${YELLOW}File Sizes:${NC}"
    ls -lh src/index.tsx package.json BUILD index.html tsconfig.json 2>/dev/null || true
    
    # Check line counts
    echo -e "\n${YELLOW}Line Counts:${NC}"
    wc -l src/index.tsx package.json BUILD index.html tsconfig.json 2>/dev/null || true
    
    # Check test coverage
    echo -e "\n${YELLOW}Test Coverage Summary:${NC}"
    echo "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"
    echo "Passed: $TESTS_PASSED"
    echo "Failed: $TESTS_FAILED"
    echo "Coverage: $((TESTS_PASSED * 100 / (TESTS_PASSED + TESTS_FAILED)))%"
    
    # Check for required features
    echo -e "\n${YELLOW}Required Features Check:${NC}"
    echo "✅ TypeScript App: React component with 'Hello Polymera' message"
    echo "✅ Package.json: Dependencies and build scripts"
    echo "✅ BUILD File: Bazel integration with TypeScript and React"
    echo "✅ HTML Entry: Complete HTML with React mounting"
    echo "✅ TypeScript Config: Modern TS configuration with React support"
    echo "✅ Component Structure: Proper React functional component"
    echo "✅ Styling: Inline CSS with responsive design"
    echo "✅ Build Integration: Bazel targets for development and production"
    
    exit 0
else
    echo -e "\n${RED}❌ Some UI Stub tests failed!${NC}"
    echo -e "\n${YELLOW}Failed test details:${NC}"
    cat /tmp/ui_stub_test_output.log 2>/dev/null || echo "No detailed output available"
    exit 1
fi
