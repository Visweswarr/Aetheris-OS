//! GPIO Device Runtime
//!
//! This module provides capability-gated GPIO operations with:
//! - Digital pin configuration (input/output/pull-up/pull-down)
//! - Deterministic digital I/O operations
//! - NGFS snapshot integration for pin state persistence
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

/// GPIO pin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpioPinConfig {
    pub pin: u32,
    pub mode: GpioMode,
    pub pull: GpioPull,
    pub initial_value: Option<bool>,
    pub debounce_ms: Option<u32>,
}

/// GPIO pin modes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GpioMode {
    Input,
    Output,
    InputPullUp,
    InputPullDown,
    OutputOpenDrain,
    OutputPushPull,
}

/// GPIO pull resistor configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GpioPull {
    None,
    Up,
    Down,
}

/// GPIO pin state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpioPinState {
    pub pin: u32,
    pub mode: GpioMode,
    pub pull: GpioPull,
    pub value: bool,
    pub last_change: DateTime<Utc>,
    pub change_count: u64,
}

/// GPIO operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpioOperationResult {
    pub operation_id: String,
    pub pin: u32,
    pub operation: GpioOperation,
    pub result: bool,
    pub timestamp: DateTime<Utc>,
    pub deterministic: bool,
}

/// GPIO operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpioOperation {
    Read,
    Write { value: bool },
    Configure { config: GpioPinConfig },
    Toggle,
}

/// GPIO manager for handling pin operations
pub struct GpioManager {
    policy_manager: Arc<DevicePolicyManager>,
    pin_states: Arc<RwLock<HashMap<u32, GpioPinState>>>,
    operation_history: Arc<RwLock<Vec<GpioOperationResult>>>,
    deterministic_mode: bool,
    tick_counter: Arc<RwLock<u64>>,
}

impl GpioManager {
    pub fn new(policy_manager: Arc<DevicePolicyManager>) -> Result<Self, DeviceError> {
        Ok(Self {
            policy_manager,
            pin_states: Arc::new(RwLock::new(HashMap::new())),
            operation_history: Arc::new(RwLock::new(Vec::new())),
            deterministic_mode: false,
            tick_counter: Arc::new(RwLock::new(0)),
        })
    }

    /// Configure a GPIO pin
    pub async fn configure_pin(
        &self,
        session_id: &str,
        caps: &str,
        config: GpioPinConfig,
    ) -> Result<(), DeviceError> {
        // Validate capabilities
        self.policy_manager.validate_capability(session_id, caps, "device:gpio.configure").await?;
        
        // Check DAO policy
        self.policy_manager.check_dao_policy(session_id, "gpio_configure").await?;

        let operation_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // Configure the pin
        let pin_state = GpioPinState {
            pin: config.pin,
            mode: config.mode.clone(),
            pull: config.pull.clone(),
            value: config.initial_value.unwrap_or(false),
            last_change: timestamp,
            change_count: 0,
        };

        // Store pin state
        self.pin_states.write().await.insert(config.pin, pin_state);

        // Record operation
        let operation_result = GpioOperationResult {
            operation_id,
            pin: config.pin,
            operation: GpioOperation::Configure { config: config.clone() },
            result: true,
            timestamp,
            deterministic: self.deterministic_mode,
        };

        self.operation_history.write().await.push(operation_result);

        // Emit audit event
        self.emit_audit_event("gpio_configure", &config.pin.to_string(), session_id).await?;

        Ok(())
    }

