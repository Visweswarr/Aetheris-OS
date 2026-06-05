//! Hardware Abstraction Layer (HAL) for Aetheris OS Device Runtime
//!
//! This module provides a pluggable hardware abstraction layer that allows
//! switching between deterministic mock providers and real hardware providers
//! while maintaining identical APIs and NGFS integration.

pub mod config;
pub mod traits;
pub mod registry;
pub mod providers;

// Re-export commonly used types
pub use config::*;
pub use traits::*;
pub use registry::*;

use std::sync::Arc;
use crate::error::DeviceError;

/// HAL initialization result
pub type HalResult<T> = Result<T, DeviceError>;

/// Provider identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProviderId {
    /// Default deterministic provider (mock/simulation)
    DefaultDeterministic,
    /// Linux hardware provider (V4L2, ALSA, libgpiod, IIO, PWM)
    Linux,
    /// Custom provider (for future extensions)
    Custom(String),
}

impl std::fmt::Display for ProviderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderId::DefaultDeterministic => write!(f, "det"),
            ProviderId::Linux => write!(f, "linux"),
            ProviderId::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl std::str::FromStr for ProviderId {
    type Err = DeviceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "det" | "deterministic" | "default" => Ok(ProviderId::DefaultDeterministic),
            "linux" => Ok(ProviderId::Linux),
            name => Ok(ProviderId::Custom(name.to_string())),
        }
    }
}

/// Device information for discovery and listing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_type: DeviceType,
    pub device_path: String,
    pub capabilities: Vec<String>,
    pub provider: ProviderId,
    pub is_available: bool,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Device types supported by the HAL
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DeviceType {
    Camera,
    Microphone,
    Gpio,
    Adc,
    Actuator,
}

/// Provider status information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderStatus {
    pub provider_id: ProviderId,
    pub is_available: bool,
    pub device_count: usize,
    pub last_error: Option<String>,
    pub capabilities: Vec<String>,
}

/// HAL configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HalConfig {
    pub default_provider: ProviderId,
    pub fallback_provider: Option<ProviderId>,
    pub auto_discovery: bool,
    pub device_filters: Vec<String>,
    pub audit_enabled: bool,
}

impl Default for HalConfig {
    fn default() -> Self {
        Self {
            default_provider: ProviderId::DefaultDeterministic,
            fallback_provider: Some(ProviderId::DefaultDeterministic),
            auto_discovery: true,
            device_filters: Vec::new(),
            audit_enabled: true,
        }
    }
}

/// HAL manager for coordinating providers and device operations
pub struct HalManager {
    config: HalConfig,
    providers: std::collections::HashMap<ProviderId, Arc<dyn Provider>>,
    device_registry: std::collections::HashMap<String, DeviceInfo>,
}

impl HalManager {
    /// Create a new HAL manager with the given configuration
    pub fn new(config: HalConfig) -> HalResult<Self> {
        let mut manager = Self {
            config,
            providers: std::collections::HashMap::new(),
            device_registry: std::collections::HashMap::new(),
        };

        // Initialize providers
        manager.initialize_providers()?;

        Ok(manager)
    }

    /// Initialize all available providers
    fn initialize_providers(&mut self) -> HalResult<()> {
        // Always initialize the deterministic provider
        let det_provider = Arc::new(providers::DefaultDeterministicProvider::new()?);
        self.providers.insert(ProviderId::DefaultDeterministic, det_provider);

        // Try to initialize Linux provider
        match providers::LinuxProvider::new() {
            Ok(linux_provider) => {
                let provider = Arc::new(linux_provider);
                self.providers.insert(ProviderId::Linux, provider);
                tracing::info!("Linux hardware provider initialized successfully");
            }
            Err(e) => {
                tracing::warn!("Linux hardware provider not available: {}", e);
                if self.config.fallback_provider.is_none() {
                    self.config.fallback_provider = Some(ProviderId::DefaultDeterministic);
                }
            }
        }

        Ok(())
    }

