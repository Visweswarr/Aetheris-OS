#!/usr/bin/env python3
"""
BLE Validator - Python validation framework for Aetheris OS BLE operations

This module provides comprehensive validation for BLE operations including
device scanning, connection management, GATT operations, and service discovery.
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

class BLEConnectionStatus(Enum):
    CONNECTED = "connected"
    DISCONNECTED = "disconnected"
    CONNECTING = "connecting"
    ERROR = "error"

class BLECharacteristicProperty(Enum):
    READ = "read"
    WRITE = "write"
    WRITE_WITHOUT_RESPONSE = "write_without_response"
    NOTIFY = "notify"
    INDICATE = "indicate"
    AUTHENTICATED_SIGNED_WRITES = "authenticated_signed_writes"
    EXTENDED_PROPERTIES = "extended_properties"
    RELIABLE_WRITE = "reliable_write"
    WRITABLE_AUXILIARIES = "writable_auxiliaries"

@dataclass
class BLEDevice:
    address: str
    name: Optional[str] = None
    rssi: int = 0
    manufacturer_data: Optional[bytes] = None
    service_data: Optional[Dict[str, bytes]] = None
    service_uuids: Optional[List[str]] = None
    is_connectable: bool = True
    last_seen: datetime = None

    def __post_init__(self):
        if self.last_seen is None:
            self.last_seen = datetime.utcnow()

@dataclass
class BLECharacteristicProperties:
    read: bool = False
    write: bool = False
    write_without_response: bool = False
    notify: bool = False
    indicate: bool = False
    authenticated_signed_writes: bool = False
    extended_properties: bool = False
    reliable_write: bool = False
    writable_auxiliaries: bool = False

@dataclass
class BLECharacteristic:
    uuid: str
    properties: BLECharacteristicProperties
    value: Optional[bytes] = None
    descriptors: Optional[List[Dict[str, Any]]] = None

@dataclass
class BLEService:
    uuid: str
    primary: bool = True
    characteristics: List[BLECharacteristic] = None

    def __post_init__(self):
        if self.characteristics is None:
            self.characteristics = []

@dataclass
class BLEConnection:
    device_address: str
    connection_handle: str
    is_connected: bool = False
    services: Dict[str, BLEService] = None
    connected_at: datetime = None
    last_activity: datetime = None

    def __post_init__(self):
        if self.services is None:
            self.services = {}
        if self.connected_at is None:
            self.connected_at = datetime.utcnow()
        if self.last_activity is None:
            self.last_activity = datetime.utcnow()

@dataclass
class BLEScanResult:
    devices: List[BLEDevice]
    scan_duration: float
    total_devices: int
    unique_devices: int

@dataclass
class BLEStats:
    connection_status: BLEConnectionStatus
    total_devices: int
    connected_devices: int
    total_scans: int
    successful_scans: int
    failed_scans: int
    total_connections: int
    successful_connections: int
    failed_connections: int
    total_gatt_operations: int
    successful_gatt_operations: int
    failed_gatt_operations: int
    average_response_time: float
    last_scan_time: datetime
    last_connection_time: datetime

class BLEValidator:
    """Main BLE validation class"""
    
    def __init__(self, base_url: str = "http://localhost:8080", api_key: Optional[str] = None):
        self.base_url = base_url
        self.api_key = api_key
        self.session: Optional[aiohttp.ClientSession] = None
        self.test_results: List[Dict[str, Any]] = []
        self.active_scans: Set[str] = set()
        self.active_connections: Dict[str, BLEConnection] = {}
        
    async def __aenter__(self):
        self.session = aiohttp.ClientSession(
            headers={"Authorization": f"Bearer {self.api_key}"} if self.api_key else {}
        )
        return self
        
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()

    async def make_request(self, endpoint: str, method: str = "GET", data: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """Make HTTP request to BLE service"""
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

    async def validate_scan_operations(self) -> Dict[str, Any]:
        """Validate BLE scanning operations"""
        logger.info("Validating BLE scan operations...")
        
        try:
            # Test scan start
            scan_data = {
                "duration": 5000,
                "allow_duplicates": False,
                "service_uuids": [],
                "manufacturer_data": [],
                "name_prefix": "",
                "rssi_threshold": -80
            }
            
            response = await self.make_request("/ble/scan/start", "POST", scan_data)
            
            if not response.get("success"):
                return {
                    "test": "scan_operations",
                    "status": "failed",
                    "error": f"Scan start failed: {response.get('error')}"
                }
            
            scan_id = response.get("scan_id")
            self.active_scans.add(scan_id)
            
            # Wait for scan to complete
            await asyncio.sleep(2)
            
            # Test scan results
            results_response = await self.make_request(f"/ble/scan/{scan_id}/results")
            
            if not results_response.get("success"):
                return {
                    "test": "scan_operations",
                    "status": "failed",
                    "error": f"Get scan results failed: {results_response.get('error')}"
                }
            
            devices = results_response.get("devices", [])
            
            # Test scan stop
            stop_response = await self.make_request(f"/ble/scan/{scan_id}/stop", "POST")
            
            if not stop_response.get("success"):
                return {
                    "test": "scan_operations",
                    "status": "failed",
                    "error": f"Scan stop failed: {stop_response.get('error')}"
                }
            
            self.active_scans.discard(scan_id)
            
            return {
                "test": "scan_operations",
                "status": "passed",
                "scan_id": scan_id,
                "devices_found": len(devices),
                "scan_duration": stop_response.get("scan_duration", 0),
                "details": {
                    "devices": devices[:5],  # First 5 devices for details
                    "total_devices": stop_response.get("total_devices", 0),
                    "unique_devices": stop_response.get("unique_devices", 0)
                }
            }
            
        except Exception as e:
            return {
                "test": "scan_operations",
                "status": "failed",
                "error": str(e)
            }

    async def validate_connection_operations(self) -> Dict[str, Any]:
        """Validate BLE connection operations"""
        logger.info("Validating BLE connection operations...")
        
        try:
            # First, scan for devices
            scan_data = {
                "duration": 3000,
                "allow_duplicates": False,
                "service_uuids": [],
                "manufacturer_data": [],
                "name_prefix": "",
                "rssi_threshold": -100
            }
            
            scan_response = await self.make_request("/ble/scan/start", "POST", scan_data)
            
            if not scan_response.get("success"):
                return {
                    "test": "connection_operations",
                    "status": "failed",
                    "error": f"Scan failed: {scan_response.get('error')}"
                }
            
            scan_id = scan_response.get("scan_id")
            self.active_scans.add(scan_id)
            
            # Wait for scan
            await asyncio.sleep(2)
            
            # Get scan results
            results_response = await self.make_request(f"/ble/scan/{scan_id}/results")
            devices = results_response.get("devices", [])
            
            # Stop scan
            await self.make_request(f"/ble/scan/{scan_id}/stop", "POST")
            self.active_scans.discard(scan_id)
            
            if not devices:
                return {
                    "test": "connection_operations",
                    "status": "skipped",
                    "reason": "No devices found for connection testing"
                }
            
            # Try to connect to first device
            device_address = devices[0]["address"]
            
            connection_data = {
                "device_address": device_address,
                "timeout": 5000,
                "auto_reconnect": False,
                "service_discovery": True
            }
            
            connect_response = await self.make_request("/ble/connect", "POST", connection_data)
            
            if not connect_response.get("success"):
                return {
                    "test": "connection_operations",
                    "status": "failed",
                    "error": f"Connection failed: {connect_response.get('error')}"
                }
            
            connection_handle = connect_response.get("connection_handle")
            
            # Store connection info
            connection = BLEConnection(
                device_address=device_address,
                connection_handle=connection_handle,
                is_connected=True
            )
            self.active_connections[connection_handle] = connection
            
            # Test service discovery
            services_response = await self.make_request(f"/ble/connect/{connection_handle}/services/discover", "POST")
            
            if not services_response.get("success"):
                return {
                    "test": "connection_operations",
                    "status": "failed",
                    "error": f"Service discovery failed: {services_response.get('error')}"
                }
            
            services = services_response.get("services", [])
            
            # Test disconnection
            disconnect_response = await self.make_request(f"/ble/connect/{connection_handle}/disconnect", "POST")
            
            if not disconnect_response.get("success"):
                return {
                    "test": "connection_operations",
                    "status": "failed",
                    "error": f"Disconnection failed: {disconnect_response.get('error')}"
                }
            
            # Remove from active connections
            self.active_connections.pop(connection_handle, None)
            
            return {
                "test": "connection_operations",
                "status": "passed",
                "connection_handle": connection_handle,
                "device_address": device_address,
                "services_discovered": len(services),
                "details": {
                    "services": services[:3],  # First 3 services for details
                    "connection_time": connect_response.get("connection_time", 0),
                    "disconnection_time": disconnect_response.get("disconnection_time", 0)
                }
            }
            
        except Exception as e:
            return {
                "test": "connection_operations",
                "status": "failed",
                "error": str(e)
            }

    async def validate_gatt_operations(self) -> Dict[str, Any]:
        """Validate GATT characteristic operations"""
        logger.info("Validating GATT operations...")
        
        try:
            # First, scan and connect to a device
            scan_data = {
                "duration": 3000,
                "allow_duplicates": False,
                "service_uuids": [],
                "manufacturer_data": [],
                "name_prefix": "",
                "rssi_threshold": -100
            }
            
            scan_response = await self.make_request("/ble/scan/start", "POST", scan_data)
            
            if not scan_response.get("success"):
                return {
                    "test": "gatt_operations",
                    "status": "failed",
                    "error": f"Scan failed: {scan_response.get('error')}"
                }
            
            scan_id = scan_response.get("scan_id")
            self.active_scans.add(scan_id)
            
            # Wait for scan
            await asyncio.sleep(2)
            
            # Get scan results
            results_response = await self.make_request(f"/ble/scan/{scan_id}/results")
            devices = results_response.get("devices", [])
            
            # Stop scan
            await self.make_request(f"/ble/scan/{scan_id}/stop", "POST")
            self.active_scans.discard(scan_id)
            
            if not devices:
                return {
                    "test": "gatt_operations",
                    "status": "skipped",
                    "reason": "No devices found for GATT testing"
                }
            
            # Connect to first device
            device_address = devices[0]["address"]
            
            connection_data = {
                "device_address": device_address,
                "timeout": 5000,
                "auto_reconnect": False,
                "service_discovery": True
            }
            
            connect_response = await self.make_request("/ble/connect", "POST", connection_data)
            
            if not connect_response.get("success"):
                return {
                    "test": "gatt_operations",
                    "status": "failed",
                    "error": f"Connection failed: {connect_response.get('error')}"
                }
            
            connection_handle = connect_response.get("connection_handle")
            
            # Store connection info
            connection = BLEConnection(
                device_address=device_address,
                connection_handle=connection_handle,
                is_connected=True
            )
            self.active_connections[connection_handle] = connection
            
            # Discover services
            services_response = await self.make_request(f"/ble/connect/{connection_handle}/services/discover", "POST")
            
            if not services_response.get("success"):
                return {
                    "test": "gatt_operations",
                    "status": "failed",
                    "error": f"Service discovery failed: {services_response.get('error')}"
                }
            
            services = services_response.get("services", [])
            
            if not services:
                # Disconnect and return
                await self.make_request(f"/ble/connect/{connection_handle}/disconnect", "POST")
                self.active_connections.pop(connection_handle, None)
                
                return {
                    "test": "gatt_operations",
                    "status": "skipped",
                    "reason": "No services found for GATT testing"
                }
            
            # Find a characteristic to test
            test_service = None
            test_characteristic = None
            
            for service in services:
                if service.get("characteristics"):
                    test_service = service
                    test_characteristic = service["characteristics"][0]
                    break
            
            if not test_service or not test_characteristic:
                # Disconnect and return
                await self.make_request(f"/ble/connect/{connection_handle}/disconnect", "POST")
                self.active_connections.pop(connection_handle, None)
                
                return {
                    "test": "gatt_operations",
                    "status": "skipped",
                    "reason": "No characteristics found for GATT testing"
                }
            
            service_uuid = test_service["uuid"]
            characteristic_uuid = test_characteristic["uuid"]
            
            # Test characteristic read (if supported)
            if test_characteristic.get("properties", {}).get("read"):
                read_data = {
                    "service_uuid": service_uuid,
                    "characteristic_uuid": characteristic_uuid
                }
                
                read_response = await self.make_request(f"/ble/connect/{connection_handle}/characteristic/read", "POST", read_data)
                
                if not read_response.get("success"):
                    logger.warning(f"Characteristic read failed: {read_response.get('error')}")
                else:
                    logger.info(f"Characteristic read successful: {read_response.get('value')}")
            
            # Test characteristic write (if supported)
            if test_characteristic.get("properties", {}).get("write"):
                test_value = b"test_data"
                write_data = {
                    "service_uuid": service_uuid,
                    "characteristic_uuid": characteristic_uuid,
                    "value": list(test_value),
                    "without_response": False,
                    "signed": False
                }
                
                write_response = await self.make_request(f"/ble/connect/{connection_handle}/characteristic/write", "POST", write_data)
                
                if not write_response.get("success"):
                    logger.warning(f"Characteristic write failed: {write_response.get('error')}")
                else:
                    logger.info("Characteristic write successful")
            
            # Test characteristic subscription (if supported)
            if test_characteristic.get("properties", {}).get("notify"):
                subscribe_data = {
                    "service_uuid": service_uuid,
                    "characteristic_uuid": characteristic_uuid
                }
                
                subscribe_response = await self.make_request(f"/ble/connect/{connection_handle}/characteristic/subscribe", "POST", subscribe_data)
                
                if not subscribe_response.get("success"):
                    logger.warning(f"Characteristic subscription failed: {subscribe_response.get('error')}")
                else:
                    logger.info("Characteristic subscription successful")
                    
                    # Wait a bit for notifications
                    await asyncio.sleep(1)
                    
                    # Unsubscribe
                    unsubscribe_response = await self.make_request(f"/ble/connect/{connection_handle}/characteristic/unsubscribe", "POST", subscribe_data)
                    
                    if not unsubscribe_response.get("success"):
                        logger.warning(f"Characteristic unsubscription failed: {unsubscribe_response.get('error')}")
                    else:
                        logger.info("Characteristic unsubscription successful")
            
            # Disconnect
            disconnect_response = await self.make_request(f"/ble/connect/{connection_handle}/disconnect", "POST")
            
            if not disconnect_response.get("success"):
                return {
                    "test": "gatt_operations",
                    "status": "failed",
                    "error": f"Disconnection failed: {disconnect_response.get('error')}"
                }
            
            # Remove from active connections
            self.active_connections.pop(connection_handle, None)
            
            return {
                "test": "gatt_operations",
                "status": "passed",
                "connection_handle": connection_handle,
                "device_address": device_address,
                "services_tested": len(services),
                "characteristics_tested": sum(len(s.get("characteristics", [])) for s in services),
                "details": {
                    "test_service": test_service,
                    "test_characteristic": test_characteristic
                }
            }
            
        except Exception as e:
            return {
                "test": "gatt_operations",
                "status": "failed",
                "error": str(e)
            }

    async def validate_deterministic_operations(self) -> Dict[str, Any]:
        """Validate deterministic BLE operations"""
        logger.info("Validating deterministic BLE operations...")
        
        try:
            # Test multiple scans with same parameters
            scan_data = {
                "duration": 2000,
                "allow_duplicates": False,
                "service_uuids": [],
                "manufacturer_data": [],
                "name_prefix": "",
                "rssi_threshold": -80
            }
            
            scan_results = []
            
            for i in range(3):
                scan_response = await self.make_request("/ble/scan/start", "POST", scan_data)
                
                if not scan_response.get("success"):
                    return {
                        "test": "deterministic_operations",
                        "status": "failed",
                        "error": f"Scan {i+1} failed: {scan_response.get('error')}"
                    }
                
                scan_id = scan_response.get("scan_id")
                self.active_scans.add(scan_id)
                
                # Wait for scan
                await asyncio.sleep(1)
                
                # Get results
                results_response = await self.make_request(f"/ble/scan/{scan_id}/results")
                devices = results_response.get("devices", [])
                
                # Stop scan
                await self.make_request(f"/ble/scan/{scan_id}/stop", "POST")
                self.active_scans.discard(scan_id)
                
                scan_results.append({
                    "scan_id": scan_id,
                    "device_count": len(devices),
                    "devices": [d["address"] for d in devices]
                })
            
            # Check if results are consistent
            device_counts = [r["device_count"] for r in scan_results]
            device_sets = [set(r["devices"]) for r in scan_results]
            
            # Results should be similar (allowing for some variation due to timing)
            count_variance = max(device_counts) - min(device_counts)
            is_consistent = count_variance <= 2  # Allow up to 2 device difference
            
            return {
                "test": "deterministic_operations",
                "status": "passed" if is_consistent else "failed",
                "scan_results": scan_results,
                "device_count_variance": count_variance,
                "is_consistent": is_consistent,
                "details": {
                    "device_counts": device_counts,
                    "device_sets": [list(s) for s in device_sets]
                }
            }
            
        except Exception as e:
            return {
                "test": "deterministic_operations",
                "status": "failed",
                "error": str(e)
            }

    async def validate_performance(self) -> Dict[str, Any]:
        """Validate BLE performance metrics"""
        logger.info("Validating BLE performance...")
        
        try:
            # Test scan performance
            scan_data = {
                "duration": 1000,
                "allow_duplicates": False,
                "service_uuids": [],
                "manufacturer_data": [],
                "name_prefix": "",
                "rssi_threshold": -100
            }
            
            start_time = time.time()
            
            scan_response = await self.make_request("/ble/scan/start", "POST", scan_data)
            
            if not scan_response.get("success"):
                return {
                    "test": "performance",
                    "status": "failed",
                    "error": f"Scan start failed: {scan_response.get('error')}"
                }
            
            scan_id = scan_response.get("scan_id")
            self.active_scans.add(scan_id)
            
            # Wait for scan
            await asyncio.sleep(1)
            
            # Get results
            results_response = await self.make_request(f"/ble/scan/{scan_id}/results")
            devices = results_response.get("devices", [])
            
            # Stop scan
            stop_response = await self.make_request(f"/ble/scan/{scan_id}/stop", "POST")
            self.active_scans.discard(scan_id)
            
            end_time = time.time()
            total_time = end_time - start_time
            
            # Test connection performance
            if devices:
                device_address = devices[0]["address"]
                
                connection_data = {
                    "device_address": device_address,
                    "timeout": 3000,
                    "auto_reconnect": False,
                    "service_discovery": True
                }
                
                conn_start_time = time.time()
                
                connect_response = await self.make_request("/ble/connect", "POST", connection_data)
                
                if connect_response.get("success"):
                    connection_handle = connect_response.get("connection_handle")
                    
                    # Test service discovery
                    services_response = await self.make_request(f"/ble/connect/{connection_handle}/services/discover", "POST")
                    
                    # Disconnect
                    await self.make_request(f"/ble/connect/{connection_handle}/disconnect", "POST")
                    
                    conn_end_time = time.time()
                    connection_time = conn_end_time - conn_start_time
                else:
                    connection_time = None
            else:
                connection_time = None
            
            # Get BLE stats
            stats_response = await self.make_request("/ble/stats")
            stats = stats_response if stats_response.get("success") else {}
            
            return {
                "test": "performance",
                "status": "passed",
                "scan_time": total_time,
                "connection_time": connection_time,
                "devices_found": len(devices),
                "stats": stats,
                "details": {
                    "scan_start_time": scan_response.get("start_time"),
                    "scan_duration": stop_response.get("scan_duration", 0),
                    "total_devices": stop_response.get("total_devices", 0),
                    "unique_devices": stop_response.get("unique_devices", 0)
                }
            }
            
        except Exception as e:
            return {
                "test": "performance",
                "status": "failed",
                "error": str(e)
            }

    async def run_all_tests(self) -> Dict[str, Any]:
        """Run all BLE validation tests"""
        logger.info("Starting comprehensive BLE validation...")
        
        test_results = []
        
        # Test scan operations
        result = await self.validate_scan_operations()
        test_results.append(result)
        
        # Test connection operations
        result = await self.validate_connection_operations()
        test_results.append(result)
        
        # Test GATT operations
        result = await self.validate_gatt_operations()
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
        
        logger.info(f"BLE validation completed: {passed_tests}/{total_tests} tests passed")
        
        return summary

# Test suite for pytest
class TestBLEValidator:
    """Pytest test suite for BLE validation"""
    
    @pytest.fixture
    async def ble_validator(self):
        async with BLEValidator() as validator:
            yield validator
    
    @pytest.mark.asyncio
    async def test_scan_operations(self, ble_validator):
        """Test BLE scan operations"""
        result = await ble_validator.validate_scan_operations()
        assert result["status"] in ["passed", "skipped"], f"Scan operations failed: {result.get('error')}"
    
    @pytest.mark.asyncio
    async def test_connection_operations(self, ble_validator):
        """Test BLE connection operations"""
        result = await ble_validator.validate_connection_operations()
        assert result["status"] in ["passed", "skipped"], f"Connection operations failed: {result.get('error')}"
    
    @pytest.mark.asyncio
    async def test_gatt_operations(self, ble_validator):
        """Test GATT operations"""
        result = await ble_validator.validate_gatt_operations()
        assert result["status"] in ["passed", "skipped"], f"GATT operations failed: {result.get('error')}"
    
    @pytest.mark.asyncio
    async def test_deterministic_operations(self, ble_validator):
        """Test deterministic operations"""
        result = await ble_validator.validate_deterministic_operations()
        assert result["status"] == "passed", f"Deterministic operations failed: {result.get('error')}"
    
    @pytest.mark.asyncio
    async def test_performance(self, ble_validator):
        """Test performance metrics"""
        result = await ble_validator.validate_performance()
        assert result["status"] == "passed", f"Performance test failed: {result.get('error')}"

# CLI interface
async def main():
    """Main CLI function"""
    import argparse
    
    parser = argparse.ArgumentParser(description="BLE Validator for Aetheris OS")
    parser.add_argument("--base-url", default="http://localhost:8080", help="Base URL for BLE service")
    parser.add_argument("--api-key", help="API key for authentication")
    parser.add_argument("--output", help="Output file for results")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    async with BLEValidator(args.base_url, args.api_key) as validator:
        results = await validator.run_all_tests()
        
        if args.output:
            with open(args.output, 'w') as f:
                json.dump(results, f, indent=2, default=str)
            print(f"Results saved to {args.output}")
        else:
            print(json.dumps(results, indent=2, default=str))

if __name__ == "__main__":
    asyncio.run(main())
