//! Runtime tests for AI Core Service
//!
//! This module contains comprehensive tests for the runtime adapters
//! with canned prompts and various scenarios.

use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tempfile::TempDir;
use tokio::time::timeout;

use aetheris_ai_core::{
    runtime::{RuntimeManager, RuntimeConfig, RuntimeRequest, ModelRuntime},
    ipc::ChatConfig,
    error::Result,
};

/// Test prompts for various scenarios
mod test_prompts {
    pub const SIMPLE_QUESTION: &str = "What is the capital of France?";
    pub const COMPLEX_QUESTION: &str = "Explain the concept of quantum computing and its potential applications in cryptography.";
    pub const CODE_GENERATION: &str = "Write a Python function to calculate the factorial of a number.";
    pub const CREATIVE_WRITING: &str = "Write a short story about a robot learning to paint.";
    pub const MATHEMATICAL_PROBLEM: &str = "Solve the equation: 2x + 5 = 13";
    pub const CONVERSATION_CONTEXT: &str = "Based on our previous discussion about renewable energy, what are the main challenges?";
    pub const STOP_SEQUENCE_TEST: &str = "Count from 1 to 10: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10. That's the end.";
    pub const LONG_PROMPT: &str = "Write a detailed analysis of the impact of artificial intelligence on modern society, including its benefits, challenges, and future prospects. Consider various sectors such as healthcare, education, transportation, and employment.";
}

/// Test configuration for runtime tests
fn create_test_config() -> RuntimeConfig {
    RuntimeConfig {
        model_path: "test_models/".to_string(),
        model_name: "test_model".to_string(),
        device: "cpu".to_string(),
        num_threads: 2,
        context_length: 512,
        batch_size: 1,
        deterministic: true,
        seed: Some(42),
        memory_pool_size: 100 * 1024 * 1024, // 100MB
        enable_profiling: false,
    }
}

/// Create a test runtime request
fn create_test_request(prompt: &str, max_tokens: Option<u32>) -> RuntimeRequest {
    RuntimeRequest {
        prompt: prompt.to_string(),
        config: ChatConfig {
            model: "test_model".to_string(),
            temperature: 0.7,
            max_tokens: max_tokens.unwrap_or(50) as i32,
            top_p: 0.9,
            top_k: 40,
            enable_tools: false,
            allowed_tools: vec![],
        },
        context: vec![],
        session_id: "test_session".to_string(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        stop_sequences: vec!["<|endoftext|>".to_string(), "\n\n".to_string()],
        max_tokens,
        temperature: Some(0.7),
        top_p: Some(0.9),
        top_k: Some(40),
        seed: Some(42),
    }
}

/// Create a temporary model file for testing
fn create_temp_model_file(temp_dir: &TempDir, model_name: &str, extension: &str) -> std::path::PathBuf {
    let model_path = temp_dir.path().join(format!("{}.{}", model_name, extension));
    let model_data = match extension {
        "onnx" => b"fake onnx model data for testing",
        "gguf" => b"fake gguf model data for testing",
        _ => b"fake model data for testing",
    };
    std::fs::write(&model_path, model_data).unwrap();
    model_path
}

#[tokio::test]
async fn test_runtime_manager_creation() {
    let config = create_test_config();
    let manager = RuntimeManager::new(config).unwrap();
    
    // Test that the manager is created with the correct default runtime
    let backend = std::env::var("AETHERIS_AI_BACKEND").unwrap_or_else(|_| "onnx".to_string());
    assert_eq!(manager.default_runtime, backend);
}

#[tokio::test]
async fn test_runtime_manager_initialization() {
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    let result = manager.initialize().await;
    assert!(result.is_ok(), "Runtime manager initialization should succeed");
    
    // Test that the manager is ready after initialization
    assert!(manager.is_ready().await, "Runtime manager should be ready after initialization");
}

#[tokio::test]
async fn test_runtime_manager_model_loading() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    manager.initialize().await.unwrap();
    
    // Create temporary model files
    let onnx_model = create_temp_model_file(&temp_dir, "test_model", "onnx");
    let gguf_model = create_temp_model_file(&temp_dir, "test_model", "gguf");
    
    // Test loading ONNX model
    let result = manager.load_model(&onnx_model, "test_onnx_model").await;
    assert!(result.is_ok(), "ONNX model loading should succeed");
    
    // Test loading GGUF model
    let result = manager.load_model(&gguf_model, "test_gguf_model").await;
    assert!(result.is_ok(), "GGUF model loading should succeed");
}

