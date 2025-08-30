//! Hello WASM - WASI-2 Module with Capability-Based Security
//! 
//! This module demonstrates WASI-2 capability-based file access control,
//! implementing a secure WebAssembly runtime with manifest-based configuration
//! and deny-by-default security policies.

pub mod lib;
pub mod host_harness;

// Re-export main components
pub use lib::{WasmModule, WasmConfig, WasmManifest, create_module_from_manifest, load_manifest, save_manifest};
pub use host_harness::{TestSuite, TestResult, run_host_harness};

/// Module version
pub const VERSION: &str = "0.1.0";

/// Module description
pub const DESCRIPTION: &str = "WASI-2 Module with Capability-Based File Access Control";

/// Supported capability types
pub const SUPPORTED_CAPABILITIES: &[&str] = &[
    "read", "write", "create", "truncate", "seek", "tell", "sync", "datasync",
    "readdir", "link", "unlink", "rename", "symlink", "path_create_file",
    "path_create_dir", "path_link", "path_open", "path_readlink", "path_rename",
    "path_symlink", "path_unlink_file", "path_remove_dir", "path_filestat_get",
    "path_filestat_set_times", "path_filestat_set_size", "fd_read", "fd_write",
    "fd_seek", "fd_tell", "fd_sync", "fd_datasync", "fd_readdir", "fd_link",
    "fd_unlink", "fd_rename", "fd_symlink", "fd_create_file", "fd_create_dir",
    "fd_open", "fd_readlink", "fd_remove_dir", "fd_filestat_get",
    "fd_filestat_set_times", "fd_filestat_set_size"
];

/// Default security policy
pub const DEFAULT_SECURITY_POLICY: &str = "deny-by-default";

/// Default allowed paths (empty for deny-by-default)
pub const DEFAULT_ALLOWED_PATHS: &[&str] = &[];

/// Default denied paths (empty for deny-by-default)
pub const DEFAULT_DENIED_PATHS: &[&str] = &[];

/// Module configuration
#[derive(Debug, Clone)]
pub struct ModuleConfig {
    /// Module name
    pub name: String,
    /// Module version
    pub version: String,
    /// Security policy
    pub security_policy: SecurityPolicy,
    /// Default capabilities
    pub default_capabilities: Vec<String>,
    /// Allowed paths
    pub allowed_paths: Vec<String>,
    /// Denied paths
    pub denied_paths: Vec<String>,
    /// Environment variables
    pub environment: std::collections::HashMap<String, String>,
    /// Command line arguments
    pub args: Vec<String>,
}

/// Security policy enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityPolicy {
    /// Deny by default, only allow explicitly permitted
    DenyByDefault,
    /// Allow by default, only deny explicitly blocked
    AllowByDefault,
    /// Custom policy
    Custom(String),
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            name: "hello-wasm".to_string(),
            version: VERSION.to_string(),
            security_policy: SecurityPolicy::DenyByDefault,
            default_capabilities: vec![],
            allowed_paths: DEFAULT_ALLOWED_PATHS.iter().map(|s| s.to_string()).collect(),
            denied_paths: DEFAULT_DENIED_PATHS.iter().map(|s| s.to_string()).collect(),
            environment: std::collections::HashMap::new(),
            args: vec![],
        }
    }
}

impl ModuleConfig {
    /// Create a new module configuration
    pub fn new(name: &str) -> Self {
        let mut config = Self::default();
        config.name = name.to_string();
        config
    }
    
    /// Set security policy
    pub fn with_security_policy(mut self, policy: SecurityPolicy) -> Self {
        self.security_policy = policy;
        self
    }
    
    /// Add capability
    pub fn with_capability(mut self, capability: &str) -> Self {
        if SUPPORTED_CAPABILITIES.contains(&capability) {
            self.default_capabilities.push(capability.to_string());
        }
        self
    }
    
    /// Add allowed path
    pub fn with_allowed_path(mut self, path: &str) -> Self {
        self.allowed_paths.push(path.to_string());
        self
    }
    
    /// Add denied path
    pub fn with_denied_path(mut self, path: &str) -> Self {
        self.denied_paths.push(path.to_string());
        self
    }
    
    /// Set environment variable
    pub fn with_environment(mut self, key: &str, value: &str) -> Self {
        self.environment.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Add command line argument
    pub fn with_arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }
    
