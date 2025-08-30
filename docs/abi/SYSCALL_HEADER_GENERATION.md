# System Call Header Generation System

**Date**: December 2024  
**Status**: ✅ **IMPLEMENTED**  
**Purpose**: Single source of truth for syscall definitions  
**Goal**: Prevent kernel/user stub drift through automated generation

## 🎯 **Overview**

The System Call Header Generation System ensures that kernel and user space stubs never drift by automatically generating all syscall-related headers from a single source of truth: `docs/abi/SYSCALLS.md`.

### **Key Benefits**
- **Single Source of Truth**: All syscall definitions in one place
- **Automatic Consistency**: Generated headers always match documentation
- **No Drift**: Kernel and user space definitions stay synchronized
- **CI Validation**: Automated checks prevent inconsistencies
- **Easy Maintenance**: Update documentation, regenerate headers

## 🏗️ **Architecture**

### **Source of Truth**
```
docs/abi/SYSCALLS.md
├── System call numbers and names
├── Calling conventions
├── Error codes
├── Data type definitions
└── ABI stability guarantees
```

### **Generated Artifacts**
```
Generated Headers
├── kernel/src/syscall/generated.rs      (Rust - Kernel use)
├── userland-stubs/include/polymera/syscalls.h  (C - User space)
├── kernel/src/syscall/generated.asm     (Assembly constants)
└── syscall_generation_summary.txt       (Generation report)
```

### **Generation Pipeline**
```
SYSCALLS.md → Python Parser → Header Generator → Validation → Artifacts
```

## 🔧 **Components**

### **1. Python Header Generator**
**File**: `scripts/generate_syscall_headers.py`

#### **Features**
- **Markdown Parser**: Extracts syscall definitions from SYSCALLS.md
- **Multi-Format Output**: Generates Rust, C, and Assembly headers
- **Validation**: Ensures generated headers are syntactically correct
- **Consistency Checks**: Verifies all formats contain identical information

#### **Parsing Capabilities**
- **System Call Tables**: Extracts number, name, description, arguments, return value
- **Error Code Tables**: Parses standard and Polymera OS specific errors
- **Data Type Definitions**: Extracts typedef statements and comments
- **Structured Output**: Generates well-formatted, documented headers

### **2. Build Scripts**
**Files**: 
- `scripts/build_syscall_headers.sh` (Linux/macOS)
- `scripts/build_syscall_headers.bat` (Windows)

#### **Features**
- **Cross-Platform**: Works on Linux, macOS, and Windows
- **Dependency Checking**: Verifies Python 3 and required tools
- **Output Validation**: Ensures all required files are generated
- **Syntax Validation**: Checks Rust and C header syntax
- **Git Integration**: Detects changes in generated files

### **3. Makefile Integration**
**Targets**:
- `make syscall-headers`: Generate all headers
- `make validate-syscalls`: Validate header consistency

#### **Usage**
```bash
# Generate headers
make syscall-headers

# Validate consistency
make validate-syscalls

# Both operations
make validate-syscalls
```

## 📋 **Usage Instructions**

### **Manual Generation**

#### **Linux/macOS**
```bash
# From project root
bash scripts/build_syscall_headers.sh

# Or with Python directly
python3 scripts/generate_syscall_headers.py .
```

#### **Windows**
```cmd
# From project root
scripts\build_syscall_headers.bat

# Or with Python directly
python scripts\generate_syscall_headers.py .
```

### **Automated Generation**

#### **Pre-commit Hook**
```bash
# Add to .git/hooks/pre-commit
#!/bin/bash
make syscall-headers
git add kernel/src/syscall/generated.rs
git add userland-stubs/include/polymera/syscalls.h
git add kernel/src/syscall/generated.asm
```

#### **CI Integration**
```yaml
# In CI workflow
- name: Generate syscall headers
  run: make syscall-headers
```

## 🔍 **Validation Process**

### **1. File Generation Validation**
- ✅ All required files are created
- ✅ File sizes are reasonable
- ✅ No empty or corrupted files

