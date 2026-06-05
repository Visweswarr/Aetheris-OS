//! Speech-to-Text tool for AI Core Service

use crate::error::{AiCoreError, Result};
use crate::tools::ToolResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SttConfig {
    pub models_dir: PathBuf,
    pub default_model: String,
    pub vad_config: VadConfig,
    pub audio_config: AudioProcessingConfig,
    pub language_config: LanguageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadConfig {
    pub enabled: bool,
    pub min_speech_duration: u64,
    pub max_silence_duration: u64,
    pub sensitivity: f32,
    pub frame_size: usize,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_speech_duration: 100,
            max_silence_duration: 1000,
            sensitivity: 0.5,
            frame_size: 1024,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioProcessingConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub format: String,
    pub max_duration: u64,
    pub enable_preprocessing: bool,
}

impl Default for AudioProcessingConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            format: "wav".to_string(),
            max_duration: 60,
            enable_preprocessing: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub auto_detect: bool,
    pub fallback_language: String,
    pub supported_languages: Vec<String>,
    pub min_confidence: f32,
}

impl Default for LanguageConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            fallback_language: "en".to_string(),
            supported_languages: vec!["en".to_string()],
            min_confidence: 0.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttInput {
    #[serde(default)]
    pub file_path: String,
    pub audio_data: Option<Vec<u8>>,
    pub model: Option<String>,
    pub language: Option<String>,
    pub enable_vad: Option<bool>,
    pub output_format: Option<String>,
    pub include_timestamps: Option<bool>,
    pub include_confidence: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttOutput {
    pub text: String,
    pub language: String,
    pub confidence: f32,
    pub processing_time_ms: u64,
    pub model_used: String,
    pub vad_segments: Option<Vec<SpeechSegment>>,
    pub timestamps: Option<Vec<WordTimestamp>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechSegment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordTimestamp {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
    pub confidence: f32,
}

pub struct SpeechToTextTool {
    config: SttConfig,
    initialized: bool,
}

impl SpeechToTextTool {
    pub fn new(config: SttConfig) -> Result<Self> {
        Ok(Self {
            config,
            initialized: false,
        })
    }

    pub async fn initialize(&mut self, _model_name: Option<&str>) -> Result<()> {
        self.initialized = true;
        // Speech-to-Text tool initialized
        Ok(())
    }

    pub async fn process_audio_file(&self, input: &SttInput) -> Result<SttOutput> {
        if !self.initialized {
            return Err(AiCoreError::InternalError(
                "STT not initialized".to_string(),
            ));
        }
        if input.audio_data.is_none() {
            if input.file_path.is_empty() {
                return Err(AiCoreError::ValidationError(
                    "Audio file path cannot be empty".to_string(),
                ));
            }
            let path = std::path::Path::new(&input.file_path);
            let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if !matches!(extension, "wav" | "mp3" | "flac" | "ogg") {
                return Err(AiCoreError::ValidationError(
                    "Unsupported audio format".to_string(),
                ));
            }
            let metadata = std::fs::metadata(path)
                .map_err(|e| AiCoreError::IoError(e.to_string()))?;
            if metadata.len() > 500_000 {
                return Err(AiCoreError::ResourceError("Audio file too large".to_string()));
            }
        }

        Ok(SttOutput {
            text: "Mock transcription result".to_string(),
            language: input
                .language
                .clone()
                .unwrap_or_else(|| self.config.language_config.fallback_language.clone()),
            confidence: 0.95,
            processing_time_ms: 100,
            model_used: input
                .model
                .clone()
                .unwrap_or_else(|| self.config.default_model.clone()),
            vad_segments: input.enable_vad.unwrap_or(self.config.vad_config.enabled).then(|| {
                vec![SpeechSegment {
                    start_ms: 0,
                    end_ms: 1000,
                    confidence: 0.95,
                }]
            }),
            timestamps: input.include_timestamps.unwrap_or(false).then(|| {
                vec![WordTimestamp {
                    start_ms: 0,
                    end_ms: 250,
                    text: "Mock".to_string(),
                    confidence: 0.95,
                }]
            }),
        })
    }

    pub async fn list_available_models(&self) -> Result<Vec<String>> {
        Ok(vec![self.config.default_model.clone()])
    }
}

pub fn default_stt_config() -> SttConfig {
    SttConfig {
        models_dir: PathBuf::from("/usr/share/aetheris/models/whisper"),
        default_model: "base".to_string(),
        vad_config: VadConfig::default(),
        audio_config: AudioProcessingConfig::default(),
        language_config: LanguageConfig::default(),
    }
}

pub trait SttInputSource {
    fn decode_stt_input(&self) -> Result<SttInput>;
}

impl SttInputSource for serde_json::Value {
    fn decode_stt_input(&self) -> Result<SttInput> {
        serde_json::from_value(self.clone())
            .map_err(|e| AiCoreError::SerializationError(e.to_string()))
    }
}

impl SttInputSource for Vec<u8> {
    fn decode_stt_input(&self) -> Result<SttInput> {
        serde_cbor::from_slice(self).map_err(|e| AiCoreError::SerializationError(e.to_string()))
    }
}

impl SttInputSource for [u8] {
    fn decode_stt_input(&self) -> Result<SttInput> {
        serde_cbor::from_slice(self).map_err(|e| AiCoreError::SerializationError(e.to_string()))
    }
}

pub async fn execute_speech_to_text<T: SttInputSource + ?Sized>(
    tool: &SpeechToTextTool,
    input: &T,
) -> Result<ToolResult> {
    let stt_input = input.decode_stt_input()?;

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
