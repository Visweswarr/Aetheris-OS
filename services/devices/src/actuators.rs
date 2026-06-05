//! Actuator Device Runtime
//!
//! This module provides capability-gated actuator operations with:
//! - PWM output control for motors, servos, and LEDs
//! - Digital relay and switch control
//! - Motor control with direction and speed
//! - Deterministic actuation with configurable patterns
//! - NGFS snapshot integration for actuator state persistence
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

/// Actuator types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActuatorType {
    PWM,
    Relay,
    Motor,
    Servo,
    LED,
    Stepper,
}

/// Actuator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActuatorConfig {
    pub actuator_id: String,
    pub actuator_type: ActuatorType,
    pub pin: u32,
    pub frequency_hz: Option<u32>,
    pub min_value: f32,
    pub max_value: f32,
    pub initial_value: f32,
    pub enabled: bool,
    pub safety_limits: Option<SafetyLimits>,
}

/// Safety limits for actuators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyLimits {
    pub max_duty_cycle: f32,
    pub max_duration_ms: u32,
    pub cooldown_ms: u32,
    pub emergency_stop: bool,
}

/// Actuator state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActuatorState {
    pub actuator_id: String,
    pub actuator_type: ActuatorType,
    pub pin: u32,
    pub current_value: f32,
    pub target_value: f32,
    pub is_active: bool,
    pub last_update: DateTime<Utc>,
    pub operation_count: u64,
    pub total_runtime_ms: u64,
}

/// Actuator operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActuatorOperation {
    pub operation_id: String,
    pub actuator_id: String,
    pub operation_type: ActuatorOperationType,
    pub value: f32,
    pub duration_ms: Option<u32>,
    pub timestamp: DateTime<Utc>,
    pub deterministic: bool,
}

/// Actuator operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActuatorOperationType {
    SetValue,
    SetDutyCycle,
    SetSpeed,
    SetAngle,
    SetDirection,
    Start,
    Stop,
    EmergencyStop,
    Configure,
}

/// PWM pattern for deterministic operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PWMPattern {
    pub pattern_id: String,
    pub actuator_id: String,
    pub pattern_type: PatternType,
    pub duration_ms: u32,
    pub steps: Vec<PatternStep>,
    pub repeat: bool,
    pub deterministic: bool,
}

/// Pattern types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    Linear,
    Sine,
    Square,
    Triangle,
    Custom,
}

/// Pattern step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStep {
    pub value: f32,
    pub duration_ms: u32,
    pub transition_type: TransitionType,
}

/// Transition types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionType {
    Immediate,
    Linear,
    Smooth,
}

/// Actuator manager for handling various actuator types
pub struct ActuatorManager {
    policy_manager: Arc<DevicePolicyManager>,
    actuator_configs: Arc<RwLock<HashMap<String, ActuatorConfig>>>,
    actuator_states: Arc<RwLock<HashMap<String, ActuatorState>>>,
    active_patterns: Arc<RwLock<HashMap<String, PWMPattern>>>,
    operation_history: Arc<RwLock<Vec<ActuatorOperation>>>,
    deterministic_mode: bool,
    tick_counter: Arc<RwLock<u64>>,
    safety_monitor: Arc<RwLock<SafetyMonitor>>,
}

/// Safety monitor for tracking actuator usage
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SafetyMonitor {
    actuator_runtime: HashMap<String, u64>,
    last_emergency_stop: Option<DateTime<Utc>>,
    safety_violations: Vec<SafetyViolation>,
}

/// Safety violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SafetyViolation {
    actuator_id: String,
    violation_type: String,
    timestamp: DateTime<Utc>,
    details: String,
}

