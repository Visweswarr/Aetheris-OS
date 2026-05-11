//! Tests for Speech-to-Text functionality
//! 
//! This module contains comprehensive tests for the STT tool integration
//! with the AI Core Service, including VAD, Whisper backend, and CLI commands.

use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;

use aetheris_ai_core::tools::stt::{
    SpeechToTextTool, SttConfig, SttInput, SttOutput, 
    VadConfig, AudioProcessingConfig, LanguageConfig,
    default_stt_config, execute_speech_to_text
};

#[tokio::test]
async fn test_stt_tool_creation() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let tool = SpeechToTextTool::new(config);
    assert!(tool.is_ok());
}

#[tokio::test]
async fn test_stt_tool_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let mut tool = SpeechToTextTool::new(config).unwrap();
    let result = tool.initialize(None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_stt_tool_process_audio() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    // Create a mock audio file
    let audio_file = temp_dir.path().join("test.wav");
    fs::write(&audio_file, b"mock audio data").unwrap();
    
    let mut tool = SpeechToTextTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    let input = SttInput {
        file_path: audio_file.to_string_lossy().to_string(),
        model: None,
        language: Some("en".to_string()),
        enable_vad: Some(true),
        output_format: Some("text".to_string()),
        include_timestamps: Some(true),
        include_confidence: Some(true),
    };
    
    let result = tool.process_audio_file(&input).await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert!(!output.text.is_empty());
    assert_eq!(output.language, "en");
    assert!(output.confidence > 0.0);
    assert!(output.processing_time_ms > 0);
}

#[tokio::test]
async fn test_stt_tool_list_models() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let tool = SpeechToTextTool::new(config).unwrap();
    let models = tool.list_available_models().await.unwrap();
    
    assert_eq!(models.len(), 1);
    assert_eq!(models[0], "base");
}

#[tokio::test]
async fn test_stt_tool_validation() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let mut tool = SpeechToTextTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    // Test empty file path
    let input = SttInput {
        file_path: "".to_string(),
        model: None,
        language: None,
        enable_vad: None,
        output_format: None,
        include_timestamps: None,
        include_confidence: None,
    };
    
    let result = tool.process_audio_file(&input).await;
    assert!(result.is_err());
    
    // Test non-existent file
    let input = SttInput {
        file_path: "/nonexistent/file.wav".to_string(),
        model: None,
        language: None,
        enable_vad: None,
        output_format: None,
        include_timestamps: None,
        include_confidence: None,
    };
    
    let result = tool.process_audio_file(&input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_execute_speech_to_text() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    // Create a mock audio file
    let audio_file = temp_dir.path().join("test.wav");
    fs::write(&audio_file, b"mock audio data").unwrap();
    
    let mut tool = SpeechToTextTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    let input = SttInput {
        file_path: audio_file.to_string_lossy().to_string(),
        model: None,
        language: Some("en".to_string()),
        enable_vad: Some(true),
        output_format: Some("text".to_string()),
        include_timestamps: Some(true),
        include_confidence: Some(true),
    };
    
    let input_cbor = serde_cbor::to_vec(&input).unwrap();
    let result = execute_speech_to_text(&tool, &input_cbor).await;
    
    assert!(result.is_ok());
    let tool_result = result.unwrap();
    assert!(tool_result.success);
    assert!(tool_result.data.is_some());
    assert!(tool_result.error.is_none());
    assert!(tool_result.execution_time_ms > 0);
}

