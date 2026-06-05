#!/usr/bin/env python3
"""
Sensor Validator - Python validation framework for Aetheris OS Sensor operations

This module provides comprehensive validation for sensor operations including
sensor registration, sampling, preview streams, and snapshot creation.
"""

import json
import time
import uuid
import hashlib
import asyncio
import logging
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Union, Set
from dataclasses import dataclass, asdict
from enum import Enum
import aiohttp
import pytest

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class SensorKind(Enum):
    ACCELEROMETER = "accelerometer"
    GYROSCOPE = "gyroscope"
    MAGNETOMETER = "magnetometer"
    TEMPERATURE = "temperature"
    HUMIDITY = "humidity"
    PRESSURE = "pressure"
    LIGHT = "light"
    PROXIMITY = "proximity"
    HEART_RATE = "heart_rate"
    CUSTOM = "custom"

class SensorValueType(Enum):
    SCALAR = "scalar"
    VECTOR2 = "vector2"
    VECTOR3 = "vector3"

@dataclass
class SensorValue:
    type: SensorValueType
    value: Union[float, List[float]]

@dataclass
class SensorDesc:
    name: str
    kind: SensorKind
    location: str
    unit: str
    range_min: float
    range_max: float
    resolution: float
    sample_rates: List[int]

@dataclass
class SensorInfo:
    sensor_id: str
    name: str
    kind: SensorKind
    location: str
    unit: str
    range_min: float
    range_max: float
    resolution: float
    sample_rates: List[int]
    registered_at: str
    active_sampling: bool

@dataclass
class SensorSample:
    sensor_id: str
    timestamp: datetime
    value: SensorValue
    sequence_number: int

@dataclass
class SamplingSession:
    handle: str
    sensor_id: str
    session_id: str
    hz: int
    seed: int
    start_time: datetime
    sample_count: int
    deterministic: bool

@dataclass
class PreviewStream:
    stream_handle: str
    sensor_id: str
    hz_max: int
    start_time: datetime
    sample_count: int

@dataclass
class SensorSnapshot:
    snapshot_id: str
    sensor_id: str
    ngfs_path: str
    created_at: datetime
    sample_count: int
    duration: float
    metadata: Dict[str, Any]

@dataclass
class SensorStats:
    connection_status: str
    total_sensors: int
    active_sensors: int
    total_sessions: int
    active_sessions: int
    total_previews: int
    active_previews: int
    total_snapshots: int
    total_samples: int
    average_sample_rate: float
    last_sample_time: datetime
    last_snapshot_time: datetime