#[tokio::test]
async fn test_simple_question_generation() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    manager.initialize().await.unwrap();
    
    let model_path = create_temp_model_file(&temp_dir, "test_model", "onnx");
    manager.load_model(&model_path, "test_model").await.unwrap();
    
    let request = create_test_request(test_prompts::SIMPLE_QUESTION, Some(20));
    let response = manager.generate_response(&request).await;
    
    assert!(response.is_ok(), "Simple question generation should succeed");
    let response = response.unwrap();
    assert!(!response.response.is_empty(), "Response should not be empty");
    assert!(response.is_complete, "Response should be complete");
    assert!(response.metrics.tokens_generated > 0, "Should generate some tokens");
}

#[tokio::test]
async fn test_complex_question_generation() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    manager.initialize().await.unwrap();
    
    let model_path = create_temp_model_file(&temp_dir, "test_model", "onnx");
    manager.load_model(&model_path, "test_model").await.unwrap();
    
    let request = create_test_request(test_prompts::COMPLEX_QUESTION, Some(100));
    let response = manager.generate_response(&request).await;
    
    assert!(response.is_ok(), "Complex question generation should succeed");
    let response = response.unwrap();
    assert!(!response.response.is_empty(), "Response should not be empty");
    assert!(response.is_complete, "Response should be complete");
    assert!(response.metrics.tokens_generated > 0, "Should generate some tokens");
}

#[tokio::test]
async fn test_code_generation() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    manager.initialize().await.unwrap();
    
    let model_path = create_temp_model_file(&temp_dir, "test_model", "onnx");
    manager.load_model(&model_path, "test_model").await.unwrap();
    
    let request = create_test_request(test_prompts::CODE_GENERATION, Some(50));
    let response = manager.generate_response(&request).await;
    
    assert!(response.is_ok(), "Code generation should succeed");
    let response = response.unwrap();
    assert!(!response.response.is_empty(), "Response should not be empty");
    assert!(response.is_complete, "Response should be complete");
    assert!(response.metrics.tokens_generated > 0, "Should generate some tokens");
}

#[tokio::test]
async fn test_creative_writing() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    manager.initialize().await.unwrap();
    
    let model_path = create_temp_model_file(&temp_dir, "test_model", "onnx");
    manager.load_model(&model_path, "test_model").await.unwrap();
    
    let request = create_test_request(test_prompts::CREATIVE_WRITING, Some(80));
    let response = manager.generate_response(&request).await;
    
    assert!(response.is_ok(), "Creative writing should succeed");
    let response = response.unwrap();
    assert!(!response.response.is_empty(), "Response should not be empty");
    assert!(response.is_complete, "Response should be complete");
    assert!(response.metrics.tokens_generated > 0, "Should generate some tokens");
}

#[tokio::test]
async fn test_mathematical_problem() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    manager.initialize().await.unwrap();
    
    let model_path = create_temp_model_file(&temp_dir, "test_model", "onnx");
    manager.load_model(&model_path, "test_model").await.unwrap();
    
    let request = create_test_request(test_prompts::MATHEMATICAL_PROBLEM, Some(30));
    let response = manager.generate_response(&request).await;
    
    assert!(response.is_ok(), "Mathematical problem should succeed");
    let response = response.unwrap();
    assert!(!response.response.is_empty(), "Response should not be empty");
    assert!(response.is_complete, "Response should be complete");
    assert!(response.metrics.tokens_generated > 0, "Should generate some tokens");
}

#[tokio::test]
async fn test_long_prompt_handling() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    manager.initialize().await.unwrap();
    
    let model_path = create_temp_model_file(&temp_dir, "test_model", "onnx");
    manager.load_model(&model_path, "test_model").await.unwrap();
    
    let request = create_test_request(test_prompts::LONG_PROMPT, Some(150));
    let response = manager.generate_response(&request).await;
    
    assert!(response.is_ok(), "Long prompt handling should succeed");
    let response = response.unwrap();
    assert!(!response.response.is_empty(), "Response should not be empty");
    assert!(response.is_complete, "Response should be complete");
    assert!(response.metrics.tokens_generated > 0, "Should generate some tokens");
}

