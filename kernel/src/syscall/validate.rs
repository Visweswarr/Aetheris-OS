/// System Call Argument Validation Module
/// 
/// This module provides centralized syscall argument validation with per-syscall schemas
/// for pointer ranges, alignment, length caps, and bounds checking.

use super::table::*;
use crate::{kprintln, klog};
use crate::log::Level;
use crate::secman::audit::{audit_log, AuditEvent, AuditLevel};
use core::mem;
use alloc::string::ToString;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

//=============================================================================
// VALIDATION CONSTANTS AND CONFIGURATION
//=============================================================================

/// Maximum size for any single syscall argument
pub const MAX_ARG_SIZE: usize = 64 * 1024; // 64KB

/// Maximum total size for all syscall arguments combined
pub const MAX_TOTAL_ARGS_SIZE: usize = 128 * 1024; // 128KB

/// Maximum number of syscall arguments
pub const MAX_SYSCALL_ARGS: usize = 4;

/// Default alignment requirements for different data types
pub const ALIGN_U8: usize = 1;
pub const ALIGN_U16: usize = 2;
pub const ALIGN_U32: usize = 4;
pub const ALIGN_U64: usize = 8;
pub const ALIGN_PTR: usize = 8;

/// User space memory boundaries
pub const USER_MEMORY_BASE: u64 = 0x400000; // 4MB
pub const USER_MEMORY_TOP: u64 = 0x7fffffffffff; // 47-bit user space

/// Kernel space memory boundaries (for guard checks)
pub const KERNEL_MEMORY_BASE: u64 = 0xffff800000000000;
pub const KERNEL_MEMORY_TOP: u64 = 0xffffffffffffffff;

//=============================================================================
// VALIDATION SCHEMAS AND TYPES
//=============================================================================

/// Argument type for validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgType {
    /// Unsigned integer types
    U8, U16, U32, U64,
    /// Signed integer types
    I8, I16, I32, I64,
    /// Pointer types
    Ptr, PtrMut, PtrConst,
    /// Buffer types
    Buffer(usize), // max size
    BufferMut(usize), // max size
    /// Boolean type
    Bool,
    /// String type
    String(usize), // max length
    /// Custom validation function
    Custom(fn(u64) -> bool),
}

impl ArgType {
    /// Get the alignment requirement for this type
    pub fn alignment(&self) -> usize {
        match self {
            ArgType::U8 | ArgType::I8 => ALIGN_U8,
            ArgType::U16 | ArgType::I16 => ALIGN_U16,
            ArgType::U32 | ArgType::I32 => ALIGN_U32,
            ArgType::U64 | ArgType::I64 | ArgType::Ptr | ArgType::PtrMut | ArgType::PtrConst => ALIGN_PTR,
            ArgType::Buffer(_) | ArgType::BufferMut(_) => ALIGN_PTR,
            ArgType::Bool => ALIGN_U8,
            ArgType::String(_) => ALIGN_PTR,
            ArgType::Custom(_) => ALIGN_PTR, // Assume pointer alignment for custom types
        }
    }
    
    /// Get the size in bytes for this type
    pub fn size(&self) -> usize {
        match self {
            ArgType::U8 | ArgType::I8 => 1,
            ArgType::U16 | ArgType::I16 => 2,
            ArgType::U32 | ArgType::I32 => 4,
            ArgType::U64 | ArgType::I64 | ArgType::Ptr | ArgType::PtrMut | ArgType::PtrConst => 8,
            ArgType::Buffer(_) | ArgType::BufferMut(_) => 8, // Pointer size
            ArgType::Bool => 1,
            ArgType::String(_) => 8, // Pointer size
            ArgType::Custom(_) => 8, // Assume pointer size for custom types
        }
    }
    
    /// Check if this type requires pointer validation
    pub fn is_pointer(&self) -> bool {
        matches!(self, 
            ArgType::Ptr | ArgType::PtrMut | ArgType::PtrConst | 
            ArgType::Buffer(_) | ArgType::BufferMut(_) | 
            ArgType::String(_)
        )
    }
}

