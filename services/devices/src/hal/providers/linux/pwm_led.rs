//! PWM and LED Actuator Backend for Linux
//!
//! This module provides PWM and LED actuator support for Linux systems using
//! sysfs interfaces (/sys/class/pwm and /sys/class/leds).

use std::collections::HashMap;
use std::path::Path;
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::RwLock;

use crate::error::DeviceError;
use super::super::super::{
    ActuatorBackend, ActuatorDevice, ActuatorConfig, ActuatorType, ActuatorPattern,
    DeviceInfo, DeviceType, ProviderId, HalResult,
};

/// PWM actuator device implementation
pub struct PwmActuatorDevice {
    device_info: DeviceInfo,
    config: ActuatorConfig,
    pwm_path: String,
    current_value: Arc<RwLock<f32>>,
    is_active: Arc<AtomicBool>,
    pattern_sequence: Arc<AtomicU64>,
}

/// LED actuator device implementation
pub struct LedActuatorDevice {
    device_info: DeviceInfo,
    config: ActuatorConfig,
    led_path: String,
    current_value: Arc<RwLock<f32>>,
    is_active: Arc<AtomicBool>,
    pattern_sequence: Arc<AtomicU64>,
}

impl PwmActuatorDevice {
    /// Create a new PWM actuator device
    fn new(device_info: DeviceInfo, config: ActuatorConfig, pwm_path: String) -> Self {
        Self {
            device_info,
            config,
            pwm_path,
            current_value: Arc::new(RwLock::new(config.initial_value)),
            is_active: Arc::new(AtomicBool::new(false)),
            pattern_sequence: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Set PWM duty cycle
    async fn set_duty_cycle(&self, duty_cycle: f32) -> HalResult<()> {
        if duty_cycle < 0.0 || duty_cycle > 1.0 {
            return Err(DeviceError::ValueOutOfRange(duty_cycle, 0.0, 1.0));
        }

        // In a real implementation, this would:
        // 1. Calculate the duty cycle in nanoseconds
        // 2. Write to the duty_cycle file in sysfs
        // 3. Enable the PWM if not already enabled

        // For now, simulate PWM control
        let mut current_value = self.current_value.write().await;
        *current_value = duty_cycle;

        tracing::debug!("Set PWM duty cycle to {}% on {}", duty_cycle * 100.0, self.pwm_path);
        Ok(())
    }

    /// Set PWM frequency
    async fn set_frequency(&self, frequency_hz: u32) -> HalResult<()> {
        // In a real implementation, this would:
        // 1. Calculate the period in nanoseconds (1e9 / frequency_hz)
        // 2. Write to the period file in sysfs

        tracing::debug!("Set PWM frequency to {} Hz on {}", frequency_hz, self.pwm_path);
        Ok(())
    }

    /// Enable PWM
    async fn enable(&self) -> HalResult<()> {
        // In a real implementation, this would:
        // 1. Write "1" to the enable file in sysfs

        self.is_active.store(true, Ordering::Relaxed);
        tracing::debug!("Enabled PWM on {}", self.pwm_path);
        Ok(())
    }

    /// Disable PWM
    async fn disable(&self) -> HalResult<()> {
        // In a real implementation, this would:
        // 1. Write "0" to the enable file in sysfs

        self.is_active.store(false, Ordering::Relaxed);
        tracing::debug!("Disabled PWM on {}", self.pwm_path);
        Ok(())
    }
}

impl ActuatorDevice for PwmActuatorDevice {
    fn set_value(&self, value: f32) -> HalResult<()> {
        if value < self.config.min_value || value > self.config.max_value {
            return Err(DeviceError::ValueOutOfRange(value, self.config.min_value, self.config.max_value));
        }

        // Convert value to duty cycle (0.0 to 1.0)
        let duty_cycle = (value - self.config.min_value) / (self.config.max_value - self.config.min_value);
        
        futures::executor::block_on(async {
            self.set_duty_cycle(duty_cycle).await
        })
    }

    fn get_value(&self) -> HalResult<f32> {
        let current_value = futures::executor::block_on(async {
            let current_value = self.current_value.read().await;
            *current_value
        });
        Ok(current_value)
    }

    async fn start_pattern(&mut self, pattern: &ActuatorPattern) -> HalResult<()> {
        if self.is_active.load(Ordering::Relaxed) {
            return Err(DeviceError::InvalidOperation("PWM already active".to_string()));
        }

        // Enable PWM
        self.enable().await?;

        // In a real implementation, this would:
        // 1. Start a background task to execute the pattern
        // 2. Apply each step with the specified timing

        self.pattern_sequence.store(0, Ordering::Relaxed);
        tracing::info!("Started PWM pattern on {}", self.pwm_path);
        Ok(())
    }

    async fn stop_pattern(&mut self) -> HalResult<()> {
        if !self.is_active.load(Ordering::Relaxed) {
            return Err(DeviceError::InvalidOperation("PWM not active".to_string()));
        }

        // Disable PWM
        self.disable().await?;

        tracing::info!("Stopped PWM pattern on {}", self.pwm_path);
        Ok(())
    }

    fn emergency_stop(&self) -> HalResult<()> {
        // Immediately disable PWM
        futures::executor::block_on(async {
            self.disable().await
        })?;

        // Set value to minimum
        self.set_value(self.config.min_value)?;

        tracing::warn!("Emergency stop on PWM {}", self.pwm_path);
        Ok(())
    }

    fn get_device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    fn get_config(&self) -> &ActuatorConfig {
        &self.config
    }
}

impl LedActuatorDevice {
    /// Create a new LED actuator device
    fn new(device_info: DeviceInfo, config: ActuatorConfig, led_path: String) -> Self {
        Self {
            device_info,
            config,
            led_path,
            current_value: Arc::new(RwLock::new(config.initial_value)),
            is_active: Arc::new(AtomicBool::new(false)),
            pattern_sequence: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Set LED brightness
    async fn set_brightness(&self, brightness: f32) -> HalResult<()> {
        if brightness < 0.0 || brightness > 1.0 {
            return Err(DeviceError::ValueOutOfRange(brightness, 0.0, 1.0));
        }

        // In a real implementation, this would:
        // 1. Convert brightness to the LED's max_brightness range
        // 2. Write to the brightness file in sysfs

        let mut current_value = self.current_value.write().await;
        *current_value = brightness;

        tracing::debug!("Set LED brightness to {}% on {}", brightness * 100.0, self.led_path);
        Ok(())
    }

    /// Enable LED
    async fn enable(&self) -> HalResult<()> {
        // In a real implementation, this would:
        // 1. Write "1" to the brightness file in sysfs

        self.is_active.store(true, Ordering::Relaxed);
        tracing::debug!("Enabled LED on {}", self.led_path);
        Ok(())
    }

    /// Disable LED
    async fn disable(&self) -> HalResult<()> {
        // In a real implementation, this would:
        // 1. Write "0" to the brightness file in sysfs

        self.is_active.store(false, Ordering::Relaxed);
        tracing::debug!("Disabled LED on {}", self.led_path);
        Ok(())
    }
}

impl ActuatorDevice for LedActuatorDevice {
    fn set_value(&self, value: f32) -> HalResult<()> {
        if value < self.config.min_value || value > self.config.max_value {
            return Err(DeviceError::ValueOutOfRange(value, self.config.min_value, self.config.max_value));
        }

        // Convert value to brightness (0.0 to 1.0)
        let brightness = (value - self.config.min_value) / (self.config.max_value - self.config.min_value);
        
        futures::executor::block_on(async {
            self.set_brightness(brightness).await
        })
    }

    fn get_value(&self) -> HalResult<f32> {
        let current_value = futures::executor::block_on(async {
            let current_value = self.current_value.read().await;
            *current_value
        });
        Ok(current_value)
    }

    async fn start_pattern(&mut self, pattern: &ActuatorPattern) -> HalResult<()> {
        if self.is_active.load(Ordering::Relaxed) {
            return Err(DeviceError::InvalidOperation("LED already active".to_string()));
        }

        // Enable LED
        self.enable().await?;

        // In a real implementation, this would:
        // 1. Start a background task to execute the pattern
        // 2. Apply each step with the specified timing

        self.pattern_sequence.store(0, Ordering::Relaxed);
        tracing::info!("Started LED pattern on {}", self.led_path);
        Ok(())
    }

    async fn stop_pattern(&mut self) -> HalResult<()> {
        if !self.is_active.load(Ordering::Relaxed) {
            return Err(DeviceError::InvalidOperation("LED not active".to_string()));
        }

        // Disable LED
        self.disable().await?;

        tracing::info!("Stopped LED pattern on {}", self.led_path);
        Ok(())
    }

    fn emergency_stop(&self) -> HalResult<()> {
        // Immediately disable LED
        futures::executor::block_on(async {
            self.disable().await
        })?;

        // Set value to minimum
        self.set_value(self.config.min_value)?;

        tracing::warn!("Emergency stop on LED {}", self.led_path);
        Ok(())
    }

    fn get_device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    fn get_config(&self) -> &ActuatorConfig {
        &self.config
    }
}

/// Discover PWM devices
pub async fn discover_pwm_devices() -> HalResult<Vec<DeviceInfo>> {
    let mut devices = Vec::new();

    // Look for PWM devices in /sys/class/pwm
    let pwm_path = Path::new("/sys/class/pwm");
    if !pwm_path.exists() {
        return Ok(devices);
    }

    let entries = fs::read_dir(pwm_path)
        .map_err(|e| DeviceError::IoError(e))?;

    for entry in entries {
        let entry = entry.map_err(|e| DeviceError::IoError(e))?;
        let path = entry.path();
        
        if let Some(file_name) = path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                if name_str.starts_with("pwmchip") {
                    // This is a PWM chip
                    let chip_path = path.to_string_lossy().to_string();
                    
                    if let Ok(mut chip_devices) = get_pwm_chip_devices(&chip_path).await {
                        devices.append(&mut chip_devices);
                    }
                }
            }
        }
    }

    Ok(devices)
}

/// Get PWM devices from a chip
async fn get_pwm_chip_devices(chip_path: &str) -> HalResult<Vec<DeviceInfo>> {
    let mut devices = Vec::new();

    // In a real implementation, this would:
    // 1. Read the npwm file to get the number of PWM channels
    // 2. For each channel, create a device info

    // For now, simulate with mock channels
    let npwm = 4; // Mock: 4 PWM channels per chip

    for channel in 0..npwm {
        let device_path = format!("{}/pwm{}", chip_path, channel);
        let device_id = format!("pwmchip{}_pwm{}", 
            chip_path.split("pwmchip").nth(1).unwrap_or("0"), channel);

        let mut metadata = HashMap::new();
        metadata.insert("chip_path".to_string(), chip_path.to_string());
        metadata.insert("channel".to_string(), channel.to_string());
        metadata.insert("device_type".to_string(), "pwm".to_string());

        devices.push(DeviceInfo {
            device_id,
            device_type: DeviceType::Actuator,
            device_path,
            capabilities: vec![
                "pwm".to_string(),
                "duty_cycle".to_string(),
                "frequency".to_string(),
            ],
            provider: ProviderId::Linux,
            is_available: true,
            metadata,
        });
    }

    Ok(devices)
}

/// Discover LED devices
pub async fn discover_led_devices() -> HalResult<Vec<DeviceInfo>> {
    let mut devices = Vec::new();

    // Look for LED devices in /sys/class/leds
    let leds_path = Path::new("/sys/class/leds");
    if !leds_path.exists() {
        return Ok(devices);
    }

    let entries = fs::read_dir(leds_path)
        .map_err(|e| DeviceError::IoError(e))?;

    for entry in entries {
        let entry = entry.map_err(|e| DeviceError::IoError(e))?;
        let path = entry.path();
        
        if let Some(file_name) = path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                // This is an LED device
                let device_path = path.to_string_lossy().to_string();
                
                let mut metadata = HashMap::new();
                metadata.insert("led_name".to_string(), name_str.to_string());
                metadata.insert("device_type".to_string(), "led".to_string());

                devices.push(DeviceInfo {
                    device_id: name_str.to_string(),
                    device_type: DeviceType::Actuator,
                    device_path,
                    capabilities: vec![
                        "led".to_string(),
                        "brightness".to_string(),
                        "blink".to_string(),
                    ],
                    provider: ProviderId::Linux,
                    is_available: true,
                    metadata,
                });
            }
        }
    }

