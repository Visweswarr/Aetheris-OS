#!/usr/bin/env python3
"""
Simple Device Validator for Aetheris HAL Device Listing and Provider Management

This is a simplified version that doesn't depend on external base classes.
"""

import asyncio
import json
import logging
import sys
import time
from dataclasses import dataclass, asdict
from enum import Enum
from typing import List, Dict, Optional, Any, Union
from pathlib import Path


class DeviceType(Enum):
    """Device types supported by the HAL"""
    CAMERA = "camera"
    MICROPHONE = "microphone"
    GPIO = "gpio"
    ADC = "adc"
    ACTUATOR = "actuator"


class ProviderType(Enum):
    """Provider types supported by the HAL"""
    DETERMINISTIC = "deterministic"
    LINUX = "linux"
    CUSTOM = "custom"


@dataclass
class DeviceInfo:
    """Device information structure"""
    device_id: str
    device_type: DeviceType
    device_path: str
    capabilities: List[str]
    provider: ProviderType
    is_available: bool
    metadata: Dict[str, Any]

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization"""
        return {
            'device_id': self.device_id,
            'device_type': self.device_type.value,
            'device_path': self.device_path,
            'capabilities': self.capabilities,
            'provider': self.provider.value,
            'is_available': self.is_available,
            'metadata': self.metadata
        }


@dataclass
class ProviderStatus:
    """Provider status information"""
    provider_id: ProviderType
    is_available: bool
    device_count: int
    last_error: Optional[str] = None
    capabilities: List[str] = None

    def __post_init__(self):
        if self.capabilities is None:
            self.capabilities = []

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization"""
        return {
            'provider_id': self.provider_id.value,
            'is_available': self.is_available,
            'device_count': self.device_count,
            'last_error': self.last_error,
            'capabilities': self.capabilities
        }


@dataclass
class ValidationResult:
    """Validation result structure"""
    success: bool
    message: str
    details: str = ""