/// Argument validation schema
#[derive(Debug, Clone)]
pub struct ArgSchema {
    /// Argument name
    pub name: String,
    /// Argument type
    pub arg_type: ArgType,
    /// Whether the argument is required
    pub required: bool,
    /// Minimum value (for numeric types)
    pub min_value: Option<u64>,
    /// Maximum value (for numeric types)
    pub max_value: Option<u64>,
    /// Custom validation function
    pub validator: Option<fn(u64) -> bool>,
    /// Description for error reporting
    pub description: String,
}

impl ArgSchema {
    /// Create a new argument schema
    pub fn new(name: &str, arg_type: ArgType, description: &str) -> Self {
        Self {
            name: name.to_string(),
            arg_type,
            required: true,
            min_value: None,
            max_value: None,
            validator: None,
            description: description.to_string(),
        }
    }
    
    /// Set minimum value constraint
    pub fn with_min(mut self, min: u64) -> Self {
        self.min_value = Some(min);
        self
    }
    
    /// Set maximum value constraint
    pub fn with_max(mut self, max: u64) -> Self {
        self.max_value = Some(max);
        self
    }
    
    /// Set custom validator
    pub fn with_validator(mut self, validator: fn(u64) -> bool) -> Self {
        self.validator = Some(validator);
        self
    }
    
    /// Set as optional
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

/// Syscall validation schema
#[derive(Debug, Clone)]
pub struct SyscallSchema {
    /// Syscall ID
    pub id: u64,
    /// Syscall name
    pub name: String,
    /// Argument schemas
    pub args: Vec<ArgSchema>,
    /// Return type
    pub return_type: String,
    /// Error codes that can be returned
    pub error_codes: Vec<String>,
    /// Whether this syscall is implemented
    pub implemented: bool,
    /// Category for grouping
    pub category: String,
}

impl SyscallSchema {
    /// Create a new syscall schema
    pub fn new(id: u64, name: &str, category: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            args: Vec::new(),
            return_type: "u64".to_string(),
            error_codes: Vec::new(),
            implemented: false,
            category: category.to_string(),
        }
    }
    
    /// Add an argument to the schema
    pub fn with_arg(mut self, arg: ArgSchema) -> Self {
        self.args.push(arg);
        self
    }
    
    /// Set the return type
    pub fn with_return_type(mut self, return_type: &str) -> Self {
        self.return_type = return_type.to_string();
        self
    }
    
    /// Add error codes
    pub fn with_error_codes(mut self, error_codes: &[&str]) -> Self {
        self.error_codes = error_codes.iter().map(|s| s.to_string()).collect();
        self
    }
    
    /// Mark as implemented
    pub fn implemented(mut self) -> Self {
        self.implemented = true;
        self
    }
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// Error code if validation failed
    pub error_code: Option<String>,
    /// Error message for debugging
    pub error_message: Option<String>,
    /// Which argument failed validation (0-indexed)
    pub failed_arg: Option<usize>,
    /// Additional context for the error
    pub context: Option<String>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn success() -> Self {
        Self {
            is_valid: true,
            error_code: None,
            error_message: None,
            failed_arg: None,
            context: None,
        }
    }
    
    /// Create a failed validation result
    pub fn failure(error_code: &str, message: &str, failed_arg: Option<usize>) -> Self {
        Self {
            is_valid: false,
            error_code: Some(error_code.to_string()),
            error_message: Some(message.to_string()),
            failed_arg,
            context: None,
        }
    }
    
    /// Add context to the validation result
    pub fn with_context(mut self, context: &str) -> Self {
        self.context = Some(context.to_string());
        self
    }
}

//=============================================================================
// VALIDATION FUNCTIONS
//=============================================================================