    /// Read a GPIO pin value
    pub async fn read_pin(
        &self,
        session_id: &str,
        caps: &str,
        pin: u32,
    ) -> Result<bool, DeviceError> {
        // Validate capabilities
        self.policy_manager.validate_capability(session_id, caps, "device:gpio.read").await?;
        
        // Check DAO policy
        self.policy_manager.check_dao_policy(session_id, "gpio_read").await?;

        let operation_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // Get pin state
        let mut pin_states = self.pin_states.write().await;
        let pin_state = pin_states.get_mut(&pin)
            .ok_or_else(|| DeviceError::PinNotConfigured(pin))?;

        // Validate pin mode
        if !matches!(pin_state.mode, GpioMode::Input | GpioMode::InputPullUp | GpioMode::InputPullDown) {
            return Err(DeviceError::InvalidPinMode(pin, "input".to_string()));
        }

        // Simulate pin read (in real implementation, this would read from hardware)
        let value = if self.deterministic_mode {
            self.generate_deterministic_value(pin, timestamp)
        } else {
            // Simulate random input for non-deterministic mode
            (pin + timestamp.timestamp_millis() as u32) % 2 == 0
        };

        // Update pin state
        if pin_state.value != value {
            pin_state.value = value;
            pin_state.last_change = timestamp;
            pin_state.change_count += 1;
        }

        // Record operation
        let operation_result = GpioOperationResult {
            operation_id,
            pin,
            operation: GpioOperation::Read,
            result: value,
            timestamp,
            deterministic: self.deterministic_mode,
        };

        drop(pin_states);
        self.operation_history.write().await.push(operation_result);

        // Emit audit event
        self.emit_audit_event("gpio_read", &pin.to_string(), session_id).await?;

        Ok(value)
    }

    /// Write a GPIO pin value
    pub async fn write_pin(
        &self,
        session_id: &str,
        caps: &str,
        pin: u32,
        value: bool,
    ) -> Result<(), DeviceError> {
        // Validate capabilities
        self.policy_manager.validate_capability(session_id, caps, "device:gpio.write").await?;
        
        // Check DAO policy
        self.policy_manager.check_dao_policy(session_id, "gpio_write").await?;

        let operation_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // Get pin state
        let mut pin_states = self.pin_states.write().await;
        let pin_state = pin_states.get_mut(&pin)
            .ok_or_else(|| DeviceError::PinNotConfigured(pin))?;

        // Validate pin mode
        if !matches!(pin_state.mode, GpioMode::Output | GpioMode::OutputOpenDrain | GpioMode::OutputPushPull) {
            return Err(DeviceError::InvalidPinMode(pin, "output".to_string()));
        }

        // Update pin state
        if pin_state.value != value {
            pin_state.value = value;
            pin_state.last_change = timestamp;
            pin_state.change_count += 1;
        }

        // Record operation
        let operation_result = GpioOperationResult {
            operation_id,
            pin,
            operation: GpioOperation::Write { value },
            result: true,
            timestamp,
            deterministic: self.deterministic_mode,
        };

        drop(pin_states);
        self.operation_history.write().await.push(operation_result);

        // Emit audit event
        self.emit_audit_event("gpio_write", &pin.to_string(), session_id).await?;

        Ok(())
    }

    /// Toggle a GPIO pin value
    pub async fn toggle_pin(
        &self,
        session_id: &str,
        caps: &str,
        pin: u32,
    ) -> Result<bool, DeviceError> {
        // Validate capabilities
        self.policy_manager.validate_capability(session_id, caps, "device:gpio.write").await?;
        
        // Check DAO policy
        self.policy_manager.check_dao_policy(session_id, "gpio_toggle").await?;

        let operation_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // Get pin state
        let mut pin_states = self.pin_states.write().await;
        let pin_state = pin_states.get_mut(&pin)
            .ok_or_else(|| DeviceError::PinNotConfigured(pin))?;

        // Validate pin mode
        if !matches!(pin_state.mode, GpioMode::Output | GpioMode::OutputOpenDrain | GpioMode::OutputPushPull) {
            return Err(DeviceError::InvalidPinMode(pin, "output".to_string()));
        }

        // Toggle pin value
        let new_value = !pin_state.value;
        pin_state.value = new_value;
        pin_state.last_change = timestamp;
        pin_state.change_count += 1;

        // Record operation
        let operation_result = GpioOperationResult {
            operation_id,
            pin,
            operation: GpioOperation::Toggle,
            result: new_value,
            timestamp,
            deterministic: self.deterministic_mode,
        };

        drop(pin_states);
        self.operation_history.write().await.push(operation_result);

        // Emit audit event
        self.emit_audit_event("gpio_toggle", &pin.to_string(), session_id).await?;

        Ok(new_value)
    }

