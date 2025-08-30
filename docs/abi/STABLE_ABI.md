# Stable ABI System

## Overview

The Stable ABI system provides a machine-readable schema for Polymera OS system calls and automatically generates all necessary components to ensure consistency and stability across the entire system call interface.

## Architecture

### Core Components

#### 1. **Schema Definition** (`abi/syscalls.yaml`)
The central source of truth for all system call definitions:

```yaml
syscalls:
  - id: 1
    name: "yield"
    description: "Yield CPU to another task"
    args: []
    return_type: "u64"
    return_description: "0 on success"
    error_codes: []
    implemented: true
    category: "task_management"
```

#### 2. **ABI Generator** (`tooling/abi/gen.rs`)
The core tool that reads the schema and generates:

- Kernel dispatch table (`kernel/src/syscall/table.rs`)
- Kernel handler skeletons (`kernel/src/syscall/handlers.rs`)
- Userland stub functions (`userland-stubs/src/lib.rs`)
- C header for external toolchains (`include/polymera_syscalls.h`)
- Markdown documentation (`docs/abi/SYSCALLS.md`)
- Schema hash for validation (`kernel/src/syscall/schema_hash.rs`)

#### 3. **Schema Validation** (`kernel/src/syscall/schema_validation.rs`)
Build-time validation to ensure generated files match the current schema.

#### 4. **Conformance Testing** (`kernel/src/syscall/conformance_test.rs`)
Runtime testing to verify syscall behavior matches the ABI specification.

### Data Flow

```
abi/syscalls.yaml → ABI Generator → Generated Files → Validation → CI Gates
       ↓                    ↓              ↓            ↓          ↓
   Schema Input      Code Generation   Artifacts   Hash Check   Diff Check
```

## Usage

### 1. **Adding a New System Call**

#### Step 1: Update the Schema
Add the new syscall to `abi/syscalls.yaml`:

```yaml
- id: 21
  name: "new_syscall"
  description: "Description of the new syscall"
  args:
    - name: "arg1"
      type: "u64"
      description: "First argument description"
  return_type: "u64"
  return_description: "Return value description"
  error_codes: ["EINVAL", "EPERM"]
  implemented: false
  category: "task_management"
```

#### Step 2: Regenerate Artifacts
Run the ABI generator:

```bash
cd tooling/abi
cargo run -- ../../abi/syscalls.yaml ../../
```

#### Step 3: Implement the Handler
The generator creates a skeleton handler in `kernel/src/syscall/handlers.rs`:

```rust
fn handle_new_syscall(a0: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    klog!(TRACE, "[SYSCALL] Handling new_syscall syscall");
    
    // TODO: Implement new_syscall handler
    kprintln!("[SYSCALL] new_syscall not yet implemented");
    
    0 // Success
}
```

#### Step 4: Update Implementation Status
Set `implemented: true` in the schema and regenerate.

### 2. **Modifying Existing System Calls**

#### Non-Breaking Changes
- Update descriptions
- Add new error codes
- Modify categories
- Change implementation status

#### Breaking Changes (Not Allowed in Stable ABI)
- Changing syscall numbers
- Modifying argument types
- Changing return types
- Removing arguments

### 3. **Running the Generator**

```bash
# From project root
cd tooling/abi
cargo run -- ../../abi/syscalls.yaml ../../

# Or build and run directly
cargo build --release
./target/release/gen ../../abi/syscalls.yaml ../../
```

## Generated Artifacts

### 1. **Kernel Dispatch Table** (`kernel/src/syscall/table.rs`)

```rust
/// Auto-generated System Call Number Table for Polymera OS
/// Generated from abi/syscalls.yaml
/// DO NOT EDIT MANUALLY - Run tooling/abi/gen.rs to regenerate

/// System call: Yield CPU to another task
pub const SYS_YIELD: u64 = 1;

/// System call: Exit current task
pub const SYS_EXIT: u64 = 2;

// ... more constants ...

/// System call table with metadata
pub const SYSCALL_TABLE: &[SyscallInfo] = &[
    SyscallInfo {
        number: 1,
        name: "yield",
        arg_count: 0,
        implemented: true,
        description: "Yield CPU to another task",
        category: "Task Management",
    },
    // ... more entries ...
];
```

### 2. **Kernel Handlers** (`kernel/src/syscall/handlers.rs`)

