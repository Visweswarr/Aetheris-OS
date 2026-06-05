#!/usr/bin/env python3
"""
Python Validator for Aetheris OS Smart Contracts

This module provides deterministic execution validation for smart contracts,
including WASM validation, gas metering verification, and execution result validation.
"""

import asyncio
import hashlib
import json
import time
import uuid
from dataclasses import dataclass, asdict
from typing import Dict, List, Optional, Any, Union, Tuple
from enum import Enum
import logging

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class ValidationStatus(Enum):
    """Validation status enumeration"""
    PENDING = "pending"
    VALID = "valid"
    INVALID = "invalid"
    ERROR = "error"

class ExecutionMode(Enum):
    """Execution mode enumeration"""
    DETERMINISTIC = "deterministic"
    ZK_PROOF = "zk_proof"
    HYBRID = "hybrid"

@dataclass
class ContractMetadata:
    """Contract metadata structure"""
    id: str
    name: str
    version: str
    author: str
    description: str
    wasm_hash: str
    created_at: int
    updated_at: int

@dataclass
class ExecutionRequest:
    """Contract execution request structure"""
    contract_id: str
    method: str
    args: bytes
    gas_limit: int
    caller: str
    value: int
    execution_mode: ExecutionMode = ExecutionMode.DETERMINISTIC

@dataclass
class ExecutionResult:
    """Contract execution result structure"""
    success: bool
    output: bytes
    gas_used: int
    execution_time: int
    events: List[Dict[str, Any]]
    error: Optional[str] = None
    zk_proof: Optional[str] = None

@dataclass
class ValidationResult:
    """Validation result structure"""
    request_id: str
    status: ValidationStatus
    execution_result: Optional[ExecutionResult]
    validation_time: int
    error: Optional[str] = None
    gas_verification: Optional[Dict[str, Any]] = None
    determinism_check: Optional[Dict[str, Any]] = None

@dataclass
class GasConfig:
    """Gas configuration structure"""
    base_gas: int = 21000
    contract_deployment_gas: int = 50000
    method_call_gas: int = 25000
    storage_read_gas: int = 200
    storage_write_gas: int = 5000
    computation_gas_per_op: int = 1
    memory_gas_per_byte: int = 3

@dataclass
class ValidationConfig:
    """Validation configuration structure"""
    max_execution_time: int = 30000  # 30 seconds
    max_gas_limit: int = 10000000    # 10M gas
    max_memory_usage: int = 64 * 1024 * 1024  # 64 MB
    enable_zk_validation: bool = True
    enable_determinism_check: bool = True
    enable_gas_verification: bool = True
    gas_config: GasConfig = None

    def __post_init__(self):
        if self.gas_config is None:
            self.gas_config = GasConfig()

class WASMValidator:
    """WASM module validator"""
    
    def __init__(self):
        self.validated_modules: Dict[str, bool] = {}
    
    def validate_wasm_module(self, wasm_bytes: bytes) -> Tuple[bool, Optional[str]]:
        """
        Validate a WASM module
        
        Args:
            wasm_bytes: The WASM module bytes
            
        Returns:
            Tuple of (is_valid, error_message)
        """
        try:
            # Check WASM magic number
            if len(wasm_bytes) < 8:
                return False, "WASM module too short"
            
            magic = wasm_bytes[:4]
            if magic != b'\x00asm':
                return False, "Invalid WASM magic number"
            
            version = wasm_bytes[4:8]
            if version != b'\x01\x00\x00\x00':
                return False, "Unsupported WASM version"
            
            # Calculate hash for caching
            wasm_hash = hashlib.sha256(wasm_bytes).hexdigest()
            
            if wasm_hash in self.validated_modules:
                return self.validated_modules[wasm_hash], None
            
            # Basic structure validation
            if not self._validate_wasm_structure(wasm_bytes):
                self.validated_modules[wasm_hash] = False
                return False, "Invalid WASM structure"
            
            # Security checks
            if not self._check_security_constraints(wasm_bytes):
                self.validated_modules[wasm_hash] = False
                return False, "WASM module violates security constraints"
            
            self.validated_modules[wasm_hash] = True
            return True, None
            
        except Exception as e:
            logger.error(f"WASM validation error: {e}")
            return False, f"WASM validation error: {str(e)}"
    
    def _validate_wasm_structure(self, wasm_bytes: bytes) -> bool:
        """Validate basic WASM structure"""
        try:
            # This is a simplified validation
            # In a real implementation, you would use a proper WASM parser
            offset = 8  # Skip magic and version
            
            while offset < len(wasm_bytes):
                if offset + 1 >= len(wasm_bytes):
                    break
                
                section_id = wasm_bytes[offset]
                offset += 1
                
                if offset >= len(wasm_bytes):
                    return False
                
                # Read section size
                section_size = 0
                shift = 0
                while offset < len(wasm_bytes):
                    byte = wasm_bytes[offset]
                    offset += 1
                    section_size |= (byte & 0x7F) << shift
                    if (byte & 0x80) == 0:
                        break
                    shift += 7
                
                if offset + section_size > len(wasm_bytes):
                    return False
                
                offset += section_size
            
            return True
            
        except Exception:
            return False
    
    def _check_security_constraints(self, wasm_bytes: bytes) -> bool:
        """Check security constraints"""
        # Check for forbidden imports
        forbidden_imports = [
            b'env.malloc',
            b'env.free',
            b'env.system',
            b'env.exec',
            b'env.fork',
            b'env.exit',
        ]
        
        wasm_str = wasm_bytes.decode('utf-8', errors='ignore')
        for forbidden in forbidden_imports:
            if forbidden in wasm_bytes:
                return False
        
        return True

