//! LLM backend stubs

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

use super::schema::{BACKEND_LOCAL, BACKEND_NULL, BACKEND_REMOTE};

/// Backend factory
pub struct BackendFactory;

impl BackendFactory {
    pub fn new() -> Self {
        Self
    }

    pub fn is_backend_supported(backend: u8) -> bool {
        matches!(backend, BACKEND_NULL | BACKEND_LOCAL | BACKEND_REMOTE)
    }
}

/// Backend registry
pub struct BackendRegistry {
    backends: BTreeMap<u8, String>,
}

impl BackendRegistry {
    pub fn new() -> Self {
        let mut backends = BTreeMap::new();
        backends.insert(BACKEND_NULL, "null".to_string());
        backends.insert(BACKEND_LOCAL, "local".to_string());
        backends.insert(BACKEND_REMOTE, "remote".to_string());
        Self { backends }
    }
    
    pub fn register(&mut self, backend: u8, backend_type: String) {
        self.backends.insert(backend, backend_type);
    }
    
    pub fn get(&self, backend: u8) -> Option<&String> {
        self.backends.get(&backend)
    }

    pub fn is_backend_available(&self, backend: u8) -> bool {
        self.backends.contains_key(&backend)
    }

    pub fn get_available_backends(&self) -> Vec<u8> {
        self.backends.keys().copied().collect()
    }
}
