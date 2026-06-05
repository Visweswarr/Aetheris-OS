//! RPC handlers for device capture operations

use std::sync::Arc;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::error::DeviceError;
use crate::{DeviceService, CaptureConfig, CaptureStats, CaptureStatus};
use crate::ble::{BleRuntime, BleScanFilters, BleDeviceInfo, BleNotification};
use crate::sensor::{SensorRuntime, SensorDesc, SensorInfo, SensorSample};

/// RPC request/response types for device operations

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraStartRequest {
    pub session_id: String,
    pub caps: String,
    pub config: CaptureConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraStartResponse {
    pub capture_id: String,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraStopRequest {
    pub capture_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraStopResponse {
    pub snapshot_id: String,
    pub stats: CaptureStats,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrophoneStartRequest {
    pub session_id: String,
    pub caps: String,
    pub config: CaptureConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrophoneStartResponse {
    pub capture_id: String,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrophoneStopRequest {
    pub capture_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrophoneStopResponse {
    pub snapshot_id: String,
    pub stats: CaptureStats,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatusRequest {
    pub capture_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatusResponse {
    pub status: CaptureStatus,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewFrameRequest {
    pub capture_id: String,
    pub session_id: String,
    pub caps: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewFrameResponse {
    pub frame_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub success: bool,
    pub message: Option<String>,
}

// BLE RPC request/response types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleScanStartRequest {
    pub session_id: String,
    pub caps: String,
    pub filters: BleScanFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleScanStartResponse {
    pub scan_handle: String,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleScanStopRequest {
    pub scan_handle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleScanStopResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleListDevicesRequest {
    // Empty request
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleListDevicesResponse {
    pub devices: Vec<BleDeviceInfo>,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleConnectRequest {
    pub session_id: String,
    pub caps: String,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleConnectResponse {
    pub conn_handle: String,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleDisconnectRequest {
    pub conn_handle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleDisconnectResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleGattReadRequest {
    pub conn_handle: String,
    pub service_uuid: String,
    pub char_uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleGattReadResponse {
    pub data: Vec<u8>,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleGattWriteRequest {
    pub conn_handle: String,
    pub service_uuid: String,
    pub char_uuid: String,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleGattWriteResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleSubscribeRequest {
    pub conn_handle: String,
    pub service_uuid: String,
    pub char_uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleSubscribeResponse {
    pub notify_handle: String,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleUnsubscribeRequest {
    pub notify_handle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleUnsubscribeResponse {
    pub success: bool,
    pub message: Option<String>,
}

// Sensor RPC request/response types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorRegisterRequest {
    pub desc: SensorDesc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorRegisterResponse {
    pub sensor_id: String,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorListRequest {
    // Empty request
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorListResponse {
    pub sensors: Vec<SensorInfo>,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorStartSamplingRequest {
    pub session_id: String,
    pub caps: String,
    pub sensor_id: String,
    pub hz: u32,
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorStartSamplingResponse {
    pub handle: String,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorStopSamplingRequest {
    pub handle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorStopSamplingResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorPreviewRequest {
    pub sensor_id: String,
    pub hz_max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorPreviewResponse {
    pub stream_handle: String,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorSnapshotRequest {
    pub sensor_id: String,
    pub out_ngfs_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorSnapshotResponse {
    pub snapshot_id: String,
    pub success: bool,
    pub message: Option<String>,
}

/// RPC service for device operations
pub struct DeviceRpcService {
    device_service: Arc<DeviceService>,
    ble_runtime: Option<Arc<dyn BleRuntime + Send + Sync>>,
    sensor_runtime: Option<Arc<dyn SensorRuntime + Send + Sync>>,
}

impl DeviceRpcService {
    pub fn new(
        device_service: Arc<DeviceService>,
        ble_runtime: Option<Arc<dyn BleRuntime + Send + Sync>>,
        sensor_runtime: Option<Arc<dyn SensorRuntime + Send + Sync>>,
    ) -> Self {
        Self { 
            device_service,
            ble_runtime,
            sensor_runtime,
        }
    }
    
    /// Start camera capture
    pub async fn start_camera_capture(
        &self,
        request: CameraStartRequest,
    ) -> Result<CameraStartResponse, DeviceError> {
        let capture_id = self.device_service
            .start_camera_capture(&request.session_id, &request.caps, request.config)
            .await?;
        
        Ok(CameraStartResponse {
            capture_id,
            success: true,
            message: None,
        })
    }
    
    /// Stop camera capture
    pub async fn stop_camera_capture(
        &self,
        request: CameraStopRequest,
    ) -> Result<CameraStopResponse, DeviceError> {
        let stats = self.device_service.stop_capture(&request.capture_id).await?;
        let snapshot_id = stats.snapshot_id.unwrap_or_default();
        
        Ok(CameraStopResponse {
            snapshot_id,
            stats,
            success: true,
            message: None,
        })
    }
    
    /// Start microphone capture
    pub async fn start_microphone_capture(
        &self,
        request: MicrophoneStartRequest,
    ) -> Result<MicrophoneStartResponse, DeviceError> {
        let capture_id = self.device_service
            .start_microphone_capture(&request.session_id, &request.caps, request.config)
            .await?;
        
        Ok(MicrophoneStartResponse {
            capture_id,
            success: true,
            message: None,
        })
    }
    
    /// Stop microphone capture
    pub async fn stop_microphone_capture(
        &self,
        request: MicrophoneStopRequest,
    ) -> Result<MicrophoneStopResponse, DeviceError> {
        let stats = self.device_service.stop_capture(&request.capture_id).await?;
        let snapshot_id = stats.snapshot_id.unwrap_or_default();
        
        Ok(MicrophoneStopResponse {
            snapshot_id,
            stats,
            success: true,
            message: None,
        })
    }
    
    /// Get device status
    pub async fn get_device_status(
        &self,
        request: DeviceStatusRequest,
    ) -> Result<DeviceStatusResponse, DeviceError> {
        let status = self.device_service.get_capture_status(&request.capture_id).await?;
        
        Ok(DeviceStatusResponse {
            status,
            success: true,
            message: None,
        })
    }
    
    /// Get preview frame
    pub async fn get_preview_frame(
        &self,
        request: PreviewFrameRequest,
    ) -> Result<PreviewFrameResponse, DeviceError> {
        let frame_data = self.device_service
            .get_preview_frame(&request.capture_id, &request.session_id, &request.caps)
            .await?;
        
        // TODO: Get actual dimensions from capture
        let width = 160; // Preview is scaled down
        let height = 90;
        
        Ok(PreviewFrameResponse {
            frame_data,
            width,
            height,
            timestamp: chrono::Utc::now(),
            success: true,
            message: None,
        })
    }

    // BLE RPC methods

    /// Start BLE scan
    pub async fn start_ble_scan(
        &self,
        request: BleScanStartRequest,
    ) -> Result<BleScanStartResponse, DeviceError> {
        let ble_runtime = self.ble_runtime.as_ref()
            .ok_or_else(|| DeviceError::InvalidOperation("BLE runtime not available".to_string()))?;
        
        let scan_handle = ble_runtime.start_scan(&request.session_id, &request.caps, request.filters).await?;
        
        Ok(BleScanStartResponse {
            scan_handle,
            success: true,
            message: None,
        })
    }

    /// Stop BLE scan
    pub async fn stop_ble_scan(
        &self,
        request: BleScanStopRequest,
    ) -> Result<BleScanStopResponse, DeviceError> {
        let ble_runtime = self.ble_runtime.as_ref()
            .ok_or_else(|| DeviceError::InvalidOperation("BLE runtime not available".to_string()))?;
        
        ble_runtime.stop_scan(&request.scan_handle).await?;
        
        Ok(BleScanStopResponse {
            success: true,
            message: None,
        })
    }

    /// List BLE devices
    pub async fn list_ble_devices(
        &self,
        _request: BleListDevicesRequest,
    ) -> Result<BleListDevicesResponse, DeviceError> {
        let ble_runtime = self.ble_runtime.as_ref()
            .ok_or_else(|| DeviceError::InvalidOperation("BLE runtime not available".to_string()))?;
        
        let devices = ble_runtime.list_devices().await?;
        
        Ok(BleListDevicesResponse {
            devices,
            success: true,
            message: None,
        })
    }

    /// Connect to BLE device
    pub async fn connect_ble_device(
        &self,
        request: BleConnectRequest,
    ) -> Result<BleConnectResponse, DeviceError> {
        let ble_runtime = self.ble_runtime.as_ref()
            .ok_or_else(|| DeviceError::InvalidOperation("BLE runtime not available".to_string()))?;
        
        let conn_handle = ble_runtime.connect(&request.session_id, &request.caps, &request.address).await?;
        
        Ok(BleConnectResponse {
            conn_handle,
            success: true,
            message: None,
        })
    }

    /// Disconnect from BLE device
    pub async fn disconnect_ble_device(
        &self,
        request: BleDisconnectRequest,
    ) -> Result<BleDisconnectResponse, DeviceError> {
        let ble_runtime = self.ble_runtime.as_ref()
            .ok_or_else(|| DeviceError::InvalidOperation("BLE runtime not available".to_string()))?;
        
        ble_runtime.disconnect(&request.conn_handle).await?;
        
        Ok(BleDisconnectResponse {
            success: true,
            message: None,
        })
    }

    /// Read GATT characteristic
    pub async fn read_gatt_characteristic(
        &self,
        request: BleGattReadRequest,
    ) -> Result<BleGattReadResponse, DeviceError> {
        let ble_runtime = self.ble_runtime.as_ref()
            .ok_or_else(|| DeviceError::InvalidOperation("BLE runtime not available".to_string()))?;
        
        let data = ble_runtime.gatt_read(&request.conn_handle, &request.service_uuid, &request.char_uuid).await?;
        
        Ok(BleGattReadResponse {
            data,
            success: true,
            message: None,
        })
    }

    /// Write GATT characteristic
    pub async fn write_gatt_characteristic(
        &self,
        request: BleGattWriteRequest,
    ) -> Result<BleGattWriteResponse, DeviceError> {
        let ble_runtime = self.ble_runtime.as_ref()
            .ok_or_else(|| DeviceError::InvalidOperation("BLE runtime not available".to_string()))?;
        
        ble_runtime.gatt_write(&request.conn_handle, &request.service_uuid, &request.char_uuid, &request.payload).await?;
        
        Ok(BleGattWriteResponse {
            success: true,
            message: None,
        })
    }

    /// Subscribe to GATT notifications
    pub async fn subscribe_gatt_notifications(
        &self,
        request: BleSubscribeRequest,
    ) -> Result<BleSubscribeResponse, DeviceError> {
        let ble_runtime = self.ble_runtime.as_ref()
            .ok_or_else(|| DeviceError::InvalidOperation("BLE runtime not available".to_string()))?;
        
        let notify_handle = ble_runtime.subscribe(&request.conn_handle, &request.service_uuid, &request.char_uuid).await?;
        
        Ok(BleSubscribeResponse {
            notify_handle,
            success: true,
            message: None,
        })
    }

    /// Unsubscribe from GATT notifications
    pub async fn unsubscribe_gatt_notifications(
        &self,
        request: BleUnsubscribeRequest,
    ) -> Result<BleUnsubscribeResponse, DeviceError> {
        let ble_runtime = self.ble_runtime.as_ref()
            .ok_or_else(|| DeviceError::InvalidOperation("BLE runtime not available".to_string()))?;
        
        ble_runtime.unsubscribe(&request.notify_handle).await?;
        
        Ok(BleUnsubscribeResponse {
            success: true,
            message: None,
        })
    }

    // Sensor RPC methods

    /// Register sensor
    pub async fn register_sensor(
        &self,
        request: SensorRegisterRequest,
    ) -> Result<SensorRegisterResponse, DeviceError> {
        let sensor_runtime = self.sensor_runtime.as_ref()
            .ok_or_else(|| DeviceError::InvalidOperation("Sensor runtime not available".to_string()))?;
        
        let sensor_id = sensor_runtime.register_sensor(request.desc).await?;
        
        Ok(SensorRegisterResponse {
            sensor_id,
            success: true,
            message: None,
        })
    }

    /// List sensors
    pub async fn list_sensors(
        &self,
        _request: SensorListRequest,
    ) -> Result<SensorListResponse, DeviceError> {
        let sensor_runtime = self.sensor_runtime.as_ref()
            .ok_or_else(|| DeviceError::InvalidOperation("Sensor runtime not available".to_string()))?;
        
        let sensors = sensor_runtime.list_sensors().await?;
        
        Ok(SensorListResponse {
            sensors,
            success: true,
            message: None,
        })
    }

    /// Start sensor sampling
    pub async fn start_sensor_sampling(
        &self,
        request: SensorStartSamplingRequest,
    ) -> Result<SensorStartSamplingResponse, DeviceError> {
        let sensor_runtime = self.sensor_runtime.as_ref()
            .ok_or_else(|| DeviceError::InvalidOperation("Sensor runtime not available".to_string()))?;
        
        let handle = sensor_runtime.start_sampling(&request.session_id, &request.caps, &request.sensor_id, request.hz, request.seed).await?;
        
        Ok(SensorStartSamplingResponse {
            handle,
            success: true,
            message: None,
        })
    }

    /// Stop sensor sampling
    pub async fn stop_sensor_sampling(
        &self,
        request: SensorStopSamplingRequest,
    ) -> Result<SensorStopSamplingResponse, DeviceError> {
        let sensor_runtime = self.sensor_runtime.as_ref()
            .ok_or_else(|| DeviceError::InvalidOperation("Sensor runtime not available".to_string()))?;
        
        sensor_runtime.stop_sampling(&request.handle).await?;
        
        Ok(SensorStopSamplingResponse {
            success: true,
            message: None,
        })
    }

    /// Start sensor preview
    pub async fn start_sensor_preview(
        &self,
        request: SensorPreviewRequest,
    ) -> Result<SensorPreviewResponse, DeviceError> {
        let sensor_runtime = self.sensor_runtime.as_ref()
            .ok_or_else(|| DeviceError::InvalidOperation("Sensor runtime not available".to_string()))?;
        
        let stream_handle = sensor_runtime.preview(&request.sensor_id, request.hz_max).await?;
        
        Ok(SensorPreviewResponse {
            stream_handle,
            success: true,
            message: None,
        })
    }

    /// Create sensor snapshot
    pub async fn create_sensor_snapshot(
        &self,
        request: SensorSnapshotRequest,
    ) -> Result<SensorSnapshotResponse, DeviceError> {
        let sensor_runtime = self.sensor_runtime.as_ref()
            .ok_or_else(|| DeviceError::InvalidOperation("Sensor runtime not available".to_string()))?;
        
        let snapshot_id = sensor_runtime.snapshot(&request.sensor_id, &request.out_ngfs_path).await?;
        
        Ok(SensorSnapshotResponse {
            snapshot_id,
            success: true,
            message: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rpc_service_creation() {
        let device_service = Arc::new(DeviceService::new().unwrap());
        let rpc_service = DeviceRpcService::new(device_service, None, None);
        
        // Test that service was created successfully
        assert!(true); // Service creation doesn't return anything to test
    }
    
    #[tokio::test]
    async fn test_camera_start_request_serialization() {
        let request = CameraStartRequest {
            session_id: "test_session".to_string(),
            caps: "device:camera.read".to_string(),
            config: CaptureConfig::default(),
        };
        
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CameraStartRequest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(request.session_id, deserialized.session_id);
        assert_eq!(request.caps, deserialized.caps);
    }
}