impl ActuatorManager {
    pub fn new(policy_manager: Arc<DevicePolicyManager>) -> Result<Self, DeviceError> {
        let safety_monitor = SafetyMonitor {
            actuator_runtime: HashMap::new(),
            last_emergency_stop: None,
            safety_violations: Vec::new(),
        };

        Ok(Self {
            policy_manager,
            actuator_configs: Arc::new(RwLock::new(HashMap::new())),
            actuator_states: Arc::new(RwLock::new(HashMap::new())),
            active_patterns: Arc::new(RwLock::new(HashMap::new())),
            operation_history: Arc::new(RwLock::new(Vec::new())),
            deterministic_mode: false,
            tick_counter: Arc::new(RwLock::new(0)),
            safety_monitor: Arc::new(RwLock::new(safety_monitor)),
        })
    }

    /// Configure an actuator
    pub async fn configure_actuator(
        &self,
        session_id: &str,
        caps: &str,
        config: ActuatorConfig,
    ) -> Result<(), DeviceError> {
        // Validate capabilities
        self.policy_manager.validate_capability(session_id, caps, "device:actuator.configure").await?;
        
        // Check DAO policy
        self.policy_manager.check_dao_policy(session_id, "actuator_configure").await?;

        let operation_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // Validate configuration
        if config.min_value >= config.max_value {
            return Err(DeviceError::InvalidConfiguration("min_value must be less than max_value".to_string()));
        }

        if config.initial_value < config.min_value || config.initial_value > config.max_value {
            return Err(DeviceError::InvalidConfiguration("initial_value must be within min/max range".to_string()));
        }

        // Store configuration
        self.actuator_configs.write().await.insert(config.actuator_id.clone(), config.clone());

        // Initialize actuator state
        let state = ActuatorState {
            actuator_id: config.actuator_id.clone(),
            actuator_type: config.actuator_type.clone(),
            pin: config.pin,
            current_value: config.initial_value,
            target_value: config.initial_value,
            is_active: false,
            last_update: timestamp,
            operation_count: 0,
            total_runtime_ms: 0,
        };

        self.actuator_states.write().await.insert(config.actuator_id.clone(), state);

        // Record operation
        let operation = ActuatorOperation {
            operation_id,
            actuator_id: config.actuator_id.clone(),
            operation_type: ActuatorOperationType::Configure,
            value: config.initial_value,
            duration_ms: None,
            timestamp,
            deterministic: self.deterministic_mode,
        };

        self.operation_history.write().await.push(operation);

        // Emit audit event
        self.emit_audit_event("actuator_configure", &config.actuator_id, session_id).await?;

        Ok(())
    }

    /// Set actuator value
    pub async fn set_actuator_value(
        &self,
        session_id: &str,
        caps: &str,
        actuator_id: &str,
        value: f32,
        duration_ms: Option<u32>,
    ) -> Result<(), DeviceError> {
        // Validate capabilities
        self.policy_manager.validate_capability(session_id, caps, "device:actuator.control").await?;
        
        // Check DAO policy
        self.policy_manager.check_dao_policy(session_id, "actuator_control").await?;

        let operation_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // Get actuator configuration and state
        let (config, mut state) = {
            let configs = self.actuator_configs.read().await;
            let mut states = self.actuator_states.write().await;
            
            let config = configs.get(actuator_id)
                .cloned()
                .ok_or_else(|| DeviceError::ActuatorNotConfigured(actuator_id.to_string()))?;
            
            let state = states.get_mut(actuator_id)
                .ok_or_else(|| DeviceError::ActuatorNotConfigured(actuator_id.to_string()))?;
            
            (config, state.clone())
        };

        if !config.enabled {
            return Err(DeviceError::ActuatorDisabled(actuator_id.to_string()));
        }

        // Validate value range
        if value < config.min_value || value > config.max_value {
            return Err(DeviceError::ValueOutOfRange(value, config.min_value, config.max_value));
        }

        // Check safety limits
        self.check_safety_limits(&config, &state, value, duration_ms).await?;

        // Update actuator state
        state.current_value = value;
        state.target_value = value;
        state.is_active = true;
        state.last_update = timestamp;
        state.operation_count += 1;

        // Update runtime tracking
        self.update_runtime_tracking(actuator_id, duration_ms.unwrap_or(0)).await;

        // Store updated state
        self.actuator_states.write().await.insert(actuator_id.to_string(), state);

        // Record operation
        let operation = ActuatorOperation {
            operation_id,
            actuator_id: actuator_id.to_string(),
            operation_type: ActuatorOperationType::SetValue,
            value,
            duration_ms,
            timestamp,
            deterministic: self.deterministic_mode,
        };

        self.operation_history.write().await.push(operation);

        // Emit audit event
        self.emit_audit_event("actuator_set_value", actuator_id, session_id).await?;

        Ok(())
    }

