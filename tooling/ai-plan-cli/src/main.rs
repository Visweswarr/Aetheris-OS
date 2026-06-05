//! AI Plan CLI - Visualization and Approval Tool
//!
//! Command-line interface for visualizing and approving AI-generated plans
//! from the Aetheris AI service with persistent storage and revision tracking.

use std::process;
use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use serde_json::Value;

use aetheris_ai::plan_ui::{
    approve_plan, get_plan_for_ui, list_plans_for_ui, render_plan_cli, UiPlan,
    init_plan_store
};
use aetheris_ai::plan_store::{PlanStore, ListQuery};

/// AI Plan CLI - Visualization and Approval Tool
#[derive(Parser)]
#[command(name = "ai-plan")]
#[command(about = "AI Plan visualization and approval tool for Aetheris OS")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List plans with optional filtering and pagination
    List {
        /// Filter by status (pending, approved)
        #[arg(short, long)]
        status: Option<String>,
        
        /// Filter by creator user ID
        #[arg(short = 'c', long)]
        creator: Option<String>,
        
        /// Number of plans to return (default: 20)
        #[arg(short, long, default_value = "20")]
        limit: usize,
        
        /// Offset for pagination (default: 0)
        #[arg(short, long, default_value = "0")]
        offset: usize,
    },
    
    /// View a specific plan with full details
    View {
        /// Plan ID to view
        plan_id: String,
        
        /// Show specific revision (default: latest)
        #[arg(short, long)]
        revision: Option<u32>,
    },
    
    /// Approve a plan for execution
    Approve {
        /// Plan ID to approve
        plan_id: String,
        
        /// User ID performing the approval
        #[arg(short, long)]
        user_id: String,
        
        /// Approval comment or reason
        #[arg(short = 'm', long)]
        message: Option<String>,
    },
    
    /// Get detailed plan status and revision history
    Status {
        /// Plan ID to check status
        plan_id: String,
        
        /// Show revision history
        #[arg(short = 'h', long)]
        history: bool,
    },
    
    /// Create a new plan from JSON file
    Create {
        /// Path to JSON file containing plan data
        #[arg(short, long)]
        file: String,
        
        /// User ID creating the plan
        #[arg(short, long)]
        user_id: String,
    },
}

fn init_logging(verbose: bool) {
    let level = if verbose { "debug" } else { "info" };
    
    tracing_subscriber::fmt()
        .with_env_filter(level)
        .with_target(false)
        .init();
}

fn format_plan_summary(plan: &UiPlan) -> String {
    let status_icon = if plan.approved { "✅" } else { "⏳" };
    let status_text = if plan.approved { "APPROVED" } else { "PENDING" };
    
    format!(
        "{} [{}] {} ({} steps)",
        status_icon,
        status_text,
        plan.id,
        plan.steps.len()
    )
}

async fn handle_list(status: Option<String>, creator: Option<String>, limit: usize, offset: usize) -> Result<()> {
    // Initialize the plan store
    let store = init_plan_store().await.context("Failed to initialize plan store")?;
    
    // Build query
    let query = ListQuery {
        status: status.clone(),
        creator: creator.clone(),
        limit,
        offset,
    };
    
    let response = store.list_plans(&query).await.context("Failed to list plans")?;
    
    if response.plans.is_empty() {
        println!("No plans found.");
        if let Some(status_filter) = &status {
            println!("(filtered by status: {})", status_filter);
        }
        if let Some(creator_filter) = &creator {
            println!("(filtered by creator: {})", creator_filter);
        }
        return Ok(());
    }
    
    println!("Plans ({} of {} total):", response.plans.len(), response.total_count);
    println!("==================={}", "=".repeat(response.total_count.to_string().len()));
    
    for record in &response.plans {
        let status_icon = match record.metadata.status {
            aetheris_ai::plan_store::PlanStatus::Pending => "⏳",
            aetheris_ai::plan_store::PlanStatus::Approved => "✅",
        };
        
        let status_text = match record.metadata.status {
            aetheris_ai::plan_store::PlanStatus::Pending => "PENDING",
            aetheris_ai::plan_store::PlanStatus::Approved => "APPROVED",
        };
        
        println!("{} [{}] {} v{} ({} steps)", 
            status_icon, 
            status_text, 
            record.metadata.plan_id, 
            record.metadata.revision,
            record.plan.steps.len()
        );
        
        println!("  Goal: {}", record.plan.user_goal);
        println!("  Creator: {} | Created: {}", 
            record.metadata.user_id, 
            record.metadata.created_at.format("%Y-%m-%d %H:%M UTC")
        );
        
        if record.metadata.status == aetheris_ai::plan_store::PlanStatus::Approved {
            if let Some(approved_by) = &record.metadata.approved_by {
                println!("  Approved by: {} at {}", 
                    approved_by,
                    record.metadata.approved_at
                        .map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string())
                        .unwrap_or_else(|| "unknown time".to_string())
                );
            }
        }
        
        println!();
    }
    
    // Show pagination info
    if response.has_more {
        println!("📄 Showing {} plans (offset {}). Use --offset {} to see more.", 
            limit, offset, offset + limit);
    }
    
    Ok(())
}

