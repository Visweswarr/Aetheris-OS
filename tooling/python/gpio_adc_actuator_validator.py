#!/usr/bin/env python3
"""
GPIO/ADC/Actuator Validator - Python validation framework for Aetheris OS Device operations

This module provides comprehensive validation for GPIO, ADC, and actuator operations including
pin configuration, digital I/O, analog sampling, PWM control, and deterministic state management.
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

class GpioMode(Enum):
    INPUT = "input"
    OUTPUT = "output"

class GpioPull(Enum):
    NONE = "none"
    UP = "up"
    DOWN = "down"

class ActuatorType(Enum):
    PWM = "pwm"
    RELAY = "relay"
    MOTOR = "motor"
    LED = "led"
    SERVO = "servo"
    STEPPER = "stepper"
    SOLENOID = "solenoid"
    VALVE = "valve"

@dataclass
class GpioConfig:
    pin: int
    mode: GpioMode
    pull: GpioPull
    initial_value: bool
    debounce_ms: int

@dataclass
class GpioState:
    pin: int
    mode: GpioMode
    pull: GpioPull
    value: bool
    last_change: int
    change_count: int

@dataclass
class AdcConfig:
    channel: int
    sample_rate: int
    resolution: int
    reference_voltage: float
    enable_calibration: bool
    calibration_offset: float
    calibration_scale: float
    oversampling: int
    enable_filtering: bool
    filter_cutoff: float

@dataclass
class AdcState:
    channel: int
    configured: bool
    enabled: bool
    sample_rate: int
    resolution: int
    reference_voltage: float
    last_sample: float
    last_sample_time: int
    sample_count: int
    min_value: float
    max_value: float
    average_value: float
    calibration_enabled: bool
    calibration_offset: float
    calibration_scale: float

@dataclass
class AdcSample:
    channel: int
    value: float
    raw_value: float
    timestamp: int
    deterministic: bool
    tick_count: int

@dataclass
class ActuatorConfig:
    name: str
    type: ActuatorType
    pin: int
    min_value: float
    max_value: float
    default_value: float
    frequency: int
    resolution: int
    enable_safety_limits: bool
    safety_min: float
    safety_max: float
    ramp_time_ms: int
    enable_ramping: bool

@dataclass
class ActuatorState:
    name: str
    type: ActuatorType
    configured: bool
    enabled: bool
    current_value: float
    target_value: float
    last_update: int
    update_count: int
    safety_enabled: bool
    safety_min: float
    safety_max: float
    ramping_enabled: bool
    ramp_time_ms: int
    deterministic: bool

@dataclass
class ActuatorPatternStep:
    value: float
    duration_ms: int

@dataclass
class ActuatorPattern:
    name: str
    step_count: int
    steps: List[ActuatorPatternStep]
    loop: bool
    loop_count: int

@dataclass
class DeviceSnapshot:
    snapshot_id: str
    ngfs_path: str
    created_at: datetime
    gpio_pin_count: int
    adc_channel_count: int
    actuator_count: int
    metadata: Dict[str, Any]

class DeviceValidator:
    """Main Device validation class for GPIO, ADC, and Actuator operations"""
    
    def __init__(self, base_url: str = "http://localhost:8080", api_key: Optional[str] = None):
        self.base_url = base_url
        self.api_key = api_key
        self.session: Optional[aiohttp.ClientSession] = None
        self.test_results: List[Dict[str, Any]] = []
        self.configured_gpio_pins: Dict[int, GpioState] = {}
        self.configured_adc_channels: Dict[int, AdcState] = {}
        self.configured_actuators: Dict[str, ActuatorState] = {}
        
    async def __aenter__(self):
        self.session = aiohttp.ClientSession(
            headers={"Authorization": f"Bearer {self.api_key}"} if self.api_key else {}
        )
        return self
        
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()

    async def make_request(self, endpoint: str, method: str = "GET", data: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """Make HTTP request to device service"""
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

    async def validate_gpio_operations(self) -> Dict[str, Any]:
        """Validate GPIO operations"""
        logger.info("Validating GPIO operations...")
        
        try:
            # Test GPIO pin configuration
            gpio_config = GpioConfig(
                pin=2,
                mode=GpioMode.OUTPUT,
                pull=GpioPull.NONE,
                initial_value=False,
                debounce_ms=0
            )
            
            session_id = f"gpio_test_{uuid.uuid4().hex[:8]}"
            caps = "device:gpio.configure"
            
            config_data = {
                "session_id": session_id,
                "caps": caps,
                "pin": gpio_config.pin,
                "mode": gpio_config.mode.value,
                "pull": gpio_config.pull.value,
                "initial_value": gpio_config.initial_value,
                "debounce_ms": gpio_config.debounce_ms
            }
            
            response = await self.make_request("/gpio/configure", "POST", config_data)
            
            if not response.get("success"):
                return {
                    "test": "gpio_operations",
                    "status": "failed",
                    "error": f"GPIO configuration failed: {response.get('error')}"
                }
            
            # Test GPIO pin write
            write_data = {
                "session_id": session_id,
                "caps": "device:gpio.write",
                "pin": gpio_config.pin,
                "value": True
            }
            
            response = await self.make_request("/gpio/write", "POST", write_data)
            
            if not response.get("success"):
                return {
                    "test": "gpio_operations",
                    "status": "failed",
                    "error": f"GPIO write failed: {response.get('error')}"
                }
            
            # Test GPIO pin read
            read_data = {
                "session_id": session_id,
                "caps": "device:gpio.read",
                "pin": gpio_config.pin
            }
            
            response = await self.make_request("/gpio/read", "POST", read_data)
            
            if not response.get("success"):
                return {
                    "test": "gpio_operations",
                    "status": "failed",
                    "error": f"GPIO read failed: {response.get('error')}"
                }
            
            read_value = response.get("value")
            
            # Test GPIO pin toggle
            toggle_data = {
                "session_id": session_id,
                "caps": "device:gpio.write",
                "pin": gpio_config.pin
            }
            
            response = await self.make_request("/gpio/toggle", "POST", toggle_data)
            
            if not response.get("success"):
                return {
                    "test": "gpio_operations",
                    "status": "failed",
                    "error": f"GPIO toggle failed: {response.get('error')}"
                }
            
            new_value = response.get("new_value")
            
            # Test GPIO pin list
            list_response = await self.make_request("/gpio/list")
            
            if not list_response.get("success"):
                return {
                    "test": "gpio_operations",
                    "status": "failed",
                    "error": f"GPIO list failed: {list_response.get('error')}"
                }
            
            pins = list_response.get("pins", [])
            
            return {
                "test": "gpio_operations",
                "status": "passed",
                "pin": gpio_config.pin,
                "read_value": read_value,
                "toggle_value": new_value,
                "total_pins": len(pins),
                "details": {
                    "config": asdict(gpio_config),
                    "pins": pins[:5]  # First 5 pins for details
                }
            }
            
        except Exception as e:
            return {
                "test": "gpio_operations",
                "status": "failed",
                "error": str(e)
            }

    async def validate_adc_operations(self) -> Dict[str, Any]:
        """Validate ADC operations"""
        logger.info("Validating ADC operations...")
        
        try:
            # Test ADC channel configuration
            adc_config = AdcConfig(
                channel=0,
                sample_rate=1000,
                resolution=12,
                reference_voltage=3.3,
                enable_calibration=False,
                calibration_offset=0.0,
                calibration_scale=1.0,
                oversampling=1,
                enable_filtering=False,
                filter_cutoff=100.0
            )
            
            session_id = f"adc_test_{uuid.uuid4().hex[:8]}"
            caps = "device:adc.configure"
            
            config_data = {
                "session_id": session_id,
                "caps": caps,
                "channel": adc_config.channel,
                "sample_rate": adc_config.sample_rate,
                "resolution": adc_config.resolution,
                "reference_voltage": adc_config.reference_voltage,
                "enable_calibration": adc_config.enable_calibration,
                "calibration_offset": adc_config.calibration_offset,
                "calibration_scale": adc_config.calibration_scale,
                "oversampling": adc_config.oversampling,
                "enable_filtering": adc_config.enable_filtering,
                "filter_cutoff": adc_config.filter_cutoff
            }
            
            response = await self.make_request("/adc/configure", "POST", config_data)
            
            if not response.get("success"):
                return {
                    "test": "adc_operations",
                    "status": "failed",
                    "error": f"ADC configuration failed: {response.get('error')}"
                }
            
            # Test ADC channel enable
            enable_data = {
                "session_id": session_id,
                "caps": "device:adc.enable",
                "channel": adc_config.channel
            }
            
            response = await self.make_request("/adc/enable", "POST", enable_data)
            
            if not response.get("success"):
                return {
                    "test": "adc_operations",
                    "status": "failed",
                    "error": f"ADC enable failed: {response.get('error')}"
                }
            
            # Test ADC channel sampling
            sample_data = {
                "session_id": session_id,
                "caps": "device:adc.sample",
                "channel": adc_config.channel
            }
            
            response = await self.make_request("/adc/sample", "POST", sample_data)
            
            if not response.get("success"):
                return {
                    "test": "adc_operations",
                    "status": "failed",
                    "error": f"ADC sampling failed: {response.get('error')}"
                }
            
            sample = response.get("sample", {})
            
            # Test ADC channel disable
            disable_data = {
                "session_id": session_id,
                "caps": "device:adc.disable",
                "channel": adc_config.channel
            }
            
            response = await self.make_request("/adc/disable", "POST", disable_data)
            
            if not response.get("success"):
                return {
                    "test": "adc_operations",
                    "status": "failed",
                    "error": f"ADC disable failed: {response.get('error')}"
                }
            
            # Test ADC channel list
            list_response = await self.make_request("/adc/list")
            
            if not list_response.get("success"):
                return {
                    "test": "adc_operations",
                    "status": "failed",
                    "error": f"ADC list failed: {list_response.get('error')}"
                }
            
            channels = list_response.get("channels", [])
            
            return {
                "test": "adc_operations",
                "status": "passed",
                "channel": adc_config.channel,
                "sample": sample,
                "total_channels": len(channels),
                "details": {
                    "config": asdict(adc_config),
                    "channels": channels[:5]  # First 5 channels for details
                }
            }
            
        except Exception as e:
            return {
                "test": "adc_operations",
                "status": "failed",
                "error": str(e)
            }

    async def validate_actuator_operations(self) -> Dict[str, Any]:
        """Validate Actuator operations"""
        logger.info("Validating Actuator operations...")
        
        try:
            # Test actuator configuration
            actuator_config = ActuatorConfig(
                name="test_led",
                type=ActuatorType.LED,
                pin=3,
                min_value=0.0,
                max_value=1.0,
                default_value=0.0,
                frequency=1000,
                resolution=8,
                enable_safety_limits=True,
                safety_min=0.0,
                safety_max=1.0,
                ramp_time_ms=100,
                enable_ramping=False
            )
            
            session_id = f"actuator_test_{uuid.uuid4().hex[:8]}"
            caps = "device:actuator.configure"
            
            config_data = {
                "session_id": session_id,
                "caps": caps,
                "name": actuator_config.name,
                "type": actuator_config.type.value,
                "pin": actuator_config.pin,
                "min_value": actuator_config.min_value,
                "max_value": actuator_config.max_value,
                "default_value": actuator_config.default_value,
                "frequency": actuator_config.frequency,
                "resolution": actuator_config.resolution,
                "enable_safety_limits": actuator_config.enable_safety_limits,
                "safety_min": actuator_config.safety_min,
                "safety_max": actuator_config.safety_max,
                "ramp_time_ms": actuator_config.ramp_time_ms,
                "enable_ramping": actuator_config.enable_ramping
            }
            
            response = await self.make_request("/actuator/configure", "POST", config_data)
            
            if not response.get("success"):
                return {
                    "test": "actuator_operations",
                    "status": "failed",
                    "error": f"Actuator configuration failed: {response.get('error')}"
                }
            
            # Test actuator enable
            enable_data = {
                "session_id": session_id,
                "caps": "device:actuator.enable",
                "name": actuator_config.name
            }
            
            response = await self.make_request("/actuator/enable", "POST", enable_data)
            
            if not response.get("success"):
                return {
                    "test": "actuator_operations",
                    "status": "failed",
                    "error": f"Actuator enable failed: {response.get('error')}"
                }
            
            # Test actuator value set
            set_data = {
                "session_id": session_id,
                "caps": "device:actuator.set",
                "name": actuator_config.name,
                "value": 0.5
            }
            
            response = await self.make_request("/actuator/set", "POST", set_data)
            
            if not response.get("success"):
                return {
                    "test": "actuator_operations",
                    "status": "failed",
                    "error": f"Actuator set failed: {response.get('error')}"
                }
            
            # Test actuator value get
            get_response = await self.make_request(f"/actuator/get/{actuator_config.name}")
            
            if not get_response.get("success"):
                return {
                    "test": "actuator_operations",
                    "status": "failed",
                    "error": f"Actuator get failed: {get_response.get('error')}"
                }
            
            current_value = get_response.get("value")
            
            # Test actuator pattern
            pattern = ActuatorPattern(
                name="test_pattern",
                step_count=3,
                steps=[
                    ActuatorPatternStep(value=0.0, duration_ms=100),
                    ActuatorPatternStep(value=0.5, duration_ms=200),
                    ActuatorPatternStep(value=1.0, duration_ms=100)
                ],
                loop=True,
                loop_count=0
            )
            
            pattern_data = {
                "session_id": session_id,
                "caps": "device:actuator.pattern",
                "name": actuator_config.name,
                "pattern_name": pattern.name,
                "steps": [{"value": step.value, "duration_ms": step.duration_ms} for step in pattern.steps],
                "step_count": pattern.step_count,
                "loop": pattern.loop,
                "loop_count": pattern.loop_count
            }
            
            response = await self.make_request("/actuator/pattern", "POST", pattern_data)
            
            if not response.get("success"):
                return {
                    "test": "actuator_operations",
                    "status": "failed",
                    "error": f"Actuator pattern failed: {response.get('error')}"
                }
            
            # Test actuator disable
            disable_data = {
                "session_id": session_id,
                "caps": "device:actuator.disable",
                "name": actuator_config.name
            }
            
            response = await self.make_request("/actuator/disable", "POST", disable_data)
            
            if not response.get("success"):
                return {
                    "test": "actuator_operations",
                    "status": "failed",
                    "error": f"Actuator disable failed: {response.get('error')}"
                }
            
            # Test actuator list
            list_response = await self.make_request("/actuator/list")
            
            if not list_response.get("success"):
                return {
                    "test": "actuator_operations",
                    "status": "failed",
                    "error": f"Actuator list failed: {list_response.get('error')}"
                }
            
            actuators = list_response.get("actuators", [])
            
            return {
                "test": "actuator_operations",
                "status": "passed",
                "actuator_name": actuator_config.name,
                "current_value": current_value,
                "total_actuators": len(actuators),
                "details": {
                    "config": asdict(actuator_config),
                    "pattern": asdict(pattern),
                    "actuators": actuators[:5]  # First 5 actuators for details
                }
            }
            
        except Exception as e:
            return {
                "test": "actuator_operations",
                "status": "failed",
                "error": str(e)
            }

    async def validate_deterministic_operations(self) -> Dict[str, Any]:
        """Validate deterministic device operations"""
        logger.info("Validating deterministic device operations...")
        
        try:
            # Test deterministic mode enable
            response = await self.make_request("/gpio/deterministic/enable", "POST")
            
            if not response.get("success"):
                return {
                    "test": "deterministic_operations",
                    "status": "failed",
                    "error": f"GPIO deterministic enable failed: {response.get('error')}"
                }
            
            # Test deterministic mode enable for ADC
            response = await self.make_request("/adc/deterministic/enable", "POST")
            
            if not response.get("success"):
                return {
                    "test": "deterministic_operations",
                    "status": "failed",
                    "error": f"ADC deterministic enable failed: {response.get('error')}"
                }
            
            # Test deterministic mode enable for Actuator
            response = await self.make_request("/actuator/deterministic/enable", "POST")
            
            if not response.get("success"):
                return {
                    "test": "deterministic_operations",
                    "status": "failed",
                    "error": f"Actuator deterministic enable failed: {response.get('error')}"
                }
            
            # Test tick advance
            response = await self.make_request("/gpio/tick/advance", "POST")
            
            if not response.get("success"):
                return {
                    "test": "deterministic_operations",
                    "status": "failed",
                    "error": f"GPIO tick advance failed: {response.get('error')}"
                }
            
            # Test tick counter
            response = await self.make_request("/gpio/tick/counter")
            
            if not response.get("success"):
                return {
                    "test": "deterministic_operations",
                    "status": "failed",
                    "error": f"GPIO tick counter failed: {response.get('error')}"
                }
            
            tick_count = response.get("tick_count")
            
            # Test deterministic mode disable
            response = await self.make_request("/gpio/deterministic/disable", "POST")
            
            if not response.get("success"):
                return {
                    "test": "deterministic_operations",
                    "status": "failed",
                    "error": f"GPIO deterministic disable failed: {response.get('error')}"
                }
            
            return {
                "test": "deterministic_operations",
                "status": "passed",
                "tick_count": tick_count,
                "details": {
                    "deterministic_enabled": True,
                    "tick_advanced": True
                }
            }
            
        except Exception as e:
            return {
                "test": "deterministic_operations",
                "status": "failed",
                "error": str(e)
            }

    async def validate_snapshot_operations(self) -> Dict[str, Any]:
        """Validate device snapshot operations"""
        logger.info("Validating device snapshot operations...")
        
        try:
            # Test GPIO snapshot creation
            output_path = f"snaps/gpio_test_{uuid.uuid4().hex[:8]}.ngfs"
            
            snapshot_data = {
                "output_path": output_path
            }
            
            response = await self.make_request("/gpio/snapshot/create", "POST", snapshot_data)
            
            if not response.get("success"):
                return {
                    "test": "snapshot_operations",
                    "status": "failed",
                    "error": f"GPIO snapshot creation failed: {response.get('error')}"
                }
            
            gpio_snapshot_id = response.get("snapshot_id")
            
            # Test ADC snapshot creation
            output_path = f"snaps/adc_test_{uuid.uuid4().hex[:8]}.ngfs"
            
            snapshot_data = {
                "output_path": output_path
            }
            
            response = await self.make_request("/adc/snapshot/create", "POST", snapshot_data)
            
            if not response.get("success"):
                return {
                    "test": "snapshot_operations",
                    "status": "failed",
                    "error": f"ADC snapshot creation failed: {response.get('error')}"
                }
            
            adc_snapshot_id = response.get("snapshot_id")
            
            # Test Actuator snapshot creation
            output_path = f"snaps/actuator_test_{uuid.uuid4().hex[:8]}.ngfs"
            
            snapshot_data = {
                "output_path": output_path
            }
            
            response = await self.make_request("/actuator/snapshot/create", "POST", snapshot_data)
            
            if not response.get("success"):
                return {
                    "test": "snapshot_operations",
                    "status": "failed",
                    "error": f"Actuator snapshot creation failed: {response.get('error')}"
                }
            
            actuator_snapshot_id = response.get("snapshot_id")
            
            return {
                "test": "snapshot_operations",
                "status": "passed",
                "gpio_snapshot_id": gpio_snapshot_id,
                "adc_snapshot_id": adc_snapshot_id,
                "actuator_snapshot_id": actuator_snapshot_id,
                "details": {
                    "snapshots_created": 3,
                    "gpio_snapshot": gpio_snapshot_id,
                    "adc_snapshot": adc_snapshot_id,
                    "actuator_snapshot": actuator_snapshot_id
                }
            }
            
        except Exception as e:
            return {
                "test": "snapshot_operations",
                "status": "failed",
                "error": str(e)
            }

    async def validate_performance(self) -> Dict[str, Any]:
        """Validate device performance metrics"""
        logger.info("Validating device performance...")
        
        try:
            # Test GPIO performance
            start_time = time.time()
            
            # Configure multiple GPIO pins
            for pin in range(5):
                config_data = {
                    "session_id": f"perf_test_{uuid.uuid4().hex[:8]}",
                    "caps": "device:gpio.configure",
                    "pin": pin,
                    "mode": "output",
                    "pull": "none",
                    "initial_value": False,
                    "debounce_ms": 0
                }
                
                response = await self.make_request("/gpio/configure", "POST", config_data)
                
                if not response.get("success"):
                    return {
                        "test": "performance",
                        "status": "failed",
                        "error": f"GPIO configuration {pin} failed: {response.get('error')}"
                    }
            
            # Test GPIO write performance
            write_start_time = time.time()
            
            for pin in range(5):
                write_data = {
                    "session_id": f"perf_test_{uuid.uuid4().hex[:8]}",
                    "caps": "device:gpio.write",
                    "pin": pin,
                    "value": True
                }
                
                response = await self.make_request("/gpio/write", "POST", write_data)
                
                if not response.get("success"):
                    return {
                        "test": "performance",
                        "status": "failed",
                        "error": f"GPIO write {pin} failed: {response.get('error')}"
                    }
            
            write_end_time = time.time()
            write_time = write_end_time - write_start_time
            
            # Test ADC performance
            adc_start_time = time.time()
            
            # Configure multiple ADC channels
            for channel in range(3):
                config_data = {
                    "session_id": f"perf_test_{uuid.uuid4().hex[:8]}",
                    "caps": "device:adc.configure",
                    "channel": channel,
                    "sample_rate": 1000,
                    "resolution": 12,
                    "reference_voltage": 3.3,
                    "enable_calibration": False,
                    "calibration_offset": 0.0,
                    "calibration_scale": 1.0,
                    "oversampling": 1,
                    "enable_filtering": False,
                    "filter_cutoff": 100.0
                }
                
                response = await self.make_request("/adc/configure", "POST", config_data)
                
                if not response.get("success"):
                    return {
                        "test": "performance",
                        "status": "failed",
                        "error": f"ADC configuration {channel} failed: {response.get('error')}"
                    }
                
                # Enable channel
                enable_data = {
                    "session_id": f"perf_test_{uuid.uuid4().hex[:8]}",
                    "caps": "device:adc.enable",
                    "channel": channel
                }
                
                response = await self.make_request("/adc/enable", "POST", enable_data)
                
                if not response.get("success"):
                    return {
                        "test": "performance",
                        "status": "failed",
                        "error": f"ADC enable {channel} failed: {response.get('error')}"
                    }
            
            # Test ADC sampling performance
            sample_start_time = time.time()
            
            for channel in range(3):
                sample_data = {
                    "session_id": f"perf_test_{uuid.uuid4().hex[:8]}",
                    "caps": "device:adc.sample",
                    "channel": channel
                }
                
                response = await self.make_request("/adc/sample", "POST", sample_data)
                
                if not response.get("success"):
                    return {
                        "test": "performance",
                        "status": "failed",
                        "error": f"ADC sampling {channel} failed: {response.get('error')}"
                    }
            
            sample_end_time = time.time()
            sample_time = sample_end_time - sample_start_time
            
            # Test Actuator performance
            actuator_start_time = time.time()
            
            # Configure multiple actuators
            for i in range(3):
                config_data = {
                    "session_id": f"perf_test_{uuid.uuid4().hex[:8]}",
                    "caps": "device:actuator.configure",
                    "name": f"perf_actuator_{i}",
                    "type": "led",
                    "pin": i + 10,
                    "min_value": 0.0,
                    "max_value": 1.0,
                    "default_value": 0.0,
                    "frequency": 1000,
                    "resolution": 8,
                    "enable_safety_limits": True,
                    "safety_min": 0.0,
                    "safety_max": 1.0,
                    "ramp_time_ms": 100,
                    "enable_ramping": False
                }
                
                response = await self.make_request("/actuator/configure", "POST", config_data)
                
                if not response.get("success"):
                    return {
                        "test": "performance",
                        "status": "failed",
                        "error": f"Actuator configuration {i} failed: {response.get('error')}"
                    }
                
                # Enable actuator
                enable_data = {
                    "session_id": f"perf_test_{uuid.uuid4().hex[:8]}",
                    "caps": "device:actuator.enable",
                    "name": f"perf_actuator_{i}"
                }
                
                response = await self.make_request("/actuator/enable", "POST", enable_data)
                
                if not response.get("success"):
                    return {
                        "test": "performance",
                        "status": "failed",
                        "error": f"Actuator enable {i} failed: {response.get('error')}"
                    }
            
            # Test Actuator set performance
            set_start_time = time.time()
            
            for i in range(3):
                set_data = {
                    "session_id": f"perf_test_{uuid.uuid4().hex[:8]}",
                    "caps": "device:actuator.set",
                    "name": f"perf_actuator_{i}",
                    "value": 0.5
                }
                
                response = await self.make_request("/actuator/set", "POST", set_data)
                
                if not response.get("success"):
                    return {
                        "test": "performance",
                        "status": "failed",
                        "error": f"Actuator set {i} failed: {response.get('error')}"
                    }
            
            set_end_time = time.time()
            set_time = set_end_time - set_start_time
            
            end_time = time.time()
            total_time = end_time - start_time
            
            return {
                "test": "performance",
                "status": "passed",
                "total_time": total_time,
                "gpio_write_time": write_time,
                "adc_sample_time": sample_time,
                "actuator_set_time": set_time,
                "details": {
                    "gpio_pins_configured": 5,
                    "adc_channels_configured": 3,
                    "actuators_configured": 3,
                    "operations_per_second": 11 / total_time if total_time > 0 else 0
                }
            }
            
        except Exception as e:
            return {
                "test": "performance",
                "status": "failed",
                "error": str(e)
            }

    async def run_all_tests(self) -> Dict[str, Any]:
        """Run all device validation tests"""
        logger.info("Starting comprehensive device validation...")
        
        test_results = []
        
        # Test GPIO operations
        result = await self.validate_gpio_operations()
        test_results.append(result)
        
        # Test ADC operations
        result = await self.validate_adc_operations()
        test_results.append(result)
        
        # Test Actuator operations
        result = await self.validate_actuator_operations()
        test_results.append(result)
        
        # Test deterministic operations
        result = await self.validate_deterministic_operations()
        test_results.append(result)
        
        # Test snapshot operations
        result = await self.validate_snapshot_operations()
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
        
        logger.info(f"Device validation completed: {passed_tests}/{total_tests} tests passed")
        
        return summary

# Test suite for pytest
class TestDeviceValidator:
    """Pytest test suite for device validation"""
    
    @pytest.fixture
    async def device_validator(self):
        async with DeviceValidator() as validator:
            yield validator
    
    @pytest.mark.asyncio
    async def test_gpio_operations(self, device_validator):
        """Test GPIO operations"""
        result = await device_validator.validate_gpio_operations()
        assert result["status"] == "passed", f"GPIO operations failed: {result.get('error')}"
    
    @pytest.mark.asyncio
    async def test_adc_operations(self, device_validator):
        """Test ADC operations"""
        result = await device_validator.validate_adc_operations()
        assert result["status"] == "passed", f"ADC operations failed: {result.get('error')}"
    
    @pytest.mark.asyncio
    async def test_actuator_operations(self, device_validator):
        """Test Actuator operations"""
        result = await device_validator.validate_actuator_operations()
        assert result["status"] == "passed", f"Actuator operations failed: {result.get('error')}"
    
    @pytest.mark.asyncio
    async def test_deterministic_operations(self, device_validator):
        """Test deterministic operations"""
        result = await device_validator.validate_deterministic_operations()
        assert result["status"] == "passed", f"Deterministic operations failed: {result.get('error')}"
    
    @pytest.mark.asyncio
    async def test_snapshot_operations(self, device_validator):
        """Test snapshot operations"""
        result = await device_validator.validate_snapshot_operations()
        assert result["status"] == "passed", f"Snapshot operations failed: {result.get('error')}"
    
    @pytest.mark.asyncio
    async def test_performance(self, device_validator):
        """Test performance metrics"""
        result = await device_validator.validate_performance()
        assert result["status"] == "passed", f"Performance test failed: {result.get('error')}"

# CLI interface
async def main():
    """Main CLI function"""
    import argparse
    
    parser = argparse.ArgumentParser(description="Device Validator for Aetheris OS")
    parser.add_argument("--base-url", default="http://localhost:8080", help="Base URL for device service")
    parser.add_argument("--api-key", help="API key for authentication")
    parser.add_argument("--output", help="Output file for results")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    async with DeviceValidator(args.base_url, args.api_key) as validator:
        results = await validator.run_all_tests()
        
        if args.output:
            with open(args.output, 'w') as f:
                json.dump(results, f, indent=2, default=str)
            print(f"Results saved to {args.output}")
        else:
            print(json.dumps(results, indent=2, default=str))

if __name__ == "__main__":
    asyncio.run(main())
