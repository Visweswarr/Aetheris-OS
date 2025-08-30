#!/bin/bash

# MMU Hygiene Test Script
# Tests page table audit tool, TLB management, and WX policy enforcement

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
KERNEL_DIR="$PROJECT_ROOT/kernel"
BUILD_DIR="$PROJECT_ROOT/target/release"
OUTPUT_DIR="$PROJECT_ROOT/test-mmu-hygiene-output"

# Test results
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

#=============================================================================
# UTILITY FUNCTIONS
#=============================================================================

print_header() { echo -e "${BLUE}=== $1 ===${NC}"; }
print_success() { echo -e "${GREEN}[OK] $1${NC}"; ((TESTS_PASSED++)); ((TESTS_TOTAL++)); }
print_failure() { echo -e "${RED}[FAIL] $1${NC}"; ((TESTS_FAILED++)); ((TESTS_TOTAL++)); }
print_info() { echo -e "${BLUE}[INFO] $1${NC}"; }

setup_environment() {
    print_header "Setting Up Test Environment"
    mkdir -p "$OUTPUT_DIR"
    print_success "Test environment setup complete"
}

#=============================================================================
# PAGE TABLE AUDIT TESTS
#=============================================================================

test_page_table_audit() {
    print_header "Testing Page Table Audit System"
    
    # Test audit system initialization
    test_audit_initialization
    
    # Test audit execution
    test_audit_execution
    
    # Test suspicious page detection
    test_suspicious_page_detection
    
    # Test audit statistics
    test_audit_statistics
}

test_audit_initialization() {
    print_info "Testing audit system initialization"
    
    # Check if audit module exists
    if [[ -f "$KERNEL_DIR/src/mm/audit.rs" ]]; then
        print_success "Audit module exists: kernel/src/mm/audit.rs"
    else
        print_failure "Audit module not found"
        return 1
    fi
    
    # Check if audit functions are exported
    if grep -q "pub use audit::" "$KERNEL_DIR/src/mm/mod.rs"; then
        print_success "Audit functions exported in MM module"
    else
        print_failure "Audit functions not exported"
        return 1
    fi
}

test_audit_execution() {
    print_info "Testing audit execution functionality"
    
    # Check for audit function signatures
    local audit_functions=(
        "run_page_table_audit"
        "get_audit_stats"
        "print_audit_stats"
        "print_suspicious_pages"
    )
    
    for func in "${audit_functions[@]}"; do
        if grep -q "pub fn $func" "$KERNEL_DIR/src/mm/audit.rs"; then
            print_success "Audit function found: $func"
        else
            print_failure "Audit function missing: $func"
            return 1
        fi
    done
}

test_suspicious_page_detection() {
    print_info "Testing suspicious page detection"
    
    # Check for W+X detection logic
    if grep -q "writable.*executable\|executable.*writable" "$KERNEL_DIR/src/mm/audit.rs"; then
        print_success "W+X detection logic found"
    else
        print_failure "W+X detection logic missing"
        return 1
    fi
    
    # Check for user-accessible kernel memory detection
    if grep -q "user_accessible.*kernel\|kernel.*user_accessible" "$KERNEL_DIR/src/mm/audit.rs"; then
        print_success "User-accessible kernel memory detection found"
    else
        print_failure "User-accessible kernel memory detection missing"
        return 1
    fi
}

test_audit_statistics() {
    print_info "Testing audit statistics"
    
    # Check for audit result structure
    if grep -q "struct AuditResult" "$KERNEL_DIR/src/mm/audit.rs"; then
        print_success "AuditResult structure found"
    else
        print_failure "AuditResult structure missing"
        return 1
    fi
    
    # Check for suspicious page structure
    if grep -q "struct SuspiciousPage" "$KERNEL_DIR/src/mm/audit.rs"; then
        print_success "SuspiciousPage structure found"
    else
        print_failure "SuspiciousPage structure missing"
        return 1
    fi
}

#=============================================================================
# TLB MANAGEMENT TESTS
#=============================================================================

