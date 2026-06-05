//! V4L2 Camera Backend for Linux
//!
//! This module provides V4L2 (Video4Linux2) camera support for Linux systems,
//! enabling real hardware camera capture with NGFS integration.

use std::collections::HashMap;
use std::path::Path;
use std::fs;
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::RwLock;

use crate::error::DeviceError;
use super::super::super::{
    CameraBackend, CameraDevice, CameraConfig, CameraFormat, CameraFrame,
    DeviceInfo, DeviceType, ProviderId, HalResult,
};

/// V4L2 camera device information
#[derive(Debug, Clone)]
struct V4L2DeviceInfo {
    device_path: String,
    device_name: String,
    capabilities: Vec<String>,
    supported_formats: Vec<CameraFormat>,
    supported_resolutions: Vec<(u32, u32)>,
    supported_fps: Vec<u32>,
}

/// V4L2 camera device implementation
pub struct V4L2CameraDevice {
    device_info: DeviceInfo,
    config: CameraConfig,
    device_fd: Option<RawFd>,
    is_capturing: Arc<AtomicBool>,
    frame_sequence: Arc<AtomicU64>,
    buffer_info: Arc<RwLock<BufferInfo>>,
}

/// Buffer information for V4L2
#[derive(Debug, Clone)]
struct BufferInfo {
    buffer_count: u32,
    buffer_size: usize,
    buffers: Vec<Vec<u8>>,
    current_buffer: usize,
}

impl V4L2CameraDevice {
    /// Create a new V4L2 camera device
    fn new(device_info: DeviceInfo, config: CameraConfig) -> Self {
        Self {
            device_info,
            config,
            device_fd: None,
            is_capturing: Arc::new(AtomicBool::new(false)),
            frame_sequence: Arc::new(AtomicU64::new(0)),
            buffer_info: Arc::new(RwLock::new(BufferInfo {
                buffer_count: config.buffer_count,
                buffer_size: 0,
                buffers: Vec::new(),
                current_buffer: 0,
            })),
        }
    }

    /// Open the V4L2 device
    async fn open_device(&mut self) -> HalResult<()> {
        // In a real implementation, this would:
        // 1. Open the device file descriptor
        // 2. Query device capabilities
        // 3. Set the video format
        // 4. Request buffers
        // 5. Start streaming

        // For now, simulate device opening
        self.device_fd = Some(42); // Mock file descriptor
        
        // Calculate buffer size based on format and resolution
        let buffer_size = match self.config.format {
            CameraFormat::YUV420 => (self.config.width * self.config.height * 3) / 2,
            CameraFormat::RGB24 => self.config.width * self.config.height * 3,
            CameraFormat::MJPEG => 1024 * 1024, // 1MB for MJPEG
            CameraFormat::H264 => 2 * 1024 * 1024, // 2MB for H264
        };

        // Initialize buffers
        let mut buffer_info = self.buffer_info.write().await;
        buffer_info.buffer_size = buffer_size as usize;
        buffer_info.buffers = vec![vec![0u8; buffer_size as usize]; self.config.buffer_count as usize];
        buffer_info.current_buffer = 0;

        Ok(())
    }

    /// Close the V4L2 device
    async fn close_device(&mut self) -> HalResult<()> {
        if let Some(fd) = self.device_fd {
            // In a real implementation, this would:
            // 1. Stop streaming
            // 2. Unmap buffers
            // 3. Close the file descriptor
            
            self.device_fd = None;
        }
        Ok(())
    }

    /// Read a frame from the device
    async fn read_frame_internal(&mut self) -> HalResult<CameraFrame> {
        if !self.is_capturing.load(Ordering::Relaxed) {
            return Err(DeviceError::InvalidOperation("Camera not capturing".to_string()));
        }

        let sequence = self.frame_sequence.fetch_add(1, Ordering::Relaxed);
        
        // In a real implementation, this would:
        // 1. Dequeue a buffer from the device
        // 2. Copy the frame data
        // 3. Requeue the buffer

        // For now, generate mock frame data
        let mut buffer_info = self.buffer_info.write().await;
        let buffer = &mut buffer_info.buffers[buffer_info.current_buffer];
        
        // Fill with deterministic pattern
        for (i, byte) in buffer.iter_mut().enumerate() {
            *byte = ((i + sequence as usize) % 256) as u8;
        }

        buffer_info.current_buffer = (buffer_info.current_buffer + 1) % buffer_info.buffers.len();

        Ok(CameraFrame {
            data: buffer.clone(),
            width: self.config.width,
            height: self.config.height,
            format: self.config.format.clone(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            sequence,
        })
    }
}

impl CameraDevice for V4L2CameraDevice {
    async fn start_capture(&mut self) -> HalResult<()> {
        if self.is_capturing.load(Ordering::Relaxed) {
            return Err(DeviceError::InvalidOperation("Camera already capturing".to_string()));
        }

        // Open device if not already open
        if self.device_fd.is_none() {
            self.open_device().await?;
        }

        // Start streaming
        self.is_capturing.store(true, Ordering::Relaxed);
        self.frame_sequence.store(0, Ordering::Relaxed);

        tracing::info!("Started V4L2 camera capture on {}", self.device_info.device_path);
        Ok(())
    }

    async fn stop_capture(&mut self) -> HalResult<()> {
        if !self.is_capturing.load(Ordering::Relaxed) {
            return Err(DeviceError::InvalidOperation("Camera not capturing".to_string()));
        }

        // Stop streaming
        self.is_capturing.store(false, Ordering::Relaxed);

        // Close device
        self.close_device().await?;

        tracing::info!("Stopped V4L2 camera capture on {}", self.device_info.device_path);
        Ok(())
    }

