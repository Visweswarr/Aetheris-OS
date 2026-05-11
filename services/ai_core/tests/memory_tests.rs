//! Comprehensive tests for the Minimal Memory Store module

use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;

use aetheris_ai_core::memory::{
    MemoryStore, MemoryConfig, MemoryEntry, RedactionPolicy, 
    default_memory_config, default_redaction_policies
};
use aetheris_ai_core::ipc::{ToolCallRequest, ToolCallResponse};

/// Test helper to create a temporary memory store
async fn create_test_memory_store() -> (TempDir, MemoryStore) {
    let temp_dir = TempDir::new().unwrap();
    let config = MemoryConfig {
        base_path: temp_dir.path().to_path_buf(),
        max_entries: 100,
        default_ttl_seconds: 3600,
        cleanup_interval_seconds: 60,
        enable_redaction: false,
        redaction_policies: Vec::new(),
        enable_ngfs: false,
        ngfs_snapshot_interval_seconds: 3600,
    };

    let store = MemoryStore::new(config).unwrap();
    store.initialize().await.unwrap();
    
    (temp_dir, store)
}

/// Test helper to create a memory store with redaction enabled
async fn create_test_memory_store_with_redaction() -> (TempDir, MemoryStore) {
    let temp_dir = TempDir::new().unwrap();
    let config = MemoryConfig {
        base_path: temp_dir.path().to_path_buf(),
        max_entries: 100,
        default_ttl_seconds: 3600,
        cleanup_interval_seconds: 60,
        enable_redaction: true,
        redaction_policies: default_redaction_policies(),
        enable_ngfs: false,
        ngfs_snapshot_interval_seconds: 3600,
    };

    let store = MemoryStore::new(config).unwrap();
    store.initialize().await.unwrap();
    
    (temp_dir, store)
}

#[tokio::test]
async fn test_memory_store_basic_operations() {
    let (temp_dir, store) = create_test_memory_store().await;

    // Test put and get
    store.put(
        "key1".to_string(),
        "value1".to_string(),
        vec!["tag1".to_string()],
        None,
        "user1".to_string(),
        None
    ).await.unwrap();
    
    let entry = store.get("key1").await.unwrap();
    assert!(entry.is_some());
    let entry = entry.unwrap();
    assert_eq!(entry.value, "value1");
    assert_eq!(entry.tags, vec!["tag1"]);
    assert_eq!(entry.user_id, "user1");
    assert_eq!(entry.access_count, 1);

    // Test query by tags
    let results = store.query(&["tag1".to_string()]).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].key, "key1");

    // Test delete
    let deleted = store.delete("key1").await.unwrap();
    assert!(deleted);

    let entry = store.get("key1").await.unwrap();
    assert!(entry.is_none());
}

#[tokio::test]
async fn test_memory_store_ttl_expiration() {
    let (temp_dir, store) = create_test_memory_store().await;

    // Store with 1 second TTL
    store.put(
        "key1".to_string(),
        "value1".to_string(),
        vec!["tag1".to_string()],
        Some(1),
        "user1".to_string(),
        None
    ).await.unwrap();
    
    // Should be available immediately
    let entry = store.get("key1").await.unwrap();
    assert!(entry.is_some());

    // Wait for expiration
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Should be expired
    let entry = store.get("key1").await.unwrap();
    assert!(entry.is_none());
}

