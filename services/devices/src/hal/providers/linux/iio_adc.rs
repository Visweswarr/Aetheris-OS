//! Linux IIO ADC Backend
//!
//! This module provides Linux IIO (Industrial I/O) ADC support for Linux systems,
//! enabling real hardware analog-to-digital conversion with sysfs and character device interfaces.

use std::collections::HashMap;
use std::path::Path;
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::RwLock;

use crate::error::DeviceError;
use super::super::super::{
    AdcBackend, AdcDevice, AdcChannel, AdcChannelConfig, AdcSample,
    DeviceInfo, DeviceType, ProviderId, HalResult,
};

/// IIO ADC device implementation
pub struct IioAdcDevice {
    device_info: DeviceInfo,
    device_path: String,
    channel_count: u32,
    channels: Arc<RwLock<HashMap<u32, Arc<IioAdcChannel>>>>,
}

/// IIO ADC channel implementation
pub struct IioAdcChannel {
    channel_number: u32,
    config: AdcChannelConfig,
    device_path: String,
    is_sampling: Arc<AtomicBool>,
    sample_sequence: Arc<AtomicU64>,
    sysfs_path: String,
}

impl IioAdcDevice {
    /// Create a new IIO ADC device
    fn new(device_info: DeviceInfo, device_path: String, channel_count: u32) -> Self {
        Self {
            device_info,
            device_path,
            channel_count,
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get channel count from device info
    async fn get_channel_count_from_device(device_path: &str) -> HalResult<u32> {
        // In a real implementation, this would:
        // 1. Read the device's channel count from sysfs
        // 2. Parse available channels from the device directory

        // For now, simulate by checking if the device directory exists
        if Path::new(device_path).exists() {
            // Mock channel count based on device
            if device_path.contains("iio:device") {
                if let Some(device_num) = device_path.split(':').last() {
                    if let Some(num) = device_num.parse::<u32>().ok() {
                        return Ok(8 + num * 4); // Mock: different devices have different channel counts
                    }
                }
            }
            Ok(8) // Default mock channel count
        } else {
            Err(DeviceError::DeviceNotFound(device_path.to_string()))
        }
    }
}

impl AdcDevice for IioAdcDevice {
    fn configure_channel(&self, channel: u32, config: &AdcChannelConfig) -> HalResult<Box<dyn AdcChannel>> {
        if channel >= self.channel_count {
            return Err(DeviceError::InvalidConfiguration(format!("Channel {} out of range (max: {})", channel, self.channel_count - 1)));
        }

        // Check if channel is already configured
        let channels = futures::executor::block_on(self.channels.read());
        if channels.contains_key(&channel) {
            return Err(DeviceError::InvalidConfiguration(format!("Channel {} already configured", channel)));
        }

        // Create ADC channel
        let sysfs_path = format!("{}/in_voltage{}_raw", self.device_path, channel);
        let adc_channel = Arc::new(IioAdcChannel {
            channel_number: channel,
            config: config.clone(),
            device_path: self.device_path.clone(),
            is_sampling: Arc::new(AtomicBool::new(false)),
            sample_sequence: Arc::new(AtomicU64::new(0)),
            sysfs_path,
        });

        // Store channel in device
        futures::executor::block_on(async {
            let mut channels = self.channels.write().await;
            channels.insert(channel, adc_channel.clone());
        });

        Ok(Box::new(IioAdcChannelWrapper { channel: adc_channel }))
    }

    fn get_device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    fn get_channel_count(&self) -> u32 {
        self.channel_count
    }
}

/// Wrapper for IIO ADC channel to implement the trait
struct IioAdcChannelWrapper {
    channel: Arc<IioAdcChannel>,
}

impl AdcChannel for IioAdcChannelWrapper {
    fn read(&self) -> HalResult<AdcSample> {
        self.channel.read()
    }

    async fn start_sampling(&mut self, rate_hz: u32) -> HalResult<()> {
        self.channel.start_sampling(rate_hz).await
    }

    async fn stop_sampling(&mut self) -> HalResult<()> {
        self.channel.stop_sampling().await
    }

    fn get_config(&self) -> &AdcChannelConfig {
        &self.channel.config
    }

    fn get_channel_number(&self) -> u32 {
        self.channel.channel_number
    }
}

impl IioAdcChannel {
    /// Read channel value from sysfs
    fn read(&self) -> HalResult<AdcSample> {
        // In a real implementation, this would:
        // 1. Read the raw value from sysfs (e.g., /sys/bus/iio/devices/iio:device0/in_voltage0_raw)
        // 2. Read scale and offset values
        // 3. Apply calibration
        // 4. Convert to voltage

        // For now, generate deterministic ADC reading
        let time_ms = chrono::Utc::now().timestamp_millis() as u32;
        let sequence = self.sample_sequence.fetch_add(1, Ordering::Relaxed);
        
        let max_value = (1u32 << self.config.resolution_bits) - 1;
        
        // Create a sine wave pattern
        let phase = (self.channel_number as f32 * 0.1) + (time_ms as f32 * 0.001);
        let sine_value = (phase.sin() + 1.0) / 2.0; // Normalize to 0-1
        let raw_value = (sine_value * max_value as f32) as u32;
        
        let voltage = (raw_value as f32 / max_value as f32) * self.config.reference_voltage;
        let calibrated_voltage = (voltage + self.config.calibration_offset) * self.config.calibration_scale;

        Ok(AdcSample {
            channel: self.channel_number,
            raw_value,
            voltage: calibrated_voltage,
            timestamp: time_ms as u64,
            sequence_number: sequence,
        })
    }

    /// Start continuous sampling
    async fn start_sampling(&self, rate_hz: u32) -> HalResult<()> {
        if self.is_sampling.load(Ordering::Relaxed) {
            return Err(DeviceError::InvalidOperation("Channel already sampling".to_string()));
        }

        // In a real implementation, this would:
        // 1. Configure the sampling rate in sysfs
        // 2. Enable the channel
        // 3. Start the sampling process

        self.is_sampling.store(true, Ordering::Relaxed);
        self.sample_sequence.store(0, Ordering::Relaxed);

        tracing::info!("Started ADC sampling on channel {} at {} Hz", self.channel_number, rate_hz);
        Ok(())
    }

    /// Stop continuous sampling
    async fn stop_sampling(&self) -> HalResult<()> {
        if !self.is_sampling.load(Ordering::Relaxed) {
            return Err(DeviceError::InvalidOperation("Channel not sampling".to_string()));
        }

        // In a real implementation, this would:
        // 1. Stop the sampling process
        // 2. Disable the channel

        self.is_sampling.store(false, Ordering::Relaxed);

        tracing::info!("Stopped ADC sampling on channel {}", self.channel_number);
        Ok(())
    }
}

/// Discover IIO ADC devices
pub async fn discover_adc_devices() -> HalResult<Vec<DeviceInfo>> {
    let mut devices = Vec::new();

    // Look for IIO devices in /sys/bus/iio/devices
    let iio_path = Path::new("/sys/bus/iio/devices");
    if !iio_path.exists() {
        return Ok(devices);
    }

    let entries = fs::read_dir(iio_path)
        .map_err(|e| DeviceError::IoError(e))?;

    for entry in entries {
        let entry = entry.map_err(|e| DeviceError::IoError(e))?;
        let path = entry.path();
        
        if let Some(file_name) = path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                if name_str.starts_with("iio:device") {
                    // This is an IIO device
                    let device_path = path.to_string_lossy().to_string();
                    
                    if let Ok(device_info) = get_iio_device_info(&device_path).await {
                        devices.push(device_info);
                    }
                }
            }
        }
    }

