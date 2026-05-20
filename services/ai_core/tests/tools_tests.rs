//! Current-contract tests for the AI Core tool registry.

use std::path::Path;

use aetheris_ai_core::ipc::ToolCallRequest;
use aetheris_ai_core::tools::{ToolImplementationType, ToolRegistry};
use serde_json::Value;
use tempfile::TempDir;

async fn registry() -> (TempDir, ToolRegistry) {
    let temp_dir = TempDir::new().unwrap();
    let registry = ToolRegistry::new(temp_dir.path()).await.unwrap();
    (temp_dir, registry)
}

fn call(tool_name: &str, parameters: Value) -> ToolCallRequest {
    ToolCallRequest {
        tool_name: tool_name.to_string(),
        call_id: format!("test-{tool_name}"),
        parameters,
        arguments: String::new(),
        cap_token: None,
        user_id: Some("test-user".to_string()),
        session_id: Some("test-session".to_string()),
    }
}

#[tokio::test]
async fn registry_loads_current_builtin_tools() {
    let (_temp, registry) = registry().await;
    let tools = registry.list_tools().await;
    let ids: Vec<_> = tools.iter().map(|tool| tool.id.as_str()).collect();

    assert!(ids.contains(&"echo"));
    assert!(ids.contains(&"log_summarizer"));
    assert!(ids.contains(&"browser_summarize"));
    assert!(ids.contains(&"browser_classify_page"));
}

#[tokio::test]
async fn echo_tool_executes_with_canonical_tool_call_request() {
    let (_temp, registry) = registry().await;
    let result = registry
        .execute_tool(&call("echo", serde_json::json!({"message": "hello"})))
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(result.tool_id, "echo");
    let decoded: Value = serde_cbor::from_slice(&result.result.unwrap()).unwrap();
    assert_eq!(decoded["message"], "hello");
}

#[tokio::test]
async fn browser_summarize_is_deterministic_and_local() {
    let (_temp, registry) = registry().await;
    let request = call(
        "browser_summarize",
        serde_json::json!({
            "text": "Polymera renders a local page. Polymera summarizes the page locally.",
            "max_chars": 120
        }),
    );

    let first = registry.execute_tool(&request).await.unwrap();
    let second = registry.execute_tool(&request).await.unwrap();
    assert!(first.success);
    assert_eq!(first.result, second.result);
    let decoded: Value = serde_json::from_slice(&first.result.unwrap()).unwrap();
    assert_eq!(decoded["metric"], "ai_browser_summaries_total");
    assert!(decoded["summary"].as_str().unwrap().contains("Polymera"));
}

#[tokio::test]
async fn browser_classify_flags_suspicious_url_and_text() {
    let (_temp, registry) = registry().await;
    let result = registry
        .execute_tool(&call(
            "browser_classify_page",
            serde_json::json!({
                "url": "http://xn--wallet-login.example/free-airdrop",
                "text": "Urgent wallet seed phrase verification required."
            }),
        ))
        .await
        .unwrap();

    let decoded: Value = serde_json::from_slice(&result.result.unwrap()).unwrap();
    assert_eq!(decoded["suspicious"], true);
    assert!(decoded["reasons"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn file_backed_browser_summarize_reads_approved_local_input() {
    let (temp, registry) = registry().await;
    let input = temp.path().join("page.txt");
    tokio::fs::write(&input, "Local browser page content. ".repeat(8))
        .await
        .unwrap();
    let result = registry
        .execute_tool(&call(
            "browser_summarize",
            serde_json::json!({"path": input, "max_chars": 100}),
        ))
        .await
        .unwrap();

    let decoded: Value = serde_json::from_slice(&result.result.unwrap()).unwrap();
    assert!(decoded["input_chars"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn validate_parameters_rejects_missing_required_fields() {
    let (_temp, registry) = registry().await;
    let err = registry
        .validate_parameters("echo", &serde_json::json!({}))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("missing required field 'message'"));
}

#[tokio::test]
async fn get_stats_tracks_executions() {
    let (_temp, registry) = registry().await;
    registry
        .execute_tool(&call("echo", serde_json::json!({"message": "hello"})))
        .await
        .unwrap();

    let stats = registry.get_stats().await;
    assert_eq!(stats.total_executions, 1);
    assert_eq!(stats.successful_executions, 1);
    assert_eq!(stats.failed_executions, 0);
}

#[tokio::test]
async fn implementation_type_is_structured_not_stringly_typed() {
    let (_temp, registry) = registry().await;
    let tool = registry.get_tool("echo").await.unwrap();
    assert_eq!(tool.implementation_type, ToolImplementationType::Builtin);
}

#[tokio::test]
async fn missing_file_is_reported_as_io_error() {
    let (_temp, registry) = registry().await;
    let missing = Path::new("definitely-missing-page.txt");
    let err = registry
        .execute_tool(&call(
            "browser_summarize",
            serde_json::json!({"path": missing}),
        ))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("cannot stat file"));
}