#[tokio::test]
async fn test_memory_store_tag_querying() {
    let (temp_dir, store) = create_test_memory_store().await;

    // Add multiple entries with different tags
    store.put("key1".to_string(), "value1".to_string(), vec!["tag1".to_string(), "tag2".to_string()], None, "user1".to_string(), None).await.unwrap();
    store.put("key2".to_string(), "value2".to_string(), vec!["tag1".to_string()], None, "user1".to_string(), None).await.unwrap();
    store.put("key3".to_string(), "value3".to_string(), vec!["tag2".to_string()], None, "user1".to_string(), None).await.unwrap();
    store.put("key4".to_string(), "value4".to_string(), vec!["tag3".to_string()], None, "user1".to_string(), None).await.unwrap();

    // Query by single tag
    let results = store.query(&["tag1".to_string()]).await.unwrap();
    assert_eq!(results.len(), 2);
    let keys: Vec<&str> = results.iter().map(|r| r.key.as_str()).collect();
    assert!(keys.contains(&"key1"));
    assert!(keys.contains(&"key2"));

    // Query by multiple tags (intersection)
    let results = store.query(&["tag1".to_string(), "tag2".to_string()]).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].key, "key1");

    // Query by non-existent tag
    let results = store.query(&["nonexistent".to_string()]).await.unwrap();
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_memory_store_access_counting() {
    let (temp_dir, store) = create_test_memory_store().await;

    // Store an entry
    store.put("key1".to_string(), "value1".to_string(), vec!["tag1".to_string()], None, "user1".to_string(), None).await.unwrap();
    
    // Access multiple times
    for _ in 0..5 {
        let entry = store.get("key1").await.unwrap();
        assert!(entry.is_some());
    }

    // Check access count
    let entry = store.get("key1").await.unwrap();
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().access_count, 6); // 1 from put + 5 from gets
}

#[tokio::test]
async fn test_memory_store_redaction() {
    let (temp_dir, store) = create_test_memory_store_with_redaction().await;

    // Test password redaction
    let value_with_password = "username: john, password: secret123";
    store.put("key1".to_string(), value_with_password.to_string(), vec!["tag1".to_string()], None, "user1".to_string(), None).await.unwrap();
    
    let entry = store.get("key1").await.unwrap();
    assert!(entry.is_some());
    let entry = entry.unwrap();
    assert!(entry.redacted);
    assert!(entry.value.contains("[REDACTED]"));
    assert!(!entry.value.contains("secret123"));

    // Test API key redaction
    let value_with_api_key = "api_key: abc123def456";
    store.put("key2".to_string(), value_with_api_key.to_string(), vec!["tag2".to_string()], None, "user1".to_string(), None).await.unwrap();
    
    let entry = store.get("key2").await.unwrap();
    assert!(entry.is_some());
    let entry = entry.unwrap();
    assert!(entry.redacted);
    assert!(entry.value.contains("[REDACTED]"));

    // Test email redaction
    let value_with_email = "Contact: john.doe@example.com";
    store.put("key3".to_string(), value_with_email.to_string(), vec!["tag3".to_string()], None, "user1".to_string(), None).await.unwrap();
    
    let entry = store.get("key3").await.unwrap();
    assert!(entry.is_some());
    let entry = entry.unwrap();
    assert!(entry.redacted);
    assert!(entry.value.contains("[EMAIL_REDACTED]"));
    assert!(!entry.value.contains("john.doe@example.com"));
}

#[tokio::test]
async fn test_memory_store_statistics() {
    let (temp_dir, store) = create_test_memory_store().await;

    // Add some entries
    store.put("key1".to_string(), "value1".to_string(), vec!["tag1".to_string()], None, "user1".to_string(), None).await.unwrap();
    store.put("key2".to_string(), "value2".to_string(), vec!["tag2".to_string()], None, "user1".to_string(), None).await.unwrap();
    store.put("key3".to_string(), "value3".to_string(), vec!["tag3".to_string()], Some(1), "user1".to_string(), None).await.unwrap(); // Will expire

    let stats = store.get_stats().await.unwrap();
    assert_eq!(stats.total_entries, 3);
    assert_eq!(stats.active_entries, 3);
    assert_eq!(stats.expired_entries, 0);

    // Wait for one entry to expire
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Access expired entry to trigger cleanup
    let _ = store.get("key3").await.unwrap();
    
    let stats = store.get_stats().await.unwrap();
    assert_eq!(stats.total_entries, 2);
    assert_eq!(stats.active_entries, 2);
    assert_eq!(stats.expired_entries, 0); // Cleaned up
}

#[tokio::test]
async fn test_memory_store_clear() {
    let (temp_dir, store) = create_test_memory_store().await;

    // Add some entries
    store.put("key1".to_string(), "value1".to_string(), vec!["tag1".to_string()], None, "user1".to_string(), None).await.unwrap();
    store.put("key2".to_string(), "value2".to_string(), vec!["tag2".to_string()], None, "user1".to_string(), None).await.unwrap();

    let stats = store.get_stats().await.unwrap();
    assert_eq!(stats.total_entries, 2);

    // Clear all entries
    store.clear().await.unwrap();

    let stats = store.get_stats().await.unwrap();
    assert_eq!(stats.total_entries, 0);
    assert_eq!(stats.active_entries, 0);

    // Verify entries are gone
    let entry1 = store.get("key1").await.unwrap();
    let entry2 = store.get("key2").await.unwrap();
    assert!(entry1.is_none());
    assert!(entry2.is_none());
}