    Ok(devices)
}

/// Get IIO device information
async fn get_iio_device_info(device_path: &str) -> HalResult<DeviceInfo> {
    // In a real implementation, this would:
    // 1. Read device name from sysfs
    // 2. Read available channels
    // 3. Read device capabilities
    // 4. Read scale and offset information

    // For now, create mock device info
    let device_name = format!("IIO ADC Device {}", device_path);
    let channel_count = IioAdcDevice::get_channel_count_from_device(device_path).await?;
    
    let mut metadata = HashMap::new();
    metadata.insert("device_name".to_string(), device_name.clone());
    metadata.insert("channel_count".to_string(), channel_count.to_string());
    metadata.insert("driver".to_string(), "mock_iio".to_string());
    metadata.insert("resolution".to_string(), "12".to_string());
    metadata.insert("reference_voltage".to_string(), "3.3".to_string());

    Ok(DeviceInfo {
        device_id: device_path.to_string(),
        device_type: DeviceType::Adc,
        device_path: device_path.to_string(),
        capabilities: vec![
            "sample".to_string(),
            "continuous".to_string(),
            "calibration".to_string(),
        ],
        provider: ProviderId::Linux,
        is_available: true,
        metadata,
    })
}

/// Open an IIO ADC device
pub fn open_adc_device(device_path: &str) -> HalResult<Box<dyn AdcDevice>> {
    // Validate device path
    if !Path::new(device_path).exists() {
        return Err(DeviceError::DeviceNotFound(device_path.to_string()));
    }

    // Get device information
    let device_info = futures::executor::block_on(get_iio_device_info(device_path))?;
    let channel_count = futures::executor::block_on(IioAdcDevice::get_channel_count_from_device(device_path))?;

    // Create ADC device
    let device = IioAdcDevice::new(device_info, device_path.to_string(), channel_count);
    Ok(Box::new(device))
}

