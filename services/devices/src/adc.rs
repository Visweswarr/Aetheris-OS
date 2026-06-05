//! ADC Device Runtime
//!
//! This module provides capability-gated ADC operations with:
//! - Analog channel sampling with calibration
//! - Deterministic sampling with configurable rates
//! - NGFS snapshot integration for sample persistence
//! - Capability-based access control with DAO policy enforcement
//! - Audit logging and compliance tracking

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::error::DeviceError;
use crate::policy::DevicePolicyManager;

/// ADC channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdcSample {
    pub channel: u32,
    pub raw_value: u32,
    pub voltage: f32,
    pub timestamp: DateTime<Utc>,
    pub sequence_number: u64,
    pub deterministic: bool,
}

/// ADC sampling session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdcSamplingSession {
    pub session_id: String,
    pub channels: Vec<u32>,
    pub sample_rate_hz: u32,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub sample_count: u64,
    pub deterministic: bool,
    pub seed: Option<u64>,
}

/// ADC operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdcOperationResult {
    pub operation_id: String,
    pub operation: AdcOperation,
    pub result: AdcOperationResultType,
    pub timestamp: DateTime<Utc>,
    pub deterministic: bool,
}

/// ADC operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdcOperation {
    SingleSample { channel: u32 },
    StartSampling { session_id: String, channels: Vec<u32>, rate_hz: u32 },
    StopSampling { session_id: String },
    ConfigureChannel { config: AdcChannelConfig },
}

/// ADC operation result types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdcOperationResultType {
    Sample(AdcSample),
    SessionStarted(String),
    SessionStopped(String),
    ChannelConfigured(u32),
    Error(String),
}

/// ADC manager for handling analog-to-digital conversion
pub struct AdcManager {
    policy_manager: Arc<DevicePolicyManager>,
    channel_configs: Arc<RwLock<HashMap<u32, AdcChannelConfig>>>,
    active_sessions: Arc<RwLock<HashMap<String, AdcSamplingSession>>>,
    sample_buffer: Arc<RwLock<Vec<AdcSample>>>,
    operation_history: Arc<RwLock<Vec<AdcOperationResult>>>,
    deterministic_mode: bool,
    tick_counter: Arc<RwLock<u64>>,
    sequence_counter: Arc<RwLock<u64>>,
}

