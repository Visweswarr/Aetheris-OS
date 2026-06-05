/// User Task Loader for Polymera OS
/// 
/// This module provides functionality for loading and validating user task images,
/// creating user task contexts with syscall gates, and managing the user/kernel boundary.

use crate::{kprintln, klog, format, lazy_static};
use crate::log::Level;
use crate::secman::cap_v2::{CapTokenV2, CapValidationResult, CapValidationFailure};
use crate::secman::audit::{AuditEntry, ops};
use crate::mm::{MemoryResult, MemoryError, VirtualAddress, PhysicalAddress, PageSize, MemoryFlags};
use crate::mm::vm::VirtualMemoryManager;
use super::header::{UserTaskHeader, ImageFormat, HeaderParseResult, create_test_header, create_invalid_header};
use core::sync::atomic::{AtomicU64, Ordering};
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;

//=============================================================================
// LOADER CONSTANTS AND CONFIGURATION
//=============================================================================

/// Maximum number of user tasks
pub const MAX_USER_TASKS: usize = 100;

/// Default user task stack size
pub const DEFAULT_USER_STACK_SIZE: usize = 64 * 1024; // 64KB

/// User task memory base address
pub const USER_MEMORY_BASE: u64 = 0x1000000; // 16MB

/// User task memory limit
pub const USER_MEMORY_LIMIT: u64 = 0x10000000; // 256MB

/// User task entry point offset
pub const USER_ENTRY_OFFSET: u64 = 0x1000;

//=============================================================================
// USER TASK CONTEXT STRUCTURES
//=============================================================================

/// User task context
#[derive(Debug, Clone)]
pub struct UserTaskContext {
    /// Process ID
    pub pid: u64,
    
    /// Task header
    pub header: UserTaskHeader,
    
    /// Memory regions
    pub memory_regions: Vec<UserMemoryRegion>,
    
    /// Stack information
    pub stack_info: UserStackInfo,
    
    /// Capability tokens
    pub capability_tokens: Vec<CapTokenV2>,
    
    /// Syscall gate address
    pub syscall_gate: u64,
    
    /// Entry point address
    pub entry_point: u64,
    
    /// Task status
    pub status: UserTaskStatus,
    
    /// Creation timestamp
    pub created_at: u64,
}

/// User memory region
#[derive(Debug, Clone)]
pub struct UserMemoryRegion {
    /// Region name
    pub name: String,
    
    /// Start address
    pub start_address: u64,
    
    /// Size in bytes
    pub size: usize,
    
    /// Memory protection flags
    pub protection: crate::mm::MemoryFlags,
    
    /// Whether region is mapped
    pub mapped: bool,
}

/// User stack information
#[derive(Debug, Clone)]
pub struct UserStackInfo {
    /// Stack base address
    pub base_address: u64,
    
    /// Stack size
    pub size: usize,
    
    /// Current stack pointer
    pub current_sp: u64,
    
    /// Stack protection enabled
    pub protection_enabled: bool,
}

/// User task status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserTaskStatus {
    /// Task is created but not yet started
    Created,
    /// Task is running
    Running,
    /// Task is suspended
    Suspended,
    /// Task has exited
    Exited,
    /// Task has crashed
    Crashed,
}

impl core::fmt::Display for UserTaskStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            UserTaskStatus::Created => write!(f, "Created"),
            UserTaskStatus::Running => write!(f, "Running"),
            UserTaskStatus::Suspended => write!(f, "Suspended"),
            UserTaskStatus::Exited => write!(f, "Exited"),
            UserTaskStatus::Crashed => write!(f, "Crashed"),
        }
    }
}

/// Loader result
#[derive(Debug, Clone)]
pub struct LoaderResult {
    /// Whether loading was successful
    pub success: bool,
    
    /// Process ID if successful
    pub pid: Option<u64>,
    
    /// Error message if loading failed
    pub error: Option<String>,
    
    /// Validation warnings
    pub warnings: Vec<String>,
}

//=============================================================================
// USER TASK LOADER IMPLEMENTATION
//=============================================================================

/// User Task Loader
/// 
/// Manages the loading and validation of user task images, including:
/// - Header validation and parsing
/// - Capability verification
/// - Memory allocation and mapping
/// - Syscall gate setup
/// - User task context creation
pub struct UserTaskLoader {
    /// Loaded user tasks
    user_tasks: Mutex<Vec<UserTaskContext>>,
    
    /// Next process ID
    next_pid: AtomicU64,
    
