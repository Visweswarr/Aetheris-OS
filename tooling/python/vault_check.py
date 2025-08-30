#!/usr/bin/env python3
"""
NGFS Vault Capability Token Validator

This script tests CapToken enforcement by attempting allowed/denied operations
and ensuring proper access control is maintained.
"""

import argparse
import json
import os
import sys
import tempfile
import subprocess
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any
from dataclasses import dataclass

@dataclass
class VaultTestResult:
    """Result of a vault operation test"""
    operation: str
    expected_success: bool
    actual_success: bool
    error_message: Optional[str] = None
    passed: bool = False

@dataclass
class VaultTestSuite:
    """Complete test suite results"""
    total_tests: int
    passed_tests: int
    failed_tests: int
    results: List[VaultTestResult]
    overall_success: bool = False

class VaultCapabilityTester:
    """Tests NGFS vault capability token enforcement"""
    
    def __init__(self, fixtures_dir: str, cap_token_file: str):
        self.fixtures_dir = fixtures_dir
        self.cap_token_file = cap_token_file
        self.ngfs_vault_path = "ngfs-vault"  # Assume in PATH
        self.test_results: List[VaultTestResult] = []
        
    def run_test_suite(self) -> VaultTestSuite:
        """Run the complete test suite"""
        print("🔐 NGFS Vault Capability Token Test Suite")
        print("=" * 50)
        
        # Test basic operations with valid capabilities
        self.test_valid_operations()
        
        # Test operations with insufficient capabilities
        self.test_insufficient_capabilities()
        
        # Test operations with expired tokens
        self.test_expired_tokens()
        
        # Test operations with invalid tokens
        self.test_invalid_tokens()
        
        # Test edge cases and security boundaries
        self.test_security_boundaries()
        
        # Calculate results
        total_tests = len(self.test_results)
        passed_tests = sum(1 for r in self.test_results if r.passed)
        failed_tests = total_tests - passed_tests
        overall_success = failed_tests == 0
        
        return VaultTestSuite(
            total_tests=total_tests,
            passed_tests=passed_tests,
            failed_tests=failed_tests,
            results=self.test_results,
            overall_success=overall_success
        )
    
    def test_valid_operations(self):
        """Test operations that should succeed with valid capabilities"""
        print("\n✅ Testing Valid Operations...")
        
        # Test list operation with list capability
        self.test_operation(
            operation="list",
            expected_success=True,
            description="List entries with list capability"
        )
        
        # Test add operation with write capability
        self.test_operation(
            operation="add",
            expected_success=True,
            description="Add entry with write capability",
            extra_args=["--id", "test-key", "--kind", "key", "--file", "test_file.txt"]
        )
        
        # Test show operation with read capability
        self.test_operation(
            operation="show",
            expected_success=True,
            description="Show entry with read capability",
            extra_args=["--id", "test-key"]
        )
        
        # Test delete operation with delete capability
        self.test_operation(
            operation="delete",
            expected_success=True,
            description="Delete entry with delete capability",
            extra_args=["--id", "test-key"]
        )
    
    def test_insufficient_capabilities(self):
        """Test operations that should fail due to insufficient capabilities"""
        print("\n❌ Testing Insufficient Capabilities...")
        
        # Test add operation without write capability
        self.test_operation(
            operation="add",
            expected_success=False,
            description="Add entry without write capability",
            extra_args=["--id", "test-key", "--kind", "key", "--file", "test_file.txt"],
            use_restricted_token=True
        )
        
        # Test show operation without read capability
        self.test_operation(
            operation="show",
            expected_success=False,
            description="Show entry without read capability",
            extra_args=["--id", "test-key"],
            use_restricted_token=True
        )
        
        # Test delete operation without delete capability
        self.test_operation(
            operation="delete",
            expected_success=False,
            description="Delete entry without delete capability",
            extra_args=["--id", "test-key"],
            use_restricted_token=True
        )
        
        # Test list operation without list capability
        self.test_operation(
            operation="list",
            expected_success=False,
            description="List entries without list capability",
            use_restricted_token=True
        )
    
    def test_expired_tokens(self):
        """Test operations with expired capability tokens"""
        print("\n⏰ Testing Expired Tokens...")
        
        # Test with expired token
        self.test_operation(
            operation="list",
            expected_success=False,
            description="List entries with expired token",
            use_expired_token=True
        )
    
    def test_invalid_tokens(self):
        """Test operations with invalid capability tokens"""
        print("\n🚫 Testing Invalid Tokens...")
        
        # Test with malformed token
        self.test_operation(
            operation="list",
            expected_success=False,
            description="List entries with malformed token",
            use_malformed_token=True
        )
        
        # Test with missing token
        self.test_operation(
            operation="list",
            expected_success=False,
            description="List entries without token",
            use_missing_token=True
        )
    
    def test_security_boundaries(self):
        """Test security boundary conditions"""
        print("\n🛡️ Testing Security Boundaries...")
        
        # Test with empty capability list
        self.test_operation(
            operation="list",
            expected_success=False,
            description="List entries with empty capabilities",
            use_empty_capabilities=True
        )
        
        # Test with invalid operation type
        self.test_operation(
            operation="invalid_op",
            expected_success=False,
            description="Invalid operation type"
        )
        
        # Test with very long entry ID
        long_id = "a" * 1000
        self.test_operation(
            operation="show",
            expected_success=False,
            description="Show entry with very long ID",
            extra_args=["--id", long_id]
        )
    
    def test_operation(
        self,
        operation: str,
        expected_success: bool,
        description: str,
        extra_args: Optional[List[str]] = None,
        use_restricted_token: bool = False,
        use_expired_token: bool = False,
        use_malformed_token: bool = False,
        use_missing_token: bool = False,
        use_empty_capabilities: bool = False
    ) -> None:
        """Test a single vault operation"""
        if extra_args is None:
            extra_args = []
        
        # Prepare command
        cmd = [self.ngfs_vault_path, operation, "--json"]
        
        # Add extra arguments
        cmd.extend(extra_args)
        
        # Handle different token scenarios
        if use_missing_token:
            # Don't add --cap flag
            pass
        elif use_expired_token:
            cmd.extend(["--cap", self.get_expired_token_path()])
        elif use_malformed_token:
            cmd.extend(["--cap", self.get_malformed_token_path()])
        elif use_restricted_token:
            cmd.extend(["--cap", self.get_restricted_token_path()])
        elif use_empty_capabilities:
            cmd.extend(["--cap", self.get_empty_capabilities_token_path()])
        else:
            cmd.extend(["--cap", self.cap_token_file])
        
        # Create test file if needed
        test_file_created = False
        if "--file" in extra_args:
            file_index = extra_args.index("--file")
            if file_index + 1 < len(extra_args):
                file_path = extra_args[file_index + 1]
                if not os.path.exists(file_path):
                    self.create_test_file(file_path)
                    test_file_created = True
        
        try:
            # Run command
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=30
            )
            
            # Parse result
            actual_success = result.returncode == 0
            error_message = None
            
            if not actual_success and result.stderr:
                error_message = result.stderr.strip()
            
            # Check if result matches expectation
            passed = actual_success == expected_success
            
            # Create test result
            test_result = VaultTestResult(
                operation=operation,
                expected_success=expected_success,
                actual_success=actual_success,
                error_message=error_message,
                passed=passed
            )
            
            self.test_results.append(test_result)
            
            # Print result
            status = "✅ PASS" if passed else "❌ FAIL"
            print(f"  {status}: {description}")
            
            if not passed:
                print(f"    Expected: {'success' if expected_success else 'failure'}")
                print(f"    Actual: {'success' if actual_success else 'failure'}")
                if error_message:
                    print(f"    Error: {error_message}")
            
        except subprocess.TimeoutExpired:
            test_result = VaultTestResult(
                operation=operation,
                expected_success=expected_success,
                actual_success=False,
                error_message="Operation timed out",
                passed=False
            )
            self.test_results.append(test_result)
            print(f"  ❌ FAIL: {description} (timeout)")
            
        except Exception as e:
            test_result = VaultTestResult(
                operation=operation,
                expected_success=expected_success,
                actual_success=False,
                error_message=str(e),
                passed=False
            )
            self.test_results.append(test_result)
            print(f"  ❌ FAIL: {description} (exception: {e})")
        
        finally:
            # Clean up test file
            if test_file_created and "--file" in extra_args:
                file_index = extra_args.index("--file")
                if file_index + 1 < len(extra_args):
                    file_path = extra_args[file_index + 1]
                    if os.path.exists(file_path):
                        os.unlink(file_path)
    
    def create_test_file(self, file_path: str) -> None:
        """Create a test file for vault operations"""
        test_content = "This is a test file for NGFS vault operations.\n"
        test_content += "It contains some sample content to test encryption.\n"
        test_content += "Generated at: " + datetime.now().isoformat() + "\n"
        
        with open(file_path, 'w') as f:
            f.write(test_content)
    
    def get_expired_token_path(self) -> str:
        """Get path to an expired capability token"""
        expired_token = {
            "version": 2,
            "issuer_did": "did:example:issuer",
            "subject_did": "did:example:subject",
            "capabilities": [
                {
                    "operation": "list",
                    "resource": "*"
                }
            ],
            "issued_at": (datetime.now() - timedelta(days=2)).isoformat(),
            "expires_at": (datetime.now() - timedelta(days=1)).isoformat(),
            "signature": "expired_signature",
            "nonce": "expired_nonce"
        }
        
        return self.create_temp_token_file(expired_token, "expired_token.json")
    
    def get_malformed_token_path(self) -> str:
        """Get path to a malformed capability token"""
        malformed_token = {
            "version": 2,
            "issuer_did": "",  # Missing issuer DID
            "subject_did": "did:example:subject",
            "capabilities": [],  # Empty capabilities
            "issued_at": datetime.now().isoformat(),
            "signature": "malformed_signature",
            "nonce": "malformed_nonce"
        }
        
        return self.create_temp_token_file(malformed_token, "malformed_token.json")
    
    def get_restricted_token_path(self) -> str:
        """Get path to a token with restricted capabilities"""
        restricted_token = {
            "version": 2,
            "issuer_did": "did:example:issuer",
            "subject_did": "did:example:subject",
            "capabilities": [
                {
                    "operation": "read",
                    "resource": "specific-entry"
                }
            ],
            "issued_at": datetime.now().isoformat(),
            "signature": "restricted_signature",
            "nonce": "restricted_nonce"
        }
        
        return self.create_temp_token_file(restricted_token, "restricted_token.json")
    
    def get_empty_capabilities_token_path(self) -> str:
        """Get path to a token with empty capabilities"""
        empty_cap_token = {
            "version": 2,
            "issuer_did": "did:example:issuer",
            "subject_did": "did:example:subject",
            "capabilities": [],
            "issued_at": datetime.now().isoformat(),
            "signature": "empty_cap_signature",
            "nonce": "empty_cap_nonce"
        }
        
        return self.create_temp_token_file(empty_cap_token, "empty_capabilities_token.json")
    
    def create_temp_token_file(self, token_data: Dict[str, Any], filename: str) -> str:
        """Create a temporary token file"""
        temp_dir = tempfile.mkdtemp(prefix="ngfs_vault_test_")
        file_path = os.path.join(temp_dir, filename)
        
        with open(file_path, 'w') as f:
            json.dump(token_data, f, indent=2)
        
        return file_path

