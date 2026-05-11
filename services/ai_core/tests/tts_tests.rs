//! Tests for Text-to-Speech functionality
//! 
//! This module contains comprehensive tests for the TTS tool integration
//! with the AI Core Service, including audio synthesis, streaming, and Opus encoding.

use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;

use aetheris_ai_core::tools::tts::{
    TextToSpeechTool, TtsConfig, TtsInput, TtsOutput, 
    VoiceConfig, AudioOutputConfig, StreamingConfig, OpusConfig,
    AudioFormat, default_tts_config, execute_text_to_speech
};

#[tokio::test]
async fn test_tts_tool_creation() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let tool = TextToSpeechTool::new(config);
    assert!(tool.is_ok());
}

#[tokio::test]
async fn test_tts_tool_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let mut tool = TextToSpeechTool::new(config).unwrap();
    let result = tool.initialize(None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tts_tool_synthesize_text() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let mut tool = TextToSpeechTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    let input = TtsInput {
        text: "Hello, this is a test.".to_string(),
        voice: Some("en_female_1".to_string()),
        language: Some("en".to_string()),
        speed: Some(1.0),
        pitch: Some(1.0),
        volume: Some(0.8),
        output_format: Some(AudioFormat::S16LE),
        enable_streaming: Some(false),
        output_device: None,
        save_to_file: None,
    };
    
    let result = tool.synthesize_text(&input).await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert!(output.duration_ms > 0);
    assert!(output.processing_time_ms > 0);
    assert_eq!(output.voice_used, "en_female_1");
    assert_eq!(output.language_used, "en");
}

#[tokio::test]
async fn test_tts_tool_list_voices() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let mut tool = TextToSpeechTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    let voices = tool.list_available_voices().await.unwrap();
    assert!(!voices.is_empty());
    
    // Check that we have the expected mock voices
    let voice_ids: Vec<&str> = voices.iter().map(|v| v.id.as_str()).collect();
    assert!(voice_ids.contains(&"en_female_1"));
    assert!(voice_ids.contains(&"en_male_1"));
    assert!(voice_ids.contains(&"es_female_1"));
}

#[tokio::test]
async fn test_tts_tool_validation() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let mut tool = TextToSpeechTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    // Test empty text
    let input = TtsInput {
        text: "".to_string(),
        voice: None,
        language: None,
        speed: None,
        pitch: None,
        volume: None,
        output_format: None,
        enable_streaming: None,
        output_device: None,
        save_to_file: None,
    };
    
    let result = tool.synthesize_text(&input).await;
    assert!(result.is_err());
    
    // Test invalid speed
    let input = TtsInput {
        text: "Hello".to_string(),
        voice: None,
        language: None,
        speed: Some(3.0), // Invalid speed
        pitch: None,
        volume: None,
        output_format: None,
        enable_streaming: None,
        output_device: None,
        save_to_file: None,
    };
    
    let result = tool.synthesize_text(&input).await;
    assert!(result.is_err());
    
    // Test invalid pitch
    let input = TtsInput {
        text: "Hello".to_string(),
        voice: None,
        language: None,
        speed: None,
        pitch: Some(3.0), // Invalid pitch
        volume: None,
        output_format: None,
        enable_streaming: None,
        output_device: None,
        save_to_file: None,
    };
    
    let result = tool.synthesize_text(&input).await;
    assert!(result.is_err());
    
    // Test invalid volume
    let input = TtsInput {
        text: "Hello".to_string(),
        voice: None,
        language: None,
        speed: None,
        pitch: None,
        volume: Some(2.0), // Invalid volume
        output_format: None,
        enable_streaming: None,
        output_device: None,
        save_to_file: None,
    };
    
    let result = tool.synthesize_text(&input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tts_tool_audio_formats() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let mut tool = TextToSpeechTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    let formats = vec![
        AudioFormat::S16LE,
        AudioFormat::F32LE,
        AudioFormat::Opus,
        AudioFormat::Wav,
    ];
    
    for format in formats {
        let input = TtsInput {
            text: "Test audio format".to_string(),
            voice: None,
            language: None,
            speed: None,
            pitch: None,
            volume: None,
            output_format: Some(format),
            enable_streaming: None,
            output_device: None,
            save_to_file: None,
        };
        
        let result = tool.synthesize_text(&input).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_tts_tool_save_to_file() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let mut tool = TextToSpeechTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    let output_file = temp_dir.path().join("test_output.wav");
    
    let input = TtsInput {
        text: "Hello, this is a test file.".to_string(),
        voice: None,
        language: None,
        speed: None,
        pitch: None,
        volume: None,
        output_format: Some(AudioFormat::Wav),
        enable_streaming: None,
        output_device: None,
        save_to_file: Some(output_file.to_string_lossy().to_string()),
    };
    
    let result = tool.synthesize_text(&input).await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert!(output.file_path.is_some());
    assert!(output.audio_data.is_some());
    
    // Verify file was created
    assert!(output_file.exists());
}

#[tokio::test]
async fn test_tts_tool_streaming() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let mut tool = TextToSpeechTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    let input = TtsInput {
        text: "This is a streaming test with longer text to ensure proper chunking.".to_string(),
        voice: None,
        language: None,
        speed: None,
        pitch: None,
        volume: None,
        output_format: Some(AudioFormat::Opus),
        enable_streaming: Some(true),
        output_device: None,
        save_to_file: None,
    };
    
    let result = tool.synthesize_text(&input).await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert!(output.streaming_session_id.is_some());
}

