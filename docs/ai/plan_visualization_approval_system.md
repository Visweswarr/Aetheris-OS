# AI Plan Visualization and Approval System

## Overview

The AI Plan Visualization and Approval System provides a comprehensive solution for reviewing and approving AI-generated plans before execution. This system ensures human oversight and control over automated AI decision-making processes.

## Architecture

### Core Components

1. **Plan UI Module** (`services/ai/src/plan_ui.rs`)
   - Converts AI planner output to user-friendly format
   - Manages approval workflow and state
   - Provides deterministic logging of all approvals
   - Implements idempotent approval operations

2. **CLI Tool** (`tooling/ai-plan-cli/`)
   - Command-line interface for plan visualization
   - Interactive approval workflow
   - Status checking and plan listing
   - Integrated with AI service via library imports

3. **HTTP API** (`tooling/ai-plan-http/`)
   - REST endpoints for plan management
   - JSON-based API for web integration
   - Example implementation for service integration
   - Demonstrates HTTP-based approval workflow

4. **API Integration** (`services/ai/src/api.rs`)
   - Updated AI service API with approval endpoints
   - Integrates plan_ui module with existing planner
   - Provides `/ai/plan/approve` endpoint for REST clients

## Features

### Plan Visualization
- **CLI Rendering**: Text-based plan visualization with Unicode icons
- **Step-by-step breakdown**: Clear enumeration of plan steps
- **Status indicators**: Visual status representation (✅ ⏳ ❌ 🔄)
- **Metadata display**: Plan name, description, goal, and creator information
- **Approval status**: Clear indication of approval state and approver

### Approval Workflow
- **User-controlled approval**: Explicit user approval required before execution
- **Idempotent operations**: Multiple approval attempts are safe and consistent
- **Approval logging**: All approvals logged to `logs/ai_plans.log`
- **Deterministic timestamps**: RFC3339 timestamps for all approval events
- **User attribution**: Track which user approved each plan

### Integration Points
- **Library integration**: Direct integration with AI service library
- **CLI commands**: `list`, `view`, `approve`, `status` commands
- **HTTP endpoints**: RESTful API for web applications
- **API updates**: Enhanced AI service API with approval support

## Usage Examples

### CLI Usage

```bash
# List all plans
ai-plan list

# View plan details
ai-plan view plan-123

# Approve a plan
ai-plan approve plan-123 --user-id alice

# Check plan status
ai-plan status plan-123
```

### HTTP API Usage

```bash
# List plans
curl http://localhost:8080/api/plans

# Get specific plan
curl http://localhost:8080/api/plans/plan-123

# Approve plan
curl -X POST http://localhost:8080/api/plans/approve \
  -H "Content-Type: application/json" \
  -d '{"plan_id": "plan-123", "user_id": "alice"}'

# Get rendered plan view
curl http://localhost:8080/api/plans/plan-123/render
```

### Library Integration

```rust
use aetheris_ai::plan_ui::{store_plan_for_ui, approve_plan, render_plan_cli};

// Store plan for UI
store_plan_for_ui(&plan);

// Approve plan
let approved_plan = approve_plan("plan-123", "alice").unwrap();

// Render plan
let cli_output = render_plan_cli(&approved_plan);
println!("{}", cli_output);
```

## Implementation Details

### Data Structures

```rust
pub struct UiPlan {
    pub id: String,
    pub steps: Vec<UiStep>,
    pub approved: bool,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
    pub metadata: HashMap<String, String>,
}

pub struct UiStep {
    pub number: usize,
    pub name: String,
    pub description: String,
    pub status: String,
}
```

### Key Functions

- `store_plan_for_ui(plan: &Plan)`: Convert and store plan for UI access
- `approve_plan(plan_id: &str, user_id: &str) -> Option<UiPlan>`: Approve plan with user attribution
- `get_plan_for_ui(plan_id: &str) -> Option<UiPlan>`: Retrieve plan for visualization
- `list_plans_for_ui() -> Vec<UiPlan>`: List all available plans
- `render_plan_cli(plan: &UiPlan) -> String`: Generate CLI visualization

### Logging Format

All approvals are logged in JSON format to `logs/ai_plans.log`:

```json
{
  "plan_id": "plan-123",
  "approved": true,
  "approved_by": "alice",
  "approved_at": "2024-01-15T10:30:00Z",
  "steps": [...],
  "metadata": {...}
}
```

## Security Considerations

1. **User Authentication**: The system accepts user IDs but does not validate them
2. **Authorization**: No role-based access control implemented
3. **Audit Trail**: Complete audit trail maintained in log files
4. **Idempotency**: Prevents duplicate approvals and ensures consistency
5. **Input Validation**: Basic validation on plan IDs and user IDs

## Testing

### Unit Tests (`tests/ai_plan_ui.rs`)
- Plan conversion and storage
- Approval workflow and idempotency
- CLI rendering and metadata handling
- List operations and data consistency

### Integration Tests
- CLI tool functionality
- HTTP API endpoints
- End-to-end approval workflow
- Logging verification

## Future Enhancements

### Planned Features
1. **User Authentication**: Integration with Aetheris OS user management
2. **Role-based Access**: Different approval levels for different users
3. **Plan Templates**: Reusable plan templates and batch operations
4. **Execution Tracking**: Real-time plan execution status updates
5. **Web UI**: Browser-based plan visualization and approval interface

### Technical Improvements
1. **Persistent Storage**: Replace in-memory storage with database
2. **Event Streaming**: Real-time plan status updates via WebSocket
3. **API Versioning**: Versioned API endpoints for backward compatibility
4. **Metrics and Monitoring**: Detailed metrics on approval patterns
5. **Integration Testing**: Comprehensive integration test suite

## Configuration

### Environment Variables
- `RUST_LOG`: Log level configuration
- `PORT`: HTTP server port (default: 8080)
- `AI_PLANS_LOG_DIR`: Log directory for approval logs (default: ./logs)

### Files
- `logs/ai_plans.log`: Approval audit log
- `~/.aetheris/plans/`: Plan storage directory (future)
- `/etc/aetheris/ai_core/`: AI service configuration

## Dependencies

### Core Dependencies
- `tokio`: Async runtime
- `serde`: Serialization framework
- `chrono`: Date/time handling
- `lazy_static`: Global state management
- `clap`: CLI argument parsing
- `tracing`: Structured logging

### AI Service Integration
- `aetheris-ai`: AI service library with Phase 5 features
- Plan integration via `planner` module
- API integration via `api` module

## Deployment

### CLI Tool
```bash
cd tooling/ai-plan-cli
cargo build --release
cp target/release/ai-plan /usr/local/bin/
```

### HTTP Server
```bash
cd tooling/ai-plan-http
cargo build --release
./target/release/ai-plan-http &
```

### Service Integration
The plan_ui module is automatically included when the AI service is built with `phase5` feature enabled.

## Monitoring and Observability

### Logging
- Structured logging via `tracing` crate
- Approval audit trail in JSON format
- Error logging for failed operations
- Debug logging available with `--verbose` flag

### Metrics (Future)
- Approval rate and timing metrics
- Plan complexity metrics
- User activity tracking
- Performance monitoring

This system provides a complete solution for AI plan visualization and approval, ensuring human oversight while maintaining the benefits of automated planning.