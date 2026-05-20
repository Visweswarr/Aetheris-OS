//! ONNX Runtime backend

use super::{RuntimeConfig, RuntimeRequest, RuntimeResponse};
use crate::error::Result;

pub struct OnnxRuntime {
    initialized: bool,
}

impl Default for OnnxRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl OnnxRuntime {
    pub fn new() -> Self {
        Self { initialized: false }
    }

    pub async fn initialize(&mut self, _config: RuntimeConfig) -> Result<()> {
        self.initialized = true;
        Ok(())
    }

    pub async fn generate_response(&self, request: &RuntimeRequest) -> Result<RuntimeResponse> {
        Ok(RuntimeResponse {
            generated_text: format!("ONNX response: {}", request.prompt),
            tokens_generated: 10,
            processing_time_ms: 100,
            heg_plan_id: None,
            placement_summary: Vec::new(),
            audit_event: None,
        })
    }

    pub fn is_ready(&self) -> bool {
        self.initialized
    }
}
