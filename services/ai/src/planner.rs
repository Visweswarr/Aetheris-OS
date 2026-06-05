//! AI Task Planner - Phase 5
//! 
//! Intelligent task planning with deterministic plan generation, step decomposition,
//! and LLM-based goal analysis. Provides structured planning capabilities for
//! complex multi-step tasks with deterministic serialization and execution tracking.
//! 
//! References:
//! - Task planning research: HTN planning, PDDL, and hierarchical task networks
//! - LLM planning: Chain-of-thought, ReAct, and plan-and-execute patterns
//! - Deterministic execution: Consistent plan generation with seeded randomness
//! - Error recovery: Plan adaptation and step retry mechanisms

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use ciborium::{from_reader, into_writer};
use uuid::Uuid;

use crate::error::AiError;
use crate::policy::AiPolicy;

/// Task planner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerConfig {
    /// Maximum plan depth (nested steps)
    pub max_plan_depth: usize,
    /// Maximum steps per plan
    pub max_steps_per_plan: usize,
    /// Enable deterministic plan generation
    pub deterministic: bool,
    /// Random seed for deterministic planning
    pub random_seed: u64,
    /// LLM model to use for planning
    pub llm_model: String,
    /// Maximum context tokens for LLM
    pub max_context_tokens: usize,
    /// Enable plan optimization
    pub enable_optimization: bool,
    /// Plan timeout in seconds
    pub plan_timeout_secs: u64,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            max_plan_depth: 5,
            max_steps_per_plan: 20,
            deterministic: true,
            random_seed: 42,
            llm_model: "llama3-8b".to_string(),
            max_context_tokens: 4096,
            enable_optimization: true,
            plan_timeout_secs: 30,
        }
    }
}

/// Plan step priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Default)]
pub enum StepPriority {
    /// Low priority step
    Low,
    /// Normal priority step
    #[default]
    Normal,
    /// High priority step
    High,
    /// Critical priority step
    Critical,
}


/// Step execution strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub enum ExecutionStrategy {
    /// Execute step sequentially
    #[default]
    Sequential,
    /// Execute step in parallel with others
    Parallel,
    /// Execute step conditionally based on previous results
    Conditional(String),
    /// Execute step with retry mechanism
    Retry { max_attempts: u32, delay_ms: u64 },
}


/// Plan step status with detailed execution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanStepStatus {
    /// Step is pending execution
    Pending,
    /// Step is currently executing
    Executing {
        /// Start timestamp (90kHz timebase)
        started_at: u64,
        /// Current attempt number
        attempt: u32,
    },
    /// Step completed successfully
    Completed {
        /// Completion timestamp
        completed_at: u64,
        /// Execution time in microseconds
        execution_time_us: u64,
    },
    /// Step failed with error
    Failed {
        /// Failure timestamp
        failed_at: u64,
        /// Error message
        error: String,
        /// Number of attempts made
        attempts: u32,
    },
    /// Step was skipped
    Skipped {
        /// Skip timestamp
        skipped_at: u64,
        /// Reason for skipping
        reason: String,
    },
    /// Step is blocked waiting for dependencies
    Blocked {
        /// Blocked timestamp
        blocked_at: u64,
        /// Dependencies that must complete
        dependencies: Vec<String>,
    },
}

/// Individual plan step with rich metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    /// Unique step identifier
    pub id: String,
    /// Step name/title
    pub name: String,
    /// Detailed step description
    pub description: String,
    /// Step type (action, decision, validation, etc.)
    pub step_type: String,
    /// Step priority
    pub priority: StepPriority,
    /// Execution strategy
    pub execution_strategy: ExecutionStrategy,
    /// Current status
    pub status: PlanStepStatus,
    /// Step dependencies (other step IDs)
    pub dependencies: Vec<String>,
    /// Expected duration in microseconds
    pub estimated_duration_us: Option<u64>,
    /// Tool to invoke (if any)
    pub tool_id: Option<String>,
    /// Tool parameters (JSON)
    pub tool_params: Option<serde_json::Value>,
    /// Expected outputs
    pub expected_outputs: Vec<String>,
    /// Actual outputs after execution
    pub actual_outputs: Option<serde_json::Value>,
    /// Sub-steps for hierarchical planning
    pub sub_steps: Vec<PlanStep>,
    /// Step metadata
    pub metadata: HashMap<String, String>,
}

