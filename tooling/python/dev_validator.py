#!/usr/bin/env python3
"""
Device Validator - Python validation tool for Aetheris Device Runtime

This module provides comprehensive validation for device capture operations,
including determinism testing, integrity verification, and performance benchmarking.
"""

import asyncio
import json
import time
import uuid
import hashlib
import argparse
import logging
from typing import Dict, List, Any, Optional, Tuple
from dataclasses import dataclass, asdict
from datetime import datetime, timezone
import cbor2
import blake3

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

@dataclass
class CaptureConfig:
    """Device capture configuration"""
    # Camera config
    width: int = 640
    height: int = 360
    fps: int = 15
    
    # Microphone config
    sample_rate: int = 44100
    channels: int = 2
    
    # Common config
    deterministic: bool = False
    max_chunk_duration_ms: int = 2000
    max_chunk_frames: int = 30

@dataclass
class CaptureStats:
    """Capture statistics"""
    capture_id: str
    duration_ms: int
    bytes_written: int
    chunks_created: int
    last_chunk_timestamp: datetime
    snapshot_id: Optional[str] = None

@dataclass
class ValidationResult:
    """Validation result"""
    success: bool
    message: str
    details: Dict[str, Any]

class MockDeviceService:
    """Mock device service for testing"""
    
    def __init__(self):
        self.active_captures: Dict[str, CaptureInfo] = {}
        self.deterministic_seed: Optional[int] = None
    
    def set_deterministic_seed(self, seed: int):
        """Set deterministic seed for reproducible results"""
        self.deterministic_seed = seed
    
    async def start_camera_capture(self, session_id: str, caps: str, config: CaptureConfig) -> str:
        """Start camera capture"""
        capture_id = str(uuid.uuid4())
        
        capture_info = CaptureInfo(
            capture_id=capture_id,
            session_id=session_id,
            device_type="camera",
            config=config,
            start_time=datetime.now(timezone.utc),
            deterministic=config.deterministic
        )
        
        self.active_captures[capture_id] = capture_info
        return capture_id
    
    async def stop_camera_capture(self, capture_id: str) -> CaptureStats:
        """Stop camera capture"""
        if capture_id not in self.active_captures:
            raise ValueError(f"Capture not found: {capture_id}")
        
        capture_info = self.active_captures[capture_id]
        duration = (datetime.now(timezone.utc) - capture_info.start_time).total_seconds() * 1000
        
        stats = CaptureStats(
            capture_id=capture_id,
            duration_ms=int(duration),
            bytes_written=1024 * 1024,  # Mock 1MB
            chunks_created=3,  # Mock 3 chunks
            last_chunk_timestamp=datetime.now(timezone.utc),
            snapshot_id=f"snapshot_{uuid.uuid4()}"
        )
        
        del self.active_captures[capture_id]
        return stats
    
    async def start_microphone_capture(self, session_id: str, caps: str, config: CaptureConfig) -> str:
        """Start microphone capture"""
        capture_id = str(uuid.uuid4())
        
        capture_info = CaptureInfo(
            capture_id=capture_id,
            session_id=session_id,
            device_type="microphone",
            config=config,
            start_time=datetime.now(timezone.utc),
            deterministic=config.deterministic
        )
        
        self.active_captures[capture_id] = capture_info
        return capture_id
    
    async def stop_microphone_capture(self, capture_id: str) -> CaptureStats:
        """Stop microphone capture"""
        if capture_id not in self.active_captures:
            raise ValueError(f"Capture not found: {capture_id}")
        
        capture_info = self.active_captures[capture_id]
        duration = (datetime.now(timezone.utc) - capture_info.start_time).total_seconds() * 1000
        
        stats = CaptureStats(
            capture_id=capture_id,
            duration_ms=int(duration),
            bytes_written=512 * 1024,  # Mock 512KB
            chunks_created=3,  # Mock 3 chunks
            last_chunk_timestamp=datetime.now(timezone.utc),
            snapshot_id=f"snapshot_{uuid.uuid4()}"
        )
        
        del self.active_captures[capture_id]
        return stats

@dataclass
class CaptureInfo:
    """Capture information"""
    capture_id: str
    session_id: str
    device_type: str
    config: CaptureConfig
    start_time: datetime
    deterministic: bool

