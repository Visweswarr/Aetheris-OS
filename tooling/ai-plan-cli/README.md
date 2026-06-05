# AI Plan CLI

Command-line interface for visualizing and approving AI-generated plans from the Aetheris AI service.

## Features

- **List Plans**: View all available AI plans with their approval status
- **View Plan Details**: Detailed visualization of plan steps and metadata
- **Approve Plans**: User-controlled approval workflow with logging
- **Status Checking**: Quick status checks for individual plans
- **Deterministic Logging**: All approvals are logged to `logs/ai_plans.log`

## Installation

```bash
cd tooling/ai-plan-cli
cargo build --release
```

The binary will be available at `target/release/ai-plan`.

## Usage

### List all plans
```bash
ai-plan list
```

### View a specific plan
```bash
ai-plan view <plan-id>
```

### Approve a plan
```bash
ai-plan approve <plan-id> --user-id <your-user-id>
```

### Check plan status
```bash
ai-plan status <plan-id>
```

### Enable verbose logging
```bash
ai-plan --verbose list
```

## Example Output

### Listing Plans
```
Available Plans:
================
⏳ [PENDING] plan-123 (3 steps)
  Description: Deploy new microservice to production
  
✅ [APPROVED] plan-456 (2 steps)
  Description: Update database schema
  Approved by: alice at 2024-01-15T10:30:00Z
```

### Viewing Plan Details
```
Plan Details:
=============
Plan ID: plan-123
Name: Deploy Microservice
Description: Deploy new microservice to production
Goal: Safely deploy the user authentication service

Steps:
------
1. ⏳ Validate Configuration
   Verify all environment variables and secrets
   Status: Pending

2. ⏳ Run Tests
   Execute integration and smoke tests
   Status: Pending

3. ⏳ Deploy to Production
   Rolling deployment with health checks
   Status: Pending

Approval Status: ⏳ PENDING APPROVAL
```

### Approving a Plan
```bash
$ ai-plan approve plan-123 --user-id alice
✅ Plan 'plan-123' approved successfully!
Approved by: alice
Approved at: 2024-01-15T10:35:22Z

Approved Plan:
==============
[Full plan details shown...]
```

## Integration

This CLI tool integrates with:

- **Aetheris AI Service**: Uses the `plan_ui` module for plan management
- **Deterministic Logging**: All approvals logged to `logs/ai_plans.log`
- **In-Memory Storage**: Plans stored in memory for demo purposes
- **REST API Integration**: Ready for future HTTP API endpoints

## Development

### Running Tests
```bash
cargo test
```

### Adding New Commands
1. Add the command to the `Commands` enum in `src/main.rs`
2. Implement the handler function
3. Add the match arm in `main()`
4. Add tests for the new functionality

### Logging
The tool uses `tracing` for structured logging. Set `RUST_LOG=debug` or use `--verbose` for detailed output.

## Architecture

The CLI tool follows the Aetheris OS coding patterns:

- **Deterministic Operations**: All operations are idempotent and logged
- **Error Handling**: Uses `anyhow` for error propagation
- **Structured Logging**: Uses `tracing` for observability
- **CLI Best Practices**: Uses `clap` with subcommands and proper help text
- **Testing**: Comprehensive unit tests for all functionality

## Future Enhancements

- [ ] REST API integration for remote plan management
- [ ] Plan execution tracking and status updates
- [ ] User authentication and authorization
- [ ] Plan templates and batch operations
- [ ] Integration with CI/CD pipelines