    /// Memory manager for user space
    memory_manager: Option<VirtualMemoryManager>,
}

impl UserTaskLoader {
    /// Create a new user task loader
    pub fn new() -> Self {
        Self {
            user_tasks: Mutex::new(Vec::new()),
            next_pid: AtomicU64::new(1000), // Start PIDs at 1000
            memory_manager: None,
        }
    }
    
    /// Load a user task image
    /// 
    /// # Arguments
    /// * `image_data` - Binary image data
    /// * `capabilities` - Available capabilities for the task
    /// 
    /// # Returns
    /// `LoaderResult` with loading result
    pub fn load_user_task(
        &mut self,
        image_data: &[u8],
        capabilities: &[CapTokenV2],
    ) -> LoaderResult {
        klog!(INFO, "[LOADER] Loading user task image ({} bytes)", image_data.len());
        
        // Parse and validate header
        let header_result = UserTaskHeader::parse(image_data);
        if !header_result.success {
            let error_msg = header_result.error.unwrap_or_else(|| "Unknown parsing error".to_string());
            klog!(ERROR, "[LOADER] Header parsing failed: {}", error_msg);
            
            // Audit the failure
            self.audit_exec_verify_fail(&error_msg);
            
            return LoaderResult {
                success: false,
                pid: None,
                error: Some(error_msg),
                warnings: header_result.warnings,
            };
        }
        
        let header = header_result.header.unwrap();
        
        // Validate header integrity
        if let Err(validation_error) = header.validate() {
            klog!(ERROR, "[LOADER] Header validation failed: {}", validation_error);
            
            // Audit the failure
            self.audit_exec_verify_fail(&validation_error);
            
            return LoaderResult {
                success: false,
                pid: None,
                error: Some(validation_error),
                warnings: header_result.warnings,
            };
        }
        
        // Verify required capabilities
        let cap_verification = self.verify_required_capabilities(&header, capabilities);
        if !cap_verification.success {
            let cap_error = cap_verification
                .error
                .clone()
                .unwrap_or_else(|| "Unknown capability verification error".to_string());
            klog!(ERROR, "[LOADER] Capability verification failed: {}", 
                  cap_error);
            
            // Audit the failure
            self.audit_exec_verify_fail(&cap_error);
            
            return LoaderResult {
                success: false,
                pid: None,
                error: Some(cap_error),
                warnings: header_result.warnings,
            };
        }
        
        // Create user task context
        let mut task_context = self.create_user_task_context(&header, capabilities);
        
        // Allocate memory for the task
        if let Err(memory_error) = self.allocate_user_memory(&mut task_context) {
            klog!(ERROR, "[LOADER] Memory allocation failed: {}", memory_error);
            
            return LoaderResult {
                success: false,
                pid: None,
                error: Some(memory_error),
                warnings: header_result.warnings,
            };
        }
        
        // Set up syscall gate
        if let Err(gate_error) = self.setup_syscall_gate(&mut task_context) {
            klog!(ERROR, "[LOADER] Syscall gate setup failed: {}", gate_error);
            
            return LoaderResult {
                success: false,
                pid: None,
                error: Some(gate_error),
                warnings: header_result.warnings,
            };
        }
        
        // Register the task
        let pid = task_context.pid;
        {
            let mut tasks = self.user_tasks.lock();
            tasks.push(task_context);
        }
        
        klog!(INFO, "[LOADER] User task loaded successfully with PID {}", pid);
        
        LoaderResult {
            success: true,
            pid: Some(pid),
            error: None,
            warnings: header_result.warnings,
        }
    }
    
    /// Verify required capabilities
    /// 
    /// # Arguments
    /// * `header` - Task header with required capabilities
    /// * `available_caps` - Available capabilities
    /// 
    /// # Returns
    /// `CapabilityVerificationResult` with verification result
    fn verify_required_capabilities(
        &self,
        header: &UserTaskHeader,
        available_caps: &[CapTokenV2],
    ) -> CapabilityVerificationResult {
        let mut warnings = Vec::new();
        
        for required_cap in &header.required_caps {
            let mut cap_found = false;
            
            for available_cap in available_caps {
                if available_cap.header.cap_type.to_string() == required_cap.cap_type {
                    // Verify capability permissions
                    if available_cap.header.permissions >= required_cap.permission_level {
                        cap_found = true;
                        break;
                    } else {
                        warnings.push(format!("Capability {} has insufficient permissions", required_cap.cap_type));
                    }
                }
            }
            
            if !cap_found {
                return CapabilityVerificationResult {
                    success: false,
                    error: Some(format!("Required capability not found: {}", required_cap.cap_type)),
                    warnings,
                };
            }
        }
        
        CapabilityVerificationResult {
            success: true,
            error: None,
            warnings,
        }
    }
    