#[tokio::test]
async fn test_stt_tool_with_fixtures() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    // Create fixture audio files
    let fixtures_dir = temp_dir.path().join("fixtures");
    fs::create_dir_all(&fixtures_dir).await.unwrap();
    
    let sample_audio = fixtures_dir.join("sample_audio.wav");
    let ask_audio = fixtures_dir.join("ask.wav");
    let spanish_audio = fixtures_dir.join("spanish_audio.wav");
    
    fs::write(&sample_audio, b"sample audio data").await.unwrap();
    fs::write(&ask_audio, b"ask audio data").await.unwrap();
    fs::write(&spanish_audio, b"spanish audio data").await.unwrap();
    
    let mut tool = SpeechToTextTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    // Test with sample audio
    let input = SttInput {
        file_path: sample_audio.to_string_lossy().to_string(),
        model: None,
        language: Some("en".to_string()),
        enable_vad: Some(true),
        output_format: Some("text".to_string()),
        include_timestamps: Some(false),
        include_confidence: Some(false),
    };
    
    let result = tool.process_audio_file(&input).await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert!(!output.text.is_empty());
    assert_eq!(output.language, "en");
    assert!(output.confidence > 0.0);
    
    // Test with ask audio
    let input = SttInput {
        file_path: ask_audio.to_string_lossy().to_string(),
        model: Some("base".to_string()),
        language: None, // Auto-detect
        enable_vad: Some(false),
        output_format: Some("json".to_string()),
        include_timestamps: Some(true),
        include_confidence: Some(true),
    };
    
    let result = tool.process_audio_file(&input).await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert!(!output.text.is_empty());
    assert_eq!(output.model_used, "base");
    
    // Test with Spanish audio
    let input = SttInput {
        file_path: spanish_audio.to_string_lossy().to_string(),
        model: None,
        language: Some("es".to_string()),
        enable_vad: Some(true),
        output_format: Some("srt".to_string()),
        include_timestamps: Some(true),
        include_confidence: Some(true),
    };
    
    let result = tool.process_audio_file(&input).await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert!(!output.text.is_empty());
    assert_eq!(output.language, "es");
}

#[tokio::test]
async fn test_stt_tool_vad_segments() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    // Create a mock audio file
    let audio_file = temp_dir.path().join("test.wav");
    fs::write(&audio_file, b"mock audio data").unwrap();
    
    let mut tool = SpeechToTextTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    let input = SttInput {
        file_path: audio_file.to_string_lossy().to_string(),
        model: None,
        language: Some("en".to_string()),
        enable_vad: Some(true),
        output_format: Some("text".to_string()),
        include_timestamps: Some(true),
        include_confidence: Some(true),
    };
    
    let result = tool.process_audio_file(&input).await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert!(output.vad_segments.is_some());
    
    let vad_segments = output.vad_segments.unwrap();
    assert!(!vad_segments.is_empty());
    
    for segment in vad_segments {
        assert!(segment.start_ms < segment.end_ms);
        assert!(segment.confidence > 0.0 && segment.confidence <= 1.0);
    }
}

#[tokio::test]
async fn test_stt_tool_timestamps() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    // Create a mock audio file
    let audio_file = temp_dir.path().join("test.wav");
    fs::write(&audio_file, b"mock audio data").unwrap();
    
    let mut tool = SpeechToTextTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    let input = SttInput {
        file_path: audio_file.to_string_lossy().to_string(),
        model: None,
        language: Some("en".to_string()),
        enable_vad: Some(false),
        output_format: Some("text".to_string()),
        include_timestamps: Some(true),
        include_confidence: Some(true),
    };
    
    let result = tool.process_audio_file(&input).await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert!(output.timestamps.is_some());
    
    let timestamps = output.timestamps.unwrap();
    assert!(!timestamps.is_empty());
    
    for timestamp in timestamps {
        assert!(timestamp.start_ms < timestamp.end_ms);
        assert!(!timestamp.text.is_empty());
        assert!(timestamp.confidence > 0.0 && timestamp.confidence <= 1.0);
    }
}

#[tokio::test]
async fn test_stt_tool_language_detection() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    // Create a mock audio file
    let audio_file = temp_dir.path().join("test.wav");
    fs::write(&audio_file, b"mock audio data").unwrap();
    
    let mut tool = SpeechToTextTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    // Test auto-detection
    let input = SttInput {
        file_path: audio_file.to_string_lossy().to_string(),
        model: None,
        language: None, // Auto-detect
        enable_vad: Some(false),
        output_format: Some("text".to_string()),
        include_timestamps: Some(false),
        include_confidence: Some(false),
    };
    
    let result = tool.process_audio_file(&input).await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert!(!output.language.is_empty());
    
    // Test with specific language
    let input = SttInput {
        file_path: audio_file.to_string_lossy().to_string(),
        model: None,
        language: Some("fr".to_string()),
        enable_vad: Some(false),
        output_format: Some("text".to_string()),
        include_timestamps: Some(false),
        include_confidence: Some(false),
    };
    
    let result = tool.process_audio_file(&input).await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert_eq!(output.language, "fr");
}

