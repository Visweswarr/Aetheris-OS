//! Runtime configuration

use crate::error::Result;
use super::RuntimeConfig;

pub struct RuntimeConfigBuilder {
    config: RuntimeConfig,
}

impl Default for RuntimeConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeConfigBuilder {
    pub fn new() -> Self { Self { config: RuntimeConfig::default() } }
    
    pub fn from_env() -> Result<Self> { Ok(Self::new()) }
    
    pub fn build(self) -> RuntimeConfig { self.config }
}

pub fn get_backend_from_env() -> String {
    std::env::var("AETHERIS_AI_BACKEND").unwrap_or_else(|_| "onnx".to_string())
}

pub fn validate_config(_config: &RuntimeConfig) -> Result<()> { Ok(()) }

pub fn print_config_summary(_config: &RuntimeConfig) {
    // Config summary printed
}