    /// Create user task context
    /// 
    /// # Arguments
    /// * `header` - Task header
    /// * `capabilities` - Available capabilities
    /// 
    /// # Returns
    /// `UserTaskContext` - Created task context
    fn create_user_task_context(
        &self,
        header: &UserTaskHeader,
        capabilities: &[CapTokenV2],
    ) -> UserTaskContext {
        let pid = self.next_pid.fetch_add(1, Ordering::Relaxed);
        
        // Create memory regions from header data regions
        let mut memory_regions = Vec::new();
        for data_region in &header.data_regions {
            let memory_region = UserMemoryRegion {
                name: data_region.name.clone(),
                start_address: data_region.start_address,
                size: data_region.size,
                protection: self.convert_memory_protection(&data_region.protection),
                mapped: false,
            };
            memory_regions.push(memory_region);
        }
        
        // Create stack info
        let stack_info = UserStackInfo {
            base_address: USER_MEMORY_BASE + (pid * 0x10000), // 64KB per task
            size: header.stack_size,
            current_sp: USER_MEMORY_BASE + (pid * 0x10000) + header.stack_size as u64,
            protection_enabled: true,
        };
        
        // Convert capabilities
        let capability_tokens = capabilities.to_vec();
        
        UserTaskContext {
            pid,
            header: header.clone(),
            memory_regions,
            stack_info,
            capability_tokens,
            syscall_gate: 0, // Will be set up later
            entry_point: header.entry_point,
            status: UserTaskStatus::Created,
            created_at: self.get_timestamp(),
        }
    }
    
    /// Allocate user memory
    /// 
    /// # Arguments
    /// * `task_context` - Task context to allocate memory for
    /// 
    /// # Returns
    /// `Result<(), String>` - Success or error
    fn allocate_user_memory(&self, task_context: &mut UserTaskContext) -> Result<(), String> {
        // This is a stub implementation
        // In a real implementation, this would:
        // 1. Allocate physical pages
        // 2. Map them to user virtual addresses
        // 3. Set appropriate permissions
        // 4. Initialize stack and data regions
        
        klog!(INFO, "[LOADER] Allocating {} bytes for user task {}", 
              task_context.header.stack_size, task_context.pid);
        
        // Mark memory regions as mapped
        for region in &mut task_context.memory_regions {
            region.mapped = true;
        }
        
        Ok(())
    }
    
    /// Set up syscall gate
    /// 
    /// # Arguments
    /// * `task_context` - Task context to set up syscall gate for
    /// 
    /// # Returns
    /// `Result<(), String>` - Success or error
    fn setup_syscall_gate(&self, task_context: &mut UserTaskContext) -> Result<(), String> {
        // This is a stub implementation
        // In a real implementation, this would:
        // 1. Set up the syscall instruction at the gate address
        // 2. Configure the gate to call the kernel syscall handler
        // 3. Set appropriate permissions for the gate
        
        let gate_address = USER_MEMORY_BASE + (task_context.pid * 0x10000) + 0x8000;
        task_context.syscall_gate = gate_address;
        
        klog!(INFO, "[LOADER] Syscall gate set up at 0x{:016x} for task {}", 
              gate_address, task_context.pid);
        
        Ok(())
    }
    
    /// Convert memory protection flags
    /// 
    /// # Arguments
    /// * `protection` - Memory protection flags
    /// 
    /// # Returns
    /// `MemoryFlags` - Converted flags
    fn convert_memory_protection(&self, protection: &super::header::MemoryProtection) -> MemoryFlags {
        MemoryFlags {
            readable: protection.read,
            writable: protection.write,
            executable: protection.execute,
            user_accessible: protection.shared,
            cached: true,
        }
    }
    
    /// Get user task by PID
    /// 
    /// # Arguments
    /// * `pid` - Process ID
    /// 
    /// # Returns
    /// `Option<UserTaskContext>` - Task context if found
    pub fn get_user_task(&self, pid: u64) -> Option<UserTaskContext> {
        let tasks = self.user_tasks.lock();
        tasks.iter().find(|task| task.pid == pid).cloned()
    }
    
    /// List all user tasks
    /// 
    /// # Returns
    /// `Vec<UserTaskContext>` - List of all user tasks
    pub fn list_user_tasks(&self) -> Vec<UserTaskContext> {
        let tasks = self.user_tasks.lock();
        tasks.clone()
    }
    