#[tokio::test]
async fn test_memory_store_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let config = MemoryConfig {
        base_path: temp_dir.path().to_path_buf(),
        max_entries: 100,
        default_ttl_seconds: 3600,
        cleanup_interval_seconds: 60,
        enable_redaction: false,
        redaction_policies: Vec::new(),
        enable_ngfs: false,
        ngfs_snapshot_interval_seconds: 3600,
    };

    // Create first store and add data
    {
        let store = MemoryStore::new(config.clone()).unwrap();
        store.initialize().await.unwrap();
        
        store.put("key1".to_string(), "value1".to_string(), vec!["tag1".to_string()], None, "user1".to_string(), None).await.unwrap();
        store.put("key2".to_string(), "value2".to_string(), vec!["tag2".to_string()], None, "user1".to_string(), None).await.unwrap();
    }

    // Create second store and verify data is loaded
    {
        let store = MemoryStore::new(config).unwrap();
        store.initialize().await.unwrap();
        
        let entry1 = store.get("key1").await.unwrap();
        let entry2 = store.get("key2").await.unwrap();
        
        assert!(entry1.is_some());
        assert!(entry2.is_some());
        assert_eq!(entry1.unwrap().value, "value1");
        assert_eq!(entry2.unwrap().value, "value2");
    }
}

#[tokio::test]
async fn test_memory_store_tool_calls() {
    let (temp_dir, store) = create_test_memory_store().await;

    // Test put tool call
    let put_request = ToolCallRequest {
        function_name: "mem.put".to_string(),
        arguments: serde_json::json!({
            "key": "test_key",
            "value": "test_value",
            "tags": ["test_tag"],
            "ttl_seconds": 3600
        }).to_string(),
        cbor_payload: None,
        call_id: "test_call_1".to_string(),
        session_id: Some("test_session".to_string()),
        metadata: None,
        timeout_seconds: 30,
        r#async: false,
    };

    let response = store.handle_put_tool(&put_request).await.unwrap();
    assert!(response.success);
    assert!(response.result.unwrap().result.contains("Stored memory entry"));

    // Test get tool call
    let get_request = ToolCallRequest {
        function_name: "mem.get".to_string(),
        arguments: serde_json::json!({
            "key": "test_key"
        }).to_string(),
        cbor_payload: None,
        call_id: "test_call_2".to_string(),
        session_id: Some("test_session".to_string()),
        metadata: None,
        timeout_seconds: 30,
        r#async: false,
    };

    let response = store.handle_get_tool(&get_request).await.unwrap();
    assert!(response.success);
    let result: serde_json::Value = serde_json::from_str(&response.result.unwrap().result).unwrap();
    assert_eq!(result["key"], "test_key");
    assert_eq!(result["value"], "test_value");

    // Test query tool call
    let query_request = ToolCallRequest {
        function_name: "mem.query".to_string(),
        arguments: serde_json::json!({
            "tags": ["test_tag"]
        }).to_string(),
        cbor_payload: None,
        call_id: "test_call_3".to_string(),
        session_id: Some("test_session".to_string()),
        metadata: None,
        timeout_seconds: 30,
        r#async: false,
    };

    let response = store.handle_query_tool(&query_request).await.unwrap();
    assert!(response.success);
    let results: Vec<serde_json::Value> = serde_json::from_str(&response.result.unwrap().result).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["key"], "test_key");

    // Test delete tool call
    let delete_request = ToolCallRequest {
        function_name: "mem.delete".to_string(),
        arguments: serde_json::json!({
            "key": "test_key"
        }).to_string(),
        cbor_payload: None,
        call_id: "test_call_4".to_string(),
        session_id: Some("test_session".to_string()),
        metadata: None,
        timeout_seconds: 30,
        r#async: false,
    };

    let response = store.handle_delete_tool(&delete_request).await.unwrap();
    assert!(response.success);
    assert!(response.result.unwrap().result.contains("Deleted: true"));

    // Test stats tool call
    let stats_request = ToolCallRequest {
        function_name: "mem.stats".to_string(),
        arguments: "{}".to_string(),
        cbor_payload: None,
        call_id: "test_call_5".to_string(),
        session_id: Some("test_session".to_string()),
        metadata: None,
        timeout_seconds: 30,
        r#async: false,
    };

    let response = store.handle_stats_tool(&stats_request).await.unwrap();
    assert!(response.success);
    let stats: serde_json::Value = serde_json::from_str(&response.result.unwrap().result).unwrap();
    assert_eq!(stats["total_entries"], 0);
}