```rust
/// Auto-generated System Call Handlers for Polymera OS
/// Generated from abi/syscalls.yaml
/// DO NOT EDIT MANUALLY - Run tooling/abi/gen.rs to regenerate

pub fn dispatch(num: u64, a0: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    // ... validation logic ...
    
    match num {
        SYS_YIELD => handle_yield(a0, a1, a2, a3),
        SYS_EXIT => handle_exit(a0, a1, a2, a3),
        // ... more handlers ...
        _ => {
            klog!(TRACE, "[SYSCALL] Dispatcher missing handler for syscall {}", num);
            u64::MAX
        }
    }
}

/// Handle yield system call
fn handle_yield(a0: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    klog!(TRACE, "[SYSCALL] Handling yield syscall");
    
    // TODO: Implement yield handler
    kprintln!("[SYSCALL] yield not yet implemented");
    
    0 // Success
}
```

### 3. **Userland Stubs** (`userland-stubs/src/lib.rs`)

```rust
/// Auto-generated Userland syscall stubs for Polymera OS
/// Generated from abi/syscalls.yaml
/// DO NOT EDIT MANUALLY - Run tooling/abi/gen.rs to regenerate

/// Yield CPU to another task
/// 
/// This syscall voluntarily gives up the CPU to allow other tasks to run.
/// 
/// # Returns
/// Always returns 0 on success
pub fn sys_yield() -> u64 { 
    unsafe { syscall(1, 0, 0, 0, 0) } 
}

/// Exit current task
/// 
/// Terminates the current task with the specified exit code.
/// 
/// # Arguments
/// * `code` - Exit code (typically 0 for success, non-zero for error)
/// 
/// # Returns
/// Never returns
pub fn sys_exit(code: i32) -> ! { 
    let result = unsafe { syscall(2, code as u64, 0, 0, 0) };
    loop { core::hint::spin_loop(); }
}
```

### 4. **C Header** (`include/polymera_syscalls.h`)

```c
/* Auto-generated C header for Polymera OS syscalls */
/* Generated from abi/syscalls.yaml */
/* DO NOT EDIT MANUALLY - Run tooling/abi/gen.rs to regenerate */

#ifndef POLYMERA_SYSCALLS_H
#define POLYMERA_SYSCALLS_H

#include <stdint.h>
#include <stddef.h>

/* Error codes */
#define ESUCCESS 0
#define EPERM 1
#define EINVAL 2
/* ... more error codes ... */

/* System call numbers */
#define SYS_YIELD 1
#define SYS_EXIT 2
/* ... more syscall numbers ... */

/* System call function declarations */
uint64_t sys_yield(void);
void sys_exit(int32_t code);
/* ... more function declarations ... */

#endif /* POLYMERA_SYSCALLS_H */
```

### 5. **Documentation** (`docs/abi/SYSCALLS.md`)

```markdown
# Polymera OS System Calls

This document describes all system calls available in Polymera OS.
Generated from `abi/syscalls.yaml` - DO NOT EDIT MANUALLY.

**Schema Version:** 1.0.0
**Last Updated:** 2024-01-01
**ABI Version:** 1.0.0
**Stability:** stable

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 0 | ESUCCESS | |
| 1 | EPERM | |
| 2 | EINVAL | |

## Task Management

System calls for creating, managing, and controlling tasks/processes

### 1. yield

Yield CPU to another task

**Arguments:** None

**Returns:** `u64` - 0 on success

**Error Codes:** None

**Status:** ✅ Implemented
```

## Validation and Testing

### 1. **Schema Hash Validation**

The generator creates a hash of the schema content:

```rust
// kernel/src/syscall/schema_hash.rs
pub const SCHEMA_HASH: &str = "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6";
```

This hash is used to:
- Detect schema changes
- Ensure generated files are up-to-date
- Validate ABI consistency

### 2. **Build-Time Validation**

```rust
// kernel/src/syscall/schema_validation.rs
pub fn validate_schema_hash() -> bool {
    assert!(!SCHEMA_HASH.is_empty(), "Schema hash cannot be empty");
    // Additional validation logic
    true
}
```

### 3. **Conformance Testing**

```rust
// kernel/src/syscall/conformance_test.rs
pub fn run_conformance_tests() -> Result<bool, Box<dyn std::error::Error>> {
    let mut test_suite = ConformanceTestSuite::new();
    test_suite.run_all_tests()?;
    
    let all_passed = test_suite.all_tests_passed();
    
    if all_passed {
        println!("🎉 All conformance tests passed!");
    } else {
        println!("⚠️ Some conformance tests failed!");
    }
    
    Ok(all_passed)
}
```

