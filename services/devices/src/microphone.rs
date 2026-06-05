//! Microphone capture manager with deterministic recording and NGFS integration

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use blake3::Hasher;

use crate::error::DeviceError;
use crate::CaptureConfig;

/// Microphone manager that handles microphone capture operations
pub struct MicrophoneManager {
    active_captures: Arc<RwLock<HashMap<String, MicrophoneCapture>>>,
    mock_driver: Arc<MockMicrophoneDriver>,
}

#[derive(Debug, Clone)]
struct MicrophoneCapture {
    capture_id: String,
    config: CaptureConfig,
    start_time: chrono::DateTime<chrono::Utc>,
    deterministic: bool,
    sample_count: u64,
    bytes_written: u64,
    chunks: Vec<AudioChunkInfo>,
}

#[derive(Debug, Clone)]
struct AudioChunkInfo {
    chunk_id: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    size: u64,
    hash: String,
    sample_count: u64,
    duration_ms: u64,
}

/// Mock microphone driver for CI/testing
struct MockMicrophoneDriver {
    deterministic_seed: Option<u64>,
}

impl MockMicrophoneDriver {
    fn new() -> Self {
        Self {
            deterministic_seed: None,
        }
    }
    
    fn set_deterministic_seed(&mut self, seed: u64) {
        self.deterministic_seed = Some(seed);
    }
    
    fn generate_samples(&self, sample_rate: u32, channels: u16, duration_ms: u64) -> Vec<i16> {
        let samples_per_channel = (sample_rate as u64 * duration_ms) / 1000;
        let total_samples = samples_per_channel * channels as u64;
        let mut samples = vec![0i16; total_samples as usize];
        
        // Generate deterministic audio pattern
        for (i, sample) in samples.iter_mut().enumerate() {
            let mut hasher = Hasher::new();
            
            if let Some(seed) = self.deterministic_seed {
                hasher.update(&seed.to_le_bytes());
            }
            hasher.update(&(i as u64).to_le_bytes());
            hasher.update(&sample_rate.to_le_bytes());
            hasher.update(&channels.to_le_bytes());
            
            let hash = hasher.finalize();
            // Convert hash to 16-bit signed integer
            *sample = ((hash.as_bytes()[0] as i16) << 8) | (hash.as_bytes()[1] as i16);
        }
        
        samples
    }
}

impl MicrophoneManager {
    pub fn new() -> Result<Self, DeviceError> {
        Ok(Self {
            active_captures: Arc::new(RwLock::new(HashMap::new())),
            mock_driver: Arc::new(MockMicrophoneDriver::new()),
        })
    }
    
    /// Start a microphone capture
    pub async fn start_capture(&self, capture_id: &str, config: &CaptureConfig) -> Result<(), DeviceError> {
        let sample_rate = config.sample_rate.unwrap_or(44100);
        let channels = config.channels.unwrap_or(2);
        
        let capture = MicrophoneCapture {
            capture_id: capture_id.to_string(),
            config: config.clone(),
            start_time: chrono::Utc::now(),
            deterministic: config.deterministic,
            sample_count: 0,
            bytes_written: 0,
            chunks: Vec::new(),
        };
        
        self.active_captures.write().await.insert(capture_id.to_string(), capture);
        
        // Start capture thread
        self.start_capture_thread(capture_id, sample_rate, channels).await?;
        
        Ok(())
    }
    
    /// Stop a microphone capture and return snapshot ID
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
    
    async fn start_capture_thread(&self, capture_id: &str, sample_rate: u32, channels: u16) -> Result<(), DeviceError> {
        let captures = self.active_captures.clone();
        let driver = self.mock_driver.clone();
        let capture_id = capture_id.to_string();
        
        tokio::spawn(async move {
            let mut chunk_start_time = chrono::Utc::now();
            let mut chunk_samples = Vec::new();
            let mut total_samples = 0u64;
            
            loop {
                // Check if capture still exists
                {
                    let captures_guard = captures.read().await;
                    if !captures_guard.contains_key(&capture_id) {
                        break;
                    }
                }
                
                // Generate audio samples (100ms chunks)
                let samples = driver.generate_samples(sample_rate, channels, 100);
                chunk_samples.extend(samples);
                total_samples += 100 * sample_rate as u64 / 1000 * channels as u64;
                
                // Check if we should seal the chunk
                let now = chrono::Utc::now();
                let chunk_duration = now - chunk_start_time;
                let should_seal = chunk_duration.num_milliseconds() >= 2000;
                
                if should_seal {
                    // Convert samples to bytes (16-bit PCM)
                    let sample_bytes: Vec<u8> = chunk_samples.iter()
                        .flat_map(|&sample| sample.to_le_bytes())
                        .collect();
                    
                    // Seal chunk
                    let chunk_id = Uuid::new_v4().to_string();
                    let chunk_size = sample_bytes.len() as u64;
                    
                    // Calculate chunk hash
                    let mut hasher = Hasher::new();
                    hasher.update(&sample_bytes);
                    let chunk_hash = hex::encode(hasher.finalize().as_bytes());
                    
                    let chunk_info = AudioChunkInfo {
                        chunk_id: chunk_id.clone(),
                        timestamp: now,
                        size: chunk_size,
                        hash: chunk_hash,
                        sample_count: chunk_samples.len() as u64 / channels as u64,
                        duration_ms: chunk_duration.num_milliseconds() as u64,
                    };
                    
                    // Update capture info
                    {
                        let mut captures_guard = captures.write().await;
                        if let Some(capture) = captures_guard.get_mut(&capture_id) {
                            capture.chunks.push(chunk_info);
                            capture.sample_count += chunk_samples.len() as u64 / channels as u64;
                            capture.bytes_written += chunk_size;
                        }
                    }
                    
                    // Reset chunk
                    chunk_samples.clear();
                    chunk_start_time = now;
                }
                
                // Simulate real-time capture
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });
        
        Ok(())
    }
    
    async fn create_snapshot(&self, capture: &MicrophoneCapture) -> Result<String, DeviceError> {
        let snapshot_id = Uuid::new_v4().to_string();
        
        // Create manifest
        let manifest = MicrophoneManifest {
            snapshot_id: snapshot_id.clone(),
            capture_id: capture.capture_id.clone(),
            config: capture.config.clone(),
            start_time: capture.start_time,
            end_time: chrono::Utc::now(),
            total_samples: capture.sample_count,
            total_bytes: capture.bytes_written,
            chunks: capture.chunks.clone(),
            deterministic: capture.deterministic,
        };
        
        // Serialize manifest
        let manifest_json = serde_json::to_string(&manifest)?;
        
        // TODO: Store in NGFS
        tracing::info!("Created microphone snapshot {} with {} chunks", snapshot_id, capture.chunks.len());
        
        Ok(snapshot_id)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct MicrophoneManifest {
    snapshot_id: String,
    capture_id: String,
    config: CaptureConfig,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
    total_samples: u64,
    total_bytes: u64,
    chunks: Vec<AudioChunkInfo>,
    deterministic: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_microphone_manager_creation() {
        let manager = MicrophoneManager::new();
        assert!(manager.is_ok());
    }
    
    #[tokio::test]
    async fn test_microphone_capture_lifecycle() {
        let manager = MicrophoneManager::new().unwrap();
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
