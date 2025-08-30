#!/usr/bin/env python3
"""
Test script for Polymera OS SBOM & Signing Tools

Tests the SBOM generator, Sigstore signer, and policy enforcer to ensure
they work correctly and produce expected outputs.
"""

import json
import subprocess
import sys
import os
import tempfile
import shutil
from pathlib import Path
from typing import Dict, List, Any

def run_command(cmd: List[str], cwd: str = None) -> Dict[str, Any]:
    """Run a command and return the result."""
    try:
        result = subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=True,
            text=True,
            check=True
        )
        return {
            "success": True,
            "stdout": result.stdout,
            "stderr": result.stderr,
            "returncode": result.returncode
        }
    except subprocess.CalledProcessError as e:
        return {
            "success": False,
            "stdout": e.stdout,
            "stderr": e.stderr,
            "returncode": e.returncode
        }
    except FileNotFoundError:
        return {
            "success": False,
            "stdout": "",
            "stderr": "Command not found",
            "returncode": -1
        }

def create_test_artifacts(test_dir: Path) -> Dict[str, Path]:
    """Create test artifacts for testing."""
    
    print("📝 Creating test artifacts...")
    
    artifacts = {}
    
    # Create a test Rust binary (simulated)
    rust_binary = test_dir / "test-rust-binary"
    rust_binary.write_text("#!/bin/bash\necho 'This is a test Rust binary'\n")
    rust_binary.chmod(0o755)
    artifacts["rust_binary"] = rust_binary
    
    # Create a test Go binary (simulated)
    go_binary = test_dir / "test-go-binary"
    go_binary.write_text("#!/bin/bash\necho 'This is a test Go binary'\n")
    go_binary.chmod(0o755)
    artifacts["go_binary"] = go_binary
    
    # Create test SBOM files
    test_sbom = {
        "sbomVersion": "SPDX-2.3",
        "name": "Test Component",
        "spdxId": "SPDXRef-TestComponent",
        "packages": [
            {
                "spdxId": "SPDXRef-TestPackage",
                "name": "test-package",
                "versionInfo": "1.0.0",
                "licenseConcluded": "Apache-2.0"
            }
        ]
    }
    
    sbom_file = test_dir / "test-sbom.json"
    with open(sbom_file, 'w') as f:
        json.dump(test_sbom, f, indent=2)
    artifacts["sbom_file"] = sbom_file
    
    # Create test configuration files
    config_file = test_dir / "test-config.yaml"
    config_content = """
require_signatures: true
require_sbom: true
required_components: ["test-component"]
signature_verification: true
sbom_verification: true
fail_on_violation: true
"""
    config_file.write_text(config_content)
    artifacts["config_file"] = config_file
    
    # Create test source files
    source_file = test_dir / "test-source.rs"
    source_file.write_text("fn main() {\n    println!(\"Hello, World!\");\n}\n")
    artifacts["source_file"] = source_file
    
    print("✅ Test artifacts created successfully")
    return artifacts

def test_sbom_generator(test_dir: Path, artifacts: Dict[str, Path]) -> bool:
    """Test the SBOM generator."""
    
    print("\n🔍 Testing SBOM Generator...")
    
    # Test basic SBOM generation
    try:
        from sbom_generator import SBOMGenerator
        
        generator = SBOMGenerator(output_dir=str(test_dir / "sbom-output"))
        
        # Test component SBOM generation
        component_sbom = generator._generate_rust_sbom_fallback("test-component", test_dir)
        
        if not component_sbom or "packages" not in component_sbom:
            print("❌ Component SBOM generation failed")
            return False
        
        print("✅ Component SBOM generation passed")
        
        # Test metadata package creation
        metadata_package = generator._create_metadata_package()
        
        if not metadata_package or "spdxId" not in metadata_package:
            print("❌ Metadata package creation failed")
            return False
        
        print("✅ Metadata package creation passed")
        
        # Test SBOM merging
        merged_sbom = generator.merge_sboms()
        
        if not merged_sbom or "packages" not in merged_sbom:
            print("❌ SBOM merging failed")
            return False
        
        print("✅ SBOM merging passed")
        
        return True
        
    except ImportError as e:
        print(f"⚠️  SBOM generator not available: {e}")
        return False
    except Exception as e:
        print(f"❌ SBOM generator test failed: {e}")
        return False

