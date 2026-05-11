//! Comprehensive tests for the Tooling Framework module

use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;
use serde_json::Value;

use aetheris_ai_core::tools::{
    ToolRegistry, Tool, ToolResult, ToolImplementationType, ToolRegistryStats
};
use aetheris_ai_core::cap::CapTokenManager;

/// Test helper to create a temporary tool registry
async fn create_test_tool_registry() -> (TempDir, ToolRegistry) {
    let temp_dir = TempDir::new().unwrap();
    let tools_dir = temp_dir.path().join("tools");
    tokio::fs::create_dir_all(&tools_dir).await.unwrap();
    
    let cap_token_manager = std::sync::Arc::new(
        CapTokenManager::new(&temp_dir.path().join("cap.toml")).await.unwrap()
    );
    let registry = ToolRegistry::new(tools_dir, cap_token_manager);
    registry.initialize().await.unwrap();
    
    (temp_dir, registry)
}

#[tokio::test]
async fn test_tool_registry_initialization() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    let tools = registry.list_tools().await.unwrap();
    assert!(!tools.is_empty());
    
    // Check that default tools are loaded
    let tool_ids: Vec<&str> = tools.iter().map(|t| t.id.as_str()).collect();
    assert!(tool_ids.contains(&"open_app"));
    assert!(tool_ids.contains(&"search_files"));
    assert!(tool_ids.contains(&"create_note"));
}

#[tokio::test]
async fn test_tool_registration() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    let tool = Tool {
        id: "test_tool".to_string(),
        name: "Test Tool".to_string(),
        description: "A test tool".to_string(),
        version: "1.0.0".to_string(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "param1": {"type": "string"}
            },
            "required": ["param1"]
        }),
        required_capabilities: vec!["test.capability".to_string()],
        category: "test".to_string(),
        tags: vec!["test".to_string()],
        implementation_type: ToolImplementationType::Builtin,
        timeout_seconds: 30,
        requires_confirmation: false,
        metadata: HashMap::new(),
    };
    
    registry.register_tool(tool).await.unwrap();
    
    let retrieved_tool = registry.get_tool("test_tool").await.unwrap();
    assert_eq!(retrieved_tool.name, "Test Tool");
}

#[tokio::test]
async fn test_tool_execution_open_app() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    let parameters = serde_json::json!({
        "app_name": "notepad"
    });
    
    let result = registry.execute_tool("open_app", &parameters.to_string()).await.unwrap();
    assert!(result.success);
    assert_eq!(result.tool_id, "open_app");
    assert!(result.output.is_some());
    
    // Verify CBOR output can be decoded
    let output_data = result.output.unwrap();
    let decoded: Value = serde_cbor::from_slice(&output_data).unwrap();
    assert_eq!(decoded["app_name"], "notepad");
    assert_eq!(decoded["success"], true);
}

#[tokio::test]
async fn test_tool_execution_search_files() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    let parameters = serde_json::json!({
        "query": "*.txt",
        "directory": "/tmp",
        "recursive": true
    });
    
    let result = registry.execute_tool("search_files", &parameters.to_string()).await.unwrap();
    assert!(result.success);
    assert_eq!(result.tool_id, "search_files");
    assert!(result.output.is_some());
    
    // Verify CBOR output can be decoded
    let output_data = result.output.unwrap();
    let decoded: Value = serde_cbor::from_slice(&output_data).unwrap();
    assert_eq!(decoded["query"], "*.txt");
    assert_eq!(decoded["directory"], "/tmp");
    assert_eq!(decoded["recursive"], true);
    assert!(decoded["files"].is_array());
}

#[tokio::test]
async fn test_tool_execution_create_note() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    let parameters = serde_json::json!({
        "title": "Test Note",
        "content": "This is a test note",
        "format": "markdown",
        "tags": ["test", "note"]
    });
    
    let result = registry.execute_tool("create_note", &parameters.to_string()).await.unwrap();
    assert!(result.success);
    assert_eq!(result.tool_id, "create_note");
    assert!(result.output.is_some());
    
    // Verify CBOR output can be decoded
    let output_data = result.output.unwrap();
    let decoded: Value = serde_cbor::from_slice(&output_data).unwrap();
    assert_eq!(decoded["title"], "Test Note");
    assert_eq!(decoded["content"], "This is a test note");
    assert_eq!(decoded["format"], "markdown");
    assert!(decoded["file_path"].is_string());
}