#[tokio::test]
async fn test_memory_store_tool_call_errors() {
    let (temp_dir, store) = create_test_memory_store().await;

    // Test put tool call with missing arguments
    let put_request = ToolCallRequest {
        function_name: "mem.put".to_string(),
        arguments: serde_json::json!({
            "key": "test_key"
            // Missing value
        }).to_string(),
        cbor_payload: None,
        call_id: "test_call_1".to_string(),
        session_id: Some("test_session".to_string()),
        metadata: None,
        timeout_seconds: 30,
        r#async: false,
    };

    let response = store.handle_put_tool(&put_request).await;
    assert!(response.is_err());

    // Test get tool call with missing key
    let get_request = ToolCallRequest {
        function_name: "mem.get".to_string(),
        arguments: "{}".to_string(), // Missing key
        cbor_payload: None,
        call_id: "test_call_2".to_string(),
        session_id: Some("test_session".to_string()),
        metadata: None,
        timeout_seconds: 30,
        r#async: false,
    };

    let response = store.handle_get_tool(&get_request).await;
    assert!(response.is_err());

    // Test query tool call with missing tags
    let query_request = ToolCallRequest {
        function_name: "mem.query".to_string(),
        arguments: "{}".to_string(), // Missing tags
        cbor_payload: None,
        call_id: "test_call_3".to_string(),
        session_id: Some("test_session".to_string()),
        metadata: None,
        timeout_seconds: 30,
        r#async: false,
    };

    let response = store.handle_query_tool(&query_request).await;
    assert!(response.is_err());
}

#[tokio::test]
async fn test_memory_store_redaction_policies() {
    let policies = default_redaction_policies();
    assert!(!policies.is_empty());

    // Check that we have the expected policies
    let policy_names: Vec<&str> = policies.iter().map(|p| p.name.as_str()).collect();
    assert!(policy_names.contains(&"password"));
    assert!(policy_names.contains(&"api_key"));
    assert!(policy_names.contains(&"token"));
    assert!(policy_names.contains(&"secret"));
    assert!(policy_names.contains(&"email"));
    assert!(policy_names.contains(&"phone"));

    // Test password policy
    let password_policy = policies.iter().find(|p| p.name == "password").unwrap();
    assert!(password_policy.pattern.contains("password"));
    assert_eq!(password_policy.replacement, "[REDACTED]");
}