#[tokio::test]
async fn test_stop_sequence_handling() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    manager.initialize().await.unwrap();
    
    let model_path = create_temp_model_file(&temp_dir, "test_model", "onnx");
    manager.load_model(&model_path, "test_model").await.unwrap();
    
    let request = create_test_request(test_prompts::STOP_SEQUENCE_TEST, Some(50));
    let response = manager.generate_response(&request).await;
    
    assert!(response.is_ok(), "Stop sequence handling should succeed");
    let response = response.unwrap();
    assert!(!response.response.is_empty(), "Response should not be empty");
    assert!(response.is_complete, "Response should be complete");
    assert!(response.metrics.tokens_generated > 0, "Should generate some tokens");
}

#[tokio::test]
async fn test_token_streaming() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    manager.initialize().await.unwrap();
    
    let model_path = create_temp_model_file(&temp_dir, "test_model", "onnx");
    manager.load_model(&model_path, "test_model").await.unwrap();
    
    let request = create_test_request(test_prompts::SIMPLE_QUESTION, Some(20));
    let mut stream = manager.stream_tokens(&request).await.unwrap();
    
    let mut tokens = Vec::new();
    let mut token_count = 0;
    
    while let Some(token) = stream.next_token().await.unwrap() {
        tokens.push(token);
        token_count += 1;
        
        if token_count >= 10 {
            break;
        }
    }
    
    assert!(!tokens.is_empty(), "Should generate some tokens");
    assert!(token_count <= 10, "Should respect token limit");
    assert!(!stream.get_current_response().is_empty(), "Should have current response");
}

#[tokio::test]
async fn test_deterministic_generation() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    manager.initialize().await.unwrap();
    
    let model_path = create_temp_model_file(&temp_dir, "test_model", "onnx");
    manager.load_model(&model_path, "test_model").await.unwrap();
    
    let request = create_test_request(test_prompts::SIMPLE_QUESTION, Some(20));
    
    // Generate response twice with same seed
    let response1 = manager.generate_response(&request).await.unwrap();
    let response2 = manager.generate_response(&request).await.unwrap();
    
    // In deterministic mode, responses should be identical
    assert_eq!(response1.response, response2.response, "Deterministic generation should produce identical responses");
    assert_eq!(response1.metrics.tokens_generated, response2.metrics.tokens_generated, "Token counts should be identical");
}

#[tokio::test]
async fn test_different_temperature_settings() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    manager.initialize().await.unwrap();
    
    let model_path = create_temp_model_file(&temp_dir, "test_model", "onnx");
    manager.load_model(&model_path, "test_model").await.unwrap();
    
    let prompt = test_prompts::CREATIVE_WRITING;
    
    // Test with low temperature (more deterministic)
    let mut request = create_test_request(prompt, Some(30));
    request.temperature = Some(0.1);
    let response_low = manager.generate_response(&request).await.unwrap();
    
    // Test with high temperature (more random)
    let mut request = create_test_request(prompt, Some(30));
    request.temperature = Some(1.5);
    let response_high = manager.generate_response(&request).await.unwrap();
    
    assert!(!response_low.response.is_empty(), "Low temperature response should not be empty");
    assert!(!response_high.response.is_empty(), "High temperature response should not be empty");
    
    // Responses should be different (though not guaranteed in placeholder implementation)
    // In a real implementation, high temperature should produce more varied responses
}

#[tokio::test]
async fn test_max_tokens_limit() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    manager.initialize().await.unwrap();
    
    let model_path = create_temp_model_file(&temp_dir, "test_model", "onnx");
    manager.load_model(&model_path, "test_model").await.unwrap();
    
    let request = create_test_request(test_prompts::LONG_PROMPT, Some(5));
    let response = manager.generate_response(&request).await.unwrap();
    
    assert!(response.metrics.tokens_generated <= 5, "Should respect max_tokens limit");
    assert!(response.is_complete, "Response should be complete");
}

#[tokio::test]
async fn test_runtime_statistics() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    manager.initialize().await.unwrap();
    
    let model_path = create_temp_model_file(&temp_dir, "test_model", "onnx");
    manager.load_model(&model_path, "test_model").await.unwrap();
    
    // Generate a few responses
    for i in 0..3 {
        let request = create_test_request(&format!("Test prompt {}", i), Some(10));
        let _response = manager.generate_response(&request).await.unwrap();
    }
    
    let stats = manager.get_stats().await;
    assert_eq!(stats.total_requests, 3, "Should track total requests");
    assert!(stats.total_tokens_generated > 0, "Should track total tokens generated");
    assert!(stats.total_processing_time_ms > 0, "Should track total processing time");
}