class GasMeter:
    """Gas metering and verification"""
    
    def __init__(self, config: GasConfig):
        self.config = config
        self.gas_used = 0
        self.gas_limit = 0
    
    def set_gas_limit(self, limit: int):
        """Set gas limit"""
        self.gas_limit = limit
        self.gas_used = 0
    
    def consume_gas(self, amount: int) -> bool:
        """
        Consume gas
        
        Args:
            amount: Amount of gas to consume
            
        Returns:
            True if gas was consumed successfully, False if limit exceeded
        """
        if self.gas_used + amount > self.gas_limit:
            return False
        
        self.gas_used += amount
        return True
    
    def estimate_contract_deployment_gas(self, wasm_size: int) -> int:
        """Estimate gas for contract deployment"""
        return self.config.contract_deployment_gas + (wasm_size * self.config.memory_gas_per_byte)
    
    def estimate_method_call_gas(self, method_name: str, args_size: int) -> int:
        """Estimate gas for method call"""
        base_gas = self.config.method_call_gas
        method_gas = len(method_name) * self.config.computation_gas_per_op
        args_gas = args_size * self.config.memory_gas_per_byte
        
        return base_gas + method_gas + args_gas
    
    def verify_gas_usage(self, expected_gas: int, actual_gas: int, tolerance: float = 0.1) -> bool:
        """
        Verify gas usage is within tolerance
        
        Args:
            expected_gas: Expected gas usage
            actual_gas: Actual gas usage
            tolerance: Tolerance percentage (0.1 = 10%)
            
        Returns:
            True if gas usage is within tolerance
        """
        if expected_gas == 0:
            return actual_gas == 0
        
        ratio = abs(actual_gas - expected_gas) / expected_gas
        return ratio <= tolerance

class DeterminismChecker:
    """Deterministic execution checker"""
    
    def __init__(self):
        self.execution_cache: Dict[str, ExecutionResult] = {}
    
    def check_determinism(self, request: ExecutionRequest, result: ExecutionResult) -> Dict[str, Any]:
        """
        Check if execution is deterministic
        
        Args:
            request: Execution request
            result: Execution result
            
        Returns:
            Determinism check results
        """
        # Create cache key
        cache_key = self._create_cache_key(request)
        
        if cache_key in self.execution_cache:
            cached_result = self.execution_cache[cache_key]
            
            # Compare results
            is_deterministic = (
                result.success == cached_result.success and
                result.output == cached_result.output and
                result.gas_used == cached_result.gas_used and
                result.events == cached_result.events
            )
            
            return {
                'is_deterministic': is_deterministic,
                'cached_result': asdict(cached_result),
                'current_result': asdict(result),
                'differences': self._find_differences(cached_result, result)
            }
        else:
            # Cache the result
            self.execution_cache[cache_key] = result
            
            return {
                'is_deterministic': True,  # First execution is considered deterministic
                'cached_result': None,
                'current_result': asdict(result),
                'differences': []
            }
    
    def _create_cache_key(self, request: ExecutionRequest) -> str:
        """Create cache key for request"""
        key_data = {
            'contract_id': request.contract_id,
            'method': request.method,
            'args': request.args.hex(),
            'gas_limit': request.gas_limit,
            'caller': request.caller,
            'value': request.value,
            'execution_mode': request.execution_mode.value
        }
        
        key_str = json.dumps(key_data, sort_keys=True)
        return hashlib.sha256(key_str.encode()).hexdigest()
    
    def _find_differences(self, cached: ExecutionResult, current: ExecutionResult) -> List[str]:
        """Find differences between cached and current results"""
        differences = []
        
        if cached.success != current.success:
            differences.append(f"Success: {cached.success} != {current.success}")
        
        if cached.output != current.output:
            differences.append(f"Output: {cached.output.hex()} != {current.output.hex()}")
        
        if cached.gas_used != current.gas_used:
            differences.append(f"Gas used: {cached.gas_used} != {current.gas_used}")
        
        if cached.events != current.events:
            differences.append(f"Events: {cached.events} != {current.events}")
        
        return differences