impl PlanStep {
    /// Create a new plan step
    pub fn new(name: &str, description: &str, step_type: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.to_string(),
            step_type: step_type.to_string(),
            priority: StepPriority::default(),
            execution_strategy: ExecutionStrategy::default(),
            status: PlanStepStatus::Pending,
            dependencies: Vec::new(),
            estimated_duration_us: None,
            tool_id: None,
            tool_params: None,
            expected_outputs: Vec::new(),
            actual_outputs: None,
            sub_steps: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a dependency to this step
    pub fn add_dependency(&mut self, step_id: &str) {
        if !self.dependencies.contains(&step_id.to_string()) {
            self.dependencies.push(step_id.to_string());
        }
    }

    /// Set tool for this step
    pub fn set_tool(&mut self, tool_id: &str, params: serde_json::Value) {
        self.tool_id = Some(tool_id.to_string());
        self.tool_params = Some(params);
    }

    /// Check if step is ready to execute (all dependencies completed)
    pub fn is_ready_to_execute(&self, completed_steps: &[String]) -> bool {
        self.dependencies.iter().all(|dep| completed_steps.contains(dep))
    }

    /// Get total estimated duration including sub-steps
    pub fn get_total_estimated_duration(&self) -> u64 {
        let own_duration = self.estimated_duration_us.unwrap_or(0);
        let sub_duration: u64 = self.sub_steps.iter()
            .map(|step| step.get_total_estimated_duration())
            .sum();
        own_duration + sub_duration
    }
}

/// Plan status with execution tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanStatus {
    /// Plan is being generated
    Generating,
    /// Plan is ready for execution
    Ready,
    /// Plan is currently executing
    Executing {
        /// Execution start timestamp
        started_at: u64,
        /// Currently executing step ID
        current_step: Option<String>,
        /// Progress percentage (0.0 to 1.0)
        progress: f64,
    },
    /// Plan completed successfully
    Completed {
        /// Completion timestamp
        completed_at: u64,
        /// Total execution time in microseconds
        execution_time_us: u64,
        /// Success rate (0.0 to 1.0)
        success_rate: f64,
    },
    /// Plan failed during execution
    Failed {
        /// Failure timestamp
        failed_at: u64,
        /// Error message
        error: String,
        /// Step ID where failure occurred
        failed_step: Option<String>,
    },
    /// Plan was cancelled
    Cancelled {
        /// Cancellation timestamp
        cancelled_at: u64,
        /// Reason for cancellation
        reason: String,
    },
    /// Plan is paused
    Paused {
        /// Pause timestamp
        paused_at: u64,
        /// Reason for pausing
        reason: String,
    },
}

/// Comprehensive task plan with hierarchical structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    /// Unique plan identifier
    pub id: String,
    /// Plan name/title
    pub name: String,
    /// Plan description
    pub description: String,
    /// Original user goal/request
    pub user_goal: String,
    /// Plan version (for updates/revisions)
    pub version: u32,
    /// Plan priority
    pub priority: StepPriority,
    /// Current status
    pub status: PlanStatus,
    /// Plan steps
    pub steps: Vec<PlanStep>,
    /// Plan metadata
    pub metadata: HashMap<String, String>,
    /// Plan tags for categorization
    pub tags: Vec<String>,
    /// Plan creator (user, agent, etc.)
    pub creator: String,
    /// Created timestamp (90kHz timebase)
    pub created_at: u64,
    /// Last updated timestamp
    pub updated_at: u64,
    /// Estimated total duration in microseconds
    pub estimated_duration_us: u64,
    /// Actual execution time (if completed)
    pub actual_duration_us: Option<u64>,
    /// Plan revision history
    pub revisions: Vec<PlanRevision>,
}

/// Plan revision for tracking changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanRevision {
    /// Revision number
    pub revision: u32,
    /// Revision timestamp
    pub timestamp: u64,
    /// Changes made
    pub changes: String,
    /// Revision author
    pub author: String,
}

