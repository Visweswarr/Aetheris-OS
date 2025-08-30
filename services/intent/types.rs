//! Planner Types
//!
//! Strongly-typed data structures for intent planning, actions, plans, and previews.
//! Provides comprehensive serialization, validation, and type safety for the Polymera OS
//! intent planning system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;
use validator::{Validate, ValidationError as ValidatorError};

/// Errors that can occur during planner type operations
#[derive(Debug, Error)]
pub enum PlannerError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid intent type: {0}")]
    InvalidIntentType(String),

    #[error("Invalid action type: {0}")]
    InvalidActionType(String),

    #[error("Invalid status transition: from {from} to {to}")]
    InvalidStatusTransition { from: String, to: String },

    #[error("Dependency cycle detected: {0}")]
    DependencyCycle(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid constraint: {0}")]
    InvalidConstraint(String),

    #[error("Plan validation failed: {0}")]
    PlanValidation(String),

    #[error("Preview generation failed: {0}")]
    PreviewGeneration(String),
}

type Result<T> = std::result::Result<T, PlannerError>;

/// Core Intent representing a user's desired outcome
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Intent {
    /// Unique identifier
    #[validate(length(min = 1))]
    pub id: String,
    
    /// Human-readable description
    #[validate(length(min = 10, max = 1000))]
    pub description: String,
    
    /// Type of intent
    pub intent_type: IntentType,
    
    /// Priority level (0 = lowest, 100 = highest)
    #[validate(range(min = 0, max = 100))]
    pub priority: u8,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Optional execution time
    pub execute_at: Option<DateTime<Utc>>,
    
    /// Optional expiration time
    pub expires_at: Option<DateTime<Utc>>,
    
    /// User who created this intent
    #[validate(length(min = 1))]
    pub user_id: String,
    
    /// Session context
    pub session_id: Option<String>,
    
    /// Intent-specific parameters
    pub parameters: HashMap<String, serde_json::Value>,
    
    /// Constraints that must be satisfied
    #[validate]
    pub constraints: Vec<Constraint>,
    
    /// Current status
    pub status: IntentStatus,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Types of intents supported by the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntentType {
    // Financial intents
    Transfer,
    Payment,
    Trade,
    Stake,
    Unstake,
    
    // Identity and access intents
    Authenticate,
    Authorize,
    RevokeAccess,
    CreateIdentity,
    
    // Data and privacy intents
    ShareData,
    RevokeData,
    BackupData,
    DeleteData,
    
    // System administration intents
    Configure,
    Deploy,
    Monitor,
    Maintenance,
    
    // Communication intents
    Message,
    Notification,
    Broadcast,
    
    // Custom intent for extensibility
    Custom(String),
}

/// Status of an intent in the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntentStatus {
    Pending,      // Waiting to be processed
    Planning,     // Plan is being generated
    Planned,      // Plan is ready for preview
    Previewing,   // User is reviewing the plan
    Approved,     // User approved the plan
    Executing,    // Plan is being executed
    Completed,    // Successfully completed
    Failed,       // Execution failed
    Cancelled,    // Cancelled by user
    Expired,      // Expired before execution
}

/// Constraints that must be satisfied for intent execution
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Constraint {
    /// Type of constraint
    pub constraint_type: ConstraintType,
    
    /// Human-readable description
    #[validate(length(min = 5, max = 500))]
    pub description: String,
    
    /// Constraint-specific parameters
    pub parameters: HashMap<String, serde_json::Value>,
    
    /// Whether this constraint is required
    pub required: bool,
    
    /// Priority of this constraint (higher = more important)
    #[validate(range(min = 0, max = 100))]
    pub priority: u8,
}

/// Types of constraints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintType {
    // Time-based constraints
    TimeWindow,    // Must execute within time window
    Deadline,      // Must complete before deadline
    Delay,         // Must wait before executing
    
    // Financial constraints
    BudgetLimit,   // Maximum cost/amount
    BalanceCheck,  // Ensure sufficient balance
    RateLimit,     // Transaction rate limiting
    
    // Security constraints
    Authentication, // Require specific auth level
    Authorization,  // Require specific permissions
    PrivacyLevel,   // Minimum privacy protection
    
    // Resource constraints
    ResourceLimit,  // CPU/memory/storage limits
    NetworkLimit,   // Bandwidth/latency limits
    Dependency,     // Depends on other intents/resources
    
    // Geographic constraints
    Geographic,     // Geographic restrictions
    Jurisdiction,   // Legal jurisdiction requirements
    
    // Custom constraint for extensibility
    Custom(String),
}

/// Individual atomic action that can be executed
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Action {
    /// Unique identifier
    #[validate(length(min = 1))]
    pub id: String,
    
    /// Human-readable description
    #[validate(length(min = 5, max = 500))]
    pub description: String,
    
    /// Type of action
    pub action_type: ActionType,
    
    /// Service or component that will execute this action
    #[validate(length(min = 1))]
    pub executor: String,
    
    /// Action-specific parameters
    pub parameters: HashMap<String, serde_json::Value>,
    
    /// Expected duration for this action
    pub estimated_duration: Option<Duration>,
    
    /// Cost estimate
    pub cost_estimate: Option<CostEstimate>,
    
    /// Risk assessment
    pub risk_assessment: Option<RiskAssessment>,
    
    /// Dependencies on other actions (action IDs)
    pub depends_on: Vec<String>,
    
    /// Current status
    pub status: ActionStatus,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Start execution timestamp
    pub started_at: Option<DateTime<Utc>>,
    
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    
    /// Error information if action failed
    pub error_message: Option<String>,
    
    /// Retry configuration
    pub retry_config: Option<RetryConfig>,
    
    /// Whether this action can be undone
    pub reversible: bool,
    
    /// Action to undo this action (if reversible)
    pub undo_action: Option<Box<Action>>,
}

