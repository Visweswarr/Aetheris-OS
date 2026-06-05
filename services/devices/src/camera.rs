//! Camera capture manager with deterministic recording and NGFS integration

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use blake3::Hasher;

use crate::error::DeviceError;
use crate::CaptureConfig;

/// Camera manager that handles camera capture operations
pub struct CameraManager {
    active_captures: Arc<RwLock<HashMap<String, CameraCapture>>>,
    mock_driver: Arc<MockCameraDriver>,
}

#[derive(Debug, Clone)]
struct CameraCapture {
    capture_id: String,
    config: CaptureConfig,
    start_time: chrono::DateTime<chrono::Utc>,
    deterministic: bool,
    frame_count: u32,
    bytes_written: u64,
    chunks: Vec<ChunkInfo>,
}

#[derive(Debug, Clone)]
struct ChunkInfo {
    chunk_id: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    size: u64,
    hash: String,
    frame_count: u32,
}

/// Mock camera driver for CI/testing
struct MockCameraDriver {
    deterministic_seed: Option<u64>,
}

impl MockCameraDriver {
    fn new() -> Self {
        Self {
            deterministic_seed: None,
        }
    }
    
    fn set_deterministic_seed(&mut self, seed: u64) {
        self.deterministic_seed = Some(seed);
    }
    
    fn generate_frame(&self, width: u32, height: u32, frame_number: u32) -> Vec<u8> {
        let mut hasher = Hasher::new();
        
        if let Some(seed) = self.deterministic_seed {
            hasher.update(&seed.to_le_bytes());
        }
        hasher.update(&frame_number.to_le_bytes());
        hasher.update(&width.to_le_bytes());
        hasher.update(&height.to_le_bytes());
        
        let hash = hasher.finalize();
        let mut frame_data = vec![0u8; (width * height * 3) as usize];
        
        // Fill with deterministic pattern based on hash
        for (i, byte) in frame_data.iter_mut().enumerate() {
            *byte = hash.as_bytes()[i % 32];
        }
        
        frame_data
    }
}

impl CameraManager {
    pub fn new() -> Result<Self, DeviceError> {
        Ok(Self {
            active_captures: Arc::new(RwLock::new(HashMap::new())),
            mock_driver: Arc::new(MockCameraDriver::new()),
        })
    }
    
    /// Start a camera capture
    pub async fn start_capture(&self, capture_id: &str, config: &CaptureConfig) -> Result<(), DeviceError> {
        let width = config.width.unwrap_or(640);
        let height = config.height.unwrap_or(360);
        
        let capture = CameraCapture {
            capture_id: capture_id.to_string(),
            config: config.clone(),
            start_time: chrono::Utc::now(),
            deterministic: config.deterministic,
            frame_count: 0,
            bytes_written: 0,
            chunks: Vec::new(),
        };
        
        self.active_captures.write().await.insert(capture_id.to_string(), capture);
        
        // Start capture thread
        self.start_capture_thread(capture_id, width, height).await?;
        
        Ok(())
    }
    
    /// Stop a camera capture and return snapshot ID
    pub async fn stop_capture(&self, capture_id: &str) -> Result<String, DeviceError> {
        let capture = {
            let mut captures = self.active_captures.write().await;
            captures.remove(capture_id)
                .ok_or_else(|| DeviceError::CaptureNotFound(capture_id.to_string()))?
        };
        
        // Create final snapshot
        let snapshot_id = self.create_snapshot(&capture).await?;
        
        Ok(snapshot_id)
    }
    
    /// Get a preview frame (throttled)
    pub async fn get_preview_frame(&self, capture_id: &str) -> Result<Vec<u8>, DeviceError> {
        let capture = {
            let captures = self.active_captures.read().await;
            captures.get(capture_id).cloned()
                .ok_or_else(|| DeviceError::CaptureNotFound(capture_id.to_string()))?
        };
        
        let width = capture.config.width.unwrap_or(640);
        let height = capture.config.height.unwrap_or(360);
        
        // Generate preview frame (scaled down)
        let preview_width = width / 4;
        let preview_height = height / 4;
        let frame_data = self.mock_driver.generate_frame(preview_width, preview_height, capture.frame_count);
        
        Ok(frame_data)
    }
    