#[tokio::test]
async fn test_parameter_validation_success() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Test with valid parameters
    let valid_parameters = serde_json::json!({
        "app_name": "notepad",
        "args": ["--help"]
    });
    
    let result = registry.execute_tool("open_app", &valid_parameters.to_string()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_parameter_validation_missing_required() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Test with missing required parameter
    let invalid_parameters = serde_json::json!({
        "args": ["--help"]
        // Missing app_name
    });
    
    let result = registry.execute_tool("open_app", &invalid_parameters.to_string()).await;
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert!(error.to_string().contains("validation failed"));
}

#[tokio::test]
async fn test_parameter_validation_wrong_type() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Test with wrong parameter type
    let invalid_parameters = serde_json::json!({
        "app_name": 123, // Should be string
        "args": "not_an_array" // Should be array
    });
    
    let result = registry.execute_tool("open_app", &invalid_parameters.to_string()).await;
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert!(error.to_string().contains("validation failed"));
}

#[tokio::test]
async fn test_tool_capability_check_without_token() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Test without capability token
    let result = registry.check_tool_capabilities("open_app", None).await.unwrap();
    assert!(!result.allowed);
    assert!(!result.missing_capabilities.is_empty());
    assert!(result.missing_capabilities.contains(&"system.apps.execute".to_string()));
}

#[tokio::test]
async fn test_tool_capability_check_with_token() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Test with capability token (mock)
    let result = registry.check_tool_capabilities("open_app", Some("mock_token")).await.unwrap();
    assert!(result.allowed);
    assert!(result.missing_capabilities.is_empty());
}

#[tokio::test]
async fn test_tool_capability_check_no_requirements() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Create a tool with no capability requirements
    let tool = Tool {
        id: "no_cap_tool".to_string(),
        name: "No Cap Tool".to_string(),
        description: "A tool with no capability requirements".to_string(),
        version: "1.0.0".to_string(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
        required_capabilities: vec![], // No requirements
        category: "test".to_string(),
        tags: vec!["test".to_string()],
        implementation_type: ToolImplementationType::Builtin,
        timeout_seconds: 30,
        requires_confirmation: false,
        metadata: HashMap::new(),
    };
    
    registry.register_tool(tool).await.unwrap();
    
    // Test capability check
    let result = registry.check_tool_capabilities("no_cap_tool", None).await.unwrap();
    assert!(result.allowed);
    assert!(result.missing_capabilities.is_empty());
    assert_eq!(result.message, "No capabilities required");
}

#[tokio::test]
async fn test_tool_search_by_category() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Search by category
    let system_tools = registry.list_tools_by_category("system").await.unwrap();
    assert!(!system_tools.is_empty());
    assert!(system_tools.iter().any(|t| t.id == "open_app"));
    
    let filesystem_tools = registry.list_tools_by_category("filesystem").await.unwrap();
    assert!(!filesystem_tools.is_empty());
    assert!(filesystem_tools.iter().any(|t| t.id == "search_files"));
    
    let productivity_tools = registry.list_tools_by_category("productivity").await.unwrap();
    assert!(!productivity_tools.is_empty());
    assert!(productivity_tools.iter().any(|t| t.id == "create_note"));
}

#[tokio::test]
async fn test_tool_search_by_tags() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Search by tags
    let search_tools = registry.search_tools_by_tags(&["search".to_string()]).await.unwrap();
    assert!(!search_tools.is_empty());
    assert!(search_tools.iter().any(|t| t.id == "search_files"));
    
    let system_tools = registry.search_tools_by_tags(&["system".to_string()]).await.unwrap();
    assert!(!system_tools.is_empty());
    assert!(system_tools.iter().any(|t| t.id == "open_app"));
    
    let note_tools = registry.search_tools_by_tags(&["note".to_string()]).await.unwrap();
    assert!(!note_tools.is_empty());
    assert!(note_tools.iter().any(|t| t.id == "create_note"));
}