impl AdcManager {
    pub fn new(policy_manager: Arc<DevicePolicyManager>) -> Result<Self, DeviceError> {
        Ok(Self {
            policy_manager,
            channel_configs: Arc::new(RwLock::new(HashMap::new())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            sample_buffer: Arc::new(RwLock::new(Vec::new())),
            operation_history: Arc::new(RwLock::new(Vec::new())),
            deterministic_mode: false,
            tick_counter: Arc::new(RwLock::new(0)),
            sequence_counter: Arc::new(RwLock::new(0)),
        })
    }

    /// Configure an ADC channel
    pub async fn configure_channel(
        &self,
        session_id: &str,
        caps: &str,
        config: AdcChannelConfig,
    ) -> Result<(), DeviceError> {
        // Validate capabilities
        self.policy_manager.validate_capability(session_id, caps, "device:adc.configure").await?;
        
        // Check DAO policy
        self.policy_manager.check_dao_policy(session_id, "adc_configure").await?;

        let operation_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // Validate configuration
        if config.resolution_bits > 24 {
            return Err(DeviceError::InvalidConfiguration("ADC resolution cannot exceed 24 bits".to_string()));
        }

        if config.sample_rate_hz == 0 || config.sample_rate_hz > 1_000_000 {
            return Err(DeviceError::InvalidConfiguration("ADC sample rate must be between 1 and 1,000,000 Hz".to_string()));
        }

        // Store channel configuration
        self.channel_configs.write().await.insert(config.channel, config.clone());

        // Record operation
        let operation_result = AdcOperationResult {
            operation_id,
            operation: AdcOperation::ConfigureChannel { config: config.clone() },
            result: AdcOperationResultType::ChannelConfigured(config.channel),
            timestamp,
            deterministic: self.deterministic_mode,
        };

        self.operation_history.write().await.push(operation_result);

        // Emit audit event
        self.emit_audit_event("adc_configure", &config.channel.to_string(), session_id).await?;

        Ok(())
    }

    /// Take a single ADC sample
    pub async fn sample_channel(
        &self,
        session_id: &str,
        caps: &str,
        channel: u32,
    ) -> Result<AdcSample, DeviceError> {
        // Validate capabilities
        self.policy_manager.validate_capability(session_id, caps, "device:adc.sample").await?;
        
        // Check DAO policy
        self.policy_manager.check_dao_policy(session_id, "adc_sample").await?;

        let operation_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // Get channel configuration
        let channel_config = {
            let configs = self.channel_configs.read().await;
            configs.get(&channel)
                .cloned()
                .ok_or_else(|| DeviceError::ChannelNotConfigured(channel))?
        };

        if !channel_config.enabled {
            return Err(DeviceError::ChannelDisabled(channel));
        }

        // Generate sample
        let sample = self.generate_sample(channel, &channel_config, timestamp).await;

        // Store sample in buffer
        self.sample_buffer.write().await.push(sample.clone());

        // Record operation
        let operation_result = AdcOperationResult {
            operation_id,
            operation: AdcOperation::SingleSample { channel },
            result: AdcOperationResultType::Sample(sample.clone()),
            timestamp,
            deterministic: self.deterministic_mode,
        };

        self.operation_history.write().await.push(operation_result);

        // Emit audit event
        self.emit_audit_event("adc_sample", &channel.to_string(), session_id).await?;

        Ok(sample)
    }

    /// Start continuous ADC sampling
    pub async fn start_sampling(
        &self,
        session_id: &str,
        caps: &str,
        channels: Vec<u32>,
        rate_hz: u32,
        seed: Option<u64>,
    ) -> Result<String, DeviceError> {
        // Validate capabilities
        self.policy_manager.validate_capability(session_id, caps, "device:adc.sample").await?;
        
        // Check DAO policy
        self.policy_manager.check_dao_policy(session_id, "adc_sampling").await?;

        let operation_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // Validate channels are configured
        let configs = self.channel_configs.read().await;
        for &channel in &channels {
            if !configs.contains_key(&channel) {
                return Err(DeviceError::ChannelNotConfigured(channel));
            }
        }

        // Create sampling session
        let sampling_session = AdcSamplingSession {
            session_id: session_id.to_string(),
            channels: channels.clone(),
            sample_rate_hz: rate_hz,
            start_time: timestamp,
            end_time: None,
            sample_count: 0,
            deterministic: self.deterministic_mode || seed.is_some(),
            seed,
        };

        // Store session
        self.active_sessions.write().await.insert(session_id.to_string(), sampling_session);

        // Record operation
        let operation_result = AdcOperationResult {
            operation_id,
            operation: AdcOperation::StartSampling { 
                session_id: session_id.to_string(), 
                channels: channels.clone(), 
                rate_hz 
            },
            result: AdcOperationResultType::SessionStarted(session_id.to_string()),
            timestamp,
            deterministic: self.deterministic_mode,
        };

        self.operation_history.write().await.push(operation_result);

        // Emit audit event
        self.emit_audit_event("adc_sampling_started", session_id, session_id).await?;

        Ok(session_id.to_string())
    }

    /// Stop continuous ADC sampling
    pub async fn stop_sampling(
        &self,
        session_id: &str,
        caps: &str,
    ) -> Result<AdcSamplingSession, DeviceError> {
        // Validate capabilities
        self.policy_manager.validate_capability(session_id, caps, "device:adc.sample").await?;
        
        // Check DAO policy
        self.policy_manager.check_dao_policy(session_id, "adc_sampling").await?;

        let operation_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // Get and update session
        let mut sessions = self.active_sessions.write().await;
        let mut session = sessions.remove(session_id)
            .ok_or_else(|| DeviceError::SessionNotFound(session_id.to_string()))?;

        session.end_time = Some(timestamp);

        // Record operation
        let operation_result = AdcOperationResult {
            operation_id,
            operation: AdcOperation::StopSampling { session_id: session_id.to_string() },
            result: AdcOperationResultType::SessionStopped(session_id.to_string()),
            timestamp,
            deterministic: self.deterministic_mode,
        };

        self.operation_history.write().await.push(operation_result);

        // Emit audit event
        self.emit_audit_event("adc_sampling_stopped", session_id, session_id).await?;

        Ok(session)
    }

    /// Get samples from buffer
    pub async fn get_samples(&self, limit: Option<usize>) -> Vec<AdcSample> {
        let buffer = self.sample_buffer.read().await;
        if let Some(limit) = limit {
            buffer.iter().rev().take(limit).cloned().collect()
        } else {
            buffer.clone()
        }
    }

    /// Get samples for a specific channel
    pub async fn get_channel_samples(&self, channel: u32, limit: Option<usize>) -> Vec<AdcSample> {
        let buffer = self.sample_buffer.read().await;
        let channel_samples: Vec<AdcSample> = buffer
            .iter()
            .rev()
            .filter(|sample| sample.channel == channel)
            .take(limit.unwrap_or(usize::MAX))
            .cloned()
            .collect();
        channel_samples
    }

    /// Get active sampling sessions
    pub async fn get_active_sessions(&self) -> Vec<AdcSamplingSession> {
        let sessions = self.active_sessions.read().await;
        sessions.values().cloned().collect()
    }

    /// Get channel configuration
    pub async fn get_channel_config(&self, channel: u32) -> Result<AdcChannelConfig, DeviceError> {
        let configs = self.channel_configs.read().await;
        configs.get(&channel)
            .cloned()
            .ok_or_else(|| DeviceError::ChannelNotConfigured(channel))
    }

    /// Get all configured channels
    pub async fn get_configured_channels(&self) -> Vec<AdcChannelConfig> {
        let configs = self.channel_configs.read().await;
        configs.values().cloned().collect()
    }

    /// Get operation history
    pub async fn get_operation_history(&self, limit: Option<usize>) -> Vec<AdcOperationResult> {
        let history = self.operation_history.read().await;
        if let Some(limit) = limit {
            history.iter().rev().take(limit).cloned().collect()
        } else {
            history.clone()
        }
    }

    /// Enable deterministic mode
    pub async fn enable_deterministic_mode(&self) {
        self.deterministic_mode = true;
    }

    /// Disable deterministic mode
    pub async fn disable_deterministic_mode(&self) {
        self.deterministic_mode = false;
    }

    /// Advance tick counter (for deterministic operations)
    pub async fn advance_tick(&self) {
        let mut counter = self.tick_counter.write().await;
        *counter += 1;
    }

    /// Get current tick counter
    pub async fn get_tick_counter(&self) -> u64 {
        let counter = self.tick_counter.read().await;
        *counter
    }

    /// Generate ADC sample
    async fn generate_sample(
        &self,
        channel: u32,
        config: &AdcChannelConfig,
        timestamp: DateTime<Utc>,
    ) -> AdcSample {
        let mut sequence_counter = self.sequence_counter.write().await;
        *sequence_counter += 1;
        let sequence_number = *sequence_counter;
        drop(sequence_counter);

        // Generate raw value
        let raw_value = if self.deterministic_mode {
            self.generate_deterministic_raw_value(channel, config, timestamp)
        } else {
            // Simulate random ADC reading
            let time_ms = timestamp.timestamp_millis() as u32;
            (channel + time_ms) % (1u32 << config.resolution_bits)
        };

        // Convert to voltage
        let max_value = (1u32 << config.resolution_bits) - 1;
        let voltage = (raw_value as f32 / max_value as f32) * config.reference_voltage;
        
        // Apply calibration
        let calibrated_voltage = (voltage + config.calibration_offset) * config.calibration_scale;

        AdcSample {
            channel,
            raw_value,
            voltage: calibrated_voltage,
            timestamp,
            sequence_number,
            deterministic: self.deterministic_mode,
        }
    }

    /// Generate deterministic raw value
    fn generate_deterministic_raw_value(
        &self,
        channel: u32,
        config: &AdcChannelConfig,
        timestamp: DateTime<Utc>,
    ) -> u32 {
        // Simple deterministic pattern based on channel, time, and configuration
        let time_ms = timestamp.timestamp_millis() as u32;
        let max_value = (1u32 << config.resolution_bits) - 1;
        
        // Create a sine wave pattern that varies by channel and time
        let phase = (channel as f32 * 0.1) + (time_ms as f32 * 0.001);
        let sine_value = (phase.sin() + 1.0) / 2.0; // Normalize to 0-1
        
        (sine_value * max_value as f32) as u32
    }

    /// Create NGFS snapshot of ADC state
    pub async fn create_snapshot(&self) -> Result<Vec<u8>, DeviceError> {
        let channel_configs = self.channel_configs.read().await;
        let active_sessions = self.active_sessions.read().await;
        let sample_buffer = self.sample_buffer.read().await;
        let operation_history = self.operation_history.read().await;
        
        let snapshot = AdcSnapshot {
            channel_configs: channel_configs.values().cloned().collect(),
            active_sessions: active_sessions.values().cloned().collect(),
            recent_samples: sample_buffer.iter().rev().take(1000).cloned().collect(),
            operation_history: operation_history.clone(),
            timestamp: Utc::now(),
            deterministic_mode: self.deterministic_mode,
            tick_counter: *self.tick_counter.read().await,
            sequence_counter: *self.sequence_counter.read().await,
        };

        // Serialize to CBOR for deterministic byte representation
        let mut buffer = Vec::new();
        ciborium::ser::into_writer(&snapshot, &mut buffer)
            .map_err(|e| DeviceError::SerializationError(e.to_string()))?;

        Ok(buffer)
    }

    /// Restore ADC state from NGFS snapshot
    pub async fn restore_snapshot(&self, data: &[u8]) -> Result<(), DeviceError> {
        let snapshot: AdcSnapshot = ciborium::de::from_reader(data)
            .map_err(|e| DeviceError::DeserializationError(e.to_string()))?;

        // Restore channel configurations
        let mut channel_configs = self.channel_configs.write().await;
        channel_configs.clear();
        for config in snapshot.channel_configs {
            channel_configs.insert(config.channel, config);
        }

        // Restore active sessions
        let mut active_sessions = self.active_sessions.write().await;
        active_sessions.clear();
        for session in snapshot.active_sessions {
            active_sessions.insert(session.session_id.clone(), session);
        }

        // Restore sample buffer
        let mut sample_buffer = self.sample_buffer.write().await;
        *sample_buffer = snapshot.recent_samples;

        // Restore operation history
        let mut operation_history = self.operation_history.write().await;
        *operation_history = snapshot.operation_history;

        // Restore deterministic mode and counters
        self.deterministic_mode = snapshot.deterministic_mode;
        let mut tick_counter = self.tick_counter.write().await;
        *tick_counter = snapshot.tick_counter;
        let mut sequence_counter = self.sequence_counter.write().await;
        *sequence_counter = snapshot.sequence_counter;

        Ok(())
    }

    async fn emit_audit_event(
        &self,
        event_type: &str,
        channel: &str,
        session_id: &str,
    ) -> Result<(), DeviceError> {
        tracing::info!(
            "ADC audit event: {} for channel {} in session {}",
            event_type,
            channel,
            session_id
        );
        Ok(())
    }
}

/// ADC snapshot for NGFS persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdcSnapshot {
    channel_configs: Vec<AdcChannelConfig>,
    active_sessions: Vec<AdcSamplingSession>,
    recent_samples: Vec<AdcSample>,
    operation_history: Vec<AdcOperationResult>,
    timestamp: DateTime<Utc>,
    deterministic_mode: bool,
    tick_counter: u64,
    sequence_counter: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::DevicePolicyManager;

