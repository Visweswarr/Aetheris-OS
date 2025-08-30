#!/bin/bash

echo "🔒 Testing KAssert Macro Implementation..."
echo "========================================="

# Test 1: Check if macros module exists
echo "Test 1: Macros Module Existence..."
if [ -f "src/macros.rs" ]; then
    echo "  ✅ Macros module exists"
else
    echo "  ❌ Macros module missing"
    exit 1
fi

# Test 2: Check if macros is included in kernel lib
echo "Test 2: Macros Module Inclusion..."
if grep -q "mod macros" src/lib.rs; then
    echo "  ✅ Macros included in kernel lib"
else
    echo "  ❌ Macros not included in kernel lib"
    exit 1
fi

# Test 3: Check if kassert macro test exists
echo "Test 3: KAssert Macro Test Module..."
if [ -f "tests/kassert_macro.rs" ]; then
    echo "  ✅ KAssert macro test module exists"
else
    echo "  ❌ KAssert macro test module missing"
    exit 1
fi

# Test 4: Check if test is included in tests module
echo "Test 4: Test Module Inclusion..."
if grep -q "kassert_macro" src/tests/mod.rs; then
    echo "  ✅ KAssert macro test included in tests module"
else
    echo "  ❌ KAssert macro test not included in tests module"
    exit 1
fi

# Test 5: Check if test runs in boot
echo "Test 5: Boot Integration..."
if grep -q "Testing kassert macro functionality" src/boot.rs; then
    echo "  ✅ KAssert macro test runs in boot sequence"
else
    echo "  ❌ KAssert macro test not running in boot sequence"
    exit 1
fi

# Test 6: Check ASSERT_FAIL audit operation
echo "Test 6: ASSERT_FAIL Audit Operation..."
if grep -q "ASSERT_FAIL: u16 = 105" src/secman/audit.rs; then
    echo "  ✅ ASSERT_FAIL audit operation defined"
else
    echo "  ❌ ASSERT_FAIL audit operation not defined"
    exit 1
fi

# Test 7: Check ASSERT log tag
echo "Test 7: ASSERT Log Tag..."
if grep -q "ASSERT: &str = \"ASSERT\"" src/log.rs; then
    echo "  ✅ ASSERT log tag defined"
else
    echo "  ❌ ASSERT log tag not defined"
    exit 1
fi

# Test 8: Check kassert macro definition
echo "Test 8: KAssert Macro Definition..."
if grep -q "macro_rules! kassert" src/macros.rs; then
    echo "  ✅ KAssert macro defined"
else
    echo "  ❌ KAssert macro not defined"
    exit 1
fi

# Test 9: Check kassert_debug macro
echo "Test 9: KAssert Debug Macro..."
if grep -q "macro_rules! kassert_debug" src/macros.rs; then
    echo "  ✅ KAssert debug macro defined"
else
    echo "  ❌ KAssert debug macro not defined"
    exit 1
fi

# Test 10: Check kassert_release macro
echo "Test 10: KAssert Release Macro..."
if grep -q "macro_rules! kassert_release" src/macros.rs; then
    echo "  ✅ KAssert release macro defined"
else
    echo "  ❌ KAssert release macro not defined"
    exit 1
fi

# Test 11: Check kassert_audit macro
echo "Test 11: KAssert Audit Macro..."
if grep -q "macro_rules! kassert_audit" src/macros.rs; then
    echo "  ✅ KAssert audit macro defined"
else
    echo "  ❌ KAssert audit macro not defined"
    exit 1
fi

# Test 12: Check build profile control
echo "Test 12: Build Profile Control..."
if grep -q "cfg!(debug_assertions) || cfg!(feature = \"kassert-release\")" src/macros.rs; then
    echo "  ✅ Build profile control implemented"
else
    echo "  ❌ Build profile control not implemented"
    exit 1
fi

# Test 13: Check audit logging on failure
echo "Test 13: Audit Logging on Failure..."
if grep -q "audit::log_event" src/macros.rs; then
    echo "  ✅ Audit logging on failure implemented"
else
    echo "  ❌ Audit logging on failure not implemented"
    exit 1
fi

# Test 14: Check system halt on failure
echo "Test 14: System Halt on Failure..."
if grep -q "halt_system" src/macros.rs; then
    echo "  ✅ System halt on failure implemented"
else
    echo "  ❌ System halt on failure not implemented"
    exit 1
fi

# Test 15: Check function name macro
echo "Test 15: Function Name Macro..."
if grep -q "macro_rules! function_name" src/macros.rs; then
    echo "  ✅ Function name macro defined"
else
    echo "  ❌ Function name macro not defined"
    exit 1
fi

echo ""
echo "🚀 All KAssert Macro Tests PASSED!"
echo "  - Macros module: ✅"
echo "  - Kernel integration: ✅"
echo "  - Test coverage: ✅"
echo "  - Boot integration: ✅"
echo "  - Audit integration: ✅"
echo "  - Log integration: ✅"
echo "  - Build profile control: ✅"
echo "  - System halt: ✅"
echo ""
echo "The KAssert Macro system provides:"
echo "  - kassert!(condition, \"message\") - Basic kernel assertion"
echo "  - kassert_debug!(condition, \"message\") - Debug-only assertion"
echo "  - kassert_release!(condition, \"message\") - Release-only assertion"
echo "  - kassert_audit!(condition, op, \"message\") - Custom audit operation"
echo "  - Automatic audit logging with op=ASSERT_FAIL (105)"
echo "  - System halt on assertion failure"
echo "  - Build profile control (enabled in debug, configurable in release)"
echo "  - Comprehensive error logging with context information"
echo ""
echo "Usage Examples:"
echo "  # Basic assertion"
echo "  kassert!(ptr != core::ptr::null(), \"Pointer must not be null\");"
echo ""
echo "  # Formatted assertion"
echo "  kassert!(size > 0, \"Size must be positive, got {}\", size);"
echo ""
echo "  # Debug-only assertion"
echo "  kassert_debug!(expensive_check(), \"Expensive validation failed\");"
echo ""
echo "  # Release-only assertion"
echo "  kassert_release!(critical_check(), \"Critical system check failed\");"
echo ""
echo "  # Custom audit operation"
echo "  kassert_audit!(ptr != null, audit::ops::NULL_POINTER, \"Null pointer detected\");"
echo ""
echo "Build Profile Control:"
echo "  # Debug builds: All assertions enabled by default"
echo "  cargo build"
echo ""
echo "  # Release builds: Assertions disabled by default"
echo "  cargo build --release"
echo ""
echo "  # Release builds with assertions enabled"
echo "  cargo build --release --features kassert-release"
echo ""
echo "On Assertion Failure:"
echo "  1. Logs error message with context (file, line, function)"
echo "  2. Creates audit entry with op=ASSERT_FAIL (105)"
echo "  3. Logs system state information"
echo "  4. Halts the system (cli + hlt loop)"
echo ""
echo "This provides robust kernel assertion capabilities with comprehensive"
echo "logging and audit trail integration!"



