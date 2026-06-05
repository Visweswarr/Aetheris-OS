//! HAL Traits for Device Backend Abstraction
//!
//! This module defines the core traits that all device providers must implement
//! to ensure consistent behavior across deterministic and hardware backends.

use std::collections::HashMap;
use crate::error::DeviceError;
use super::{DeviceInfo, DeviceType, ProviderId, HalResult};

/// Base trait for all device providers
pub trait Provider: Send + Sync {
    /// Get the provider identifier
    fn get_provider_id(&self) -> ProviderId;

    /// Get provider capabilities
    fn get_capabilities(&self) -> Vec<String>;

    /// Check if the provider is available on this system
    fn is_available(&self) -> bool;

    /// Discover available devices
    async fn discover_devices(&self) -> HalResult<Vec<DeviceInfo>>;

    /// Get provider-specific metadata
    fn get_metadata(&self) -> HashMap<String, String>;
}

/// Camera backend trait
pub trait CameraBackend: Send + Sync {
    /// Open a camera device
    fn open(&self, device_path: &str, config: &CameraConfig) -> HalResult<Box<dyn CameraDevice>>;

    /// Get available camera devices
    async fn list_devices(&self) -> HalResult<Vec<DeviceInfo>>;
}

/// Camera device trait
pub trait CameraDevice: Send + Sync {
    /// Start capture
    async fn start_capture(&mut self) -> HalResult<()>;

    /// Stop capture
    async fn stop_capture(&mut self) -> HalResult<()>;

    /// Read a frame
    async fn read_frame(&mut self) -> HalResult<CameraFrame>;

    /// Get device information
    fn get_device_info(&self) -> &DeviceInfo;

    /// Check if capture is active
    fn is_capturing(&self) -> bool;
}

/// Camera configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CameraConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub format: CameraFormat,
    pub buffer_count: u32,
}

/// Camera frame format
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CameraFormat {
    YUV420,
    RGB24,
    MJPEG,
    H264,
}

/// Camera frame data
#[derive(Debug, Clone)]
pub struct CameraFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: CameraFormat,
    pub timestamp: u64,
    pub sequence: u64,
}

/// Microphone backend trait
pub trait MicrophoneBackend: Send + Sync {
    /// Open a microphone device
    fn open(&self, device_path: &str, config: &MicrophoneConfig) -> HalResult<Box<dyn MicrophoneDevice>>;

    /// Get available microphone devices
    async fn list_devices(&self) -> HalResult<Vec<DeviceInfo>>;
}

/// Microphone device trait
pub trait MicrophoneDevice: Send + Sync {
    /// Start capture
    async fn start_capture(&mut self) -> HalResult<()>;

    /// Stop capture
    async fn stop_capture(&mut self) -> HalResult<()>;

    /// Read audio samples
    async fn read_samples(&mut self) -> HalResult<AudioSamples>;

    /// Get device information
    fn get_device_info(&self) -> &DeviceInfo;

    /// Check if capture is active
    fn is_capturing(&self) -> bool;
}

/// Microphone configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MicrophoneConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub format: AudioFormat,
    pub buffer_size: u32,
}

/// Audio format
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AudioFormat {
    S16LE,
    F32LE,
    S24LE,
}

/// Audio samples data
#[derive(Debug, Clone)]
pub struct AudioSamples {
    pub data: Vec<u8>,
    pub sample_rate: u32,
    pub channels: u16,
    pub format: AudioFormat,
    pub timestamp: u64,
    pub frame_count: u32,
}

/// GPIO backend trait
pub trait GpioBackend: Send + Sync {
    /// Open a GPIO chip
    fn open_chip(&self, chip_path: &str) -> HalResult<Box<dyn GpioChip>>;

    /// Get available GPIO chips
    async fn list_chips(&self) -> HalResult<Vec<DeviceInfo>>;
}

/// GPIO chip trait
pub trait GpioChip: Send + Sync {
    /// Configure a GPIO line
    fn configure_line(&self, line: u32, config: &GpioLineConfig) -> HalResult<Box<dyn GpioLine>>;

    /// Get chip information
    fn get_chip_info(&self) -> &DeviceInfo;

    /// Get number of lines
    fn get_line_count(&self) -> u32;
}

/// GPIO line trait
pub trait GpioLine: Send + Sync {
    /// Read line value
    fn read(&self) -> HalResult<bool>;

    /// Write line value
    fn write(&self, value: bool) -> HalResult<()>;

    /// Toggle line value
    fn toggle(&self) -> HalResult<bool>;

    /// Get line configuration
    fn get_config(&self) -> &GpioLineConfig;

    /// Get line number
    fn get_line_number(&self) -> u32;
}

/// GPIO line configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GpioLineConfig {
    pub line: u32,
    pub mode: GpioMode,
    pub pull: GpioPull,
    pub initial_value: Option<bool>,
    pub debounce_ms: Option<u32>,
}