/// Validate a single syscall argument
fn validate_arg(arg: u64, schema: &ArgSchema, arg_index: usize) -> ValidationResult {
    // Check if argument is required
    if schema.required && arg == 0 {
        return ValidationResult::failure(
            "EINVAL",
            &format!("Argument '{}' is required but was 0", schema.name),
            Some(arg_index)
        );
    }
    
    // Check minimum value constraint
    if let Some(min) = schema.min_value {
        if arg < min {
            return ValidationResult::failure(
                "EINVAL",
                &format!("Argument '{}' value {} is below minimum {}", schema.name, arg, min),
                Some(arg_index)
            );
        }
    }
    
    // Check maximum value constraint
    if let Some(max) = schema.max_value {
        if arg > max {
            return ValidationResult::failure(
                "EINVAL",
                &format!("Argument '{}' value {} exceeds maximum {}", schema.name, arg, max),
                Some(arg_index)
            );
        }
    }
    
    // Check alignment
    let alignment = schema.arg_type.alignment();
    if arg % alignment as u64 != 0 {
        return ValidationResult::failure(
            "EINVAL",
            &format!("Argument '{}' value 0x{:x} is not aligned to {} bytes", 
                     schema.name, arg, alignment),
            Some(arg_index)
        );
    }
    
    // Validate pointer types
    if schema.arg_type.is_pointer() && arg != 0 {
        if !is_valid_user_pointer(arg) {
            return ValidationResult::failure(
                "EFAULT",
                &format!("Argument '{}' pointer 0x{:x} is not a valid user space address", 
                         schema.name, arg),
                Some(arg_index)
            );
        }
        
        // Additional pointer-specific validation
        match &schema.arg_type {
            ArgType::Buffer(max_size) | ArgType::BufferMut(max_size) => {
                if let Some(size) = get_buffer_size(arg) {
                    if size > *max_size {
                        return ValidationResult::failure(
                            "EINVAL",
                            &format!("Argument '{}' buffer size {} exceeds maximum {}", 
                                     schema.name, size, max_size),
                            Some(arg_index)
                        );
                    }
                }
            }
            ArgType::String(max_length) => {
                if let Some(length) = get_string_length(arg) {
                    if length > *max_length {
                        return ValidationResult::failure(
                            "EINVAL",
                            &format!("Argument '{}' string length {} exceeds maximum {}", 
                                     schema.name, length, max_length),
                            Some(arg_index)
                        );
                    }
                }
            }
            _ => {}
        }
    }
    
    // Custom validation
    if let Some(validator) = schema.validator {
        if !validator(arg) {
            return ValidationResult::failure(
                "EINVAL",
                &format!("Argument '{}' failed custom validation", schema.name),
                Some(arg_index)
            );
        }
    }
    
    ValidationResult::success()
}

/// Validate all arguments for a syscall
pub fn validate_syscall_args(
    syscall_id: u64,
    args: &[u64; MAX_SYSCALL_ARGS],
) -> ValidationResult {
    // Get the schema for this syscall
    let schema = match get_syscall_schema(syscall_id) {
        Some(s) => s,
        None => {
            return ValidationResult::failure(
                "EINVAL",
                &format!("Unknown syscall ID: {}", syscall_id),
                None
            );
        }
    };
    
    // Check if syscall is implemented
    if !schema.implemented {
        return ValidationResult::failure(
            "ENOSYS",
            &format!("Syscall '{}' is not implemented", schema.name),
            None
        );
    }
    
    // Validate each argument
    for (i, arg_schema) in schema.args.iter().enumerate() {
        if i >= MAX_SYSCALL_ARGS {
            break;
        }
        
        let arg_value = args[i];
        let validation_result = validate_arg(arg_value, arg_schema, i);
        
        if !validation_result.is_valid {
            // Log validation failure for audit
            audit_log(
                AuditEvent::SyscallValidationFailure,
                AuditLevel::WARNING,
                &format!("Syscall {} validation failed: arg {} '{}': {}", 
                         syscall_id, i, arg_schema.name, 
                         validation_result.error_message.as_ref().unwrap_or(&"Unknown error".to_string())),
            );
            
            return validation_result.with_context(&format!("Syscall: {}", schema.name));
        }
    }
    
    // Check total argument size constraints
    let total_size = schema.args.iter()
        .map(|arg| arg.arg_type.size())
        .sum::<usize>();
    
    if total_size > MAX_TOTAL_ARGS_SIZE {
        return ValidationResult::failure(
            "EINVAL",
            &format!("Total argument size {} exceeds maximum {}", total_size, MAX_TOTAL_ARGS_SIZE),
            None
        );
    }
    
    ValidationResult::success()
}

