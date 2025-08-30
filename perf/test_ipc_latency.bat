@echo off
REM IPC Latency Collector Test Script for Windows
REM Tests the IPC latency tracking functionality in Polymera OS

setlocal enabledelayedexpansion

echo 🔬 Testing IPC Latency Collector
echo =================================

REM Test configuration
set TEST_DURATION_MS=1000
set MESSAGE_COUNT=50
set TARGET_P50_US=200
set TARGET_P95_US=500

echo Test Configuration:
echo   Duration: %TEST_DURATION_MS%ms
echo   Messages: %MESSAGE_COUNT%
echo   Target P50: ^<%TARGET_P50_US%μs
echo   Target P95: ^<%TARGET_P95_US%μs
echo.

REM Function to run performance test
:run_performance_test
echo Running IPC Performance Test...
echo Building performance script...

REM Check for Bazel
where bazel >nul 2>&1
if %errorlevel% equ 0 (
    echo Testing Bazel build...
    bazel build //perf:check_ipc
    if %errorlevel% equ 0 (
        echo ✅ Bazel build successful
        set PERF_BIN=bazel-bin\perf\check_ipc.exe
    ) else (
        echo ❌ Bazel build failed
        goto :build_failed
    )
) else (
    echo Bazel not found, trying Cargo...
    goto :try_cargo
)

:try_cargo
REM Check for Cargo
where cargo >nul 2>&1
if %errorlevel% equ 0 (
    echo Testing Cargo build...
    cd perf
    cargo build --release
    if %errorlevel% equ 0 (
        echo ✅ Cargo build successful
        cd ..
        set PERF_BIN=perf\target\release\check_ipc.exe
    ) else (
        echo ❌ Cargo build failed
        cd ..
        goto :build_failed
    )
) else (
    echo ❌ Neither Bazel nor Cargo found
    goto :build_failed
)

REM Run the test
echo Executing performance test...
if exist "!PERF_BIN!" (
    "!PERF_BIN!"
    set TEST_EXIT_CODE=!errorlevel!
    
    if !TEST_EXIT_CODE! equ 0 (
        echo ✅ Performance test PASSED
        goto :test_passed
    ) else (
        echo ❌ Performance test FAILED
        goto :test_failed
    )
) else (
    echo ❌ Performance binary not found: !PERF_BIN!
    goto :test_failed
)

:build_failed
echo ❌ Build system test FAILED
set exit_code=1
goto :end

:test_passed
echo ✅ Performance test PASSED
goto :end

:test_failed
echo ❌ Performance test FAILED
set exit_code=1
goto :end

:end
echo.
echo =================================
if defined exit_code (
    echo ❌ SOME TESTS FAILED
    echo Please check the output above for details.
) else (
    echo 🎉 ALL TESTS PASSED!
    echo IPC Latency Collector is working correctly.
)
echo =================================

exit /b %exit_code%