/// Types of actions that can be executed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    // Transaction actions
    SendTransaction,
    SignMessage,
    ApproveToken,
    RevokeApproval,
    
    // Contract interaction actions
    DeployContract,
    CallContract,
    UpgradeContract,
    
    // Identity and authentication actions
    CreateIdentity,
    VerifyIdentity,
    RotateKeys,
    BackupKeys,
    
    // Data management actions
    StoreData,
    RetrieveData,
    DeleteData,
    EncryptData,
    DecryptData,
    
    // Network actions
    SendMessage,
    EstablishConnection,
    CloseConnection,
    
    // System actions
    ConfigureSystem,
    StartService,
    StopService,
    RestartService,
    
    // Validation actions
    ValidateSignature,
    VerifyProof,
    CheckBalance,
    
    // Custom action for extensibility
    Custom(String),
}

/// Status of an individual action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    Pending,     // Waiting to be executed
    Ready,       // Dependencies satisfied, ready to execute
    Executing,   // Currently being executed
    Completed,   // Successfully completed
    Failed,      // Execution failed
    Cancelled,   // Cancelled before execution
    Skipped,     // Skipped due to conditions
    Retrying,    // Retrying after failure
}

/// Cost estimation for an action
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CostEstimate {
    /// Estimated gas cost
    pub gas_estimate: u64,
    
    /// Gas price estimate
    pub gas_price: u64,
    
    /// Total cost in native currency
    #[validate(length(min = 1))]
    pub total_cost: String,
    
    /// Currency denomination
    #[validate(length(min = 1))]
    pub currency: String,
    
    /// Network fees
    pub network_fee: Option<String>,
    
    /// Service fees
    pub service_fee: Option<String>,
    
    /// Confidence level of the estimate (0.0 - 1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    pub confidence: f64,
    
    /// Additional cost breakdown
    pub cost_breakdown: HashMap<String, String>,
}

/// Risk assessment for an action
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RiskAssessment {
    /// Overall risk level
    pub risk_level: RiskLevel,
    
    /// Risk score (0.0 = no risk, 1.0 = maximum risk)
    #[validate(range(min = 0.0, max = 1.0))]
    pub risk_score: f64,
    
    /// Specific risk factors
    #[validate]
    pub risk_factors: Vec<RiskFactor>,
    
    /// Mitigation strategies
    pub mitigations: Vec<String>,
    
    /// Confidence in risk assessment
    #[validate(range(min = 0.0, max = 1.0))]
    pub confidence: f64,
}

/// Risk levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
    Critical,
}

/// Individual risk factor
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RiskFactor {
    /// Type of risk
    #[validate(length(min = 1))]
    pub risk_type: String,
    
    /// Description of the risk
    #[validate(length(min = 5, max = 500))]
    pub description: String,
    
    /// Impact if risk materializes
    pub impact: RiskLevel,
    
    /// Probability of risk occurring
    #[validate(range(min = 0.0, max = 1.0))]
    pub probability: f64,
    
    /// Recommended mitigation
    pub mitigation: Option<String>,
}

/// Retry configuration for actions
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RetryConfig {
    /// Maximum number of retries
    #[validate(range(min = 0, max = 10))]
    pub max_retries: u32,
    
    /// Initial delay between retries
    pub initial_delay: Duration,
    
    /// Maximum delay between retries
    pub max_delay: Duration,
    
    /// Backoff multiplier
    #[validate(range(min = 1.0, max = 10.0))]
    pub backoff_multiplier: f64,
    
    /// Whether to use exponential backoff
    pub exponential_backoff: bool,
    
    /// Conditions that trigger a retry
    pub retry_conditions: Vec<RetryCondition>,
}

/// Conditions under which an action should be retried
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryCondition {
    /// Type of condition
    pub condition_type: RetryConditionType,
    
    /// Condition-specific parameters
    pub parameters: HashMap<String, String>,
}

/// Types of retry conditions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetryConditionType {
    NetworkError,
    Timeout,
    InsufficientGas,
    NonceError,
    RateLimited,
    ServiceUnavailable,
    Custom(String),
}

