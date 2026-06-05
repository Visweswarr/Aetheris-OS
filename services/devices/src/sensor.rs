//! Sensor Framework - Generic sensor registry and sampling engine
//!
//! This module provides a capability-gated sensor framework for managing
//! various types of sensors (accelerometer, gyroscope, temperature, light, etc.)
//! with deterministic sampling, preview streams, and NGFS persistence.
//!
//! Key patterns from reference OSes:
//! - Zephyr: Device tree integration, power profiles, pull model sampling
//! - RIOT: SAUL registry for sensor abstraction, event/interrupt model
//! - Tock: Capsules for sensors, grant regions, capability traits
//! - Fuchsia: FIDL device protocols, asynchronous driver model
//! - Genode: Capability-oriented device sessions

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use blake3::Hasher;

use crate::error::DeviceError;

/// Sensor runtime trait for device operations
#[async_trait::async_trait]
pub trait SensorRuntime: Send + Sync {
    async fn register_sensor(&self, desc: SensorDesc) -> Result<String, DeviceError>;
    async fn list_sensors(&self) -> Result<Vec<SensorInfo>, DeviceError>;
    async fn start_sampling(&self, session_id: &str, caps: &str, sensor_id: &str, hz: u32, seed: u64) -> Result<String, DeviceError>;
    async fn stop_sampling(&self, handle: &str) -> Result<(), DeviceError>;
    async fn preview(&self, sensor_id: &str, hz_max: u32) -> Result<String, DeviceError>;
    async fn snapshot(&self, sensor_id: &str, out_ngfs_path: &str) -> Result<String, DeviceError>;
}

/// Sensor description for registration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SensorDesc {
    pub name: String,
    pub kind: SensorKind,
    pub location: String,
    pub unit: String,
    pub range_min: f64,
    pub range_max: f64,
    pub resolution: f64,
    pub sample_rates: Vec<u32>,
}

/// Sensor kind enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SensorKind {
    Accelerometer,
    Gyroscope,
    Magnetometer,
    Temperature,
    Humidity,
    Pressure,
    Light,
    Proximity,
    HeartRate,
    Custom(String),
}

/// Sensor information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SensorInfo {
    pub sensor_id: String,
    pub name: String,
    pub kind: SensorKind,
    pub location: String,
    pub unit: String,
    pub range_min: f64,
    pub range_max: f64,
    pub resolution: f64,
    pub sample_rates: Vec<u32>,
    pub registered_at: chrono::DateTime<chrono::Utc>,
    pub active_sampling: bool,
}

/// Sensor sample data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SensorSample {
    pub sensor_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: SensorValue,
    pub sequence_number: u64,
}

/// Sensor value (scalar or vector)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SensorValue {
    Scalar(f64),
    Vector3 { x: f64, y: f64, z: f64 },
    Vector2 { x: f64, y: f64 },
}

/// Sampling session information
#[derive(Debug, Clone)]
pub struct SamplingSession {
    pub handle: String,
    pub sensor_id: String,
    pub session_id: String,
    pub hz: u32,
    pub seed: u64,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub sample_count: u64,
    pub deterministic: bool,
}

/// Preview stream information
#[derive(Debug, Clone)]
pub struct PreviewStream {
    pub stream_handle: String,
    pub sensor_id: String,
    pub hz_max: u32,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub sample_count: u64,
}

/// Mock sensor runtime implementation
pub struct MockSensorRuntime {
    registered_sensors: Arc<RwLock<HashMap<String, SensorInfo>>>,
    active_sampling: Arc<RwLock<HashMap<String, SamplingSession>>>,
    active_previews: Arc<RwLock<HashMap<String, PreviewStream>>>,
    deterministic_seed: Option<u64>,
}

impl MockSensorRuntime {
    pub fn new() -> Result<Self, DeviceError> {
        Ok(Self {
            registered_sensors: Arc::new(RwLock::new(HashMap::new())),
            active_sampling: Arc::new(RwLock::new(HashMap::new())),
            active_previews: Arc::new(RwLock::new(HashMap::new())),
            deterministic_seed: None,
        })
    }

    pub fn set_deterministic_seed(&mut self, seed: u64) {
        self.deterministic_seed = Some(seed);
    }

