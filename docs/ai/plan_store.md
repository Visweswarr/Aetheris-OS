# Plan Store - Deterministic Plan Registry

The Plan Store provides persistent, crash-safe storage for AI-generated plans with deterministic Plan IDs, revision tracking, and efficient retrieval. It serves as the source of truth for all plan data and approval workflow state.

## Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Plan Store    │    │   DashMap Index │    │   NDJSON Log    │
│                 │◄──►│                 │◄──►│                 │
│ • Store plans   │    │ • Fast lookups  │    │ • Durability    │
│ • Generate IDs  │    │ • Thread-safe   │    │ • Crash recovery│
│ • Approve plans │    │ • In-memory     │    │ • Append-only   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Core Components

### Deterministic Plan IDs

Plan IDs are generated using `blake3(canonical_json(plan))` to ensure:
- **Deterministic**: Identical plans always produce identical IDs
- **Unique**: Different plans always produce different IDs  
- **Cryptographically secure**: 256-bit Blake3 hash prevents collisions
- **Canonical**: JSON serialization uses sorted keys for consistency

```rust
// Example Plan ID generation
let canonical_json = canonicalize_value(&serde_json::to_value(&plan)?)?;
let plan_id = blake3::hash(&canonical_json.as_bytes()).to_hex().to_string();
```

### Plan Records

Each plan is stored as a `PlanRecord` containing:

```rust
pub struct PlanRecord {
    pub metadata: PlanMetadata,
    pub plan: Plan,
}

pub struct PlanMetadata {
    pub plan_id: String,           // Blake3 hash of canonical plan JSON
    pub user_id: String,           // Creator of the plan
    pub revision: u32,             // Revision number (starts at 1)
    pub status: PlanStatus,        // Pending or Approved
    pub created_at: DateTime<Utc>, // Creation timestamp
    pub approved_by: Option<String>, // Approver user ID (if approved)
    pub approved_at: Option<DateTime<Utc>>, // Approval timestamp
}
```

### Persistence Layer

**NDJSON Log Format**: Each operation is logged as a single JSON line for atomic writes and crash recovery.

```jsonl
{"event_type":"PlanCreated","timestamp":"2024-01-15T10:30:00Z","plan_id":"abc123...","user_id":"creator","revision":1,"plan":{...}}
{"event_type":"PlanApproved","timestamp":"2024-01-15T10:35:00Z","plan_id":"abc123...","approved_by":"approver","revision":2}
```

**Event Types**:
- `PlanCreated`: New plan stored
- `PlanApproved`: Plan approved by user

## API Reference

### Store Operations

#### `store_plan(plan: &Plan, user_id: &str) -> Result<PlanRecord>`
Stores a plan with deterministic ID generation. If the same plan (same canonical JSON) is stored multiple times, the revision number increments but the plan_id remains the same.

**Returns**: `PlanRecord` with generated metadata

#### `get_plan_by_id(plan_id: &str) -> Result<Option<PlanRecord>>`
Retrieves the latest revision of a plan by its ID.

**Returns**: `Some(PlanRecord)` if found, `None` if not found

#### `approve_plan(plan_id: &str, user_id: &str) -> Result<PlanRecord>`
Approves a plan, updating its status to `Approved` and incrementing the revision.

**Idempotent**: Multiple approvals of the same plan by any user are handled gracefully.

**Returns**: Updated `PlanRecord` with approval metadata

### Query Operations

#### `list_plans(query: &ListQuery) -> Result<ListResponse>`
Lists plans with filtering and pagination support.

```rust
pub struct ListQuery {
    pub status: Option<String>,    // Filter by "pending" or "approved"
    pub creator: Option<String>,   // Filter by creator user ID
    pub limit: usize,              // Maximum results to return
    pub offset: usize,             // Pagination offset
}

pub struct ListResponse {
    pub plans: Vec<PlanRecord>,    // Matching plan records
    pub total_count: usize,        // Total matching plans (ignoring pagination)
    pub has_more: bool,            // Whether more results exist
}
```

### ID Generation

#### `generate_plan_id(plan: &Plan) -> Result<String>`
Generates a deterministic Blake3 hash ID for any plan. Used internally by `store_plan` but exposed for testing and verification.

## Canonical JSON Serialization

