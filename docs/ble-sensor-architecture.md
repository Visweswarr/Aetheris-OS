# BLE and Sensor Services Architecture

## System Architecture Overview

The BLE and Sensor services in Aetheris OS follow a layered architecture that provides separation of concerns, security, and scalability.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    Aetheris OS BLE & Sensor Services            │
├─────────────────────────────────────────────────────────────────┤
│  Web Applications  │  CLI Tools  │  IoT Devices  │  System Apps │
│  (TypeScript)      │  (Go)       │  (C/C++)      │  (Rust)      │
├─────────────────────────────────────────────────────────────────┤
│                    API Layer (HTTP/gRPC)                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   BLE Bridge    │  │  Sensor Bridge  │  │   C FFI Layer   │ │
│  │  (TypeScript)   │  │  (TypeScript)   │  │      (C)        │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                    RPC Handler Layer                            │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   BLE RPC       │  │  Sensor RPC     │  │   Policy RPC    │ │
│  │   Handlers      │  │   Handlers      │  │   Handlers      │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                    Policy & Security Layer                      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   CapToken      │  │   DAO Policy    │  │   Access        │ │
│  │   Validation    │  │   Enforcement   │  │   Control       │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                    Service Runtime Layer                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   BLE Runtime   │  │  Sensor Runtime │  │   Device        │ │
│  │   (Rust)        │  │   (Rust)        │  │   Manager       │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                    Hardware Abstraction Layer                   │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   Bluetooth     │  │   Sensor        │  │   System        │ │
│  │   Stack         │  │   Drivers       │  │   Resources     │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. API Layer

The API layer provides multiple interfaces for different types of clients:

#### TypeScript Bridges
- **BLE Bridge**: Web application integration for BLE operations
- **Sensor Bridge**: Web application integration for sensor operations
- **Features**: Event-driven architecture, automatic reconnection, error handling

#### C FFI Layer
- **BLE FFI**: C language bindings for system-level integration
- **Sensor FFI**: C language bindings for embedded systems
- **Features**: Low-level control, minimal overhead, direct hardware access

#### Go CLI Tools
- **BLE Commands**: Command-line interface for BLE management
- **Sensor Commands**: Command-line interface for sensor management
- **Features**: Human-readable output, batch operations, scripting support

### 2. RPC Handler Layer

The RPC handler layer processes HTTP/gRPC requests and routes them to appropriate services:

#### BLE RPC Handlers
```rust
// Example BLE RPC handler structure
pub struct BLERPCHandler {
    service: Arc<BLEService>,
    policy_enforcer: Arc<PolicyEnforcer>,
    metrics: Arc<MetricsCollector>,
}

impl BLERPCHandler {
    pub async fn handle_scan_start(&self, request: ScanRequest) -> Result<ScanResponse> {
        // Validate CapToken
        self.policy_enforcer.check_scope(&request.caps, "device:ble.scan")?;
        
        // Check DAO policies
        self.policy_enforcer.check_dao_policy("ble_scan", &request.context)?;
        
        // Execute operation
        let result = self.service.start_scan(request).await?;
        
        // Record metrics
        self.metrics.record_scan_start();
        
        Ok(result)
    }
}
```

#### Sensor RPC Handlers
```rust
// Example Sensor RPC handler structure
pub struct SensorRPCHandler {
    service: Arc<SensorService>,
    policy_enforcer: Arc<PolicyEnforcer>,
    metrics: Arc<MetricsCollector>,
}

impl SensorRPCHandler {
    pub async fn handle_sampling_start(&self, request: SamplingRequest) -> Result<SamplingResponse> {
        // Validate CapToken
        self.policy_enforcer.check_scope(&request.caps, "device:sensor.sample")?;
        
        // Check DAO policies
        self.policy_enforcer.check_dao_policy("sensor_sampling", &request.context)?;
        
        // Execute operation
        let result = self.service.start_sampling(request).await?;
        
        // Record metrics
        self.metrics.record_sampling_start();
        
        Ok(result)
    }
}
```

### 3. Policy & Security Layer

The policy and security layer ensures that all operations are properly authorized and comply with system policies:

#### CapToken Validation
```rust
// CapToken scope validation
pub struct CapTokenValidator {
    token_store: Arc<TokenStore>,
    scope_cache: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl CapTokenValidator {
    pub fn validate_scope(&self, token: &str, required_scope: &str) -> Result<()> {
        let scopes = self.get_token_scopes(token)?;
        
        if !scopes.contains(&required_scope.to_string()) {
            return Err(ValidationError::InsufficientScope {
                required: required_scope.to_string(),
                available: scopes,
            });
        }
        
        Ok(())
    }
}
```