#[tokio::test]
async fn test_tts_tool_voice_parameters() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let mut tool = TextToSpeechTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    // Test different voice parameters
    let test_cases = vec![
        (1.5, 1.2, 0.9), // Fast, high pitch, high volume
        (0.8, 0.9, 0.5), // Slow, low pitch, low volume
        (1.0, 1.0, 0.8), // Normal parameters
    ];
    
    for (speed, pitch, volume) in test_cases {
        let input = TtsInput {
            text: "Testing voice parameters".to_string(),
            voice: None,
            language: None,
            speed: Some(speed),
            pitch: Some(pitch),
            volume: Some(volume),
            output_format: Some(AudioFormat::S16LE),
            enable_streaming: None,
            output_device: None,
            save_to_file: None,
        };
        
        let result = tool.synthesize_text(&input).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_tts_tool_language_support() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let mut tool = TextToSpeechTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    let languages = vec!["en", "es", "fr", "de"];
    
    for language in languages {
        let input = TtsInput {
            text: "Hello world".to_string(),
            voice: None,
            language: Some(language.to_string()),
            speed: None,
            pitch: None,
            volume: None,
            output_format: Some(AudioFormat::S16LE),
            enable_streaming: None,
            output_device: None,
            save_to_file: None,
        };
        
        let result = tool.synthesize_text(&input).await;
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert_eq!(output.language_used, language);
    }
}

#[tokio::test]
async fn test_tts_tool_opus_encoding() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let mut tool = TextToSpeechTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    let input = TtsInput {
        text: "Testing Opus encoding for efficient audio compression.".to_string(),
        voice: None,
        language: None,
        speed: None,
        pitch: None,
        volume: None,
        output_format: Some(AudioFormat::Opus),
        enable_streaming: None,
        output_device: None,
        save_to_file: None,
    };
    
    let result = tool.synthesize_text(&input).await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert_eq!(output.audio_format, AudioFormat::Opus);
}

#[tokio::test]
async fn test_tts_tool_wav_output() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let mut tool = TextToSpeechTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    let input = TtsInput {
        text: "Testing WAV output format.".to_string(),
        voice: None,
        language: None,
        speed: None,
        pitch: None,
        volume: None,
        output_format: Some(AudioFormat::Wav),
        enable_streaming: None,
        output_device: None,
        save_to_file: None,
    };
    
    let result = tool.synthesize_text(&input).await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert_eq!(output.audio_format, AudioFormat::Wav);
}

