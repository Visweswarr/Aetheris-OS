//! Backend Registry
//!
//! Manages discovery and registration of language backends for the polyglot runtime.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;
use alloc::collections::BTreeMap;
use core::fmt;

use crate::klog;
use super::backend::{LanguageBackend, LanguageType, ValidationError, RuntimeError};

/// Information about a registered backend
#[derive(Clone)]
pub struct BackendInfo {
    /// Backend instance
    pub backend: Arc<dyn LanguageBackend>,
    /// Backend version
    pub version: String,
    /// Supported capabilities
    pub capabilities: Vec<String>,
    /// Load timestamp (milliseconds)
    pub loaded_at: u64,
}

impl fmt::Debug for BackendInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BackendInfo")
            .field("version", &self.version)
            .field("capabilities", &self.capabilities)
            .field("loaded_at", &self.loaded_at)
            .finish()
    }
}

/// Registry error types
#[derive(Debug, Clone)]
pub enum RegistryError {
    /// Backend already registered
    AlreadyRegistered(LanguageType),
    /// Backend not found
    NotFound(LanguageType),
    /// Validation failed
    ValidationFailed {
        language: LanguageType,
        missing_methods: Vec<String>,
    },
    /// Discovery failed
    DiscoveryFailed(String),
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::AlreadyRegistered(lang) => {
                write!(f, "Backend already registered for language: {}", lang)
            }
            RegistryError::NotFound(lang) => {
                write!(f, "Backend not found for language: {}", lang)
            }
            RegistryError::ValidationFailed { language, missing_methods } => {
                write!(f, "Backend validation failed for {}: missing {:?}", language, missing_methods)
            }
            RegistryError::DiscoveryFailed(reason) => {
                write!(f, "Backend discovery failed: {}", reason)
            }
        }
    }
}

/// Required interface methods that all backends must implement
pub const REQUIRED_METHODS: &[&str] = &[
    "language_type",
    "validate",
    "load",
    "execute",
    "memory_usage",
    "unload",
    "debug_info",
];

/// Backend registry - discovers and manages language backends
pub struct BackendRegistry {
    /// Registered backends by language type
    backends: BTreeMap<u8, BackendInfo>,
    /// Plugins directory path (for future dynamic loading)
    plugins_dir: String,
}

impl BackendRegistry {
    /// Create a new backend registry
    pub const fn new() -> Self {
        Self {
            backends: BTreeMap::new(),
            plugins_dir: String::new(),
        }
    }

    /// Create a registry with a plugins directory
    pub fn with_plugins_dir(plugins_dir: String) -> Self {
        Self {
            backends: BTreeMap::new(),
            plugins_dir,
        }
    }

    /// Set the plugins directory
    pub fn set_plugins_dir(&mut self, path: String) {
        self.plugins_dir = path;
    }

    /// Discover backends from the plugins directory
    /// 
    /// Note: In a no_std environment, this is a placeholder for future
    /// dynamic loading support. Currently returns an empty list.
    pub fn discover(&mut self) -> Result<Vec<LanguageType>, RegistryError> {
        // In no_std, we can't do filesystem operations
        // This would be implemented with platform-specific code
        klog!(INFO, "[POLYGLOT] Backend discovery from '{}' (no_std stub)", self.plugins_dir);
        Ok(Vec::new())
    }

    /// Register a backend manually
    pub fn register(&mut self, backend: Arc<dyn LanguageBackend>) -> Result<(), RegistryError> {
        let language = backend.language_type();
        let key = language as u8;

        // Check if already registered
        if self.backends.contains_key(&key) {
            return Err(RegistryError::AlreadyRegistered(language));
        }

        // Validate the backend
        if let Err(validation_error) = backend.validate() {
            return Err(RegistryError::ValidationFailed {
                language,
                missing_methods: validation_error.missing_methods,
            });
        }

        let info = BackendInfo {
            backend,
            version: "1.0.0".into(),
            capabilities: REQUIRED_METHODS.iter().map(|s| (*s).into()).collect(),
            loaded_at: crate::security::get_current_time_ms(),
        };

        self.backends.insert(key, info);
        klog!(INFO, "[POLYGLOT] Registered backend for language: {}", language);

        Ok(())
    }

    /// Unregister a backend
    pub fn unregister(&mut self, language: LanguageType) -> Result<(), RegistryError> {
        let key = language as u8;
        if self.backends.remove(&key).is_some() {
            klog!(INFO, "[POLYGLOT] Unregistered backend for language: {}", language);
            Ok(())
        } else {
            Err(RegistryError::NotFound(language))
        }
    }

    /// Get backend for a language type
    pub fn get(&self, language: LanguageType) -> Option<Arc<dyn LanguageBackend>> {
        self.backends.get(&(language as u8)).map(|info| info.backend.clone())
    }

    /// Get backend info for a language type
    pub fn get_info(&self, language: LanguageType) -> Option<&BackendInfo> {
        self.backends.get(&(language as u8))
    }

    /// List all registered backends
    pub fn list(&self) -> Vec<(LanguageType, &BackendInfo)> {
        self.backends.iter().filter_map(|(key, info)| {
            let lang = match *key {
                0 => Some(LanguageType::Wasm),
                1 => Some(LanguageType::Python),
                2 => Some(LanguageType::JavaScript),
                3 => Some(LanguageType::Rust),
                4 => Some(LanguageType::Go),
                5 => Some(LanguageType::Cpp),
                6 => Some(LanguageType::CSharp),
                _ => None,
            };
            lang.map(|l| (l, info))
        }).collect()
    }
    
    /// List all "Avengers" backends (the core polyglot heroes)
    pub fn list_avengers(&self) -> Vec<(LanguageType, &BackendInfo)> {
        LanguageType::avengers()
            .iter()
            .filter_map(|lang| {
                self.get_info(*lang).map(|info| (*lang, info))
            })
            .collect()
    }
    
    /// Check if all Avengers are assembled (all 5 core backends registered)
    pub fn avengers_assembled(&self) -> bool {
        LanguageType::avengers().iter().all(|lang| self.is_registered(*lang))
    }
    
    /// Get missing Avengers (backends not yet registered)
    pub fn missing_avengers(&self) -> Vec<LanguageType> {
        LanguageType::avengers()
            .iter()
            .filter(|lang| !self.is_registered(**lang))
            .copied()
            .collect()
    }

    /// Check if a backend is registered for a language
    pub fn is_registered(&self, language: LanguageType) -> bool {
        self.backends.contains_key(&(language as u8))
    }

    /// Get the number of registered backends
    pub fn count(&self) -> usize {
        self.backends.len()
    }

    /// Validate a backend against required interface methods
    pub fn validate_backend(backend: &dyn LanguageBackend) -> Result<(), ValidationError> {
        backend.validate()
    }

    /// Get error with unsupported language identifier
    pub fn unsupported_language_error(language: LanguageType) -> RuntimeError {
        RuntimeError::BackendNotFound(language)
    }
}

impl Default for BackendRegistry {
    fn default() -> Self {
        Self::new()
    }
}
