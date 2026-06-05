//! Linux Hardware Provider
//!
//! This module provides real hardware backends for Linux systems using
//! V4L2 (cameras), ALSA (microphones), libgpiod (GPIO), IIO (ADC), and
//! PWM/LED sysfs interfaces.

pub mod v4l2_camera;
pub mod alsa_mic;
pub mod libgpiod_gpio;
pub mod iio_adc;
pub mod pwm_led;

// Re-export Linux provider
pub use super::super::super::traits::*;
use std::collections::HashMap;
use std::path::Path;
use crate::error::DeviceError;
use super::super::{Provider, ProviderId, DeviceInfo, DeviceType, HalResult};

/// Linux hardware provider implementation
pub struct LinuxProvider {
    provider_id: ProviderId,
    capabilities: Vec<String>,
    system_supported: bool,
}

impl LinuxProvider {
    /// Create a new Linux provider
    pub fn new() -> HalResult<Self> {
        if !Self::is_system_supported() {
            return Err(DeviceError::DeviceInitFailed(
                "Linux hardware provider not supported on this system".to_string()
            ));
        }

        Ok(Self {
            provider_id: ProviderId::Linux,
            capabilities: vec![
                "camera.capture".to_string(),
                "microphone.capture".to_string(),
                "gpio.read".to_string(),
                "gpio.write".to_string(),
                "gpio.interrupt".to_string(),
                "adc.sample".to_string(),
                "adc.continuous".to_string(),
                "actuator.pwm".to_string(),
                "actuator.led".to_string(),
                "actuator.motor".to_string(),
                "hardware".to_string(),
            ],
            system_supported: true,
        })
    }

    /// Check if Linux hardware provider is supported on this system
    pub fn is_system_supported() -> bool {
        // Check if we're on Linux
        if !cfg!(target_os = "linux") {
            return false;
        }

        // Check for required system interfaces
        let required_paths = vec![
            "/dev/video0",           // V4L2 cameras
            "/dev/snd",              // ALSA audio
            "/dev/gpiochip0",        // GPIO chips
            "/sys/bus/iio/devices",  // IIO ADC devices
            "/sys/class/pwm",        // PWM devices
            "/sys/class/leds",       // LED devices
        ];

        // At least one of these should exist for basic hardware support
        required_paths.iter().any(|path| Path::new(path).exists())
    }

    /// Discover V4L2 camera devices
    async fn discover_v4l2_cameras(&self) -> HalResult<Vec<DeviceInfo>> {
        v4l2_camera::discover_cameras().await
    }

    /// Discover ALSA microphone devices
    async fn discover_alsa_mics(&self) -> HalResult<Vec<DeviceInfo>> {
        alsa_mic::discover_microphones().await
    }

    /// Discover GPIO chips
    async fn discover_gpio_chips(&self) -> HalResult<Vec<DeviceInfo>> {
        libgpiod_gpio::discover_gpio_chips().await
    }

    /// Discover IIO ADC devices
    async fn discover_iio_adcs(&self) -> HalResult<Vec<DeviceInfo>> {
        iio_adc::discover_adc_devices().await
    }

    /// Discover PWM and LED devices
    async fn discover_actuators(&self) -> HalResult<Vec<DeviceInfo>> {
        let mut devices = Vec::new();
        
        // Discover PWM devices
        match pwm_led::discover_pwm_devices().await {
            Ok(mut pwm_devices) => devices.append(&mut pwm_devices),
            Err(e) => tracing::warn!("Failed to discover PWM devices: {}", e),
        }

        // Discover LED devices
        match pwm_led::discover_led_devices().await {
            Ok(mut led_devices) => devices.append(&mut led_devices),
            Err(e) => tracing::warn!("Failed to discover LED devices: {}", e),
        }

        Ok(devices)
    }
}

impl Provider for LinuxProvider {
    fn get_provider_id(&self) -> ProviderId {
        self.provider_id.clone()
    }