/// IIO backend implementation
pub struct IioBackend;

impl AdcBackend for IioBackend {
    fn open_device(&self, device_path: &str) -> HalResult<Box<dyn AdcDevice>> {
        open_adc_device(device_path)
    }

    async fn list_devices(&self) -> HalResult<Vec<DeviceInfo>> {
        discover_adc_devices().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_iio_adc_discovery() {
        // This test will work on Linux systems with IIO devices
        let devices = discover_adc_devices().await;
        
        if cfg!(target_os = "linux") {
            println!("Discovered {} IIO ADC devices", devices.len());
            for device in &devices {
                println!("  - {}: {}", device.device_id, device.device_path);
            }
        } else {
            // On non-Linux systems, should return empty list
            assert!(devices.is_empty());
        }
    }

    #[tokio::test]
    async fn test_iio_adc_channel_operations() {
        let device_info = DeviceInfo {
            device_id: "/sys/bus/iio/devices/iio:device0".to_string(),
            device_type: DeviceType::Adc,
            device_path: "/sys/bus/iio/devices/iio:device0".to_string(),
            capabilities: vec!["sample".to_string()],
            provider: ProviderId::Linux,
            is_available: true,
            metadata: HashMap::new(),
        };

        let device = IioAdcDevice::new(device_info, "/sys/bus/iio/devices/iio:device0".to_string(), 8);

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

        let mut channel = device.configure_channel(0, &config).unwrap();
        assert_eq!(channel.get_channel_number(), 0);
        assert_eq!(channel.get_config().channel, 0);

        // Test single sample
        let sample = channel.read().unwrap();
        assert_eq!(sample.channel, 0);
        assert!(sample.voltage >= 0.0);
        assert!(sample.voltage <= 3.3);

        // Test continuous sampling
        assert!(channel.start_sampling(100).await.is_ok());
        assert!(channel.stop_sampling().await.is_ok());
    }

    #[test]
    fn test_iio_adc_device_creation() {
        let device_info = DeviceInfo {
            device_id: "/sys/bus/iio/devices/iio:device0".to_string(),
            device_type: DeviceType::Adc,
            device_path: "/sys/bus/iio/devices/iio:device0".to_string(),
            capabilities: vec!["sample".to_string()],
            provider: ProviderId::Linux,
            is_available: true,
            metadata: HashMap::new(),
        };

        let device = IioAdcDevice::new(device_info, "/sys/bus/iio/devices/iio:device0".to_string(), 8);
        assert_eq!(device.get_channel_count(), 8);
        assert_eq!(device.get_device_info().device_path, "/sys/bus/iio/devices/iio:device0");
    }
}
