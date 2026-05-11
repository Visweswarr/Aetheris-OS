//! Tools module for AI Core Service
//!
//! This module provides the tooling framework including the tool registry,
//! speech-to-text, and text-to-speech tools.

pub mod registry;
pub mod stt;
pub mod tts;

// Re-export commonly used types
pub use registry::{ToolRegistry, Tool, ToolResult, ToolImplementationType, ToolRegistryStats};
pub use stt::{SpeechToTextTool, SttConfig, default_stt_config, execute_speech_to_text};
pub use tts::{TextToSpeechTool, TtsConfig, default_tts_config, execute_text_to_speech};