#[tokio::test]
async fn test_tts_tool_performance() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let mut tool = TextToSpeechTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    let input = TtsInput {
        text: "Performance test with moderate length text to measure synthesis time.".to_string(),
        voice: None,
        language: None,
        speed: None,
        pitch: None,
        volume: None,
        output_format: Some(AudioFormat::S16LE),
        enable_streaming: None,
        output_device: None,
        save_to_file: None,
    };
    
    // Measure performance
    let start = std::time::Instant::now();
    let result = tool.synthesize_text(&input).await;
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    assert!(duration.as_millis() < 5000); // Should complete within 5 seconds (mock)
    
    let output = result.unwrap();
    assert!(output.processing_time_ms > 0);
}

#[tokio::test]
async fn test_tts_tool_configuration() {
    let temp_dir = TempDir::new().unwrap();
    
    // Test with custom configuration
    let config = TtsConfig {
        models_dir: temp_dir.path().join("models"),
        default_model: "custom-tts".to_string(),
        voice_config: VoiceConfig {
            default_voice: "custom_voice".to_string(),
            speed: 1.2,
            pitch: 1.1,
            volume: 0.9,
            emotion: Some("happy".to_string()),
            language: "es".to_string(),
        },
        audio_config: AudioOutputConfig {
            sample_rate: 44100,
            channels: 2,
            format: AudioFormat::F32LE,
            output_device: Some("custom_device".to_string()),
            enable_streaming: false,
            buffer_size: 8192,
        },
        streaming_config: StreamingConfig {
            enabled: false,
            chunk_size: 2048,
            latency_ms: 50,
            real_time: false,
        },
        opus_config: OpusConfig {
            enabled: true,
            bitrate: 128,
            quality: 10,
            frame_size_ms: 10,
            enable_vbr: false,
        },
    };
    
    let tool = TextToSpeechTool::new(config);
    assert!(tool.is_ok());
}

#[tokio::test]
async fn test_execute_text_to_speech() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let mut tool = TextToSpeechTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    let input = TtsInput {
        text: "Hello, this is a test.".to_string(),
        voice: Some("en_female_1".to_string()),
        language: Some("en".to_string()),
        speed: Some(1.0),
        pitch: Some(1.0),
        volume: Some(0.8),
        output_format: Some(AudioFormat::S16LE),
        enable_streaming: Some(false),
        output_device: None,
        save_to_file: None,
    };
    
    let input_cbor = serde_cbor::to_vec(&input).unwrap();
    let result = execute_text_to_speech(&tool, &input_cbor).await;
    
    assert!(result.is_ok());
    let tool_result = result.unwrap();
    assert!(tool_result.success);
    assert!(tool_result.data.is_some());
    assert!(tool_result.error.is_none());
    assert!(tool_result.execution_time_ms > 0);
}

#[tokio::test]
async fn test_tts_tool_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    let mut tool = TextToSpeechTool::new(config).unwrap();
    tool.initialize(None).await.unwrap();
    
    // Test text too long
    let long_text = "a".repeat(10001); // Exceeds 10000 character limit
    let input = TtsInput {
        text: long_text,
        voice: None,
        language: None,
        speed: None,
        pitch: None,
        volume: None,
        output_format: None,
        enable_streaming: None,
        output_device: None,
        save_to_file: None,
    };
    
    let result = tool.synthesize_text(&input).await;
    assert!(result.is_err());
}

fn create_test_config(temp_dir: &TempDir) -> TtsConfig {
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).unwrap();
    
    // Create a mock model file
    fs::write(models_dir.join("coqui-tts.bin"), b"mock model data").unwrap();
    
    TtsConfig {
        models_dir,
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