#### DAO Policy Enforcement
```rust
// DAO policy enforcement
pub struct DAOPolicyEnforcer {
    dao_client: Arc<DAOClient>,
    policy_cache: Arc<Mutex<HashMap<String, PolicyResult>>>,
}

impl DAOPolicyEnforcer {
    pub async fn check_policy(&self, policy_name: &str, context: &PolicyContext) -> Result<PolicyResult> {
        // Check cache first
        if let Some(cached) = self.policy_cache.lock().unwrap().get(policy_name) {
            return Ok(cached.clone());
        }
        
        // Query DAO
        let result = self.dao_client.check_policy(policy_name, context).await?;
        
        // Cache result
        self.policy_cache.lock().unwrap().insert(policy_name.to_string(), result.clone());
        
        Ok(result)
    }
}
```

### 4. Service Runtime Layer

The service runtime layer contains the core business logic for BLE and sensor operations:

#### BLE Runtime
```rust
// BLE runtime core structure
pub struct BLERuntime {
    adapter: Arc<BluetoothAdapter>,
    connection_manager: Arc<ConnectionManager>,
    scan_manager: Arc<ScanManager>,
    gatt_manager: Arc<GATTManager>,
    event_loop: Arc<EventLoop>,
}

impl BLERuntime {
    pub async fn start_scan(&self, params: ScanParams) -> Result<ScanHandle> {
        let scan_id = self.generate_scan_id();
        
        // Configure scan parameters
        let scan_config = ScanConfig {
            duration: params.duration,
            allow_duplicates: params.allow_duplicates,
            service_uuids: params.service_uuids,
            manufacturer_data: params.manufacturer_data,
            name_prefix: params.name_prefix,
            rssi_threshold: params.rssi_threshold,
        };
        
        // Start scanning
        let handle = self.scan_manager.start_scan(scan_id, scan_config).await?;
        
        Ok(handle)
    }
    
    pub async fn connect_device(&self, address: &str, params: ConnectionParams) -> Result<ConnectionHandle> {
        let connection_id = self.generate_connection_id();
        
        // Create connection
        let connection = BLEConnection::new(
            connection_id,
            address.to_string(),
            params.timeout,
            params.auto_reconnect,
        );
        
        // Establish connection
        let handle = self.connection_manager.connect(connection).await?;
        
        // Discover services if requested
        if params.service_discovery {
            self.gatt_manager.discover_services(handle).await?;
        }
        
        Ok(handle)
    }
}
```

#### Sensor Runtime
```rust
// Sensor runtime core structure
pub struct SensorRuntime {
    registry: Arc<SensorRegistry>,
    sampling_manager: Arc<SamplingManager>,
    preview_manager: Arc<PreviewManager>,
    snapshot_manager: Arc<SnapshotManager>,
    data_generator: Arc<DataGenerator>,
}

impl SensorRuntime {
    pub async fn register_sensor(&self, desc: SensorDesc) -> Result<SensorId> {
        let sensor_id = self.generate_sensor_id();
        
        // Validate sensor description
        self.validate_sensor_desc(&desc)?;
        
        // Create sensor info
        let sensor_info = SensorInfo {
            sensor_id: sensor_id.clone(),
            name: desc.name,
            kind: desc.kind,
            location: desc.location,
            unit: desc.unit,
            range_min: desc.range_min,
            range_max: desc.range_max,
            resolution: desc.resolution,
            sample_rates: desc.sample_rates,
            registered_at: Utc::now(),
            active_sampling: false,
        };
        
        // Register sensor
        self.registry.register_sensor(sensor_info).await?;
        
        Ok(sensor_id)
    }
    
    pub async fn start_sampling(&self, params: SamplingParams) -> Result<SamplingHandle> {
        let handle = self.generate_sampling_handle();
        
        // Validate sensor exists
        let sensor = self.registry.get_sensor(&params.sensor_id).await?;
        
        // Validate sampling rate
        if !sensor.sample_rates.contains(&params.hz) {
            return Err(SensorError::UnsupportedSampleRate(params.hz));
        }
        
        // Create sampling session
        let session = SamplingSession {
            handle: handle.clone(),
            sensor_id: params.sensor_id,
            session_id: params.session_id,
            hz: params.hz,
            seed: params.seed,
            start_time: Utc::now(),
            sample_count: 0,
            deterministic: params.seed != 0,
        };
        
        // Start sampling
        self.sampling_manager.start_session(session).await?;
        
        Ok(handle)
    }
}
```

### 5. Hardware Abstraction Layer

The hardware abstraction layer provides a unified interface to different hardware components:

