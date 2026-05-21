//! CLI Integration Smoke Tests for AI Core.

use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_cli_runtime_run_success_and_error_metrics() {
    let cli_path = env!("CARGO_BIN_EXE_ai_core_cli");
    let metrics_file_path = "target/tmp_dashboard_metrics.js";

    // Clean up any stale metrics file
    if Path::new(metrics_file_path).exists() {
        let _ = fs::remove_file(metrics_file_path);
    }

    // --- Part 1: Success Path ---
    let success_output = Command::new(cli_path)
        .arg("runtime-run")
        .arg("--prompt")
        .arg("What is Polymera OS?")
        .arg("--dashboard-metrics")
        .arg(metrics_file_path)
        .output()
        .expect("failed to execute ai_core_cli binary");

    assert!(
        success_output.status.success(),
        "CLI did not exit successfully. stderr: {}",
        String::from_utf8_lossy(&success_output.stderr)
    );

    // Verify metrics file content
    assert!(
        Path::new(metrics_file_path).exists(),
        "Metrics file was not created"
    );

    let content = fs::read_to_string(metrics_file_path)
        .expect("failed to read created metrics file");

    assert!(
        content.contains("window.POLYMERA_AI_METRICS ="),
        "Metrics file doesn't contain expected JS window variable"
    );

    // Extract JSON object from the JS file
    let json_start = content.find('{').expect("no opening brace found");
    let json_end = content.rfind('}').expect("no closing brace found");
    let metrics_json: serde_json::Value = serde_json::from_str(&content[json_start..=json_end])
        .expect("failed to parse metrics JSON from file");

    let executions = metrics_json["ai_runtime_backend_executions_total"]
        .as_u64()
        .expect("ai_runtime_backend_executions_total is missing or not a number");
    let errors = metrics_json["ai_runtime_backend_errors_total"]
        .as_u64()
        .expect("ai_runtime_backend_errors_total is missing or not a number");
    let tokens = metrics_json["ai_runtime_local_tokens_total"]
        .as_u64()
        .expect("ai_runtime_local_tokens_total is missing or not a number");

    assert_eq!(executions, 1, "Expected 1 backend execution");
    assert_eq!(errors, 0, "Expected 0 backend errors");
    assert!(tokens > 0, "Expected generated tokens to be greater than 0");

    // --- Part 2: Error Path ---
    let error_output = Command::new(cli_path)
        .arg("runtime-run")
        .arg("--prompt")
        .arg("   ")
        .arg("--dashboard-metrics")
        .arg(metrics_file_path)
        .output()
        .expect("failed to execute ai_core_cli binary on error path");

    // The CLI should propagate the error, resulting in a non-zero exit status
    assert!(
        !error_output.status.success(),
        "CLI exited successfully but error was expected"
    );

    let content_after_error = fs::read_to_string(metrics_file_path)
        .expect("failed to read updated metrics file");

    let json_start_err = content_after_error.find('{').expect("no opening brace found after error");
    let json_end_err = content_after_error.rfind('}').expect("no closing brace found after error");
    let metrics_json_err: serde_json::Value = serde_json::from_str(&content_after_error[json_start_err..=json_end_err])
        .expect("failed to parse updated metrics JSON");

    let executions_after = metrics_json_err["ai_runtime_backend_executions_total"]
        .as_u64()
        .expect("ai_runtime_backend_executions_total after error missing or invalid");
    let errors_after = metrics_json_err["ai_runtime_backend_errors_total"]
        .as_u64()
        .expect("ai_runtime_backend_errors_total after error missing or invalid");

    assert_eq!(executions_after, 2, "Expected total backend executions to be incremented to 2");
    assert_eq!(errors_after, 1, "Expected total backend errors to be incremented to 1");

    // Clean up
    let _ = fs::remove_file(metrics_file_path);
}