/// Check if a pointer is a valid user space address
pub fn is_valid_user_pointer(ptr: u64) -> bool {
    // Check if pointer is in user space range
    if ptr < USER_MEMORY_BASE || ptr > USER_MEMORY_TOP {
        return false;
    }
    
    // Check if pointer is aligned
    if ptr % ALIGN_PTR as u64 != 0 {
        return false;
    }
    
    // Additional checks could be added here:
    // - Check if the page is mapped
    // - Check if the page is readable/writable
    // - Check if the page is in user space
    
    true
}

/// Check if a pointer is in kernel space (for guard checks)
pub fn is_kernel_pointer(ptr: u64) -> bool {
    ptr >= KERNEL_MEMORY_BASE && ptr <= KERNEL_MEMORY_TOP
}

/// Get buffer size from a pointer (stub implementation)
fn get_buffer_size(ptr: u64) -> Option<usize> {
    // In a real implementation, this would:
    // 1. Check if the page is mapped
    // 2. Look for length information in the buffer header
    // 3. Check against page boundaries
    
    // For now, return a reasonable default
    Some(1024) // 1KB default
}

/// Get string length from a pointer (stub implementation)
fn get_string_length(ptr: u64) -> Option<usize> {
    // In a real implementation, this would:
    // 1. Check if the page is mapped
    // 2. Scan for null terminator
    // 3. Check against page boundaries
    
    // For now, return a reasonable default
    Some(256) // 256 chars default
}

//=============================================================================
// SYSCALL SCHEMA DEFINITIONS
//=============================================================================

/// Get the validation schema for a syscall
pub fn get_syscall_schema(syscall_id: u64) -> Option<SyscallSchema> {
    match syscall_id {
        SYS_YIELD => Some(SyscallSchema::new(SYS_YIELD, "yield", "task_management")
            .with_return_type("u64")
            .with_error_codes(&[])
            .implemented()),
            
        SYS_EXIT => Some(SyscallSchema::new(SYS_EXIT, "exit", "task_management")
            .with_arg(ArgSchema::new("code", ArgType::I32, "Exit code"))
            .with_return_type("!")
            .with_error_codes(&[])
            .implemented()),
            
        SYS_SEND => Some(SyscallSchema::new(SYS_SEND, "send", "ipc")
            .with_arg(ArgSchema::new("dst", ArgType::U64, "Destination task ID")
                .with_min(1)
                .with_max(10000))
            .with_arg(ArgSchema::new("buf", ArgType::Buffer(8192), "Message buffer")
                .with_min(1))
            .with_return_type("u64")
            .with_error_codes(&["EPERM", "EINVAL", "ENOSPC"])
            .implemented()),
            
        SYS_RECV => Some(SyscallSchema::new(SYS_RECV, "recv", "ipc")
            .with_arg(ArgSchema::new("block", ArgType::Bool, "Whether to block"))
            .with_arg(ArgSchema::new("out", ArgType::BufferMut(8192), "Output buffer")
                .with_min(1))
            .with_return_type("u64")
            .with_error_codes(&["EAGAIN", "EINVAL", "EFAULT"])
            .implemented()),
            
        SYS_CHAN_CREATE => Some(SyscallSchema::new(SYS_CHAN_CREATE, "chan_create", "ipc")
            .with_arg(ArgSchema::new("capacity", ArgType::U64, "Channel capacity")
                .with_min(1)
                .with_max(1000000))
            .with_return_type("u64")
            .with_error_codes(&["ENOMEM", "EINVAL"])
            .implemented()),
            
        SYS_EXEC => Some(SyscallSchema::new(SYS_EXEC, "exec", "task_management")
            .with_arg(ArgSchema::new("image", ArgType::Buffer(1024*1024), "Executable image")
                .with_min(1))
            .with_arg(ArgSchema::new("caps", ArgType::Buffer(1024), "Capabilities")
                .optional())
            .with_return_type("u64")
            .with_error_codes(&["EINVAL", "ENOMEM", "EPERM"])
            .implemented()),
            
        SYS_STATS => Some(SyscallSchema::new(SYS_STATS, "stats", "system")
            .with_arg(ArgSchema::new("stats_type", ArgType::U64, "Statistics type")
                .with_max(100))
            .with_return_type("u64")
            .with_error_codes(&["EINVAL"])
            .implemented()),
            
        SYS_DEBUG => Some(SyscallSchema::new(SYS_DEBUG, "debug", "system")
            .with_arg(ArgSchema::new("op", ArgType::U64, "Operation code")
                .with_max(1000))
            .with_arg(ArgSchema::new("arg1", ArgType::U64, "First argument")
                .optional())
            .with_arg(ArgSchema::new("arg2", ArgType::U64, "Second argument")
                .optional())
            .with_arg(ArgSchema::new("arg3", ArgType::U64, "Third argument")
                .optional())
            .with_return_type("u64")
            .with_error_codes(&["EINVAL", "EPERM"])
            .implemented()),
            
        _ => None
    }
}