/// Execution plan containing ordered actions to fulfill an intent
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Plan {
    /// Unique identifier
    #[validate(length(min = 1))]
    pub id: String,
    
    /// Intent this plan is for
    #[validate(length(min = 1))]
    pub intent_id: String,
    
    /// Version of this plan (incremented on updates)
    #[validate(range(min = 1))]
    pub version: u32,
    
    /// Human-readable description of the plan
    #[validate(length(min = 10, max = 1000))]
    pub description: String,
    
    /// Actions to be executed, in dependency order
    #[validate]
    pub actions: Vec<Action>,
    
    /// Execution strategy
    pub strategy: ExecutionStrategy,
    
    /// Total estimated duration
    pub estimated_duration: Option<Duration>,
    
    /// Total cost estimate
    pub total_cost_estimate: Option<CostEstimate>,
    
    /// Overall risk assessment
    pub risk_assessment: Option<RiskAssessment>,
    
    /// Plan status
    pub status: PlanStatus,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    
    /// Who/what generated this plan
    #[validate(length(min = 1))]
    pub created_by: String,
    
    /// Simulation results
    pub simulation_result: Option<SimulationResult>,
    
    /// Rollback plan in case of failure
    pub rollback_plan: Option<Box<Plan>>,
    
    /// Plan validation results
    pub validation_result: Option<ValidationResult>,
    
    /// Whether this plan requires user approval
    pub requires_approval: bool,
    
    /// Approval status if required
    pub approval_status: Option<ApprovalStatus>,
}

/// Execution strategies for plans
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStrategy {
    Sequential,    // Execute actions one by one
    Parallel,      // Execute independent actions in parallel
    Optimistic,    // Execute optimistically with rollback
    Conservative,  // Execute with maximum safety checks
    Atomic,        // All actions succeed or all fail
    BestEffort,    // Execute as many actions as possible
}

/// Status of a plan
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Draft,         // Plan is being created
    Validating,    // Plan is being validated
    Valid,         // Plan is valid and ready
    Invalid,       // Plan validation failed
    Simulating,    // Plan is being simulated
    Simulated,     // Simulation completed
    Approved,      // Plan approved for execution
    Rejected,      // Plan rejected by user/system
    Executing,     // Plan is being executed
    Completed,     // Plan executed successfully
    Failed,        // Plan execution failed
    Cancelled,     // Plan cancelled by user
    RolledBack,    // Plan was rolled back
}

/// Results of plan simulation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SimulationResult {
    /// Whether simulation was successful
    pub success: bool,
    
    /// Simulated execution time
    pub execution_time: Option<Duration>,
    
    /// Simulated final state
    pub final_state: HashMap<String, serde_json::Value>,
    
    /// State changes during simulation
    #[validate]
    pub state_changes: Vec<StateChange>,
    
    /// Warnings encountered during simulation
    pub warnings: Vec<String>,
    
    /// Errors encountered during simulation
    pub errors: Vec<String>,
    
    /// Confidence in simulation results
    #[validate(range(min = 0.0, max = 1.0))]
    pub confidence: f64,
    
    /// Resource usage during simulation
    pub resource_usage: Option<ResourceUsage>,
}

/// A state change during plan execution
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct StateChange {
    /// Component or resource affected
    #[validate(length(min = 1))]
    pub component: String,
    
    /// Property that changed
    #[validate(length(min = 1))]
    pub property: String,
    
    /// Previous value
    pub old_value: Option<serde_json::Value>,
    
    /// New value
    pub new_value: serde_json::Value,
    
    /// When the change occurred
    pub timestamp: DateTime<Utc>,
    
    /// Action that caused this change
    pub action_id: Option<String>,
}

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ResourceUsage {
    /// CPU usage (percentage)
    #[validate(range(min = 0.0, max = 100.0))]
    pub cpu_usage: Option<f64>,
    
    /// Memory usage (bytes)
    pub memory_usage: Option<u64>,
    
    /// Network bandwidth used (bytes)
    pub network_usage: Option<u64>,
    
    /// Storage used (bytes)
    pub storage_usage: Option<u64>,
    
    /// Gas consumed
    pub gas_consumed: Option<u64>,
    
    /// Custom resource metrics
    pub custom_metrics: HashMap<String, f64>,
}

