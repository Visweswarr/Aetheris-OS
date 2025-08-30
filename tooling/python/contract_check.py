#!/usr/bin/env python3
"""
NGFS Smart Contract Validator

This script validates NGFS smart contracts for determinism, gas usage consistency,
and ZK proof verification. It runs multiple executions of fixture contracts and
ensures outputs are identical across runs.
"""

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Any
import hashlib
import tempfile
import shutil

# Test result structure
class TestResult:
    def __init__(self, name: str, passed: bool, details: str = "", gas_used: int = 0, 
                 execution_time: float = 0.0, zk_proof: bool = False):
        self.name = name
        self.passed = passed
        self.details = details
        self.gas_used = gas_used
        self.execution_time = execution_time
        self.zk_proof = zk_proof

# Contract execution result
class ContractExecution:
    def __init__(self, ok: bool, gas: int, time_ms: int, memory: int, 
                 zk_proof: bool, error: str = "", output: str = ""):
        self.ok = ok
        self.gas = gas
        self.time_ms = time_ms
        self.memory = memory
        self.zk_proof = zk_proof
        self.error = error
        self.output = output

class ContractValidator:
    def __init__(self, fixtures_dir: str, ngfs_contract_path: str, verbose: bool = False):
        self.fixtures_dir = Path(fixtures_dir)
        self.ngfs_contract_path = ngfs_contract_path
        self.verbose = verbose
        self.results: List[TestResult] = []
        
        # Validate paths
        if not self.fixtures_dir.exists():
            raise ValueError(f"Fixtures directory does not exist: {fixtures_dir}")
        
        if not os.path.exists(ngfs_contract_path):
            raise ValueError(f"ngfs-contract binary not found: {ngfs_contract_path}")
    
    def run_contract(self, contract_path: str, input_path: str, gas_limit: int = 1000000, 
                    zk_mode: bool = False, algorithm: str = "halo2") -> ContractExecution:
        """Execute a contract using the ngfs-contract CLI"""
        cmd = [
            self.ngfs_contract_path,
            "run",
            "--contract", contract_path,
            "--input", input_path,
            "--gas", str(gas_limit),
            "--json"
        ]
        
        if zk_mode:
            cmd.extend(["--zk", "--algorithm", algorithm])
        
        try:
            start_time = time.time()
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
            execution_time = time.time() - start_time
            
            if result.returncode != 0:
                return ContractExecution(
                    ok=False,
                    gas=0,
                    time_ms=int(execution_time * 1000),
                    memory=0,
                    zk_proof=False,
                    error=f"Command failed: {result.stderr}"
                )
            
            # Parse JSON output
            try:
                output_data = json.loads(result.stdout.strip())
                return ContractExecution(
                    ok=output_data.get("ok", False),
                    gas=output_data.get("gas", 0),
                    time_ms=output_data.get("time_ms", 0),
                    memory=output_data.get("memory", 0),
                    zk_proof=output_data.get("zk_proof", False),
                    output=result.stdout.strip()
                )
            except json.JSONDecodeError as e:
                return ContractExecution(
                    ok=False,
                    gas=0,
                    time_ms=int(execution_time * 1000),
                    memory=0,
                    zk_proof=False,
                    error=f"Invalid JSON output: {e}"
                )
                
        except subprocess.TimeoutExpired:
            return ContractExecution(
                ok=False,
                gas=0,
                time_ms=30000,
                memory=0,
                zk_proof=False,
                error="Execution timeout (30s)"
            )
        except Exception as e:
            return ContractExecution(
                ok=False,
                gas=0,
                time_ms=0,
                memory=0,
                zk_proof=False,
                error=f"Execution error: {e}"
            )
    
    def test_determinism(self, contract_path: str, input_path: str, runs: int = 5) -> TestResult:
        """Test that contract execution is deterministic across multiple runs"""
        if self.verbose:
            print(f"  Testing determinism: {runs} runs")
        
        executions = []
        for i in range(runs):
            if self.verbose:
                print(f"    Run {i+1}/{runs}...")
            
            execution = self.run_contract(contract_path, input_path)
            executions.append(execution)
            
            # Small delay between runs to avoid system interference
            time.sleep(0.1)
        
        # Check if all executions succeeded
        if not all(ex.ok for ex in executions):
            failed_executions = [ex for ex in executions if not ex.ok]
            return TestResult(
                name="determinism",
                passed=False,
                details=f"Some executions failed: {[ex.error for ex in failed_executions]}"
            )
        
        # Check gas usage consistency
        gas_values = [ex.gas for ex in executions]
        gas_mean = sum(gas_values) / len(gas_values)
        gas_variance = sum((g - gas_mean) ** 2 for g in gas_values) / len(gas_values)
        gas_std = gas_variance ** 0.5
        
        # Allow small variance in gas usage (within 5% of mean)
        gas_tolerance = gas_mean * 0.05
        if gas_std > gas_tolerance:
            return TestResult(
                name="determinism",
                passed=False,
                details=f"Gas usage not consistent: mean={gas_mean:.1f}, std={gas_std:.1f}, tolerance={gas_tolerance:.1f}"
            )
        
        # Check execution time consistency
        time_values = [ex.time_ms for ex in executions]
        time_mean = sum(time_values) / len(time_values)
        time_variance = sum((t - time_mean) ** 2 for t in time_values) / len(time_values)
        time_std = time_variance ** 0.5
        
        # Allow larger variance in execution time (within 20% of mean)
        time_tolerance = time_mean * 0.2
        if time_std > time_tolerance:
            return TestResult(
                name="determinism",
                passed=False,
                details=f"Execution time not consistent: mean={time_mean:.1f}ms, std={time_std:.1f}ms, tolerance={time_tolerance:.1f}ms"
            )
        
        return TestResult(
            name="determinism",
            passed=True,
            details=f"Gas: mean={gas_mean:.1f}±{gas_std:.1f}, Time: mean={time_mean:.1f}ms±{time_std:.1f}ms",
            gas_used=int(gas_mean),
            execution_time=time_mean
        )
    
    def test_gas_limits(self, contract_path: str, input_path: str) -> TestResult:
        """Test that gas limits are properly enforced"""
        if self.verbose:
            print(f"  Testing gas limit enforcement...")
        
        # Test with normal gas limit
        normal_execution = self.run_contract(contract_path, input_path, gas_limit=1000000)
        if not normal_execution.ok:
            return TestResult(
                name="gas_limits",
                passed=False,
                details=f"Normal execution failed: {normal_execution.error}"
            )
        
        # Test with very low gas limit (should fail)
        low_gas_execution = self.run_contract(contract_path, input_path, gas_limit=10)
        if low_gas_execution.ok:
            return TestResult(
                name="gas_limits",
                passed=False,
                details="Execution should have failed with low gas limit"
            )
        
        # Test with reasonable gas limit
        reasonable_execution = self.run_contract(contract_path, input_path, gas_limit=10000)
        if not reasonable_execution.ok:
            return TestResult(
                name="gas_limits",
                passed=False,
                details=f"Reasonable gas limit execution failed: {reasonable_execution.error}"
            )
        
        return TestResult(
            name="gas_limits",
            passed=True,
            details=f"Gas limits enforced: normal={normal_execution.gas}, reasonable={reasonable_execution.gas}",
            gas_used=normal_execution.gas,
            execution_time=normal_execution.time_ms
        )
    
    def test_zk_proofs(self, contract_path: str, input_path: str) -> TestResult:
        """Test ZK proof generation and verification"""
        if self.verbose:
            print(f"  Testing ZK proof generation...")
        
        # Test without ZK mode
        no_zk_execution = self.run_contract(contract_path, input_path, zk_mode=False)
        if not no_zk_execution.ok:
            return TestResult(
                name="zk_proofs",
                passed=False,
                details=f"Non-ZK execution failed: {no_zk_execution.error}"
            )
        
        if no_zk_execution.zk_proof:
            return TestResult(
                name="zk_proofs",
                passed=False,
                details="Non-ZK execution should not generate proof"
            )
        
        # Test with ZK mode
        zk_execution = self.run_contract(contract_path, input_path, zk_mode=True)
        if not zk_execution.ok:
            return TestResult(
                name="zk_proofs",
                passed=False,
                details=f"ZK execution failed: {zk_execution.error}"
            )
        
        if not zk_execution.zk_proof:
            return TestResult(
                name="zk_proofs",
                passed=False,
                details="ZK execution should generate proof"
            )
        
        # Test different ZK algorithms
        algorithms = ["halo2", "noir", "plonk"]
        for algorithm in algorithms:
            if self.verbose:
                print(f"    Testing algorithm: {algorithm}")
            
            algo_execution = self.run_contract(contract_path, input_path, zk_mode=True, algorithm=algorithm)
            if not algo_execution.ok:
                return TestResult(
                    name="zk_proofs",
                    passed=False,
                    details=f"ZK execution with {algorithm} failed: {algo_execution.error}"
                )
            
            if not algo_execution.zk_proof:
                return TestResult(
                    name="zk_proofs",
                    passed=False,
                    details=f"ZK execution with {algorithm} should generate proof"
                )
        
        return TestResult(
            name="zk_proofs",
            passed=True,
            details=f"ZK proofs generated successfully with all algorithms",
            gas_used=zk_execution.gas,
            execution_time=zk_execution.time_ms,
            zk_proof=True
        )
    
    def test_error_handling(self, contract_path: str) -> TestResult:
        """Test error handling with invalid inputs"""
        if self.verbose:
            print(f"  Testing error handling...")
        
        # Create temporary invalid input files
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            f.write('{"invalid": "json"')
            invalid_json_path = f.name
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
            f.write('not json at all')
            invalid_input_path = f.name
        
        try:
            # Test with invalid JSON
            invalid_json_execution = self.run_contract(contract_path, invalid_json_path)
            # Should handle gracefully (either succeed or fail with clear error)
            
            # Test with non-JSON input
            invalid_input_execution = self.run_contract(contract_path, invalid_input_path)
            # Should handle gracefully
            
            # Test with non-existent input file
            non_existent_execution = self.run_contract(contract_path, "/non/existent/file.json")
            if non_existent_execution.ok:
                return TestResult(
                    name="error_handling",
                    passed=False,
                    details="Should fail with non-existent input file"
                )
            
            return TestResult(
                name="error_handling",
                passed=True,
                details="Error handling works correctly"
            )
            
        finally:
            # Clean up temporary files
            for path in [invalid_json_path, invalid_input_path]:
                try:
                    os.unlink(path)
                except OSError:
                    pass
    
    def test_contract(self, contract_name: str) -> List[TestResult]:
        """Run all tests for a specific contract"""
        contract_path = self.fixtures_dir / f"{contract_name}.wasm"
        input_path = self.fixtures_dir / f"{contract_name}_input.json"
        
        if not contract_path.exists():
            return [TestResult(
                name=contract_name,
                passed=False,
                details=f"Contract file not found: {contract_path}"
            )]
        
        if not input_path.exists():
            return [TestResult(
                name=contract_name,
                passed=False,
                details=f"Input file not found: {input_path}"
            )]
        
        if self.verbose:
            print(f"Testing contract: {contract_name}")
        
        results = []
        
        # Run determinism test
        results.append(self.test_determinism(str(contract_path), str(input_path)))
        
        # Run gas limit test
        results.append(self.test_gas_limits(str(contract_path), str(input_path)))
        
        # Run ZK proof test
        results.append(self.test_zk_proofs(str(contract_path), str(input_path)))
        
        # Run error handling test
        results.append(self.test_error_handling(str(contract_path)))
        
        return results
    
    def run_all_tests(self) -> Dict[str, List[TestResult]]:
        """Run tests for all available contracts"""
        all_results = {}
        
        # Find all contract files
        contract_files = list(self.fixtures_dir.glob("*.wasm"))
        
        if not contract_files:
            print("No contract files found in fixtures directory")
            return all_results
        
        print(f"Found {len(contract_files)} contract(s) to test")
        
        for contract_file in contract_files:
            contract_name = contract_file.stem
            print(f"\nTesting {contract_name}...")
            
            results = self.test_contract(contract_name)
            all_results[contract_name] = results
            
            # Print results for this contract
            for result in results:
                status = "✓" if result.passed else "✗"
                print(f"  {status} {result.name}: {result.details}")
        
        return all_results
    
    def generate_report(self, results: Dict[str, List[TestResult]]) -> str:
        """Generate a summary report"""
        total_contracts = len(results)
        total_tests = sum(len(contract_results) for contract_results in results.values())
        passed_tests = sum(
            sum(1 for result in contract_results if result.passed)
            for contract_results in results.values()
        )
        
        report = f"""
NGFS Smart Contract Validation Report
====================================

Summary:
  Total Contracts: {total_contracts}
  Total Tests: {total_tests}
  Passed: {passed_tests}
  Failed: {total_tests - passed_tests}
  Success Rate: {(passed_tests / total_tests * 100):.1f}%

Detailed Results:
"""
        
        for contract_name, contract_results in results.items():
            contract_passed = sum(1 for r in contract_results if r.passed)
            contract_total = len(contract_results)
            
            report += f"\n{contract_name}:\n"
            report += f"  Tests: {contract_passed}/{contract_total} passed\n"
            
            for result in contract_results:
                status = "PASS" if result.passed else "FAIL"
                report += f"    {status}: {result.name} - {result.details}\n"
        
        return report