#[tokio::test]
async fn test_tool_statistics() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    let stats = registry.get_stats().await.unwrap();
    assert!(stats.total_tools > 0);
    assert!(stats.tools_by_category.contains_key("system"));
    assert!(stats.tools_by_category.contains_key("filesystem"));
    assert!(stats.tools_by_category.contains_key("productivity"));
    assert!(stats.tools_by_type.contains_key("builtin"));
    
    // Execute a tool to update execution stats
    let parameters = serde_json::json!({"app_name": "test"});
    registry.execute_tool("open_app", &parameters.to_string()).await.unwrap();
    
    let updated_stats = registry.get_stats().await.unwrap();
    assert_eq!(updated_stats.total_executions, 1);
    assert_eq!(updated_stats.successful_executions, 1);
    assert_eq!(updated_stats.failed_executions, 0);
    assert!(updated_stats.avg_execution_time_ms > 0.0);
    assert!(updated_stats.last_execution_timestamp > 0);
}

#[tokio::test]
async fn test_tool_validation_invalid_id() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Test invalid tool definition with empty ID
    let invalid_tool = Tool {
        id: "".to_string(), // Empty ID
        name: "Test".to_string(),
        description: "Test".to_string(),
        version: "1.0.0".to_string(),
        parameters_schema: serde_json::json!({}),
        required_capabilities: vec![],
        category: "test".to_string(),
        tags: vec![],
        implementation_type: ToolImplementationType::Builtin,
        timeout_seconds: 30,
        requires_confirmation: false,
        metadata: HashMap::new(),
    };
    
    let result = registry.register_tool(invalid_tool).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Tool ID cannot be empty"));
}

#[tokio::test]
async fn test_tool_validation_invalid_schema() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Test invalid tool definition with invalid JSON schema
    let invalid_tool = Tool {
        id: "test_tool".to_string(),
        name: "Test".to_string(),
        description: "Test".to_string(),
        version: "1.0.0".to_string(),
        parameters_schema: serde_json::json!("invalid_schema"), // Not an object
        required_capabilities: vec![],
        category: "test".to_string(),
        tags: vec![],
        implementation_type: ToolImplementationType::Builtin,
        timeout_seconds: 30,
        requires_confirmation: false,
        metadata: HashMap::new(),
    };
    
    let result = registry.register_tool(invalid_tool).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("schema must be a JSON object"));
}

#[tokio::test]
async fn test_tool_validation_invalid_timeout() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Test invalid tool definition with zero timeout
    let invalid_tool = Tool {
        id: "test_tool".to_string(),
        name: "Test".to_string(),
        description: "Test".to_string(),
        version: "1.0.0".to_string(),
        parameters_schema: serde_json::json!({}),
        required_capabilities: vec![],
        category: "test".to_string(),
        tags: vec![],
        implementation_type: ToolImplementationType::Builtin,
        timeout_seconds: 0, // Invalid timeout
        requires_confirmation: false,
        metadata: HashMap::new(),
    };
    
    let result = registry.register_tool(invalid_tool).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("timeout must be greater than 0"));
}

#[tokio::test]
async fn test_tool_not_found() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Test executing a non-existent tool
    let parameters = serde_json::json!({"param": "value"});
    let result = registry.execute_tool("nonexistent_tool", &parameters.to_string()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Tool not found"));
}

#[tokio::test]
async fn test_tool_get_not_found() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Test getting a non-existent tool
    let result = registry.get_tool("nonexistent_tool").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Tool not found"));
}

#[tokio::test]
async fn test_tool_capability_check_not_found() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Test capability check for non-existent tool
    let result = registry.check_tool_capabilities("nonexistent_tool", None).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Tool not found"));
}

#[tokio::test]
async fn test_tool_implementation_type_to_string() {
    assert_eq!(ToolImplementationType::Builtin.to_string(), "builtin");
    assert_eq!(ToolImplementationType::External.to_string(), "external");
    assert_eq!(ToolImplementationType::Webhook.to_string(), "webhook");
    assert_eq!(ToolImplementationType::Script.to_string(), "script");
}