class SensorValidator:
    """Main Sensor validation class"""
    
    def __init__(self, base_url: str = "http://localhost:8080", api_key: Optional[str] = None):
        self.base_url = base_url
        self.api_key = api_key
        self.session: Optional[aiohttp.ClientSession] = None
        self.test_results: List[Dict[str, Any]] = []
        self.registered_sensors: Dict[str, SensorInfo] = {}
        self.active_sessions: Dict[str, SamplingSession] = {}
        self.active_previews: Dict[str, PreviewStream] = {}
        
    async def __aenter__(self):
        self.session = aiohttp.ClientSession(
            headers={"Authorization": f"Bearer {self.api_key}"} if self.api_key else {}
        )
        return self
        
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()

    async def make_request(self, endpoint: str, method: str = "GET", data: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """Make HTTP request to sensor service"""
        if not self.session:
            raise RuntimeError("Session not initialized")
        
        url = f"{self.base_url}{endpoint}"
        
        try:
            if method == "GET":
                async with self.session.get(url) as response:
                    return await response.json()
            elif method == "POST":
                async with self.session.post(url, json=data) as response:
                    return await response.json()
            elif method == "PUT":
                async with self.session.put(url, json=data) as response:
                    return await response.json()
            elif method == "DELETE":
                async with self.session.delete(url) as response:
                    return await response.json()
            else:
                raise ValueError(f"Unsupported HTTP method: {method}")
        except Exception as e:
            logger.error(f"Request failed: {e}")
            raise

    async def validate_sensor_registration(self) -> Dict[str, Any]:
        """Validate sensor registration operations"""
        logger.info("Validating sensor registration...")
        
        try:
            # Test sensor registration
            sensor_desc = SensorDesc(
                name="Test Accelerometer",
                kind=SensorKind.ACCELEROMETER,
                location="IMU",
                unit="m/s²",
                range_min=-20.0,
                range_max=20.0,
                resolution=0.01,
                sample_rates=[10, 50, 100, 200]
            )
            
            registration_data = {
                "name": sensor_desc.name,
                "kind": sensor_desc.kind.value,
                "location": sensor_desc.location,
                "unit": sensor_desc.unit,
                "range_min": sensor_desc.range_min,
                "range_max": sensor_desc.range_max,
                "resolution": sensor_desc.resolution,
                "sample_rates": sensor_desc.sample_rates
            }
            
            response = await self.make_request("/sensor/register", "POST", registration_data)
            
            if not response.get("success"):
                return {
                    "test": "sensor_registration",
                    "status": "failed",
                    "error": f"Sensor registration failed: {response.get('error')}"
                }
            
            sensor_id = response.get("sensor_id")
            
            # Store sensor info
            sensor_info = SensorInfo(
                sensor_id=sensor_id,
                name=sensor_desc.name,
                kind=sensor_desc.kind,
                location=sensor_desc.location,
                unit=sensor_desc.unit,
                range_min=sensor_desc.range_min,
                range_max=sensor_desc.range_max,
                resolution=sensor_desc.resolution,
                sample_rates=sensor_desc.sample_rates,
                registered_at=datetime.utcnow().isoformat(),
                active_sampling=False
            )
            
            self.registered_sensors[sensor_id] = sensor_info
            
            # Test sensor listing
            list_response = await self.make_request("/sensor/list")
            
            if not list_response.get("success"):
                return {
                    "test": "sensor_registration",
                    "status": "failed",
                    "error": f"Sensor listing failed: {list_response.get('error')}"
                }
            
            sensors = list_response.get("sensors", [])
            
            # Verify our sensor is in the list
            our_sensor = next((s for s in sensors if s["sensor_id"] == sensor_id), None)
            
            if not our_sensor:
                return {
                    "test": "sensor_registration",
                    "status": "failed",
                    "error": "Registered sensor not found in list"
                }
            
            return {
                "test": "sensor_registration",
                "status": "passed",
                "sensor_id": sensor_id,
                "sensor_info": asdict(sensor_info),
                "total_sensors": len(sensors),
                "details": {
                    "registered_sensor": our_sensor,
                    "all_sensors": sensors[:5]  # First 5 sensors for details
                }
            }
            
        except Exception as e:
            return {
                "test": "sensor_registration",
                "status": "failed",
                "error": str(e)
            }

    async def validate_sampling_operations(self) -> Dict[str, Any]:
        """Validate sensor sampling operations"""
        logger.info("Validating sensor sampling operations...")
        
        try:
            # First, register a test sensor if we don't have one
            if not self.registered_sensors:
                reg_result = await self.validate_sensor_registration()
                if reg_result["status"] != "passed":
                    return {
                        "test": "sampling_operations",
                        "status": "failed",
                        "error": f"Failed to register test sensor: {reg_result.get('error')}"
                    }
            
            # Get a registered sensor
            sensor_id = next(iter(self.registered_sensors.keys()))
            sensor_info = self.registered_sensors[sensor_id]
            
            # Test sampling start
            session_id = f"test_session_{uuid.uuid4().hex[:8]}"
            caps = "device:sensor.sample"
            hz = 10
            seed = 12345  # Deterministic seed
            
            sampling_data = {
                "session_id": session_id,
                "caps": caps,
                "sensor_id": sensor_id,
                "hz": hz,
                "seed": seed
            }
            
            response = await self.make_request("/sensor/sample/start", "POST", sampling_data)
            
            if not response.get("success"):
                return {
                    "test": "sampling_operations",
                    "status": "failed",
                    "error": f"Sampling start failed: {response.get('error')}"
                }
            
            handle = response.get("handle")
            
            # Store session info
            session = SamplingSession(
                handle=handle,
                sensor_id=sensor_id,
                session_id=session_id,
                hz=hz,
                seed=seed,
                start_time=datetime.utcnow(),
                sample_count=0,
                deterministic=True
            )
            
            self.active_sessions[handle] = session
            
            # Wait for some samples to be generated
            await asyncio.sleep(2)
            
            # Test getting samples
            samples_response = await self.make_request(f"/sensor/sample/{handle}/samples", "GET", {"limit": 10})
            
            if not samples_response.get("success"):
                return {
                    "test": "sampling_operations",
                    "status": "failed",
                    "error": f"Get samples failed: {samples_response.get('error')}"
                }
            
            samples = samples_response.get("samples", [])
            
            # Test sampling stop
            stop_response = await self.make_request(f"/sensor/sample/{handle}/stop", "POST")
            
            if not stop_response.get("success"):
                return {
                    "test": "sampling_operations",
                    "status": "failed",
                    "error": f"Sampling stop failed: {stop_response.get('error')}"
                }
            
            # Remove from active sessions
            self.active_sessions.pop(handle, None)
            
            return {
                "test": "sampling_operations",
                "status": "passed",
                "handle": handle,
                "sensor_id": sensor_id,
                "session_id": session_id,
                "samples_collected": len(samples),
                "sampling_rate": hz,
                "details": {
                    "samples": samples[:5],  # First 5 samples for details
                    "session_duration": (datetime.utcnow() - session.start_time).total_seconds(),
                    "deterministic": True
                }
            }
            
        except Exception as e:
            return {
                "test": "sampling_operations",
                "status": "failed",
                "error": str(e)
            }

    async def validate_preview_operations(self) -> Dict[str, Any]:
        """Validate sensor preview operations"""
        logger.info("Validating sensor preview operations...")
        
        try:
            # First, register a test sensor if we don't have one
            if not self.registered_sensors:
                reg_result = await self.validate_sensor_registration()
                if reg_result["status"] != "passed":
                    return {
                        "test": "preview_operations",
                        "status": "failed",
                        "error": f"Failed to register test sensor: {reg_result.get('error')}"
                    }
            
            # Get a registered sensor
            sensor_id = next(iter(self.registered_sensors.keys()))
            sensor_info = self.registered_sensors[sensor_id]
            
            # Test preview start
            hz_max = 5
            
            preview_data = {
                "sensor_id": sensor_id,
                "hz_max": hz_max
            }
            
            response = await self.make_request("/sensor/preview/start", "POST", preview_data)
            
            if not response.get("success"):
                return {
                    "test": "preview_operations",
                    "status": "failed",
                    "error": f"Preview start failed: {response.get('error')}"
                }
            
            stream_handle = response.get("stream_handle")
            
            # Store stream info
            stream = PreviewStream(
                stream_handle=stream_handle,
                sensor_id=sensor_id,
                hz_max=hz_max,
                start_time=datetime.utcnow(),
                sample_count=0
            )
            
            self.active_previews[stream_handle] = stream
            
            # Wait for some preview samples
            await asyncio.sleep(2)
            
            # Test getting preview samples
            samples_response = await self.make_request(f"/sensor/preview/{stream_handle}/samples", "GET", {"limit": 5})
            
            if not samples_response.get("success"):
                return {
                    "test": "preview_operations",
                    "status": "failed",
                    "error": f"Get preview samples failed: {samples_response.get('error')}"
                }
            
            samples = samples_response.get("samples", [])
            
            # Test preview stop
            stop_response = await self.make_request(f"/sensor/preview/{stream_handle}/stop", "POST")
            
            if not stop_response.get("success"):
                return {
                    "test": "preview_operations",
                    "status": "failed",
                    "error": f"Preview stop failed: {stop_response.get('error')}"
                }
            
            # Remove from active previews
            self.active_previews.pop(stream_handle, None)
            
            return {
                "test": "preview_operations",
                "status": "passed",
                "stream_handle": stream_handle,
                "sensor_id": sensor_id,
                "hz_max": hz_max,
                "samples_collected": len(samples),
                "details": {
                    "samples": samples,
                    "stream_duration": (datetime.utcnow() - stream.start_time).total_seconds()
                }
            }
            
        except Exception as e:
            return {
                "test": "preview_operations",
                "status": "failed",
                "error": str(e)
            }

    async def validate_snapshot_operations(self) -> Dict[str, Any]:
        """Validate sensor snapshot operations"""
        logger.info("Validating sensor snapshot operations...")
        
        try:
            # First, register a test sensor if we don't have one
            if not self.registered_sensors:
                reg_result = await self.validate_sensor_registration()
                if reg_result["status"] != "passed":
                    return {
                        "test": "snapshot_operations",
                        "status": "failed",
                        "error": f"Failed to register test sensor: {reg_result.get('error')}"
                    }
            
            # Get a registered sensor
            sensor_id = next(iter(self.registered_sensors.keys()))
            sensor_info = self.registered_sensors[sensor_id]
            
            # Test snapshot creation
            output_path = f"snaps/test_{sensor_id}_{uuid.uuid4().hex[:8]}.ngfs"
            
            snapshot_data = {
                "sensor_id": sensor_id,
                "output_path": output_path
            }
            
            response = await self.make_request("/sensor/snapshot/create", "POST", snapshot_data)
            
            if not response.get("success"):
                return {
                    "test": "snapshot_operations",
                    "status": "failed",
                    "error": f"Snapshot creation failed: {response.get('error')}"
                }
            
            snapshot_id = response.get("snapshot_id")
            
            # Create snapshot object
            snapshot = SensorSnapshot(
                snapshot_id=snapshot_id,
                sensor_id=sensor_id,
                ngfs_path=output_path,
                created_at=datetime.utcnow(),
                sample_count=response.get("sample_count", 0),
                duration=response.get("duration", 0),
                metadata=response.get("metadata", {})
            )
            
            return {
                "test": "snapshot_operations",
                "status": "passed",
                "snapshot_id": snapshot_id,
                "sensor_id": sensor_id,
                "output_path": output_path,
                "sample_count": snapshot.sample_count,
                "duration": snapshot.duration,
                "details": {
                    "snapshot": asdict(snapshot),
                    "metadata": snapshot.metadata
                }
            }
            
        except Exception as e:
            return {
                "test": "snapshot_operations",
                "status": "failed",
                "error": str(e)
            }

    async def validate_deterministic_operations(self) -> Dict[str, Any]:
        """Validate deterministic sensor operations"""
        logger.info("Validating deterministic sensor operations...")
        
        try:
            # First, register a test sensor if we don't have one
            if not self.registered_sensors:
                reg_result = await self.validate_sensor_registration()
                if reg_result["status"] != "passed":
                    return {
                        "test": "deterministic_operations",
                        "status": "failed",
                        "error": f"Failed to register test sensor: {reg_result.get('error')}"
                    }
            
            # Get a registered sensor
            sensor_id = next(iter(self.registered_sensors.keys()))
            sensor_info = self.registered_sensors[sensor_id]
            
            # Test multiple sampling sessions with same seed
            session_results = []
            seed = 54321  # Fixed seed for determinism
            
            for i in range(3):
                session_id = f"deterministic_session_{i}_{uuid.uuid4().hex[:8]}"
                caps = "device:sensor.sample"
                hz = 10
                
                sampling_data = {
                    "session_id": session_id,
                    "caps": caps,
                    "sensor_id": sensor_id,
                    "hz": hz,
                    "seed": seed
                }
                
                response = await self.make_request("/sensor/sample/start", "POST", sampling_data)
                
                if not response.get("success"):
                    return {
                        "test": "deterministic_operations",
                        "status": "failed",
                        "error": f"Sampling session {i+1} failed: {response.get('error')}"
                    }
                
                handle = response.get("handle")
                
                # Store session info
                session = SamplingSession(
                    handle=handle,
                    sensor_id=sensor_id,
                    session_id=session_id,
                    hz=hz,
                    seed=seed,
                    start_time=datetime.utcnow(),
                    sample_count=0,
                    deterministic=True
                )
                
                self.active_sessions[handle] = session
                
                # Wait for samples
                await asyncio.sleep(1)
                
                # Get samples
                samples_response = await self.make_request(f"/sensor/sample/{handle}/samples", "GET", {"limit": 5})
                
                if not samples_response.get("success"):
                    return {
                        "test": "deterministic_operations",
                        "status": "failed",
                        "error": f"Get samples for session {i+1} failed: {samples_response.get('error')}"
                    }
                
                samples = samples_response.get("samples", [])
                
                # Stop session
                await self.make_request(f"/sensor/sample/{handle}/stop", "POST")
                self.active_sessions.pop(handle, None)
                
                session_results.append({
                    "session_id": session_id,
                    "handle": handle,
                    "samples": samples,
                    "sample_count": len(samples)
                })
            
            # Check if results are deterministic
            # For deterministic operations, the first few samples should be identical
            is_deterministic = True
            if len(session_results) >= 2:
                first_samples = session_results[0]["samples"]
                for i in range(1, len(session_results)):
                    other_samples = session_results[i]["samples"]
                    if len(first_samples) != len(other_samples):
                        is_deterministic = False
                        break
                    
                    for j in range(min(len(first_samples), len(other_samples))):
                        if first_samples[j] != other_samples[j]:
                            is_deterministic = False
                            break
                    
                    if not is_deterministic:
                        break
            
            return {
                "test": "deterministic_operations",
                "status": "passed" if is_deterministic else "failed",
                "session_results": session_results,
                "is_deterministic": is_deterministic,
                "details": {
                    "seed": seed,
                    "sessions_tested": len(session_results),
                    "sample_counts": [r["sample_count"] for r in session_results]
                }
            }
            
        except Exception as e:
            return {
                "test": "deterministic_operations",
                "status": "failed",
                "error": str(e)
            }

    async def validate_performance(self) -> Dict[str, Any]:
        """Validate sensor performance metrics"""
        logger.info("Validating sensor performance...")
        
        try:
            # First, register a test sensor if we don't have one
            if not self.registered_sensors:
                reg_result = await self.validate_sensor_registration()
                if reg_result["status"] != "passed":
                    return {
                        "test": "performance",
                        "status": "failed",
                        "error": f"Failed to register test sensor: {reg_result.get('error')}"
                    }
            
            # Get a registered sensor
            sensor_id = next(iter(self.registered_sensors.keys()))
            sensor_info = self.registered_sensors[sensor_id]
            
            # Test sampling performance
            session_id = f"perf_session_{uuid.uuid4().hex[:8]}"
            caps = "device:sensor.sample"
            hz = 100  # High rate for performance testing
            seed = 0  # Non-deterministic for performance
            
            sampling_data = {
                "session_id": session_id,
                "caps": caps,
                "sensor_id": sensor_id,
                "hz": hz,
                "seed": seed
            }
            
            start_time = time.time()
            
            response = await self.make_request("/sensor/sample/start", "POST", sampling_data)
            
            if not response.get("success"):
                return {
                    "test": "performance",
                    "status": "failed",
                    "error": f"Sampling start failed: {response.get('error')}"
                }
            
            handle = response.get("handle")
            
            # Store session info
            session = SamplingSession(
                handle=handle,
                sensor_id=sensor_id,
                session_id=session_id,
                hz=hz,
                seed=seed,
                start_time=datetime.utcnow(),
                sample_count=0,
                deterministic=False
            )
            
            self.active_sessions[handle] = session
            
            # Wait for samples
            await asyncio.sleep(2)
            
            # Get samples
            samples_response = await self.make_request(f"/sensor/sample/{handle}/samples", "GET", {"limit": 100})
            
            if not samples_response.get("success"):
                return {
                    "test": "performance",
                    "status": "failed",
                    "error": f"Get samples failed: {samples_response.get('error')}"
                }
            
            samples = samples_response.get("samples", [])
            
            # Stop session
            stop_response = await self.make_request(f"/sensor/sample/{handle}/stop", "POST")
            
            end_time = time.time()
            total_time = end_time - start_time
            
            # Remove from active sessions
            self.active_sessions.pop(handle, None)
            
            # Test preview performance
            preview_data = {
                "sensor_id": sensor_id,
                "hz_max": 10
            }
            
            preview_start_time = time.time()
            
            preview_response = await self.make_request("/sensor/preview/start", "POST", preview_data)
            
            if preview_response.get("success"):
                stream_handle = preview_response.get("stream_handle")
                
                # Wait for preview samples
                await asyncio.sleep(1)
                
                # Get preview samples
                preview_samples_response = await self.make_request(f"/sensor/preview/{stream_handle}/samples", "GET", {"limit": 10})
                
                # Stop preview
                await self.make_request(f"/sensor/preview/{stream_handle}/stop", "POST")
                
                preview_end_time = time.time()
                preview_time = preview_end_time - preview_start_time
            else:
                preview_time = None
            
            # Get sensor stats
            stats_response = await self.make_request("/sensor/stats")
            stats = stats_response if stats_response.get("success") else {}
            
            return {
                "test": "performance",
                "status": "passed",
                "sampling_time": total_time,
                "preview_time": preview_time,
                "samples_collected": len(samples),
                "sampling_rate": hz,
                "stats": stats,
                "details": {
                    "session_duration": (datetime.utcnow() - session.start_time).total_seconds(),
                    "samples_per_second": len(samples) / total_time if total_time > 0 else 0,
                    "preview_samples": preview_samples_response.get("samples", []) if preview_response.get("success") else []
                }
            }
            
        except Exception as e:
            return {
                "test": "performance",
                "status": "failed",
                "error": str(e)
            }

    async def run_all_tests(self) -> Dict[str, Any]:
        """Run all sensor validation tests"""
        logger.info("Starting comprehensive sensor validation...")
        
        test_results = []
        
        # Test sensor registration
        result = await self.validate_sensor_registration()
        test_results.append(result)
        
        # Test sampling operations
        result = await self.validate_sampling_operations()
        test_results.append(result)
        
        # Test preview operations
        result = await self.validate_preview_operations()
        test_results.append(result)
        
        # Test snapshot operations
        result = await self.validate_snapshot_operations()
        test_results.append(result)
        
        # Test deterministic operations
        result = await self.validate_deterministic_operations()
        test_results.append(result)
        
        # Test performance
        result = await self.validate_performance()
        test_results.append(result)
        
        # Calculate summary
        passed_tests = sum(1 for r in test_results if r["status"] == "passed")
        failed_tests = sum(1 for r in test_results if r["status"] == "failed")
        skipped_tests = sum(1 for r in test_results if r["status"] == "skipped")
        total_tests = len(test_results)
        
        summary = {
            "total_tests": total_tests,
            "passed_tests": passed_tests,
            "failed_tests": failed_tests,
            "skipped_tests": skipped_tests,
            "success_rate": (passed_tests / total_tests * 100) if total_tests > 0 else 0,
            "test_results": test_results
        }
        
        logger.info(f"Sensor validation completed: {passed_tests}/{total_tests} tests passed")
        
        return summary

# Test suite for pytest
class TestSensorValidator:
    """Pytest test suite for sensor validation"""
    
    @pytest.fixture
    async def sensor_validator(self):
        async with SensorValidator() as validator:
            yield validator
    
    @pytest.mark.asyncio
    async def test_sensor_registration(self, sensor_validator):
        """Test sensor registration"""
        result = await sensor_validator.validate_sensor_registration()
        assert result["status"] == "passed", f"Sensor registration failed: {result.get('error')}"
    
    @pytest.mark.asyncio
    async def test_sampling_operations(self, sensor_validator):
        """Test sampling operations"""
        result = await sensor_validator.validate_sampling_operations()
        assert result["status"] == "passed", f"Sampling operations failed: {result.get('error')}"
    
    @pytest.mark.asyncio
    async def test_preview_operations(self, sensor_validator):
        """Test preview operations"""
        result = await sensor_validator.validate_preview_operations()
        assert result["status"] == "passed", f"Preview operations failed: {result.get('error')}"
    
    @pytest.mark.asyncio
    async def test_snapshot_operations(self, sensor_validator):
        """Test snapshot operations"""
        result = await sensor_validator.validate_snapshot_operations()
        assert result["status"] == "passed", f"Snapshot operations failed: {result.get('error')}"
    
    @pytest.mark.asyncio
    async def test_deterministic_operations(self, sensor_validator):
        """Test deterministic operations"""
        result = await sensor_validator.validate_deterministic_operations()
        assert result["status"] == "passed", f"Deterministic operations failed: {result.get('error')}"
    
    @pytest.mark.asyncio
    async def test_performance(self, sensor_validator):
        """Test performance metrics"""
        result = await sensor_validator.validate_performance()
        assert result["status"] == "passed", f"Performance test failed: {result.get('error')}"

# CLI interface
async def main():
    """Main CLI function"""
    import argparse
    
    parser = argparse.ArgumentParser(description="Sensor Validator for Aetheris OS")
    parser.add_argument("--base-url", default="http://localhost:8080", help="Base URL for sensor service")
    parser.add_argument("--api-key", help="API key for authentication")
    parser.add_argument("--output", help="Output file for results")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    async with SensorValidator(args.base_url, args.api_key) as validator:
        results = await validator.run_all_tests()
        
        if args.output:
            with open(args.output, 'w') as f:
                json.dump(results, f, indent=2, default=str)
            print(f"Results saved to {args.output}")
        else:
            print(json.dumps(results, indent=2, default=str))

if __name__ == "__main__":
    asyncio.run(main())