    /// Set PWM duty cycle
    pub async fn set_pwm_duty_cycle(
        &self,
        session_id: &str,
        caps: &str,
        actuator_id: &str,
        duty_cycle: f32,
    ) -> Result<(), DeviceError> {
        // Validate capabilities
        self.policy_manager.validate_capability(session_id, caps, "device:actuator.pwm").await?;
        
        // Check DAO policy
        self.policy_manager.check_dao_policy(session_id, "actuator_pwm").await?;

        let operation_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // Validate duty cycle (0.0 to 1.0)
        if duty_cycle < 0.0 || duty_cycle > 1.0 {
            return Err(DeviceError::InvalidConfiguration("Duty cycle must be between 0.0 and 1.0".to_string()));
        }

        // Get actuator configuration and state
        let (config, mut state) = {
            let configs = self.actuator_configs.read().await;
            let mut states = self.actuator_states.write().await;
            
            let config = configs.get(actuator_id)
                .cloned()
                .ok_or_else(|| DeviceError::ActuatorNotConfigured(actuator_id.to_string()))?;
            
            let state = states.get_mut(actuator_id)
                .ok_or_else(|| DeviceError::ActuatorNotConfigured(actuator_id.to_string()))?;
            
            (config, state.clone())
        };

        if !config.enabled {
            return Err(DeviceError::ActuatorDisabled(actuator_id.to_string()));
        }

        // Convert duty cycle to actuator value
        let value = config.min_value + (duty_cycle * (config.max_value - config.min_value));

        // Update actuator state
        state.current_value = value;
        state.target_value = value;
        state.is_active = true;
        state.last_update = timestamp;
        state.operation_count += 1;

        // Store updated state
        self.actuator_states.write().await.insert(actuator_id.to_string(), state);

        // Record operation
        let operation = ActuatorOperation {
            operation_id,
            actuator_id: actuator_id.to_string(),
            operation_type: ActuatorOperationType::SetDutyCycle,
            value: duty_cycle,
            duration_ms: None,
            timestamp,
            deterministic: self.deterministic_mode,
        };

        self.operation_history.write().await.push(operation);

        // Emit audit event
        self.emit_audit_event("actuator_set_pwm", actuator_id, session_id).await?;

        Ok(())
    }

    /// Start actuator pattern
    pub async fn start_pattern(
        &self,
        session_id: &str,
        caps: &str,
        pattern: PWMPattern,
    ) -> Result<(), DeviceError> {
        // Validate capabilities
        self.policy_manager.validate_capability(session_id, caps, "device:actuator.pattern").await?;
        
        // Check DAO policy
        self.policy_manager.check_dao_policy(session_id, "actuator_pattern").await?;

        let operation_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // Validate actuator exists
        let configs = self.actuator_configs.read().await;
        if !configs.contains_key(&pattern.actuator_id) {
            return Err(DeviceError::ActuatorNotConfigured(pattern.actuator_id.clone()));
        }

        // Store active pattern
        self.active_patterns.write().await.insert(pattern.pattern_id.clone(), pattern.clone());

        // Record operation
        let operation = ActuatorOperation {
            operation_id,
            actuator_id: pattern.actuator_id.clone(),
            operation_type: ActuatorOperationType::Start,
            value: 0.0,
            duration_ms: Some(pattern.duration_ms),
            timestamp,
            deterministic: self.deterministic_mode,
        };

        self.operation_history.write().await.push(operation);

        // Emit audit event
        self.emit_audit_event("actuator_start_pattern", &pattern.actuator_id, session_id).await?;

        Ok(())
    }