#### Bluetooth Stack Integration
```rust
// Bluetooth stack abstraction
pub trait BluetoothAdapter {
    async fn start_scan(&self, config: ScanConfig) -> Result<ScanHandle>;
    async fn stop_scan(&self, handle: ScanHandle) -> Result<ScanResult>;
    async fn connect(&self, address: &str) -> Result<ConnectionHandle>;
    async fn disconnect(&self, handle: ConnectionHandle) -> Result<()>;
    async fn discover_services(&self, handle: ConnectionHandle) -> Result<Vec<Service>>;
    async fn read_characteristic(&self, handle: ConnectionHandle, service_uuid: &str, char_uuid: &str) -> Result<Vec<u8>>;
    async fn write_characteristic(&self, handle: ConnectionHandle, service_uuid: &str, char_uuid: &str, value: &[u8]) -> Result<()>;
    async fn subscribe_characteristic(&self, handle: ConnectionHandle, service_uuid: &str, char_uuid: &str) -> Result<()>;
}

// Platform-specific implementations
pub struct LinuxBluetoothAdapter {
    bluez_client: Arc<BlueZClient>,
}

pub struct WindowsBluetoothAdapter {
    winrt_client: Arc<WinRTClient>,
}

pub struct MacOSBluetoothAdapter {
    core_bluetooth: Arc<CoreBluetoothClient>,
}
```

#### Sensor Driver Integration
```rust
// Sensor driver abstraction
pub trait SensorDriver {
    async fn read_value(&self) -> Result<SensorValue>;
    async fn configure(&self, config: SensorConfig) -> Result<()>;
    async fn start_sampling(&self, rate: u32) -> Result<()>;
    async fn stop_sampling(&self) -> Result<()>;
    async fn get_capabilities(&self) -> Result<SensorCapabilities>;
}

// Platform-specific implementations
pub struct LinuxSensorDriver {
    iio_client: Arc<I2CClient>,
}

pub struct WindowsSensorDriver {
    winrt_sensors: Arc<WinRTSensors>,
}

pub struct MacOSSensorDriver {
    core_motion: Arc<CoreMotionClient>,
}
```

## Data Flow

### BLE Operation Flow

```
1. Client Request (TypeScript/Go/C)
   ↓
2. RPC Handler (HTTP/gRPC)
   ↓
3. CapToken Validation
   ↓
4. DAO Policy Check
   ↓
5. BLE Runtime
   ↓
6. Bluetooth Stack
   ↓
7. Hardware (Bluetooth Adapter)
```

### Sensor Operation Flow

```
1. Client Request (TypeScript/Go/C)
   ↓
2. RPC Handler (HTTP/gRPC)
   ↓
3. CapToken Validation
   ↓
4. DAO Policy Check
   ↓
5. Sensor Runtime
   ↓
6. Sensor Driver
   ↓
7. Hardware (Sensor)
```

## Security Architecture

### Multi-Layer Security

1. **Transport Security**: TLS/SSL encryption for all API communications
2. **Authentication**: CapToken-based authentication for all operations
3. **Authorization**: Scope-based access control with DAO policy integration
4. **Audit Logging**: Comprehensive logging of all operations
5. **Rate Limiting**: API rate limiting to prevent abuse
6. **Input Validation**: Strict input validation and sanitization

### Security Flow

```
1. Client Authentication (CapToken)
   ↓
2. Scope Validation (Required permissions)
   ↓
3. DAO Policy Check (Governance rules)
   ↓
4. Operation Execution (Authorized operation)
   ↓
5. Audit Logging (Operation recorded)
```

## Performance Architecture

### Optimization Strategies

1. **Connection Pooling**: Reuse BLE connections when possible
2. **Async Operations**: All operations are asynchronous
3. **Caching**: Policy and token caching for performance
4. **Batch Operations**: Group multiple operations
5. **Resource Management**: Efficient resource allocation and cleanup

### Performance Monitoring

1. **Metrics Collection**: Built-in metrics for all operations
2. **Performance Counters**: Response time, throughput, error rates
3. **Resource Usage**: Memory, CPU, network usage tracking
4. **Health Checks**: Service health monitoring

## Scalability Architecture

### Horizontal Scaling

1. **Stateless Services**: Services are stateless for easy scaling
2. **Load Balancing**: Multiple service instances can be load balanced
3. **Database Sharding**: Sensor data can be sharded across multiple databases
4. **Caching**: Distributed caching for improved performance

### Vertical Scaling

1. **Resource Optimization**: Efficient resource usage
2. **Memory Management**: Optimized memory allocation
3. **CPU Optimization**: Multi-threaded processing
4. **I/O Optimization**: Async I/O operations

## Conclusion

The BLE and Sensor services architecture provides a robust, scalable, and secure foundation for IoT and embedded applications in Aetheris OS. The layered approach ensures separation of concerns, while the comprehensive security and policy integration provides enterprise-grade access control and governance.
