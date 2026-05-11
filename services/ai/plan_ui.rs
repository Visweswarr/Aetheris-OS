//! Plan Visualization and Approval UI for Aetheris OS AI Service
//!
//! Renders plans from planner as text-based UI and manages approval flow.
//!
//! Follows reference_os/ui style for rendering.

use std::collections::{HashMap, HashSet};
use std::fs::{OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex};
use chrono::{Utc};
use serde::{Deserialize, Serialize};
use crate::planner::{Plan, PlanStep};

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
        UiPlan {
            id: plan.id.clone(),
            steps,
            approved: false,
            approved_by: None,
            approved_at: None,
            metadata: plan.metadata.clone(),
        }
    }

    pub fn render_cli(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("Plan ID: {}\n", self.id));
        out.push_str("Steps:\n");
        for step in &self.steps {
            out.push_str(&format!("{}. {}\n   {}\n   Status: {}\n", step.number, step.name, step.description, step.status));
        }
        out.push_str(&format!("Approved: {}\n", self.approved));
        if let Some(by) = &self.approved_by {
            out.push_str(&format!("Approved by: {} at {}\n", by, self.approved_at.as_deref().unwrap_or("")));
        }
        out
    }
}

// In-memory approval store (for demo)
lazy_static::lazy_static! {
    static ref APPROVALS: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    static ref PLAN_APPROVAL_DATA: Arc<Mutex<HashMap<String, UiPlan>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub fn approve_plan(plan_id: &str, user_id: &str) -> Option<UiPlan> {
    let mut approvals = APPROVALS.lock().unwrap();
    let mut data = PLAN_APPROVAL_DATA.lock().unwrap();
    let now = Utc::now().to_rfc3339();
    if let Some(plan) = data.get_mut(plan_id) {
        if plan.approved {
            // Already approved, idempotent
            return Some(plan.clone());
        }
        plan.approved = true;
        plan.approved_by = Some(user_id.to_string());
        plan.approved_at = Some(now.clone());
        approvals.insert(plan_id.to_string());
        log_approval(plan, user_id);
        return Some(plan.clone());
    }
    None
}

pub fn store_plan_for_ui(plan: &Plan) {
    let mut data = PLAN_APPROVAL_DATA.lock().unwrap();
    let ui_plan = UiPlan::from_plan(plan);
    data.insert(plan.id.clone(), ui_plan);
}

fn log_approval(plan: &UiPlan, user_id: &str) {
    let log_entry = serde_json::to_string(&serde_json::json!({
        "plan_id": plan.id,
        "approved": true,
        "approved_by": user_id,
        "approved_at": plan.approved_at,
        "steps": plan.steps,
        "metadata": plan.metadata,
    })).unwrap();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/ai_plans.log")
        .unwrap();
    writeln!(file, "{}", log_entry).unwrap();
}

pub fn get_ui_plan(plan_id: &str) -> Option<UiPlan> {
    let data = PLAN_APPROVAL_DATA.lock().unwrap();
    data.get(plan_id).cloned()
}

pub fn get_all_ui_plans() -> Vec<UiPlan> {
    let data = PLAN_APPROVAL_DATA.lock().unwrap();
    data.values().cloned().collect()
}
