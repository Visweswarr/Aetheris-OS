@echo off
REM OPA Policy Testing Script for Polymera OS
REM This script tests the p2_boot.rego policy with various scenarios

echo [POLICY] Testing OPA policy with allow/deny matrix...

REM Check if OPA is installed
where opa >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] OPA not found. Please install OPA from https://www.openpolicyagent.org/
    exit /b 1
)

REM Check if policy source exists
if not exist "p2_boot.rego" (
    echo [ERROR] Policy source file p2_boot.rego not found
    exit /b 1
)

echo [POLICY] Testing fail-closed mode (production)...
echo.

REM Test 1: Fail-closed mode with full authentication
echo [TEST] Fail-closed + Full Auth (cap=true, mac=true)
echo {"environment": "production", "has_capability": true, "has_valid_mac": true} | opa eval --data p2_boot.rego --entrypoint polymera/policy/main
echo.

REM Test 2: Fail-closed mode with missing capability
echo [TEST] Fail-closed + Missing Cap (cap=false, mac=true)
echo {"environment": "production", "has_capability": false, "has_valid_mac": true} | opa eval --data p2_boot.rego --entrypoint polymera/policy/main
echo.

REM Test 3: Fail-closed mode with missing MAC
echo [TEST] Fail-closed + Missing MAC (cap=true, mac=false)
echo {"environment": "production", "has_capability": true, "has_valid_mac": false} | opa eval --data p2_boot.rego --entrypoint polymera/policy/main
echo.

REM Test 4: Fail-closed mode with no authentication
echo [TEST] Fail-closed + No Auth (cap=false, mac=false)
echo {"environment": "production", "has_capability": false, "has_valid_mac": false} | opa eval --data p2_boot.rego --entrypoint polymera/policy/main
echo.

echo [POLICY] Testing fail-open mode (development)...
echo.

REM Test 5: Fail-open mode with full authentication
echo [TEST] Fail-open + Full Auth (cap=true, mac=true)
echo {"environment": "development", "has_capability": true, "has_valid_mac": true} | opa eval --data p2_boot.rego --entrypoint polymera/policy/main
echo.

REM Test 6: Fail-open mode with capability only
echo [TEST] Fail-open + Cap Only (cap=true, mac=false)
echo {"environment": "development", "has_capability": true, "has_valid_mac": false} | opa eval --data p2_boot.rego --entrypoint polymera/policy/main
echo.

REM Test 7: Fail-open mode with MAC only
echo [TEST] Fail-open + MAC Only (cap=false, mac=true)
echo {"environment": "development", "has_capability": false, "has_valid_mac": true} | opa eval --data p2_boot.rego --entrypoint polymera/policy/main
echo.

REM Test 8: Fail-open mode with no authentication
echo [TEST] Fail-open + No Auth (cap=false, mac=false)
echo {"environment": "development", "has_capability": false, "has_valid_mac": false} | opa eval --data p2_boot.rego --entrypoint polymera/policy/main
echo.

echo [POLICY] Testing debug build detection...
echo.

REM Test 9: Debug build detection
echo [TEST] Debug Build Detection
echo {"debug_build": true, "has_capability": false, "has_valid_mac": false} | opa eval --data p2_boot.rego --entrypoint polymera/policy/main
echo.

echo [POLICY] Policy testing complete
echo [POLICY] Expected results:
echo [POLICY] - Fail-closed: Only allow with both cap and MAC
echo [POLICY] - Fail-open: Allow all, but audit incomplete auth
echo [POLICY] - Debug builds: Automatically use fail-open mode