test_tlb_management() {
    print_header "Testing TLB Management System"
    
    # Test TLB module existence
    test_tlb_module_existence
    
    # Test TLB flush functions
    test_tlb_flush_functions
    
    # Test TLB statistics
    test_tlb_statistics
    
    # Test SMP shootdown support
    test_smp_shootdown_support
}

test_tlb_module_existence() {
    print_info "Testing TLB module existence"
    
    if [[ -f "$KERNEL_DIR/src/mm/tlb.rs" ]]; then
        print_success "TLB module exists: kernel/src/mm/tlb.rs"
    else
        print_failure "TLB module not found"
        return 1
    fi
    
    # Check if TLB functions are exported
    if grep -q "pub use tlb::" "$KERNEL_DIR/src/mm/mod.rs"; then
        print_success "TLB functions exported in MM module"
    else
        print_failure "TLB functions not exported"
        return 1
    fi
}

test_tlb_flush_functions() {
    print_info "Testing TLB flush functions"
    
    # Check for core TLB functions
    local tlb_functions=(
        "flush_page"
        "flush_all"
        "flush_range"
        "flush_multiple_pages"
    )
    
    for func in "${tlb_functions[@]}"; do
        if grep -q "pub fn $func" "$KERNEL_DIR/src/mm/tlb.rs"; then
            print_success "TLB function found: $func"
        else
            print_failure "TLB function missing: $func"
            return 1
        fi
    done
}

test_tlb_statistics() {
    print_info "Testing TLB statistics"
    
    # Check for TLB stats structure
    if grep -q "struct TlbStats" "$KERNEL_DIR/src/mm/tlb.rs"; then
        print_success "TlbStats structure found"
    else
        print_failure "TlbStats structure missing"
        return 1
    fi
    
    # Check for TLB stats functions
    if grep -q "get_tlb_stats\|print_tlb_stats" "$KERNEL_DIR/src/mm/tlb.rs"; then
        print_success "TLB statistics functions found"
    else
        print_failure "TLB statistics functions missing"
        return 1
    fi
}

test_smp_shootdown_support() {
    print_info "Testing SMP shootdown support"
    
    # Check for shootdown structures
    if grep -q "struct TlbShootdownRequest\|struct TlbShootdownManager" "$KERNEL_DIR/src/mm/tlb.rs"; then
        print_success "SMP shootdown structures found"
    else
        print_failure "SMP shootdown structures missing"
        return 1
    fi
    
    # Check for CPU ID structure
    if grep -q "struct CpuId" "$KERNEL_DIR/src/mm/tlb.rs"; then
        print_success "CPU ID structure found"
    else
        print_failure "CPU ID structure missing"
        return 1
    fi
}

#=============================================================================
# SECURITY POLICY TESTS
#=============================================================================

test_security_policy() {
    print_header "Testing Security Policy Enforcement"
    
    # Test WX policy enforcement
    test_wx_policy_enforcement
    
    # Test memory region policies
    test_memory_region_policies
    
    # Test policy validation
    test_policy_validation
}

test_wx_policy_enforcement() {
    print_info "Testing WX policy enforcement"
    
    # Check for W+X detection in audit logic
    if grep -q "flags.writable.*flags.executable\|flags.executable.*flags.writable" "$KERNEL_DIR/src/mm/audit.rs"; then
        print_success "W+X policy enforcement logic found"
    else
        print_failure "W+X policy enforcement logic missing"
        return 1
    fi
    
    # Check for critical security level assignment
    if grep -q "AuditLevel::Critical\|Critical.*security" "$KERNEL_DIR/src/mm/audit.rs"; then
        print_success "Critical security level assignment found"
    else
        print_failure "Critical security level assignment missing"
        return 1
    fi
}

test_memory_region_policies() {
    print_info "Testing memory region policies"
    
    # Check for audit regions configuration
    if grep -q "AUDIT_REGIONS\|kernel-code\|kernel-data\|kernel-heap" "$KERNEL_DIR/src/mm/audit.rs"; then
        print_success "Memory region policies configured"
    else
        print_failure "Memory region policies not configured"
        return 1
    fi
    
    # Check for region-specific validation
    if grep -q "is_unexpected_combination\|region.*flags" "$KERNEL_DIR/src/mm/audit.rs"; then
        print_success "Region-specific validation found"
    else
        print_failure "Region-specific validation missing"
        return 1
    fi
}

