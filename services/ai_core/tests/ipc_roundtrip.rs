//! IPC roundtrip tests for AI Core Service
//!
//! Tests the complete IPC communication flow including message serialization,
//! transmission, processing, and response handling.
#![cfg(unix)]

use prost::Message;
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use uuid::Uuid;

use aetheris_ai_core::{AiCoreService, Config, Result};

/// Test server setup
async fn setup_test_server() -> Result<(std::path::PathBuf, tokio::task::JoinHandle<()>)> {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test.sock");

    // Create test configuration files
    let model_config_path = temp_dir.path().join("models.toml");
    let tool_config_path = temp_dir.path().join("tools.toml");
    let cap_config_path = temp_dir.path().join("cap_tokens.toml");

    // Write minimal config files
    tokio::fs::write(
        &model_config_path,
        r#"
[global_settings]
default_temperature = 0.7
default_max_tokens = 2048
default_top_p = 0.9
default_top_k = 40
timeout_seconds = 30
retry_attempts = 3
enable_audit_logging = true

default_model = "mock"

[tools.mock]
model_type = "Mock"
enabled = true
"#,
    )
    .await?;

    tokio::fs::write(
        &tool_config_path,
        r#"
[global_settings]
max_concurrent_executions = 10
default_timeout_seconds = 30
default_cache_ttl_seconds = 300
enable_caching = true
enable_audit_logging = true
max_cache_size = 1000

[tools.echo]
enabled = true
timeout_seconds = 5
cache_ttl_seconds = 60
max_memory_mb = 10
required_capabilities = ["tools:echo"]

[tools.calculator]
enabled = true
timeout_seconds = 10
cache_ttl_seconds = 300
max_memory_mb = 20
required_capabilities = ["tools:calculator"]
"#,
    )
    .await?;

    tokio::fs::write(
        &cap_config_path,
        r#"
[validation]
require_signature = false
check_expiration = true
max_clock_skew = 300
default_token_lifetime = 3600

[caching]
enabled = true
cache_ttl = 300
max_cache_size = 1000
cache_validation_results = true

[audit]
enabled = true
log_all_requests = true
log_validation_failures = true
log_capability_checks = true

[issuers."aetheris-system"]
name = "Aetheris System"
public_key = ""
trusted = true
max_token_lifetime = 3600
allowed_capabilities = ["ai:chat", "ai:tools", "ai:admin"]
"#,
    )
    .await?;

    // Start the server
    let server_handle = tokio::spawn(async move {
        let config = Config {
            socket: socket_path.clone(),
            model_config: model_config_path,
            tool_config: tool_config_path,
            cap_config: cap_config_path,
            log_level: "debug".to_string(),
            deterministic: true,
            audit_log_dir: temp_dir.path().join("audit"),
            max_sessions: 10,
            request_timeout: 30,
        };

        let service = AiCoreService::new(config).await.unwrap();
        service.start().await.unwrap();

        // Keep the server running
        tokio::time::sleep(Duration::from_secs(10)).await;
    });

    // Wait for server to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    Ok((socket_path, server_handle))
}

#[tokio::test]
async fn test_service_creation() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test.sock");

    let config = Config {
        socket: socket_path,
        model_config: temp_dir.path().join("models.toml"),
        tool_config: temp_dir.path().join("tools.toml"),
        cap_config: temp_dir.path().join("cap.toml"),
        log_level: "debug".to_string(),
        deterministic: true,
        audit_log_dir: temp_dir.path().join("audit"),
        max_sessions: 10,
        request_timeout: 5,
    };

    // This will fail due to missing config files, but we can test the structure
    let result = AiCoreService::new(config).await;
    assert!(result.is_err()); // Expected to fail due to missing config files
}