    fn get_capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }

    fn is_available(&self) -> bool {
        self.system_supported
    }

    async fn discover_devices(&self) -> HalResult<Vec<DeviceInfo>> {
        let mut all_devices = Vec::new();

        // Discover cameras
        match self.discover_v4l2_cameras().await {
            Ok(mut cameras) => all_devices.append(&mut cameras),
            Err(e) => tracing::warn!("Failed to discover V4L2 cameras: {}", e),
        }

        // Discover microphones
        match self.discover_alsa_mics().await {
            Ok(mut mics) => all_devices.append(&mut mics),
            Err(e) => tracing::warn!("Failed to discover ALSA microphones: {}", e),
        }

        // Discover GPIO chips
        match self.discover_gpio_chips().await {
            Ok(mut gpio_chips) => all_devices.append(&mut gpio_chips),
            Err(e) => tracing::warn!("Failed to discover GPIO chips: {}", e),
        }

        // Discover ADC devices
        match self.discover_iio_adcs().await {
            Ok(mut adcs) => all_devices.append(&mut adcs),
            Err(e) => tracing::warn!("Failed to discover IIO ADC devices: {}", e),
        }

        // Discover actuators
        match self.discover_actuators().await {
            Ok(mut actuators) => all_devices.append(&mut actuators),
            Err(e) => tracing::warn!("Failed to discover actuator devices: {}", e),
        }

        Ok(all_devices)
    }

    fn get_metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("provider_type".to_string(), "linux_hardware".to_string());
        metadata.insert("version".to_string(), "1.0.0".to_string());
        metadata.insert("description".to_string(), "Linux hardware device provider".to_string());
        metadata.insert("system_supported".to_string(), self.system_supported.to_string());
        
        // Add system information
        if let Ok(kernel_version) = std::process::Command::new("uname")
            .arg("-r")
            .output()
        {
            if let Ok(version) = String::from_utf8(kernel_version.stdout) {
                metadata.insert("kernel_version".to_string(), version.trim().to_string());
            }
        }

        Ok(())
    }
}

impl CameraBackend for LinuxProvider {
    fn open(&self, device_path: &str, config: &CameraConfig) -> HalResult<Box<dyn CameraDevice>> {
        v4l2_camera::open_camera(device_path, config)
    }

    async fn list_devices(&self) -> HalResult<Vec<DeviceInfo>> {
        self.discover_v4l2_cameras().await
    }
}

impl MicrophoneBackend for LinuxProvider {
    fn open(&self, device_path: &str, config: &MicrophoneConfig) -> HalResult<Box<dyn MicrophoneDevice>> {
        alsa_mic::open_microphone(device_path, config)
    }

    async fn list_devices(&self) -> HalResult<Vec<DeviceInfo>> {
        self.discover_alsa_mics().await
    }
}

impl GpioBackend for LinuxProvider {
    fn open_chip(&self, chip_path: &str) -> HalResult<Box<dyn GpioChip>> {
        libgpiod_gpio::open_gpio_chip(chip_path)
    }

    async fn list_chips(&self) -> HalResult<Vec<DeviceInfo>> {
        self.discover_gpio_chips().await
    }
}

impl AdcBackend for LinuxProvider {
    fn open_device(&self, device_path: &str) -> HalResult<Box<dyn AdcDevice>> {
        iio_adc::open_adc_device(device_path)
    }

    async fn list_devices(&self) -> HalResult<Vec<DeviceInfo>> {
        self.discover_iio_adcs().await
    }
}

impl ActuatorBackend for LinuxProvider {
    fn open_device(&self, device_path: &str, config: &ActuatorConfig) -> HalResult<Box<dyn ActuatorDevice>> {
        // Determine device type from path
        if device_path.contains("/sys/class/pwm") {
            pwm_led::open_pwm_device(device_path, config)
        } else if device_path.contains("/sys/class/leds") {
            pwm_led::open_led_device(device_path, config)
        } else {
            Err(DeviceError::DeviceNotFound(format!("Unknown actuator device: {}", device_path)))
        }
    }

    async fn list_devices(&self) -> HalResult<Vec<DeviceInfo>> {
        self.discover_actuators().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_provider_system_support() {
        // This test will pass on Linux systems with hardware, fail on others
        let supported = LinuxProvider::is_system_supported();
        
        if cfg!(target_os = "linux") {
            // On Linux, we expect it to be supported if any hardware interfaces exist
            println!("Linux system support: {}", supported);
        } else {
            // On non-Linux systems, it should never be supported
            assert!(!supported);
        }
    }

    #[tokio::test]
    async fn test_linux_provider_creation() {
        match LinuxProvider::new() {
            Ok(provider) => {
                assert_eq!(provider.get_provider_id(), ProviderId::Linux);
                assert!(provider.is_available());
                
                // Try to discover devices (may fail on systems without hardware)
                let devices = provider.discover_devices().await;
                println!("Discovered {} devices", devices.map(|d| d.len()).unwrap_or(0));
            }
            Err(e) => {
                // Expected on systems without Linux hardware support
                println!("Linux provider not available: {}", e);
            }
        }
    }
}