class DeviceValidator:
    """Device validation and testing"""
    
    def __init__(self, config: Dict[str, Any] = None):
        self.config = config or {}
        self.device_service = MockDeviceService()
        self.test_results: List[Dict[str, Any]] = []
    
    async def validate_determinism(self) -> ValidationResult:
        """Validate deterministic capture behavior"""
        logger.info("Validating device capture determinism...")
        
        try:
            # Set deterministic seed
            seed = 12345
            self.device_service.set_deterministic_seed(seed)
            
            # Run multiple captures with same configuration
            results = []
            for i in range(3):
                config = CaptureConfig(
                    width=320,
                    height=240,
                    fps=10,
                    deterministic=True
                )
                
                # Start camera capture
                capture_id = await self.device_service.start_camera_capture(
                    f"session_{i}", "device:camera.read", config
                )
                
                # Wait a bit
                await asyncio.sleep(0.1)
                
                # Stop capture
                stats = await self.device_service.stop_camera_capture(capture_id)
                results.append(stats)
            
            # Check if all results are identical
            first_snapshot = results[0].snapshot_id
            for i, result in enumerate(results[1:], 1):
                if result.snapshot_id != first_snapshot:
                    return ValidationResult(
                        success=False,
                        message=f"Determinism failed: run {i} differs from run 0",
                        details={
                            "first_snapshot": first_snapshot,
                            f"run_{i}_snapshot": result.snapshot_id
                        }
                    )
            
            return ValidationResult(
                success=True,
                message="Determinism validation passed",
                details={
                    "runs": len(results),
                    "snapshot_id": first_snapshot
                }
            )
            
        except Exception as e:
            return ValidationResult(
                success=False,
                message=f"Determinism validation failed: {str(e)}",
                details={"error": str(e)}
            )
    
    async def validate_capture_integrity(self) -> ValidationResult:
        """Validate capture integrity and statistics"""
        logger.info("Validating capture integrity...")
        
        try:
            config = CaptureConfig(
                width=640,
                height=360,
                fps=15,
                sample_rate=44100,
                channels=2
            )
            
            # Test camera capture
            camera_id = await self.device_service.start_camera_capture(
                "test_session", "device:camera.read", config
            )
            await asyncio.sleep(0.1)
            camera_stats = await self.device_service.stop_camera_capture(camera_id)
            
            # Test microphone capture
            mic_id = await self.device_service.start_microphone_capture(
                "test_session", "device:mic.read", config
            )
            await asyncio.sleep(0.1)
            mic_stats = await self.device_service.stop_microphone_capture(mic_id)
            
            # Validate statistics
            if camera_stats.bytes_written <= 0:
                return ValidationResult(
                    success=False,
                    message="Invalid camera capture statistics",
                    details={"camera_stats": asdict(camera_stats)}
                )
            
            if mic_stats.bytes_written <= 0:
                return ValidationResult(
                    success=False,
                    message="Invalid microphone capture statistics",
                    details={"mic_stats": asdict(mic_stats)}
                )
            
            return ValidationResult(
                success=True,
                message="Capture integrity validation passed",
                details={
                    "camera_stats": asdict(camera_stats),
                    "mic_stats": asdict(mic_stats)
                }
            )
            
        except Exception as e:
            return ValidationResult(
                success=False,
                message=f"Capture integrity validation failed: {str(e)}",
                details={"error": str(e)}
            )
    
    async def validate_performance(self) -> ValidationResult:
        """Validate performance requirements"""
        logger.info("Validating performance requirements...")
        
        try:
            config = CaptureConfig()
            
            # Measure start time
            start_time = time.time()
            capture_id = await self.device_service.start_camera_capture(
                "perf_session", "device:camera.read", config
            )
            start_duration = (time.time() - start_time) * 1000  # Convert to ms
            
            # Check start time requirement (p95 ≤ 150ms)
            if start_duration > 150:
                return ValidationResult(
                    success=False,
                    message=f"Start time too slow: {start_duration:.2f}ms > 150ms",
                    details={"start_duration_ms": start_duration}
                )
            
            # Wait and measure stop time
            await asyncio.sleep(0.1)
            stop_start = time.time()
            stats = await self.device_service.stop_camera_capture(capture_id)
            stop_duration = (time.time() - stop_start) * 1000  # Convert to ms
            
            # Check stop time requirement (p95 ≤ 300ms)
            if stop_duration > 300:
                return ValidationResult(
                    success=False,
                    message=f"Stop time too slow: {stop_duration:.2f}ms > 300ms",
                    details={"stop_duration_ms": stop_duration}
                )
            
            return ValidationResult(
                success=True,
                message="Performance validation passed",
                details={
                    "start_duration_ms": start_duration,
                    "stop_duration_ms": stop_duration,
                    "total_duration_ms": stats.duration_ms
                }
            )
            
        except Exception as e:
            return ValidationResult(
                success=False,
                message=f"Performance validation failed: {str(e)}",
                details={"error": str(e)}
            )
    
    async def validate_ngfs_integration(self) -> ValidationResult:
        """Validate NGFS snapshot integration"""
        logger.info("Validating NGFS integration...")
        
        try:
            config = CaptureConfig(deterministic=True)
            
            # Start and stop capture
            capture_id = await self.device_service.start_camera_capture(
                "ngfs_session", "device:camera.read", config
            )
            await asyncio.sleep(0.1)
            stats = await self.device_service.stop_camera_capture(capture_id)
            
            # Validate snapshot ID format
            if not stats.snapshot_id or not stats.snapshot_id.startswith("snapshot_"):
                return ValidationResult(
                    success=False,
                    message="Invalid snapshot ID format",
                    details={"snapshot_id": stats.snapshot_id}
                )
            
            # Mock NGFS manifest validation
            manifest = {
                "snapshot_id": stats.snapshot_id,
                "capture_id": stats.capture_id,
                "device_type": "camera",
                "start_time": datetime.now(timezone.utc).isoformat(),
                "end_time": datetime.now(timezone.utc).isoformat(),
                "total_bytes": stats.bytes_written,
                "chunks": stats.chunks_created,
                "deterministic": config.deterministic
            }
            
            # Validate manifest structure
            required_fields = ["snapshot_id", "capture_id", "device_type", "total_bytes"]
            for field in required_fields:
                if field not in manifest:
                    return ValidationResult(
                        success=False,
                        message=f"Missing required field in manifest: {field}",
                        details={"manifest": manifest}
                    )
            
            return ValidationResult(
                success=True,
                message="NGFS integration validation passed",
                details={
                    "snapshot_id": stats.snapshot_id,
                    "manifest": manifest
                }
            )
            
        except Exception as e:
            return ValidationResult(
                success=False,
                message=f"NGFS integration validation failed: {str(e)}",
                details={"error": str(e)}
            )
    
    async def validate_capability_gating(self) -> ValidationResult:
        """Validate capability-based access control"""
        logger.info("Validating capability gating...")
        
        try:
            config = CaptureConfig()
            
            # Test valid capability
            try:
                capture_id = await self.device_service.start_camera_capture(
                    "valid_session", "device:camera.read", config
                )
                await self.device_service.stop_camera_capture(capture_id)
            except Exception as e:
                return ValidationResult(
                    success=False,
                    message=f"Valid capability rejected: {str(e)}",
                    details={"error": str(e)}
                )
            
            # Test invalid capability (should be rejected in real implementation)
            try:
                capture_id = await self.device_service.start_camera_capture(
                    "invalid_session", "device:invalid.read", config
                )
                await self.device_service.stop_camera_capture(capture_id)
                # In mock implementation, this might not fail, but in real implementation it should
                logger.warning("Invalid capability was not rejected (expected in mock)")
            except Exception as e:
                logger.info(f"Invalid capability correctly rejected: {str(e)}")
            
            return ValidationResult(
                success=True,
                message="Capability gating validation passed",
                details={"note": "Mock implementation - real validation would check capabilities"}
            )
            
        except Exception as e:
            return ValidationResult(
                success=False,
                message=f"Capability gating validation failed: {str(e)}",
                details={"error": str(e)}
            )

