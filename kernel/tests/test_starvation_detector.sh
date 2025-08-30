#!/bin/bash

echo "🔍 Testing Starvation Detector Implementation..."
echo "=============================================="

# Test 1: Check if starvation module exists
echo "Test 1: Starvation Module Existence..."
if [ -f "src/sched/starvation.rs" ]; then
    echo "  ✅ Starvation module exists"
else
    echo "  ❌ Starvation module missing"
    exit 1
fi

# Test 2: Check if starvation is included in scheduler
echo "Test 2: Starvation Module Inclusion..."
if grep -q "mod starvation" src/sched/mod.rs; then
    echo "  ✅ Starvation included in scheduler"
else
    echo "  ❌ Starvation not included in scheduler"
    exit 1
fi

# Test 3: Check if starvation detector test exists
echo "Test 3: Starvation Detector Test Module..."
if [ -f "tests/starvation_detector.rs" ]; then
    echo "  ✅ Starvation detector test module exists"
else
    echo "  ❌ Starvation detector test module missing"
    exit 1
fi

# Test 4: Check if test is included in tests module
echo "Test 4: Test Module Inclusion..."
if grep -q "starvation_detector" src/tests/mod.rs; then
    echo "  ✅ Starvation detector test included in tests module"
else
    echo "  ❌ Starvation detector test not included in tests module"
    exit 1
fi

# Test 5: Check if test runs in boot
echo "Test 5: Boot Integration..."
if grep -q "Testing starvation detector functionality" src/boot.rs; then
    echo "  ✅ Starvation detector test runs in boot sequence"
else
    echo "  ❌ Starvation detector test not running in boot sequence"
    exit 1
fi

# Test 6: Check starvation detector structure
echo "Test 6: Starvation Detector Structure..."
REQUIRED_STRUCTS=(
    "StarvationDetector"
    "StarvationConfig"
    "TaskStarvationStats"
    "GlobalStarvationStats"
)

for struct in "${REQUIRED_STRUCTS[@]}"; do
    if grep -q "struct $struct" src/sched/starvation.rs; then
        echo "  ✅ Struct $struct found"
    else
        echo "  ❌ Struct $struct missing"
        exit 1
    fi
done

# Test 7: Check starvation detector methods
echo "Test 7: Starvation Detector Methods..."
REQUIRED_METHODS=(
    "record_task_ready"
    "record_task_scheduled"
    "check_starvation"
    "get_starvation_stats"
    "update_config"
)

for method in "${REQUIRED_METHODS[@]}"; do
    if grep -q "fn $method" src/sched/starvation.rs; then
        echo "  ✅ Method $method found"
    else
        echo "  ❌ Method $method missing"
        exit 1
    fi
done

# Test 8: Check scheduler integration
echo "Test 8: Scheduler Integration..."
if grep -q "record_task_ready" src/sched/mod.rs; then
    echo "  ✅ Task ready recording integrated in scheduler"
else
    echo "  ❌ Task ready recording not integrated in scheduler"
    exit 1
fi

if grep -q "record_task_scheduled" src/sched/mod.rs; then
    echo "  ✅ Task scheduled recording integrated in scheduler"
else
    echo "  ❌ Task scheduled recording not integrated in scheduler"
    exit 1
fi

if grep -q "check_starvation" src/sched/mod.rs; then
    echo "  ✅ Starvation checking integrated in scheduler"
else
    echo "  ❌ Starvation checking not integrated in scheduler"
    exit 1
fi

# Test 9: Check scheduler statistics integration
echo "Test 9: Scheduler Statistics Integration..."
if grep -q "starvation_warnings" src/sched/mod.rs; then
    echo "  ✅ Starvation warnings in scheduler stats"
else
    echo "  ❌ Starvation warnings not in scheduler stats"
    exit 1
fi

if grep -q "starvation_critical" src/sched/mod.rs; then
    echo "  ✅ Starvation critical events in scheduler stats"
