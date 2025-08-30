@echo off
REM Test script for Phase 1 Documentation Guard (Windows version)
REM Simulates different scenarios to validate the guard logic

setlocal enabledelayedexpansion

echo 🧪 Testing Phase 1 Documentation Guard
echo Project root: %CD%

REM Check if Node.js is available
node --version >nul 2>&1
if errorlevel 1 (
    echo ❌ Node.js is required but not installed
    exit /b 1
)

REM Install dependencies if needed
if not exist "tooling\ci\node_modules" (
    echo 📦 Installing CI tool dependencies...
    cd tooling\ci
    npm install
    cd ..\..
)

echo.
echo === Testing Phase 1 Guard Logic ===

REM Test 1: Run the test suite
echo.
echo 🧪 Test 1: Running unit tests
cd tooling\ci
npm test
if errorlevel 1 (
    echo ❌ Unit tests failed
    cd ..\..
    exit /b 1
) else (
    echo ✅ Unit tests passed
)
cd ..\..

REM Test 2: Simulate current state
echo.
echo 🧪 Test 2: Current repository state
cd tooling\ci
set BASE_SHA=HEAD~1
set HEAD_SHA=HEAD
node check_phase_docs.ts
if errorlevel 1 (
    echo ❌ Current state validation failed
) else (
    echo ✅ Current state validation passed
)
cd ..\..

echo.
echo 🎉 Phase 1 guard basic tests completed!
echo.
echo The CI guard appears to be working. For full testing including
echo Git operations, please run the bash version on WSL or Linux:
echo   bash scripts/test-phase1-guard.sh
echo.
echo Manual testing:
echo   1. Make changes to kernel/ files
echo   2. Run: cd tooling\ci ^&^& npm run check-phase-docs
echo   3. Verify it fails without docs/phase-1/ updates
echo   4. Add docs changes and verify it passes

pause