    Ok(devices)
}

/// Open a PWM device
pub fn open_pwm_device(device_path: &str, config: &ActuatorConfig) -> HalResult<Box<dyn ActuatorDevice>> {
    // Validate device path
    if !Path::new(device_path).exists() {
        return Err(DeviceError::DeviceNotFound(device_path.to_string()));
    }

    // Create device information
    let device_info = DeviceInfo {
        device_id: config.actuator_id.clone(),
        device_type: DeviceType::Actuator,
        device_path: device_path.to_string(),
        capabilities: vec![
            "pwm".to_string(),
            "duty_cycle".to_string(),
            "frequency".to_string(),
        ],
        provider: ProviderId::Linux,
        is_available: true,
        metadata: HashMap::new(),
    };

    // Create PWM device
    let pwm_device = PwmActuatorDevice::new(device_info, config.clone(), device_path.to_string());
    Ok(Box::new(pwm_device))
}

/// Open an LED device
pub fn open_led_device(device_path: &str, config: &ActuatorConfig) -> HalResult<Box<dyn ActuatorDevice>> {
    // Validate device path
    if !Path::new(device_path).exists() {
        return Err(DeviceError::DeviceNotFound(device_path.to_string()));
    }

    // Create device information
    let device_info = DeviceInfo {
        device_id: config.actuator_id.clone(),
        device_type: DeviceType::Actuator,
        device_path: device_path.to_string(),
        capabilities: vec![
            "led".to_string(),
            "brightness".to_string(),
            "blink".to_string(),
        ],
        provider: ProviderId::Linux,
        is_available: true,
        metadata: HashMap::new(),
    };

    // Create LED device
    let led_device = LedActuatorDevice::new(device_info, config.clone(), device_path.to_string());
    Ok(Box::new(led_device))
}