## CI Integration

### 1. **ABI Validation Workflow**

The `.github/workflows/abi-validation.yml` workflow:

- Triggers on changes to ABI-related files
- Builds and runs the ABI generator
- Validates generated files match the schema
- Runs conformance tests
- Checks for breaking changes in PRs

### 2. **Validation Steps**

#### Schema Consistency Check
```bash
# Generate artifacts from current schema
./tooling/abi/gen ../../abi/syscalls.yaml ../../

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo "❌ Generated files are out of sync with schema!"
    exit 1
fi
```

#### Breaking Change Detection
```bash
# Compare syscall numbers
if diff -r base/kernel/src/syscall/table.rs kernel/src/syscall/table.rs | grep -E "^[<>].*SYS_.*:"; then
    echo "❌ ABI breaking change detected: syscall numbers changed!"
    exit 1
fi

# Compare function signatures
if diff -r base/userland-stubs/src/lib.rs userland-stubs/src/lib.rs | grep -E "^[<>].*pub fn sys_"; then
    echo "❌ ABI breaking change detected: syscall signatures changed!"
    exit 1
fi
```

### 3. **PR Comments**

The workflow automatically comments on PRs with validation results:

```markdown
## 🔍 ABI Validation Results

✅ **Schema Consistency**: Generated files match the current schema
✅ **No Breaking Changes**: ABI remains stable
✅ **Conformance Tests**: All syscalls pass validation

**Schema Hash**: `a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6`

**Generated Files**:
- Kernel dispatch table: `kernel/src/syscall/table.rs`
- Kernel handlers: `kernel/src/syscall/handlers.rs`
- Userland stubs: `userland-stubs/src/lib.rs`
- C header: `include/polymera_syscalls.h`
- Documentation: `docs/abi/SYSCALLS.md`
```

## Best Practices

### 1. **Schema Maintenance**

- **Never edit generated files manually**
- **Always update the schema first**
- **Use descriptive names and descriptions**
- **Categorize syscalls appropriately**
- **Document all error codes**

### 2. **Versioning Strategy**

- **Major version bumps**: Breaking changes (not allowed in stable ABI)
- **Minor version bumps**: New syscalls, new error codes
- **Patch version bumps**: Documentation updates, bug fixes

### 3. **Testing Strategy**

- **Unit tests**: Test individual components
- **Integration tests**: Test syscall flow
- **Conformance tests**: Verify ABI compliance
- **Performance tests**: Measure overhead

### 4. **Documentation Strategy**

- **Keep schema descriptions clear and concise**
- **Document all error conditions**
- **Provide usage examples**
- **Maintain changelog**

## Troubleshooting

### 1. **Common Issues**

#### Generated Files Out of Sync
```bash
# Regenerate all artifacts
cd tooling/abi
cargo run -- ../../abi/syscalls.yaml ../../

# Check for uncommitted changes
git status
```

#### Schema Validation Failures
```bash
# Check schema syntax
cd abi
python -c "import yaml; yaml.safe_load(open('syscalls.yaml', 'r'))"

# Validate schema hash
cd kernel/src/syscall
cargo check --features schema_validation
```

#### Conformance Test Failures
```bash
# Run tests with verbose output
cd kernel/src/syscall
cargo test conformance_test --features schema_validation -- --nocapture
```

### 2. **Debugging Tips**

- **Check schema syntax**: Use YAML validators
- **Verify file paths**: Ensure generator can access all files
- **Check permissions**: Ensure write access to output directories
- **Review logs**: Check CI workflow output for detailed error messages

## Future Enhancements

### 1. **Planned Features**

- **Multi-language support**: Generate bindings for Python, Go, etc.
- **Version compatibility**: Support multiple ABI versions
- **Performance profiling**: Measure syscall overhead
- **Security analysis**: Detect potential vulnerabilities

### 2. **Tooling Improvements**

- **Interactive schema editor**: GUI for schema management
- **Visualization tools**: Diagrams of syscall relationships
- **Migration helpers**: Tools for ABI version upgrades
- **Documentation generators**: Multiple output formats

## Conclusion

The Stable ABI system provides:

- **Consistency**: All components generated from single source
- **Stability**: Breaking changes prevented by CI gates
- **Maintainability**: Schema-driven development workflow
- **Documentation**: Auto-generated comprehensive docs
- **Testing**: Built-in validation and conformance testing

This system ensures that Polymera OS maintains a stable, well-documented, and thoroughly tested system call interface that can evolve safely over time.