    #[tokio::test]
    async fn test_adc_manager_creation() {
        let policy_manager = Arc::new(DevicePolicyManager::new().unwrap());
        let adc_manager = AdcManager::new(policy_manager);
        assert!(adc_manager.is_ok());
    }

    #[tokio::test]
    async fn test_channel_configuration() {
        let policy_manager = Arc::new(DevicePolicyManager::new().unwrap());
        let adc_manager = AdcManager::new(policy_manager).unwrap();

        let config = AdcChannelConfig {
            channel: 0,
            reference_voltage: 3.3,
            resolution_bits: 12,
            sample_rate_hz: 1000,
            calibration_offset: 0.0,
            calibration_scale: 1.0,
            enabled: true,
        };

        let result = adc_manager.configure_channel("test_session", "device:adc.configure", config).await;
        assert!(result.is_ok());

        let retrieved_config = adc_manager.get_channel_config(0).await.unwrap();
        assert_eq!(retrieved_config.channel, 0);
        assert_eq!(retrieved_config.reference_voltage, 3.3);
        assert_eq!(retrieved_config.resolution_bits, 12);
    }

    #[tokio::test]
    async fn test_single_sample() {
        let policy_manager = Arc::new(DevicePolicyManager::new().unwrap());
        let adc_manager = AdcManager::new(policy_manager).unwrap();

        // Configure channel
        let config = AdcChannelConfig {
            channel: 0,
            reference_voltage: 3.3,
            resolution_bits: 12,
            sample_rate_hz: 1000,
            calibration_offset: 0.0,
            calibration_scale: 1.0,
            enabled: true,
        };
        adc_manager.configure_channel("test_session", "device:adc.configure", config).await.unwrap();

        // Take sample
        let sample = adc_manager.sample_channel("test_session", "device:adc.sample", 0).await.unwrap();
        assert_eq!(sample.channel, 0);
        assert!(sample.voltage >= 0.0);
        assert!(sample.voltage <= 3.3);
        assert!(sample.sequence_number > 0);
    }