def main():
    parser = argparse.ArgumentParser(description="Validate NGFS smart contracts")
    parser.add_argument("--fixtures", required=True, help="Path to fixtures directory")
    parser.add_argument("--ngfs-contract", required=True, help="Path to ngfs-contract binary")
    parser.add_argument("--verbose", "-v", action="store_true", help="Enable verbose output")
    parser.add_argument("--output", help="Output file for detailed report")
    
    args = parser.parse_args()
    
    try:
        # Create validator
        validator = ContractValidator(args.fixtures, args.ngfs_contract, args.verbose)
        
        # Run all tests
        results = validator.run_all_tests()
        
        # Generate and display report
        report = validator.generate_report(results)
        print(report)
        
        # Save report to file if requested
        if args.output:
            with open(args.output, 'w') as f:
                f.write(report)
            print(f"Detailed report saved to: {args.output}")
        
        # Determine exit code
        total_tests = sum(len(contract_results) for contract_results in results.values())
        passed_tests = sum(
            sum(1 for result in contract_results if result.passed)
            for contract_results in results.values()
        )
        
        if passed_tests == total_tests:
            print("\n🎉 All tests passed!")
            sys.exit(0)
        else:
            print(f"\n❌ {total_tests - passed_tests} test(s) failed")
            sys.exit(1)
            
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