impl Plan {
    /// Create a new plan
    pub fn new(name: &str, description: &str, user_goal: &str, creator: &str) -> Self {
        let now = Self::current_timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.to_string(),
            user_goal: user_goal.to_string(),
            version: 1,
            priority: StepPriority::Normal,
            status: PlanStatus::Generating,
            steps: Vec::new(),
            metadata: HashMap::new(),
            tags: Vec::new(),
            creator: creator.to_string(),
            created_at: now,
            updated_at: now,
            estimated_duration_us: 0,
            actual_duration_us: None,
            revisions: Vec::new(),
        }
    }

    /// Add a step to the plan
    pub fn add_step(&mut self, step: PlanStep) {
        self.estimated_duration_us += step.get_total_estimated_duration();
        self.steps.push(step);
        self.updated_at = Self::current_timestamp();
    }

    /// Get steps ready for execution
    pub fn get_ready_steps(&self) -> Vec<&PlanStep> {
        let completed_steps: Vec<String> = self.steps.iter()
            .filter_map(|step| match &step.status {
                PlanStepStatus::Completed { .. } => Some(step.id.clone()),
                _ => None,
            })
            .collect();

        self.steps.iter()
            .filter(|step| matches!(step.status, PlanStepStatus::Pending))
            .filter(|step| step.is_ready_to_execute(&completed_steps))
            .collect()
    }

    /// Calculate plan progress (0.0 to 1.0)
    pub fn calculate_progress(&self) -> f64 {
        if self.steps.is_empty() {
            return 0.0;
        }

        let completed_count = self.steps.iter()
            .filter(|step| matches!(step.status, PlanStepStatus::Completed { .. }))
            .count();

        completed_count as f64 / self.steps.len() as f64
    }

    /// Get current timestamp (90kHz timebase)
    fn current_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        duration
            .as_secs()
            .saturating_mul(90_000)
            .saturating_add((duration.subsec_nanos() as u64).saturating_mul(90_000) / 1_000_000_000)
    }

    /// Add a revision to the plan
    pub fn add_revision(&mut self, changes: &str, author: &str) {
        self.version += 1;
        self.updated_at = Self::current_timestamp();
        self.revisions.push(PlanRevision {
            revision: self.version,
            timestamp: self.updated_at,
            changes: changes.to_string(),
            author: author.to_string(),
        });
    }
}

/// Plan generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanRequest {
    /// User goal/objective
    pub goal: String,
    /// Additional context or constraints
    pub context: Option<String>,
    /// Required tools/capabilities
    pub required_tools: Vec<String>,
    /// Plan priority
    pub priority: StepPriority,
    /// Maximum execution time in seconds
    pub max_execution_time: Option<u64>,
    /// Tags for the plan
    pub tags: Vec<String>,
    /// Request metadata
    pub metadata: HashMap<String, String>,
}

/// Plan generation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanResult {
    /// Generated plan
    pub plan: Option<Plan>,
    /// Generation success
    pub success: bool,
    /// Generation time in microseconds
    pub generation_time_us: u64,
    /// Error message if failed
    pub error: Option<String>,
    /// LLM usage statistics
    pub llm_stats: LlmUsageStats,
}

/// LLM usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmUsageStats {
    /// Input tokens used
    pub input_tokens: u32,
    /// Output tokens generated
    pub output_tokens: u32,
    /// Total inference time in microseconds
    pub inference_time_us: u64,
    /// Model used
    pub model: String,
}

/// Task planner trait for intelligent plan generation
#[async_trait::async_trait]
pub trait TaskPlanner: Send + Sync {
    /// Initialize the planner with configuration
    async fn init(&self, config: PlannerConfig) -> Result<(), AiError>;
    
    /// Generate a plan from a user goal
    async fn generate_plan(&self, request: PlanRequest) -> Result<PlanResult, AiError>;
    
    /// Update an existing plan with new information
    async fn update_plan(&self, plan_id: &str, updates: PlanRequest) -> Result<Plan, AiError>;
    
    /// Get a plan by ID
    async fn get_plan(&self, plan_id: &str) -> Result<Plan, AiError>;
    
    /// List all plans
    async fn list_plans(&self) -> Result<Vec<Plan>, AiError>;
    
    /// Delete a plan
    async fn delete_plan(&self, plan_id: &str) -> Result<(), AiError>;
    
    /// Execute a plan step
    async fn execute_step(&self, plan_id: &str, step_id: &str) -> Result<PlanStep, AiError>;
    
    /// Get planner statistics
    async fn get_stats(&self) -> Result<PlannerStats, AiError>;
    
    /// Serialize plan to deterministic bytes
    async fn serialize_plan(&self, plan: &Plan) -> Result<Vec<u8>, AiError>;
    
    /// Deserialize plan from bytes
    async fn deserialize_plan(&self, data: &[u8]) -> Result<Plan, AiError>;
}

/// Planner statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerStats {
    /// Total plans generated
    pub total_plans: u64,
    /// Active plans
    pub active_plans: u64,
    /// Completed plans
    pub completed_plans: u64,
    /// Failed plans
    pub failed_plans: u64,
    /// Average plan generation time (microseconds)
    pub avg_generation_time_us: u64,
    /// Average plan success rate (0.0 to 1.0)
    pub avg_success_rate: f64,
    /// Total LLM tokens used
    pub total_tokens_used: u64,
}

