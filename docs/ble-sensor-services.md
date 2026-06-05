# BLE and Sensor Services Documentation

## Overview

This document provides comprehensive documentation for the BLE (Bluetooth Low Energy) and Sensor services in Aetheris OS. These services enable device communication, sensor data collection, and real-time monitoring capabilities.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [BLE Service](#ble-service)
3. [Sensor Service](#sensor-service)
4. [API Reference](#api-reference)
5. [Integration Examples](#integration-examples)
6. [Security and Policies](#security-and-policies)
7. [Performance Considerations](#performance-considerations)
8. [Troubleshooting](#troubleshooting)

## Architecture Overview

The BLE and Sensor services are built on a modular architecture with the following components:

- **Rust Runtime Services**: Core service implementations
- **RPC Handlers**: HTTP/gRPC API endpoints
- **Policy Enforcement**: CapToken and DAO-based access control
- **C FFI**: C language bindings for system integration
- **Go CLI**: Command-line interface for management
- **TypeScript Bridge**: Web application integration
- **Python Validators**: Testing and validation tools

## BLE Service

### Features

- **Device Scanning**: Discover nearby BLE devices
- **Connection Management**: Establish and maintain BLE connections
- **GATT Operations**: Read/write characteristics and subscribe to notifications
- **Service Discovery**: Automatically discover available services
- **Multi-device Support**: Manage multiple concurrent connections

### Core Components

#### 1. BLE Runtime (`rust/services/ble/`)

The BLE runtime provides the core functionality for Bluetooth Low Energy operations:

```rust
// Main BLE service structure
pub struct BLEService {
    runtime: Arc<BLERuntime>,
    connections: Arc<Mutex<HashMap<String, BLEConnection>>>,
    active_scans: Arc<Mutex<HashSet<String>>>,
    policy_enforcer: Arc<PolicyEnforcer>,
}
```

#### 2. RPC Handlers (`rust/services/ble/rpc.rs`)

HTTP/gRPC endpoints for BLE operations:

- `POST /ble/scan/start` - Start device scanning
- `POST /ble/scan/{scan_id}/stop` - Stop scanning
- `GET /ble/scan/{scan_id}/results` - Get scan results
- `POST /ble/connect` - Connect to device
- `POST /ble/connect/{handle}/disconnect` - Disconnect from device
- `POST /ble/connect/{handle}/services/discover` - Discover GATT services
- `POST /ble/connect/{handle}/characteristic/read` - Read characteristic
- `POST /ble/connect/{handle}/characteristic/write` - Write characteristic
- `POST /ble/connect/{handle}/characteristic/subscribe` - Subscribe to notifications

#### 3. Policy Enforcement

BLE operations are protected by CapToken scopes:

```rust
// Required scopes for BLE operations
const BLE_SCAN_SCOPE: &str = "device:ble.scan";
const BLE_CONNECT_SCOPE: &str = "device:ble.connect";
const BLE_GATT_SCOPE: &str = "device:ble.gatt";
```

### Usage Examples

#### TypeScript/Web Application

```typescript
import { BLEBridge } from './ble_bridge';

const ble = new BLEBridge({
  endpoint: 'http://localhost:8080',
  enablePolicyEnforcement: true
});

// Start scanning for devices
const scanId = await ble.startScan({
  duration: 10000,
  rssiThreshold: -80
});

// Get scan results
const devices = await ble.getScanResults(scanId);

// Connect to a device
const connectionHandle = await ble.connectDevice(devices[0].address);

// Discover services
const services = await ble.discoverServices(connectionHandle);

// Read a characteristic
const value = await ble.readCharacteristic(
  connectionHandle,
  serviceUuid,
  characteristicUuid
);
```

#### Go CLI

```bash
# Start BLE scanning
devctl ble scan start --duration 10s --rssi-threshold -80

# List scan results
devctl ble scan results <scan_id>

# Connect to device
devctl ble connect <device_address> --timeout 5s

# Discover services
devctl ble services discover <connection_handle>

# Read characteristic
devctl ble characteristic read <connection_handle> <service_uuid> <characteristic_uuid>
```

#### C FFI

```c
#include "ble_ffi.h"

// Initialize BLE service
ble_service_t* ble = ble_service_new("http://localhost:8080");

// Start scanning
ble_scan_handle_t scan_handle = ble_start_scan(ble, 10000, -80);

// Get scan results
ble_device_list_t* devices = ble_get_scan_results(ble, scan_handle);

// Connect to device
ble_connection_handle_t conn = ble_connect_device(ble, devices->devices[0].address);

// Read characteristic
ble_characteristic_value_t value = ble_read_characteristic(ble, conn, service_uuid, char_uuid);
```

## Sensor Service

### Features

- **Sensor Registration**: Register new sensors with capabilities
- **Data Sampling**: Collect sensor data at configurable rates
- **Preview Streams**: Real-time sensor data preview
- **Snapshot Creation**: Deterministic sensor data snapshots
- **Multi-sensor Support**: Manage multiple sensors concurrently

### Core Components

#### 1. Sensor Runtime (`rust/services/sensor/`)

The sensor runtime provides the core functionality for sensor operations:

```rust
// Main sensor service structure
pub struct SensorService {
    runtime: Arc<SensorRuntime>,
    registry: Arc<Mutex<SensorRegistry>>,
    active_sessions: Arc<Mutex<HashMap<String, SamplingSession>>>,
    active_previews: Arc<Mutex<HashMap<String, PreviewStream>>>,
    policy_enforcer: Arc<PolicyEnforcer>,
}
```

#### 2. RPC Handlers (`rust/services/sensor/rpc.rs`)

HTTP/gRPC endpoints for sensor operations:

- `POST /sensor/register` - Register a new sensor
- `GET /sensor/list` - List registered sensors
- `POST /sensor/sample/start` - Start sensor sampling
- `POST /sensor/sample/{handle}/stop` - Stop sampling
- `GET /sensor/sample/{handle}/samples` - Get collected samples
- `POST /sensor/preview/start` - Start preview stream
- `POST /sensor/preview/{handle}/stop` - Stop preview
- `GET /sensor/preview/{handle}/samples` - Get preview samples
- `POST /sensor/snapshot/create` - Create sensor snapshot

#### 3. Policy Enforcement

Sensor operations are protected by CapToken scopes:

```rust
// Required scopes for sensor operations
const SENSOR_REGISTER_SCOPE: &str = "device:sensor.register";
const SENSOR_SAMPLE_SCOPE: &str = "device:sensor.sample";
const SENSOR_PREVIEW_SCOPE: &str = "device:sensor.preview";
const SENSOR_SNAPSHOT_SCOPE: &str = "device:sensor.snapshot";
```

### Usage Examples

#### TypeScript/Web Application

```typescript
import { SensorBridge } from './sensor_bridge';

const sensor = new SensorBridge({
  endpoint: 'http://localhost:8080',
  enablePolicyEnforcement: true
});

// Register a new sensor
const sensorId = await sensor.registerSensor({
  name: 'Accelerometer',
  kind: 'accelerometer',
  location: 'IMU',
  unit: 'm/s²',
  rangeMin: -20.0,
  rangeMax: 20.0,
  resolution: 0.01,
  sampleRates: [10, 50, 100, 200]
});

// Start sampling
const handle = await sensor.startSampling(
  'session_123',
  'device:sensor.sample',
  sensorId,
  50, // 50 Hz
  12345 // deterministic seed
);

// Get samples
const samples = await sensor.getSamples(handle, 100);

// Start preview
const streamHandle = await sensor.startPreview(sensorId, 5);

// Create snapshot
const snapshotId = await sensor.createSnapshot(sensorId, 'snaps/accel.ngfs');
```

#### Go CLI

```bash
# Register a sensor
devctl sensor register --name "Accelerometer" --kind accelerometer --unit "m/s²" --range-min -20 --range-max 20

# List sensors
devctl sensor list

# Start sampling
devctl sensor start <sensor_id> --hz 50 --seed 12345

# Stop sampling
devctl sensor stop <handle>

# Start preview
devctl sensor preview <sensor_id> --fps 5

# Create snapshot
devctl sensor snapshot <sensor_id> --out snaps/accel.ngfs
```

#### C FFI

```c
#include "sensor_ffi.h"

// Initialize sensor service
sensor_service_t* sensor = sensor_service_new("http://localhost:8080");

// Register sensor
sensor_desc_t desc = {
    .name = "Accelerometer",
    .kind = SENSOR_ACCELEROMETER,
    .location = "IMU",
    .unit = "m/s²",
    .range_min = -20.0,
    .range_max = 20.0,
    .resolution = 0.01,
    .sample_rates = {10, 50, 100, 200}
};

sensor_id_t sensor_id = sensor_register(sensor, &desc);

// Start sampling
sensor_session_handle_t handle = sensor_start_sampling(sensor, "session_123", "device:sensor.sample", sensor_id, 50, 12345);

// Get samples
sensor_sample_list_t* samples = sensor_get_samples(sensor, handle, 100);
```

## API Reference

### BLE Service API

#### Scan Operations

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/ble/scan/start` | POST | Start BLE device scanning |
| `/ble/scan/{scan_id}/stop` | POST | Stop scanning and get results |
| `/ble/scan/{scan_id}/results` | GET | Get current scan results |

#### Connection Operations

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/ble/connect` | POST | Connect to BLE device |
| `/ble/connect/{handle}/disconnect` | POST | Disconnect from device |
| `/ble/connect/{handle}/services/discover` | POST | Discover GATT services |

#### GATT Operations

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/ble/connect/{handle}/characteristic/read` | POST | Read characteristic value |
| `/ble/connect/{handle}/characteristic/write` | POST | Write characteristic value |
| `/ble/connect/{handle}/characteristic/subscribe` | POST | Subscribe to notifications |
| `/ble/connect/{handle}/characteristic/unsubscribe` | POST | Unsubscribe from notifications |

### Sensor Service API

#### Registration Operations

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/sensor/register` | POST | Register a new sensor |
| `/sensor/list` | GET | List registered sensors |

#### Sampling Operations

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/sensor/sample/start` | POST | Start sensor sampling |
| `/sensor/sample/{handle}/stop` | POST | Stop sampling |
| `/sensor/sample/{handle}/samples` | GET | Get collected samples |

#### Preview Operations

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/sensor/preview/start` | POST | Start preview stream |
| `/sensor/preview/{handle}/stop` | POST | Stop preview |
| `/sensor/preview/{handle}/samples` | GET | Get preview samples |

#### Snapshot Operations

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/sensor/snapshot/create` | POST | Create sensor snapshot |

## Integration Examples

### Web Application Integration

```typescript
// Complete BLE integration example
class BLEDeviceManager {
  private ble: BLEBridge;
  private devices: Map<string, BLEDevice> = new Map();
  
  constructor() {
    this.ble = new BLEBridge({
      endpoint: 'http://localhost:8080',
      enablePolicyEnforcement: true,
      autoReconnect: true
    });
    
    this.ble.on('deviceConnected', this.onDeviceConnected.bind(this));
    this.ble.on('characteristicRead', this.onCharacteristicRead.bind(this));
  }
  
  async scanForDevices(): Promise<BLEDevice[]> {
    const scanId = await this.ble.startScan({
      duration: 10000,
      rssiThreshold: -80
    });
    
    const devices = await this.ble.getScanResults(scanId);
    devices.forEach(device => this.devices.set(device.address, device));
    
    return devices;
  }
  
  async connectToDevice(address: string): Promise<string> {
    const connectionHandle = await this.ble.connectDevice(address);
    const services = await this.ble.discoverServices(connectionHandle);
    
    // Subscribe to notifications for all characteristics that support it
    for (const service of services) {
      for (const characteristic of service.characteristics) {
        if (characteristic.properties.notify) {
          await this.ble.subscribeCharacteristic(
            connectionHandle,
            service.uuid,
            characteristic.uuid
          );
        }
      }
    }
    
    return connectionHandle;
  }
  
  private onDeviceConnected(data: { deviceAddress: string; connectionHandle: string }) {
    console.log(`Connected to device ${data.deviceAddress}`);
  }
  
  private onCharacteristicRead(data: { connectionHandle: string; serviceUuid: string; characteristicUuid: string; value: Uint8Array }) {
    console.log(`Characteristic read: ${data.serviceUuid}/${data.characteristicUuid} = ${data.value}`);
  }
}
```

### IoT Device Integration

```c
// Complete sensor integration example
#include "sensor_ffi.h"
#include "ble_ffi.h"

typedef struct {
    sensor_service_t* sensor;
    ble_service_t* ble;
    sensor_id_t sensor_id;
    ble_connection_handle_t ble_conn;
} iot_device_t;

iot_device_t* iot_device_new(const char* sensor_endpoint, const char* ble_endpoint) {
    iot_device_t* device = malloc(sizeof(iot_device_t));
    
    device->sensor = sensor_service_new(sensor_endpoint);
    device->ble = ble_service_new(ble_endpoint);
    
    return device;
}

int iot_device_setup_sensor(iot_device_t* device) {
    // Register accelerometer sensor
    sensor_desc_t desc = {
        .name = "Device Accelerometer",
        .kind = SENSOR_ACCELEROMETER,
        .location = "IMU",
        .unit = "m/s²",
        .range_min = -20.0,
        .range_max = 20.0,
        .resolution = 0.01,
        .sample_rates = {10, 50, 100}
    };
    
    device->sensor_id = sensor_register(device->sensor, &desc);
    if (device->sensor_id == NULL) {
        return -1;
    }
    
    return 0;
}

int iot_device_connect_ble(iot_device_t* device, const char* device_address) {
    device->ble_conn = ble_connect_device(device->ble, device_address);
    if (device->ble_conn == NULL) {
        return -1;
    }
    
    // Discover services
    ble_service_list_t* services = ble_discover_services(device->ble, device->ble_conn);
    if (services == NULL) {
        return -1;
    }
    
    return 0;
}

int iot_device_start_sampling(iot_device_t* device, int hz) {
    sensor_session_handle_t handle = sensor_start_sampling(
        device->sensor,
        "iot_session",
        "device:sensor.sample",
        device->sensor_id,
        hz,
        0 // Non-deterministic
    );
    
    return handle != NULL ? 0 : -1;
}
```

## Security and Policies

### CapToken Scopes

Both BLE and Sensor services use CapToken-based access control:

#### BLE Scopes
- `device:ble.scan` - Scan for BLE devices
- `device:ble.connect` - Connect to BLE devices
- `device:ble.gatt` - Perform GATT operations

#### Sensor Scopes
- `device:sensor.register` - Register new sensors
- `device:sensor.sample` - Start sensor sampling
- `device:sensor.preview` - Start preview streams
- `device:sensor.snapshot` - Create sensor snapshots

### DAO Policy Integration

Services integrate with the DAO system for governance:

```rust
// Policy check example
async fn check_ble_policy(&self, operation: &str, context: &BLEContext) -> Result<(), PolicyError> {
    let policy_result = self.dao_client.check_policy(
        "ble_operations",
        operation,
        context.user_did,
        context.device_info
    ).await?;
    
    if !policy_result.allowed {
        return Err(PolicyError::AccessDenied(policy_result.reason));
    }
    
    Ok(())
}
```

### Security Considerations

1. **Device Authentication**: BLE devices should be authenticated before connection
2. **Data Encryption**: Sensitive sensor data should be encrypted in transit
3. **Access Logging**: All operations are logged for audit purposes
4. **Rate Limiting**: API endpoints have rate limiting to prevent abuse
5. **Input Validation**: All inputs are validated and sanitized

## Performance Considerations

### BLE Service Performance

- **Connection Pooling**: Reuse connections when possible
- **Async Operations**: All operations are asynchronous
- **Batch Operations**: Group multiple GATT operations
- **Connection Timeout**: Configurable connection timeouts

### Sensor Service Performance

- **Sampling Rate Limits**: Configurable maximum sampling rates
- **Buffer Management**: Efficient sample buffering
- **Deterministic Mode**: Optional deterministic sampling for testing
- **Preview Optimization**: Optimized preview streams for real-time display

### Optimization Tips

1. **Use Connection Pooling**: Reuse BLE connections
2. **Batch GATT Operations**: Group multiple reads/writes
3. **Optimize Sampling Rates**: Use appropriate rates for your use case
4. **Enable Deterministic Mode**: For testing and debugging
5. **Monitor Performance**: Use built-in statistics endpoints

## Troubleshooting

### Common BLE Issues

#### Connection Failures
- Check device compatibility
- Verify Bluetooth is enabled
- Check signal strength (RSSI)
- Ensure device is in range

#### GATT Operation Failures
- Verify characteristic properties
- Check service discovery
- Ensure proper permissions
- Validate UUIDs

### Common Sensor Issues

#### Registration Failures
- Check sensor capabilities
- Verify sample rates
- Ensure proper permissions
- Validate sensor description

#### Sampling Issues
- Check sampling rate limits
- Verify sensor is active
- Check buffer capacity
- Ensure proper session management

### Debugging Tools

#### Python Validators
```bash
# Run BLE validation tests
python tooling/python/ble_validator.py --verbose

# Run sensor validation tests
python tooling/python/sensor_validator.py --verbose
```

#### Go CLI Debugging
```bash
# Check BLE service status
devctl ble stats

# Check sensor service status
devctl sensor stats

# List active connections
devctl ble connections

# List active sessions
devctl sensor sessions
```

#### Logging
Enable debug logging for detailed troubleshooting:

```rust
// Enable debug logging
env_logger::Builder::from_default_env()
    .filter_level(log::LevelFilter::Debug)
    .init();
```

### Performance Monitoring

#### BLE Statistics
```typescript
const stats = await ble.getStats();
console.log(`Connected devices: ${stats.connectedDevices}`);
console.log(`Average response time: ${stats.averageResponseTime}ms`);
```

#### Sensor Statistics
```typescript
const stats = await sensor.getStats();
console.log(`Active sensors: ${stats.activeSensors}`);
console.log(`Total samples: ${stats.totalSamples}`);
```

## Conclusion

The BLE and Sensor services provide comprehensive functionality for device communication and sensor data collection in Aetheris OS. With their modular architecture, security features, and multiple integration options, they enable a wide range of IoT and embedded applications.

For more information, see the individual service documentation and API references.
