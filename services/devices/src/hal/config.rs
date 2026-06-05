//! HAL Configuration Management
//!
//! This module handles configuration loading, validation, and provider selection
//! for the Hardware Abstraction Layer.

use std::collections::HashMap;
use std::env;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::error::DeviceError;
use super::{ProviderId, HalConfig, HalResult};

/// HAL configuration with environment variable support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalConfiguration {
    /// Default provider to use
    pub default_provider: ProviderId,
    
    /// Fallback provider if default is unavailable
    pub fallback_provider: Option<ProviderId>,
    
    /// Enable automatic device discovery
    pub auto_discovery: bool,
    
    /// Device discovery interval in seconds
    pub discovery_interval: u64,
    
    /// Device filters
    pub device_filters: DeviceFilters,
    
    /// Provider-specific configurations
    pub provider_configs: HashMap<String, ProviderConfig>,
    
    /// Audit configuration
    pub audit: AuditConfig,
    
    /// Performance settings
    pub performance: PerformanceConfig,
}

/// Device filtering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFilters {
    /// Allowed device types
    pub allowed_types: Vec<String>,
    
    /// Blocked device types
    pub blocked_types: Vec<String>,
    
    /// Allowed device paths (regex patterns)
    pub allowed_paths: Vec<String>,
    
    /// Blocked device paths (regex patterns)
    pub blocked_paths: Vec<String>,
    
    /// Required capabilities
    pub required_capabilities: Vec<String>,
}

/// Provider-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider-specific settings
    pub settings: HashMap<String, serde_json::Value>,
    
    /// Device-specific overrides
    pub device_overrides: HashMap<String, HashMap<String, serde_json::Value>>,
    
    /// Enable/disable provider
    pub enabled: bool,
}

/// Audit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable audit logging
    pub enabled: bool,
    
    /// Audit log level
    pub log_level: String,
    
    /// Include device metadata in audit logs
    pub include_metadata: bool,
    
    /// Audit log retention days
    pub retention_days: u32,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum concurrent device operations
    pub max_concurrent_ops: usize,
    
    /// Device operation timeout in milliseconds
    pub operation_timeout_ms: u64,
    
    /// Buffer sizes for different device types
    pub buffer_sizes: HashMap<String, usize>,
    
    /// Enable performance monitoring
    pub enable_monitoring: bool,
}

impl Default for HalConfiguration {
    fn default() -> Self {
        Self {
            default_provider: ProviderId::DefaultDeterministic,
            fallback_provider: Some(ProviderId::DefaultDeterministic),
            auto_discovery: true,
            discovery_interval: 30,
            device_filters: DeviceFilters::default(),
            provider_configs: HashMap::new(),
            audit: AuditConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl Default for DeviceFilters {
    fn default() -> Self {
        Self {
            allowed_types: vec![
                "camera".to_string(),
                "microphone".to_string(),
                "gpio".to_string(),
                "adc".to_string(),
                "actuator".to_string(),
            ],
            blocked_types: Vec::new(),
            allowed_paths: Vec::new(),
            blocked_paths: Vec::new(),
            required_capabilities: Vec::new(),
        }
    }
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_level: "info".to_string(),
            include_metadata: true,
            retention_days: 30,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        let mut buffer_sizes = HashMap::new();
        buffer_sizes.insert("camera".to_string(), 4 * 1024 * 1024); // 4MB
        buffer_sizes.insert("microphone".to_string(), 64 * 1024);    // 64KB
        buffer_sizes.insert("gpio".to_string(), 1024);               // 1KB
        buffer_sizes.insert("adc".to_string(), 8 * 1024);            // 8KB
        buffer_sizes.insert("actuator".to_string(), 1024);           // 1KB

        Self {
            max_concurrent_ops: 10,
            operation_timeout_ms: 5000,
            buffer_sizes,
            enable_monitoring: false,
        }
    }
}

/// Configuration loader and manager
pub struct ConfigManager {
    config: HalConfiguration,
    config_path: Option<String>,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self {
            config: HalConfiguration::default(),
            config_path: None,
        }
    }

    /// Load configuration from file
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> HalResult<()> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(DeviceError::DeviceInitFailed(
                format!("Configuration file not found: {}", path.display())
            ));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| DeviceError::IoError(e))?;

        let config: HalConfiguration = toml::from_str(&content)
            .map_err(|e| DeviceError::SerializationError(e.to_string()))?;