def print_test_summary(test_suite: VaultTestSuite) -> None:
    """Print a summary of test results"""
    print("\n" + "=" * 50)
    print("📊 Test Summary")
    print("=" * 50)
    print(f"Total Tests: {test_suite.total_tests}")
    print(f"Passed: {test_suite.passed_tests} ✅")
    print(f"Failed: {test_suite.failed_tests} ❌")
    print(f"Success Rate: {(test_suite.passed_tests / test_suite.total_tests * 100):.1f}%")
    
    if test_suite.overall_success:
        print("\n🎉 All tests passed! Capability enforcement is working correctly.")
    else:
        print(f"\n⚠️  {test_suite.failed_tests} test(s) failed. Capability enforcement may have issues.")
        
        # Show failed tests
        print("\nFailed Tests:")
        for result in test_suite.results:
            if not result.passed:
                print(f"  ❌ {result.operation}: {result.error_message or 'Unknown error'}")

def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(
        description="Test NGFS vault capability token enforcement",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Test with default fixtures and capability token
  python3 vault_check.py --fixtures tests/vault/fixtures --cap token.json
  
  # Test with verbose output
  python3 vault_check.py --fixtures tests/vault/fixtures --cap token.json --verbose
  
  # Test specific operations only
  python3 vault_check.py --fixtures tests/vault/fixtures --cap token.json --operations list,show
        """
    )
    
    parser.add_argument(
        "--fixtures",
        required=True,
        help="Directory containing test fixtures"
    )
    
    parser.add_argument(
        "--cap",
        required=True,
        help="Path to capability token file"
    )
    
    parser.add_argument(
        "--operations",
        help="Comma-separated list of operations to test (default: all)"
    )
    
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Enable verbose output"
    )
    
    parser.add_argument(
        "--output",
        help="Output results to JSON file"
    )
    
    args = parser.parse_args()
    
    # Validate inputs
    if not os.path.isdir(args.fixtures):
        print(f"❌ Error: Fixtures directory '{args.fixtures}' does not exist")
        sys.exit(1)
    
    if not os.path.isfile(args.cap):
        print(f"❌ Error: Capability token file '{args.cap}' does not exist")
        sys.exit(1)
    
    # Create tester
    tester = VaultCapabilityTester(args.fixtures, args.cap)
    
    # Run test suite
    test_suite = tester.run_test_suite()
    
    # Print summary
    print_test_summary(test_suite)
    
    # Output results if requested
    if args.output:
        output_results(test_suite, args.output)
    
    # Exit with appropriate code
    if test_suite.overall_success:
        print("\n✅ Capability enforcement test suite completed successfully")
        sys.exit(0)
    else:
        print("\n❌ Capability enforcement test suite failed")
        sys.exit(1)

def output_results(test_suite: VaultTestSuite, output_file: str) -> None:
    """Output test results to JSON file"""
    output_data = {
        "timestamp": datetime.now().isoformat(),
        "test_suite": "ngfs_vault_capability_enforcement",
        "summary": {
            "total_tests": test_suite.total_tests,
            "passed_tests": test_suite.passed_tests,
            "failed_tests": test_suite.failed_tests,
            "success_rate": test_suite.passed_tests / test_suite.total_tests,
            "overall_success": test_suite.overall_success
        },
        "results": [
            {
                "operation": r.operation,
                "expected_success": r.expected_success,
                "actual_success": r.actual_success,
                "error_message": r.error_message,
                "passed": r.passed
            }
            for r in test_suite.results
        ]
    }
    
    try:
        with open(output_file, 'w') as f:
            json.dump(output_data, f, indent=2)
        print(f"\n📄 Results written to: {output_file}")
    except Exception as e:
        print(f"\n⚠️  Warning: Failed to write results to {output_file}: {e}")

if __name__ == "__main__":
    main()