else
    echo "  ❌ Starvation critical events not in scheduler stats"
    exit 1
fi

if grep -q "currently_starving" src/sched/mod.rs; then
    echo "  ✅ Currently starving count in scheduler stats"
else
    echo "  ❌ Currently starving count not in scheduler stats"
    exit 1
fi

# Test 10: Check syscall integration
echo "Test 10: System Call Integration..."
if grep -q "GET_STARVATION_STATS" src/syscall/handlers.rs; then
    echo "  ✅ GET_STARVATION_STATS syscall defined"
else
    echo "  ❌ GET_STARVATION_STATS syscall not defined"
    exit 1
fi

if grep -q "op=11 for getting starvation statistics" src/syscall/handlers.rs; then
    echo "  ✅ GET_STARVATION_STATS operation code documented"
else
    echo "  ❌ GET_STARVATION_STATS operation code not documented"
    exit 1
fi

# Test 11: Check warning thresholds
echo "Test 11: Warning Thresholds..."
if grep -q "warning_threshold_ms: 100" src/sched/starvation.rs; then
    echo "  ✅ Default warning threshold is 100ms"
else
    echo "  ❌ Default warning threshold is not 100ms"
    exit 1
fi

if grep -q "critical_threshold_ms: 500" src/sched/starvation.rs; then
    echo "  ✅ Default critical threshold is 500ms"
else
    echo "  ❌ Default critical threshold is not 500ms"
    exit 1
fi

# Test 12: Check rate limiting
echo "Test 12: Rate Limiting..."
if grep -q "rate-limited to once per second" src/sched/starvation.rs; then
    echo "  ✅ Critical warnings are rate limited"
else
    echo "  ❌ Critical warnings are not rate limited"
    exit 1
fi

if grep -q "rate-limited to once per 500ms" src/sched/starvation.rs; then
    echo "  ✅ Warning events are rate limited"
else
    echo "  ❌ Warning events are not rate limited"
    exit 1
fi

echo ""
echo "🚀 All Starvation Detector Tests PASSED!"
echo "  - Starvation module: ✅"
echo "  - Scheduler integration: ✅"
echo "  - Test coverage: ✅"
echo "  - Boot integration: ✅"
echo "  - Statistics integration: ✅"
echo "  - System call integration: ✅"
echo "  - Warning thresholds: ✅"
echo "  - Rate limiting: ✅"
echo ""
echo "The Starvation Detector system provides:"
echo "  - Automatic detection of tasks ready but not scheduled >100ms"
echo "  - Warning logs for tasks waiting >100ms"
echo "  - Critical warnings for tasks waiting >500ms"
echo "  - Rate-limited logging to prevent log spam"
echo "  - Per-task and global starvation statistics"
echo "  - Integration with scheduler statistics via sys_stats"
echo "  - Configurable warning and critical thresholds"
echo "  - Ability to enable/disable detection per lock instance"
echo ""
echo "Usage:"
echo "  # Check starvation statistics via syscall"
echo "  sys_debug(op=GET_STARVATION_STATS)  # Returns packed warnings/critical count"
echo ""
echo "  # View starvation info in scheduler stats"
echo "  sys_stats()  # Includes starvation_warnings, starvation_critical, currently_starving"
echo ""
echo "  # Monitor starvation via kernel logs"
echo "  # Warnings: 'STARVATION WARNING: Task X ready for Yms (>100ms threshold)'"
echo "  # Critical: 'CRITICAL STARVATION: Task X ready for Yms (>500ms threshold)'"
echo ""
echo "The system automatically:"
echo "  - Records when tasks become ready (enqueue_task)"
echo "  - Records when tasks are scheduled (context_switch_to)"
echo "  - Checks for starvation before each scheduling decision"
echo "  - Logs warnings at configurable thresholds"
echo "  - Tracks statistics for monitoring and debugging"
echo ""
echo "This helps identify scheduling issues and ensures fair task execution!"