/// Get all syscall schemas
pub fn get_all_syscall_schemas() -> Vec<SyscallSchema> {
    let mut schemas = Vec::new();
    
    for id in 1..=100 { // Reasonable range for syscall IDs
        if let Some(schema) = get_syscall_schema(id) {
            schemas.push(schema);
        }
    }
    
    schemas
}

/// Validate a syscall before execution
pub fn validate_syscall(
    syscall_id: u64,
    args: &[u64; MAX_SYSCALL_ARGS],
) -> Result<(), String> {
    let validation_result = validate_syscall_args(syscall_id, args);
    
    if validation_result.is_valid {
        Ok(())
    } else {
        let error_msg = validation_result.error_message
            .unwrap_or_else(|| "Unknown validation error".to_string());
        let error_code = validation_result.error_code
            .unwrap_or_else(|| "EINVAL".to_string());
        
        Err(format!("{}: {}", error_code, error_msg))
    }
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arg_type_alignment() {
        assert_eq!(ArgType::U8.alignment(), 1);
        assert_eq!(ArgType::U16.alignment(), 2);
        assert_eq!(ArgType::U32.alignment(), 4);
        assert_eq!(ArgType::U64.alignment(), 8);
        assert_eq!(ArgType::Ptr.alignment(), 8);
    }
    
    #[test]
    fn test_arg_type_size() {
        assert_eq!(ArgType::U8.size(), 1);
        assert_eq!(ArgType::U16.size(), 2);
        assert_eq!(ArgType::U32.size(), 4);
        assert_eq!(ArgType::U64.size(), 8);
        assert_eq!(ArgType::Ptr.size(), 8);
    }
    
    #[test]
    fn test_pointer_validation() {
        // Valid user space pointer
        assert!(is_valid_user_pointer(0x400000));
        
        // Invalid kernel space pointer
        assert!(!is_valid_user_pointer(0xffff800000000000));
        
        // Invalid null pointer
        assert!(!is_valid_user_pointer(0));
        
        // Invalid unaligned pointer
        assert!(!is_valid_user_pointer(0x400001));
    }
    
    #[test]
    fn test_syscall_validation() {
        // Valid yield syscall
        let args = [0, 0, 0, 0];
        let result = validate_syscall(SYS_YIELD, &args);
        assert!(result.is_ok());
        
        // Valid exit syscall
        let args = [0, 0, 0, 0];
        let result = validate_syscall(SYS_EXIT, &args);
        assert!(result.is_ok());
        
        // Invalid syscall ID
        let args = [0, 0, 0, 0];
        let result = validate_syscall(999, &args);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_arg_schema_validation() {
        let schema = ArgSchema::new("test", ArgType::U64, "Test argument")
            .with_min(10)
            .with_max(100);
        
        // Valid value
        let result = validate_arg(50, &schema, 0);
        assert!(result.is_valid);
        
        // Value too low
        let result = validate_arg(5, &schema, 0);
        assert!(!result.is_valid);
        assert_eq!(result.error_code, Some("EINVAL".to_string()));
        
        // Value too high
        let result = validate_arg(150, &schema, 0);
        assert!(!result.is_valid);
        assert_eq!(result.error_code, Some("EINVAL".to_string()));
    }
}