    /// Stop actuator pattern
    pub async fn stop_pattern(
        &self,
        session_id: &str,
        caps: &str,
        pattern_id: &str,
    ) -> Result<(), DeviceError> {
        // Validate capabilities
        self.policy_manager.validate_capability(session_id, caps, "device:actuator.pattern").await?;
        
        // Check DAO policy
        self.policy_manager.check_dao_policy(session_id, "actuator_pattern").await?;

        let operation_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // Remove active pattern
        let pattern = self.active_patterns.write().await.remove(pattern_id)
            .ok_or_else(|| DeviceError::PatternNotFound(pattern_id.to_string()))?;

        // Record operation
        let operation = ActuatorOperation {
            operation_id,
            actuator_id: pattern.actuator_id,
            operation_type: ActuatorOperationType::Stop,
            value: 0.0,
            duration_ms: None,
            timestamp,
            deterministic: self.deterministic_mode,
        };

        self.operation_history.write().await.push(operation);

        // Emit audit event
        self.emit_audit_event("actuator_stop_pattern", pattern_id, session_id).await?;

        Ok(())
    }

    /// Emergency stop all actuators
    pub async fn emergency_stop(
        &self,
        session_id: &str,
        caps: &str,
    ) -> Result<(), DeviceError> {
        // Validate capabilities
        self.policy_manager.validate_capability(session_id, caps, "device:actuator.emergency").await?;
        
        // Check DAO policy
        self.policy_manager.check_dao_policy(session_id, "actuator_emergency").await?;

        let operation_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // Stop all active patterns
        self.active_patterns.write().await.clear();

        // Set all actuators to safe state
        let mut states = self.actuator_states.write().await;
        for (actuator_id, state) in states.iter_mut() {
            state.current_value = 0.0;
            state.target_value = 0.0;
            state.is_active = false;
            state.last_update = timestamp;
        }

        // Update safety monitor
        let mut safety_monitor = self.safety_monitor.write().await;
        safety_monitor.last_emergency_stop = Some(timestamp);

        // Record operation
        let operation = ActuatorOperation {
            operation_id,
            actuator_id: "ALL".to_string(),
            operation_type: ActuatorOperationType::EmergencyStop,
            value: 0.0,
            duration_ms: None,
            timestamp,
            deterministic: self.deterministic_mode,
        };

        self.operation_history.write().await.push(operation);

        // Emit audit event
        self.emit_audit_event("actuator_emergency_stop", "ALL", session_id).await?;

        Ok(())
    }

    /// Get actuator state
    pub async fn get_actuator_state(&self, actuator_id: &str) -> Result<ActuatorState, DeviceError> {
        let states = self.actuator_states.read().await;
        states.get(actuator_id)
            .cloned()
            .ok_or_else(|| DeviceError::ActuatorNotConfigured(actuator_id.to_string()))
    }

    /// Get all actuator states
    pub async fn get_all_actuator_states(&self) -> Vec<ActuatorState> {
        let states = self.actuator_states.read().await;
        states.values().cloned().collect()
    }

    /// Get actuator configuration
    pub async fn get_actuator_config(&self, actuator_id: &str) -> Result<ActuatorConfig, DeviceError> {
        let configs = self.actuator_configs.read().await;
        configs.get(actuator_id)
            .cloned()
            .ok_or_else(|| DeviceError::ActuatorNotConfigured(actuator_id.to_string()))
    }

    /// Get active patterns
    pub async fn get_active_patterns(&self) -> Vec<PWMPattern> {
        let patterns = self.active_patterns.read().await;
        patterns.values().cloned().collect()
    }