    fn generate_deterministic_sample(&self, sensor_id: &str, sequence: u64, kind: &SensorKind) -> SensorSample {
        let mut hasher = Hasher::new();
        
        if let Some(seed) = self.deterministic_seed {
            hasher.update(&seed.to_le_bytes());
        }
        hasher.update(sensor_id.as_bytes());
        hasher.update(&sequence.to_le_bytes());
        
        let hash = hasher.finalize();
        let hash_bytes = hash.as_bytes();
        
        // Generate deterministic value based on sensor kind
        let value = match kind {
            SensorKind::Accelerometer => {
                // Generate 3D acceleration vector
                let x = (hash_bytes[0] as f64 / 255.0) * 20.0 - 10.0; // -10 to +10 m/s²
                let y = (hash_bytes[1] as f64 / 255.0) * 20.0 - 10.0;
                let z = (hash_bytes[2] as f64 / 255.0) * 20.0 - 10.0;
                SensorValue::Vector3 { x, y, z }
            }
            SensorKind::Gyroscope => {
                // Generate 3D angular velocity vector
                let x = (hash_bytes[3] as f64 / 255.0) * 2000.0 - 1000.0; // -1000 to +1000 deg/s
                let y = (hash_bytes[4] as f64 / 255.0) * 2000.0 - 1000.0;
                let z = (hash_bytes[5] as f64 / 255.0) * 2000.0 - 1000.0;
                SensorValue::Vector3 { x, y, z }
            }
            SensorKind::Temperature => {
                // Generate temperature value
                let temp = (hash_bytes[6] as f64 / 255.0) * 50.0 + 10.0; // 10 to 60°C
                SensorValue::Scalar(temp)
            }
            SensorKind::Light => {
                // Generate light intensity
                let light = (hash_bytes[7] as f64 / 255.0) * 1000.0; // 0 to 1000 lux
                SensorValue::Scalar(light)
            }
            _ => {
                // Default scalar value
                let val = (hash_bytes[8] as f64 / 255.0) * 100.0;
                SensorValue::Scalar(val)
            }
        };
        
        SensorSample {
            sensor_id: sensor_id.to_string(),
            timestamp: chrono::Utc::now(),
            value,
            sequence_number: sequence,
        }
    }

    async fn start_sampling_thread(&self, session: SamplingSession) -> Result<(), DeviceError> {
        let sessions = self.active_sampling.clone();
        let sensors = self.registered_sensors.clone();
        let runtime = Arc::new(self.clone());
        
        tokio::spawn(async move {
            let mut sequence = 0u64;
            let interval_ms = 1000 / session.hz;
            
            loop {
                // Check if session is still active
                {
                    let sessions_guard = sessions.read().await;
                    if !sessions_guard.contains_key(&session.handle) {
                        break;
                    }
                }
                
                // Get sensor info
                let sensor_info = {
                    let sensors_guard = sensors.read().await;
                    sensors_guard.get(&session.sensor_id).cloned()
                };
                
                if let Some(sensor_info) = sensor_info {
                    // Generate sample
                    let sample = runtime.generate_deterministic_sample(
                        &session.sensor_id,
                        sequence,
                        &sensor_info.kind
                    );
                    
                    // Update session
                    {
                        let mut sessions_guard = sessions.write().await;
                        if let Some(session_ref) = sessions_guard.get_mut(&session.handle) {
                            session_ref.sample_count += 1;
                        }
                    }
                    
                    // TODO: Emit sample event or store in buffer
                    tracing::debug!("Sensor sample: {:?}", sample);
                    
                    sequence += 1;
                }
                
                // Wait for next sample
                tokio::time::sleep(std::time::Duration::from_millis(interval_ms)).await;
            }
        });
        
        Ok(())
    }

    async fn start_preview_thread(&self, stream: PreviewStream) -> Result<(), DeviceError> {
        let previews = self.active_previews.clone();
        let sensors = self.registered_sensors.clone();
        let runtime = Arc::new(self.clone());
        
        tokio::spawn(async move {
            let mut sequence = 0u64;
            let interval_ms = 1000 / stream.hz_max;
            
            loop {
                // Check if preview is still active
                {
                    let previews_guard = previews.read().await;
                    if !previews_guard.contains_key(&stream.stream_handle) {
                        break;
                    }
                }
                
                // Get sensor info
                let sensor_info = {
                    let sensors_guard = sensors.read().await;
                    sensors_guard.get(&stream.sensor_id).cloned()
                };
                
                if let Some(sensor_info) = sensor_info {
                    // Generate sample
                    let sample = runtime.generate_deterministic_sample(
                        &stream.sensor_id,
                        sequence,
                        &sensor_info.kind
                    );
                    
                    // Update preview
                    {
                        let mut previews_guard = previews.write().await;
                        if let Some(preview_ref) = previews_guard.get_mut(&stream.stream_handle) {
                            preview_ref.sample_count += 1;
                        }
                    }
                    
                    // TODO: Emit preview event
                    tracing::debug!("Preview sample: {:?}", sample);
                    
                    sequence += 1;
                }
                
                // Wait for next sample
                tokio::time::sleep(std::time::Duration::from_millis(interval_ms)).await;
            }
        });
        
        Ok(())
    }
}

#[async_trait::async_trait]
impl SensorRuntime for MockSensorRuntime {
    async fn register_sensor(&self, desc: SensorDesc) -> Result<String, DeviceError> {
        let sensor_id = Uuid::new_v4().to_string();
        
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
            registered_at: chrono::Utc::now(),
            active_sampling: false,
        };
        
