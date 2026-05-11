//! GGUF (llama.cpp) Runtime backend

use crate::error::Result;
use super::{RuntimeConfig, RuntimeRequest, RuntimeResponse};

pub struct GgufRuntime { initialized: bool }

impl Default for GgufRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl GgufRuntime {
    pub fn new() -> Self { Self { initialized: false } }
    
    pub async fn initialize(&mut self, _config: RuntimeConfig) -> Result<()> {
        self.initialized = true;
        Ok(())
    }

    pub async fn generate_response(&self, request: &RuntimeRequest) -> Result<RuntimeResponse> {
        Ok(RuntimeResponse {
            generated_text: format!("GGUF response: {}", request.prompt),
            tokens_generated: 10,
            processing_time_ms: 100,
        })
    }

    pub fn is_ready(&self) -> bool { self.initialized }
}