        self.config = config;
        self.config_path = Some(path.to_string_lossy().to_string());
        
        Ok(())
    }

    /// Load configuration from environment variables
    pub fn load_from_env(&mut self) -> HalResult<()> {
        // Load default provider
        if let Ok(provider) = env::var("AETHERIS_DEV_PROVIDER") {
            self.config.default_provider = provider.parse()
                .map_err(|e| DeviceError::InvalidConfiguration(format!("Invalid provider: {}", e)))?;
        }

        // Load fallback provider
        if let Ok(fallback) = env::var("AETHERIS_DEV_FALLBACK_PROVIDER") {
            self.config.fallback_provider = Some(fallback.parse()
                .map_err(|e| DeviceError::InvalidConfiguration(format!("Invalid fallback provider: {}", e)))?);
        }

        // Load auto discovery setting
        if let Ok(auto_discovery) = env::var("AETHERIS_DEV_AUTO_DISCOVERY") {
            self.config.auto_discovery = auto_discovery.parse()
                .map_err(|e| DeviceError::InvalidConfiguration(format!("Invalid auto_discovery: {}", e)))?;
        }

        // Load discovery interval
        if let Ok(interval) = env::var("AETHERIS_DEV_DISCOVERY_INTERVAL") {
            self.config.discovery_interval = interval.parse()
                .map_err(|e| DeviceError::InvalidConfiguration(format!("Invalid discovery_interval: {}", e)))?;
        }

        // Load audit settings
        if let Ok(audit_enabled) = env::var("AETHERIS_DEV_AUDIT_ENABLED") {
            self.config.audit.enabled = audit_enabled.parse()
                .map_err(|e| DeviceError::InvalidConfiguration(format!("Invalid audit_enabled: {}", e)))?;
        }

        // Load performance settings
        if let Ok(max_ops) = env::var("AETHERIS_DEV_MAX_CONCURRENT_OPS") {
            self.config.performance.max_concurrent_ops = max_ops.parse()
                .map_err(|e| DeviceError::InvalidConfiguration(format!("Invalid max_concurrent_ops: {}", e)))?;
        }

        Ok(())
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> HalResult<()> {
        let content = toml::to_string_pretty(&self.config)
            .map_err(|e| DeviceError::SerializationError(e.to_string()))?;

        std::fs::write(path, content)
            .map_err(|e| DeviceError::IoError(e))?;

        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> &HalConfiguration {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: HalConfiguration) -> HalResult<()> {
        self.validate_config(&config)?;
        self.config = config;
        Ok(())
    }

    /// Validate configuration
    pub fn validate_config(&self, config: &HalConfiguration) -> HalResult<()> {
        // Validate provider IDs
        if config.default_provider == ProviderId::Custom("".to_string()) {
            return Err(DeviceError::InvalidConfiguration("Default provider cannot be empty".to_string()));
        }

        // Validate discovery interval
        if config.discovery_interval == 0 {
            return Err(DeviceError::InvalidConfiguration("Discovery interval must be greater than 0".to_string()));
        }

        // Validate performance settings
        if config.performance.max_concurrent_ops == 0 {
            return Err(DeviceError::InvalidConfiguration("Max concurrent operations must be greater than 0".to_string()));
        }

        if config.performance.operation_timeout_ms == 0 {
            return Err(DeviceError::InvalidConfiguration("Operation timeout must be greater than 0".to_string()));
        }

        // Validate buffer sizes
        for (device_type, size) in &config.performance.buffer_sizes {
            if *size == 0 {
                return Err(DeviceError::InvalidConfiguration(
                    format!("Buffer size for {} must be greater than 0", device_type)
                ));
            }
        }

        Ok(())
    }

    /// Get provider configuration
    pub fn get_provider_config(&self, provider_id: &str) -> Option<&ProviderConfig> {
        self.config.provider_configs.get(provider_id)
    }

    /// Set provider configuration
    pub fn set_provider_config(&mut self, provider_id: String, config: ProviderConfig) {
        self.config.provider_configs.insert(provider_id, config);
    }

    /// Convert to HAL config
    pub fn to_hal_config(&self) -> HalConfig {
        HalConfig {
            default_provider: self.config.default_provider.clone(),
            fallback_provider: self.config.fallback_provider.clone(),
            auto_discovery: self.config.auto_discovery,
            device_filters: self.config.device_filters.allowed_types.clone(),
            audit_enabled: self.config.audit.enabled,
        }
    }

    /// Load configuration with fallback chain
    pub fn load_with_fallback(&mut self) -> HalResult<()> {
        // Try to load from environment first
        if let Err(e) = self.load_from_env() {
            tracing::warn!("Failed to load configuration from environment: {}", e);
        }

        // Try to load from default config file
        let default_config_paths = vec![
            "aetheris-dev.toml",
            "config/aetheris-dev.toml",
            "/etc/aetheris/dev.toml",
            "~/.config/aetheris/dev.toml",
        ];

        for path in default_config_paths {
            let expanded_path = shellexpand::tilde(path).to_string();
            if Path::new(&expanded_path).exists() {
                if let Err(e) = self.load_from_file(&expanded_path) {
                    tracing::warn!("Failed to load configuration from {}: {}", expanded_path, e);
                } else {
                    tracing::info!("Loaded configuration from {}", expanded_path);
                    break;
                }
            }
        }

        // Validate final configuration
        self.validate_config(&self.config)?;

        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Environment variable names
pub mod env_vars {
    pub const DEV_PROVIDER: &str = "AETHERIS_DEV_PROVIDER";
    pub const FALLBACK_PROVIDER: &str = "AETHERIS_DEV_FALLBACK_PROVIDER";
    pub const AUTO_DISCOVERY: &str = "AETHERIS_DEV_AUDIT_ENABLED";
    pub const DISCOVERY_INTERVAL: &str = "AETHERIS_DEV_DISCOVERY_INTERVAL";
    pub const AUDIT_ENABLED: &str = "AETHERIS_DEV_AUDIT_ENABLED";
    pub const MAX_CONCURRENT_OPS: &str = "AETHERIS_DEV_MAX_CONCURRENT_OPS";
    pub const CONFIG_FILE: &str = "AETHERIS_DEV_CONFIG_FILE";
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading_from_env() {
        env::set_var(env_vars::DEV_PROVIDER, "linux");
        env::set_var(env_vars::AUTO_DISCOVERY, "false");
        env::set_var(env_vars::AUDIT_ENABLED, "false");

        let mut config_manager = ConfigManager::new();
        assert!(config_manager.load_from_env().is_ok());

        let config = config_manager.get_config();
        assert_eq!(config.default_provider, ProviderId::Linux);
        assert!(!config.auto_discovery);
        assert!(!config.audit.enabled);

        // Clean up
        env::remove_var(env_vars::DEV_PROVIDER);
        env::remove_var(env_vars::AUTO_DISCOVERY);
        env::remove_var(env_vars::AUDIT_ENABLED);
    }

    #[test]
    fn test_config_loading_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
default_provider = "linux"
fallback_provider = "det"
auto_discovery = true
discovery_interval = 60

[audit]
enabled = true
log_level = "debug"
include_metadata = true
retention_days = 7

[performance]
max_concurrent_ops = 20
operation_timeout_ms = 10000
enable_monitoring = true
"#;

        temp_file.write_all(config_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let mut config_manager = ConfigManager::new();
        assert!(config_manager.load_from_file(temp_file.path()).is_ok());

        let config = config_manager.get_config();
        assert_eq!(config.default_provider, ProviderId::Linux);
        assert_eq!(config.fallback_provider, Some(ProviderId::DefaultDeterministic));
        assert!(config.auto_discovery);
        assert_eq!(config.discovery_interval, 60);
        assert!(config.audit.enabled);
        assert_eq!(config.audit.log_level, "debug");
        assert_eq!(config.performance.max_concurrent_ops, 20);
    }

    #[test]
    fn test_config_validation() {
        let mut config = HalConfiguration::default();
        config.discovery_interval = 0; // Invalid

        let config_manager = ConfigManager::new();
        assert!(config_manager.validate_config(&config).is_err());
    }

    #[test]
    fn test_hal_config_conversion() {
        let config_manager = ConfigManager::new();
        let hal_config = config_manager.to_hal_config();

        assert_eq!(hal_config.default_provider, ProviderId::DefaultDeterministic);
        assert_eq!(hal_config.fallback_provider, Some(ProviderId::DefaultDeterministic));
        assert!(hal_config.auto_discovery);
        assert!(hal_config.audit_enabled);
    }
}
