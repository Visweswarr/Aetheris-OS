# Task Planner Module - Phase 5 Implementation

## Overview

The Task Planner module provides intelligent task planning capabilities for the Polymera OS AI service. It enables users to submit high-level goals and receive structured, executable plans with deterministic execution tracking.

## Features

### Core Functionality

1. **Plan Generation**: Convert user goals into structured, multi-step plans
2. **Deterministic Execution**: Reproducible plan generation with seeded randomness
3. **Step Management**: Hierarchical task breakdown with dependencies
4. **Progress Tracking**: Real-time execution progress monitoring
5. **Serialization**: CBOR-based deterministic plan serialization
6. **API Integration**: HTTP-style API interface for external access

### Plan Structure

Each plan contains:
- **Plan Metadata**: ID, name, description, user goal, priority, status
- **Execution Steps**: Ordered list of tasks with dependencies
- **Progress Tracking**: Current status and completion percentage
- **Revision History**: Track plan changes and updates
- **Tool Integration**: Associate steps with external tools/capabilities

### Step Types

- **Analysis**: Requirements analysis and constraint identification
- **Planning**: Detailed implementation planning
- **Setup**: Resource and environment preparation
- **Implementation**: Core functionality development
- **Testing**: Validation and verification
- **Deployment**: Solution deployment
- **Verification**: Final requirement verification

## Usage Examples

### Basic Plan Generation

```rust
use aetheris_ai::planner::{TaskPlanner, StubTaskPlanner, PlanRequest, StepPriority};
use aetheris_ai::policy::AiPolicy;
use std::sync::Arc;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize planner
    let policy = Arc::new(AiPolicy::default());
    let planner = Arc::new(StubTaskPlanner::new(policy)) as Arc<dyn TaskPlanner>;
    
    // Configure planner
    let config = PlannerConfig::default();
    planner.init(config).await?;
    
    // Create plan request
    let request = PlanRequest {
        goal: "Create a web application with user authentication".to_string(),
        context: Some("Build a secure app with login/logout functionality".to_string()),
        required_tools: vec!["npm".to_string(), "node".to_string(), "database".to_string()],
        priority: StepPriority::Normal,
        max_execution_time: Some(3600), // 1 hour
        tags: vec!["web".to_string(), "auth".to_string()],
        metadata: HashMap::new(),
    };
    
    // Generate plan
    let result = planner.generate_plan(request).await?;
    
    if result.success {
        let plan = result.plan.unwrap();
        println!("Generated plan: {} with {} steps", plan.name, plan.steps.len());
        
        // Execute first step
        if !plan.steps.is_empty() {
            let step_id = &plan.steps[0].id;
            let executed_step = planner.execute_step(&plan.id, step_id).await?;
            println!("Executed step: {}", executed_step.name);
        }
    }
    
    Ok(())
}
```

### HTTP API Usage

```rust
use aetheris_ai::api::{PlannerApi, CreatePlanRequest};
use aetheris_ai::policy::AiPolicy;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize API
    let policy = Arc::new(AiPolicy::default());
    let api = PlannerApi::new(policy);
    api.init().await?;
    
    // Create plan via API
    let request = CreatePlanRequest {
        goal: "Deploy microservice to Kubernetes".to_string(),
        context: Some("Deploy with scaling and monitoring".to_string()),
        required_tools: Some(vec!["kubectl".to_string(), "docker".to_string()]),
        priority: Some("high".to_string()),
        max_execution_time: Some(1800),
        tags: Some(vec!["k8s".to_string(), "deployment".to_string()]),
    };
    
    let response = api.create_plan(request).await;
    
    if response.success {
        let plan_id = response.plan_id.unwrap();
        println!("Created plan: {}", plan_id);
        
        // Get plan details
        let get_response = api.get_plan(&plan_id).await;
        if let Some(plan) = get_response.plan {
            println!("Plan '{}' has {} steps", plan.name, plan.steps.len());
        }
    }
    
    Ok(())
}
```

## API Endpoints

The planner exposes the following conceptual endpoints (actual implementation depends on integration):

### POST /ai/plan
Create a new task plan

**Request:**
```json
{
  "goal": "User's objective description",
  "context": "Additional context or constraints",
  "required_tools": ["tool1", "tool2"],
  "priority": "normal|high|low|critical",
  "max_execution_time": 3600,
  "tags": ["tag1", "tag2"]
}
```

**Response:**
```json
{
  "success": true,
  "plan_id": "plan_uuid",
  "plan": {
    "id": "plan_uuid",
    "name": "Generated plan name",
    "user_goal": "User's objective",
    "status": "ready",
    "steps": [...],
    "progress": 0.0
  },
  "generation_time_ms": 250,
  "llm_stats": {
    "input_tokens": 35,
    "output_tokens": 100,
    "model": "llama3-8b"
  }
}
```

