//! Simple HTTP API for Plan Visualization and Approval
//!
//! Example integration showing how to expose plan visualization 
//! and approval over HTTP using the plan_ui module.

use std::sync::Arc;
use tokio::net::TcpListener;
use serde::{Deserialize, Serialize};
use serde_json::json;

// Import from the AI service
use aetheris_ai::plan_ui::{
    approve_plan, get_plan_for_ui, list_plans_for_ui, render_plan_cli, UiPlan
};

#[derive(Deserialize)]
struct ApprovePlanRequest {
    plan_id: String,
    user_id: String,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    fn error(message: String) -> Self {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

async fn handle_list_plans() -> Result<String, Box<dyn std::error::Error>> {
    let plans = list_plans_for_ui();
    let response = ApiResponse::success(plans);
    Ok(serde_json::to_string(&response)?)
}

async fn handle_get_plan(plan_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    match get_plan_for_ui(plan_id) {
        Some(plan) => {
            let response = ApiResponse::success(plan);
            Ok(serde_json::to_string(&response)?)
        }
        None => {
            let response: ApiResponse<UiPlan> = ApiResponse::error(
                "Plan not found".to_string()
            );
            Ok(serde_json::to_string(&response)?)
        }
    }
}

async fn handle_approve_plan(body: &str) -> Result<String, Box<dyn std::error::Error>> {
    let request: ApprovePlanRequest = serde_json::from_str(body)?;
    
    match approve_plan(&request.plan_id, &request.user_id) {
        Some(approved_plan) => {
            let response = ApiResponse::success(approved_plan);
            Ok(serde_json::to_string(&response)?)
        }
        None => {
            let response: ApiResponse<UiPlan> = ApiResponse::error(
                "Plan not found or cannot be approved".to_string()
            );
            Ok(serde_json::to_string(&response)?)
        }
    }
}

async fn handle_render_plan(plan_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    match get_plan_for_ui(plan_id) {
        Some(plan) => {
            let rendered = render_plan_cli(&plan);
            let response = ApiResponse::success(json!({
                "plan_id": plan_id,
                "rendered": rendered,
                "approved": plan.approved,
                "approved_by": plan.approved_by,
                "approved_at": plan.approved_at,
            }));
            Ok(serde_json::to_string(&response)?)
        }
        None => {
            let response: ApiResponse<serde_json::Value> = ApiResponse::error(
                "Plan not found".to_string()
            );
            Ok(serde_json::to_string(&response)?)
        }
    }
}

// Simple HTTP request parser
struct HttpRequest {
    method: String,
    path: String,
    body: String,
}

impl HttpRequest {
    fn parse(request: &str) -> Option<Self> {
        let lines: Vec<&str> = request.lines().collect();
        if lines.is_empty() {
            return None;
        }
        
        let first_line_parts: Vec<&str> = lines[0].split_whitespace().collect();
        if first_line_parts.len() < 2 {
            return None;
        }
        
        let method = first_line_parts[0].to_string();
        let path = first_line_parts[1].to_string();
        
        // Find the body (after the empty line)
        let mut body = String::new();
        let mut found_empty_line = false;
        for line in lines.iter().skip(1) {
            if found_empty_line {
                body.push_str(line);
                body.push('\n');
            } else if line.is_empty() {
                found_empty_line = true;
            }
        }
        
        Some(HttpRequest { method, path, body })
    }
}

async fn handle_request(request: &str) -> String {
    let req = match HttpRequest::parse(request) {
        Some(req) => req,
        None => {
            return "HTTP/1.1 400 Bad Request\r\n\r\nBad Request".to_string();
        }
    };
    
    let response_body = match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/api/plans") => {
            handle_list_plans().await.unwrap_or_else(|e| {
                serde_json::to_string(&ApiResponse::<()>::error(e.to_string())).unwrap()
            })
        }
        ("GET", path) if path.starts_with("/api/plans/") => {
            let plan_id = &path[11..]; // Remove "/api/plans/"
            handle_get_plan(plan_id).await.unwrap_or_else(|e| {
                serde_json::to_string(&ApiResponse::<()>::error(e.to_string())).unwrap()
            })
        }
        ("GET", path) if path.starts_with("/api/plans/") && path.ends_with("/render") => {
            let plan_id = &path[11..path.len()-7]; // Remove "/api/plans/" and "/render"
            handle_render_plan(plan_id).await.unwrap_or_else(|e| {
                serde_json::to_string(&ApiResponse::<()>::error(e.to_string())).unwrap()
            })
        }
        ("POST", "/api/plans/approve") => {
            handle_approve_plan(&req.body).await.unwrap_or_else(|e| {
                serde_json::to_string(&ApiResponse::<()>::error(e.to_string())).unwrap()
            })
        }
        _ => {
            serde_json::to_string(&ApiResponse::<()>::error("Not Found".to_string())).unwrap()
        }
    };
    
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        response_body.len(),
        response_body
    )
}

