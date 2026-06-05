//! Provider Registry for HAL Device Backends
//!
//! This module manages the registration and discovery of device providers,
//! allowing dynamic loading and switching between different hardware backends.

use std::collections::HashMap;
use std::sync::Arc;
use crate::error::DeviceError;
use super::{Provider, ProviderId, DeviceInfo, DeviceType, HalResult};

/// Provider registry for managing device backends
pub struct ProviderRegistry {
    providers: HashMap<ProviderId, Arc<dyn Provider>>,
    device_cache: HashMap<String, DeviceInfo>,
    last_discovery: Option<std::time::Instant>,
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            device_cache: HashMap::new(),
            last_discovery: None,
        }
    }

    /// Register a provider
    pub fn register_provider(&mut self, provider: Arc<dyn Provider>) -> HalResult<()> {
        let provider_id = provider.get_provider_id();
        
        if self.providers.contains_key(&provider_id) {
            return Err(DeviceError::DeviceInitFailed(
                format!("Provider already registered: {}", provider_id)
            ));
        }

        self.providers.insert(provider_id, provider);
        Ok(())
    }

    /// Unregister a provider
    pub fn unregister_provider(&mut self, provider_id: &ProviderId) -> HalResult<()> {
        if self.providers.remove(provider_id).is_none() {
            return Err(DeviceError::DeviceNotFound(
                format!("Provider not found: {}", provider_id)
            ));
        }

        // Remove cached devices from this provider
        self.device_cache.retain(|_, device| device.provider != *provider_id);
        Ok(())
    }

    /// Get a provider by ID
    pub fn get_provider(&self, provider_id: &ProviderId) -> HalResult<Arc<dyn Provider>> {
        self.providers
            .get(provider_id)
            .cloned()
            .ok_or_else(|| DeviceError::DeviceNotFound(
                format!("Provider not found: {}", provider_id)
            ))
    }

    /// Get all registered providers
    pub fn get_providers(&self) -> Vec<Arc<dyn Provider>> {
        self.providers.values().cloned().collect()
    }

    /// Get provider IDs
    pub fn get_provider_ids(&self) -> Vec<ProviderId> {
        self.providers.keys().cloned().collect()
    }

    /// Check if a provider is registered
    pub fn has_provider(&self, provider_id: &ProviderId) -> bool {
        self.providers.contains_key(provider_id)
    }

    /// Discover devices from all providers
    pub async fn discover_devices(&mut self) -> HalResult<Vec<DeviceInfo>> {
        let mut all_devices = Vec::new();

        for (provider_id, provider) in &self.providers {
            match provider.discover_devices().await {
                Ok(devices) => {
                    for device in devices {
                        self.device_cache.insert(device.device_id.clone(), device.clone());
                        all_devices.push(device);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to discover devices from provider {}: {}", provider_id, e);
                }
            }
        }

        self.last_discovery = Some(std::time::Instant::now());
        Ok(all_devices)
    }

    /// Get cached devices
    pub fn get_cached_devices(&self) -> Vec<DeviceInfo> {
        self.device_cache.values().cloned().collect()
    }

    /// Get devices by type
    pub fn get_devices_by_type(&self, device_type: &DeviceType) -> Vec<DeviceInfo> {
        self.device_cache
            .values()
            .filter(|device| &device.device_type == device_type)
            .cloned()
            .collect()
    }

    /// Get devices by provider
    pub fn get_devices_by_provider(&self, provider_id: &ProviderId) -> Vec<DeviceInfo> {
        self.device_cache
            .values()
            .filter(|device| &device.provider == provider_id)
            .cloned()
            .collect()
    }

    /// Get a specific device by ID
    pub fn get_device(&self, device_id: &str) -> HalResult<DeviceInfo> {
        self.device_cache
            .get(device_id)
            .cloned()
            .ok_or_else(|| DeviceError::DeviceNotFound(device_id.to_string()))
    }

    /// Check if device discovery is stale
    pub fn is_discovery_stale(&self, max_age: std::time::Duration) -> bool {
        if let Some(last_discovery) = self.last_discovery {
            last_discovery.elapsed() > max_age
        } else {
            true
        }
    }

    /// Clear device cache
    pub fn clear_cache(&mut self) {
        self.device_cache.clear();
        self.last_discovery = None;
    }

    /// Get registry statistics
    pub fn get_stats(&self) -> RegistryStats {
        let mut stats = RegistryStats::default();
        
        stats.provider_count = self.providers.len();
        stats.device_count = self.device_cache.len();
        
        for device in self.device_cache.values() {
            match device.device_type {
                DeviceType::Camera => stats.camera_count += 1,
                DeviceType::Microphone => stats.microphone_count += 1,
                DeviceType::Gpio => stats.gpio_count += 1,
                DeviceType::Adc => stats.adc_count += 1,
                DeviceType::Actuator => stats.actuator_count += 1,
            }
        }

        stats
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RegistryStats {
    pub provider_count: usize,
    pub device_count: usize,
    pub camera_count: usize,
    pub microphone_count: usize,
    pub gpio_count: usize,
    pub adc_count: usize,
    pub actuator_count: usize,
}

/// Device filter for discovery
#[derive(Debug, Clone)]
pub struct DeviceFilter {
    pub device_types: Option<Vec<DeviceType>>,
    pub providers: Option<Vec<ProviderId>>,
    pub capabilities: Option<Vec<String>>,
    pub device_paths: Option<Vec<String>>,
}

impl DeviceFilter {
    /// Create a new device filter
    pub fn new() -> Self {
        Self {
            device_types: None,
            providers: None,
            capabilities: None,
            device_paths: None,
        }
    }

    /// Filter by device types
    pub fn with_device_types(mut self, types: Vec<DeviceType>) -> Self {
        self.device_types = Some(types);
        self
    }

    /// Filter by providers
    pub fn with_providers(mut self, providers: Vec<ProviderId>) -> Self {
        self.providers = Some(providers);
        self
    }

    /// Filter by capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = Some(capabilities);
        self
    }

    /// Filter by device paths
    pub fn with_device_paths(mut self, paths: Vec<String>) -> Self {
        self.device_paths = Some(paths);
        self
    }

    /// Apply filter to device list
    pub fn apply(&self, devices: &[DeviceInfo]) -> Vec<DeviceInfo> {
        devices
            .iter()
            .filter(|device| {
                // Filter by device types
                if let Some(ref types) = self.device_types {
                    if !types.contains(&device.device_type) {
                        return false;
                    }
                }

                // Filter by providers
                if let Some(ref providers) = self.providers {
                    if !providers.contains(&device.provider) {
                        return false;
                    }
                }

                // Filter by capabilities
                if let Some(ref capabilities) = self.capabilities {
                    if !capabilities.iter().any(|cap| device.capabilities.contains(cap)) {
                        return false;
                    }
                }

                // Filter by device paths
                if let Some(ref paths) = self.device_paths {
                    if !paths.contains(&device.device_path) {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect()
    }
}

impl Default for DeviceFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hal::traits::*;

    // Mock provider for testing
    struct MockProvider {
        provider_id: ProviderId,
        devices: Vec<DeviceInfo>,
    }

    impl MockProvider {
        fn new(provider_id: ProviderId, devices: Vec<DeviceInfo>) -> Self {
            Self { provider_id, devices }
        }
    }

    impl Provider for MockProvider {
        fn get_provider_id(&self) -> ProviderId {
            self.provider_id.clone()
        }

        fn get_capabilities(&self) -> Vec<String> {
            vec!["test".to_string()]
        }

        fn is_available(&self) -> bool {
            true
        }

        async fn discover_devices(&self) -> HalResult<Vec<DeviceInfo>> {
            Ok(self.devices.clone())
        }

        fn get_metadata(&self) -> HashMap<String, String> {
            HashMap::new()
        }
    }

    #[test]
    fn test_provider_registration() {
        let mut registry = ProviderRegistry::new();
        
        let provider = Arc::new(MockProvider::new(
            ProviderId::DefaultDeterministic,
            vec![]
        ));

        assert!(registry.register_provider(provider).is_ok());
        assert!(registry.has_provider(&ProviderId::DefaultDeterministic));
    }

    #[test]
    fn test_duplicate_provider_registration() {
        let mut registry = ProviderRegistry::new();
        
        let provider1 = Arc::new(MockProvider::new(
            ProviderId::DefaultDeterministic,
            vec![]
        ));
        let provider2 = Arc::new(MockProvider::new(
            ProviderId::DefaultDeterministic,
            vec![]
        ));

        assert!(registry.register_provider(provider1).is_ok());
        assert!(registry.register_provider(provider2).is_err());
    }

    #[test]
    fn test_device_filter() {
        let devices = vec![
            DeviceInfo {
                device_id: "camera1".to_string(),
                device_type: DeviceType::Camera,
                device_path: "/dev/video0".to_string(),
                capabilities: vec!["capture".to_string()],
                provider: ProviderId::Linux,
                is_available: true,
                metadata: HashMap::new(),
            },
            DeviceInfo {
                device_id: "gpio1".to_string(),
                device_type: DeviceType::Gpio,
                device_path: "/dev/gpiochip0".to_string(),
                capabilities: vec!["read".to_string(), "write".to_string()],
                provider: ProviderId::Linux,
                is_available: true,
                metadata: HashMap::new(),
            },
        ];

        let filter = DeviceFilter::new()
            .with_device_types(vec![DeviceType::Camera])
            .with_providers(vec![ProviderId::Linux]);

        let filtered = filter.apply(&devices);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].device_type, DeviceType::Camera);
    }
}