/// Plan validation results
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidationResult {
    /// Whether plan is valid
    pub is_valid: bool,
    
    /// Validation errors
    #[validate]
    pub errors: Vec<ValidationError>,
    
    /// Validation warnings
    #[validate]
    pub warnings: Vec<ValidationWarning>,
    
    /// Suggestions for improvement
    pub suggestions: Vec<String>,
    
    /// Validation score (0.0 - 1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    pub validation_score: f64,
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidationError {
    /// Error code
    #[validate(length(min = 1))]
    pub code: String,
    
    /// Human-readable message
    #[validate(length(min = 5))]
    pub message: String,
    
    /// Severity level
    pub severity: ValidationSeverity,
    
    /// Component that caused the error
    pub component: Option<String>,
    
    /// Suggested fix
    pub suggested_fix: Option<String>,
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidationWarning {
    /// Warning code
    #[validate(length(min = 1))]
    pub code: String,
    
    /// Human-readable message
    #[validate(length(min = 5))]
    pub message: String,
    
    /// Component that caused the warning
    pub component: Option<String>,
    
    /// Suggestion to address warning
    pub suggestion: Option<String>,
}

/// Validation severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Approval status for plans requiring approval
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,       // Waiting for approval
    Approved,      // Approved by user/system
    Rejected,      // Rejected by user/system
    Expired,       // Approval request expired
    AutoApproved,  // Automatically approved
}

/// Preview of a plan before execution, with user-friendly formatting
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Preview {
    /// Unique identifier
    #[validate(length(min = 1))]
    pub id: String,
    
    /// Plan this preview is for
    #[validate(length(min = 1))]
    pub plan_id: String,
    
    /// Intent this preview relates to
    #[validate(length(min = 1))]
    pub intent_id: String,
    
    /// Summary of what will happen
    #[validate(length(min = 10, max = 500))]
    pub summary: String,
    
    /// Detailed steps in human-readable format
    #[validate]
    pub steps: Vec<PreviewStep>,
    
    /// Total estimated time
    pub estimated_time: String,
    
    /// Total estimated cost in user-friendly format
    pub estimated_cost: String,
    
    /// Risk level summary
    pub risk_summary: String,
    
    /// Important warnings for the user
    pub warnings: Vec<String>,
    
    /// Required confirmations from user
    #[validate]
    pub required_confirmations: Vec<Confirmation>,
    
    /// What the user will gain from this plan
    pub benefits: Vec<String>,
    
    /// What the user might lose or spend
    pub costs: Vec<String>,
    
    /// Dependencies that must be satisfied
    pub prerequisites: Vec<String>,
    
    /// When this preview was generated
    pub generated_at: DateTime<Utc>,
    
    /// When this preview expires
    pub expires_at: Option<DateTime<Utc>>,
    
    /// Visual representation hints for UI
    pub visualization: Option<PreviewVisualization>,
    
    /// Comparison with alternative plans
    #[validate]
    pub alternatives: Vec<PlanComparison>,
}

/// A single step in the preview
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PreviewStep {
    /// Step number in sequence
    #[validate(range(min = 1))]
    pub step_number: u32,
    
    /// Human-readable description
    #[validate(length(min = 5, max = 200))]
    pub description: String,
    
    /// What this step will accomplish
    #[validate(length(min = 5, max = 200))]
    pub outcome: String,
    
    /// Estimated time for this step
    pub estimated_time: Option<String>,
    
    /// Estimated cost for this step
    pub estimated_cost: Option<String>,
    
    /// Risk level for this step
    pub risk_level: Option<RiskLevel>,
    
    /// Whether this step is reversible
    pub reversible: bool,
    
    /// Dependencies for this step
    pub dependencies: Vec<String>,
    
    /// Visual hints for UI
    pub visual_hints: HashMap<String, String>,
    
    /// Additional details (expandable in UI)
    pub details: HashMap<String, String>,
}

/// Required user confirmation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Confirmation {
    /// Type of confirmation required
    pub confirmation_type: ConfirmationType,
    
    /// What the user is confirming
    #[validate(length(min = 5, max = 200))]
    pub description: String,
    
    /// Prompt to show the user
    #[validate(length(min = 5, max = 300))]
    pub prompt: String,
    
    /// Default response (if any)
    pub default_response: Option<bool>,
    
    /// Whether this confirmation is required
    pub required: bool,
    
    /// Additional context for the confirmation
    pub context: HashMap<String, String>,
}

/// Types of confirmations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationType {
    SpendAmount,          // Confirm spending specific amount
    RiskAcceptance,       // Accept identified risks
    DataSharing,          // Confirm data sharing
    PermissionGrant,      // Grant specific permissions
    IrreversibleAction,   // Confirm irreversible action
    ExternalInteraction,  // Confirm external service interaction
    Custom(String),       // Custom confirmation
}

/// Visual representation hints for the preview
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PreviewVisualization {
    /// Suggested layout type
    pub layout_type: Option<String>,
    
    /// Color scheme suggestions
    pub colors: HashMap<String, String>,
    
    /// Icon suggestions for each step
    pub icons: HashMap<String, String>,
    
    /// Progress indicators
    #[validate]
    pub progress_indicators: Vec<ProgressIndicator>,
    
    /// Charts or graphs to display
    #[validate]
    pub charts: Vec<ChartSpec>,
    
    /// Timeline visualization
    pub timeline: Option<TimelineSpec>,
}

/// Progress indicator specification
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ProgressIndicator {
    /// Type of progress indicator
    #[validate(length(min = 1))]
    pub indicator_type: String,
    
    /// Total steps or units
    #[validate(range(min = 1))]
    pub total: u32,
    
    /// Current position
    #[validate(range(min = 0))]
    pub current: u32,
    
    /// Label for the indicator
    #[validate(length(min = 1))]
    pub label: String,
}

/// Chart specification for visualization
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ChartSpec {
    /// Type of chart (bar, pie, line, etc.)
    #[validate(length(min = 1))]
    pub chart_type: String,
    
    /// Chart title
    #[validate(length(min = 1))]
    pub title: String,
    
    /// Data to display
    pub data: HashMap<String, f64>,
    
    /// Chart configuration
    pub config: HashMap<String, String>,
}

/// Timeline specification
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TimelineSpec {
    /// Timeline events
    #[validate]
    pub events: Vec<TimelineEvent>,
    
    /// Timeline scale (minutes, hours, days)
    #[validate(length(min = 1))]
    pub scale: String,
    
    /// Whether to show dependencies
    pub show_dependencies: bool,
}