#[tokio::test]
async fn test_tool_metadata() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Create a tool with metadata
    let mut metadata = HashMap::new();
    metadata.insert("author".to_string(), serde_json::json!("Test Author"));
    metadata.insert("license".to_string(), serde_json::json!("MIT"));
    
    let tool = Tool {
        id: "metadata_tool".to_string(),
        name: "Metadata Tool".to_string(),
        description: "A tool with metadata".to_string(),
        version: "1.0.0".to_string(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
        required_capabilities: vec![],
        category: "test".to_string(),
        tags: vec!["test".to_string()],
        implementation_type: ToolImplementationType::Builtin,
        timeout_seconds: 30,
        requires_confirmation: false,
        metadata: metadata.clone(),
    };
    
    registry.register_tool(tool).await.unwrap();
    
    let retrieved_tool = registry.get_tool("metadata_tool").await.unwrap();
    assert_eq!(retrieved_tool.metadata, metadata);
    assert_eq!(retrieved_tool.metadata["author"], "Test Author");
    assert_eq!(retrieved_tool.metadata["license"], "MIT");
}

#[tokio::test]
async fn test_tool_requires_confirmation() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Create a tool that requires confirmation
    let tool = Tool {
        id: "confirmation_tool".to_string(),
        name: "Confirmation Tool".to_string(),
        description: "A tool that requires confirmation".to_string(),
        version: "1.0.0".to_string(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
        required_capabilities: vec![],
        category: "test".to_string(),
        tags: vec!["test".to_string()],
        implementation_type: ToolImplementationType::Builtin,
        timeout_seconds: 30,
        requires_confirmation: true,
        metadata: HashMap::new(),
    };
    
    registry.register_tool(tool).await.unwrap();
    
    let retrieved_tool = registry.get_tool("confirmation_tool").await.unwrap();
    assert!(retrieved_tool.requires_confirmation);
}

#[tokio::test]
async fn test_tool_timeout_configuration() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Create a tool with custom timeout
    let tool = Tool {
        id: "timeout_tool".to_string(),
        name: "Timeout Tool".to_string(),
        description: "A tool with custom timeout".to_string(),
        version: "1.0.0".to_string(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
        required_capabilities: vec![],
        category: "test".to_string(),
        tags: vec!["test".to_string()],
        implementation_type: ToolImplementationType::Builtin,
        timeout_seconds: 120, // 2 minutes
        requires_confirmation: false,
        metadata: HashMap::new(),
    };
    
    registry.register_tool(tool).await.unwrap();
    
    let retrieved_tool = registry.get_tool("timeout_tool").await.unwrap();
    assert_eq!(retrieved_tool.timeout_seconds, 120);
}

#[tokio::test]
async fn test_tool_version_tracking() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Execute a tool and check version tracking
    let parameters = serde_json::json!({"app_name": "test"});
    let result = registry.execute_tool("open_app", &parameters.to_string()).await.unwrap();
    
    assert_eq!(result.tool_version, "1.0.0");
    assert_eq!(result.tool_id, "open_app");
}

#[tokio::test]
async fn test_tool_execution_time_tracking() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Execute a tool and check execution time tracking
    let parameters = serde_json::json!({"app_name": "test"});
    let result = registry.execute_tool("open_app", &parameters.to_string()).await.unwrap();
    
    assert!(result.execution_time_ms >= 0);
    assert!(result.execution_time_ms < 1000); // Should be fast for mock implementation
}

#[tokio::test]
async fn test_tool_result_metadata() {
    let (temp_dir, registry) = create_test_tool_registry().await;
    
    // Execute a tool and check result metadata
    let parameters = serde_json::json!({"app_name": "test"});
    let result = registry.execute_tool("open_app", &parameters.to_string()).await.unwrap();
    
    assert!(result.success);
    assert!(result.error.is_none());
    assert!(!result.cache_hit); // Mock implementation doesn't use caching
    assert_eq!(result.memory_used_mb, 0); // Mock implementation doesn't track memory
    assert!(result.metadata.is_empty()); // Mock implementation doesn't set metadata
}
