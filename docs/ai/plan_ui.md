# Plan Visualization and Approval UI

This module provides a text-based UI and API for visualizing AI-generated plans and handling user approval before execution. It integrates with the Plan Store for persistent, crash-safe storage with deterministic Plan IDs.

## Features

- Parses planner JSON into structured Plan objects
- Renders plans as numbered lists in CLI
- REST API with pagination, filtering, and revision tracking
- Approval endpoint with revision increments and metadata
- Idempotent approval flow (multiple approvals handled gracefully)
- Persistent storage with deterministic Plan IDs using Blake3 hashing
- NDJSON event logging for durability and crash recovery
- Thread-safe concurrent access with DashMap indexing

## Usage

### Enhanced CLI Tool

The `ai-plan` CLI tool now supports comprehensive plan management:

```sh
# List plans with pagination and filtering
ai-plan list --status pending --limit 10 --offset 0
ai-plan list --creator alice --limit 5

# View plan details with revision support
ai-plan view <plan_id>
ai-plan view <plan_id> --revision 2

# Create plans from JSON files
ai-plan create --file plan.json --user-id creator

# Approve plans with optional messages
ai-plan approve <plan_id> --user-id approver --message "Looks good"

# Check plan status and revision history
ai-plan status <plan_id> --history
```

### Enhanced REST API

The API now provides full CRUD operations with persistence:

- `POST /ai/plan` → creates plan, returns deterministic Plan ID and metadata
- `GET /ai/plan/:id` → retrieves plan with full metadata and revision info
- `GET /ai/plans?status=pending&creator=alice&limit=10&offset=0` → lists plans with filtering and pagination
- `POST /ai/plan/approve` → approves plan, increments revision, returns updated record

### Response Formats

**Plan Summary** (for listing):
```json
{
  "id": "abc123...",
  "name": "Deploy Service",
  "user_goal": "Deploy the service to production",
  "priority": "High",
  "status": "pending",
  "created_at": "2024-01-15T10:30:00Z",
  "step_count": 5,
  "progress": 0.6,
  "creator": "alice",
  "revision": 1
}
```

**Plan Record** (for details):
```json
{
  "metadata": {
    "plan_id": "abc123...",
    "user_id": "alice",
    "revision": 2,
    "status": "approved",
    "created_at": "2024-01-15T10:30:00Z",
    "approved_by": "bob",
    "approved_at": "2024-01-15T10:35:00Z"
  },
  "plan": { /* full plan data */ }
}
```

## Architecture Integration

### Plan Store Integration

The UI module integrates with the Plan Store through:
- **`init_plan_store()`**: Initializes shared store instance
- **Backward compatibility**: Existing UI functions work seamlessly
- **Enhanced metadata**: Revision tracking, timestamps, approval history

### Persistent Storage

- **Deterministic IDs**: Blake3 hash of canonical plan JSON
- **NDJSON logging**: Append-only event log for durability
- **Crash recovery**: Automatic state reconstruction on startup
- **Thread safety**: Concurrent access through DashMap

## Implementation Notes

- Follows reference_os/ui style for rendering
- Plan storage is persistent and crash-safe
- All operations are idempotent and thread-safe
- Serialization and logging are deterministic
- Revision tracking maintains approval history

## Testing

Comprehensive test coverage includes:

- `tests/ai_plan_ui.rs`: UI integration, pagination, concurrency, idempotency
- `tests/ai_plan_store.rs`: Plan store persistence, recovery, deterministic IDs
- CLI integration tests in `tooling/ai-plan-cli/src/main.rs`

## Migration from Previous Version

Existing plans stored in memory can be migrated:

1. Export plans to JSON format
2. Use `ai-plan create` to import with proper Plan IDs
3. Update approval status using `ai-plan approve`
4. Verify data integrity with `ai-plan list` and `ai-plan status`