    /// Convert to WASM manifest
    pub fn to_manifest(&self) -> WasmManifest {
        WasmManifest {
            name: self.name.clone(),
            version: self.version.clone(),
            required_caps: self.default_capabilities.clone(),
            allowed_paths: self.allowed_paths.clone(),
            denied_paths: self.denied_paths.clone(),
            deny_by_default: matches!(self.security_policy, SecurityPolicy::DenyByDefault),
            environment: self.environment.clone(),
            args: self.args.clone(),
        }
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // Check name
        if self.name.is_empty() {
            errors.push("Module name cannot be empty".to_string());
        }
        
        // Check version
        if self.version.is_empty() {
            errors.push("Module version cannot be empty".to_string());
        }
        
        // Check capabilities
        for cap in &self.default_capabilities {
            if !SUPPORTED_CAPABILITIES.contains(&cap.as_str()) {
                errors.push(format!("Unsupported capability: {}", cap));
            }
        }
        
        // Check paths
        for path in &self.allowed_paths {
            if path.is_empty() {
                errors.push("Allowed path cannot be empty".to_string());
            }
        }
        
        for path in &self.denied_paths {
            if path.is_empty() {
                errors.push("Denied path cannot be empty".to_string());
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Module factory
pub struct ModuleFactory;

impl ModuleFactory {
    /// Create a module from configuration
    pub fn create_module(config: &ModuleConfig) -> Result<WasmModule, Box<dyn std::error::Error>> {
        // Validate configuration
        config.validate()?;
        
        // Convert to manifest
        let manifest = config.to_manifest();
        
        // Create module
        let module = create_module_from_manifest(&manifest)?;
        
        Ok(module)
    }
    
    /// Create a module with basic read capability
    pub fn create_read_only_module(allowed_paths: Vec<String>) -> Result<WasmModule, Box<dyn std::error::Error>> {
        let config = ModuleConfig::new("read-only-module")
            .with_security_policy(SecurityPolicy::DenyByDefault)
            .with_capability("read");
        
        let mut config = config;
        for path in allowed_paths {
            config = config.with_allowed_path(&path);
        }
        
        Self::create_module(&config)
    }
    
    /// Create a module with basic write capability
    pub fn create_write_only_module(allowed_paths: Vec<String>) -> Result<WasmModule, Box<dyn std::error::Error>> {
        let config = ModuleConfig::new("write-only-module")
            .with_security_policy(SecurityPolicy::DenyByDefault)
            .with_capability("write")
            .with_capability("create");
        
        let mut config = config;
        for path in allowed_paths {
            config = config.with_allowed_path(&path);
        }
        
        Self::create_module(&config)
    }
    
    /// Create a module with read and write capabilities
    pub fn create_read_write_module(allowed_paths: Vec<String>) -> Result<WasmModule, Box<dyn std::error::Error>> {
        let config = ModuleConfig::new("read-write-module")
            .with_security_policy(SecurityPolicy::DenyByDefault)
            .with_capability("read")
            .with_capability("write")
            .with_capability("create");
        
        let mut config = config;
        for path in allowed_paths {
            config = config.with_allowed_path(&path);
        }
        
        Self::create_module(&config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_module_config_default() {
        let config = ModuleConfig::default();
        assert_eq!(config.name, "hello-wasm");
        assert_eq!(config.version, VERSION);
        assert!(matches!(config.security_policy, SecurityPolicy::DenyByDefault));
        assert!(config.default_capabilities.is_empty());
        assert!(config.allowed_paths.is_empty());
        assert!(config.denied_paths.is_empty());
    }
    
    #[test]
    fn test_module_config_builder() {
        let config = ModuleConfig::new("test-module")
            .with_security_policy(SecurityPolicy::DenyByDefault)
            .with_capability("read")
            .with_allowed_path("/tmp")
            .with_environment("TEST", "value")
            .with_arg("--test");
        
        assert_eq!(config.name, "test-module");
        assert!(matches!(config.security_policy, SecurityPolicy::DenyByDefault));
        assert!(config.default_capabilities.contains(&"read".to_string()));
        assert!(config.allowed_paths.contains(&"/tmp".to_string()));
        assert_eq!(config.environment.get("TEST"), Some(&"value".to_string()));
        assert!(config.args.contains(&"--test".to_string()));
    }
    
    #[test]
    fn test_module_config_validation() {
        let config = ModuleConfig::new("test-module");
        assert!(config.validate().is_ok());
        
        let invalid_config = ModuleConfig::new("")
            .with_capability("invalid_capability");
        let validation_result = invalid_config.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.contains(&"Module name cannot be empty".to_string()));
        assert!(errors.contains(&"Unsupported capability: invalid_capability".to_string()));
    }
    
    #[test]
    fn test_module_config_to_manifest() {
        let config = ModuleConfig::new("test-module")
            .with_capability("read")
            .with_allowed_path("/tmp");
        
        let manifest = config.to_manifest();
        
        assert_eq!(manifest.name, "test-module");
        assert!(manifest.deny_by_default);
        assert!(manifest.required_caps.contains(&"read".to_string()));
        assert!(manifest.allowed_paths.contains(&"/tmp".to_string()));
    }
    
    #[test]
    fn test_module_factory() {
        let config = ModuleConfig::new("test-module")
            .with_capability("read")
            .with_allowed_path("/tmp");
        
        let module = ModuleFactory::create_module(&config);
        assert!(module.is_ok());
    }
    
    #[test]
    fn test_read_only_module_factory() {
        let module = ModuleFactory::create_read_only_module(vec!["/tmp".to_string()]);
        assert!(module.is_ok());
        
        let module = module.unwrap();
        assert!(module.has_capability(wasi_common::file::FileCaps::READ));
        assert!(!module.has_capability(wasi_common::file::FileCaps::WRITE));
    }
    
    #[test]
    fn test_write_only_module_factory() {
        let module = ModuleFactory::create_write_only_module(vec!["/tmp".to_string()]);
        assert!(module.is_ok());
        
        let module = module.unwrap();
        assert!(!module.has_capability(wasi_common::file::FileCaps::READ));
        assert!(module.has_capability(wasi_common::file::FileCaps::WRITE));
        assert!(module.has_capability(wasi_common::file::FileCaps::CREATE));
    }
    
    #[test]
    fn test_read_write_module_factory() {
        let module = ModuleFactory::create_read_write_module(vec!["/tmp".to_string()]);
        assert!(module.is_ok());
        
        let module = module.unwrap();
        assert!(module.has_capability(wasi_common::file::FileCaps::READ));
        assert!(module.has_capability(wasi_common::file::FileCaps::WRITE));
        assert!(module.has_capability(wasi_common::file::FileCaps::CREATE));
    }
}