async fn handle_view(plan_id: &str, revision: Option<u32>) -> Result<()> {
    let store = init_plan_store().await.context("Failed to initialize plan store")?;
    
    match store.get_plan_by_id(plan_id).await.context("Failed to get plan")? {
        Some(record) => {
            // For now, just show the latest version. In a full implementation,
            // we'd support showing specific revisions from the log.
            if let Some(requested_rev) = revision {
                if requested_rev != record.metadata.revision {
                    println!("⚠️  Requested revision {} not available, showing latest (v{})", 
                        requested_rev, record.metadata.revision);
                }
            }
            
            let plan = &record.plan;
            let metadata = &record.metadata;
            
            println!("Plan Details: {} (v{})", metadata.plan_id, metadata.revision);
            println!("============={}", "=".repeat(metadata.plan_id.len() + metadata.revision.to_string().len() + 5));
            
            println!("Name: {}", plan.name);
            println!("Goal: {}", plan.user_goal);
            println!("Priority: {:?}", plan.priority);
            println!("Progress: {:.1}%", plan.progress * 100.0);
            println!("Creator: {}", metadata.user_id);
            println!("Created: {}", metadata.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
            
            // Show approval status
            println!("\nApproval Status:");
            println!("----------------");
            match metadata.status {
                aetheris_ai::plan_store::PlanStatus::Pending => {
                    println!("⏳ PENDING APPROVAL");
                }
                aetheris_ai::plan_store::PlanStatus::Approved => {
                    println!("✅ APPROVED");
                    if let Some(approved_by) = &metadata.approved_by {
                        println!("Approved by: {}", approved_by);
                    }
                    if let Some(approved_at) = metadata.approved_at {
                        println!("Approved at: {}", approved_at.format("%Y-%m-%d %H:%M:%S UTC"));
                    }
                }
            }
            
            // Show plan steps
            println!("\nPlan Steps ({}):", plan.steps.len());
            println!("-------------{}", "-".repeat(plan.steps.len().to_string().len()));
            
            for (idx, step) in plan.steps.iter().enumerate() {
                let status_icon = match step.status {
                    aetheris_ai::planner::StepStatus::Pending => "⏳",
                    aetheris_ai::planner::StepStatus::InProgress => "🔄",
                    aetheris_ai::planner::StepStatus::Completed => "✅",
                    aetheris_ai::planner::StepStatus::Failed => "❌",
                    aetheris_ai::planner::StepStatus::Skipped => "⏭️",
                };
                
                println!("{}. {} {} [{}]", 
                    idx + 1, 
                    status_icon, 
                    step.name, 
                    format!("{:?}", step.status).to_lowercase()
                );
                
                if !step.description.is_empty() {
                    println!("   Description: {}", step.description);
                }
                
                if let Some(duration) = step.estimated_duration_secs {
                    println!("   Estimated: {}s", duration);
                }
                
                if !step.dependencies.is_empty() {
                    println!("   Dependencies: {}", step.dependencies.join(", "));
                }
                
                println!();
            }
            
            // Show tags if any
            if !plan.tags.is_empty() {
                println!("Tags: {}", plan.tags.join(", "));
            }
        }
        None => {
            eprintln!("❌ Plan '{}' not found", plan_id);
            process::exit(1);
        }
    }
    
    Ok(())
}

async fn handle_approve(plan_id: &str, user_id: &str, message: Option<String>) -> Result<()> {
    let store = init_plan_store().await.context("Failed to initialize plan store")?;
    
    // First check if the plan exists and its current status
    match store.get_plan_by_id(plan_id).await.context("Failed to get plan")? {
        Some(record) => {
            match record.metadata.status {
                aetheris_ai::plan_store::PlanStatus::Approved => {
                    eprintln!("❌ Plan '{}' is already approved", plan_id);
                    if let Some(approved_by) = &record.metadata.approved_by {
                        eprintln!("   Previously approved by: {}", approved_by);
                    }
                    process::exit(1);
                }
                aetheris_ai::plan_store::PlanStatus::Pending => {
                    // Proceed with approval
                    match store.approve_plan(plan_id, user_id).await {
                        Ok(updated_record) => {
                            println!("✅ Plan '{}' approved successfully!", plan_id);
                            println!("Approved by: {}", user_id);
                            println!("New revision: v{}", updated_record.metadata.revision);
                            
                            if let Some(approved_at) = updated_record.metadata.approved_at {
                                println!("Approved at: {}", approved_at.format("%Y-%m-%d %H:%M:%S UTC"));
                            }
                            
                            if let Some(msg) = message {
                                println!("Message: {}", msg);
                            }
                            
                            // Show plan summary
                            println!("\nApproved Plan Summary:");
                            println!("=====================");
                            println!("Name: {}", updated_record.plan.name);
                            println!("Goal: {}", updated_record.plan.user_goal);
                            println!("Steps: {}", updated_record.plan.steps.len());
                            println!("Priority: {:?}", updated_record.plan.priority);
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to approve plan '{}': {}", plan_id, e);
                            process::exit(1);
                        }
                    }
                }
            }
        }
        None => {
            eprintln!("❌ Plan '{}' not found", plan_id);
            process::exit(1);
        }
    }
    
    Ok(())
}

async fn handle_status(plan_id: &str, show_history: bool) -> Result<()> {
    let store = init_plan_store().await.context("Failed to initialize plan store")?;
    
    match store.get_plan_by_id(plan_id).await.context("Failed to get plan")? {
        Some(record) => {
            let plan = &record.plan;
            let metadata = &record.metadata;
            
            println!("Plan Status: {} (v{})", metadata.plan_id, metadata.revision);
            println!("============={}", "=".repeat(metadata.plan_id.len() + metadata.revision.to_string().len() + 5));
            
            // Overall status
            match metadata.status {
                aetheris_ai::plan_store::PlanStatus::Pending => {
                    println!("Status: ⏳ PENDING APPROVAL");
                }
                aetheris_ai::plan_store::PlanStatus::Approved => {
                    println!("Status: ✅ APPROVED");
                    if let Some(approved_by) = &metadata.approved_by {
                        println!("Approved by: {}", approved_by);
                    }
                    if let Some(approved_at) = metadata.approved_at {
                        println!("Approved at: {}", approved_at.format("%Y-%m-%d %H:%M:%S UTC"));
                    }
                }
            }
            
            println!("Creator: {}", metadata.user_id);
            println!("Created: {}", metadata.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("Progress: {:.1}%", plan.progress * 100.0);
            
            // Step breakdown
            println!("\nStep Breakdown ({} total):", plan.steps.len());
            println!("-------------------------{}", "-".repeat(plan.steps.len().to_string().len()));
            
            let mut step_counts = std::collections::HashMap::new();
            
            for step in &plan.steps {
                *step_counts.entry(&step.status).or_insert(0) += 1;
                
                let status_icon = match step.status {
                    aetheris_ai::planner::StepStatus::Pending => "⏳",
                    aetheris_ai::planner::StepStatus::InProgress => "🔄",
                    aetheris_ai::planner::StepStatus::Completed => "✅",
                    aetheris_ai::planner::StepStatus::Failed => "❌",
                    aetheris_ai::planner::StepStatus::Skipped => "⏭️",
                };
                
                println!("  {} {} [{}]", status_icon, step.name, format!("{:?}", step.status).to_lowercase());
            }
            
            // Summary counts
            println!("\nStep Summary:");
            println!("------------");
            for (status, count) in step_counts {
                let icon = match status {
                    aetheris_ai::planner::StepStatus::Pending => "⏳",
                    aetheris_ai::planner::StepStatus::InProgress => "🔄",
                    aetheris_ai::planner::StepStatus::Completed => "✅",
                    aetheris_ai::planner::StepStatus::Failed => "❌",
                    aetheris_ai::planner::StepStatus::Skipped => "⏭️",
                };
                println!("  {} {}: {}", icon, format!("{:?}", status), count);
            }
            
            // Show revision history if requested (placeholder for now)
            if show_history {
                println!("\nRevision History:");
                println!("----------------");
                println!("v{}: Created by {} at {}", 
                    metadata.revision, 
                    metadata.user_id,
                    metadata.created_at.format("%Y-%m-%d %H:%M:%S UTC")
                );
                
                if metadata.status == aetheris_ai::plan_store::PlanStatus::Approved {
                    if let (Some(approved_by), Some(approved_at)) = (&metadata.approved_by, metadata.approved_at) {
                        println!("v{}: Approved by {} at {}", 
                            metadata.revision, 
                            approved_by,
                            approved_at.format("%Y-%m-%d %H:%M:%S UTC")
                        );
                    }
                }
                
                // In a full implementation, we'd parse the log file to show full history
                println!("  (Full revision history from log file not yet implemented)");
            }
        }
        None => {
            eprintln!("❌ Plan '{}' not found", plan_id);
            process::exit(1);
        }
    }
    
    Ok(())
}

async fn handle_create(file_path: &str, user_id: &str) -> Result<()> {
    use std::fs;
    use aetheris_ai::planner::Plan;
    
    let store = init_plan_store().await.context("Failed to initialize plan store")?;
    
    // Read and parse the JSON file
    let json_content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path))?;
    
    let plan: Plan = serde_json::from_str(&json_content)
        .with_context(|| format!("Failed to parse JSON from file: {}", file_path))?;
    
    // Store the plan
    match store.store_plan(&plan, user_id).await {
        Ok(record) => {
            println!("✅ Plan created successfully!");
            println!("Plan ID: {}", record.metadata.plan_id);
            println!("Revision: v{}", record.metadata.revision);
            println!("Name: {}", plan.name);
            println!("Goal: {}", plan.user_goal);
            println!("Steps: {}", plan.steps.len());
            println!("Creator: {}", user_id);
            println!("Created: {}", record.metadata.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
            
            println!("\nUse 'ai-plan view {}' to see the full plan details.", record.metadata.plan_id);
        }
        Err(e) => {
            eprintln!("❌ Failed to create plan: {}", e);
            process::exit(1);
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    init_logging(cli.verbose);
    
    let result = match cli.command {
        Commands::List { status, creator, limit, offset } => 
            handle_list(status, creator, limit, offset).await,
        Commands::View { plan_id, revision } => 
            handle_view(&plan_id, revision).await,
        Commands::Approve { plan_id, user_id, message } => 
            handle_approve(&plan_id, &user_id, message).await,
        Commands::Status { plan_id, history } => 
            handle_status(&plan_id, history).await,
        Commands::Create { file, user_id } => 
            handle_create(&file, &user_id).await,
    };
    
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aetheris_ai::planner::{Plan, Step, StepPriority, StepStatus};
    use aetheris_ai::plan_store::PlanStore;
    use std::collections::HashMap;
    use tempfile::TempDir;
    
    fn create_test_plan() -> Plan {
        Plan {
            id: "test-plan-123".to_string(),
            name: "Test Plan".to_string(),
            description: "A test plan for CLI testing".to_string(),
            user_goal: "Test the CLI functionality".to_string(),
            version: 1,
            priority: StepPriority::Normal,
            status: StepStatus::Pending,
            steps: vec![
                Step {
                    id: "step-1".to_string(),
                    name: "Initialize".to_string(),
                    description: "Initialize the test".to_string(),
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
    async fn test_create_and_list_plans() {
        let temp_dir = TempDir::new().unwrap();
        let store = PlanStore::new(temp_dir.path().to_path_buf()).await.unwrap();
        
        let plan = create_test_plan();
        let record = store.store_plan(&plan, "test-user").await.unwrap();
        
        // Test that we can list the plan
        let query = ListQuery {
            status: None,
            creator: None,
            limit: 10,
            offset: 0,
        };
        let response = store.list_plans(&query).await.unwrap();
        
        assert_eq!(response.plans.len(), 1);
        assert_eq!(response.plans[0].metadata.plan_id, record.metadata.plan_id);
        assert_eq!(response.total_count, 1);
        assert!(!response.has_more);
    }
    
    #[tokio::test]
    async fn test_approve_plan_flow() {
        let temp_dir = TempDir::new().unwrap();
        let store = PlanStore::new(temp_dir.path().to_path_buf()).await.unwrap();
        
        let plan = create_test_plan();
        let record = store.store_plan(&plan, "creator").await.unwrap();
        
        // Plan should start as pending
        assert_eq!(record.metadata.status, aetheris_ai::plan_store::PlanStatus::Pending);
        
        // Approve the plan
        let approved_record = store.approve_plan(&record.metadata.plan_id, "approver").await.unwrap();
        
        // Plan should now be approved
        assert_eq!(approved_record.metadata.status, aetheris_ai::plan_store::PlanStatus::Approved);
        assert_eq!(approved_record.metadata.approved_by, Some("approver".to_string()));
        assert!(approved_record.metadata.approved_at.is_some());
        assert_eq!(approved_record.metadata.revision, 2); // Should increment revision
    }
    
    #[tokio::test]
    async fn test_plan_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let store = PlanStore::new(temp_dir.path().to_path_buf()).await.unwrap();
        
        let result = store.get_plan_by_id("nonexistent-plan").await.unwrap();
        assert!(result.is_none());
        
        // Test approve nonexistent plan
        let result = store.approve_plan("nonexistent-plan", "approver").await;
        assert!(result.is_err());
    }
}