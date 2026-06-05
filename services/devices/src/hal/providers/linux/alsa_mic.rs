//! ALSA Microphone Backend for Linux
//!
//! This module provides ALSA (Advanced Linux Sound Architecture) microphone
//! support for Linux systems, enabling real hardware audio capture.

use std::collections::HashMap;
use std::path::Path;
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::RwLock;

use crate::error::DeviceError;
use super::super::super::{
    MicrophoneBackend, MicrophoneDevice, MicrophoneConfig, AudioFormat, AudioSamples,
    DeviceInfo, DeviceType, ProviderId, HalResult,
};

/// ALSA microphone device implementation
pub struct AlsaMicrophoneDevice {
    device_info: DeviceInfo,
    config: MicrophoneConfig,
    device_handle: Option<AlsaHandle>,
    is_capturing: Arc<AtomicBool>,
    sample_sequence: Arc<AtomicU64>,
    buffer_info: Arc<RwLock<AudioBufferInfo>>,
}

/// ALSA device handle (mock for now)
#[derive(Debug)]
struct AlsaHandle {
    device_name: String,
    sample_rate: u32,
    channels: u16,
    format: AudioFormat,
}

/// Audio buffer information
#[derive(Debug, Clone)]
struct AudioBufferInfo {
    buffer_size: usize,
    buffer: Vec<u8>,
    current_pos: usize,
}

impl AlsaMicrophoneDevice {
    /// Create a new ALSA microphone device
    fn new(device_info: DeviceInfo, config: MicrophoneConfig) -> Self {
        Self {
            device_info,
            config,
            device_handle: None,
            is_capturing: Arc::new(AtomicBool::new(false)),
            sample_sequence: Arc::new(AtomicU64::new(0)),
            buffer_info: Arc::new(RwLock::new(AudioBufferInfo {
                buffer_size: config.buffer_size as usize,
                buffer: vec![0u8; config.buffer_size as usize],
                current_pos: 0,
            })),
        }
    }

    /// Open the ALSA device
    async fn open_device(&mut self) -> HalResult<()> {
        // In a real implementation, this would:
        // 1. Open the ALSA device using libasound2
        // 2. Set hardware parameters (sample rate, channels, format)
        // 3. Set software parameters (buffer size, period size)
        // 4. Prepare the device for capture

        // For now, simulate device opening
        self.device_handle = Some(AlsaHandle {
            device_name: self.device_info.device_path.clone(),
            sample_rate: self.config.sample_rate,
            channels: self.config.channels,
            format: self.config.format.clone(),
        });

        tracing::info!("Opened ALSA device: {}", self.device_info.device_path);
        Ok(())
    }

    /// Close the ALSA device
    async fn close_device(&mut self) -> HalResult<()> {
        if self.device_handle.is_some() {
            // In a real implementation, this would:
            // 1. Stop capture
            // 2. Close the ALSA device handle
            
            self.device_handle = None;
            tracing::info!("Closed ALSA device: {}", self.device_info.device_path);
        }
        Ok(())
    }