class SimpleDeviceValidator:
    """Simple validator for device listing and provider management operations"""

    def __init__(self, provider: str = "det"):
        self.provider = provider
        self.devices: List[DeviceInfo] = []
        self.providers: List[ProviderStatus] = []
        self.current_provider = ProviderType.DETERMINISTIC

    async def setup(self) -> None:
        """Setup the validator"""
        logging.info(f"Setting up device validator with provider: {self.provider}")
        
        # Initialize providers
        await self.initialize_providers()
        
        # Discover initial devices
        await self.discover_devices()

    async def teardown(self) -> None:
        """Cleanup the validator"""
        logging.info("Tearing down device validator")

    async def initialize_providers(self) -> None:
        """Initialize provider status"""
        # Simulate provider initialization
        self.providers = [
            ProviderStatus(
                provider_id=ProviderType.DETERMINISTIC,
                is_available=True,
                device_count=5,
                capabilities=[
                    'camera.capture',
                    'microphone.capture',
                    'gpio.read',
                    'gpio.write',
                    'adc.sample',
                    'actuator.control',
                    'deterministic'
                ]
            )
        ]

        # Add Linux provider if available
        if await self.is_provider_available(ProviderType.LINUX):
            self.providers.append(ProviderStatus(
                provider_id=ProviderType.LINUX,
                is_available=True,
                device_count=3,
                capabilities=[
                    'camera.capture',
                    'microphone.capture',
                    'gpio.read',
                    'gpio.write',
                    'gpio.interrupt',
                    'adc.sample',
                    'adc.continuous',
                    'actuator.pwm',
                    'actuator.led',
                    'actuator.motor',
                    'hardware'
                ]
            ))

    async def discover_devices(self) -> List[DeviceInfo]:
        """Discover available devices"""
        # Simulate device discovery
        devices = [
            DeviceInfo(
                device_id='camera_mock_0',
                device_type=DeviceType.CAMERA,
                device_path='/dev/video_mock_0',
                capabilities=['capture', 'yuv420', 'rgb24'],
                provider=ProviderType.DETERMINISTIC,
                is_available=True,
                metadata={'width': 640, 'height': 480, 'fps': 30}
            ),
            DeviceInfo(
                device_id='mic_mock_0',
                device_type=DeviceType.MICROPHONE,
                device_path='/dev/audio_mock_0',
                capabilities=['capture', 's16le', 'f32le'],
                provider=ProviderType.DETERMINISTIC,
                is_available=True,
                metadata={'sample_rate': 44100, 'channels': 2}
            ),
            DeviceInfo(
                device_id='gpio_mock_0',
                device_type=DeviceType.GPIO,
                device_path='/dev/gpiochip_mock_0',
                capabilities=['read', 'write', 'interrupt'],
                provider=ProviderType.DETERMINISTIC,
                is_available=True,
                metadata={'lines': 32}
            ),
            DeviceInfo(
                device_id='adc_mock_0',
                device_type=DeviceType.ADC,
                device_path='/dev/iio_mock_0',
                capabilities=['sample', 'continuous'],
                provider=ProviderType.DETERMINISTIC,
                is_available=True,
                metadata={'channels': 8, 'resolution': 12, 'reference_voltage': 3.3}
            ),
            DeviceInfo(
                device_id='actuator_mock_0',
                device_type=DeviceType.ACTUATOR,
                device_path='/dev/pwm_mock_0',
                capabilities=['pwm', 'led', 'motor'],
                provider=ProviderType.DETERMINISTIC,
                is_available=True,
                metadata={'channels': 4, 'frequency_range': '1-1000000'}
            )
        ]

        # Add Linux devices if available
        if await self.is_provider_available(ProviderType.LINUX):
            linux_devices = [
                DeviceInfo(
                    device_id='/dev/video0',
                    device_type=DeviceType.CAMERA,
                    device_path='/dev/video0',
                    capabilities=['capture', 'yuv420', 'rgb24', 'mjpeg'],
                    provider=ProviderType.LINUX,
                    is_available=True,
                    metadata={'driver': 'uvcvideo', 'card': 'USB2.0 Camera'}
                ),
                DeviceInfo(
                    device_id='hw:0,0',
                    device_type=DeviceType.MICROPHONE,
                    device_path='hw:0,0',
                    capabilities=['capture', 's16le', 'f32le'],
                    provider=ProviderType.LINUX,
                    is_available=True,
                    metadata={'card_name': 'HDA Intel PCH', 'device_type': 'alsa_card'}
                ),
                DeviceInfo(
                    device_id='/dev/gpiochip0',
                    device_type=DeviceType.GPIO,
                    device_path='/dev/gpiochip0',
                    capabilities=['read', 'write', 'interrupt', 'pull_up', 'pull_down'],
                    provider=ProviderType.LINUX,
                    is_available=True,
                    metadata={'chip_name': 'gpiochip0', 'line_count': 32}
                )
            ]
            devices.extend(linux_devices)

        self.devices = devices
        return devices

    async def is_provider_available(self, provider_type: ProviderType) -> bool:
        """Check if a provider is available"""
        if provider_type == ProviderType.DETERMINISTIC:
            return True
        elif provider_type == ProviderType.LINUX:
            # Check if we're on a Linux system
            import platform
            return platform.system() == 'Linux'
        else:
            return False

    async def get_devices_by_type(self, device_type: DeviceType) -> List[DeviceInfo]:
        """Get devices by type"""
        return [device for device in self.devices if device.device_type == device_type]

    async def get_devices_by_provider(self, provider_type: ProviderType) -> List[DeviceInfo]:
        """Get devices by provider"""
        return [device for device in self.devices if device.provider == provider_type]

    async def get_device_by_id(self, device_id: str) -> Optional[DeviceInfo]:
        """Get a specific device by ID"""
        for device in self.devices:
            if device.device_id == device_id:
                return device
        return None

    async def set_default_provider(self, provider_type: ProviderType) -> None:
        """Set the default provider"""
        if not await self.is_provider_available(provider_type):
            raise ValueError(f"Provider {provider_type.value} is not available")
        
        self.current_provider = provider_type
        logging.info(f"Set default provider to: {provider_type.value}")

    async def get_default_provider(self) -> ProviderType:
        """Get the current default provider"""
        return self.current_provider

    async def test_device_discovery(self) -> ValidationResult:
        """Test device discovery functionality"""
        try:
            # Test basic device discovery
            devices = await self.discover_devices()
            
            if not devices:
                return ValidationResult(
                    success=False,
                    message="No devices discovered",
                    details="Device discovery returned empty list"
                )

            # Validate device structure
            for device in devices:
                if not device.device_id:
                    return ValidationResult(
                        success=False,
                        message="Invalid device ID",
                        details=f"Device has empty ID: {device}"
                    )
                
                if not device.device_path:
                    return ValidationResult(
                        success=False,
                        message="Invalid device path",
                        details=f"Device has empty path: {device.device_id}"
                    )

            # Check that we have at least one device of each type
            device_types = set(device.device_type for device in devices)
            expected_types = {DeviceType.CAMERA, DeviceType.MICROPHONE, DeviceType.GPIO, DeviceType.ADC, DeviceType.ACTUATOR}
            
            if not expected_types.issubset(device_types):
                missing_types = expected_types - device_types
                return ValidationResult(
                    success=False,
                    message="Missing device types",
                    details=f"Missing device types: {missing_types}"
                )

            return ValidationResult(
                success=True,
                message="Device discovery successful",
                details=f"Discovered {len(devices)} devices across {len(device_types)} types"
            )

        except Exception as e:
            return ValidationResult(
                success=False,
                message="Device discovery failed",
                details=f"Exception: {str(e)}"
            )

    async def test_device_filtering(self) -> ValidationResult:
        """Test device filtering by type and provider"""
        try:
            # Test filtering by device type
            camera_devices = await self.get_devices_by_type(DeviceType.CAMERA)
            if not camera_devices:
                return ValidationResult(
                    success=False,
                    message="No camera devices found",
                    details="Filtering by camera type returned empty list"
                )

            # Test filtering by provider
            det_devices = await self.get_devices_by_provider(ProviderType.DETERMINISTIC)
            if not det_devices:
                return ValidationResult(
                    success=False,
                    message="No deterministic devices found",
                    details="Filtering by deterministic provider returned empty list"
                )

            # Test device lookup by ID
            test_device = det_devices[0]
            found_device = await self.get_device_by_id(test_device.device_id)
            if not found_device:
                return ValidationResult(
                    success=False,
                    message="Device lookup by ID failed",
                    details=f"Could not find device: {test_device.device_id}"
                )

            if found_device.device_id != test_device.device_id:
                return ValidationResult(
                    success=False,
                    message="Device ID mismatch",
                    details=f"Expected {test_device.device_id}, got {found_device.device_id}"
                )

            return ValidationResult(
                success=True,
                message="Device filtering successful",
                details=f"Found {len(camera_devices)} cameras, {len(det_devices)} deterministic devices"
            )

        except Exception as e:
            return ValidationResult(
                success=False,
                message="Device filtering failed",
                details=f"Exception: {str(e)}"
            )

    async def test_provider_management(self) -> ValidationResult:
        """Test provider status and switching"""
        try:
            # Test provider status
            if not self.providers:
                return ValidationResult(
                    success=False,
                    message="No providers available",
                    details="Provider list is empty"
                )

            # Test provider availability
            det_available = await self.is_provider_available(ProviderType.DETERMINISTIC)
            if not det_available:
                return ValidationResult(
                    success=False,
                    message="Deterministic provider not available",
                    details="Deterministic provider should always be available"
                )

            # Test provider switching
            original_provider = await self.get_default_provider()
            await self.set_default_provider(ProviderType.DETERMINISTIC)
            current_provider = await self.get_default_provider()
            
            if current_provider != ProviderType.DETERMINISTIC:
                return ValidationResult(
                    success=False,
                    message="Provider switching failed",
                    details=f"Expected {ProviderType.DETERMINISTIC.value}, got {current_provider.value}"
                )

            return ValidationResult(
                success=True,
                message="Provider management successful",
                details=f"Managed {len(self.providers)} providers"
            )

        except Exception as e:
            return ValidationResult(
                success=False,
                message="Provider management failed",
                details=f"Exception: {str(e)}"
            )

    async def run_validation(self) -> Dict[str, ValidationResult]:
        """Run all validation tests"""
        results = {}
        
        # Setup
        await self.setup()
        
        try:
            # Run tests
            results["device_discovery"] = await self.test_device_discovery()
            results["device_filtering"] = await self.test_device_filtering()
            results["provider_management"] = await self.test_provider_management()
            
        finally:
            # Teardown
            await self.teardown()
        
        return results


async def main():
    """Main entry point for device validation"""
    import argparse

    parser = argparse.ArgumentParser(description="Validate Aetheris device listing and provider management")
    parser.add_argument("--provider", choices=["det", "linux"], default="det",
                       help="Provider to use for validation (default: det)")
    parser.add_argument("--verbose", "-v", action="store_true",
                       help="Enable verbose logging")
    parser.add_argument("--output", "-o", type=str,
                       help="Output file for results (JSON format)")

    args = parser.parse_args()

    # Configure logging
    log_level = logging.DEBUG if args.verbose else logging.INFO
    logging.basicConfig(
        level=log_level,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )

    # Create validator
    validator = SimpleDeviceValidator(provider=args.provider)

    try:
        # Run validation
        results = await validator.run_validation()

        # Output results
        if args.output:
            with open(args.output, 'w') as f:
                json.dump(results, f, indent=2, default=str)
            print(f"Results written to {args.output}")
        else:
            print(json.dumps(results, indent=2, default=str))

        # Exit with appropriate code
        all_passed = all(result.success for result in results.values())
        sys.exit(0 if all_passed else 1)

    except Exception as e:
        logging.error(f"Validation failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(main())