    /// Get operation history
    pub async fn get_operation_history(&self, limit: Option<usize>) -> Vec<ActuatorOperation> {
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

    /// Check safety limits
    async fn check_safety_limits(
        &self,
        config: &ActuatorConfig,
        state: &ActuatorState,
        value: f32,
        duration_ms: Option<u32>,
    ) -> Result<(), DeviceError> {
        if let Some(limits) = &config.safety_limits {
            // Check duty cycle limit
            let duty_cycle = (value - config.min_value) / (config.max_value - config.min_value);
            if duty_cycle > limits.max_duty_cycle {
                self.record_safety_violation(
                    &config.actuator_id,
                    "duty_cycle_exceeded",
                    &format!("Duty cycle {} exceeds limit {}", duty_cycle, limits.max_duty_cycle),
                ).await;
                return Err(DeviceError::SafetyViolation("Duty cycle limit exceeded".to_string()));
            }

            // Check duration limit
            if let Some(duration) = duration_ms {
                if duration > limits.max_duration_ms {
                    self.record_safety_violation(
                        &config.actuator_id,
                        "duration_exceeded",
                        &format!("Duration {}ms exceeds limit {}ms", duration, limits.max_duration_ms),
                    ).await;
                    return Err(DeviceError::SafetyViolation("Duration limit exceeded".to_string()));
                }
            }

            // Check cooldown period
            if let Some(last_stop) = self.safety_monitor.read().await.last_emergency_stop {
                let cooldown_duration = Utc::now() - last_stop;
                if cooldown_duration.num_milliseconds() < limits.cooldown_ms as i64 {
                    return Err(DeviceError::SafetyViolation("Cooldown period not elapsed".to_string()));
                }
            }
        }

        Ok(())
    }

    /// Update runtime tracking
    async fn update_runtime_tracking(&self, actuator_id: &str, duration_ms: u32) {
        let mut safety_monitor = self.safety_monitor.write().await;
        let current_runtime = safety_monitor.actuator_runtime.get(actuator_id).unwrap_or(&0);
        safety_monitor.actuator_runtime.insert(actuator_id.to_string(), current_runtime + duration_ms as u64);
    }

    /// Record safety violation
    async fn record_safety_violation(&self, actuator_id: &str, violation_type: &str, details: &str) {
        let mut safety_monitor = self.safety_monitor.write().await;
        let violation = SafetyViolation {
            actuator_id: actuator_id.to_string(),
            violation_type: violation_type.to_string(),
            timestamp: Utc::now(),
            details: details.to_string(),
        };
        safety_monitor.safety_violations.push(violation);
    }

    /// Create NGFS snapshot of actuator state
    pub async fn create_snapshot(&self) -> Result<Vec<u8>, DeviceError> {
        let actuator_configs = self.actuator_configs.read().await;
        let actuator_states = self.actuator_states.read().await;
        let active_patterns = self.active_patterns.read().await;
        let operation_history = self.operation_history.read().await;
        let safety_monitor = self.safety_monitor.read().await;
        
        let snapshot = ActuatorSnapshot {
            actuator_configs: actuator_configs.values().cloned().collect(),
            actuator_states: actuator_states.values().cloned().collect(),
            active_patterns: active_patterns.values().cloned().collect(),
            operation_history: operation_history.clone(),
            safety_monitor: safety_monitor.clone(),
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

    /// Restore actuator state from NGFS snapshot
    pub async fn restore_snapshot(&self, data: &[u8]) -> Result<(), DeviceError> {
        let snapshot: ActuatorSnapshot = ciborium::de::from_reader(data)
            .map_err(|e| DeviceError::DeserializationError(e.to_string()))?;

        // Restore actuator configurations
        let mut actuator_configs = self.actuator_configs.write().await;
        actuator_configs.clear();
        for config in snapshot.actuator_configs {
            actuator_configs.insert(config.actuator_id.clone(), config);
        }

        // Restore actuator states
        let mut actuator_states = self.actuator_states.write().await;
        actuator_states.clear();
        for state in snapshot.actuator_states {
            actuator_states.insert(state.actuator_id.clone(), state);
        }

        // Restore active patterns
        let mut active_patterns = self.active_patterns.write().await;
        active_patterns.clear();
        for pattern in snapshot.active_patterns {
            active_patterns.insert(pattern.pattern_id.clone(), pattern);
        }

        // Restore operation history
        let mut operation_history = self.operation_history.write().await;
        *operation_history = snapshot.operation_history;

        // Restore safety monitor
        let mut safety_monitor = self.safety_monitor.write().await;
        *safety_monitor = snapshot.safety_monitor;

        // Restore deterministic mode and tick counter
        self.deterministic_mode = snapshot.deterministic_mode;
        let mut tick_counter = self.tick_counter.write().await;
        *tick_counter = snapshot.tick_counter;

        Ok(())
    }

    async fn emit_audit_event(
        &self,
        event_type: &str,
        actuator_id: &str,
        session_id: &str,
    ) -> Result<(), DeviceError> {
        tracing::info!(
            "Actuator audit event: {} for actuator {} in session {}",
            event_type,
            actuator_id,
            session_id
        );
        Ok(())
    }
}

/// Actuator snapshot for NGFS persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ActuatorSnapshot {
    actuator_configs: Vec<ActuatorConfig>,
    actuator_states: Vec<ActuatorState>,
    active_patterns: Vec<PWMPattern>,
    operation_history: Vec<ActuatorOperation>,
    safety_monitor: SafetyMonitor,
    timestamp: DateTime<Utc>,
    deterministic_mode: bool,
    tick_counter: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::DevicePolicyManager;

    #[tokio::test]
    async fn test_actuator_manager_creation() {
        let policy_manager = Arc::new(DevicePolicyManager::new().unwrap());
        let actuator_manager = ActuatorManager::new(policy_manager);
        assert!(actuator_manager.is_ok());
    }

    #[tokio::test]
    async fn test_actuator_configuration() {
        let policy_manager = Arc::new(DevicePolicyManager::new().unwrap());
        let actuator_manager = ActuatorManager::new(policy_manager).unwrap();

        let config = ActuatorConfig {
            actuator_id: "motor1".to_string(),
            actuator_type: ActuatorType::Motor,
            pin: 5,
            frequency_hz: Some(1000),
            min_value: 0.0,
            max_value: 100.0,
            initial_value: 0.0,
            enabled: true,
            safety_limits: Some(SafetyLimits {
                max_duty_cycle: 0.8,
                max_duration_ms: 5000,
                cooldown_ms: 1000,
                emergency_stop: true,
            }),
        };

        let result = actuator_manager.configure_actuator("test_session", "device:actuator.configure", config).await;
        assert!(result.is_ok());

        let state = actuator_manager.get_actuator_state("motor1").await.unwrap();
        assert_eq!(state.actuator_id, "motor1");
        assert_eq!(state.actuator_type, ActuatorType::Motor);
        assert_eq!(state.current_value, 0.0);
    }

    #[tokio::test]
    async fn test_actuator_value_control() {
        let policy_manager = Arc::new(DevicePolicyManager::new().unwrap());
        let actuator_manager = ActuatorManager::new(policy_manager).unwrap();

        // Configure actuator
        let config = ActuatorConfig {
            actuator_id: "led1".to_string(),
            actuator_type: ActuatorType::LED,
            pin: 13,
            frequency_hz: None,
            min_value: 0.0,
            max_value: 255.0,
            initial_value: 0.0,
            enabled: true,
            safety_limits: None,
        };
        actuator_manager.configure_actuator("test_session", "device:actuator.configure", config).await.unwrap();

        // Set actuator value
        actuator_manager.set_actuator_value("test_session", "device:actuator.control", "led1", 128.0, None).await.unwrap();

        let state = actuator_manager.get_actuator_state("led1").await.unwrap();
        assert_eq!(state.current_value, 128.0);
        assert_eq!(state.target_value, 128.0);
        assert!(state.is_active);
    }

    #[tokio::test]
    async fn test_pwm_duty_cycle() {
        let policy_manager = Arc::new(DevicePolicyManager::new().unwrap());
        let actuator_manager = ActuatorManager::new(policy_manager).unwrap();

        // Configure PWM actuator
        let config = ActuatorConfig {
            actuator_id: "servo1".to_string(),
            actuator_type: ActuatorType::Servo,
            pin: 9,
            frequency_hz: Some(50),
            min_value: 0.0,
            max_value: 180.0,
            initial_value: 90.0,
            enabled: true,
            safety_limits: None,
        };
        actuator_manager.configure_actuator("test_session", "device:actuator.configure", config).await.unwrap();

        // Set PWM duty cycle
        actuator_manager.set_pwm_duty_cycle("test_session", "device:actuator.pwm", "servo1", 0.5).await.unwrap();

        let state = actuator_manager.get_actuator_state("servo1").await.unwrap();
        assert_eq!(state.current_value, 90.0); // 0.5 * (180.0 - 0.0) = 90.0
    }

    #[tokio::test]
    async fn test_emergency_stop() {
        let policy_manager = Arc::new(DevicePolicyManager::new().unwrap());
        let actuator_manager = ActuatorManager::new(policy_manager).unwrap();

        // Configure multiple actuators
        let config1 = ActuatorConfig {
            actuator_id: "motor1".to_string(),
            actuator_type: ActuatorType::Motor,
            pin: 5,
            frequency_hz: None,
            min_value: 0.0,
            max_value: 100.0,
            initial_value: 50.0,
            enabled: true,
            safety_limits: None,
        };
        actuator_manager.configure_actuator("test_session", "device:actuator.configure", config1).await.unwrap();

        let config2 = ActuatorConfig {
            actuator_id: "motor2".to_string(),
            actuator_type: ActuatorType::Motor,
            pin: 6,
            frequency_hz: None,
            min_value: 0.0,
            max_value: 100.0,
            initial_value: 75.0,
            enabled: true,
            safety_limits: None,
        };
        actuator_manager.configure_actuator("test_session", "device:actuator.configure", config2).await.unwrap();

        // Emergency stop
        actuator_manager.emergency_stop("test_session", "device:actuator.emergency").await.unwrap();

        // Check all actuators are stopped
        let state1 = actuator_manager.get_actuator_state("motor1").await.unwrap();
        let state2 = actuator_manager.get_actuator_state("motor2").await.unwrap();
        
        assert_eq!(state1.current_value, 0.0);
        assert!(!state1.is_active);
        assert_eq!(state2.current_value, 0.0);
        assert!(!state2.is_active);
    }

    #[tokio::test]
    async fn test_snapshot_restore() {
        let policy_manager = Arc::new(DevicePolicyManager::new().unwrap());
        let actuator_manager = ActuatorManager::new(policy_manager).unwrap();

        // Configure actuator
        let config = ActuatorConfig {
            actuator_id: "relay1".to_string(),
            actuator_type: ActuatorType::Relay,
            pin: 2,
            frequency_hz: None,
            min_value: 0.0,
            max_value: 1.0,
            initial_value: 0.0,
            enabled: true,
            safety_limits: Some(SafetyLimits {
                max_duty_cycle: 1.0,
                max_duration_ms: 10000,
                cooldown_ms: 500,
                emergency_stop: true,
            }),
        };
        actuator_manager.configure_actuator("test_session", "device:actuator.configure", config).await.unwrap();

        // Set some value
        actuator_manager.set_actuator_value("test_session", "device:actuator.control", "relay1", 1.0, Some(1000)).await.unwrap();

        // Create snapshot
        let snapshot_data = actuator_manager.create_snapshot().await.unwrap();

        // Create new manager and restore
        let policy_manager2 = Arc::new(DevicePolicyManager::new().unwrap());
        let actuator_manager2 = ActuatorManager::new(policy_manager2).unwrap();
        actuator_manager2.restore_snapshot(&snapshot_data).await.unwrap();

        // Verify restored state
        let state = actuator_manager2.get_actuator_state("relay1").await.unwrap();
        assert_eq!(state.actuator_id, "relay1");
        assert_eq!(state.actuator_type, ActuatorType::Relay);
        assert_eq!(state.current_value, 1.0);
        assert!(state.is_active);

        let config = actuator_manager2.get_actuator_config("relay1").await.unwrap();
        assert_eq!(config.actuator_id, "relay1");
        assert!(config.safety_limits.is_some());
    }
}