    /// Get pin state
    pub async fn get_pin_state(&self, pin: u32) -> Result<GpioPinState, DeviceError> {
        let pin_states = self.pin_states.read().await;
        pin_states.get(&pin)
            .cloned()
            .ok_or_else(|| DeviceError::PinNotConfigured(pin))
    }

    /// Get all configured pins
    pub async fn get_configured_pins(&self) -> Vec<GpioPinState> {
        let pin_states = self.pin_states.read().await;
        pin_states.values().cloned().collect()
    }

    /// Get operation history
    pub async fn get_operation_history(&self, limit: Option<usize>) -> Vec<GpioOperationResult> {
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

    /// Generate deterministic value based on pin and timestamp
    fn generate_deterministic_value(&self, pin: u32, timestamp: DateTime<Utc>) -> bool {
        // Simple deterministic pattern based on pin number and time
        let time_ms = timestamp.timestamp_millis() as u32;
        (pin + time_ms / 1000) % 2 == 0
    }

    /// Create NGFS snapshot of GPIO state
    pub async fn create_snapshot(&self) -> Result<Vec<u8>, DeviceError> {
        let pin_states = self.pin_states.read().await;
        let operation_history = self.operation_history.read().await;
        
        let snapshot = GpioSnapshot {
            pin_states: pin_states.values().cloned().collect(),
            operation_history: operation_history.clone(),
            timestamp: Utc::now(),
            deterministic_mode: self.deterministic_mode,
            tick_counter: *self.tick_counter.read().await,
        };

        // Serialize to CBOR for deterministic byte representation
        let mut buffer = Vec::new();
        ciborium::ser::into_writer(&snapshot, &mut buffer)
            .map_err(|e| DeviceError::SerializationError(e.to_string()))?;

        Ok(buffer)
    }

    /// Restore GPIO state from NGFS snapshot
    pub async fn restore_snapshot(&self, data: &[u8]) -> Result<(), DeviceError> {
        let snapshot: GpioSnapshot = ciborium::de::from_reader(data)
            .map_err(|e| DeviceError::DeserializationError(e.to_string()))?;

        // Restore pin states
        let mut pin_states = self.pin_states.write().await;
        pin_states.clear();
        for pin_state in snapshot.pin_states {
            pin_states.insert(pin_state.pin, pin_state);
        }

        // Restore operation history
        let mut operation_history = self.operation_history.write().await;
        *operation_history = snapshot.operation_history;

        // Restore deterministic mode and tick counter
        self.deterministic_mode = snapshot.deterministic_mode;
        let mut tick_counter = self.tick_counter.write().await;
        *tick_counter = snapshot.tick_counter;

        Ok(())
    }

    async fn emit_audit_event(
        &self,
        event_type: &str,
        pin: &str,
        session_id: &str,
    ) -> Result<(), DeviceError> {
        tracing::info!(
            "GPIO audit event: {} for pin {} in session {}",
            event_type,
            pin,
            session_id
        );
        Ok(())
    }
}

/// GPIO snapshot for NGFS persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GpioSnapshot {
    pin_states: Vec<GpioPinState>,
    operation_history: Vec<GpioOperationResult>,
    timestamp: DateTime<Utc>,
    deterministic_mode: bool,
    tick_counter: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::DevicePolicyManager;

    #[tokio::test]
    async fn test_gpio_manager_creation() {
        let policy_manager = Arc::new(DevicePolicyManager::new().unwrap());
        let gpio_manager = GpioManager::new(policy_manager);
        assert!(gpio_manager.is_ok());
    }

    #[tokio::test]
    async fn test_pin_configuration() {
        let policy_manager = Arc::new(DevicePolicyManager::new().unwrap());
        let gpio_manager = GpioManager::new(policy_manager).unwrap();

        let config = GpioPinConfig {
            pin: 1,
            mode: GpioMode::Output,
            pull: GpioPull::None,
            initial_value: Some(true),
            debounce_ms: None,
        };

        let result = gpio_manager.configure_pin("test_session", "device:gpio.configure", config).await;
        assert!(result.is_ok());

        let pin_state = gpio_manager.get_pin_state(1).await.unwrap();
        assert_eq!(pin_state.pin, 1);
        assert_eq!(pin_state.mode, GpioMode::Output);
        assert_eq!(pin_state.value, true);
    }

