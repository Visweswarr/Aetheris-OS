#!/usr/bin/env python3
"""
Tests for the NGFS Smart Contract Validator

This module tests the contract validation functionality including
determinism checks, gas limit enforcement, ZK proof generation,
and error handling.
"""

import pytest
import tempfile
import os
import json
import subprocess
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock
import sys

# Add the parent directory to the path to import the contract_check module
sys.path.insert(0, str(Path(__file__).parent.parent))

from contract_check import (
    ContractValidator, TestResult, ContractExecution,
    ContractV1, ContractMeta, ContractCategory
)

class TestTestResult:
    """Test the TestResult class"""
    
    def test_test_result_creation(self):
        """Test creating a TestResult instance"""
        result = TestResult(
            name="test_name",
            passed=True,
            details="Test details",
            gas_used=1000,
            execution_time=50.5,
            zk_proof=True
        )
        
        assert result.name == "test_name"
        assert result.passed is True
        assert result.details == "Test details"
        assert result.gas_used == 1000
        assert result.execution_time == 50.5
        assert result.zk_proof is True
    
    def test_test_result_defaults(self):
        """Test TestResult with default values"""
        result = TestResult("test_name", True)
        
        assert result.name == "test_name"
        assert result.passed is True
        assert result.details == ""
        assert result.gas_used == 0
        assert result.execution_time == 0.0
        assert result.zk_proof is False

class TestContractExecution:
    """Test the ContractExecution class"""
    
    def test_contract_execution_creation(self):
        """Test creating a ContractExecution instance"""
        execution = ContractExecution(
            ok=True,
            gas=1500,
            time_ms=75,
            memory=2048,
            zk_proof=True,
            error="",
            output="success"
        )
        
        assert execution.ok is True
        assert execution.gas == 1500
        assert execution.time_ms == 75
        assert execution.memory == 2048
        assert execution.zk_proof is True
        assert execution.error == ""
        assert execution.output == "success"
    
    def test_contract_execution_defaults(self):
        """Test ContractExecution with default values"""
        execution = ContractExecution(True, 1000, 50, 1024)
        
        assert execution.ok is True
        assert execution.gas == 1000
        assert execution.time_ms == 50
        assert execution.memory == 1024
        assert execution.zk_proof is False
        assert execution.error == ""
        assert execution.output == ""