#[tokio::test]
async fn test_stt_tool_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let mut tool = SpeechToTextTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    // Test with unsupported file format
    let input = SttInput {
        file_path: "test.txt".to_string(),
        model: None,
        language: None,
        enable_vad: None,
        output_format: None,
        include_timestamps: None,
        include_confidence: None,
    };
    
    let result = tool.process_audio_file(&input).await;
    assert!(result.is_err());
    
    // Test with file too large
    let large_file = temp_dir.path().join("large.wav");
    let large_data = vec![0u8; 1000000]; // 1MB
    fs::write(&large_file, &large_data).await.unwrap();
    
    let input = SttInput {
        file_path: large_file.to_string_lossy().to_string(),
        model: None,
        language: None,
        enable_vad: None,
        output_format: None,
        include_timestamps: None,
        include_confidence: None,
    };
    
    let result = tool.process_audio_file(&input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_stt_tool_configuration() {
    let temp_dir = TempDir::new().unwrap();
    
    // Test with custom configuration
    let config = SttConfig {
        models_dir: temp_dir.path().join("models"),
        default_model: "small".to_string(),
        vad_config: VadConfig {
            enabled: false,
            min_speech_duration: 200,
            max_silence_duration: 2000,
            sensitivity: 0.7,
            frame_size: 2048,
        },
        audio_config: AudioProcessingConfig {
            sample_rate: 22050,
            channels: 2,
            format: "wav,mp3".to_string(),
            max_duration: 600, // 10 minutes
            enable_preprocessing: false,
        },
        language_config: LanguageConfig {
            auto_detect: false,
            fallback_language: "es".to_string(),
            supported_languages: vec!["es".to_string(), "fr".to_string()],
            min_confidence: 0.8,
        },
    };
    
    let tool = SpeechToTextTool::new(config);
    assert!(tool.is_ok());
}

#[tokio::test]
async fn test_stt_tool_performance() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    // Create a mock audio file
    let audio_file = temp_dir.path().join("test.wav");
    fs::write(&audio_file, b"mock audio data").unwrap();
    
    let mut tool = SpeechToTextTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    let input = SttInput {
        file_path: audio_file.to_string_lossy().to_string(),
        model: None,
        language: Some("en".to_string()),
        enable_vad: Some(true),
        output_format: Some("text".to_string()),
        include_timestamps: Some(true),
        include_confidence: Some(true),
    };
    
    // Measure performance
    let start = std::time::Instant::now();
    let result = tool.process_audio_file(&input).await;
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    assert!(duration.as_millis() < 1000); // Should complete within 1 second (mock)
    
    let output = result.unwrap();
    assert!(output.processing_time_ms > 0);
}

fn create_test_config(temp_dir: &TempDir) -> SttConfig {
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).unwrap();
    
    // Create a mock model file
    fs::write(models_dir.join("base.bin"), b"mock model data").unwrap();
    
    SttConfig {
        models_dir,
        default_model: "base".to_string(),
        vad_config: VadConfig {
            enabled: true,
            min_speech_duration: 100,
            max_silence_duration: 1000,
            sensitivity: 0.5,
            frame_size: 1024,
        },
        audio_config: AudioProcessingConfig {
            sample_rate: 16000,
            channels: 1,
            format: "wav".to_string(),
            max_duration: 60,
            enable_preprocessing: true,
        },
        language_config: LanguageConfig {
            auto_detect: true,
            fallback_language: "en".to_string(),
            supported_languages: vec!["en".to_string()],
            min_confidence: 0.7,
        },
    }
}