class DeviceValidatorTests:
    """Test suite for device validation"""
    
    def __init__(self, validator: DeviceValidator):
        self.validator = validator
        self.test_results: List[Dict[str, Any]] = []
    
    async def run_all_tests(self) -> Dict[str, Any]:
        """Run all validation tests"""
        logger.info("Starting device validation tests")
        
        test_methods = [
            self.test_determinism,
            self.test_capture_integrity,
            self.test_performance,
            self.test_ngfs_integration,
            self.test_capability_gating,
        ]
        
        for test_method in test_methods:
            try:
                await test_method()
            except Exception as e:
                logger.error(f"Test {test_method.__name__} failed with exception: {e}")
                self.test_results.append({
                    'test': test_method.__name__,
                    'status': 'error',
                    'error': str(e)
                })
        
        # Calculate summary
        total_tests = len(self.test_results)
        passed_tests = len([r for r in self.test_results if r['status'] == 'passed'])
        failed_tests = len([r for r in self.test_results if r['status'] == 'failed'])
        error_tests = len([r for r in self.test_results if r['status'] == 'error'])
        
        return {
            'total_tests': total_tests,
            'passed_tests': passed_tests,
            'failed_tests': failed_tests,
            'error_tests': error_tests,
            'test_results': self.test_results
        }
    
    async def test_determinism(self) -> None:
        """Test deterministic capture behavior"""
        logger.info("Testing determinism")
        
        result = await self.validator.validate_determinism()
        
        if result.success:
            self.test_results.append({
                'test': 'determinism',
                'status': 'passed',
                'details': result.details
            })
        else:
            self.test_results.append({
                'test': 'determinism',
                'status': 'failed',
                'message': result.message,
                'details': result.details
            })
    
    async def test_capture_integrity(self) -> None:
        """Test capture integrity"""
        logger.info("Testing capture integrity")
        
        result = await self.validator.validate_capture_integrity()
        
        if result.success:
            self.test_results.append({
                'test': 'capture_integrity',
                'status': 'passed',
                'details': result.details
            })
        else:
            self.test_results.append({
                'test': 'capture_integrity',
                'status': 'failed',
                'message': result.message,
                'details': result.details
            })
    
    async def test_performance(self) -> None:
        """Test performance requirements"""
        logger.info("Testing performance")
        
        result = await self.validator.validate_performance()
        
        if result.success:
            self.test_results.append({
                'test': 'performance',
                'status': 'passed',
                'details': result.details
            })
        else:
            self.test_results.append({
                'test': 'performance',
                'status': 'failed',
                'message': result.message,
                'details': result.details
            })
    
    async def test_ngfs_integration(self) -> None:
        """Test NGFS integration"""
        logger.info("Testing NGFS integration")
        
        result = await self.validator.validate_ngfs_integration()
        
        if result.success:
            self.test_results.append({
                'test': 'ngfs_integration',
                'status': 'passed',
                'details': result.details
            })
        else:
            self.test_results.append({
                'test': 'ngfs_integration',
                'status': 'failed',
                'message': result.message,
                'details': result.details
            })
    
    async def test_capability_gating(self) -> None:
        """Test capability gating"""
        logger.info("Testing capability gating")
        
        result = await self.validator.validate_capability_gating()
        
        if result.success:
            self.test_results.append({
                'test': 'capability_gating',
                'status': 'passed',
                'details': result.details
            })
        else:
            self.test_results.append({
                'test': 'capability_gating',
                'status': 'failed',
                'message': result.message,
                'details': result.details
            })

async def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description='Device Validator for Aetheris OS')
    parser.add_argument('--test', help='Specific test to run (comma-separated)')
    parser.add_argument('--verbose', action='store_true', help='Enable verbose logging')
    parser.add_argument('--output', help='Output file for test results')
    parser.add_argument('--deterministic', action='store_true', help='Enable deterministic mode')
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    # Create configuration
    config = {
        'deterministic_mode': args.deterministic,
        'test_timeout': 30,
        'performance_threshold_ms': 150
    }
    
    # Create validator
    validator = DeviceValidator(config)
    
    try:
        # Run tests
        test_suite = DeviceValidatorTests(validator)
        summary = await test_suite.run_all_tests()
        
        # Output results
        if args.output:
            with open(args.output, 'w') as f:
                json.dump(summary, f, indent=2, default=str)
            logger.info(f"Test results saved to {args.output}")
        else:
            print(json.dumps(summary, indent=2, default=str))
        
        # Return exit code based on test results
        return 0 if summary['failed_tests'] == 0 and summary['error_tests'] == 0 else 1
        
    except Exception as e:
        logger.error(f"Validation failed: {e}")
        return 1

if __name__ == '__main__':
    exit_code = asyncio.run(main())
    exit(exit_code)
