//! GGUF (llama.cpp) Runtime backend

use super::{backend::RuntimeBackendKind, RuntimeConfig, RuntimeRequest, RuntimeResponse};
use crate::contracts::sha256_hex;
use crate::error::Result;
use std::collections::HashMap;

pub struct GgufRuntime {
    initialized: bool,
}

impl Default for GgufRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl GgufRuntime {
    pub fn new() -> Self {
        Self { initialized: false }
    }

    pub async fn initialize(&mut self, _config: RuntimeConfig) -> Result<()> {
        self.initialized = true;
        Ok(())
    }

    pub async fn generate_response(&self, request: &RuntimeRequest) -> Result<RuntimeResponse> {
        let generated_text = format!("GGUF adapter placeholder: {}", request.prompt);
        let input_hash = sha256_hex(request.prompt.as_bytes());
        let output_hash = sha256_hex(generated_text.as_bytes());
        Ok(RuntimeResponse {
            generated_text,
            tokens_generated: 10,
            processing_time_ms: 100,
            backend_kind: RuntimeBackendKind::Gguf,
            backend_metadata: HashMap::from([(
                "execution_mode".to_string(),
                "gguf_adapter_stub".to_string(),
            )]),
            model_id: None,
            model_source: None,
            model_revision: None,
            remote_execution: false,
            input_hash,
            output_hash,
            heg_plan_id: None,
            placement_summary: Vec::new(),
            audit_event: None,
        })
    }

    pub fn is_ready(&self) -> bool {
        self.initialized
    }
}
