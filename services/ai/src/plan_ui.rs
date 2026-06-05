//! Plan Visualization and Approval UI for Aetheris OS AI Service
//!
//! Renders plans from planner as text-based UI and manages approval flow.
//! Now integrated with plan_store for persistence and deterministic IDs.
//!
//! Follows reference_os/ui style for rendering.

use std::collections::HashMap;
use std::fs::{OpenOptions};
use std::io::Write;
use std::sync::OnceLock;
use chrono::{Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};
use crate::planner::{Plan, PlanStep};
use crate::plan_store::{PlanStore, PlanRecord, PlanStatus, ListQuery, ListResponse};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPlan {
    pub id: String,
    pub steps: Vec<UiStep>,
    pub approved: bool,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiStep {
    pub number: usize,
    pub name: String,
    pub description: String,
    pub status: String,
}

impl UiPlan {
    pub fn from_plan(plan: &Plan) -> Self {
        let steps = plan.steps.iter().enumerate().map(|(i, s)| UiStep {
            number: i + 1,
            name: s.name.clone(),
            description: s.description.clone(),
            status: format!("{:?}", s.status),
        }).collect();
        
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), plan.name.clone());
        metadata.insert("description".to_string(), plan.description.clone());
        metadata.insert("user_goal".to_string(), plan.user_goal.clone());
        metadata.insert("creator".to_string(), plan.creator.clone());
        metadata.insert("created_at".to_string(), plan.created_at.to_string());
        
        UiPlan {
            id: plan.id.clone(),
            steps,
            approved: false,
            approved_by: None,
            approved_at: None,
            metadata,
        }
    }
    
    pub fn from_plan_record(record: &PlanRecord) -> Self {
        let steps = record.plan.steps.iter().enumerate().map(|(i, s)| UiStep {
            number: i + 1,
            name: s.name.clone(),
            description: s.description.clone(),
            status: format!("{:?}", s.status),
        }).collect();
        
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), record.plan.name.clone());
        metadata.insert("description".to_string(), record.plan.description.clone());
        metadata.insert("user_goal".to_string(), record.plan.user_goal.clone());
        metadata.insert("creator".to_string(), record.plan.creator.clone());
        metadata.insert("created_at".to_string(), record.metadata.created_at.to_rfc3339());
        metadata.insert("plan_id".to_string(), record.metadata.plan_id.clone());
        metadata.insert("revision".to_string(), record.metadata.revision.to_string());
        
        UiPlan {
            id: record.metadata.plan_id.clone(),
            steps,
            approved: record.metadata.status == PlanStatus::Approved,
            approved_by: record.approved_by.clone(),
            approved_at: record.approved_at.map(|dt| dt.to_rfc3339()),
            metadata,
        }
    }
}

// Global plan store instance
static PLAN_STORE: OnceLock<PlanStore> = OnceLock::new();

/// Initialize the plan store (call once at startup)
pub async fn init_plan_store() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let store = PlanStore::default().await?;
    PLAN_STORE.set(store).map_err(|_| "Plan store already initialized")?;
    Ok(())
}

/// Get the global plan store instance
fn get_plan_store() -> &'static PlanStore {
    PLAN_STORE.get().expect("Plan store not initialized - call init_plan_store() first")
}

/// Create or get existing plan (now uses persistent store)
pub async fn store_plan_for_ui(plan: &Plan, user_id: &str) -> Result<UiPlan, Box<dyn std::error::Error + Send + Sync>> {
    let store = get_plan_store();
    let record = store.create_plan(plan.clone(), user_id.to_string()).await?;
    Ok(UiPlan::from_plan_record(&record))
}

/// Approve plan with optional modifications (now uses persistent store)
pub async fn approve_plan(
    plan_id: &str, 
    user_id: &str,
    revision: Option<u32>,
    modifications: Option<Vec<PlanStep>>,
) -> Result<UiPlan, Box<dyn std::error::Error + Send + Sync>> {
    let store = get_plan_store();
    let record = store.approve_plan(plan_id, revision, user_id.to_string(), modifications).await?;
    
    // Log approval for audit trail
    log_approval(&record, user_id);
    
    Ok(UiPlan::from_plan_record(&record))
}

/// Legacy approve function for backward compatibility
pub async fn approve_plan_simple(plan_id: &str, user_id: &str) -> Option<UiPlan> {
    match approve_plan(plan_id, user_id, None, None).await {
        Ok(ui_plan) => Some(ui_plan),
        Err(e) => {
            error!("Failed to approve plan {}: {}", plan_id, e);
            None
        }
    }
}