#[tokio::test]
async fn test_memory_store_configuration_validation() {
    let temp_dir = TempDir::new().unwrap();

    // Test invalid max_entries
    let invalid_config = MemoryConfig {
        base_path: temp_dir.path().to_path_buf(),
        max_entries: 0, // Invalid
        default_ttl_seconds: 3600,
        cleanup_interval_seconds: 60,
        enable_redaction: false,
        redaction_policies: Vec::new(),
        enable_ngfs: false,
        ngfs_snapshot_interval_seconds: 3600,
    };

    let result = MemoryStore::new(invalid_config);
    assert!(result.is_err());

    // Test invalid default_ttl_seconds
    let invalid_config = MemoryConfig {
        base_path: temp_dir.path().to_path_buf(),
        max_entries: 100,
        default_ttl_seconds: 0, // Invalid
        cleanup_interval_seconds: 60,
        enable_redaction: false,
        redaction_policies: Vec::new(),
        enable_ngfs: false,
        ngfs_snapshot_interval_seconds: 3600,
    };

    let result = MemoryStore::new(invalid_config);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_memory_store_invalid_regex_policies() {
    let temp_dir = TempDir::new().unwrap();
    let invalid_config = MemoryConfig {
        base_path: temp_dir.path().to_path_buf(),
        max_entries: 100,
        default_ttl_seconds: 3600,
        cleanup_interval_seconds: 60,
        enable_redaction: true,
        redaction_policies: vec![
            RedactionPolicy {
                name: "invalid".to_string(),
                pattern: "[invalid regex".to_string(), // Invalid regex
                replacement: "[REDACTED]".to_string(),
                description: None,
            }
        ],
        enable_ngfs: false,
        ngfs_snapshot_interval_seconds: 3600,
    };

    let result = MemoryStore::new(invalid_config);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_memory_store_user_session_tracking() {
    let (temp_dir, store) = create_test_memory_store().await;

    // Store entries with different users and sessions
    store.put("key1".to_string(), "value1".to_string(), vec!["tag1".to_string()], None, "user1".to_string(), Some("session1".to_string())).await.unwrap();
    store.put("key2".to_string(), "value2".to_string(), vec!["tag2".to_string()], None, "user2".to_string(), Some("session2".to_string())).await.unwrap();
    store.put("key3".to_string(), "value3".to_string(), vec!["tag3".to_string()], None, "user1".to_string(), Some("session3".to_string())).await.unwrap();

    // Retrieve and verify user/session tracking
    let entry1 = store.get("key1").await.unwrap().unwrap();
    assert_eq!(entry1.user_id, "user1");
    assert_eq!(entry1.session_id, Some("session1".to_string()));

    let entry2 = store.get("key2").await.unwrap().unwrap();
    assert_eq!(entry2.user_id, "user2");
    assert_eq!(entry2.session_id, Some("session2".to_string()));

    let entry3 = store.get("key3").await.unwrap().unwrap();
    assert_eq!(entry3.user_id, "user1");
    assert_eq!(entry3.session_id, Some("session3".to_string()));
}

#[tokio::test]
async fn test_memory_store_concurrent_access() {
    let (temp_dir, store) = create_test_memory_store().await;

    // Spawn multiple tasks to test concurrent access
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let store_clone = store.clone();
        let handle = tokio::spawn(async move {
            let key = format!("key_{}", i);
            let value = format!("value_{}", i);
            let tags = vec![format!("tag_{}", i)];
            
            // Put entry
            store_clone.put(key.clone(), value.clone(), tags, None, "user1".to_string(), None).await.unwrap();
            
            // Get entry
            let entry = store_clone.get(&key).await.unwrap();
            assert!(entry.is_some());
            assert_eq!(entry.unwrap().value, value);
            
            // Delete entry
            let deleted = store_clone.delete(&key).await.unwrap();
            assert!(deleted);
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all entries are gone
    let stats = store.get_stats().await.unwrap();
    assert_eq!(stats.total_entries, 0);
}

#[tokio::test]
async fn test_memory_store_large_data() {
    let (temp_dir, store) = create_test_memory_store().await;

    // Store large value
    let large_value = "x".repeat(10000); // 10KB string
    store.put("large_key".to_string(), large_value.clone(), vec!["large".to_string()], None, "user1".to_string(), None).await.unwrap();
    
    // Retrieve and verify
    let entry = store.get("large_key").await.unwrap();
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().value, large_value);

    // Check statistics
    let stats = store.get_stats().await.unwrap();
    assert!(stats.storage_size_bytes > 10000);
}

#[tokio::test]
async fn test_memory_store_unicode_data() {
    let (temp_dir, store) = create_test_memory_store().await;

    // Store unicode data
    let unicode_value = "Hello 世界! 🌍 测试";
    let unicode_tags = vec!["unicode".to_string(), "测试".to_string()];
    
    store.put("unicode_key".to_string(), unicode_value.to_string(), unicode_tags.clone(), None, "user1".to_string(), None).await.unwrap();
    
    // Retrieve and verify
    let entry = store.get("unicode_key").await.unwrap();
    assert!(entry.is_some());
    let entry = entry.unwrap();
    assert_eq!(entry.value, unicode_value);
    assert_eq!(entry.tags, unicode_tags);

    // Query by unicode tag
    let results = store.query(&["测试".to_string()]).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].key, "unicode_key");
}