/// Example HTTP server for plan visualization and approval
/// 
/// This is a demonstration of how to integrate the plan_ui module
/// with HTTP endpoints. In production, you would use a proper web framework.
pub async fn run_http_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    
    println!("AI Plan HTTP API server running on http://{}", addr);
    println!("Available endpoints:");
    println!("  GET    /api/plans                 - List all plans");
    println!("  GET    /api/plans/<id>            - Get specific plan");
    println!("  GET    /api/plans/<id>/render     - Get rendered plan CLI view");
    println!("  POST   /api/plans/approve         - Approve a plan");
    
    loop {
        let (mut socket, _) = listener.accept().await?;
        
        tokio::spawn(async move {
            let mut buffer = [0; 4096];
            
            match socket.try_read(&mut buffer) {
                Ok(n) => {
                    let request = String::from_utf8_lossy(&buffer[..n]);
                    let response = handle_request(&request).await;
                    
                    if let Err(e) = socket.try_write(response.as_bytes()) {
                        eprintln!("Failed to write response: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read from socket: {}", e);
                }
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);
    
    run_http_server(port).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use aetheris_ai::plan_ui::store_plan_for_ui;
    use aetheris_ai::planner::{Plan, Step, StepPriority, StepStatus};
    
    fn create_test_plan() -> Plan {
        Plan {
            id: "test-plan-123".to_string(),
            name: "Test Plan".to_string(),
            description: "A test plan".to_string(),
            user_goal: "Test goal".to_string(),
            version: 1,
            priority: StepPriority::Normal,
            status: StepStatus::Pending,
            steps: vec![
                Step {
                    id: "step-1".to_string(),
                    name: "Test Step".to_string(),
                    description: "A test step".to_string(),
                    step_type: "action".to_string(),
                    priority: StepPriority::Normal,
                    status: StepStatus::Pending,
                    dependencies: vec![],
                    estimated_duration_secs: Some(10),
                    tool_id: None,
                    expected_outputs: vec![],
                    sub_steps: vec![],
                },
            ],
            tags: vec!["test".to_string()],
            creator: "test-user".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            estimated_duration_secs: 10,
            actual_duration_secs: None,
            progress: 0.0,
        }
    }
    
    #[tokio::test]
    async fn test_handle_list_plans() {
        let plan = create_test_plan();
        store_plan_for_ui(&plan);
        
        let response = handle_list_plans().await.unwrap();
        assert!(response.contains("success"));
        assert!(response.contains(&plan.id));
    }
    
    #[tokio::test]
    async fn test_handle_get_plan() {
        let plan = create_test_plan();
        store_plan_for_ui(&plan);
        
        let response = handle_get_plan(&plan.id).await.unwrap();
        assert!(response.contains("success"));
        assert!(response.contains(&plan.id));
        
        let response = handle_get_plan("nonexistent").await.unwrap();
        assert!(response.contains("Plan not found"));
    }
    
    #[tokio::test]
    async fn test_handle_approve_plan() {
        let plan = create_test_plan();
        store_plan_for_ui(&plan);
        
        let request_body = serde_json::to_string(&ApprovePlanRequest {
            plan_id: plan.id.clone(),
            user_id: "test-user".to_string(),
        }).unwrap();
        
        let response = handle_approve_plan(&request_body).await.unwrap();
        assert!(response.contains("success"));
        assert!(response.contains("approved"));
    }
    
    #[test]
    fn test_http_request_parse() {
        let request = "GET /api/plans HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let parsed = HttpRequest::parse(request).unwrap();
        
        assert_eq!(parsed.method, "GET");
        assert_eq!(parsed.path, "/api/plans");
    }
}