    #[tokio::test]
    async fn test_sampling_session() {
        let policy_manager = Arc::new(DevicePolicyManager::new().unwrap());
        let adc_manager = AdcManager::new(policy_manager).unwrap();

        // Configure channels
        let config = AdcChannelConfig {
            channel: 0,
            reference_voltage: 3.3,
            resolution_bits: 12,
            sample_rate_hz: 1000,
            calibration_offset: 0.0,
            calibration_scale: 1.0,
            enabled: true,
        };
        adc_manager.configure_channel("test_session", "device:adc.configure", config).await.unwrap();

        // Start sampling
        let session_id = adc_manager.start_sampling(
            "test_session", 
            "device:adc.sample", 
            vec![0], 
            100, 
            Some(12345)
        ).await.unwrap();

        // Check active sessions
        let sessions = adc_manager.get_active_sessions().await;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, "test_session");

        // Stop sampling
        let session = adc_manager.stop_sampling("test_session", "device:adc.sample").await.unwrap();
        assert_eq!(session.session_id, "test_session");
        assert!(session.end_time.is_some());
    }

    #[tokio::test]
    async fn test_deterministic_mode() {
        let policy_manager = Arc::new(DevicePolicyManager::new().unwrap());
        let adc_manager = AdcManager::new(policy_manager).unwrap();

        adc_manager.enable_deterministic_mode().await;

        let config = AdcChannelConfig {
            channel: 0,
            reference_voltage: 3.3,
            resolution_bits: 12,
            sample_rate_hz: 1000,
            calibration_offset: 0.0,
            calibration_scale: 1.0,
            enabled: true,
        };
        adc_manager.configure_channel("test_session", "device:adc.configure", config).await.unwrap();

        // Take multiple samples - should be deterministic
        let sample1 = adc_manager.sample_channel("test_session", "device:adc.sample", 0).await.unwrap();
        let sample2 = adc_manager.sample_channel("test_session", "device:adc.sample", 0).await.unwrap();
        
        // In deterministic mode, values should be consistent
        assert_eq!(sample1.raw_value, sample2.raw_value);
        assert_eq!(sample1.voltage, sample2.voltage);
    }

    #[tokio::test]
    async fn test_snapshot_restore() {
        let policy_manager = Arc::new(DevicePolicyManager::new().unwrap());
        let adc_manager = AdcManager::new(policy_manager).unwrap();

        // Configure channel
        let config = AdcChannelConfig {
            channel: 0,
            reference_voltage: 3.3,
            resolution_bits: 12,
            sample_rate_hz: 1000,
            calibration_offset: 0.1,
            calibration_scale: 1.05,
            enabled: true,
        };
        adc_manager.configure_channel("test_session", "device:adc.configure", config).await.unwrap();

        // Take some samples
        adc_manager.sample_channel("test_session", "device:adc.sample", 0).await.unwrap();
        adc_manager.sample_channel("test_session", "device:adc.sample", 0).await.unwrap();

        // Create snapshot
        let snapshot_data = adc_manager.create_snapshot().await.unwrap();

        // Create new manager and restore
        let policy_manager2 = Arc::new(DevicePolicyManager::new().unwrap());
        let adc_manager2 = AdcManager::new(policy_manager2).unwrap();
        adc_manager2.restore_snapshot(&snapshot_data).await.unwrap();

        // Verify restored state
        let restored_config = adc_manager2.get_channel_config(0).await.unwrap();
        assert_eq!(restored_config.channel, 0);
        assert_eq!(restored_config.reference_voltage, 3.3);
        assert_eq!(restored_config.calibration_offset, 0.1);
        assert_eq!(restored_config.calibration_scale, 1.05);

        let samples = adc_manager2.get_samples(Some(2)).await;
        assert_eq!(samples.len(), 2);
    }
}