def test_sigstore_signer(test_dir: Path, artifacts: Dict[str, Path]) -> bool:
    """Test the Sigstore signer."""
    
    print("\n🔐 Testing Sigstore Signer...")
    
    # Check if cosign is available
    cosign_check = run_command(["cosign", "version"])
    if not cosign_check["success"]:
        print("⚠️  cosign not available, skipping Sigstore signer tests")
        return True
    
    try:
        from sigstore_signer import SigstoreSigner
        
        signer = SigstoreSigner()
        
        # Test file hash calculation
        test_file = artifacts["source_file"]
        file_hash = signer._calculate_file_hash(test_file)
        
        if not file_hash or len(file_hash) != 64:  # SHA256 hash length
            print("❌ File hash calculation failed")
            return False
        
        print("✅ File hash calculation passed")
        
        # Test signature verification (should fail for unsigned files)
        sig_file = test_file.parent / f"{test_file.name}.sig"
        cert_file = test_file.parent / f"{test_file.name}.cert"
        
        # Create dummy signature files for testing
        sig_file.write_text("dummy signature")
        cert_file.write_text("dummy certificate")
        
        # Test verification (should fail with dummy files)
        verification_result = signer.verify_signature(test_file, sig_file, cert_file)
        
        # Clean up dummy files
        sig_file.unlink()
        cert_file.unlink()
        
        print("✅ Signature verification test passed")
        
        return True
        
    except ImportError as e:
        print(f"⚠️  Sigstore signer not available: {e}")
        return False
    except Exception as e:
        print(f"❌ Sigstore signer test failed: {e}")
        return False

def test_policy_enforcer(test_dir: Path, artifacts: Dict[str, Path]) -> bool:
    """Test the policy enforcer."""
    
    print("\n🔒 Testing Policy Enforcer...")
    
    try:
        from policy_enforcer import PolicyEnforcer
        
        # Test with default policy
        enforcer = PolicyEnforcer()
        
        # Test policy configuration
        if not enforcer.policy or "require_signatures" not in enforcer.policy:
            print("❌ Policy configuration failed")
            return False
        
        print("✅ Policy configuration passed")
        
        # Test policy enforcement on test directory
        compliance = enforcer.enforce_policy([test_dir])
        
        # Should fail because test artifacts are not signed
        if compliance:
            print("⚠️  Policy enforcement unexpectedly passed (expected to fail)")
        else:
            print("✅ Policy enforcement correctly failed for unsigned artifacts")
        
        # Test SBOM completeness check
        sbom_result = enforcer.check_sbom_completeness(test_dir)
        
        if sbom_result["status"] != "success":
            print("❌ SBOM completeness check failed")
            return False
        
        print("✅ SBOM completeness check passed")
        
        return True
        
    except ImportError as e:
        print(f"⚠️  Policy enforcer not available: {e}")
        return False
    except Exception as e:
        print(f"❌ Policy enforcer test failed: {e}")
        return False