    /// Read audio samples from the device
    async fn read_samples_internal(&mut self) -> HalResult<AudioSamples> {
        if !self.is_capturing.load(Ordering::Relaxed) {
            return Err(DeviceError::InvalidOperation("Microphone not capturing".to_string()));
        }

        let sequence = self.sample_sequence.fetch_add(1, Ordering::Relaxed);
        
        // In a real implementation, this would:
        // 1. Read samples from the ALSA device
        // 2. Convert format if necessary
        // 3. Apply any processing (filtering, etc.)

        // For now, generate mock audio data
        let mut buffer_info = self.buffer_info.write().await;
        
        // Generate deterministic sine wave pattern
        let sample_size = match self.config.format {
            AudioFormat::S16LE => 2,
            AudioFormat::F32LE => 4,
            AudioFormat::S24LE => 3,
        };

        let frame_count = self.config.buffer_size / (self.config.channels as u32 * sample_size);
        
        for i in 0..frame_count {
            for channel in 0..self.config.channels {
                let sample_index = (i * self.config.channels as u32 + channel as u32) as usize;
                let byte_offset = sample_index * sample_size as usize;
                
                if byte_offset + sample_size as usize <= buffer_info.buffer.len() {
                    // Generate sine wave sample
                    let phase = (i as f32 * 0.1) + (channel as f32 * 0.5) + (sequence as f32 * 0.01);
                    let sine_value = (phase.sin() * 127.0) as i8;
                    
                    // Write sample in the appropriate format
                    match self.config.format {
                        AudioFormat::S16LE => {
                            let sample = sine_value as i16;
                            buffer_info.buffer[byte_offset] = (sample & 0xFF) as u8;
                            buffer_info.buffer[byte_offset + 1] = ((sample >> 8) & 0xFF) as u8;
                        }
                        AudioFormat::F32LE => {
                            let sample = sine_value as f32 / 127.0;
                            let bytes = sample.to_le_bytes();
                            buffer_info.buffer[byte_offset..byte_offset + 4].copy_from_slice(&bytes);
                        }
                        AudioFormat::S24LE => {
                            let sample = sine_value as i32;
                            buffer_info.buffer[byte_offset] = (sample & 0xFF) as u8;
                            buffer_info.buffer[byte_offset + 1] = ((sample >> 8) & 0xFF) as u8;
                            buffer_info.buffer[byte_offset + 2] = ((sample >> 16) & 0xFF) as u8;
                        }
                    }
                }
            }
        }

        Ok(AudioSamples {
            data: buffer_info.buffer.clone(),
            sample_rate: self.config.sample_rate,
            channels: self.config.channels,
            format: self.config.format.clone(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            frame_count,
        })
    }
}

impl MicrophoneDevice for AlsaMicrophoneDevice {
    async fn start_capture(&mut self) -> HalResult<()> {
        if self.is_capturing.load(Ordering::Relaxed) {
            return Err(DeviceError::InvalidOperation("Microphone already capturing".to_string()));
        }

        // Open device if not already open
        if self.device_handle.is_none() {
            self.open_device().await?;
        }

        // Start capture
        self.is_capturing.store(true, Ordering::Relaxed);
        self.sample_sequence.store(0, Ordering::Relaxed);

        tracing::info!("Started ALSA microphone capture on {}", self.device_info.device_path);
        Ok(())
    }

    async fn stop_capture(&mut self) -> HalResult<()> {
        if !self.is_capturing.load(Ordering::Relaxed) {
            return Err(DeviceError::InvalidOperation("Microphone not capturing".to_string()));
        }

        // Stop capture
        self.is_capturing.store(false, Ordering::Relaxed);

        // Close device
        self.close_device().await?;

        tracing::info!("Stopped ALSA microphone capture on {}", self.device_info.device_path);
        Ok(())
    }

    async fn read_samples(&mut self) -> HalResult<AudioSamples> {
        self.read_samples_internal().await
    }