class TestContractValidator:
    """Test the ContractValidator class"""
    
    @pytest.fixture
    def temp_fixtures_dir(self):
        """Create a temporary fixtures directory"""
        with tempfile.TemporaryDirectory() as temp_dir:
            fixtures_dir = Path(temp_dir) / "fixtures"
            fixtures_dir.mkdir()
            
            # Create mock contract files
            (fixtures_dir / "test_contract.wasm").write_bytes(b"mock_wasm_data")
            (fixtures_dir / "test_contract_input.json").write_text('{"test": "data"}')
            
            yield fixtures_dir
    
    @pytest.fixture
    def mock_ngfs_contract_path(self):
        """Create a mock ngfs-contract binary path"""
        return "/mock/path/ngfs-contract"
    
    @pytest.fixture
    def validator(self, temp_fixtures_dir, mock_ngfs_contract_path):
        """Create a ContractValidator instance for testing"""
        with patch('os.path.exists', return_value=True):
            return ContractValidator(
                str(temp_fixtures_dir),
                mock_ngfs_contract_path,
                verbose=False
            )
    
    def test_validator_creation(self, temp_fixtures_dir, mock_ngfs_contract_path):
        """Test ContractValidator creation"""
        with patch('os.path.exists', return_value=True):
            validator = ContractValidator(
                str(temp_fixtures_dir),
                mock_ngfs_contract_path,
                verbose=False
            )
            
            assert validator.fixtures_dir == temp_fixtures_dir
            assert validator.ngfs_contract_path == mock_ngfs_contract_path
            assert validator.verbose is False
    
    def test_validator_creation_fixtures_not_found(self, mock_ngfs_contract_path):
        """Test ContractValidator creation with non-existent fixtures directory"""
        with pytest.raises(ValueError, match="Fixtures directory does not exist"):
            ContractValidator(
                "/non/existent/fixtures",
                mock_ngfs_contract_path,
                verbose=False
            )
    
    def test_validator_creation_binary_not_found(self, temp_fixtures_dir):
        """Test ContractValidator creation with non-existent binary"""
        with patch('os.path.exists', side_effect=[True, False]):
            with pytest.raises(ValueError, match="ngfs-contract binary not found"):
                ContractValidator(
                    str(temp_fixtures_dir),
                    "/non/existent/binary",
                    verbose=False
                )
    
    @patch('subprocess.run')
    def test_run_contract_success(self, mock_run, validator):
        """Test successful contract execution"""
        # Mock successful subprocess execution
        mock_result = Mock()
        mock_result.returncode = 0
        mock_result.stdout = '{"ok": true, "gas": 1000, "time_ms": 50, "memory": 1024, "zk_proof": false}'
        mock_result.stderr = ""
        mock_run.return_value = mock_result
        
        execution = validator.run_contract(
            "test_contract.wasm",
            "test_input.json",
            gas_limit=10000,
            zk_mode=False
        )
        
        assert execution.ok is True
        assert execution.gas == 1000
        assert execution.time_ms == 50
        assert execution.memory == 1024
        assert execution.zk_proof is False
        assert execution.error == ""
    
    @patch('subprocess.run')
    def test_run_contract_failure(self, mock_run, validator):
        """Test failed contract execution"""
        # Mock failed subprocess execution
        mock_result = Mock()
        mock_result.returncode = 1
        mock_result.stdout = ""
        mock_result.stderr = "Contract execution failed"
        mock_run.return_value = mock_result
        
        execution = validator.run_contract(
            "test_contract.wasm",
            "test_input.json"
        )
        
        assert execution.ok is False
        assert execution.gas == 0
        assert execution.error == "Command failed: Contract execution failed"
    
    @patch('subprocess.run')
    def test_run_contract_timeout(self, mock_run, validator):
        """Test contract execution timeout"""
        # Mock timeout exception
        mock_run.side_effect = subprocess.TimeoutExpired("ngfs-contract", 30)
        
        execution = validator.run_contract(
            "test_contract.wasm",
            "test_input.json"
        )
        
        assert execution.ok is False
        assert execution.time_ms == 30000
        assert execution.error == "Execution timeout (30s)"
    
    @patch('subprocess.run')
    def test_run_contract_invalid_json(self, mock_run, validator):
        """Test contract execution with invalid JSON output"""
        # Mock successful execution but invalid JSON
        mock_result = Mock()
        mock_result.returncode = 0
        mock_result.stdout = "invalid json"
        mock_result.stderr = ""
        mock_run.return_value = mock_result
        
        execution = validator.run_contract(
            "test_contract.wasm",
            "test_input.json"
        )
        
        assert execution.ok is False
        assert "Invalid JSON output" in execution.error
    
    @patch('subprocess.run')
    def test_run_contract_with_zk(self, mock_run, validator):
        """Test contract execution with ZK mode enabled"""
        # Mock successful ZK execution
        mock_result = Mock()
        mock_result.returncode = 0
        mock_result.stdout = '{"ok": true, "gas": 2000, "time_ms": 100, "memory": 2048, "zk_proof": true}'
        mock_result.stderr = ""
        mock_run.return_value = mock_result
        
        execution = validator.run_contract(
            "test_contract.wasm",
            "test_input.json",
            gas_limit=10000,
            zk_mode=True,
            algorithm="halo2"
        )
        
        assert execution.ok is True
        assert execution.zk_proof is True
        
        # Verify the command included ZK flags
        mock_run.assert_called_once()
        call_args = mock_run.call_args[0][0]
        assert "--zk" in call_args
        assert "--algorithm" in call_args
        assert "halo2" in call_args
    
    def test_test_determinism_success(self, validator):
        """Test successful determinism test"""
        with patch.object(validator, 'run_contract') as mock_run:
            # Mock consistent execution results
            mock_execution = ContractExecution(
                ok=True, gas=1000, time_ms=50, memory=1024, zk_proof=False
            )
            mock_run.return_value = mock_execution
            
            result = validator.test_determinism(
                "test_contract.wasm",
                "test_input.json",
                runs=3
            )
            
            assert result.passed is True
            assert result.name == "determinism"
            assert "Gas: mean=1000.0±0.0" in result.details
            assert "Time: mean=50.0ms±0.0ms" in result.details
    
    def test_test_determinism_failure(self, validator):
        """Test failed determinism test due to execution failures"""
        with patch.object(validator, 'run_contract') as mock_run:
            # Mock failed execution
            mock_execution = ContractExecution(
                ok=False, gas=0, time_ms=0, memory=0, zk_proof=False,
                error="Execution failed"
            )
            mock_run.return_value = mock_execution
            
            result = validator.test_determinism(
                "test_contract.wasm",
                "test_input.json",
                runs=3
            )
            
            assert result.passed is False
            assert "Some executions failed" in result.details
    
    def test_test_determinism_gas_variance(self, validator):
        """Test determinism test failure due to gas usage variance"""
        with patch.object(validator, 'run_contract') as mock_run:
            # Mock executions with high gas variance
            executions = [
                ContractExecution(ok=True, gas=1000, time_ms=50, memory=1024, zk_proof=False),
                ContractExecution(ok=True, gas=2000, time_ms=50, memory=1024, zk_proof=False),
                ContractExecution(ok=True, gas=3000, time_ms=50, memory=1024, zk_proof=False),
            ]
            mock_run.side_effect = executions
            
            result = validator.test_determinism(
                "test_contract.wasm",
                "test_input.json",
                runs=3
            )
            
            assert result.passed is False
            assert "Gas usage not consistent" in result.details
    
    def test_test_gas_limits_success(self, validator):
        """Test successful gas limit enforcement test"""
        with patch.object(validator, 'run_contract') as mock_run:
            # Mock successful executions with different gas limits
            mock_run.side_effect = [
                ContractExecution(ok=True, gas=1000, time_ms=50, memory=1024, zk_proof=False),
                ContractExecution(ok=False, gas=15, time_ms=5, memory=512, zk_proof=False, error="gas limit exceeded"),
                ContractExecution(ok=True, gas=500, time_ms=25, memory=768, zk_proof=False),
            ]
            
            result = validator.test_gas_limits(
                "test_contract.wasm",
                "test_input.json"
            )
            
            assert result.passed is True
            assert "Gas limits enforced" in result.details
    
    def test_test_gas_limits_normal_failure(self, validator):
        """Test gas limit test failure due to normal execution failure"""
        with patch.object(validator, 'run_contract') as mock_run:
            # Mock normal execution failure
            mock_run.return_value = ContractExecution(
                ok=False, gas=0, time_ms=0, memory=0, zk_proof=False,
                error="Normal execution failed"
            )
            
            result = validator.test_gas_limits(
                "test_contract.wasm",
                "test_input.json"
            )
            
            assert result.passed is False
            assert "Normal execution failed" in result.details
    
    def test_test_zk_proofs_success(self, validator):
        """Test successful ZK proof generation test"""
        with patch.object(validator, 'run_contract') as mock_run:
            # Mock successful executions with and without ZK
            mock_run.side_effect = [
                # Non-ZK execution
                ContractExecution(ok=True, gas=1000, time_ms=50, memory=1024, zk_proof=False),
                # ZK execution
                ContractExecution(ok=True, gas=2000, time_ms=100, memory=2048, zk_proof=True),
                # Algorithm variants
                ContractExecution(ok=True, gas=2000, time_ms=100, memory=2048, zk_proof=True),
                ContractExecution(ok=True, gas=2000, time_ms=100, memory=2048, zk_proof=True),
                ContractExecution(ok=True, gas=2000, time_ms=100, memory=2048, zk_proof=True),
            ]
            
            result = validator.test_zk_proofs(
                "test_contract.wasm",
                "test_input.json"
            )
            
            assert result.passed is True
            assert "ZK proofs generated successfully" in result.details
    
    def test_test_zk_proofs_non_zk_failure(self, validator):
        """Test ZK proof test failure due to non-ZK execution failure"""
        with patch.object(validator, 'run_contract') as mock_run:
            # Mock non-ZK execution failure
            mock_run.return_value = ContractExecution(
                ok=False, gas=0, time_ms=0, memory=0, zk_proof=False,
                error="Non-ZK execution failed"
            )
            
            result = validator.test_zk_proofs(
                "test_contract.wasm",
                "test_input.json"
            )
            
            assert result.passed is False
            assert "Non-ZK execution failed" in result.details
    
    def test_test_zk_proofs_zk_failure(self, validator):
        """Test ZK proof test failure due to ZK execution failure"""
        with patch.object(validator, 'run_contract') as mock_run:
            # Mock successful non-ZK execution but failed ZK execution
            mock_run.side_effect = [
                ContractExecution(ok=True, gas=1000, time_ms=50, memory=1024, zk_proof=False),
                ContractExecution(ok=False, gas=0, time_ms=0, memory=0, zk_proof=False, error="ZK execution failed"),
            ]
            
            result = validator.test_zk_proofs(
                "test_contract.wasm",
                "test_input.json"
            )
            
            assert result.passed is False
            assert "ZK execution failed" in result.details
    
    def test_test_error_handling_success(self, validator):
        """Test successful error handling test"""
        with patch.object(validator, 'run_contract') as mock_run:
            # Mock successful error handling
            mock_run.return_value = ContractExecution(
                ok=False, gas=0, time_ms=10, memory=0, zk_proof=False,
                error="File not found"
            )
            
            result = validator.test_error_handling("test_contract.wasm")
            
            assert result.passed is True
            assert "Error handling works correctly" in result.details
    
    def test_test_error_handling_failure(self, validator):
        """Test error handling test failure"""
        with patch.object(validator, 'run_contract') as mock_run:
            # Mock unexpected success with non-existent file
            mock_run.return_value = ContractExecution(
                ok=True, gas=1000, time_ms=50, memory=1024, zk_proof=False
            )
            
            result = validator.test_error_handling("test_contract.wasm")
            
            assert result.passed is False
            assert "Should fail with non-existent input file" in result.details
    
    def test_test_contract_success(self, validator):
        """Test successful contract testing"""
        with patch.object(validator, 'test_determinism') as mock_det, \
             patch.object(validator, 'test_gas_limits') as mock_gas, \
             patch.object(validator, 'test_zk_proofs') as mock_zk, \
             patch.object(validator, 'test_error_handling') as mock_err:
            
            # Mock successful test results
            mock_det.return_value = TestResult("determinism", True, "Passed")
            mock_gas.return_value = TestResult("gas_limits", True, "Passed")
            mock_zk.return_value = TestResult("zk_proofs", True, "Passed")
            mock_err.return_value = TestResult("error_handling", True, "Passed")
            
            results = validator.test_contract("test_contract")
            
            assert len(results) == 4
            assert all(result.passed for result in results)
    
    def test_test_contract_missing_files(self, validator):
        """Test contract testing with missing files"""
        # Test with missing WASM file
        results = validator.test_contract("missing_contract")
        
        assert len(results) == 1
        assert not results[0].passed
        assert "Contract file not found" in results[0].details
    
    def test_run_all_tests_success(self, validator):
        """Test running all tests successfully"""
        with patch.object(validator, 'test_contract') as mock_test:
            # Mock successful test results
            mock_test.return_value = [
                TestResult("determinism", True, "Passed"),
                TestResult("gas_limits", True, "Passed"),
                TestResult("zk_proofs", True, "Passed"),
                TestResult("error_handling", True, "Passed"),
            ]
            
            results = validator.run_all_tests()
            
            assert len(results) == 1  # One contract
            assert len(results["test_contract"]) == 4  # Four tests per contract
            assert all(all(result.passed for result in contract_results) 
                      for contract_results in results.values())
    
    def test_run_all_tests_no_contracts(self, validator):
        """Test running tests with no contract files"""
        # Create empty fixtures directory
        empty_dir = Path(tempfile.mkdtemp())
        
        with patch.object(validator, 'fixtures_dir', empty_dir):
            results = validator.run_all_tests()
            
            assert len(results) == 0
    
    def test_generate_report(self, validator):
        """Test report generation"""
        # Create mock test results
        results = {
            "contract1": [
                TestResult("test1", True, "Passed"),
                TestResult("test2", False, "Failed"),
            ],
            "contract2": [
                TestResult("test1", True, "Passed"),
                TestResult("test2", True, "Passed"),
            ]
        }
        
        report = validator.generate_report(results)
        
        assert "Total Contracts: 2" in report
        assert "Total Tests: 4" in report
        assert "Passed: 3" in report
        assert "Failed: 1" in report
        assert "Success Rate: 75.0%" in report
        assert "contract1:" in report
        assert "contract2:" in report
        assert "FAIL: test2 - Failed" in report
        assert "PASS: test1 - Passed" in report

class TestIntegration:
    """Integration tests for the contract validator"""
    
    @pytest.fixture
    def temp_fixtures_dir(self):
        """Create a temporary fixtures directory with test contracts"""
        with tempfile.TemporaryDirectory() as temp_dir:
            fixtures_dir = Path(temp_dir) / "fixtures"
            fixtures_dir.mkdir()
            
            # Create test contract files
            (fixtures_dir / "test.wasm").write_bytes(b"mock_wasm_data")
            (fixtures_dir / "test_input.json").write_text('{"test": "data"}')
            
            yield fixtures_dir
    
    def test_end_to_end_validation(self, temp_fixtures_dir):
        """Test end-to-end contract validation workflow"""
        # This test would require a real ngfs-contract binary
        # For now, we'll test the structure and error handling
        
        mock_binary = "/non/existent/ngfs-contract"
        
        with pytest.raises(ValueError, match="ngfs-contract binary not found"):
            ContractValidator(
                str(temp_fixtures_dir),
                mock_binary,
                verbose=False
            )

if __name__ == "__main__":
    pytest.main([__file__])