    /// Update task status
    /// 
    /// # Arguments
    /// * `pid` - Process ID
    /// * `status` - New status
    /// 
    /// # Returns
    /// `Result<(), String>` - Success or error
    pub fn update_task_status(&self, pid: u64, status: UserTaskStatus) -> Result<(), String> {
        let mut tasks = self.user_tasks.lock();
        
        if let Some(task) = tasks.iter_mut().find(|t| t.pid == pid) {
            task.status = status;
            klog!(INFO, "[LOADER] Task {} status updated to {}", pid, status);
            Ok(())
        } else {
            Err(format!("Task with PID {} not found", pid))
        }
    }
    
    /// Remove user task
    /// 
    /// # Arguments
    /// * `pid` - Process ID
    /// 
    /// # Returns
    /// `Result<(), String>` - Success or error
    pub fn remove_user_task(&self, pid: u64) -> Result<(), String> {
        let mut tasks = self.user_tasks.lock();
        
        if let Some(index) = tasks.iter().position(|t| t.pid == pid) {
            tasks.remove(index);
            klog!(INFO, "[LOADER] Task {} removed", pid);
            Ok(())
        } else {
            Err(format!("Task with PID {} not found", pid))
        }
    }
    
    /// Audit execution verification failure
    /// 
    /// # Arguments
    /// * `reason` - Failure reason
    fn audit_exec_verify_fail(&self, reason: &str) {
        // Create audit entry for execution verification failure
        let audit_entry = AuditEntry::new(0, ops::SEC_AUTH_FAIL, reason.len() as u64);
        klog!(WARN, "[AUDIT] EXEC_VERIFY_FAIL: {}", audit_entry);
        
        // TODO: Send audit entry to audit system
    }
    
    /// Get current timestamp
    fn get_timestamp(&self) -> u64 {
        // TODO: Integrate with actual time system
        core::sync::atomic::AtomicU64::new(0).fetch_add(1, Ordering::Relaxed)
    }
}

/// Capability verification result
#[derive(Debug, Clone)]
struct CapabilityVerificationResult {
    /// Whether verification was successful
    success: bool,
    
    /// Error message if verification failed
    error: Option<String>,
    
    /// Verification warnings
    warnings: Vec<String>,
}

//=============================================================================
// GLOBAL USER TASK LOADER
//=============================================================================

/// Global user task loader instance
lazy_static! {
    static ref USER_TASK_LOADER: Mutex<UserTaskLoader> = Mutex::new(UserTaskLoader::new());
}

/// Initialize user task loader
pub fn init_user_task_loader() {
    kprintln!("[LOADER] Initializing user task loader");
    
    let loader = UserTaskLoader::new();
    *USER_TASK_LOADER.lock() = loader;
    
    kprintln!("[LOADER] User task loader initialized");
}

/// Get the global user task loader
pub fn get_user_task_loader() -> &'static Mutex<UserTaskLoader> {
    &USER_TASK_LOADER
}

/// Load a user task image
/// 
/// # Arguments
/// * `image_data` - Binary image data
/// * `capabilities` - Available capabilities for the task
/// 
/// # Returns
/// `LoaderResult` with loading result
pub fn load_user_task(
    image_data: &[u8],
    capabilities: &[CapTokenV2],
) -> LoaderResult {
    let mut loader = get_user_task_loader().lock();
    loader.load_user_task(image_data, capabilities)
}

/// Create a user task (stub implementation)
/// 
/// # Arguments
/// * `image` - Image data
/// * `caps` - Capabilities
/// 
/// # Returns
/// `Result<u64, String>` - Process ID if successful
pub fn create_user_task(image: &[u8], caps: &[CapTokenV2]) -> Result<u64, String> {
    let result = load_user_task(image, caps);
    
    if result.success {
        Ok(result.pid.unwrap())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown loading error".to_string()))
    }
}

/// Get user task by PID
/// 
/// # Arguments
/// * `pid` - Process ID
/// 
/// # Returns
/// `Option<UserTaskContext>` - Task context if found
pub fn get_user_task(pid: u64) -> Option<UserTaskContext> {
    let loader = get_user_task_loader();
    loader.lock().get_user_task(pid)
}

/// List all user tasks
/// 
/// # Returns
/// `Vec<UserTaskContext>` - List of all user tasks
    pub fn list_user_tasks() -> Vec<UserTaskContext> {
        let loader = get_user_task_loader();
        loader.lock().list_user_tasks()
    }