/// GPIO mode
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GpioMode {
    Input,
    Output,
    InputPullUp,
    InputPullDown,
    OutputOpenDrain,
    OutputPushPull,
}

/// GPIO pull configuration
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GpioPull {
    None,
    Up,
    Down,
}

/// ADC backend trait
pub trait AdcBackend: Send + Sync {
    /// Open an ADC device
    fn open_device(&self, device_path: &str) -> HalResult<Box<dyn AdcDevice>>;

    /// Get available ADC devices
    async fn list_devices(&self) -> HalResult<Vec<DeviceInfo>>;
}

/// ADC device trait
pub trait AdcDevice: Send + Sync {
    /// Configure a channel
    fn configure_channel(&self, channel: u32, config: &AdcChannelConfig) -> HalResult<Box<dyn AdcChannel>>;

    /// Get device information
    fn get_device_info(&self) -> &DeviceInfo;

    /// Get number of channels
    fn get_channel_count(&self) -> u32;
}

/// ADC channel trait
pub trait AdcChannel: Send + Sync {
    /// Read channel value
    fn read(&self) -> HalResult<AdcSample>;

    /// Start continuous sampling
    async fn start_sampling(&mut self, rate_hz: u32) -> HalResult<()>;

    /// Stop continuous sampling
    async fn stop_sampling(&mut self) -> HalResult<()>;

    /// Get channel configuration
    fn get_config(&self) -> &AdcChannelConfig;

    /// Get channel number
    fn get_channel_number(&self) -> u32;
}

/// ADC channel configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdcChannelConfig {
    pub channel: u32,
    pub reference_voltage: f32,
    pub resolution_bits: u8,
    pub sample_rate_hz: u32,
    pub calibration_offset: f32,
    pub calibration_scale: f32,
    pub enabled: bool,
}

/// ADC sample
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdcSample {
    pub channel: u32,
    pub raw_value: u32,
    pub voltage: f32,
    pub timestamp: u64,
    pub sequence_number: u64,
}

/// Actuator backend trait
pub trait ActuatorBackend: Send + Sync {
    /// Open an actuator device
    fn open_device(&self, device_path: &str, config: &ActuatorConfig) -> HalResult<Box<dyn ActuatorDevice>>;

    /// Get available actuator devices
    async fn list_devices(&self) -> HalResult<Vec<DeviceInfo>>;
}

/// Actuator device trait
pub trait ActuatorDevice: Send + Sync {
    /// Set actuator value
    fn set_value(&self, value: f32) -> HalResult<()>;

    /// Get current value
    fn get_value(&self) -> HalResult<f32>;

    /// Start pattern execution
    async fn start_pattern(&mut self, pattern: &ActuatorPattern) -> HalResult<()>;

    /// Stop pattern execution
    async fn stop_pattern(&mut self) -> HalResult<()>;

    /// Emergency stop
    fn emergency_stop(&self) -> HalResult<()>;

    /// Get device information
    fn get_device_info(&self) -> &DeviceInfo;

    /// Get actuator configuration
    fn get_config(&self) -> &ActuatorConfig;
}

/// Actuator configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActuatorConfig {
    pub actuator_id: String,
    pub actuator_type: ActuatorType,
    pub device_path: String,
    pub min_value: f32,
    pub max_value: f32,
    pub initial_value: f32,
    pub frequency_hz: Option<u32>,
    pub enabled: bool,
}

/// Actuator type
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ActuatorType {
    PWM,
    Relay,
    Motor,
    LED,
    Servo,
    Stepper,
}

/// Actuator pattern
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActuatorPattern {
    pub pattern_id: String,
    pub pattern_type: PatternType,
    pub duration_ms: u32,
    pub steps: Vec<PatternStep>,
    pub repeat: bool,
}

/// Pattern type
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PatternType {
    Linear,
    Sine,
    Square,
    Triangle,
    Custom,
}

/// Pattern step
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatternStep {
    pub value: f32,
    pub duration_ms: u32,
    pub transition_type: TransitionType,
}

/// Transition type
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TransitionType {
    Immediate,
    Linear,
    Smooth,
}

/// Audit event for device operations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceAuditEvent {
    pub event_type: String,
    pub device_id: String,
    pub device_path: String,
    pub operation: String,
    pub session_id: String,
    pub capability_scope: String,
    pub policy_hash: Option<String>,
    pub timestamp: u64,
    pub result: bool,
    pub error_message: Option<String>,
}

/// Audit logger trait
pub trait AuditLogger: Send + Sync {
    /// Log a device operation
    fn log_operation(&self, event: &DeviceAuditEvent);
}

/// Default audit logger implementation
pub struct DefaultAuditLogger;

impl AuditLogger for DefaultAuditLogger {
    fn log_operation(&self, event: &DeviceAuditEvent) {
        tracing::info!(
            "Device audit: {} {} {} {} {} {}",
            event.event_type,
            event.device_id,
            event.operation,
            event.session_id,
            event.capability_scope,
            if event.result { "SUCCESS" } else { "FAILED" }
        );
    }
}