        self.registered_sensors.write().await.insert(sensor_id.clone(), sensor_info);
        Ok(sensor_id)
    }

    async fn list_sensors(&self) -> Result<Vec<SensorInfo>, DeviceError> {
        let sensors = self.registered_sensors.read().await;
        Ok(sensors.values().cloned().collect())
    }

    async fn start_sampling(&self, session_id: &str, caps: &str, sensor_id: &str, hz: u32, seed: u64) -> Result<String, DeviceError> {
        // Validate capability
        if !caps.contains("device:sensor.sample") {
            return Err(DeviceError::InvalidCapability("device:sensor.sample required".to_string()));
        }

        // Check if sensor exists
        let sensor_info = {
            let sensors = self.registered_sensors.read().await;
            sensors.get(sensor_id).cloned()
                .ok_or_else(|| DeviceError::DeviceNotFound(sensor_id.to_string()))?
        };

        // Check if requested sample rate is supported
        if !sensor_info.sample_rates.contains(&hz) {
            return Err(DeviceError::InvalidOperation(format!("Sample rate {} Hz not supported", hz)));
        }

        let handle = Uuid::new_v4().to_string();
        
        let session = SamplingSession {
            handle: handle.clone(),
            sensor_id: sensor_id.to_string(),
            session_id: session_id.to_string(),
            hz,
            seed,
            start_time: chrono::Utc::now(),
            sample_count: 0,
            deterministic: seed != 0,
        };
        
        self.active_sampling.write().await.insert(handle.clone(), session.clone());
        
        // Update sensor status
        {
            let mut sensors = self.registered_sensors.write().await;
            if let Some(sensor) = sensors.get_mut(sensor_id) {
                sensor.active_sampling = true;
            }
        }
        
        // Start sampling thread
        self.start_sampling_thread(session).await?;
        
        Ok(handle)
    }

    async fn stop_sampling(&self, handle: &str) -> Result<(), DeviceError> {
        let session = {
            let mut sessions = self.active_sampling.write().await;
            sessions.remove(handle)
                .ok_or_else(|| DeviceError::DeviceNotFound(handle.to_string()))?
        };
        
        // Update sensor status
        {
            let mut sensors = self.registered_sensors.write().await;
            if let Some(sensor) = sensors.get_mut(&session.sensor_id) {
                sensor.active_sampling = false;
            }
        }
        
        Ok(())
    }

    async fn preview(&self, sensor_id: &str, hz_max: u32) -> Result<String, DeviceError> {
        // Check if sensor exists
        let _sensor_info = {
            let sensors = self.registered_sensors.read().await;
            sensors.get(sensor_id).cloned()
                .ok_or_else(|| DeviceError::DeviceNotFound(sensor_id.to_string()))?
        };

        let stream_handle = Uuid::new_v4().to_string();
        
        let stream = PreviewStream {
            stream_handle: stream_handle.clone(),
            sensor_id: sensor_id.to_string(),
            hz_max,
            start_time: chrono::Utc::now(),
            sample_count: 0,
        };
        
        self.active_previews.write().await.insert(stream_handle.clone(), stream.clone());
        
        // Start preview thread
        self.start_preview_thread(stream).await?;
        
        Ok(stream_handle)
    }

    async fn snapshot(&self, sensor_id: &str, out_ngfs_path: &str) -> Result<String, DeviceError> {
        // Check if sensor exists
        let sensor_info = {
            let sensors = self.registered_sensors.read().await;
            sensors.get(sensor_id).cloned()
                .ok_or_else(|| DeviceError::DeviceNotFound(sensor_id.to_string()))?
        };

        let snapshot_id = Uuid::new_v4().to_string();
        
        // Generate deterministic samples for snapshot
        let mut samples = Vec::new();
        let sample_count = 100; // Generate 100 samples for snapshot
        
        for i in 0..sample_count {
            let sample = self.generate_deterministic_sample(sensor_id, i, &sensor_info.kind);
            samples.push(sample);
        }
        
        // Create manifest
        let manifest = SensorSnapshotManifest {
            snapshot_id: snapshot_id.clone(),
            sensor_id: sensor_id.to_string(),
            sensor_info: sensor_info.clone(),
            sample_count,
            start_time: chrono::Utc::now(),
            end_time: chrono::Utc::now(),
            samples: samples.clone(),
            deterministic: true,
        };
        
        // Serialize manifest
        let manifest_json = serde_json::to_string(&manifest)?;
        
        // TODO: Store in NGFS
        tracing::info!("Created sensor snapshot {} with {} samples at {}", 
                      snapshot_id, sample_count, out_ngfs_path);
        
        Ok(snapshot_id)
    }
}

