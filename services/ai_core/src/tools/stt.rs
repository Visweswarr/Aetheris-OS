//! Speech-to-Text tool for AI Core Service

use crate::error::{AiCoreError, Result};
use crate::tools::ToolResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SttConfig {
    pub models_dir: PathBuf,
    pub default_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttInput {
    pub file_path: Option<String>,
    pub audio_data: Option<Vec<u8>>,
    pub model: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttOutput {
    pub text: String,
    pub language: String,
    pub confidence: f32,
    pub processing_time_ms: u64,
}

pub struct SpeechToTextTool {
    config: SttConfig,
    initialized: bool,
}

impl SpeechToTextTool {
    pub fn new(config: SttConfig) -> Result<Self> {
        Ok(Self { config, initialized: false })
    }

    pub async fn initialize(&mut self, _model_name: Option<&str>) -> Result<()> {
        self.initialized = true;
        // Speech-to-Text tool initialized
        Ok(())
    }

    pub async fn process_audio_file(&self, input: &SttInput) -> Result<SttOutput> {
        if !self.initialized {
            return Err(AiCoreError::InternalError("STT not initialized".to_string()));
        }
        
        Ok(SttOutput {
            text: "Mock transcription result".to_string(),
            language: input.language.clone().unwrap_or_else(|| "en".to_string()),
            confidence: 0.95,
            processing_time_ms: 100,
        })
    }
}

pub fn default_stt_config() -> SttConfig {
    SttConfig {
        models_dir: PathBuf::from("/usr/share/aetheris/models/whisper"),
        default_model: "base".to_string(),
    }
}

pub async fn execute_speech_to_text(tool: &SpeechToTextTool, input: &serde_json::Value) -> Result<ToolResult> {
    let stt_input: SttInput = serde_json::from_value(input.clone())
        .map_err(|e| AiCoreError::SerializationError(e.to_string()))?;
    
    let output = tool.process_audio_file(&stt_input).await?;
    
    Ok(ToolResult {
        tool_id: "stt".to_string(),
        success: true,
        result: Some(serde_cbor::to_vec(&output).unwrap_or_default()),
        error_message: None,
        execution_time_ms: output.processing_time_ms,
        metadata: HashMap::new(),
    })
}