class ContractsValidator:
    """Main contracts validator"""
    
    def __init__(self, config: ValidationConfig = None):
        self.config = config or ValidationConfig()
        self.wasm_validator = WASMValidator()
        self.gas_meter = GasMeter(self.config.gas_config)
        self.determinism_checker = DeterminismChecker()
        self.validation_history: List[ValidationResult] = []
    
    async def validate_contract_deployment(self, wasm_bytes: bytes, metadata: ContractMetadata) -> ValidationResult:
        """
        Validate contract deployment
        
        Args:
            wasm_bytes: WASM module bytes
            metadata: Contract metadata
            
        Returns:
            Validation result
        """
        request_id = str(uuid.uuid4())
        start_time = time.time()
        
        try:
            # Validate WASM module
            is_valid, error = self.wasm_validator.validate_wasm_module(wasm_bytes)
            if not is_valid:
                return ValidationResult(
                    request_id=request_id,
                    status=ValidationStatus.INVALID,
                    execution_result=None,
                    validation_time=int((time.time() - start_time) * 1000),
                    error=f"WASM validation failed: {error}"
                )
            
            # Estimate gas for deployment
            estimated_gas = self.gas_meter.estimate_contract_deployment_gas(len(wasm_bytes))
            
            # Create mock execution result
            execution_result = ExecutionResult(
                success=True,
                output=b'',
                gas_used=estimated_gas,
                execution_time=100,  # Mock deployment time
                events=[]
            )
            
            # Gas verification
            gas_verification = {
                'estimated_gas': estimated_gas,
                'actual_gas': estimated_gas,
                'gas_efficient': True,
                'gas_usage_ratio': 1.0
            }
            
            result = ValidationResult(
                request_id=request_id,
                status=ValidationStatus.VALID,
                execution_result=execution_result,
                validation_time=int((time.time() - start_time) * 1000),
                gas_verification=gas_verification
            )
            
            self.validation_history.append(result)
            return result
            
        except Exception as e:
            logger.error(f"Contract deployment validation error: {e}")
            return ValidationResult(
                request_id=request_id,
                status=ValidationStatus.ERROR,
                execution_result=None,
                validation_time=int((time.time() - start_time) * 1000),
                error=str(e)
            )
    
    async def validate_contract_execution(self, request: ExecutionRequest, result: ExecutionResult) -> ValidationResult:
        """
        Validate contract execution
        
        Args:
            request: Execution request
            result: Execution result
            
        Returns:
            Validation result
        """
        request_id = str(uuid.uuid4())
        start_time = time.time()
        
        try:
            # Gas verification
            gas_verification = None
            if self.config.enable_gas_verification:
                estimated_gas = self.gas_meter.estimate_method_call_gas(request.method, len(request.args))
                gas_verification = {
                    'estimated_gas': estimated_gas,
                    'actual_gas': result.gas_used,
                    'gas_efficient': self.gas_meter.verify_gas_usage(estimated_gas, result.gas_used),
                    'gas_usage_ratio': result.gas_used / estimated_gas if estimated_gas > 0 else 0
                }
            
            # Determinism check
            determinism_check = None
            if self.config.enable_determinism_check:
                determinism_check = self.determinism_checker.check_determinism(request, result)
            
            # Overall validation
            is_valid = (
                result.success and
                (gas_verification is None or gas_verification['gas_efficient']) and
                (determinism_check is None or determinism_check['is_deterministic'])
            )
            
            status = ValidationStatus.VALID if is_valid else ValidationStatus.INVALID
            
            validation_result = ValidationResult(
                request_id=request_id,
                status=status,
                execution_result=result,
                validation_time=int((time.time() - start_time) * 1000),
                gas_verification=gas_verification,
                determinism_check=determinism_check
            )
            
            self.validation_history.append(validation_result)
            return validation_result
            
        except Exception as e:
            logger.error(f"Contract execution validation error: {e}")
            return ValidationResult(
                request_id=request_id,
                status=ValidationStatus.ERROR,
                execution_result=result,
                validation_time=int((time.time() - start_time) * 1000),
                error=str(e)
            )
    
    async def validate_zk_proof(self, proof: str, public_inputs: List[str]) -> bool:
        """
        Validate ZK proof
        
        Args:
            proof: ZK proof string
            public_inputs: Public inputs for verification
            
        Returns:
            True if proof is valid
        """
        # Mock ZK proof validation
        # In a real implementation, you would use a proper ZK proof verifier
        try:
            # Basic proof format validation
            if not proof or len(proof) < 100:
                return False
            
            # Mock verification
            await asyncio.sleep(0.1)  # Simulate verification time
            
            return True
            
        except Exception as e:
            logger.error(f"ZK proof validation error: {e}")
            return False
    
    def get_validation_stats(self) -> Dict[str, Any]:
        """Get validation statistics"""
        total_validations = len(self.validation_history)
        valid_count = sum(1 for r in self.validation_history if r.status == ValidationStatus.VALID)
        invalid_count = sum(1 for r in self.validation_history if r.status == ValidationStatus.INVALID)
        error_count = sum(1 for r in self.validation_history if r.status == ValidationStatus.ERROR)
        
        avg_validation_time = sum(r.validation_time for r in self.validation_history) / total_validations if total_validations > 0 else 0
        
        return {
            'total_validations': total_validations,
            'valid_count': valid_count,
            'invalid_count': invalid_count,
            'error_count': error_count,
            'success_rate': valid_count / total_validations if total_validations > 0 else 0,
            'average_validation_time_ms': avg_validation_time,
            'wasm_modules_validated': len(self.wasm_validator.validated_modules),
            'determinism_cache_size': len(self.determinism_checker.execution_cache)
        }
    
    def clear_validation_history(self):
        """Clear validation history"""
        self.validation_history.clear()
        self.wasm_validator.validated_modules.clear()
        self.determinism_checker.execution_cache.clear()

