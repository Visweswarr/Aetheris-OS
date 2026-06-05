//! Error types for the device service

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    
    #[error("Capture not found: {0}")]
    CaptureNotFound(String),
    
    #[error("Invalid capability: {0}")]
    InvalidCapability(String),
    
    #[error("DAO policy denied: {0}")]
    DaoPolicyDenied(String),
    
    #[error("Device initialization failed: {0}")]
    DeviceInitFailed(String),
    
    #[error("Capture start failed: {0}")]
    CaptureStartFailed(String),
    
    #[error("Capture stop failed: {0}")]
    CaptureStopFailed(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("NGFS error: {0}")]
    NgfsError(String),
    
    #[error("Encoding error: {0}")]
    EncodingError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("CBOR error: {0}")]
    CborError(#[from] cbor2::ser::Error),
    
    #[error("UUID error: {0}")]
    UuidError(#[from] uuid::Error),
    
    #[error("Chrono error: {0}")]
    ChronoError(#[from] chrono::ParseError),
    
    // BLE-specific errors
    #[error("BLE scan failed: {0}")]
    BleScanFailed(String),
    
    #[error("BLE connection failed: {0}")]
    BleConnectionFailed(String),
    
    #[error("BLE GATT operation failed: {0}")]
    BleGattFailed(String),
    
    #[error("BLE notification subscription failed: {0}")]
    BleNotificationFailed(String),
    
    // Sensor-specific errors
    #[error("Sensor registration failed: {0}")]
    SensorRegistrationFailed(String),
    
    #[error("Sensor sampling failed: {0}")]
    SensorSamplingFailed(String),
    
    #[error("Sensor preview failed: {0}")]
    SensorPreviewFailed(String),
    
    #[error("Sensor snapshot failed: {0}")]
    SensorSnapshotFailed(String),
    
    #[error("Invalid sample rate: {0}")]
    InvalidSampleRate(String),
    
    // GPIO-specific errors
    #[error("Pin not configured: {0}")]
    PinNotConfigured(u32),
    
    #[error("Invalid pin mode for pin {0}: {1}")]
    InvalidPinMode(u32, String),
    
    #[error("GPIO operation failed: {0}")]
    GpioOperationFailed(String),
    
    // ADC-specific errors
    #[error("Channel not configured: {0}")]
    ChannelNotConfigured(u32),
    
    #[error("Channel disabled: {0}")]
    ChannelDisabled(u32),
    
    #[error("ADC sampling failed: {0}")]
    AdcSamplingFailed(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    // Actuator-specific errors
    #[error("Actuator not configured: {0}")]
    ActuatorNotConfigured(String),
    
    #[error("Actuator disabled: {0}")]
    ActuatorDisabled(String),
    
    #[error("Value out of range: {0} (min: {1}, max: {2})")]
    ValueOutOfRange(f32, f32, f32),
    
    #[error("Safety violation: {0}")]
    SafetyViolation(String),
    
    #[error("Pattern not found: {0}")]
    PatternNotFound(String),
    
    #[error("Actuator operation failed: {0}")]
    ActuatorOperationFailed(String),
    
    // Session errors
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    
    // Serialization errors
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

impl From<DeviceError> for String {
    fn from(err: DeviceError) -> Self {
        err.to_string()
    }
}