/// Timeline event
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TimelineEvent {
    /// Event name
    #[validate(length(min = 1))]
    pub name: String,
    
    /// Start time offset
    pub start_offset: Duration,
    
    /// Duration of event
    pub duration: Duration,
    
    /// Event type
    #[validate(length(min = 1))]
    pub event_type: String,
    
    /// Dependencies
    pub dependencies: Vec<String>,
}

/// Comparison with alternative plans
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PlanComparison {
    /// Alternative plan ID
    #[validate(length(min = 1))]
    pub plan_id: String,
    
    /// Brief description of alternative
    #[validate(length(min = 5, max = 200))]
    pub description: String,
    
    /// Cost comparison
    pub cost_difference: Option<String>,
    
    /// Time comparison
    pub time_difference: Option<String>,
    
    /// Risk comparison
    pub risk_difference: Option<String>,
    
    /// Pros of this alternative
    pub pros: Vec<String>,
    
    /// Cons of this alternative
    pub cons: Vec<String>,
    
    /// Recommendation score (0.0 - 1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    pub recommendation_score: f64,
}

// Implementation of constructors and utility methods

impl Intent {
    /// Create a new intent with required fields
    pub fn new(
        description: String,
        intent_type: IntentType,
        user_id: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            description,
            intent_type,
            priority: 50, // Default medium priority
            created_at: Utc::now(),
            execute_at: None,
            expires_at: None,
            user_id,
            session_id: None,
            parameters: HashMap::new(),
            constraints: Vec::new(),
            status: IntentStatus::Pending,
            metadata: HashMap::new(),
        }
    }

    /// Validate this intent
    pub fn validate(&self) -> Result<()> {
        // Use validator crate for basic validation
        self.validate().map_err(|e| PlannerError::Validation(format!("{:?}", e)))?;

        // Custom validation logic
        if let Some(execute_at) = self.execute_at {
            if execute_at <= self.created_at {
                return Err(PlannerError::Validation(
                    "execute_at must be after created_at".to_string()
                ));
            }
        }

        if let Some(expires_at) = self.expires_at {
            if expires_at <= self.created_at {
                return Err(PlannerError::Validation(
                    "expires_at must be after created_at".to_string()
                ));
            }
        }

        if let (Some(execute_at), Some(expires_at)) = (self.execute_at, self.expires_at) {
            if execute_at >= expires_at {
                return Err(PlannerError::Validation(
                    "execute_at must be before expires_at".to_string()
                ));
            }
        }

        // Validate constraints
        for constraint in &self.constraints {
            constraint.validate()?;
        }

        Ok(())
    }

    /// Check if this intent has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() >= expires_at
        } else {
            false
        }
    }

    /// Check if this intent is ready for execution
    pub fn is_ready_for_execution(&self) -> bool {
        if self.is_expired() {
            return false;
        }

        if let Some(execute_at) = self.execute_at {
            Utc::now() >= execute_at
        } else {
            true
        }
    }
}

impl Action {
    /// Create a new action with required fields
    pub fn new(
        description: String,
        action_type: ActionType,
        executor: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            description,
            action_type,
            executor,
            parameters: HashMap::new(),
            estimated_duration: None,
            cost_estimate: None,
            risk_assessment: None,
            depends_on: Vec::new(),
            status: ActionStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
            retry_config: None,
            reversible: false,
            undo_action: None,
        }
    }

    /// Validate this action
    pub fn validate(&self) -> Result<()> {
        // Use validator crate for basic validation
        self.validate().map_err(|e| PlannerError::Validation(format!("{:?}", e)))?;

        // Check for self-dependency
        if self.depends_on.contains(&self.id) {
            return Err(PlannerError::DependencyCycle(
                format!("Action {} depends on itself", self.id)
            ));
        }

        // Validate timestamps
        if let (Some(started_at), Some(completed_at)) = (self.started_at, self.completed_at) {
            if completed_at <= started_at {
                return Err(PlannerError::Validation(
                    "completed_at must be after started_at".to_string()
                ));
            }
        }

        // Validate cost estimate if present
        if let Some(ref cost_estimate) = self.cost_estimate {
            cost_estimate.validate()?;
        }

        // Validate risk assessment if present
        if let Some(ref risk_assessment) = self.risk_assessment {
            risk_assessment.validate()?;
        }

        Ok(())
    }

    /// Check if all dependencies are satisfied (completed)
    pub fn dependencies_satisfied(&self, actions: &[Action]) -> bool {
        for dep_id in &self.depends_on {
            if let Some(dep_action) = actions.iter().find(|a| a.id == *dep_id) {
                if dep_action.status != ActionStatus::Completed {
                    return false;
                }
            } else {
                // Dependency not found
                return false;
            }
        }
        true
    }

    /// Check if this action is ready to execute
    pub fn is_ready_to_execute(&self, actions: &[Action]) -> bool {
        self.status == ActionStatus::Pending && self.dependencies_satisfied(actions)
    }
}