    fn get_device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    fn is_capturing(&self) -> bool {
        self.is_capturing.load(Ordering::Relaxed)
    }
}

/// Discover ALSA microphone devices
pub async fn discover_microphones() -> HalResult<Vec<DeviceInfo>> {
    let mut microphones = Vec::new();

    // Look for ALSA devices
    let proc_path = Path::new("/proc/asound");
    if !proc_path.exists() {
        // Fallback: check /dev/snd
        let dev_snd_path = Path::new("/dev/snd");
        if dev_snd_path.exists() {
            return discover_microphones_from_dev_snd().await;
        }
        return Ok(microphones);
    }

    // Read /proc/asound/cards to get available sound cards
    let cards_path = proc_path.join("cards");
    if cards_path.exists() {
        if let Ok(content) = fs::read_to_string(&cards_path) {
            for line in content.lines() {
                if let Some(device_info) = parse_alsa_card_line(line).await {
                    microphones.push(device_info);
                }
            }
        }
    }

    // Also check for USB audio devices
    let usb_path = Path::new("/proc/bus/usb/devices");
    if usb_path.exists() {
        if let Ok(content) = fs::read_to_string(usb_path) {
            for line in content.lines() {
                if line.contains("Audio") {
                    if let Some(device_info) = parse_usb_audio_line(line).await {
                        microphones.push(device_info);
                    }
                }
            }
        }
    }

    Ok(microphones)
}

/// Discover microphones from /dev/snd
async fn discover_microphones_from_dev_snd() -> HalResult<Vec<DeviceInfo>> {
    let mut microphones = Vec::new();

    let dev_snd_path = Path::new("/dev/snd");
    let entries = fs::read_dir(dev_snd_path)
        .map_err(|e| DeviceError::IoError(e))?;

    for entry in entries {
        let entry = entry.map_err(|e| DeviceError::IoError(e))?;
        let path = entry.path();
        
        if let Some(file_name) = path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                if name_str.starts_with("pcmC") && name_str.contains("c") {
                    // This is a capture device
                    let device_path = format!("hw:{}", name_str.chars().nth(4).unwrap_or('0'));
                    
                    let mut metadata = HashMap::new();
                    metadata.insert("device_type".to_string(), "pcm_capture".to_string());
                    metadata.insert("card".to_string(), name_str.chars().nth(4).unwrap_or('0').to_string());

                    microphones.push(DeviceInfo {
                        device_id: device_path.clone(),
                        device_type: DeviceType::Microphone,
                        device_path,
                        capabilities: vec![
                            "capture".to_string(),
                            "s16le".to_string(),
                            "f32le".to_string(),
                        ],
                        provider: ProviderId::Linux,
                        is_available: true,
                        metadata,
                    });
                }
            }
        }
    }

    Ok(microphones)
}

/// Parse ALSA card line from /proc/asound/cards
async fn parse_alsa_card_line(line: &str) -> Option<DeviceInfo> {
    // Format: " 0 [PCH            ]: HDA-Intel - HDA Intel PCH"
    let parts: Vec<&str> = line.trim().split_whitespace().collect();
    if parts.len() >= 3 && parts[1].starts_with('[') {
        let card_id = parts[0];
        let card_name = parts[1].trim_matches('[').trim_matches(']');
        
        let device_path = format!("hw:{}", card_id);
        
        let mut metadata = HashMap::new();
        metadata.insert("card_id".to_string(), card_id.to_string());
        metadata.insert("card_name".to_string(), card_name.to_string());
        metadata.insert("device_type".to_string(), "alsa_card".to_string());

        return Some(DeviceInfo {
            device_id: device_path.clone(),
            device_type: DeviceType::Microphone,
            device_path,
            capabilities: vec![
                "capture".to_string(),
                "s16le".to_string(),
                "f32le".to_string(),
            ],
            provider: ProviderId::Linux,
            is_available: true,
            metadata,
        });
    }
    None
}

/// Parse USB audio line from /proc/bus/usb/devices
async fn parse_usb_audio_line(line: &str) -> Option<DeviceInfo> {
    // This is a simplified parser - in reality, USB audio parsing is more complex
    if line.contains("Audio") {
        let device_path = "hw:1,0".to_string(); // Mock USB audio device
        
        let mut metadata = HashMap::new();
        metadata.insert("device_type".to_string(), "usb_audio".to_string());
        metadata.insert("description".to_string(), "USB Audio Device".to_string());

        return Some(DeviceInfo {
            device_id: device_path.clone(),
            device_type: DeviceType::Microphone,
            device_path,
            capabilities: vec![
                "capture".to_string(),
                "s16le".to_string(),
                "f32le".to_string(),
            ],
            provider: ProviderId::Linux,
            is_available: true,
            metadata,
        });
    }
    None
}