    /// Get a provider by ID
    pub fn get_provider(&self, provider_id: &ProviderId) -> HalResult<Arc<dyn Provider>> {
        self.providers
            .get(provider_id)
            .cloned()
            .ok_or_else(|| DeviceError::DeviceNotFound(format!("Provider not found: {}", provider_id)))
    }

    /// Get the default provider
    pub fn get_default_provider(&self) -> HalResult<Arc<dyn Provider>> {
        self.get_provider(&self.config.default_provider)
    }

    /// Get the fallback provider
    pub fn get_fallback_provider(&self) -> HalResult<Arc<dyn Provider>> {
        if let Some(fallback_id) = &self.config.fallback_provider {
            self.get_provider(fallback_id)
        } else {
            self.get_provider(&ProviderId::DefaultDeterministic)
        }
    }

    /// Discover devices from all providers
    pub async fn discover_devices(&mut self) -> HalResult<Vec<DeviceInfo>> {
        let mut all_devices = Vec::new();

        for (provider_id, provider) in &self.providers {
            match provider.discover_devices().await {
                Ok(devices) => {
                    for device in devices {
                        self.device_registry.insert(device.device_id.clone(), device.clone());
                        all_devices.push(device);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to discover devices from provider {}: {}", provider_id, e);
                }
            }
        }

        Ok(all_devices)
    }

    /// Get all discovered devices
    pub fn get_devices(&self) -> Vec<DeviceInfo> {
        self.device_registry.values().cloned().collect()
    }

    /// Get devices by type
    pub fn get_devices_by_type(&self, device_type: &DeviceType) -> Vec<DeviceInfo> {
        self.device_registry
            .values()
            .filter(|device| &device.device_type == device_type)
            .cloned()
            .collect()
    }

    /// Get provider status for all providers
    pub async fn get_provider_status(&self) -> Vec<ProviderStatus> {
        let mut statuses = Vec::new();

        for (provider_id, provider) in &self.providers {
            let device_count = self.device_registry
                .values()
                .filter(|device| device.provider == *provider_id)
                .count();

            let status = ProviderStatus {
                provider_id: provider_id.clone(),
                is_available: true, // If we have the provider, it's available
                device_count,
                last_error: None,
                capabilities: provider.get_capabilities(),
            };

            statuses.push(status);
        }

        statuses
    }

    /// Set the default provider
    pub fn set_default_provider(&mut self, provider_id: ProviderId) -> HalResult<()> {
        if !self.providers.contains_key(&provider_id) {
            return Err(DeviceError::DeviceNotFound(format!("Provider not available: {}", provider_id)));
        }

        self.config.default_provider = provider_id;
        Ok(())
    }

    /// Get HAL configuration
    pub fn get_config(&self) -> &HalConfig {
        &self.config
    }

    /// Update HAL configuration
    pub fn update_config(&mut self, config: HalConfig) -> HalResult<()> {
        // Validate that the default provider is available
        if !self.providers.contains_key(&config.default_provider) {
            return Err(DeviceError::DeviceNotFound(format!("Default provider not available: {}", config.default_provider)));
        }

        self.config = config;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_id_parsing() {
        assert_eq!("det".parse::<ProviderId>().unwrap(), ProviderId::DefaultDeterministic);
        assert_eq!("linux".parse::<ProviderId>().unwrap(), ProviderId::Linux);
        assert_eq!("custom".parse::<ProviderId>().unwrap(), ProviderId::Custom("custom".to_string()));
    }

    #[test]
    fn test_provider_id_display() {
        assert_eq!(ProviderId::DefaultDeterministic.to_string(), "det");
        assert_eq!(ProviderId::Linux.to_string(), "linux");
        assert_eq!(ProviderId::Custom("test".to_string()).to_string(), "test");
    }

    #[tokio::test]
    async fn test_hal_manager_creation() {
        let config = HalConfig::default();
        let manager = HalManager::new(config);
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_device_discovery() {
        let config = HalConfig::default();
        let mut manager = HalManager::new(config).unwrap();
        let devices = manager.discover_devices().await;
        assert!(devices.is_ok());
    }
}