To ensure deterministic IDs, plans are converted to canonical JSON format:

1. **Recursive BTreeMap conversion**: All objects become sorted key-value maps
2. **Consistent ordering**: Keys are always alphabetically sorted
3. **Type preservation**: All JSON types (strings, numbers, booleans, arrays, objects) preserved
4. **Whitespace normalization**: Compact JSON with no extra whitespace

```rust
fn canonicalize_value(value: &Value) -> Result<Value> {
    match value {
        Value::Object(map) => {
            let mut canonical_map = BTreeMap::new();
            for (k, v) in map {
                canonical_map.insert(k.clone(), canonicalize_value(v)?);
            }
            Ok(Value::Object(canonical_map.into_iter().collect()))
        }
        Value::Array(arr) => {
            let canonical_array: Result<Vec<_>, _> = arr.iter()
                .map(canonicalize_value)
                .collect();
            Ok(Value::Array(canonical_array?))
        }
        _ => Ok(value.clone()),
    }
}
```

## Concurrency and Thread Safety

- **DashMap**: Thread-safe concurrent HashMap for fast in-memory lookups
- **Parking Lot RwLock**: Reader-writer lock for log file access
- **Atomic Operations**: All mutations are atomic at the operation level
- **Lock-free Reads**: Plan lookups don't require exclusive locks

## Crash Recovery

On startup, the Plan Store replays the entire NDJSON log to rebuild the in-memory index:

1. **Parse each log line** as a `PlanEvent`
2. **Apply events sequentially** to rebuild state
3. **Handle corrupt lines** gracefully (log error, continue)
4. **Validate consistency** between log and index

```rust
async fn replay_log(&self) -> Result<()> {
    let file = File::open(&self.log_path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    while let Some(line) = lines.next_line().await? {
        if let Ok(event) = serde_json::from_str::<PlanEvent>(&line) {
            self.apply_event(&event).await?;
        }
    }
    Ok(())
}
```

## Performance Characteristics

- **ID Generation**: O(1) Blake3 hash computation
- **Plan Storage**: O(1) DashMap insertion + O(1) log append
- **Plan Retrieval**: O(1) DashMap lookup
- **List Operations**: O(n) where n = result set size
- **Startup Recovery**: O(m) where m = total logged events

## Integration Points

### Plan UI Module
The Plan Store integrates with the existing Plan UI module through the `init_plan_store()` function, which initializes a shared store instance.

### API Endpoints
REST API endpoints use the Plan Store for all persistence operations:
- `POST /ai/plan` → `store_plan()`
- `GET /ai/plan/:id` → `get_plan_by_id()`
- `GET /ai/plans` → `list_plans()`
- `POST /ai/plan/approve` → `approve_plan()`

### CLI Tools
The `ai-plan-cli` tool uses the Plan Store directly for all operations, providing pagination, filtering, and revision tracking.

## Configuration

```rust
// Initialize with custom data directory
let store = PlanStore::new("/path/to/data/dir").await?;

// Default location: "./data/plans/"
let store = PlanStore::default().await?;
```

## Error Handling

All operations return `Result<T, PlanStoreError>` with detailed error information:

- **`PlanNotFound`**: Requested plan ID doesn't exist
- **`SerializationError`**: JSON serialization/deserialization failed
- **`IoError`**: File system operation failed
- **`HashingError`**: Blake3 hash generation failed

## Testing

Comprehensive test coverage includes:

- **Deterministic ID generation**: Identical plans produce identical IDs
- **Canonical JSON consistency**: Field order doesn't affect IDs
- **Persistence and recovery**: Data survives process restart
- **Concurrency safety**: Multiple threads can access store safely
- **Idempotent operations**: Duplicate operations are handled correctly
- **Error conditions**: Invalid inputs are handled gracefully

See `tests/ai_plan_store.rs` for full test suite.

## Migration from Legacy Storage

Plans stored using the previous in-memory approach can be migrated:

1. **Export existing plans** to JSON files
2. **Use CLI create command** to import each plan
3. **Verify IDs and metadata** after import
4. **Update approval status** as needed

```bash
# Import existing plan
ai-plan create --file plan.json --user-id original-creator

# Approve if it was previously approved
ai-plan approve <plan-id> --user-id original-approver
```