# Example usage and testing
async def main():
    """Example usage of the contracts validator"""
    
    # Create validator
    config = ValidationConfig(
        max_execution_time=30000,
        max_gas_limit=1000000,
        enable_zk_validation=True,
        enable_determinism_check=True,
        enable_gas_verification=True
    )
    
    validator = ContractsValidator(config)
    
    # Example WASM bytes (mock)
    wasm_bytes = b'\x00asm\x01\x00\x00\x00' + b'\x00' * 100
    
    # Example metadata
    metadata = ContractMetadata(
        id="contract_123",
        name="HelloWorld",
        version="1.0.0",
        author="Alice",
        description="A simple hello world contract",
        wasm_hash="abc123",
        created_at=int(time.time()),
        updated_at=int(time.time())
    )
    
    # Validate contract deployment
    print("Validating contract deployment...")
    deployment_result = await validator.validate_contract_deployment(wasm_bytes, metadata)
    print(f"Deployment validation: {deployment_result.status.value}")
    
    # Example execution request
    request = ExecutionRequest(
        contract_id="contract_123",
        method="hello",
        args=b'{"name": "World"}',
        gas_limit=100000,
        caller="0x1234567890123456789012345678901234567890",
        value=0
    )
    
    # Example execution result
    result = ExecutionResult(
        success=True,
        output=b'Hello, World!',
        gas_used=25000,
        execution_time=50,
        events=[{"type": "Hello", "data": "World"}]
    )
    
    # Validate contract execution
    print("Validating contract execution...")
    execution_result = await validator.validate_contract_execution(request, result)
    print(f"Execution validation: {execution_result.status.value}")
    
    # Get validation stats
    stats = validator.get_validation_stats()
    print(f"Validation stats: {stats}")

if __name__ == "__main__":
    asyncio.run(main())