/// Update task status
/// 
/// # Arguments
/// * `pid` - Process ID
/// * `status` - New status
/// 
/// # Returns
/// `Result<(), String>` - Success or error
pub fn update_task_status(pid: u64, status: UserTaskStatus) -> Result<(), String> {
    let loader = get_user_task_loader();
    loader.lock().update_task_status(pid, status)
}

/// Remove user task
/// 
/// # Arguments
/// * `pid` - Process ID
/// 
/// # Returns
/// `Result<(), String>` - Success or error
pub fn remove_user_task(pid: u64) -> Result<(), String> {
    let loader = get_user_task_loader();
    loader.lock().remove_user_task(pid)
}

//=============================================================================
// TESTING AND DEMONSTRATION
//=============================================================================

/// Test user task loading functionality
#[allow(dead_code)]
pub fn test_user_task_loading() {
    kprintln!("[LOADER] Testing user task loading functionality...");
    
    // Create test header
    let test_header = create_test_header();
    let image_data = test_header.serialize();
    
    // Create test capabilities
    let test_caps = Vec::new(); // Empty for now
    
    // Test loading
    let load_result = load_user_task(&image_data, &test_caps);
    if load_result.success {
        if let Some(pid) = load_result.pid {
            kprintln!("[LOADER] Successfully loaded user task with PID {}", pid);
            
            // Test getting the task
            if let Some(task) = get_user_task(pid) {
                kprintln!("[LOADER] Retrieved task: PID {}, Status: {}", task.pid, task.status);
                
                // Test status update
                if let Err(e) = update_task_status(pid, UserTaskStatus::Running) {
                    kprintln!("[LOADER] Failed to update task status: {}", e);
                } else {
                    kprintln!("[LOADER] Task status updated to Running");
                }
                
                // Test removal
                if let Err(e) = remove_user_task(pid) {
                    kprintln!("[LOADER] Failed to remove task: {}", e);
                } else {
                    kprintln!("[LOADER] Task removed successfully");
                }
            } else {
                kprintln!("[LOADER] Failed to retrieve loaded task");
            }
        } else {
            kprintln!("[LOADER] Load reported success without a PID");
        }
    } else {
        kprintln!(
            "[LOADER] Failed to load user task: {}",
            load_result.error.unwrap_or_else(|| "unknown error".to_string())
        );
    }
    
    // Test invalid header
    let invalid_header = create_invalid_header();
    let invalid_image_data = invalid_header.serialize();
    
    let invalid_result = load_user_task(&invalid_image_data, &test_caps);
    if invalid_result.success {
        if let Some(pid) = invalid_result.pid {
            kprintln!("[LOADER] Unexpectedly loaded invalid task with PID {}", pid);
        } else {
            kprintln!("[LOADER] Invalid task reported success without a PID");
        }
    } else {
        kprintln!(
            "[LOADER] Correctly rejected invalid task: {}",
            invalid_result.error.unwrap_or_else(|| "unknown error".to_string())
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_loader_creation() {
        let loader = UserTaskLoader::new();
        assert_eq!(loader.next_pid.load(Ordering::Relaxed), 1000);
        assert_eq!(loader.user_tasks.lock().len(), 0);
    }
    
    #[test]
    fn test_task_context_creation() {
        let loader = UserTaskLoader::new();
        let header = create_test_header();
        let caps = Vec::new();
        
        let context = loader.create_user_task_context(&header, &caps);
        
        assert_eq!(context.pid, 1000);
        assert_eq!(context.header.entry_point, header.entry_point);
        assert_eq!(context.status, UserTaskStatus::Created);
        assert_eq!(context.capability_tokens.len(), 0);
    }
    
    #[test]
    fn test_memory_protection_conversion() {
        let loader = UserTaskLoader::new();
        let protection = super::super::header::MemoryProtection {
            read: true,
            write: false,
            execute: true,
            shared: false,
        };
        
        let flags = loader.convert_memory_protection(&protection);
        assert!(flags.contains(MemoryFlags::READ));
        assert!(!flags.contains(MemoryFlags::WRITE));
        assert!(flags.contains(MemoryFlags::EXECUTE));
        assert!(!flags.contains(MemoryFlags::SHARED));
    }
    
    #[test]
    fn test_task_status_display() {
        let status = UserTaskStatus::Running;
        assert_eq!(format!("{}", status), "Running");
        
        let status = UserTaskStatus::Exited;
        assert_eq!(format!("{}", status), "Exited");
    }
}