// Implement Clone for MockSensorRuntime
impl Clone for MockSensorRuntime {
    fn clone(&self) -> Self {
        Self {
            registered_sensors: self.registered_sensors.clone(),
            active_sampling: self.active_sampling.clone(),
            active_previews: self.active_previews.clone(),
            deterministic_seed: self.deterministic_seed,
        }
    }
}

/// Sensor snapshot manifest
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SensorSnapshotManifest {
    snapshot_id: String,
    sensor_id: String,
    sensor_info: SensorInfo,
    sample_count: usize,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
    samples: Vec<SensorSample>,
    deterministic: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_sensor_runtime_creation() {
        let runtime = MockSensorRuntime::new();
        assert!(runtime.is_ok());
    }
    
    #[tokio::test]
    async fn test_sensor_registration() {
        let runtime = MockSensorRuntime::new().unwrap();
        
        let desc = SensorDesc {
            name: "Test Accelerometer".to_string(),
            kind: SensorKind::Accelerometer,
            location: "IMU".to_string(),
            unit: "m/s²".to_string(),
            range_min: -20.0,
            range_max: 20.0,
            resolution: 0.01,
            sample_rates: vec![10, 50, 100, 200],
        };
        
        let sensor_id = runtime.register_sensor(desc).await.unwrap();
        assert!(!sensor_id.is_empty());
        
        let sensors = runtime.list_sensors().await.unwrap();
        assert_eq!(sensors.len(), 1);
        assert_eq!(sensors[0].name, "Test Accelerometer");
    }
    
    #[tokio::test]
    async fn test_sensor_sampling() {
        let runtime = MockSensorRuntime::new().unwrap();
        
        // Register sensor
        let desc = SensorDesc {
            name: "Test Sensor".to_string(),
            kind: SensorKind::Temperature,
            location: "CPU".to_string(),
            unit: "°C".to_string(),
            range_min: 0.0,
            range_max: 100.0,
            resolution: 0.1,
            sample_rates: vec![1, 10, 100],
        };
        
        let sensor_id = runtime.register_sensor(desc).await.unwrap();
        
        // Start sampling
        let handle = runtime.start_sampling("test_session", "device:sensor.sample", &sensor_id, 10, 12345).await.unwrap();
        assert!(!handle.is_empty());
        
        // Wait a bit
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Stop sampling
        runtime.stop_sampling(&handle).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_sensor_preview() {
        let runtime = MockSensorRuntime::new().unwrap();
        
        // Register sensor
        let desc = SensorDesc {
            name: "Test Sensor".to_string(),
            kind: SensorKind::Light,
            location: "Ambient".to_string(),
            unit: "lux".to_string(),
            range_min: 0.0,
            range_max: 1000.0,
            resolution: 1.0,
            sample_rates: vec![1, 5, 10],
        };
        
        let sensor_id = runtime.register_sensor(desc).await.unwrap();
        
        // Start preview
        let stream_handle = runtime.preview(&sensor_id, 5).await.unwrap();
        assert!(!stream_handle.is_empty());
        
        // Wait a bit
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    
    #[tokio::test]
    async fn test_sensor_snapshot() {
        let runtime = MockSensorRuntime::new().unwrap();
        
        // Register sensor
        let desc = SensorDesc {
            name: "Test Sensor".to_string(),
            kind: SensorKind::Gyroscope,
            location: "IMU".to_string(),
            unit: "deg/s".to_string(),
            range_min: -1000.0,
            range_max: 1000.0,
            resolution: 0.1,
            sample_rates: vec![10, 100, 1000],
        };
        
        let sensor_id = runtime.register_sensor(desc).await.unwrap();
        
        // Create snapshot
        let snapshot_id = runtime.snapshot(&sensor_id, "/tmp/test.ngfs").await.unwrap();
        assert!(!snapshot_id.is_empty());
    }
    
    #[tokio::test]
    async fn test_deterministic_sampling() {
        let mut runtime = MockSensorRuntime::new().unwrap();
        runtime.set_deterministic_seed(54321);
        
        // Register sensor
        let desc = SensorDesc {
            name: "Test Sensor".to_string(),
            kind: SensorKind::Accelerometer,
            location: "IMU".to_string(),
            unit: "m/s²".to_string(),
            range_min: -20.0,
            range_max: 20.0,
            resolution: 0.01,
            sample_rates: vec![100],
        };
        
        let sensor_id = runtime.register_sensor(desc).await.unwrap();
        
        // Generate two samples with same parameters
        let sample1 = runtime.generate_deterministic_sample(&sensor_id, 0, &SensorKind::Accelerometer);
        let sample2 = runtime.generate_deterministic_sample(&sensor_id, 0, &SensorKind::Accelerometer);
        
        // Should be identical
        assert_eq!(sample1.value, sample2.value);
    }
}