    async fn start_capture_thread(&self, capture_id: &str, width: u32, height: u32) -> Result<(), DeviceError> {
        let captures = self.active_captures.clone();
        let driver = self.mock_driver.clone();
        let capture_id = capture_id.to_string();
        
        tokio::spawn(async move {
            let mut frame_number = 0u32;
            let mut chunk_start_time = chrono::Utc::now();
            let mut chunk_frames = Vec::new();
            
            loop {
                // Check if capture still exists
                {
                    let captures_guard = captures.read().await;
                    if !captures_guard.contains_key(&capture_id) {
                        break;
                    }
                }
                
                // Generate frame
                let frame_data = driver.generate_frame(width, height, frame_number);
                
                // Add to current chunk
                chunk_frames.push(frame_data);
                
                // Check if we should seal the chunk
                let now = chrono::Utc::now();
                let chunk_duration = now - chunk_start_time;
                let should_seal = chunk_duration.num_milliseconds() >= 2000 || chunk_frames.len() >= 30;
                
                if should_seal {
                    // Seal chunk
                    let chunk_id = Uuid::new_v4().to_string();
                    let chunk_size: u64 = chunk_frames.iter().map(|f| f.len() as u64).sum();
                    
                    // Calculate chunk hash
                    let mut hasher = Hasher::new();
                    for frame in &chunk_frames {
                        hasher.update(frame);
                    }
                    let chunk_hash = hex::encode(hasher.finalize().as_bytes());
                    
                    let chunk_info = ChunkInfo {
                        chunk_id: chunk_id.clone(),
                        timestamp: now,
                        size: chunk_size,
                        hash: chunk_hash,
                        frame_count: chunk_frames.len() as u32,
                    };
                    
                    // Update capture info
                    {
                        let mut captures_guard = captures.write().await;
                        if let Some(capture) = captures_guard.get_mut(&capture_id) {
                            capture.chunks.push(chunk_info);
                            capture.frame_count += chunk_frames.len() as u32;
                            capture.bytes_written += chunk_size;
                        }
                    }
                    
                    // Reset chunk
                    chunk_frames.clear();
                    chunk_start_time = now;
                }
                
                frame_number += 1;
                
                // Simulate frame rate
                let fps = 15; // Default FPS
                let frame_duration = std::time::Duration::from_millis(1000 / fps as u64);
                tokio::time::sleep(frame_duration).await;
            }
        });
        
        Ok(())
    }
    
    async fn create_snapshot(&self, capture: &CameraCapture) -> Result<String, DeviceError> {
        let snapshot_id = Uuid::new_v4().to_string();
        
        // Create manifest
        let manifest = CameraManifest {
            snapshot_id: snapshot_id.clone(),
            capture_id: capture.capture_id.clone(),
            config: capture.config.clone(),
            start_time: capture.start_time,
            end_time: chrono::Utc::now(),
            total_frames: capture.frame_count,
            total_bytes: capture.bytes_written,
            chunks: capture.chunks.clone(),
            deterministic: capture.deterministic,
        };
        
        // Serialize manifest
        let manifest_json = serde_json::to_string(&manifest)?;
        
        // TODO: Store in NGFS
        tracing::info!("Created camera snapshot {} with {} chunks", snapshot_id, capture.chunks.len());
        
        Ok(snapshot_id)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CameraManifest {
    snapshot_id: String,
    capture_id: String,
    config: CaptureConfig,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
    total_frames: u32,
    total_bytes: u64,
    chunks: Vec<ChunkInfo>,
    deterministic: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_camera_manager_creation() {
        let manager = CameraManager::new();
        assert!(manager.is_ok());
    }
    
    #[tokio::test]
    async fn test_camera_capture_lifecycle() {
        let manager = CameraManager::new().unwrap();
        let config = CaptureConfig::default();
        
        // Start capture
        let capture_id = "test_capture";
        manager.start_capture(capture_id, &config).await.unwrap();
        
        // Wait a bit
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Stop capture
        let snapshot_id = manager.stop_capture(capture_id).await.unwrap();
        assert!(!snapshot_id.is_empty());
    }
}