/// Stub implementation of the Task Planner
pub struct StubTaskPlanner {
    config: Arc<RwLock<Option<PlannerConfig>>>,
    plans: Arc<RwLock<HashMap<String, Plan>>>,
    stats: Arc<RwLock<PlannerStats>>,
    policy: Arc<AiPolicy>,
}

impl StubTaskPlanner {
    /// Create a new stub task planner
    pub fn new(policy: Arc<AiPolicy>) -> Self {
        Self {
            config: Arc::new(RwLock::new(None)),
            plans: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(PlannerStats {
                total_plans: 0,
                active_plans: 0,
                completed_plans: 0,
                failed_plans: 0,
                avg_generation_time_us: 0,
                avg_success_rate: 0.0,
                total_tokens_used: 0,
            })),
            policy,
        }
    }

    /// Generate a deterministic seed from input
    fn deterministic_seed(input: &str, base_seed: u64) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        base_seed.hash(&mut hasher);
        hasher.finish()
    }

    /// Generate sample steps for a given goal (stub implementation)
    fn generate_sample_steps(goal: &str, seed: u64) -> Vec<PlanStep> {
        // This is a stub implementation - in a real system, this would use an LLM
        let mut rng = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            seed.hash(&mut hasher);
            hasher.finish()
        };

        let step_templates = [("Analyze", "Analyze the requirements and constraints", "analysis"),
            ("Plan", "Create detailed implementation plan", "planning"),
            ("Setup", "Set up necessary resources and environment", "setup"),
            ("Implement", "Implement the core functionality", "implementation"),
            ("Test", "Test the implementation thoroughly", "testing"),
            ("Deploy", "Deploy the solution", "deployment"),
            ("Verify", "Verify the solution meets requirements", "verification")];

        let num_steps = ((rng % 5) + 3) as usize; // 3-7 steps
        let mut steps: Vec<PlanStep> = Vec::new();

        for i in 0..num_steps.min(step_templates.len()) {
            let template = &step_templates[i];
            let mut step = PlanStep::new(
                &format!("{} {}", template.0, goal),
                &format!("{} for: {}", template.1, goal),
                template.2,
            );
            
            step.estimated_duration_us = Some((((rng % 300000) + 60000))); // 1-5 minutes
            
            // Add dependencies (each step depends on previous one)
            if i > 0 {
                step.add_dependency(&steps[i-1].id);
            }
            
            steps.push(step);
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345); // Simple LCG
        }

        steps
    }
}

#[async_trait::async_trait]
impl TaskPlanner for StubTaskPlanner {
    async fn init(&self, config: PlannerConfig) -> Result<(), AiError> {
        let mut config_guard = self.config.write().await;
        *config_guard = Some(config);
        Ok(())
    }

    async fn generate_plan(&self, request: PlanRequest) -> Result<PlanResult, AiError> {
        let start_time = std::time::Instant::now();
        
        let config = self.config.read().await;
        let config = config.as_ref().ok_or_else(|| {
            AiError::configuration("Planner not initialized")
        })?;

        // Generate deterministic seed
        let seed = if config.deterministic {
            Self::deterministic_seed(&request.goal, config.random_seed)
        } else {
            rand::random()
        };

        // Create plan
        let mut plan = Plan::new(
            &format!("Plan for: {}", request.goal),
            &format!("Generated plan to accomplish: {}", request.goal),
            &request.goal,
            "ai-planner",
        );

        plan.priority = request.priority.clone();
        plan.tags = request.tags.clone();
        plan.metadata = request.metadata.clone();

        // Generate steps
        let steps = Self::generate_sample_steps(&request.goal, seed);
        for step in steps {
            plan.add_step(step);
        }

        plan.status = PlanStatus::Ready;

        // Store plan
        let plan_id = plan.id.clone();
        {
            let mut plans = self.plans.write().await;
            plans.insert(plan_id, plan.clone());
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_plans += 1;
            stats.active_plans += 1;
        }

        let generation_time = start_time.elapsed().as_micros() as u64;

        Ok(PlanResult {
            plan: Some(plan),
            success: true,
            generation_time_us: generation_time,
            error: None,
            llm_stats: LlmUsageStats {
                input_tokens: request.goal.len() as u32 / 4, // Rough token estimate
                output_tokens: 100, // Stub value
                inference_time_us: generation_time / 2,
                model: config.llm_model.clone(),
            },
        })
    }

