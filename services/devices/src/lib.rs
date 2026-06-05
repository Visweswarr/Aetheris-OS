//! Device Runtime - Camera, Microphone, BLE & Sensor Services
//!
//! This module provides capability-gated device services with:
//! - Camera and microphone capture with deterministic recording
//! - BLE device management (scan, connect, GATT operations)
//! - Generic sensor framework with deterministic sampling
//! - NGFS snapshot integration for persistent storage
//! - Capability-based access control with DAO policy enforcement
//! - Polyglot bindings for C, Go, TypeScript, and Python
//! - Audit logging and compliance tracking

pub mod camera;
pub mod microphone;
pub mod ble;
pub mod sensor;
pub mod gpio;
pub mod adc;
pub mod actuators;
pub mod mux;
pub mod policy;
pub mod error;
pub mod rpc;
pub mod hal;

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::DeviceError;
use crate::camera::CameraManager;
use crate::microphone::MicrophoneManager;
use crate::gpio::GpioManager;
use crate::adc::AdcManager;
use crate::actuators::ActuatorManager;
use crate::mux::MediaMuxer;
use crate::policy::DevicePolicyManager;
use crate::hal::{HalManager, HalConfig, ProviderId};

/// Main device service that coordinates all device operations
pub struct DeviceService {
    camera_manager: Arc<CameraManager>,
    microphone_manager: Arc<MicrophoneManager>,
    gpio_manager: Arc<GpioManager>,
    adc_manager: Arc<AdcManager>,
    actuator_manager: Arc<ActuatorManager>,
    media_muxer: Arc<MediaMuxer>,
    policy_manager: Arc<DevicePolicyManager>,
    hal_manager: Arc<HalManager>,
    active_captures: Arc<RwLock<std::collections::HashMap<String, CaptureInfo>>>,
}

