//! Scheme Registry
//!
//! Manages the mapping of Scheme Names to Implementations.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use spin::RwLock;
use super::{Scheme, SchemeId, SchemeError, Result};

pub struct SchemeRegistry {
    schemes: BTreeMap<String, Arc<dyn Scheme>>,
    ids: BTreeMap<SchemeId, String>,
    next_id: usize,
}

impl SchemeRegistry {
    pub fn new() -> Self {
        Self {
            schemes: BTreeMap::new(),
            ids: BTreeMap::new(),
            next_id: 1,
        }
    }

    /// Register a new scheme
    pub fn register(&mut self, name: &str, scheme: Arc<dyn Scheme>) -> Result<SchemeId> {
        if self.schemes.contains_key(name) {
            // Already exists
            return Err(SchemeError::PermissionDenied);
        }

        let id = SchemeId(self.next_id);
        self.next_id += 1;

        self.schemes.insert(String::from(name), scheme);
        self.ids.insert(id, String::from(name));

        crate::kprintln!("[SCHEME] Registered '{}' scheme", name);
        Ok(id)
    }

    /// Get scheme by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Scheme>> {
        self.schemes.get(name).cloned()
    }
}

// Global registry
pub static SCHEMES: RwLock<SchemeRegistry> = RwLock::new(SchemeRegistry {
    schemes: BTreeMap::new(),
    ids: BTreeMap::new(),
    next_id: 1,
});

/// Public API to register a scheme
pub fn register_scheme(name: &str, scheme: Arc<dyn Scheme>) -> Result<SchemeId> {
    SCHEMES.write().register(name, scheme)
}