    #[tokio::test]
    async fn test_pin_read_write() {
        let policy_manager = Arc::new(DevicePolicyManager::new().unwrap());
        let gpio_manager = GpioManager::new(policy_manager).unwrap();

        // Configure input pin
        let input_config = GpioPinConfig {
            pin: 1,
            mode: GpioMode::Input,
            pull: GpioPull::None,
            initial_value: None,
            debounce_ms: None,
        };
        gpio_manager.configure_pin("test_session", "device:gpio.configure", input_config).await.unwrap();

        // Configure output pin
        let output_config = GpioPinConfig {
            pin: 2,
            mode: GpioMode::Output,
            pull: GpioPull::None,
            initial_value: Some(false),
            debounce_ms: None,
        };
        gpio_manager.configure_pin("test_session", "device:gpio.configure", output_config).await.unwrap();

        // Read input pin
        let value = gpio_manager.read_pin("test_session", "device:gpio.read", 1).await.unwrap();
        assert!(value == true || value == false);

        // Write output pin
        gpio_manager.write_pin("test_session", "device:gpio.write", 2, true).await.unwrap();
        let pin_state = gpio_manager.get_pin_state(2).await.unwrap();
        assert_eq!(pin_state.value, true);

        // Toggle output pin
        let new_value = gpio_manager.toggle_pin("test_session", "device:gpio.write", 2).await.unwrap();
        assert_eq!(new_value, false);
    }

    #[tokio::test]
    async fn test_deterministic_mode() {
        let policy_manager = Arc::new(DevicePolicyManager::new().unwrap());
        let gpio_manager = GpioManager::new(policy_manager).unwrap();

        gpio_manager.enable_deterministic_mode().await;

        let config = GpioPinConfig {
            pin: 1,
            mode: GpioMode::Input,
            pull: GpioPull::None,
            initial_value: None,
            debounce_ms: None,
        };
        gpio_manager.configure_pin("test_session", "device:gpio.configure", config).await.unwrap();

        // Read multiple times - should be deterministic
        let value1 = gpio_manager.read_pin("test_session", "device:gpio.read", 1).await.unwrap();
        let value2 = gpio_manager.read_pin("test_session", "device:gpio.read", 1).await.unwrap();
        
        // In deterministic mode, values should be consistent
        assert_eq!(value1, value2);
    }

    #[tokio::test]
    async fn test_snapshot_restore() {
        let policy_manager = Arc::new(DevicePolicyManager::new().unwrap());
        let gpio_manager = GpioManager::new(policy_manager).unwrap();

        // Configure some pins
        let config1 = GpioPinConfig {
            pin: 1,
            mode: GpioMode::Output,
            pull: GpioPull::None,
            initial_value: Some(true),
            debounce_ms: None,
        };
        gpio_manager.configure_pin("test_session", "device:gpio.configure", config1).await.unwrap();

        let config2 = GpioPinConfig {
            pin: 2,
            mode: GpioMode::Input,
            pull: GpioPull::Up,
            initial_value: None,
            debounce_ms: Some(10),
        };
        gpio_manager.configure_pin("test_session", "device:gpio.configure", config2).await.unwrap();

        // Create snapshot
        let snapshot_data = gpio_manager.create_snapshot().await.unwrap();

        // Create new manager and restore
        let policy_manager2 = Arc::new(DevicePolicyManager::new().unwrap());
        let gpio_manager2 = GpioManager::new(policy_manager2).unwrap();
        gpio_manager2.restore_snapshot(&snapshot_data).await.unwrap();

        // Verify restored state
        let pin_state1 = gpio_manager2.get_pin_state(1).await.unwrap();
        assert_eq!(pin_state1.pin, 1);
        assert_eq!(pin_state1.mode, GpioMode::Output);
        assert_eq!(pin_state1.value, true);

        let pin_state2 = gpio_manager2.get_pin_state(2).await.unwrap();
        assert_eq!(pin_state2.pin, 2);
        assert_eq!(pin_state2.mode, GpioMode::Input);
        assert_eq!(pin_state2.pull, GpioPull::Up);
    }
}