#[derive(Debug, Clone)]
pub struct CaptureInfo {
    pub capture_id: String,
    pub session_id: String,
    pub device_type: DeviceType,
    pub config: CaptureConfig,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub deterministic: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DeviceType {
    Camera,
    Microphone,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CaptureConfig {
    // Camera config
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<u32>,
    
    // Microphone config
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    
    // Common config
    pub deterministic: bool,
    pub max_chunk_duration_ms: u32,
    pub max_chunk_frames: u32,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            width: Some(640),
            height: Some(360),
            fps: Some(15),
            sample_rate: Some(44100),
            channels: Some(2),
            deterministic: false,
            max_chunk_duration_ms: 2000,
            max_chunk_frames: 30,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CaptureStats {
    pub capture_id: String,
    pub duration_ms: u64,
    pub bytes_written: u64,
    pub chunks_created: u32,
    pub last_chunk_timestamp: chrono::DateTime<chrono::Utc>,
    pub snapshot_id: Option<String>,
}

impl DeviceService {
    pub fn new() -> Result<Self, DeviceError> {
        let camera_manager = Arc::new(CameraManager::new()?);
        let microphone_manager = Arc::new(MicrophoneManager::new()?);
        let media_muxer = Arc::new(MediaMuxer::new()?);
        let policy_manager = Arc::new(DevicePolicyManager::new()?);
        let gpio_manager = Arc::new(GpioManager::new(policy_manager.clone())?);
        let adc_manager = Arc::new(AdcManager::new(policy_manager.clone())?);
        let actuator_manager = Arc::new(ActuatorManager::new(policy_manager.clone())?);
        
        // Initialize HAL manager with default configuration
        let hal_config = HalConfig::default();
        let hal_manager = Arc::new(HalManager::new(hal_config)?);
        
        Ok(Self {
            camera_manager,
            microphone_manager,
            gpio_manager,
            adc_manager,
            actuator_manager,
            media_muxer,
            policy_manager,
            hal_manager,
            active_captures: Arc::new(RwLock::new(std::collections::HashMap::new())),
        })
    }
    
    /// Start a camera capture session
    pub async fn start_camera_capture(
        &self,
        session_id: &str,
        caps: &str,
        config: CaptureConfig,
    ) -> Result<String, DeviceError> {
        // Validate capabilities
        self.policy_manager.validate_capability(session_id, caps, "device:camera.read").await?;
        
        // Check DAO policy if in shared room
        self.policy_manager.check_dao_policy(session_id, "camera_capture").await?;
        
        let capture_id = Uuid::new_v4().to_string();
        
        // Start camera capture
        self.camera_manager.start_capture(&capture_id, &config).await?;
        
        // Register capture info
        let capture_info = CaptureInfo {
            capture_id: capture_id.clone(),
            session_id: session_id.to_string(),
            device_type: DeviceType::Camera,
            config,
            start_time: chrono::Utc::now(),
            deterministic: config.deterministic,
        };
        
        self.active_captures.write().await.insert(capture_id.clone(), capture_info);
        
        // Emit audit event
        self.emit_audit_event("camera_capture_started", &capture_id, session_id).await?;
        
        Ok(capture_id)
    }
    
    /// Start a microphone capture session
    pub async fn start_microphone_capture(
        &self,
        session_id: &str,
        caps: &str,
        config: CaptureConfig,
    ) -> Result<String, DeviceError> {
        // Validate capabilities
        self.policy_manager.validate_capability(session_id, caps, "device:mic.read").await?;
        
        // Check DAO policy if in shared room
        self.policy_manager.check_dao_policy(session_id, "microphone_capture").await?;
        
        let capture_id = Uuid::new_v4().to_string();
        
        // Start microphone capture
        self.microphone_manager.start_capture(&capture_id, &config).await?;
        
        // Register capture info
        let capture_info = CaptureInfo {
            capture_id: capture_id.clone(),
            session_id: session_id.to_string(),
            device_type: DeviceType::Microphone,
            config,
            start_time: chrono::Utc::now(),
            deterministic: config.deterministic,
        };
        
        self.active_captures.write().await.insert(capture_id.clone(), capture_info);
        
        // Emit audit event
        self.emit_audit_event("microphone_capture_started", &capture_id, session_id).await?;
        
        Ok(capture_id)
    }
    
    /// Stop a capture session and return snapshot ID
    pub async fn stop_capture(&self, capture_id: &str) -> Result<CaptureStats, DeviceError> {
        let capture_info = {
            let captures = self.active_captures.read().await;
            captures.get(capture_id).cloned()
                .ok_or_else(|| DeviceError::CaptureNotFound(capture_id.to_string()))?
        };
        
        // Stop the appropriate device capture
        let snapshot_id = match capture_info.device_type {
            DeviceType::Camera => {
                self.camera_manager.stop_capture(capture_id).await?
            }
            DeviceType::Microphone => {
                self.microphone_manager.stop_capture(capture_id).await?
            }
        };
        
        // Calculate stats
        let duration = chrono::Utc::now() - capture_info.start_time;
        let stats = CaptureStats {
            capture_id: capture_id.to_string(),
            duration_ms: duration.num_milliseconds() as u64,
            bytes_written: 0, // TODO: Get from device manager
            chunks_created: 0, // TODO: Get from device manager
            last_chunk_timestamp: chrono::Utc::now(),
            snapshot_id: Some(snapshot_id),
        };
        
        // Remove from active captures
        self.active_captures.write().await.remove(capture_id);
        
        // Emit audit event
        self.emit_audit_event("capture_stopped", capture_id, &capture_info.session_id).await?;
        
        Ok(stats)
    }
    
    /// Get status of an active capture
    pub async fn get_capture_status(&self, capture_id: &str) -> Result<CaptureStatus, DeviceError> {
        let capture_info = {
            let captures = self.active_captures.read().await;
            captures.get(capture_id).cloned()
                .ok_or_else(|| DeviceError::CaptureNotFound(capture_id.to_string()))?
        };
        
        let running = true; // TODO: Check actual device status
        let bytes_written = 0; // TODO: Get from device manager
        let last_timestamp = chrono::Utc::now();
        
        Ok(CaptureStatus {
            capture_id: capture_id.to_string(),
            running,
            bytes_written,
            last_timestamp,
        })
    }
    
    /// Get preview frame (throttled, requires device:preview capability)
    pub async fn get_preview_frame(
        &self,
        capture_id: &str,
        session_id: &str,
        caps: &str,
    ) -> Result<Vec<u8>, DeviceError> {
        // Validate preview capability
        self.policy_manager.validate_capability(session_id, caps, "device:preview").await?;
        
        // Get preview frame from appropriate device
        let capture_info = {
            let captures = self.active_captures.read().await;
            captures.get(capture_id).cloned()
                .ok_or_else(|| DeviceError::CaptureNotFound(capture_id.to_string()))?
        };
        
        match capture_info.device_type {
            DeviceType::Camera => {
                self.camera_manager.get_preview_frame(capture_id).await
            }
            DeviceType::Microphone => {
                Err(DeviceError::InvalidOperation("Preview not available for microphone".to_string()))
            }
        }
    }
    
    /// Get GPIO manager
    pub fn gpio_manager(&self) -> Arc<GpioManager> {
        self.gpio_manager.clone()
    }

    /// Get ADC manager
    pub fn adc_manager(&self) -> Arc<AdcManager> {
        self.adc_manager.clone()
    }

    /// Get actuator manager
    pub fn actuator_manager(&self) -> Arc<ActuatorManager> {
        self.actuator_manager.clone()
    }

    /// Get policy manager
    pub fn policy_manager(&self) -> Arc<DevicePolicyManager> {
        self.policy_manager.clone()
    }

    /// Get HAL manager
    pub fn hal_manager(&self) -> Arc<HalManager> {
        self.hal_manager.clone()
    }

    /// Enable deterministic mode for all device managers
    pub async fn enable_deterministic_mode(&self) {
        self.gpio_manager.enable_deterministic_mode().await;
        self.adc_manager.enable_deterministic_mode().await;
        self.actuator_manager.enable_deterministic_mode().await;
    }

    /// Disable deterministic mode for all device managers
    pub async fn disable_deterministic_mode(&self) {
        self.gpio_manager.disable_deterministic_mode().await;
        self.adc_manager.disable_deterministic_mode().await;
        self.actuator_manager.disable_deterministic_mode().await;
    }

    /// Advance tick counter for all device managers
    pub async fn advance_tick(&self) {
        self.gpio_manager.advance_tick().await;
        self.adc_manager.advance_tick().await;
        self.actuator_manager.advance_tick().await;
    }

    /// Create comprehensive NGFS snapshot of all device states
    pub async fn create_device_snapshot(&self) -> Result<Vec<u8>, DeviceError> {
        let gpio_snapshot = self.gpio_manager.create_snapshot().await?;
        let adc_snapshot = self.adc_manager.create_snapshot().await?;
        let actuator_snapshot = self.actuator_manager.create_snapshot().await?;

        let device_snapshot = DeviceSnapshot {
            gpio_data: gpio_snapshot,
            adc_data: adc_snapshot,
            actuator_data: actuator_snapshot,
            timestamp: chrono::Utc::now(),
            version: 1,
        };

        // Serialize to CBOR for deterministic byte representation
        let mut buffer = Vec::new();
        ciborium::ser::into_writer(&device_snapshot, &mut buffer)
            .map_err(|e| DeviceError::SerializationError(e.to_string()))?;

        Ok(buffer)
    }

    /// Restore comprehensive device state from NGFS snapshot
    pub async fn restore_device_snapshot(&self, data: &[u8]) -> Result<(), DeviceError> {
        let device_snapshot: DeviceSnapshot = ciborium::de::from_reader(data)
            .map_err(|e| DeviceError::DeserializationError(e.to_string()))?;

        // Restore each component
        self.gpio_manager.restore_snapshot(&device_snapshot.gpio_data).await?;
        self.adc_manager.restore_snapshot(&device_snapshot.adc_data).await?;
        self.actuator_manager.restore_snapshot(&device_snapshot.actuator_data).await?;

        Ok(())
    }

    /// Discover available devices
    pub async fn discover_devices(&self) -> Result<Vec<crate::hal::DeviceInfo>, DeviceError> {
        self.hal_manager.discover_devices().await
    }

    /// Get all discovered devices
    pub fn get_devices(&self) -> Vec<crate::hal::DeviceInfo> {
        self.hal_manager.get_devices()
    }

    /// Get devices by type
    pub fn get_devices_by_type(&self, device_type: &crate::hal::DeviceType) -> Vec<crate::hal::DeviceInfo> {
        self.hal_manager.get_devices_by_type(device_type)
    }

    /// Get provider status
    pub async fn get_provider_status(&self) -> Vec<crate::hal::ProviderStatus> {
        self.hal_manager.get_provider_status().await
    }

    /// Set default provider
    pub fn set_default_provider(&self, provider_id: ProviderId) -> Result<(), DeviceError> {
        // This would need to be implemented with interior mutability
        // For now, return an error indicating it's not implemented
        Err(DeviceError::InvalidOperation("Provider switching not yet implemented".to_string()))
    }

    /// Get HAL configuration
    pub fn get_hal_config(&self) -> &HalConfig {
        self.hal_manager.get_config()
    }

    async fn emit_audit_event(
        &self,
        event_type: &str,
        capture_id: &str,
        session_id: &str,
    ) -> Result<(), DeviceError> {
        // TODO: Integrate with audit service
        tracing::info!(
            "Audit event: {} for capture {} in session {}",
            event_type,
            capture_id,
            session_id
        );
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CaptureStatus {
    pub capture_id: String,
    pub running: bool,
    pub bytes_written: u64,
    pub last_timestamp: chrono::DateTime<chrono::Utc>,
}

/// Comprehensive device snapshot for NGFS persistence
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceSnapshot {
    pub gpio_data: Vec<u8>,
    pub adc_data: Vec<u8>,
    pub actuator_data: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_device_service_creation() {
        let service = DeviceService::new();
        assert!(service.is_ok());
    }
    
    #[tokio::test]
    async fn test_capture_config_default() {
        let config = CaptureConfig::default();
        assert_eq!(config.width, Some(640));
        assert_eq!(config.height, Some(360));
        assert_eq!(config.fps, Some(15));
        assert_eq!(config.sample_rate, Some(44100));
        assert_eq!(config.channels, Some(2));
        assert!(!config.deterministic);
    }
}
