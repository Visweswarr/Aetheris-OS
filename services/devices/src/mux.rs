//! Media multiplexer for handling timebase synchronization and chunking

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use blake3::Hasher;

use crate::error::DeviceError;

/// Media multiplexer that handles timebase synchronization and chunking
pub struct MediaMuxer {
    active_sessions: Arc<RwLock<HashMap<String, MuxSession>>>,
    timebase_hz: u64,
}

#[derive(Debug, Clone)]
struct MuxSession {
    session_id: String,
    timebase_hz: u64,
    start_time: chrono::DateTime<chrono::Utc>,
    chunks: Vec<MuxChunk>,
    deterministic: bool,
}

#[derive(Debug, Clone)]
struct MuxChunk {
    chunk_id: String,
    timestamp: u64, // In timebase units
    duration: u64,  // In timebase units
    size: u64,
    hash: String,
    media_type: MediaType,
}

#[derive(Debug, Clone)]
enum MediaType {
    Video,
    Audio,
    Metadata,
}

impl MediaMuxer {
    pub fn new() -> Result<Self, DeviceError> {
        Ok(Self {
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            timebase_hz: 90000, // 90 kHz timebase (standard for media)
        })
    }
    
    /// Create a new mux session
    pub async fn create_session(
        &self,
        session_id: &str,
        deterministic: bool,
    ) -> Result<(), DeviceError> {
        let session = MuxSession {
            session_id: session_id.to_string(),
            timebase_hz: self.timebase_hz,
            start_time: chrono::Utc::now(),
            chunks: Vec::new(),
            deterministic,
        };
        
        self.active_sessions.write().await.insert(session_id.to_string(), session);
        Ok(())
    }
    
    /// Add a media chunk to a session
    pub async fn add_chunk(
        &self,
        session_id: &str,
        media_type: MediaType,
        data: &[u8],
        duration_ms: u64,
    ) -> Result<String, DeviceError> {
        let chunk_id = Uuid::new_v4().to_string();
        
        // Convert duration to timebase units
        let duration_timebase = (duration_ms * self.timebase_hz) / 1000;
        
        // Calculate timestamp
        let timestamp = {
            let sessions = self.active_sessions.read().await;
            let session = sessions.get(session_id)
                .ok_or_else(|| DeviceError::DeviceNotFound(session_id.to_string()))?;
            
            // Calculate current timestamp based on session start time
            let elapsed = chrono::Utc::now() - session.start_time;
            (elapsed.num_milliseconds() as u64 * self.timebase_hz) / 1000
        };
        
        // Calculate hash
        let mut hasher = Hasher::new();
        hasher.update(data);
        let hash = hex::encode(hasher.finalize().as_bytes());
        
        let chunk = MuxChunk {
            chunk_id: chunk_id.clone(),
            timestamp,
            duration: duration_timebase,
            size: data.len() as u64,
            hash,
            media_type,
        };
        
        // Add to session
        {
            let mut sessions = self.active_sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.chunks.push(chunk);
            }
        }
        
        Ok(chunk_id)
    }
    
    /// Finalize a session and create snapshot
    pub async fn finalize_session(&self, session_id: &str) -> Result<String, DeviceError> {
        let session = {
            let mut sessions = self.active_sessions.write().await;
            sessions.remove(session_id)
                .ok_or_else(|| DeviceError::DeviceNotFound(session_id.to_string()))?
        };
        
        let snapshot_id = Uuid::new_v4().to_string();
        
        // Create manifest
        let manifest = MuxManifest {
            snapshot_id: snapshot_id.clone(),
            session_id: session.session_id,
            timebase_hz: session.timebase_hz,
            start_time: session.start_time,
            end_time: chrono::Utc::now(),
            chunks: session.chunks,
            deterministic: session.deterministic,
        };
        
        // Serialize manifest
        let manifest_json = serde_json::to_string(&manifest)?;
        
        // TODO: Store in NGFS
        tracing::info!("Finalized mux session {} with snapshot {}", session_id, snapshot_id);
        
        Ok(snapshot_id)
    }
    
    /// Get session statistics
    pub async fn get_session_stats(&self, session_id: &str) -> Result<MuxStats, DeviceError> {
        let session = {
            let sessions = self.active_sessions.read().await;
            sessions.get(session_id).cloned()
                .ok_or_else(|| DeviceError::DeviceNotFound(session_id.to_string()))?
        };
        
        let total_size: u64 = session.chunks.iter().map(|c| c.size).sum();
        let total_duration_ms = {
            let elapsed = chrono::Utc::now() - session.start_time;
            elapsed.num_milliseconds() as u64
        };
        
        Ok(MuxStats {
            session_id: session_id.to_string(),
            chunk_count: session.chunks.len() as u32,
            total_size,
            total_duration_ms,
            timebase_hz: session.timebase_hz,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct MuxManifest {
    snapshot_id: String,
    session_id: String,
    timebase_hz: u64,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
    chunks: Vec<MuxChunk>,
    deterministic: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MuxStats {
    pub session_id: String,
    pub chunk_count: u32,
    pub total_size: u64,
    pub total_duration_ms: u64,
    pub timebase_hz: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_media_muxer_creation() {
        let muxer = MediaMuxer::new();
        assert!(muxer.is_ok());
    }
    
    #[tokio::test]
    async fn test_mux_session_lifecycle() {
        let muxer = MediaMuxer::new().unwrap();
        let session_id = "test_session";
        
        // Create session
        muxer.create_session(session_id, false).await.unwrap();
        
        // Add chunks
        let data1 = b"video_data";
        let data2 = b"audio_data";
        
        muxer.add_chunk(session_id, MediaType::Video, data1, 1000).await.unwrap();
        muxer.add_chunk(session_id, MediaType::Audio, data2, 1000).await.unwrap();
        
        // Get stats
        let stats = muxer.get_session_stats(session_id).await.unwrap();
        assert_eq!(stats.chunk_count, 2);
        
        // Finalize session
        let snapshot_id = muxer.finalize_session(session_id).await.unwrap();
        assert!(!snapshot_id.is_empty());
    }
}