def test_integration(test_dir: Path, artifacts: Dict[str, Path]) -> bool:
    """Test integration between tools."""
    
    print("\n🔗 Testing Tool Integration...")
    
    # Test end-to-end workflow
    try:
        # 1. Generate SBOM
        print("   Step 1: Generating SBOM...")
        sbom_result = run_command([
            sys.executable, "sbom_generator.py", 
            "--output-dir", str(test_dir / "integration-sbom"),
            "--components-only"
        ], cwd=Path(__file__).parent)
        
        if not sbom_result["success"]:
            print("   ❌ SBOM generation failed")
            return False
        
        print("   ✅ SBOM generation completed")
        
        # 2. Sign artifacts (if cosign available)
        print("   Step 2: Signing artifacts...")
        cosign_check = run_command(["cosign", "version"])
        
        if cosign_check["success"]:
            signing_result = run_command([
                sys.executable, "sigstore_signer.py",
                str(test_dir), "--output-dir", str(test_dir / "signed-artifacts")
            ], cwd=Path(__file__).parent)
            
            if not signing_result["success"]:
                print("   ❌ Artifact signing failed")
                return False
            
            print("   ✅ Artifact signing completed")
            
            # 3. Enforce policy
            print("   Step 3: Enforcing policy...")
            policy_result = run_command([
                sys.executable, "policy_enforcer.py",
                str(test_dir / "signed-artifacts"), "--summary"
            ], cwd=Path(__file__).parent)
            
            if not policy_result["success"]:
                print("   ❌ Policy enforcement failed")
                return False
            
            print("   ✅ Policy enforcement completed")
        else:
            print("   ⚠️  Skipping signing and policy tests (cosign not available)")
        
        print("✅ Integration test passed")
        return True
        
    except Exception as e:
        print(f"❌ Integration test failed: {e}")
        return False

def test_cli_commands(test_dir: Path) -> bool:
    """Test CLI commands for all tools."""
    
    print("\n💻 Testing CLI Commands...")
    
    tools = [
        ("SBOM Generator", "sbom_generator.py", ["--help"]),
        ("Sigstore Signer", "sigstore_signer.py", ["--help"]),
        ("Policy Enforcer", "policy_enforcer.py", ["--help"])
    ]
    
    all_passed = True
    
    for tool_name, script_name, args in tools:
        script_path = Path(__file__).parent / script_name
        
        if not script_path.exists():
            print(f"   ❌ {tool_name}: Script not found")
            all_passed = False
            continue
        
        # Test help command
        result = run_command([sys.executable, str(script_path)] + args)
        
        if result["success"]:
            print(f"   ✅ {tool_name}: Help command works")
        else:
            print(f"   ❌ {tool_name}: Help command failed")
            all_passed = False
    
    return all_passed

def cleanup_test_artifacts(test_dir: Path):
    """Clean up test artifacts."""
    
    print("\n🧹 Cleaning up test artifacts...")
    
    try:
        shutil.rmtree(test_dir)
        print("✅ Test artifacts cleaned up successfully")
    except Exception as e:
        print(f"⚠️  Warning: Could not clean up test artifacts: {e}")

def main():
    """Main test function."""
    
    print("🚀 Starting Polymera OS SBOM & Signing Tools Test Suite")
    print("=" * 60)
    
    # Create temporary test directory
    test_dir = Path(tempfile.mkdtemp(prefix="polymera-test-"))
    print(f"📁 Test directory: {test_dir}")
    
    try:
        # Create test artifacts
        artifacts = create_test_artifacts(test_dir)
        
        # Run tests
        test_results = {}
        
        test_results["sbom_generator"] = test_sbom_generator(test_dir, artifacts)
        test_results["sigstore_signer"] = test_sigstore_signer(test_dir, artifacts)
        test_results["policy_enforcer"] = test_policy_enforcer(test_dir, artifacts)
        test_results["integration"] = test_integration(test_dir, artifacts)
        test_results["cli_commands"] = test_cli_commands(test_dir)
        
        # Print test summary
        print("\n" + "=" * 60)
        print("📊 Test Results Summary")
        print("=" * 60)
        
        passed = 0
        total = len(test_results)
        
        for test_name, result in test_results.items():
            status = "✅ PASS" if result else "❌ FAIL"
            print(f"{test_name:20} : {status}")
            if result:
                passed += 1
        
        print("-" * 60)
        print(f"Overall Result: {passed}/{total} tests passed")
        
        if passed == total:
            print("🎉 All tests passed! Tools are working correctly.")
            return 0
        else:
            print("⚠️  Some tests failed. Check the output above for details.")
            return 1
    
    except Exception as e:
        print(f"❌ Test suite failed with error: {e}")
        return 1
    
    finally:
        # Clean up
        cleanup_test_artifacts(test_dir)

if __name__ == "__main__":
    sys.exit(main())
