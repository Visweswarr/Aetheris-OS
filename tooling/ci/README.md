# Polymera OS CI Tools

This directory contains CI/CD validation tools for Polymera OS development.

## Phase 1 Documentation Guard

### Overview

The Phase 1 Documentation Guard enforces the development pattern:
**SPEC → DESIGN → TASKS → CODE/TESTS → DOCS**

It ensures that any changes to kernel code are accompanied by updates to the corresponding Phase 1 documentation.

### Files

- **`check_phase_docs.ts`** - Main validation script
- **`check_phase_docs.test.ts`** - Test suite for the validation logic
- **`package.json`** - Node.js dependencies and scripts
- **`tsconfig.json`** - TypeScript configuration

### Usage

#### In GitHub Actions

The script is automatically run by the Phase 1 guard workflow:
```yaml
# .github/workflows/phase-1-guard.yml
- name: Run Phase 1 documentation check
  working-directory: tooling/ci
  run: npx tsx check_phase_docs.ts
```

#### Local Development

```bash
# Install dependencies
cd tooling/ci
npm install

# Run the check
npm run check-phase-docs

# Run tests
npm test

# Build TypeScript
npm run build
```

#### Manual Execution

```bash
# Run directly with tsx
npx tsx check_phase_docs.ts

# Or compile and run
npx tsc
node dist/check_phase_docs.js
```

### Rule Logic

The script checks:

1. **Get changed files** - Uses `git diff` to find files changed in the PR
2. **Identify kernel changes** - Looks for files starting with `kernel/`
3. **Check documentation updates** - Looks for changes to:
   - `docs/phase-1/SPEC.md`
   - `docs/phase-1/DESIGN.md`
4. **Apply rule** - If kernel files changed, docs must also be changed

### Exit Codes

- **0** - Documentation requirements satisfied
- **1** - Documentation requirements violated
- **2** - Script error (Git not available, etc.)

### Environment Variables

The script uses these environment variables when running in CI:

- `BASE_SHA` - Base commit SHA for comparison
- `HEAD_SHA` - Head commit SHA for comparison
- `GITHUB_TOKEN` - GitHub API token
- `PR_NUMBER` - Pull request number
- `REPOSITORY` - Repository name

### Examples

#### ✅ Allowed: Kernel changes with SPEC update
```
Changed files:
  kernel/src/scheduler/mod.rs
  kernel/src/memory/allocator.rs
  docs/phase-1/SPEC.md
```

#### ✅ Allowed: Non-kernel changes
```
Changed files:
  services/wallet/src/main.rs
  ui/components/Button.tsx
  docs/README.md
```

#### ❌ Blocked: Kernel changes without docs
```
Changed files:
  kernel/src/panic.rs
  kernel/src/log.rs
```

### Testing

The test suite covers various scenarios:

```bash
npm test
```

Test cases include:
- No changes
- Non-kernel changes only
- Kernel changes with SPEC update
- Kernel changes with DESIGN update
- Kernel changes with both docs updated
- Kernel changes without doc updates
- Mixed changes
- Large refactors

### Troubleshooting

#### Common Issues

**Error: "Not in a Git repository"**
```bash
# Ensure you're in the project root
cd /path/to/polymera-os
git status
```

**Error: "Unable to determine changed files"**
```bash
# Check Git configuration
git log --oneline -5
git diff --name-only HEAD~1
```

**Error: "Cannot find module 'tsx'"**
```bash
# Install dependencies
npm install
# Or install tsx globally
npm install -g tsx
```

#### Debug Mode

Add debug output by setting environment variables:
```bash
export DEBUG=1
export VERBOSE=1
npm run check-phase-docs
```

### Integration

#### GitHub Actions Integration

The script integrates with GitHub Actions through:

1. **Workflow trigger** - Runs on PRs touching `kernel/**` or `docs/phase-1/**`
2. **Environment setup** - Node.js and npm dependencies
3. **Git context** - Access to changed files via Git commands
4. **Status reporting** - Exit codes and formatted output

#### Local Git Hooks

You can also run this as a Git pre-push hook:

```bash
#!/bin/sh
# .git/hooks/pre-push
cd tooling/ci
npm run check-phase-docs
```

### Development

#### Adding New Rules

To add new validation rules:

1. Modify `analyzeChanges()` to detect new patterns
2. Update `validatePhase1Requirements()` with new logic
3. Add test cases to `check_phase_docs.test.ts`
4. Update this README

#### Code Style

The project uses:
- TypeScript with strict mode
- ESLint for linting
- Consistent error handling
- Comprehensive logging

### License

Apache 2.0 - See the project root LICENSE file.