/// Log approval to audit trail
fn log_approval(record: &PlanRecord, user_id: &str) {
    let log_entry = serde_json::to_string(&serde_json::json!({
        "timestamp": Utc::now().to_rfc3339(),
        "plan_id": record.metadata.plan_id,
        "revision": record.metadata.revision,
        "approved": true,
        "approved_by": user_id,
        "approved_at": record.approved_at,
        "user_id": record.metadata.user_id,
        "status": record.metadata.status,
        "plan_hash": blake3::hash(serde_json::to_string(&record.plan).unwrap_or_default().as_bytes()).to_hex().to_string(),
    })).unwrap_or_else(|e| {
        warn!("Failed to serialize approval log entry: {}", e);
        String::new()
    });
    
    if log_entry.is_empty() {
        return;
    }
    
    // Create logs directory if it doesn't exist
    if std::fs::create_dir_all("logs").is_err() {
        warn!("Could not create logs directory");
        return;
    }
    
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/ai_plans.log") {
        if let Err(e) = writeln!(file, "{}", log_entry) {
            warn!("Could not write to log file: {}", e);
        }
    } else {
        warn!("Could not open log file");
    }
}

/// Get plan by ID (now uses persistent store)
pub fn get_plan_for_ui(plan_id: &str) -> Option<UiPlan> {
    let store = get_plan_store();
    match store.get_plan(plan_id) {
        Ok(record) => Some(UiPlan::from_plan_record(&record)),
        Err(e) => {
            warn!("Failed to get plan {}: {}", plan_id, e);
            None
        }
    }
}

/// List plans with optional filtering (now uses persistent store)
pub fn list_plans_for_ui(
    status: Option<PlanStatus>,
    limit: Option<usize>,
    cursor: Option<String>,
) -> ListResponse {
    let store = get_plan_store();
    let query = ListQuery { status, limit, cursor };
    let response = store.list_plans(query);
    
    // Convert to UiPlan response format for backward compatibility
    response
}

/// Legacy list function for backward compatibility
pub fn list_plans_for_ui_simple() -> Vec<UiPlan> {
    let response = list_plans_for_ui(None, None, None);
    response.plans.into_iter().map(|record| UiPlan::from_plan_record(&record)).collect()
}

pub fn render_plan_cli(plan: &UiPlan) -> String {
    let mut out = String::new();
    out.push_str(&format!("Plan ID: {}\n", plan.id));
    
    // Show metadata
    if let Some(name) = plan.metadata.get("name") {
        out.push_str(&format!("Name: {}\n", name));
    }
    if let Some(description) = plan.metadata.get("description") {
        out.push_str(&format!("Description: {}\n", description));
    }
    if let Some(goal) = plan.metadata.get("user_goal") {
        out.push_str(&format!("Goal: {}\n", goal));
    }
    
    out.push_str("\nSteps:\n");
    out.push_str("------\n");
    for step in &plan.steps {
        let status_icon = match step.status.as_str() {
            "Completed" => "✅",
            "InProgress" => "🔄",
            "Failed" => "❌",
            _ => "⏳",
        };
        out.push_str(&format!("{}. {} {}\n", step.number, status_icon, step.name));
        out.push_str(&format!("   {}\n", step.description));
        out.push_str(&format!("   Status: {}\n\n", step.status));
    }
    
    out.push_str(&format!("\nApproval Status: {}\n", if plan.approved { "✅ APPROVED" } else { "⏳ PENDING" }));
    if let Some(by) = &plan.approved_by {
        out.push_str(&format!("Approved by: {} at {}\n", by, plan.approved_at.as_deref().unwrap_or("unknown time")));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::{PlanStep, PlanStatus};
    
    fn create_test_plan() -> Plan {
        let mut plan = Plan::new("Test Plan", "A test plan", "Test goal", "test-user");
        plan.id = "test-plan-123".to_string();
        plan.status = PlanStatus::Ready;
        let mut step = PlanStep::new("Test Step", "A test step", "action");
        step.id = "step-1".to_string();
        step.estimated_duration_us = Some(10_000_000);
        plan.add_step(step);
        plan.tags = vec!["test".to_string()];
        plan
    }
    
    #[test]
    fn test_ui_plan_from_plan() {
        let plan = create_test_plan();
        let ui_plan = UiPlan::from_plan(&plan);
        
        assert_eq!(ui_plan.id, plan.id);
        assert_eq!(ui_plan.steps.len(), 1);
        assert!(!ui_plan.approved);
        assert!(ui_plan.approved_by.is_none());
    }
    
    #[test]
    fn test_render_plan_cli() {
        let plan = create_test_plan();
        let ui_plan = UiPlan::from_plan(&plan);
        let rendered = render_plan_cli(&ui_plan);
        assert!(rendered.contains("Test Plan"));
        assert!(rendered.contains("Test Step"));
    }
}
