//! Tools module for AI Core Service
//!
//! This module provides the tooling framework including the tool registry,
//! speech-to-text, and text-to-speech tools.

pub mod registry;
pub mod stt;
pub mod tts;

// Re-export commonly used types
pub use registry::{Tool, ToolImplementationType, ToolRegistry, ToolRegistryStats, ToolResult};
pub use stt::{default_stt_config, execute_speech_to_text, SpeechToTextTool, SttConfig};
pub use tts::{default_tts_config, execute_text_to_speech, TextToSpeechTool, TtsConfig};