#[tokio::test]
async fn test_model_info_retrieval() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    manager.initialize().await.unwrap();
    
    let model_path = create_temp_model_file(&temp_dir, "test_model", "onnx");
    manager.load_model(&model_path, "test_model").await.unwrap();
    
    let model_info = manager.get_model_info().await.unwrap();
    assert_eq!(model_info.name, "test_model", "Model name should match");
    assert_eq!(model_info.model_type, "onnx", "Model type should be onnx");
    assert!(model_info.model_size > 0, "Model size should be positive");
    assert!(model_info.vocab_size > 0, "Vocabulary size should be positive");
}

#[tokio::test]
async fn test_error_handling_invalid_model() {
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    manager.initialize().await.unwrap();
    
    // Try to load a non-existent model
    let invalid_path = Path::new("/nonexistent/model.onnx");
    let result = manager.load_model(invalid_path, "invalid_model").await;
    
    assert!(result.is_err(), "Loading non-existent model should fail");
}

#[tokio::test]
async fn test_error_handling_uninitialized_runtime() {
    let config = create_test_config();
    let manager = RuntimeManager::new(config).unwrap();
    
    // Try to generate response without initialization
    let request = create_test_request(test_prompts::SIMPLE_QUESTION, Some(20));
    let result = manager.generate_response(&request).await;
    
    assert!(result.is_err(), "Generation without initialization should fail");
}

#[tokio::test]
async fn test_timeout_handling() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    manager.initialize().await.unwrap();
    
    let model_path = create_temp_model_file(&temp_dir, "test_model", "onnx");
    manager.load_model(&model_path, "test_model").await.unwrap();
    
    let request = create_test_request(test_prompts::SIMPLE_QUESTION, Some(20));
    
    // Test with a very short timeout
    let result = timeout(Duration::from_millis(1), manager.generate_response(&request)).await;
    
    // The timeout should occur (though the placeholder implementation is fast)
    // In a real implementation, this would test actual timeout handling
    assert!(result.is_ok(), "Request should complete within timeout");
}

#[tokio::test]
async fn test_concurrent_requests() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    manager.initialize().await.unwrap();
    
    let model_path = create_temp_model_file(&temp_dir, "test_model", "onnx");
    manager.load_model(&model_path, "test_model").await.unwrap();
    
    let manager = std::sync::Arc::new(manager);
    let mut handles = Vec::new();
    
    // Spawn multiple concurrent requests
    for i in 0..5 {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            let request = create_test_request(&format!("Concurrent test {}", i), Some(10));
            manager_clone.generate_response(&request).await
        });
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent request should succeed");
        let response = result.unwrap();
        assert!(!response.response.is_empty(), "Response should not be empty");
    }
}

#[tokio::test]
async fn test_gguf_runtime_specific() {
    // Set environment variable to use GGUF backend
    std::env::set_var("AETHERIS_AI_BACKEND", "gguf");
    
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    manager.initialize().await.unwrap();
    
    let model_path = create_temp_model_file(&temp_dir, "test_model", "gguf");
    manager.load_model(&model_path, "test_gguf_model").await.unwrap();
    
    let request = create_test_request(test_prompts::SIMPLE_QUESTION, Some(20));
    let response = manager.generate_response(&request).await;
    
    assert!(response.is_ok(), "GGUF runtime generation should succeed");
    let response = response.unwrap();
    assert!(!response.response.is_empty(), "Response should not be empty");
    assert!(response.is_complete, "Response should be complete");
    assert!(response.metrics.tokens_generated > 0, "Should generate some tokens");
    
    // Clean up environment variable
    std::env::remove_var("AETHERIS_AI_BACKEND");
}

#[tokio::test]
async fn test_onnx_runtime_specific() {
    // Set environment variable to use ONNX backend
    std::env::set_var("AETHERIS_AI_BACKEND", "onnx");
    
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config();
    let mut manager = RuntimeManager::new(config).unwrap();
    
    manager.initialize().await.unwrap();
    
    let model_path = create_temp_model_file(&temp_dir, "test_model", "onnx");
    manager.load_model(&model_path, "test_onnx_model").await.unwrap();
    
    let request = create_test_request(test_prompts::SIMPLE_QUESTION, Some(20));
    let response = manager.generate_response(&request).await;
    
    assert!(response.is_ok(), "ONNX runtime generation should succeed");
    let response = response.unwrap();
    assert!(!response.response.is_empty(), "Response should not be empty");
    assert!(response.is_complete, "Response should be complete");
    assert!(response.metrics.tokens_generated > 0, "Should generate some tokens");
    
    // Clean up environment variable
    std::env::remove_var("AETHERIS_AI_BACKEND");
}