### GET /ai/plan/{plan_id}
Retrieve a specific plan

**Response:**
```json
{
  "success": true,
  "plan": {
    "id": "plan_uuid",
    "name": "Plan name",
    "status": "executing",
    "progress": 0.4,
    "steps": [...]
  }
}
```

### GET /ai/plans
List all plans

**Response:**
```json
{
  "success": true,
  "plans": [
    {
      "id": "plan_uuid",
      "name": "Plan name",
      "user_goal": "User objective",
      "status": "completed",
      "progress": 1.0,
      "step_count": 5
    }
  ],
  "total_count": 1
}
```

## Configuration

### PlannerConfig Options

```rust
pub struct PlannerConfig {
    /// Maximum plan depth (nested steps)
    pub max_plan_depth: usize,        // Default: 5
    /// Maximum steps per plan
    pub max_steps_per_plan: usize,    // Default: 20
    /// Enable deterministic plan generation
    pub deterministic: bool,          // Default: true
    /// Random seed for deterministic planning
    pub random_seed: u64,            // Default: 42
    /// LLM model to use for planning
    pub llm_model: String,           // Default: "llama3-8b"
    /// Maximum context tokens for LLM
    pub max_context_tokens: usize,   // Default: 4096
    /// Enable plan optimization
    pub enable_optimization: bool,    // Default: true
    /// Plan timeout in seconds
    pub plan_timeout_secs: u64,      // Default: 30
}
```

## Integration with Polymera OS

### Phase 5 Feature Flag

The planner is enabled with the `phase5` feature flag:

```toml
[features]
phase5 = ["ciborium", "blake3"]
```

### Service Integration

The planner integrates with the main AI service:

```rust
// In AiService
#[cfg(feature = "phase5")]
pub fn task_planner(&self) -> Arc<dyn TaskPlanner> {
    self.task_planner.clone()
}

#[cfg(feature = "phase5")]
pub async fn generate_plan(&self, goal: &str) -> Result<PlanResult, AiError> {
    self.policy_manager.check_capability("ai:plan.generate")?;
    // ... implementation
}
```

## Testing

### Unit Tests

Comprehensive test coverage includes:

- Plan creation and basic properties
- Step dependencies and execution order
- Progress calculation and tracking
- Deterministic serialization/deserialization
- Error handling and edge cases
- API request/response handling
- End-to-end workflow testing

### Running Tests

```bash
# Run all planner tests
cargo test ai_planner --features phase5

# Run specific test
cargo test test_plan_generation_deterministic --features phase5

# Run with output
cargo test ai_planner --features phase5 -- --nocapture
```

## Error Handling

The planner provides comprehensive error handling:

- **Configuration Errors**: Invalid planner setup
- **Validation Errors**: Malformed requests or missing plans
- **Serialization Errors**: CBOR encoding/decoding issues
- **Timeout Errors**: Plan generation exceeding limits
- **Internal Errors**: Unexpected system failures

All errors implement the `AiError` type with detailed error messages.

## Performance Characteristics

### Stub Implementation Performance

- **Plan Generation**: ~250ms for typical 5-7 step plans
- **Serialization**: ~1ms for average plan (deterministic CBOR)
- **Step Execution**: ~100ms simulated execution time
- **Memory Usage**: ~1KB per plan, ~200 bytes per step

### Scalability Considerations

- Plans stored in memory (HashMap) - consider persistent storage for production
- Thread-safe implementation with Arc<RwLock<>> for concurrent access
- Configurable limits on plan depth and step count
- LRU eviction recommended for large deployments

## Future Enhancements

### Planned Features

1. **Real LLM Integration**: Replace stub with actual language model
2. **Persistent Storage**: Database backend for plan persistence
3. **Advanced Scheduling**: Cron-like scheduling for plan execution
4. **Plan Templates**: Reusable plan templates for common tasks
5. **Execution Monitoring**: Real-time step execution monitoring
6. **Plan Optimization**: Automatic plan optimization and parallelization
7. **Tool Integration**: Direct integration with system tools and APIs
8. **Collaborative Planning**: Multi-user plan creation and editing

### Integration Points

- **Intent Bus**: Subscribe to planning intents from system components
- **IPC Protocol**: Extend protobuf definitions for native IPC support
- **Event System**: Emit plan state change events
- **Capability System**: Integrate with OS capability management
- **Resource Management**: Coordinate with system resource manager

## Security Considerations

- **Capability Tokens**: All operations require appropriate capability tokens
- **Input Validation**: Comprehensive validation of all plan inputs
- **Resource Limits**: Configurable limits to prevent resource exhaustion
- **Audit Logging**: All planning operations logged for security auditing
- **Deterministic Execution**: Reproducible plans for security analysis

## License

MIT License - see LICENSE file for details.

## Contributing

See CONTRIBUTING.md for development guidelines and contribution process.