/// PWM/LED backend implementation
pub struct PwmLedBackend;

impl ActuatorBackend for PwmLedBackend {
    fn open_device(&self, device_path: &str, config: &ActuatorConfig) -> HalResult<Box<dyn ActuatorDevice>> {
        // Determine device type from path
        if device_path.contains("/sys/class/pwm") {
            open_pwm_device(device_path, config)
        } else if device_path.contains("/sys/class/leds") {
            open_led_device(device_path, config)
        } else {
            Err(DeviceError::DeviceNotFound(format!("Unknown actuator device: {}", device_path)))
        }
    }

    async fn list_devices(&self) -> HalResult<Vec<DeviceInfo>> {
        let mut devices = Vec::new();
        
        // Discover PWM devices
        match discover_pwm_devices().await {
            Ok(mut pwm_devices) => devices.append(&mut pwm_devices),
            Err(e) => tracing::warn!("Failed to discover PWM devices: {}", e),
        }

        // Discover LED devices
        match discover_led_devices().await {
            Ok(mut led_devices) => devices.append(&mut led_devices),
            Err(e) => tracing::warn!("Failed to discover LED devices: {}", e),
        }

        Ok(devices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pwm_led_discovery() {
        // This test will work on Linux systems with PWM and LED devices
        let pwm_devices = discover_pwm_devices().await;
        let led_devices = discover_led_devices().await;
        
        if cfg!(target_os = "linux") {
            println!("Discovered {} PWM devices", pwm_devices.len());
            for device in &pwm_devices {
                println!("  - {}: {}", device.device_id, device.device_path);
            }
            
            println!("Discovered {} LED devices", led_devices.len());
            for device in &led_devices {
                println!("  - {}: {}", device.device_id, device.device_path);
            }
        } else {
            // On non-Linux systems, should return empty lists
            assert!(pwm_devices.is_empty());
            assert!(led_devices.is_empty());
        }
    }

    #[tokio::test]
    async fn test_pwm_actuator_operations() {
        let device_info = DeviceInfo {
            device_id: "pwm0".to_string(),
            device_type: DeviceType::Actuator,
            device_path: "/sys/class/pwm/pwmchip0/pwm0".to_string(),
            capabilities: vec!["pwm".to_string()],
            provider: ProviderId::Linux,
            is_available: true,
            metadata: HashMap::new(),
        };

        let config = ActuatorConfig {
            actuator_id: "pwm0".to_string(),
            actuator_type: ActuatorType::PWM,
            device_path: "/sys/class/pwm/pwmchip0/pwm0".to_string(),
            min_value: 0.0,
            max_value: 1.0,
            initial_value: 0.0,
            frequency_hz: Some(1000),
            enabled: true,
        };

        let pwm_device = PwmActuatorDevice::new(device_info, config, "/sys/class/pwm/pwmchip0/pwm0".to_string());

        // Test value operations
        assert!(pwm_device.set_value(0.5).is_ok());
        assert_eq!(pwm_device.get_value().unwrap(), 0.5);

        // Test pattern operations
        let pattern = ActuatorPattern {
            pattern_id: "test".to_string(),
            pattern_type: super::super::super::PatternType::Linear,
            duration_ms: 1000,
            steps: vec![],
            repeat: false,
        };

        assert!(pwm_device.start_pattern(&pattern).await.is_ok());
        assert!(pwm_device.stop_pattern().await.is_ok());

        // Test emergency stop
        assert!(pwm_device.emergency_stop().is_ok());
    }

    #[tokio::test]
    async fn test_led_actuator_operations() {
        let device_info = DeviceInfo {
            device_id: "led0".to_string(),
            device_type: DeviceType::Actuator,
            device_path: "/sys/class/leds/led0".to_string(),
            capabilities: vec!["led".to_string()],
            provider: ProviderId::Linux,
            is_available: true,
            metadata: HashMap::new(),
        };

        let config = ActuatorConfig {
            actuator_id: "led0".to_string(),
            actuator_type: ActuatorType::LED,
            device_path: "/sys/class/leds/led0".to_string(),
            min_value: 0.0,
            max_value: 1.0,
            initial_value: 0.0,
            frequency_hz: None,
            enabled: true,
        };

        let led_device = LedActuatorDevice::new(device_info, config, "/sys/class/leds/led0".to_string());

        // Test value operations
        assert!(led_device.set_value(0.8).is_ok());
        assert_eq!(led_device.get_value().unwrap(), 0.8);

        // Test pattern operations
        let pattern = ActuatorPattern {
            pattern_id: "test".to_string(),
            pattern_type: super::super::super::PatternType::Square,
            duration_ms: 500,
            steps: vec![],
            repeat: true,
        };

        assert!(led_device.start_pattern(&pattern).await.is_ok());
        assert!(led_device.stop_pattern().await.is_ok());

        // Test emergency stop
        assert!(led_device.emergency_stop().is_ok());
    }
}