### **2. Syntax Validation**
- ✅ **Rust**: `rustc --edition 2021 --target x86_64-unknown-none`
- ✅ **C**: `gcc -fsyntax-only -std=c99`
- ✅ **Assembly**: Basic format validation

### **3. Consistency Validation**
- ✅ Same syscall count in all formats
- ✅ Identical syscall numbers across formats
- ✅ Matching error code definitions
- ✅ Consistent data type definitions

### **4. Drift Detection**
- ✅ Kernel and user space headers match
- ✅ Assembly constants align with headers
- ✅ No version mismatches between formats

## 🚨 **CI Gates and Validation**

### **System Call Validation Workflow**
**File**: `.github/workflows/syscall-validation.yml`

#### **Triggers**
- Changes to `kernel/src/syscall/**`
- Changes to `docs/abi/SYSCALLS.md`
- Changes to `scripts/generate_syscall_headers.py`

#### **Validation Steps**
1. **Header Generation**: Generate all headers from SYSCALLS.md
2. **File Validation**: Ensure all required files exist
3. **Syntax Validation**: Validate Rust and C syntax
4. **Change Detection**: Check for syscall handler modifications
5. **Consistency Validation**: Verify header consistency
6. **Drift Detection**: Check for kernel/user stub drift

#### **Failure Conditions**
- ❌ Header generation fails
- ❌ Generated files missing or invalid
- ❌ Syntax validation fails
- ❌ Syscall handlers changed without SYSCALLS.md update
- ❌ Header inconsistency detected
- ❌ Drift between kernel and user space

### **Blocking Rules**
- **Must Pass**: All validation steps must succeed
- **Merge Blocking**: Failed validation prevents PR merge
- **Automatic Comments**: Success/failure notifications on PRs
- **Artifact Upload**: Generated headers available for review

## 📊 **Generated Header Examples**

### **Rust Header (generated.rs)**
```rust
//! System Call Definitions - Auto-generated from SYSCALLS.md
//! Do not edit this file directly. Edit docs/abi/SYSCALLS.md instead.

#![allow(non_upper_case_globals)]
#![allow(dead_code)]

// Data Types
/// Process identifier
pub type pid_t = u64;

/// User identifier
pub type uid_t = u64;

// System Call Numbers
/// Terminate process
/// Arguments: int status
/// Returns: Never returns
pub const SYS_EXIT: i64 = 0;

/// Read from file descriptor
/// Arguments: int fd, void *buf, size_t count
/// Returns: Bytes read or error
pub const SYS_READ: i64 = 1;

// Error Codes
/// Operation not permitted
pub const EPERM: i64 = -1;

/// No such file or directory
pub const ENOENT: i64 = -2;

// Helper Functions
/// Check if a return value indicates an error
pub fn is_error(ret: i64) -> bool {
    ret < 0
}
```

### **C Header (syscalls.h)**
```c
/* System Call Definitions - Auto-generated from SYSCALLS.md */
/* Do not edit this file directly. Edit docs/abi/SYSCALLS.md instead. */

#ifndef POLYMERA_SYSCALLS_H
#define POLYMERA_SYSCALLS_H

#include <stdint.h>
#include <sys/types.h>

/* Data Types */
/* Process identifier */
typedef uint64_t pid_t;

/* User identifier */
typedef uint64_t uid_t;

/* System Call Numbers */
/* Terminate process */
/* Arguments: int status */
/* Returns: Never returns */
#define SYS_EXIT 0

/* Read from file descriptor */
/* Arguments: int fd, void *buf, size_t count */
/* Returns: Bytes read or error */
#define SYS_READ 1

/* Error Codes */
/* Operation not permitted */
#define EPERM -1

/* No such file or directory */
#define ENOENT -2

/* Helper Functions */
/* Check if a return value indicates an error */
static inline int is_error(int64_t ret) {
    return ret < 0;
}

/* System Call Function */
int64_t syscall(int64_t number, ...);

#endif /* POLYMERA_SYSCALLS_H */
```

