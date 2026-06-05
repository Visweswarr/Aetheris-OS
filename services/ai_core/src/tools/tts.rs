//! Text-to-Speech tool for AI Core Service

use crate::error::{AiCoreError, Result};
use crate::tools::ToolResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioFormat {
    S16LE,
    F32LE,
    Opus,
    Wav,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub default_voice: String,
    pub speed: f32,
    pub pitch: f32,
    pub volume: f32,
    pub emotion: Option<String>,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioOutputConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub format: AudioFormat,
    pub output_device: Option<String>,
    pub enable_streaming: bool,
    pub buffer_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    pub enabled: bool,
    pub chunk_size: usize,
    pub latency_ms: u64,
    pub real_time: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpusConfig {
    pub enabled: bool,
    pub bitrate: u32,
    pub quality: u8,
    pub frame_size_ms: u16,
    pub enable_vbr: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceInfo {
    pub id: String,
    pub name: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    pub models_dir: PathBuf,
    pub default_model: String,
    pub voice_config: VoiceConfig,
    pub audio_config: AudioOutputConfig,
    pub streaming_config: StreamingConfig,
    pub opus_config: OpusConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsInput {
    pub text: String,
    pub voice: Option<String>,
    pub language: Option<String>,
    pub speed: Option<f32>,
    pub pitch: Option<f32>,
    pub volume: Option<f32>,
    pub output_format: Option<AudioFormat>,
    pub enable_streaming: Option<bool>,
    pub output_device: Option<String>,
    pub save_to_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsOutput {
    pub audio_data: Option<Vec<u8>>,
    pub duration_ms: u64,
    pub processing_time_ms: u64,
    pub voice_used: String,
    pub language_used: String,
    pub audio_format: AudioFormat,
    pub file_path: Option<String>,
    pub streaming_session_id: Option<String>,
}

pub struct TextToSpeechTool {
    config: TtsConfig,
    initialized: bool,
}

impl TextToSpeechTool {
    pub fn new(config: TtsConfig) -> Result<Self> {
        Ok(Self {
            config,
            initialized: false,
        })
    }

    pub async fn initialize(&mut self, _model_name: Option<&str>) -> Result<()> {
        self.initialized = true;
        // Text-to-Speech tool initialized
        Ok(())
    }

    pub async fn synthesize_text(&self, input: &TtsInput) -> Result<TtsOutput> {
        if !self.initialized {
            return Err(AiCoreError::InternalError(
                "TTS not initialized".to_string(),
            ));
        }

        if input.text.is_empty() {
            return Err(AiCoreError::ValidationError(
                "Text cannot be empty".to_string(),
            ));
        }
        if input.text.len() > 10_000 {
            return Err(AiCoreError::ValidationError(
                "Text exceeds 10000 character limit".to_string(),
            ));
        }
        if let Some(speed) = input.speed {
            if !(0.25..=2.0).contains(&speed) {
                return Err(AiCoreError::ValidationError("Invalid speed".to_string()));
            }
        }
        if let Some(pitch) = input.pitch {
            if !(0.25..=2.0).contains(&pitch) {
                return Err(AiCoreError::ValidationError("Invalid pitch".to_string()));
            }
        }
        if let Some(volume) = input.volume {
            if !(0.0..=1.0).contains(&volume) {
                return Err(AiCoreError::ValidationError("Invalid volume".to_string()));
            }
        }

        let audio_format = input.output_format.unwrap_or(self.config.audio_config.format);
        let audio_data = Some(vec![0u8; 1000]);
        if let (Some(path), Some(data)) = (&input.save_to_file, &audio_data) {
            tokio::fs::write(path, data)
                .await
                .map_err(|e| AiCoreError::IoError(e.to_string()))?;
        }

        Ok(TtsOutput {
            audio_data,
            duration_ms: 1000,
            processing_time_ms: 100,
            voice_used: input
                .voice
                .clone()
                .unwrap_or_else(|| self.config.voice_config.default_voice.clone()),
            language_used: input
                .language
                .clone()
                .unwrap_or_else(|| self.config.voice_config.language.clone()),
            audio_format,
            file_path: input.save_to_file.clone(),
            streaming_session_id: input
                .enable_streaming
                .unwrap_or(self.config.streaming_config.enabled)
                .then(|| "tts-stream-0001".to_string()),
        })
    }

    pub async fn list_available_voices(&self) -> Result<Vec<VoiceInfo>> {
        Ok(vec![
            VoiceInfo {
                id: "en_female_1".to_string(),
                name: "English Female 1".to_string(),
                language: "en".to_string(),
            },
            VoiceInfo {
                id: "en_male_1".to_string(),
                name: "English Male 1".to_string(),
                language: "en".to_string(),
            },
            VoiceInfo {
                id: "es_female_1".to_string(),
                name: "Spanish Female 1".to_string(),
                language: "es".to_string(),
            },
        ])
    }
}

pub fn default_tts_config() -> TtsConfig {
    TtsConfig {
        models_dir: PathBuf::from("/usr/share/aetheris/models/tts"),
        default_model: "coqui-tts".to_string(),
        voice_config: VoiceConfig {
            default_voice: "en_female_1".to_string(),
            speed: 1.0,
            pitch: 1.0,
            volume: 0.8,
            emotion: None,
            language: "en".to_string(),
        },
        audio_config: AudioOutputConfig {
            sample_rate: 22050,
            channels: 1,
            format: AudioFormat::S16LE,
            output_device: None,
            enable_streaming: false,
            buffer_size: 4096,
        },
        streaming_config: StreamingConfig {
            enabled: false,
            chunk_size: 1024,
            latency_ms: 100,
            real_time: false,
        },
        opus_config: OpusConfig {
            enabled: true,
            bitrate: 64,
            quality: 8,
            frame_size_ms: 20,
            enable_vbr: true,
        },
    }
}

pub trait TtsInputSource {
    fn decode_tts_input(&self) -> Result<TtsInput>;
}

impl TtsInputSource for serde_json::Value {
    fn decode_tts_input(&self) -> Result<TtsInput> {
        serde_json::from_value(self.clone())
            .map_err(|e| AiCoreError::SerializationError(e.to_string()))
    }
}

impl TtsInputSource for Vec<u8> {
    fn decode_tts_input(&self) -> Result<TtsInput> {
        serde_cbor::from_slice(self).map_err(|e| AiCoreError::SerializationError(e.to_string()))
    }
}

impl TtsInputSource for [u8] {
    fn decode_tts_input(&self) -> Result<TtsInput> {
        serde_cbor::from_slice(self).map_err(|e| AiCoreError::SerializationError(e.to_string()))
    }
}

pub async fn execute_text_to_speech<T: TtsInputSource + ?Sized>(
    tool: &TextToSpeechTool,
    input: &T,
) -> Result<ToolResult> {
    let tts_input = input.decode_tts_input()?;

    let output = tool.synthesize_text(&tts_input).await?;

    Ok(ToolResult {
        tool_id: "tts".to_string(),
        success: true,
        result: Some(serde_cbor::to_vec(&output).unwrap_or_default()),
        error_message: None,
        execution_time_ms: output.processing_time_ms,
        metadata: HashMap::new(),
    })
}