impl Plan {
    /// Create a new plan with required fields
    pub fn new(
        intent_id: String,
        description: String,
        created_by: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            intent_id,
            version: 1,
            description,
            actions: Vec::new(),
            strategy: ExecutionStrategy::Sequential,
            estimated_duration: None,
            total_cost_estimate: None,
            risk_assessment: None,
            status: PlanStatus::Draft,
            created_at: now,
            updated_at: now,
            created_by,
            simulation_result: None,
            rollback_plan: None,
            validation_result: None,
            requires_approval: false,
            approval_status: None,
        }
    }

    /// Validate this plan
    pub fn validate(&self) -> Result<()> {
        // Use validator crate for basic validation
        self.validate().map_err(|e| PlannerError::Validation(format!("{:?}", e)))?;

        // Validate actions
        for action in &self.actions {
            action.validate()?;
        }

        // Check for dependency cycles
        self.check_dependency_cycles()?;

        // Validate that all action dependencies exist
        for action in &self.actions {
            for dep_id in &action.depends_on {
                if !self.actions.iter().any(|a| a.id == *dep_id) {
                    return Err(PlannerError::Validation(
                        format!("Action {} depends on non-existent action {}", action.id, dep_id)
                    ));
                }
            }
        }

        Ok(())
    }

    /// Check for dependency cycles in the action graph
    pub fn check_dependency_cycles(&self) -> Result<()> {
        // Create adjacency list
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        for action in &self.actions {
            graph.insert(action.id.clone(), action.depends_on.clone());
        }

        // Use DFS to detect cycles
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();

        for action in &self.actions {
            if !visited.contains(&action.id) {
                if self.has_cycle_util(&action.id, &graph, &mut visited, &mut rec_stack)? {
                    return Err(PlannerError::DependencyCycle(
                        "Cycle detected in action dependencies".to_string()
                    ));
                }
            }
        }

        Ok(())
    }

    /// Utility function for cycle detection using DFS
    fn has_cycle_util(
        &self,
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
    ) -> Result<bool> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if self.has_cycle_util(neighbor, graph, visited, rec_stack)? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(neighbor) {
                    return Ok(true);
                }
            }
        }

        rec_stack.remove(node);
        Ok(false)
    }

    /// Get actions that are ready to execute
    pub fn get_ready_actions(&self) -> Vec<&Action> {
        self.actions.iter()
            .filter(|action| action.is_ready_to_execute(&self.actions))
            .collect()
    }

    /// Get the topological order of actions for execution
    pub fn get_execution_order(&self) -> Result<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize in-degree and graph
        for action in &self.actions {
            in_degree.insert(action.id.clone(), 0);
            graph.insert(action.id.clone(), Vec::new());
        }

        // Build graph and calculate in-degrees
        for action in &self.actions {
            for dep_id in &action.depends_on {
                if let Some(neighbors) = graph.get_mut(dep_id) {
                    neighbors.push(action.id.clone());
                }
                *in_degree.get_mut(&action.id).unwrap() += 1;
            }
        }

        // Topological sort using Kahn's algorithm
        let mut queue: std::collections::VecDeque<String> = std::collections::VecDeque::new();
        let mut result = Vec::new();

        // Find all nodes with in-degree 0
        for (node, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node.clone());
            }
        }

        while let Some(node) = queue.pop_front() {
            result.push(node.clone());

            if let Some(neighbors) = graph.get(&node) {
                for neighbor in neighbors {
                    let degree = in_degree.get_mut(neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        if result.len() != self.actions.len() {
            return Err(PlannerError::DependencyCycle(
                "Topological sort failed - cycle detected".to_string()
            ));
        }

        Ok(result)
    }
}

impl Preview {
    /// Create a new preview from a plan
    pub fn from_plan(plan: &Plan, intent: &Intent) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            plan_id: plan.id.clone(),
            intent_id: intent.id.clone(),
            summary: format!("Execute {} actions to {}", plan.actions.len(), intent.description),
            steps: plan.actions.iter().enumerate().map(|(i, action)| {
                PreviewStep {
                    step_number: (i + 1) as u32,
                    description: action.description.clone(),
                    outcome: format!("Complete {}", action.action_type.to_string()),
                    estimated_time: action.estimated_duration
                        .map(|d| format!("{}s", d.num_seconds()))
                        .unwrap_or_else(|| "Unknown".to_string()),
                    estimated_cost: action.cost_estimate
                        .as_ref()
                        .map(|c| format!("{} {}", c.total_cost, c.currency))
                        .unwrap_or_else(|| "Free".to_string()),
                    risk_level: action.risk_assessment
                        .as_ref()
                        .map(|r| r.risk_level.clone()),
                    reversible: action.reversible,
                    dependencies: action.depends_on.clone(),
                    visual_hints: HashMap::new(),
                    details: HashMap::new(),
                }
            }).collect(),
            estimated_time: plan.estimated_duration
                .map(|d| format!("{}s", d.num_seconds()))
                .unwrap_or_else(|| "Unknown".to_string()),
            estimated_cost: plan.total_cost_estimate
                .as_ref()
                .map(|c| format!("{} {}", c.total_cost, c.currency))
                .unwrap_or_else(|| "Free".to_string()),
            risk_summary: plan.risk_assessment
                .as_ref()
                .map(|r| format!("{:?} risk", r.risk_level))
                .unwrap_or_else(|| "Unknown risk".to_string()),
            warnings: Vec::new(),
            required_confirmations: Vec::new(),
            benefits: vec![format!("Achieve: {}", intent.description)],
            costs: Vec::new(),
            prerequisites: Vec::new(),
            generated_at: Utc::now(),
            expires_at: Some(Utc::now() + Duration::hours(1)), // Expire in 1 hour
            visualization: None,
            alternatives: Vec::new(),
        }
    }

    /// Validate this preview
    pub fn validate(&self) -> Result<()> {
        // Use validator crate for basic validation
        self.validate().map_err(|e| PlannerError::Validation(format!("{:?}", e)))?;

        // Validate steps
        for step in &self.steps {
            step.validate()?;
        }

        // Validate confirmations
        for confirmation in &self.required_confirmations {
            confirmation.validate()?;
        }

        Ok(())
    }
}