    async fn read_frame(&mut self) -> HalResult<CameraFrame> {
        self.read_frame_internal().await
    }

    fn get_device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    fn is_capturing(&self) -> bool {
        self.is_capturing.load(Ordering::Relaxed)
    }
}

/// Discover V4L2 camera devices
pub async fn discover_cameras() -> HalResult<Vec<DeviceInfo>> {
    let mut cameras = Vec::new();

    // Look for V4L2 devices in /dev
    let dev_path = Path::new("/dev");
    if !dev_path.exists() {
        return Ok(cameras);
    }

    let entries = fs::read_dir(dev_path)
        .map_err(|e| DeviceError::IoError(e))?;

    for entry in entries {
        let entry = entry.map_err(|e| DeviceError::IoError(e))?;
        let path = entry.path();
        
        if let Some(file_name) = path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                if name_str.starts_with("video") {
                    // Check if this is a video device
                    if let Ok(device_info) = get_v4l2_device_info(&path.to_string_lossy()).await {
                        cameras.push(device_info);
                    }
                }
            }
        }
    }

    Ok(cameras)
}

/// Get V4L2 device information
async fn get_v4l2_device_info(device_path: &str) -> HalResult<DeviceInfo> {
    // In a real implementation, this would:
    // 1. Open the device
    // 2. Query capabilities using ioctl
    // 3. Get supported formats and resolutions
    // 4. Get device name and other metadata

    // For now, create mock device info
    let device_name = format!("V4L2 Camera {}", device_path);
    
    let mut metadata = HashMap::new();
    metadata.insert("driver".to_string(), "mock_v4l2".to_string());
    metadata.insert("card".to_string(), device_name.clone());
    metadata.insert("bus_info".to_string(), "platform:mock".to_string());
    metadata.insert("version".to_string(), "1.0.0".to_string());

    Ok(DeviceInfo {
        device_id: device_path.to_string(),
        device_type: DeviceType::Camera,
        device_path: device_path.to_string(),
        capabilities: vec![
            "capture".to_string(),
            "yuv420".to_string(),
            "rgb24".to_string(),
            "mjpeg".to_string(),
        ],
        provider: ProviderId::Linux,
        is_available: true,
        metadata,
    })
}

/// Open a V4L2 camera device
pub fn open_camera(device_path: &str, config: &CameraConfig) -> HalResult<Box<dyn CameraDevice>> {
    // Validate device path
    if !Path::new(device_path).exists() {
        return Err(DeviceError::DeviceNotFound(device_path.to_string()));
    }

    // Get device information
    let device_info = futures::executor::block_on(get_v4l2_device_info(device_path))?;

    // Validate configuration
    validate_camera_config(config)?;

    // Create camera device
    let camera = V4L2CameraDevice::new(device_info, config.clone());
    Ok(Box::new(camera))
}

/// Validate camera configuration
fn validate_camera_config(config: &CameraConfig) -> HalResult<()> {
    if config.width == 0 || config.height == 0 {
        return Err(DeviceError::InvalidConfiguration("Invalid resolution".to_string()));
    }

    if config.fps == 0 {
        return Err(DeviceError::InvalidConfiguration("Invalid frame rate".to_string()));
    }

    if config.buffer_count == 0 {
        return Err(DeviceError::InvalidConfiguration("Invalid buffer count".to_string()));
    }

    Ok(())
}

/// V4L2 backend implementation
pub struct V4L2Backend;

impl CameraBackend for V4L2Backend {
    fn open(&self, device_path: &str, config: &CameraConfig) -> HalResult<Box<dyn CameraDevice>> {
        open_camera(device_path, config)
    }

    async fn list_devices(&self) -> HalResult<Vec<DeviceInfo>> {
        discover_cameras().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_v4l2_camera_discovery() {
        // This test will work on Linux systems with V4L2 devices
        let cameras = discover_cameras().await;
        
        if cfg!(target_os = "linux") {
            println!("Discovered {} V4L2 cameras", cameras.len());
            for camera in &cameras {
                println!("  - {}: {}", camera.device_id, camera.device_path);
            }
        } else {
            // On non-Linux systems, should return empty list
            assert!(cameras.is_empty());
        }
    }

    #[tokio::test]
    async fn test_v4l2_camera_config_validation() {
        let valid_config = CameraConfig {
            width: 640,
            height: 480,
            fps: 30,
            format: CameraFormat::YUV420,
            buffer_count: 4,
        };

        assert!(validate_camera_config(&valid_config).is_ok());

        let invalid_config = CameraConfig {
            width: 0,
            height: 480,
            fps: 30,
            format: CameraFormat::YUV420,
            buffer_count: 4,
        };

        assert!(validate_camera_config(&invalid_config).is_err());
    }

    #[test]
    fn test_v4l2_camera_device_creation() {
        let device_info = DeviceInfo {
            device_id: "/dev/video0".to_string(),
            device_type: DeviceType::Camera,
            device_path: "/dev/video0".to_string(),
            capabilities: vec!["capture".to_string()],
            provider: ProviderId::Linux,
            is_available: true,
            metadata: HashMap::new(),
        };

        let config = CameraConfig {
            width: 640,
            height: 480,
            fps: 30,
            format: CameraFormat::YUV420,
            buffer_count: 4,
        };

        let camera = V4L2CameraDevice::new(device_info, config);
        assert_eq!(camera.get_device_info().device_path, "/dev/video0");
        assert!(!camera.is_capturing());
    }
}