    async fn update_plan(&self, plan_id: &str, _updates: PlanRequest) -> Result<Plan, AiError> {
        let mut plans = self.plans.write().await;
        let plan = plans.get_mut(plan_id)
            .ok_or_else(|| AiError::validation(format!("Plan not found: {}", plan_id)))?;

        plan.add_revision("Updated with new requirements", "ai-planner");
        // In a real implementation, we would regenerate steps based on updates
        
        Ok(plan.clone())
    }

    async fn get_plan(&self, plan_id: &str) -> Result<Plan, AiError> {
        let plans = self.plans.read().await;
        plans.get(plan_id)
            .cloned()
            .ok_or_else(|| AiError::validation(format!("Plan not found: {}", plan_id)))
    }

    async fn list_plans(&self) -> Result<Vec<Plan>, AiError> {
        let plans = self.plans.read().await;
        Ok(plans.values().cloned().collect())
    }

    async fn delete_plan(&self, plan_id: &str) -> Result<(), AiError> {
        let mut plans = self.plans.write().await;
        plans.remove(plan_id)
            .ok_or_else(|| AiError::validation(format!("Plan not found: {}", plan_id)))?;

        let mut stats = self.stats.write().await;
        stats.active_plans = stats.active_plans.saturating_sub(1);
        
        Ok(())
    }

    async fn execute_step(&self, plan_id: &str, step_id: &str) -> Result<PlanStep, AiError> {
        let mut plans = self.plans.write().await;
        let plan = plans.get_mut(plan_id)
            .ok_or_else(|| AiError::validation(format!("Plan not found: {}", plan_id)))?;

        let step = plan.steps.iter_mut()
            .find(|s| s.id == step_id)
            .ok_or_else(|| AiError::validation(format!("Step not found: {}", step_id)))?;

        // Simulate step execution
        let now = Plan::current_timestamp();
        step.status = PlanStepStatus::Executing {
            started_at: now,
            attempt: 1,
        };

        // Simulate some processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let execution_time = 100_000; // 100ms in microseconds
        step.status = PlanStepStatus::Completed {
            completed_at: now + execution_time,
            execution_time_us: execution_time,
        };

        step.actual_outputs = Some(serde_json::json!({
            "status": "completed",
            "result": "Step executed successfully"
        }));

        Ok(step.clone())
    }

    async fn get_stats(&self) -> Result<PlannerStats, AiError> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }

    async fn serialize_plan(&self, plan: &Plan) -> Result<Vec<u8>, AiError> {
        let mut buffer = Vec::new();
        into_writer(plan, &mut buffer)
            .map_err(|e| AiError::serialization(format!("Failed to serialize plan: {}", e)))?;
        Ok(buffer)
    }

    async fn deserialize_plan(&self, data: &[u8]) -> Result<Plan, AiError> {
        from_reader(data)
            .map_err(|e| AiError::deserialization(format!("Failed to deserialize plan: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::AiPolicy;

    #[tokio::test]
    async fn test_plan_creation() {
        let plan = Plan::new(
            "Test Plan",
            "A test plan for validation",
            "Test goal",
            "test-user"
        );
        
        assert_eq!(plan.name, "Test Plan");
        assert_eq!(plan.user_goal, "Test goal");
        assert_eq!(plan.creator, "test-user");
        assert_eq!(plan.version, 1);
    }

    #[tokio::test]
    async fn test_step_dependencies() {
        let mut step1 = PlanStep::new("Step 1", "First step", "setup");
        let step2 = PlanStep::new("Step 2", "Second step", "implementation");
        
        step1.add_dependency(&step2.id);
        assert!(step1.dependencies.contains(&step2.id));
        
        // Step should not be ready if dependency not completed
        assert!(!step1.is_ready_to_execute(&[]));
        
        // Step should be ready if dependency is completed
        assert!(step1.is_ready_to_execute(&[step2.id.clone()]));
    }

    #[tokio::test]
    async fn test_stub_planner() {
        let policy = Arc::new(AiPolicy::default());
        let planner = StubTaskPlanner::new(policy);
        
        let config = PlannerConfig::default();
        planner.init(config).await.unwrap();
        
        let request = PlanRequest {
            goal: "Create a simple web application".to_string(),
            context: None,
            required_tools: vec![],
            priority: StepPriority::Normal,
            max_execution_time: None,
            tags: vec!["web".to_string(), "development".to_string()],
            metadata: HashMap::new(),
        };
        
        let result = planner.generate_plan(request).await.unwrap();
        assert!(result.success);
        assert!(result.plan.is_some());
        
        let plan = result.plan.unwrap();
        assert!(!plan.steps.is_empty());
        assert_eq!(plan.user_goal, "Create a simple web application");
    }
}