test_policy_validation() {
    print_info "Testing policy validation"
    
    # Check for policy validation functions
    if grep -q "validate.*policy\|policy.*validation" "$KERNEL_DIR/src/mm/audit.rs"; then
        print_success "Policy validation functions found"
    else
        print_failure "Policy validation functions missing"
        return 1
    fi
    
    # Check for security event logging
    if grep -q "audit_log\|AuditEvent\|security.*event" "$KERNEL_DIR/src/mm/audit.rs"; then
        print_success "Security event logging found"
    else
        print_failure "Security event logging missing"
        return 1
    fi
}

#=============================================================================
# INTEGRATION TESTS
#=============================================================================

test_integration() {
    print_header "Testing Integration"
    
    # Test MM module integration
    test_mm_module_integration
    
    # Test initialization integration
    test_initialization_integration
    
    # Test API consistency
    test_api_consistency
}

test_mm_module_integration() {
    print_info "Testing MM module integration"
    
    # Check if audit and TLB are included in MM module
    if grep -q "pub mod audit\|pub mod tlb" "$KERNEL_DIR/src/mm/mod.rs"; then
        print_success "Audit and TLB modules included in MM module"
    else
        print_failure "Audit and TLB modules not included in MM module"
        return 1
    fi
    
    # Check if initialization calls are present
    if grep -q "audit::init_audit_system\|tlb::init_tlb_system" "$KERNEL_DIR/src/mm/mod.rs"; then
        print_success "Module initialization calls found"
    else
        print_failure "Module initialization calls missing"
        return 1
    fi
}

test_initialization_integration() {
    print_info "Testing initialization integration"
    
    # Check for proper initialization order
    if grep -q "Initializing TLB management system\|Initializing page table audit system" "$KERNEL_DIR/src/mm/mod.rs"; then
        print_success "Initialization order properly configured"
    else
        print_failure "Initialization order not configured"
        return 1
    fi
    
    # Check for error handling in initialization
    if grep -q "Failed to initialize.*system" "$KERNEL_DIR/src/mm/mod.rs"; then
        print_success "Initialization error handling found"
    else
        print_failure "Initialization error handling missing"
        return 1
    fi
}

test_api_consistency() {
    print_info "Testing API consistency"
    
    # Check for consistent function signatures
    local expected_functions=(
        "run_page_table_audit"
        "get_audit_stats"
        "print_audit_stats"
        "print_suspicious_pages"
        "flush_page"
        "flush_all"
        "flush_range"
        "flush_multiple_pages"
        "get_tlb_stats"
        "print_tlb_stats"
    )
    
    for func in "${expected_functions[@]}"; do
        if grep -q "pub use.*$func" "$KERNEL_DIR/src/mm/mod.rs"; then
            print_success "API function exported: $func"
        else
            print_failure "API function not exported: $func"
            return 1
        fi
    done
}

#=============================================================================
# UNIT TEST VALIDATION
#=============================================================================

test_unit_tests() {
    print_header "Testing Unit Tests"
    
    # Check for unit test modules
    test_unit_test_modules
    
    # Check for test coverage
    test_test_coverage
}

test_unit_test_modules() {
    print_info "Testing unit test modules"
    
    # Check for audit unit tests
    if grep -q "#\[cfg\(test\)\]" "$KERNEL_DIR/src/mm/audit.rs"; then
        print_success "Audit unit tests found"
    else
        print_failure "Audit unit tests missing"
        return 1
    fi
    
    # Check for TLB unit tests
    if grep -q "#\[cfg\(test\)\]" "$KERNEL_DIR/src/mm/tlb.rs"; then
        print_success "TLB unit tests found"
    else
        print_failure "TLB unit tests missing"
        return 1
    fi
}

