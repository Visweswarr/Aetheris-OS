//! Current-contract tests for Speech-to-Text functionality.

use std::fs;
use tempfile::TempDir;

use aetheris_ai_core::tools::stt::{
    execute_speech_to_text, default_stt_config, SpeechToTextTool, SttConfig, SttInput,
};

fn create_test_config(temp_dir: &TempDir) -> SttConfig {
    let mut config = default_stt_config();
    config.models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&config.models_dir).unwrap();
    fs::write(config.models_dir.join("base.bin"), b"mock model data").unwrap();
    config
}

#[tokio::test]
async fn test_stt_tool_initialization_and_model_listing() {
    let temp_dir = TempDir::new().unwrap();
    let mut tool = SpeechToTextTool::new(create_test_config(&temp_dir)).unwrap();

    tool.initialize(None).await.unwrap();
    let models = tool.list_available_models().await.unwrap();

    assert_eq!(models, vec!["base".to_string()]);
}

#[tokio::test]
async fn test_stt_tool_processes_file_deterministically() {
    let temp_dir = TempDir::new().unwrap();
    let audio_file = temp_dir.path().join("sample.wav");
    fs::write(&audio_file, b"mock audio data").unwrap();

    let mut tool = SpeechToTextTool::new(create_test_config(&temp_dir)).unwrap();
    tool.initialize(None).await.unwrap();

    let input = SttInput {
        file_path: audio_file.to_string_lossy().to_string(),
        audio_data: None,
        model: Some("base".to_string()),
        language: Some("en".to_string()),
        enable_vad: Some(true),
        output_format: Some("text".to_string()),
        include_timestamps: Some(true),
        include_confidence: Some(true),
    };

    let first = tool.process_audio_file(&input).await.unwrap();
    let second = tool.process_audio_file(&input).await.unwrap();

    assert_eq!(first.text, second.text);
    assert_eq!(first.language, "en");
    assert_eq!(first.model_used, "base");
    assert!(first.vad_segments.as_ref().is_some_and(|segments| !segments.is_empty()));
    assert!(first.timestamps.as_ref().is_some_and(|timestamps| !timestamps.is_empty()));
}

#[tokio::test]
async fn test_stt_tool_accepts_inline_audio_data() {
    let temp_dir = TempDir::new().unwrap();
    let mut tool = SpeechToTextTool::new(create_test_config(&temp_dir)).unwrap();
    tool.initialize(None).await.unwrap();

    let input = SttInput {
        file_path: String::new(),
        audio_data: Some(b"mock audio data".to_vec()),
        model: None,
        language: None,
        enable_vad: Some(false),
        output_format: Some("json".to_string()),
        include_timestamps: Some(false),
        include_confidence: Some(true),
    };

    let output = tool.process_audio_file(&input).await.unwrap();
    assert_eq!(output.language, "en");
    assert_eq!(output.model_used, "base");
    assert!(output.vad_segments.is_none());
}

#[tokio::test]
async fn test_stt_tool_validation_errors() {
    let temp_dir = TempDir::new().unwrap();
    let mut tool = SpeechToTextTool::new(create_test_config(&temp_dir)).unwrap();
    tool.initialize(None).await.unwrap();

    let empty = SttInput {
        file_path: String::new(),
        audio_data: None,
        model: None,
        language: None,
        enable_vad: None,
        output_format: None,
        include_timestamps: None,
        include_confidence: None,
    };
    assert!(tool.process_audio_file(&empty).await.is_err());

    let unsupported = SttInput {
        file_path: "sample.txt".to_string(),
        audio_data: None,
        model: None,
        language: None,
        enable_vad: None,
        output_format: None,
        include_timestamps: None,
        include_confidence: None,
    };
    assert!(tool.process_audio_file(&unsupported).await.is_err());
}

#[tokio::test]
async fn test_execute_speech_to_text_accepts_cbor() {
    let temp_dir = TempDir::new().unwrap();
    let audio_file = temp_dir.path().join("sample.wav");
    fs::write(&audio_file, b"mock audio data").unwrap();

    let mut tool = SpeechToTextTool::new(create_test_config(&temp_dir)).unwrap();
    tool.initialize(None).await.unwrap();

    let input = SttInput {
        file_path: audio_file.to_string_lossy().to_string(),
        audio_data: None,
        model: None,
        language: Some("en".to_string()),
        enable_vad: Some(false),
        output_format: Some("text".to_string()),
        include_timestamps: Some(false),
        include_confidence: Some(true),
    };
    let encoded = serde_cbor::to_vec(&input).unwrap();

    let result = execute_speech_to_text(&tool, &encoded).await.unwrap();

    assert!(result.success);
    assert!(result.result.is_some());
    assert!(result.error_message.is_none());
}
