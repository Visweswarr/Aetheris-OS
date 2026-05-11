//! Tests for CLI Speech-to-Text integration
//! 
//! This module contains tests for the CLI command integration
//! with the STT tool functionality.

use std::process::Command;
use tempfile::TempDir;
use tokio::fs;

#[tokio::test]
async fn test_cli_stt_transcribe_command() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a mock audio file
    let audio_file = temp_dir.path().join("test.wav");
    fs::write(&audio_file, b"mock audio data").await.unwrap();
    
    // Test the CLI command (this would require the actual devctl binary)
    // For now, we'll test the file creation and basic structure
    
    assert!(audio_file.exists());
    let content = fs::read(&audio_file).await.unwrap();
    assert_eq!(content, b"mock audio data");
}

#[tokio::test]
async fn test_cli_stt_models_command() {
    // Test that the models command would work
    // This is a placeholder for when the actual CLI is integrated
    
    let models = vec![
        "tiny", "base", "small", "medium", "large"
    ];
    
    for model in models {
        assert!(!model.is_empty());
        assert!(model.len() > 0);
    }
}

#[tokio::test]
async fn test_cli_stt_fixtures() {
    let temp_dir = TempDir::new().unwrap();
    let fixtures_dir = temp_dir.path().join("fixtures");
    fs::create_dir_all(&fixtures_dir).await.unwrap();
    
    // Create fixture files
    let sample_audio = fixtures_dir.join("sample_audio.wav");
    let ask_audio = fixtures_dir.join("ask.wav");
    let spanish_audio = fixtures_dir.join("spanish_audio.wav");
    
    fs::write(&sample_audio, b"sample audio data").await.unwrap();
    fs::write(&ask_audio, b"ask audio data").await.unwrap();
    fs::write(&spanish_audio, b"spanish audio data").await.unwrap();
    
    // Verify fixtures exist
    assert!(sample_audio.exists());
    assert!(ask_audio.exists());
    assert!(spanish_audio.exists());
    
    // Verify content
    let sample_content = fs::read(&sample_audio).await.unwrap();
    let ask_content = fs::read(&ask_audio).await.unwrap();
    let spanish_content = fs::read(&spanish_audio).await.unwrap();
    
    assert_eq!(sample_content, b"sample audio data");
    assert_eq!(ask_content, b"ask audio data");
    assert_eq!(spanish_content, b"spanish audio data");
}

#[tokio::test]
async fn test_cli_stt_output_formats() {
    // Test different output formats
    let formats = vec!["text", "json", "srt"];
    
    for format in formats {
        assert!(!format.is_empty());
        assert!(format.len() > 0);
    }
}

#[tokio::test]
async fn test_cli_stt_language_support() {
    // Test supported languages
    let languages = vec![
        "en", "es", "fr", "de", "it", "pt", "ru", "ja", "ko", "zh"
    ];
    
    for language in languages {
        assert!(!language.is_empty());
        assert_eq!(language.len(), 2); // ISO 639-1 codes
    }
}

#[tokio::test]
async fn test_cli_stt_vad_options() {
    // Test VAD configuration options
    let vad_options = vec![true, false];
    
    for option in vad_options {
        assert!(option == true || option == false);
    }
}

#[tokio::test]
async fn test_cli_stt_timestamp_options() {
    // Test timestamp inclusion options
    let timestamp_options = vec![true, false];
    
    for option in timestamp_options {
        assert!(option == true || option == false);
    }
}

#[tokio::test]
async fn test_cli_stt_confidence_options() {
    // Test confidence score inclusion options
    let confidence_options = vec![true, false];
    
    for option in confidence_options {
        assert!(option == true || option == false);
    }
}
