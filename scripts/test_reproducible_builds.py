#!/usr/bin/env python3
"""
Test Script for Reproducible Build Verification

This script demonstrates the reproducible build verification system
and provides a simple way to test it without running full builds.
"""

import sys
import os
from pathlib import Path

def test_script_availability():
    """Test if the verification script is available"""
    print("🔍 Testing Reproducible Build Verification System")
    print("=" * 60)
    
    # Check if main script exists
    script_path = Path("scripts/verify_reproducible_builds.py")
    if script_path.exists():
        print("✅ Main verification script found")
        print(f"   Path: {script_path}")
        
        # Check script permissions
        if os.access(script_path, os.R_OK):
            print("✅ Script is readable")
        else:
            print("❌ Script is not readable")
            return False
            
        if os.access(script_path, os.X_OK):
            print("✅ Script is executable")
        else:
            print("⚠️  Script is not executable (may need chmod +x)")
    else:
        print("❌ Main verification script not found")
        print(f"   Expected: {script_path}")
        return False
    
    return True

def test_dependencies():
    """Test if required dependencies are available"""
    print("\n📦 Testing Dependencies")
    print("-" * 30)
    
    # Check Python version
    python_version = sys.version_info
    print(f"Python version: {python_version.major}.{python_version.minor}.{python_version.micro}")
    
    if python_version >= (3, 7):
        print("✅ Python version is compatible (>= 3.7)")
    else:
        print("❌ Python version is too old (< 3.7)")
        return False
    
    # Check required modules
    required_modules = [
        'hashlib', 'tempfile', 'shutil', 'json', 'time', 'pathlib', 'dataclasses'
    ]
    
    missing_modules = []
    for module in required_modules:
        try:
            __import__(module)
            print(f"✅ {module} module available")
        except ImportError:
            print(f"❌ {module} module missing")
            missing_modules.append(module)
    
    if missing_modules:
        print(f"\n❌ Missing required modules: {', '.join(missing_modules)}")
        return False
    
    return True

def test_kernel_directory():
    """Test if kernel directory structure is correct"""
    print("\n🔧 Testing Kernel Directory")
    print("-" * 30)
    
    kernel_dir = Path("kernel")
    if not kernel_dir.exists():
        print("❌ Kernel directory not found")
        return False
    
    print(f"✅ Kernel directory found: {kernel_dir}")
    
    # Check for Cargo.toml
    cargo_toml = kernel_dir / "Cargo.toml"
    if cargo_toml.exists():
        print("✅ Cargo.toml found")
    else:
        print("❌ Cargo.toml not found in kernel directory")
        return False
    
    # Check for source files
    src_dir = kernel_dir / "src"
    if src_dir.exists():
        print("✅ Source directory found")
        
        # Count Rust files
        rust_files = list(src_dir.rglob("*.rs"))
        print(f"   Found {len(rust_files)} Rust source files")
        
        if rust_files:
            print("✅ Rust source files present")
        else:
            print("⚠️  No Rust source files found")
    else:
        print("❌ Source directory not found")
        return False
    
    return True

def test_ci_workflow():
    """Test if CI workflow is properly configured"""
    print("\n🚀 Testing CI Workflow")
    print("-" * 30)
    
    workflow_path = Path(".github/workflows/reproducible-builds.yml")
    if workflow_path.exists():
        print("✅ CI workflow file found")
        print(f"   Path: {workflow_path}")
        
        # Check workflow content
        try:
            with open(workflow_path, 'r') as f:
                content = f.read()
                
            if "verify-reproducible-builds" in content:
                print("✅ Workflow contains verification job")
            else:
                print("❌ Workflow missing verification job")
                return False
                
            if "phase-1.5-repro" in content:
                print("✅ Workflow references correct CI gate")
            else:
                print("❌ Workflow missing CI gate reference")
                return False
                
        except Exception as e:
            print(f"❌ Could not read workflow file: {e}")
            return False
    else:
        print("❌ CI workflow file not found")
        return False
    
    return True

def test_makefile_targets():
    """Test if Makefile targets are properly configured"""
    print("\n🔨 Testing Makefile Targets")
    print("-" * 30)
    
    makefile_path = Path("Makefile")
    if not makefile_path.exists():
        print("❌ Makefile not found")
        return False
    
    print("✅ Makefile found")
    
    # Check for reproducible build targets
    try:
        with open(makefile_path, 'r') as f:
            content = f.read()
            
        if "verify-reproducible" in content:
            print("✅ verify-reproducible target found")
        else:
            print("❌ verify-reproducible target missing")
            return False
            
        if "reproducible-builds" in content:
            print("✅ reproducible-builds target found")
        else:
            print("❌ reproducible-builds target missing")
            return False
            
    except Exception as e:
        print(f"❌ Could not read Makefile: {e}")
        return False
    
    return True

def run_dry_run_test():
    """Run a dry-run test of the verification system"""
    print("\n🧪 Running Dry-Run Test")
    print("-" * 30)
    
    try:
        # Import the verification module
        sys.path.insert(0, 'scripts')
        from verify_reproducible_builds import ReproducibleBuildVerifier
        
        print("✅ Successfully imported verification module")
        
        # Create verifier instance
        verifier = ReproducibleBuildVerifier("kernel")
        print("✅ Successfully created verifier instance")
        
        # Test environment creation (without actual building)
        print("✅ Verification system is ready for use")
        
    except ImportError as e:
        print(f"❌ Could not import verification module: {e}")
        return False
    except Exception as e:
        print(f"❌ Error during dry-run test: {e}")
        return False
    
    return True

def main():
    """Main test function"""
    print("🧪 Reproducible Build Verification System Test")
    print("=" * 60)
    
    tests = [
        ("Script Availability", test_script_availability),
        ("Dependencies", test_dependencies),
        ("Kernel Directory", test_kernel_directory),
        ("CI Workflow", test_ci_workflow),
        ("Makefile Targets", test_makefile_targets),
        ("Dry-Run Test", run_dry_run_test)
    ]
    
    passed = 0
    total = len(tests)
    
    for test_name, test_func in tests:
        try:
            if test_func():
                passed += 1
            else:
                print(f"❌ {test_name} test failed")
        except Exception as e:
            print(f"❌ {test_name} test crashed: {e}")
    
    print("\n📊 Test Results Summary")
    print("=" * 60)
    print(f"Tests passed: {passed}/{total}")
    
    if passed == total:
        print("🎉 All tests passed! Reproducible build system is ready.")
        print("\nTo run verification:")
        print("  make verify-reproducible")
        print("  python3 scripts/verify_reproducible_builds.py kernel")
        return 0
    else:
        print("❌ Some tests failed. Please fix issues before using the system.")
        return 1

if __name__ == "__main__":
    sys.exit(main())