### **Assembly Constants (generated.asm)**
```nasm
; System Call Constants - Auto-generated from SYSCALLS.md
; Do not edit this file directly. Edit docs/abi/SYSCALLS.md instead.

; System Call Numbers
; Terminate process
; Arguments: int status
; Returns: Never returns
SYS_EXIT equ 0

; Read from file descriptor
; Arguments: int fd, void *buf, size_t count
; Returns: Bytes read or error
SYS_READ equ 1

; Error Codes
; Operation not permitted
EPERM equ -1

; No such file or directory
ENOENT equ -2
```

## 🔄 **Maintenance and Updates**

### **Adding New System Calls**

#### **1. Update SYSCALLS.md**
```markdown
| 999 | `SYS_NEW_FEATURE` | New feature description | `arg1, arg2` | Success value or error |
```

#### **2. Regenerate Headers**
```bash
make syscall-headers
```

#### **3. Update Kernel Implementation**
```rust
// In kernel/src/syscall/handlers.rs
pub fn handle_new_feature(arg1: u64, arg2: u64) -> Result<u64, SyscallError> {
    // Implementation
}
```

#### **4. Update User Space Code**
```c
// In user space code
long result = syscall(SYS_NEW_FEATURE, arg1, arg2);
```

### **Modifying Existing System Calls**

#### **⚠️ Breaking Changes**
- **Never change syscall numbers** once assigned
- **Never change calling convention** for existing syscalls
- **Add new syscalls** instead of modifying existing ones

#### **Non-Breaking Changes**
- **Update descriptions** in SYSCALLS.md
- **Add new error codes** for better error handling
- **Extend data types** with new fields

### **Versioning and Migration**

#### **Major Version Changes**
- **New syscall numbers** for breaking changes
- **Deprecation notices** for old syscalls
- **Migration guides** for user space code

#### **Backward Compatibility**
- **Kernel maintains** old syscall implementations
- **User space can** use old or new interfaces
- **Gradual migration** over multiple versions

## 🧪 **Testing and Validation**

### **Unit Tests**
```bash
# Test header generation
python3 -m pytest scripts/test_header_generator.py

# Test consistency
make validate-syscalls
```

### **Integration Tests**
```bash
# Test kernel compilation with generated headers
cd kernel && cargo check

# Test user space compilation with generated headers
cd userland-stubs && make test
```

### **CI Validation**
- **Automated generation** on every PR
- **Consistency checks** between all formats
- **Syntax validation** for all generated headers
- **Drift detection** between kernel and user space

## 🚀 **Best Practices**

### **Documentation**
- **Keep SYSCALLS.md up to date** with all changes
- **Document breaking changes** clearly
- **Provide migration examples** for complex changes
- **Maintain ABI stability** documentation

### **Development Workflow**
- **Always regenerate headers** after SYSCALLS.md changes
- **Test with generated headers** before committing
- **Validate consistency** across all platforms
- **Use CI validation** to catch issues early

### **Error Handling**
- **Consistent error codes** across all syscalls
- **Clear error messages** for debugging
- **Proper error propagation** from kernel to user space
- **Error code documentation** in SYSCALLS.md

## 🔮 **Future Enhancements**

### **Advanced Features**
- **Version-aware generation** for different ABI versions
- **Backward compatibility** checking
- **Performance benchmarking** integration
- **Security validation** for syscall interfaces

### **Tooling Improvements**
- **IDE integration** for header generation
- **Real-time validation** during development
- **Automated testing** with generated headers
- **Performance impact** analysis

### **Documentation Enhancements**
- **Interactive syscall explorer** web interface
- **Automated API documentation** generation
- **Change tracking** and migration guides
- **Performance characteristics** documentation

---

**System Status**: ✅ **FULLY OPERATIONAL**  
**Maintenance**: @polymera-os-team  
**Last Updated**: December 2024  
**Next Review**: January 2025
