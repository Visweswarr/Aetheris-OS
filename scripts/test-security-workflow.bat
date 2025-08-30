@echo off
REM Test script for Security & Side-Channel CI Integration

echo 🔒 Testing Security & Side-Channel CI Integration
echo ==================================================

REM Test counters
set TESTS_PASSED=0
set TESTS_FAILED=0

REM Phase 1: File Structure Validation
echo.
echo Phase 1: File Structure Validation
echo ----------------------------------------

REM Check workflow file
if exist ".github\workflows\phase-2-security.yml" (
    echo ✅ Found: Security workflow file
    set /a TESTS_PASSED+=1
) else (
    echo ❌ Missing: Security workflow file
    set /a TESTS_FAILED+=1
)

REM Check security configuration
if exist "security\allowlist.yaml" (
    echo ✅ Found: Security allowlist configuration
    set /a TESTS_PASSED+=1
) else (
    echo ❌ Missing: Security allowlist configuration
    set /a TESTS_FAILED+=1
)

REM Check documentation
if exist "docs\security\SCANS.md" (
    echo ✅ Found: Security scanning documentation
    set /a TESTS_PASSED+=1
) else (
    echo ❌ Missing: Security scanning documentation
    set /a TESTS_FAILED+=1
)

REM Check baseline update script
if exist "tooling\analysis\update_security_baselines.py" (
    echo ✅ Found: Security baseline update script
    set /a TESTS_PASSED+=1
) else (
    echo ❌ Missing: Security baseline update script
    set /a TESTS_FAILED+=1
)

REM Check baseline directory
if exist "security\baselines" (
    echo ✅ Found: Security baselines directory
    set /a TESTS_PASSED+=1
) else (
    echo ❌ Missing: Security baselines directory
    set /a TESTS_FAILED+=1
)

REM Phase 2: Workflow Syntax Validation
echo.
echo Phase 2: Workflow Syntax Validation
echo ----------------------------------------

REM Check if workflow file is valid YAML (basic check)
findstr /C:"name:" ".github\workflows\phase-2-security.yml" >nul 2>&1
if %errorlevel% equ 0 (
    echo ✅ Basic workflow structure validation
    set /a TESTS_PASSED+=1
) else (
    echo ❌ Workflow structure validation failed
    set /a TESTS_FAILED+=1
)

REM Check if allowlist is valid YAML (basic check)
findstr /C:"allowlist:" "security\allowlist.yaml" >nul 2>&1
if %errorlevel% equ 0 (
    echo ✅ Basic allowlist structure validation
    set /a TESTS_PASSED+=1
) else (
    echo ❌ Allowlist structure validation failed
    set /a TESTS_FAILED+=1
)

REM Phase 3: Python Script Validation
echo.
echo Phase 3: Python Script Validation
echo ----------------------------------------

REM Check Python script syntax
python -m py_compile "tooling\analysis\update_security_baselines.py" >nul 2>&1
if %errorlevel% equ 0 (
    echo ✅ Baseline update script syntax validation
    set /a TESTS_PASSED+=1
) else (
    echo ❌ Baseline update script syntax validation failed
    set /a TESTS_FAILED+=1
)

REM Phase 4: Side-Channel Scanner Validation
echo.
echo Phase 4: Side-Channel Scanner Validation
echo ----------------------------------------

REM Check if side-channel scanner exists
if exist "tooling\analysis\sidechan_scan.py" (
    echo ✅ Found: Side-channel scanner
    set /a TESTS_PASSED+=1
    
    REM Check scanner syntax
    python -m py_compile "tooling\analysis\sidechan_scan.py" >nul 2>&1
    if %errorlevel% equ 0 (
        echo ✅ Side-channel scanner syntax validation
        set /a TESTS_PASSED+=1
    ) else (
        echo ❌ Side-channel scanner syntax validation failed
        set /a TESTS_FAILED+=1
    )
) else (
    echo ❌ Missing: Side-channel scanner
    set /a TESTS_FAILED+=1
)

REM Phase 5: Integration Validation
echo.
echo Phase 5: Integration Validation
echo ----------------------------------------

REM Check if security directories exist in project structure
if exist "kernel" (
    if exist "crypto" (
        if exist "services" (
            if exist "tooling" (
                echo ✅ Found: All required scopes for security analysis
                set /a TESTS_PASSED+=1
            ) else (
                echo ❌ Missing: tooling scope
                set /a TESTS_FAILED+=1
            )
        ) else (
            echo ❌ Missing: services scope
            set /a TESTS_FAILED+=1
        )
    ) else (
        echo ❌ Missing: crypto scope
        set /a TESTS_FAILED+=1
    )
) else (
    echo ❌ Missing: kernel scope
    set /a TESTS_FAILED+=1
)

REM Phase 6: Configuration Validation
echo.
echo Phase 6: Configuration Validation
echo ----------------------------------------

REM Check allowlist configuration structure (basic check)
findstr /C:"settings:" "security\allowlist.yaml" >nul 2>&1
if %errorlevel% equ 0 (
    echo ✅ Allowlist configuration structure validation
    set /a TESTS_PASSED+=1
) else (
    echo ❌ Allowlist configuration structure validation failed
    set /a TESTS_FAILED+=1
)

REM Test Summary
echo.
echo Test Summary
echo =============
echo Total Tests: %TESTS_PASSED%
set /a TOTAL_TESTS=%TESTS_PASSED%+%TESTS_FAILED%
echo Total Tests: %TOTAL_TESTS%
echo Passed: %TESTS_PASSED%
echo Failed: %TESTS_FAILED%

if %TESTS_FAILED% equ 0 (
    echo.
    echo 🎉 All security workflow tests passed!
    echo The Security & Side-Channel CI Integration is ready for use.
    exit /b 0
) else (
    echo.
    echo ❌ Some tests failed. Please review the issues above.
    exit /b 1
)