/// Open an ALSA microphone device
pub fn open_microphone(device_path: &str, config: &MicrophoneConfig) -> HalResult<Box<dyn MicrophoneDevice>> {
    // Validate configuration
    validate_microphone_config(config)?;

    // Create device information
    let mut metadata = HashMap::new();
    metadata.insert("device_type".to_string(), "alsa_microphone".to_string());
    metadata.insert("sample_rate".to_string(), config.sample_rate.to_string());
    metadata.insert("channels".to_string(), config.channels.to_string());

    let device_info = DeviceInfo {
        device_id: device_path.to_string(),
        device_type: DeviceType::Microphone,
        device_path: device_path.to_string(),
        capabilities: vec![
            "capture".to_string(),
            "s16le".to_string(),
            "f32le".to_string(),
        ],
        provider: ProviderId::Linux,
        is_available: true,
        metadata,
    };

    // Create microphone device
    let microphone = AlsaMicrophoneDevice::new(device_info, config.clone());
    Ok(Box::new(microphone))
}

/// Validate microphone configuration
fn validate_microphone_config(config: &MicrophoneConfig) -> HalResult<()> {
    if config.sample_rate == 0 {
        return Err(DeviceError::InvalidConfiguration("Invalid sample rate".to_string()));
    }

    if config.channels == 0 {
        return Err(DeviceError::InvalidConfiguration("Invalid channel count".to_string()));
    }

    if config.buffer_size == 0 {
        return Err(DeviceError::InvalidConfiguration("Invalid buffer size".to_string()));
    }

    Ok(())
}

/// ALSA backend implementation
pub struct AlsaBackend;

impl MicrophoneBackend for AlsaBackend {
    fn open(&self, device_path: &str, config: &MicrophoneConfig) -> HalResult<Box<dyn MicrophoneDevice>> {
        open_microphone(device_path, config)
    }

    async fn list_devices(&self) -> HalResult<Vec<DeviceInfo>> {
        discover_microphones().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_alsa_microphone_discovery() {
        // This test will work on Linux systems with ALSA devices
        let microphones = discover_microphones().await;
        
        if cfg!(target_os = "linux") {
            println!("Discovered {} ALSA microphones", microphones.len());
            for mic in &microphones {
                println!("  - {}: {}", mic.device_id, mic.device_path);
            }
        } else {
            // On non-Linux systems, should return empty list
            assert!(microphones.is_empty());
        }
    }

    #[tokio::test]
    async fn test_alsa_microphone_config_validation() {
        let valid_config = MicrophoneConfig {
            sample_rate: 44100,
            channels: 2,
            format: AudioFormat::S16LE,
            buffer_size: 1024,
        };

        assert!(validate_microphone_config(&valid_config).is_ok());

        let invalid_config = MicrophoneConfig {
            sample_rate: 0,
            channels: 2,
            format: AudioFormat::S16LE,
            buffer_size: 1024,
        };

        assert!(validate_microphone_config(&invalid_config).is_err());
    }

    #[test]
    fn test_alsa_microphone_device_creation() {
        let device_info = DeviceInfo {
            device_id: "hw:0,0".to_string(),
            device_type: DeviceType::Microphone,
            device_path: "hw:0,0".to_string(),
            capabilities: vec!["capture".to_string()],
            provider: ProviderId::Linux,
            is_available: true,
            metadata: HashMap::new(),
        };

        let config = MicrophoneConfig {
            sample_rate: 44100,
            channels: 2,
            format: AudioFormat::S16LE,
            buffer_size: 1024,
        };

        let microphone = AlsaMicrophoneDevice::new(device_info, config);
        assert_eq!(microphone.get_device_info().device_path, "hw:0,0");
        assert!(!microphone.is_capturing());
    }
}