// Implement Display for common enums
impl std::fmt::Display for IntentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntentType::Transfer => write!(f, "transfer"),
            IntentType::Payment => write!(f, "payment"),
            IntentType::Trade => write!(f, "trade"),
            IntentType::Stake => write!(f, "stake"),
            IntentType::Unstake => write!(f, "unstake"),
            IntentType::Authenticate => write!(f, "authenticate"),
            IntentType::Authorize => write!(f, "authorize"),
            IntentType::RevokeAccess => write!(f, "revoke_access"),
            IntentType::CreateIdentity => write!(f, "create_identity"),
            IntentType::ShareData => write!(f, "share_data"),
            IntentType::RevokeData => write!(f, "revoke_data"),
            IntentType::BackupData => write!(f, "backup_data"),
            IntentType::DeleteData => write!(f, "delete_data"),
            IntentType::Configure => write!(f, "configure"),
            IntentType::Deploy => write!(f, "deploy"),
            IntentType::Monitor => write!(f, "monitor"),
            IntentType::Maintenance => write!(f, "maintenance"),
            IntentType::Message => write!(f, "message"),
            IntentType::Notification => write!(f, "notification"),
            IntentType::Broadcast => write!(f, "broadcast"),
            IntentType::Custom(s) => write!(f, "custom({})", s),
        }
    }
}

impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionType::SendTransaction => write!(f, "send_transaction"),
            ActionType::SignMessage => write!(f, "sign_message"),
            ActionType::ApproveToken => write!(f, "approve_token"),
            ActionType::RevokeApproval => write!(f, "revoke_approval"),
            ActionType::DeployContract => write!(f, "deploy_contract"),
            ActionType::CallContract => write!(f, "call_contract"),
            ActionType::UpgradeContract => write!(f, "upgrade_contract"),
            ActionType::CreateIdentity => write!(f, "create_identity"),
            ActionType::VerifyIdentity => write!(f, "verify_identity"),
            ActionType::RotateKeys => write!(f, "rotate_keys"),
            ActionType::BackupKeys => write!(f, "backup_keys"),
            ActionType::StoreData => write!(f, "store_data"),
            ActionType::RetrieveData => write!(f, "retrieve_data"),
            ActionType::DeleteData => write!(f, "delete_data"),
            ActionType::EncryptData => write!(f, "encrypt_data"),
            ActionType::DecryptData => write!(f, "decrypt_data"),
            ActionType::SendMessage => write!(f, "send_message"),
            ActionType::EstablishConnection => write!(f, "establish_connection"),
            ActionType::CloseConnection => write!(f, "close_connection"),
            ActionType::ConfigureSystem => write!(f, "configure_system"),
            ActionType::StartService => write!(f, "start_service"),
            ActionType::StopService => write!(f, "stop_service"),
            ActionType::RestartService => write!(f, "restart_service"),
            ActionType::ValidateSignature => write!(f, "validate_signature"),
            ActionType::VerifyProof => write!(f, "verify_proof"),
            ActionType::CheckBalance => write!(f, "check_balance"),
            ActionType::Custom(s) => write!(f, "custom({})", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_intent() -> Intent {
        Intent::new(
            "Transfer 100 tokens to Alice".to_string(),
            IntentType::Transfer,
            "user123".to_string(),
        )
    }

    fn create_test_action() -> Action {
        Action::new(
            "Send transaction".to_string(),
            ActionType::SendTransaction,
            "wallet_service".to_string(),
        )
    }

    fn create_test_plan() -> Plan {
        Plan::new(
            "intent123".to_string(),
            "Transfer plan".to_string(),
            "planner_service".to_string(),
        )
    }

    #[test]
    fn test_intent_creation_and_validation() {
        let intent = create_test_intent();
        assert_eq!(intent.intent_type, IntentType::Transfer);
        assert_eq!(intent.status, IntentStatus::Pending);
        assert_eq!(intent.priority, 50);
        
        // Validation should pass for a properly created intent
        assert!(intent.validate().is_ok());
    }

    #[test]
    fn test_intent_expiration() {
        let mut intent = create_test_intent();
        
        // Not expired initially
        assert!(!intent.is_expired());
        
        // Set expiration in the past
        intent.expires_at = Some(Utc::now() - Duration::hours(1));
        assert!(intent.is_expired());
    }

    #[test]
    fn test_action_creation_and_validation() {
        let action = create_test_action();
        assert_eq!(action.action_type, ActionType::SendTransaction);
        assert_eq!(action.status, ActionStatus::Pending);
        assert!(!action.reversible);
        
        // Validation should pass
        assert!(action.validate().is_ok());
    }

    #[test]
    fn test_action_dependencies() {
        let mut action1 = create_test_action();
        action1.id = "action1".to_string();
        
        let mut action2 = create_test_action();
        action2.id = "action2".to_string();
        action2.depends_on = vec!["action1".to_string()];
        
        let mut action3 = create_test_action();
        action3.id = "action3".to_string();
        action3.status = ActionStatus::Completed;
        
        let actions = vec![action1, action2.clone(), action3];
        
        // action2 depends on action1 which is not completed
        assert!(!action2.dependencies_satisfied(&actions));
        assert!(!action2.is_ready_to_execute(&actions));
    }

    #[test]
    fn test_plan_creation_and_validation() {
        let plan = create_test_plan();
        assert_eq!(plan.version, 1);
        assert_eq!(plan.status, PlanStatus::Draft);
        assert_eq!(plan.strategy, ExecutionStrategy::Sequential);
        
        // Empty plan should validate
        assert!(plan.validate().is_ok());
    }

    #[test]
    fn test_plan_with_actions() {
        let mut plan = create_test_plan();
        
        let mut action1 = create_test_action();
        action1.id = "action1".to_string();
        
        let mut action2 = create_test_action();
        action2.id = "action2".to_string();
        action2.depends_on = vec!["action1".to_string()];
        
        plan.actions = vec![action1, action2];
        
        // Should validate successfully
        assert!(plan.validate().is_ok());
        
        // Should have proper execution order
        let order = plan.get_execution_order().unwrap();
        assert_eq!(order, vec!["action1".to_string(), "action2".to_string()]);
    }

    #[test]
    fn test_plan_dependency_cycle_detection() {
        let mut plan = create_test_plan();
        
        let mut action1 = create_test_action();
        action1.id = "action1".to_string();
        action1.depends_on = vec!["action2".to_string()];
        
        let mut action2 = create_test_action();
        action2.id = "action2".to_string();
        action2.depends_on = vec!["action1".to_string()];
        
        plan.actions = vec![action1, action2];
        
        // Should detect cycle
        assert!(plan.validate().is_err());
        assert!(plan.check_dependency_cycles().is_err());
    }

    #[test]
    fn test_preview_creation() {
        let intent = create_test_intent();
        let plan = create_test_plan();
        let preview = Preview::from_plan(&plan, &intent);
        
        assert_eq!(preview.plan_id, plan.id);
        assert_eq!(preview.intent_id, intent.id);
        assert!(preview.summary.contains("Transfer 100 tokens"));
        
        // Should validate
        assert!(preview.validate().is_ok());
    }

    #[test]
    fn test_serialization_round_trip() {
        let intent = create_test_intent();
        
        // Serialize to JSON
        let json = serde_json::to_string(&intent).unwrap();
        
        // Deserialize back
        let deserialized: Intent = serde_json::from_str(&json).unwrap();
        
        // Should be equal
        assert_eq!(intent.id, deserialized.id);
        assert_eq!(intent.description, deserialized.description);
        assert_eq!(intent.intent_type, deserialized.intent_type);
        assert_eq!(intent.user_id, deserialized.user_id);
    }

    #[test]
    fn test_risk_assessment_validation() {
        let mut risk_factor = RiskFactor {
            risk_type: "financial".to_string(),
            description: "High transaction amount".to_string(),
            impact: RiskLevel::High,
            probability: 0.3,
            mitigation: Some("Use smaller amounts".to_string()),
        };
        
        // Should validate successfully
        assert!(risk_factor.validate().is_ok());
        
        // Invalid probability should fail
        risk_factor.probability = 1.5;
        assert!(risk_factor.validate().is_err());
    }

    #[test]
    fn test_cost_estimate_validation() {
        let mut cost_estimate = CostEstimate {
            gas_estimate: 21000,
            gas_price: 20000000000,
            total_cost: "0.0004".to_string(),
            currency: "ETH".to_string(),
            network_fee: Some("0.0002".to_string()),
            service_fee: Some("0.0002".to_string()),
            confidence: 0.95,
            cost_breakdown: HashMap::new(),
        };
        
        // Should validate successfully
        assert!(cost_estimate.validate().is_ok());
        
        // Invalid confidence should fail
        cost_estimate.confidence = 1.5;
        assert!(cost_estimate.validate().is_err());
    }

    #[test]
    fn test_invalid_field_lengths() {
        // Test short description
        let mut intent = create_test_intent();
        intent.description = "short".to_string();
        assert!(intent.validate().is_err());
        
        // Test empty user_id
        intent.description = "This is a valid description that is long enough".to_string();
        intent.user_id = "".to_string();
        assert!(intent.validate().is_err());
    }

    #[test]
    fn test_intent_status_transitions() {
        let mut intent = create_test_intent();
        
        // Valid transition
        intent.status = IntentStatus::Planning;
        assert!(intent.validate().is_ok());
        
        intent.status = IntentStatus::Planned;
        assert!(intent.validate().is_ok());
    }

    #[test]
    fn test_custom_types() {
        let custom_intent_type = IntentType::Custom("migrate_data".to_string());
        assert_eq!(custom_intent_type.to_string(), "custom(migrate_data)");
        
        let custom_action_type = ActionType::Custom("backup_db".to_string());
        assert_eq!(custom_action_type.to_string(), "custom(backup_db)");
    }
}
