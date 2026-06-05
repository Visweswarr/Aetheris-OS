//! Device Providers for HAL
//!
//! This module contains implementations of device providers for different
//! hardware backends, including deterministic mock providers and real
//! hardware providers.

pub mod deterministic;
pub mod linux;

// Re-export provider implementations
pub use deterministic::DefaultDeterministicProvider;
pub use linux::LinuxProvider;

use crate::error::DeviceError;
use super::{Provider, ProviderId, DeviceInfo, HalResult};

/// Provider factory for creating provider instances
pub struct ProviderFactory;

impl ProviderFactory {
    /// Create a provider by ID
    pub fn create_provider(provider_id: &ProviderId) -> HalResult<Box<dyn Provider>> {
        match provider_id {
            ProviderId::DefaultDeterministic => {
                Ok(Box::new(DefaultDeterministicProvider::new()?))
            }
            ProviderId::Linux => {
                Ok(Box::new(LinuxProvider::new()?))
            }
            ProviderId::Custom(name) => {
                Err(DeviceError::DeviceNotFound(format!("Custom provider not implemented: {}", name)))
            }
        }
    }

    /// Get available provider IDs
    pub fn get_available_providers() -> Vec<ProviderId> {
        let mut providers = vec![ProviderId::DefaultDeterministic];
        
        // Check if Linux provider is available
        if LinuxProvider::is_system_supported() {
            providers.push(ProviderId::Linux);
        }
        
        providers
    }

    /// Check if a provider is available on this system
    pub fn is_provider_available(provider_id: &ProviderId) -> bool {
        match provider_id {
            ProviderId::DefaultDeterministic => true,
            ProviderId::Linux => LinuxProvider::is_system_supported(),
            ProviderId::Custom(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_factory() {
        let providers = ProviderFactory::get_available_providers();
        assert!(providers.contains(&ProviderId::DefaultDeterministic));
        
        // Linux provider availability depends on system
        if LinuxProvider::is_system_supported() {
            assert!(providers.contains(&ProviderId::Linux));
        }
    }

    #[test]
    fn test_provider_availability() {
        assert!(ProviderFactory::is_provider_available(&ProviderId::DefaultDeterministic));
        
        // Linux provider availability depends on system
        let linux_available = ProviderFactory::is_provider_available(&ProviderId::Linux);
        assert_eq!(linux_available, LinuxProvider::is_system_supported());
    }
}