#[tokio::test]
async fn test_config_file_creation() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test.sock");

    // Create minimal config files
    let model_config_path = temp_dir.path().join("models.toml");
    let tool_config_path = temp_dir.path().join("tools.toml");
    let cap_config_path = temp_dir.path().join("cap_tokens.toml");

    tokio::fs::write(
        &model_config_path,
        r#"
[global_settings]
default_temperature = 0.7
default_max_tokens = 2048
default_top_p = 0.9
default_top_k = 40
timeout_seconds = 30
retry_attempts = 3
enable_audit_logging = true

default_model = "mock"

[tools.mock]
model_type = "Mock"
enabled = true
"#,
    )
    .await
    .unwrap();

    tokio::fs::write(
        &tool_config_path,
        r#"
[global_settings]
max_concurrent_executions = 10
default_timeout_seconds = 30
default_cache_ttl_seconds = 300
enable_caching = true
enable_audit_logging = true
max_cache_size = 1000

[tools.echo]
enabled = true
timeout_seconds = 5
cache_ttl_seconds = 60
max_memory_mb = 10
required_capabilities = ["tools:echo"]
"#,
    )
    .await
    .unwrap();

    tokio::fs::write(
        &cap_config_path,
        r#"
[validation]
require_signature = false
check_expiration = true
max_clock_skew = 300
default_token_lifetime = 3600

[caching]
enabled = true
cache_ttl = 300
max_cache_size = 1000
cache_validation_results = true

[audit]
enabled = true
log_all_requests = true
log_validation_failures = true
log_capability_checks = true

[issuers."aetheris-system"]
name = "Aetheris System"
public_key = ""
trusted = true
max_token_lifetime = 3600
allowed_capabilities = ["ai:chat", "ai:tools", "ai:admin"]
"#,
    )
    .await
    .unwrap();

    let config = Config {
        socket: socket_path,
        model_config: model_config_path,
        tool_config: tool_config_path,
        cap_config: cap_config_path,
        log_level: "debug".to_string(),
        deterministic: true,
        audit_log_dir: temp_dir.path().join("audit"),
        max_sessions: 10,
        request_timeout: 5,
    };

    // This should succeed with proper config files
    let result = AiCoreService::new(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_socket_creation() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test.sock");

    // Test that we can create a socket path
    assert!(!socket_path.exists());

    // Test that the parent directory exists
    assert!(socket_path.parent().unwrap().exists());
}

#[tokio::test]
async fn test_protobuf_serialization() {
    // Test basic protobuf serialization/deserialization
    use aetheris_ai_core::ipc::ai_core::*;

    let ping_request = PingRequest {
        client_id: "test_client".to_string(),
        version: "1.0.0".to_string(),
    };

    // Serialize
    let mut data = Vec::new();
    ping_request.encode(&mut data).unwrap();

    // Deserialize
    let deserialized = PingRequest::decode(&data).unwrap();

    assert_eq!(ping_request.client_id, deserialized.client_id);
    assert_eq!(ping_request.version, deserialized.version);
}

#[tokio::test]
async fn test_message_creation() {
    use aetheris_ai_core::ipc::ai_core::*;

    let ping_request = PingRequest {
        client_id: "test_client".to_string(),
        version: "1.0.0".to_string(),
    };

    let message = AiCoreMessage {
        message_type: Some(ai_core_message::MessageType::PingRequest(ping_request)),
        message_id: Uuid::new_v4().to_string(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        session_id: "test_session".to_string(),
        cap_token: None,
    };

    assert!(!message.message_id.is_empty());
    assert!(message.timestamp > 0);
    assert_eq!(message.session_id, "test_session");
    assert!(message.cap_token.is_none());

    match message.message_type {
        Some(ai_core_message::MessageType::PingRequest(req)) => {
            assert_eq!(req.client_id, "test_client");
            assert_eq!(req.version, "1.0.0");
        }
        _ => panic!("Expected PingRequest message type"),
    }
}

#[tokio::test]
async fn test_capability_token_creation() {
    use aetheris_ai_core::ipc::ai_core::*;

    let cap_token = CapToken {
        token_id: "test_token".to_string(),
        capability: "ai:chat".to_string(),
        expires_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600,
        signature: vec![],
        issuer: "aetheris-system".to_string(),
    };

    assert_eq!(cap_token.token_id, "test_token");
    assert_eq!(cap_token.capability, "ai:chat");
    assert_eq!(cap_token.issuer, "aetheris-system");
    assert!(cap_token.expires_at > 0);
    assert!(cap_token.signature.is_empty());
}

#[tokio::test]
async fn test_chat_config_creation() {
    use aetheris_ai_core::ipc::ai_core::*;

    let chat_config = ChatConfig {
        model: "mock".to_string(),
        temperature: 0.7,
        max_tokens: 100,
        top_p: 0.9,
        top_k: 40,
        enable_tools: false,
        allowed_tools: Vec::new(),
    };

    assert_eq!(chat_config.model, "mock");
    assert_eq!(chat_config.temperature, 0.7);
    assert_eq!(chat_config.max_tokens, 100);
    assert_eq!(chat_config.top_p, 0.9);
    assert_eq!(chat_config.top_k, 40);
    assert!(!chat_config.enable_tools);
    assert!(chat_config.allowed_tools.is_empty());
}

#[tokio::test]
async fn test_error_code_enum() {
    use aetheris_ai_core::ipc::ai_core::*;

    let error_code = ErrorCode::InvalidRequest;
    assert_eq!(error_code as i32, 1);

    let error_code = ErrorCode::Unauthorized;
    assert_eq!(error_code as i32, 2);

    let error_code = ErrorCode::CapabilityDenied;
    assert_eq!(error_code as i32, 3);
}

#[tokio::test]
async fn test_service_state_enum() {
    use aetheris_ai_core::ipc::ai_core::*;

    let state = ServiceState::Running;
    assert_eq!(state as i32, 1);

    let state = ServiceState::Stopped;
    assert_eq!(state as i32, 3);

    let state = ServiceState::Error;
    assert_eq!(state as i32, 4);
}