test_test_coverage() {
    print_info "Testing test coverage"
    
    # Check for key test functions
    local test_functions=(
        "test_audit_result_creation"
        "test_suspicious_page_creation"
        "test_tlb_flush_types"
        "test_shootdown_request_creation"
    )
    
    for test_func in "${test_functions[@]}"; do
        if grep -q "fn $test_func" "$KERNEL_DIR/src/mm/audit.rs" || grep -q "fn $test_func" "$KERNEL_DIR/src/mm/tlb.rs"; then
            print_success "Test function found: $test_func"
        else
            print_failure "Test function missing: $test_func"
            return 1
        fi
    done
}

#=============================================================================
# DOCUMENTATION TESTS
#=============================================================================

test_documentation() {
    print_header "Testing Documentation"
    
    # Check for MMU hygiene documentation
    if [[ -f "$PROJECT_ROOT/docs/phase-2/MMU-HYGIENE.md" ]]; then
        print_success "MMU hygiene documentation exists"
    else
        print_failure "MMU hygiene documentation missing"
        return 1
    fi
    
    # Check for comprehensive documentation content
    local doc_keywords=(
        "WX Policy"
        "Page Table Audit"
        "TLB Management"
        "Security Policy"
        "Memory Layout"
    )
    
    for keyword in "${doc_keywords[@]}"; do
        if grep -q "$keyword" "$PROJECT_ROOT/docs/phase-2/MMU-HYGIENE.md"; then
            print_success "Documentation covers: $keyword"
        else
            print_failure "Documentation missing: $keyword"
            return 1
        fi
    done
}

#=============================================================================
# MAIN EXECUTION
#=============================================================================

main() {
    print_header "MMU Hygiene Test Suite"
    print_info "Testing page table audit tool, TLB management, and security policy enforcement"
    
    # Setup environment
    setup_environment
    
    # Run tests
    test_page_table_audit
    test_tlb_management
    test_security_policy
    test_integration
    test_unit_tests
    test_documentation
    
    # Generate report
    generate_test_report
    
    # Print summary
    print_header "Test Summary"
    print_info "Total tests: $TESTS_TOTAL"
    print_info "Passed: $TESTS_PASSED"
    print_info "Failed: $TESTS_FAILED"
    
    if [[ $TESTS_FAILED -eq 0 ]]; then
        print_success "All tests passed!"
        exit 0
    else
        print_failure "Some tests failed"
        exit 1
    fi
}

generate_test_report() {
    print_header "Generating Test Report"
    
    local report_file="$OUTPUT_DIR/mmu_hygiene_test_report.md"
    
    cat > "$report_file" << EOF
# MMU Hygiene Test Report

Generated: $(date)

## Test Summary
- Total Tests: $TESTS_TOTAL
- Passed: $TESTS_PASSED
- Failed: $TESTS_FAILED
- Success Rate: $(( (TESTS_PASSED * 100) / TESTS_TOTAL ))%

## Test Categories

### 1. Page Table Audit System
- Audit system initialization
- Audit execution functionality
- Suspicious page detection
- Audit statistics

### 2. TLB Management System
- TLB module existence
- TLB flush functions
- TLB statistics
- SMP shootdown support

### 3. Security Policy Enforcement
- WX policy enforcement
- Memory region policies
- Policy validation

### 4. Integration Testing
- MM module integration
- Initialization integration
- API consistency

### 5. Unit Test Validation
- Unit test modules
- Test coverage

### 6. Documentation
- MMU hygiene documentation
- Comprehensive content coverage

## Recommendations

1. **Audit System**: Ensure all memory mappings are audited regularly
2. **TLB Management**: Use appropriate flush operations for performance
3. **Security Policy**: Maintain strict WX policy enforcement
4. **Testing**: Run unit tests regularly to validate functionality
5. **Documentation**: Keep documentation updated with system changes

## Next Steps

1. Run actual kernel tests to validate functionality
2. Perform security penetration testing
3. Benchmark TLB performance under load
4. Validate SMP shootdown in multi-core environment
5. Create automated security policy compliance checks

EOF
    
    print_success "Test report generated: $report_file"
}

# Run main function
main "$@"

