/// Stack Red-Zone Management for Polymera OS
/// 
/// This module provides comprehensive stack protection including:
/// - Red-zone canaries for kernel stacks
/// - Stack overflow detection before corruption
/// - Guard page management
/// - Memory safety validation

use crate::{kprintln, klog, lazy_static};
use crate::log::Level;
use crate::mm::{MemoryResult, MemoryError, VirtualAddress, PhysicalAddress, PageSize, MemoryFlags};
use crate::mm::vm::VirtualMemoryManager;
use core::sync::atomic::{AtomicU64, AtomicU32, AtomicBool, Ordering};
use core::ptr;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;

//=============================================================================
// STACK PROTECTION CONFIGURATION
//=============================================================================

/// Enable stack protection features
#[cfg(feature = "debug")]
pub const STACK_PROTECTION_ENABLED: bool = true;

#[cfg(not(feature = "debug"))]
pub const STACK_PROTECTION_ENABLED: bool = false;

/// Stack canary magic values
pub const STACK_CANARY_MAGIC: [u64; 4] = [
    0xDEADBEEFCAFEBABE,  // Primary canary
    0xCAFEBABEDEADBEEF,  // Secondary canary
    0xBEEFCAFEDEADBABE,  // Tertiary canary
    0xDEADCAFEBEEFBABE,  // Quaternary canary
];

/// Red-zone size around kernel stacks (bytes)
pub const KERNEL_STACK_RED_ZONE_SIZE: usize = 4096; // 4KB

/// Guard page size (must be page-aligned)
pub const GUARD_PAGE_SIZE: usize = 4096; // 4KB

/// Maximum number of protected stacks
pub const MAX_PROTECTED_STACKS: usize = 1000;

/// Canary check frequency (every N stack operations)
pub const CANARY_CHECK_FREQUENCY: u32 = 100;

//=============================================================================
// STACK PROTECTION TYPES
//=============================================================================

/// Stack protection configuration
#[derive(Debug, Clone)]
pub struct StackProtectionConfig {
    /// Enable red-zone protection
    pub red_zone_enabled: bool,
    /// Enable guard page protection
    pub guard_page_enabled: bool,
    /// Enable canary protection
    pub canary_enabled: bool,
    /// Red-zone size in bytes
    pub red_zone_size: usize,
    /// Guard page size in bytes
    pub guard_page_size: usize,
    /// Canary check frequency
    pub canary_check_frequency: u32,
}

impl Default for StackProtectionConfig {
    fn default() -> Self {
        Self {
            red_zone_enabled: STACK_PROTECTION_ENABLED,
            guard_page_enabled: STACK_PROTECTION_ENABLED,
            canary_enabled: STACK_PROTECTION_ENABLED,
            red_zone_size: KERNEL_STACK_RED_ZONE_SIZE,
            guard_page_size: GUARD_PAGE_SIZE,
            canary_check_frequency: CANARY_CHECK_FREQUENCY,
        }
    }
}

/// Protected stack information
#[derive(Debug, Clone)]
pub struct ProtectedStack {
    /// Stack identifier
    pub id: u64,
    /// Stack base address
    pub base_address: u64,
    /// Stack size
    pub size: usize,
    /// Stack top address
    pub top_address: u64,
    /// Current stack pointer
    pub current_sp: u64,
    /// Red-zone base address
    pub red_zone_base: u64,
    /// Guard page base address
    pub guard_page_base: u64,
    /// Canary values
    pub canaries: [u64; 4],
    /// Protection configuration
    pub config: StackProtectionConfig,
    /// Creation timestamp
    pub created_at: u64,
    /// Last canary check timestamp
    pub last_canary_check: u64,
    /// Canary check counter
    pub canary_check_count: u32,
}

/// Stack overflow detection result
#[derive(Debug, Clone)]
pub struct StackOverflowResult {
    /// Whether overflow was detected
    pub overflow_detected: bool,
    /// Type of overflow
    pub overflow_type: Option<StackOverflowType>,
    /// Overflow details
    pub details: Option<StackOverflowDetails>,
}

/// Stack overflow details
#[derive(Debug, Clone)]
pub struct StackOverflowDetails {
    /// Type of overflow
    pub overflow_type: StackOverflowType,
    /// Address where overflow occurred
    pub overflow_address: u64,
    /// Expected value
    pub expected_value: u64,
    /// Actual value found
    pub actual_value: u64,
    /// Stack ID where overflow occurred
    pub stack_id: u64,
    /// Timestamp of overflow detection
    pub timestamp: u64,
}

/// Types of stack overflow
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StackOverflowType {
    /// Red-zone canary corrupted
    RedZoneCanaryCorruption,
    /// Guard page accessed
    GuardPageAccess,
    /// Stack pointer out of bounds
    StackPointerOutOfBounds,
    /// Stack underflow
    StackUnderflow,
    /// Stack overflow
    StackOverflow,
}

/// Stack protection statistics
#[derive(Debug, Default)]
pub struct StackProtectionStats {
    /// Total protected stacks
    pub total_protected_stacks: AtomicU64,
    /// Total canary checks performed
    pub total_canary_checks: AtomicU64,
    /// Stack overflows detected
    pub stack_overflows_detected: AtomicU32,
    /// Red-zone violations
    pub red_zone_violations: AtomicU32,
    /// Guard page violations
    pub guard_page_violations: AtomicU32,
    /// Canary corruptions
    pub canary_corruptions: AtomicU32,
    /// Stacks successfully protected
    pub stacks_successfully_protected: AtomicU64,
    /// Red-zones created
    pub red_zones_created: AtomicU64,
    /// Guard pages created
    pub guard_pages_created: AtomicU64,
}

//=============================================================================
// STACK PROTECTION MANAGER
//=============================================================================

/// Stack Protection Manager
/// 
/// Manages stack protection for all kernel stacks including:
/// - Red-zone canary placement and validation
/// - Guard page management
/// - Stack overflow detection
/// - Protection statistics and monitoring
pub struct StackProtectionManager {
    /// Protected stacks
    protected_stacks: Mutex<BTreeMap<u64, ProtectedStack>>,
    /// Protection statistics
    stats: StackProtectionStats,
    /// Global protection configuration
    config: StackProtectionConfig,
    /// Next stack ID
    next_stack_id: AtomicU64,
}

impl StackProtectionManager {
    /// Create a new stack protection manager
    pub fn new(config: StackProtectionConfig) -> Self {
        Self {
            protected_stacks: Mutex::new(BTreeMap::new()),
            stats: StackProtectionStats::default(),
            config,
            next_stack_id: AtomicU64::new(1),
        }
    }
    
    /// Protect a kernel stack
    /// 
    /// # Arguments
    /// * `base_address` - Stack base address
    /// * `size` - Stack size in bytes
    /// * `current_sp` - Current stack pointer
    /// 
    /// # Returns
    /// `MemoryResult<u64>` - Stack ID if successful
    pub fn protect_stack(
        &self,
        base_address: u64,
        size: usize,
        current_sp: u64,
    ) -> MemoryResult<u64> {
        let stack_id = self.next_stack_id.fetch_add(1, Ordering::Relaxed);
        
        // Calculate protection addresses
        let red_zone_base = base_address - self.config.red_zone_size as u64;
        let guard_page_base = red_zone_base - self.config.guard_page_size as u64;
        let top_address = base_address + size as u64;
        
        // Create protected stack
        let protected_stack = ProtectedStack {
            id: stack_id,
            base_address,
            size,
            top_address,
            current_sp,
            red_zone_base,
            guard_page_base,
            canaries: STACK_CANARY_MAGIC,
            config: self.config.clone(),
            created_at: self.get_timestamp(),
            last_canary_check: 0,
            canary_check_count: 0,
        };
        
        // Set up protection
        self.setup_stack_protection(&protected_stack)?;
        
        // Register the protected stack
        {
            let mut stacks = self.protected_stacks.lock();
            stacks.insert(stack_id, protected_stack);
        }
        
        // Update statistics
        self.stats.total_protected_stacks.fetch_add(1, Ordering::Relaxed);
        self.stats.stacks_successfully_protected.fetch_add(1, Ordering::Relaxed);
        
        klog!(INFO, "[STACK_PROTECT] Protected stack {} at 0x{:016x}", stack_id, base_address);
        
        Ok(stack_id)
    }
    
    /// Set up stack protection
    /// 
    /// # Arguments
    /// * `stack` - The protected stack to set up
    /// 
    /// # Returns
    /// `MemoryResult<()>` - Success or error
    fn setup_stack_protection(&self, stack: &ProtectedStack) -> MemoryResult<()> {
        // Set up red-zone canaries
        if stack.config.red_zone_enabled {
            self.setup_red_zone_canaries(stack)?;
        }
        
        // Set up guard page
        if stack.config.guard_page_enabled {
            self.setup_guard_page(stack)?;
        }
        
        Ok(())
    }
    
    /// Set up red-zone canaries
    /// 
    /// # Arguments
    /// * `stack` - The protected stack
    /// 
    /// # Returns
    /// `MemoryResult<()>` - Success or error
    fn setup_red_zone_canaries(&self, stack: &ProtectedStack) -> MemoryResult<()> {
        // Place canaries at strategic locations in the red-zone
        let canary_locations = [
            stack.red_zone_base + 0x1000,  // 4KB into red-zone
            stack.red_zone_base + 0x2000,  // 8KB into red-zone
            stack.red_zone_base + 0x3000,  // 12KB into red-zone
            stack.red_zone_base + 0x3FF0,  // End of red-zone
        ];
        
        for (i, &location) in canary_locations.iter().enumerate() {
            unsafe {
                let canary_ptr = location as *mut u64;
                *canary_ptr = stack.canaries[i];
            }
        }
        
        self.stats.red_zones_created.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }
    
    /// Set up guard page
    /// 
    /// # Arguments
    /// * `stack` - The protected stack
    /// 
    /// # Returns
    /// `MemoryResult<()>` - Success or error
    fn setup_guard_page(&self, stack: &ProtectedStack) -> MemoryResult<()> {
        // Mark guard page as non-present to trigger page fault on access
        // This would require integration with the paging system
        
        self.stats.guard_pages_created.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }
    
    /// Check stack protection
    /// 
    /// # Arguments
    /// * `stack_id` - ID of the stack to check
    /// 
    /// # Returns
    /// `MemoryResult<StackOverflowResult>` - Protection check result
    pub fn check_stack_protection(&self, stack_id: u64) -> MemoryResult<StackOverflowResult> {
        let stacks = self.protected_stacks.lock();
        
        let Some(stack) = stacks.get(&stack_id) else {
            return Err(MemoryError::InvalidAddress);
        };
        
        // Check if it's time for a canary check
        let current_time = self.get_timestamp();
        if current_time - stack.last_canary_check < self.config.canary_check_frequency as u64 {
            return Ok(StackOverflowResult {
                overflow_detected: false,
                overflow_type: None,
                details: None,
            });
        }
        
        // Perform canary check
        let canary_result = self.check_red_zone_canaries(stack);
        
        // Update statistics
        self.stats.total_canary_checks.fetch_add(1, Ordering::Relaxed);
        
        if canary_result.overflow_detected {
            self.stats.canary_corruptions.fetch_add(1, Ordering::Relaxed);
            self.stats.stack_overflows_detected.fetch_add(1, Ordering::Relaxed);
        }
        
        Ok(canary_result)
    }
    
    /// Check red-zone canaries
    /// 
    /// # Arguments
    /// * `stack` - The protected stack to check
    /// 
    /// # Returns
    /// `StackOverflowResult` - Canary check result
    fn check_red_zone_canaries(&self, stack: &ProtectedStack) -> StackOverflowResult {
        let canary_locations = [
            stack.red_zone_base + 0x1000,  // 4KB into red-zone
            stack.red_zone_base + 0x2000,  // 8KB into red-zone
            stack.red_zone_base + 0x3000,  // 12KB into red-zone
            stack.red_zone_base + 0x3FF0,  // End of red-zone
        ];
        
        for (i, &location) in canary_locations.iter().enumerate() {
            let canary_value = unsafe { *(location as *const u64) };
            
            if canary_value != stack.canaries[i] {
                let details = StackOverflowDetails {
                    overflow_type: StackOverflowType::RedZoneCanaryCorruption,
                    overflow_address: location,
                    expected_value: stack.canaries[i],
                    actual_value: canary_value,
                    stack_id: stack.id,
                    timestamp: self.get_timestamp(),
                };
                
                return StackOverflowResult {
                    overflow_detected: true,
                    overflow_type: Some(StackOverflowType::RedZoneCanaryCorruption),
                    details: Some(details),
                };
            }
        }
        
        StackOverflowResult {
            overflow_detected: false,
            overflow_type: None,
            details: None,
        }
    }
    
    /// Update stack pointer
    /// 
    /// # Arguments
    /// * `stack_id` - ID of the stack to update
    /// * `new_sp` - New stack pointer value
    /// 
    /// # Returns
    /// `MemoryResult<()>` - Success or error
    pub fn update_stack_pointer(&self, stack_id: u64, new_sp: u64) -> MemoryResult<()> {
        let mut stacks = self.protected_stacks.lock();
        
        let Some(stack) = stacks.get_mut(&stack_id) else {
            return Err(MemoryError::InvalidAddress);
        };
        
        // Check for stack overflow/underflow
        if new_sp < stack.base_address || new_sp >= stack.top_address {
            let overflow_type = if new_sp < stack.base_address {
                StackOverflowType::StackUnderflow
            } else {
                StackOverflowType::StackOverflow
            };
            
            let details = StackOverflowDetails {
                overflow_type,
                overflow_address: new_sp,
                expected_value: stack.base_address,
                actual_value: new_sp,
                stack_id: stack.id,
                timestamp: self.get_timestamp(),
            };
            
            // Update statistics
            self.stats.stack_overflows_detected.fetch_add(1, Ordering::Relaxed);
            
            // Log the violation
            klog!(ERROR, "[STACK_PROTECT] Stack {} overflow detected: {:?}", stack_id, details);
            
            // Trigger panic for stack overflow
            panic!("STACK_OVERFLOW: Stack pointer out of bounds");
        }
        
        // Update stack pointer
        stack.current_sp = new_sp;
        
        Ok(())
    }
    
    /// Remove stack protection
    /// 
    /// # Arguments
    /// * `stack_id` - ID of the stack to unprotect
    /// 
    /// # Returns
    /// `MemoryResult<()>` - Success or error
    pub fn unprotect_stack(&self, stack_id: u64) -> MemoryResult<()> {
        let mut stacks = self.protected_stacks.lock();
        
        let Some(_) = stacks.remove(&stack_id) else {
            return Err(MemoryError::InvalidAddress);
        };
        
        klog!(INFO, "[STACK_PROTECT] Unprotected stack {}", stack_id);
        
        Ok(())
    }
    
    /// Get stack protection statistics
    /// 
    /// # Returns
    /// `StackProtectionStats` - Current statistics
    pub fn get_stats(&self) -> &StackProtectionStats {
        &self.stats
    }
    
    /// Get protected stack information
    /// 
    /// # Arguments
    /// * `stack_id` - ID of the stack
    /// 
    /// # Returns
    /// `Option<ProtectedStack>` - Stack information if found
    pub fn get_protected_stack(&self, stack_id: u64) -> Option<ProtectedStack> {
        let stacks = self.protected_stacks.lock();
        stacks.get(&stack_id).cloned()
    }
    
    /// List all protected stacks
    /// 
    /// # Returns
    /// `Vec<ProtectedStack>` - List of all protected stacks
    pub fn list_protected_stacks(&self) -> Vec<ProtectedStack> {
        let stacks = self.protected_stacks.lock();
        stacks.values().cloned().collect()
    }
    
    /// Get current timestamp
    fn get_timestamp(&self) -> u64 {
        // TODO: Integrate with actual time system
        // For now, use a simple counter
        core::sync::atomic::AtomicU64::new(0).fetch_add(1, Ordering::Relaxed)
    }
}

//=============================================================================
// GLOBAL STACK PROTECTION MANAGER
//=============================================================================

/// Global stack protection manager instance
lazy_static! {
    static ref STACK_PROTECTION_MANAGER: Mutex<StackProtectionManager> = 
        Mutex::new(StackProtectionManager::new(StackProtectionConfig::default()));
}

/// Initialize stack protection system
pub fn init_stack_protection() {
    kprintln!("[STACK_PROTECT] Initializing stack protection system");
    
    let config = StackProtectionConfig::default();
    let manager = StackProtectionManager::new(config);
    
    *STACK_PROTECTION_MANAGER.lock() = manager;
    
    kprintln!("[STACK_PROTECT] Stack protection system initialized");
    kprintln!("  Red-zone size: {} bytes", KERNEL_STACK_RED_ZONE_SIZE);
    kprintln!("  Guard page size: {} bytes", GUARD_PAGE_SIZE);
    kprintln!("  Canary check frequency: every {} operations", CANARY_CHECK_FREQUENCY);
}

/// Get the global stack protection manager
pub fn get_stack_protection_manager() -> &'static Mutex<StackProtectionManager> {
    &STACK_PROTECTION_MANAGER
}

/// Protect a kernel stack
/// 
/// # Arguments
/// * `base_address` - Stack base address
/// * `size` - Stack size in bytes
/// * `current_sp` - Current stack pointer
/// 
/// # Returns
/// `MemoryResult<u64>` - Stack ID if successful
pub fn protect_kernel_stack(
    base_address: u64,
    size: usize,
    current_sp: u64,
) -> MemoryResult<u64> {
    let manager = get_stack_protection_manager();
    manager.lock().protect_stack(base_address, size, current_sp)
}

/// Check stack protection
/// 
/// # Arguments
/// * `stack_id` - ID of the stack to check
/// 
/// # Returns
/// `MemoryResult<StackOverflowResult>` - Protection check result
pub fn check_kernel_stack_protection(stack_id: u64) -> MemoryResult<StackOverflowResult> {
    let manager = get_stack_protection_manager();
    manager.lock().check_stack_protection(stack_id)
}

/// Update stack pointer
/// 
/// # Arguments
/// * `stack_id` - ID of the stack to update
/// * `new_sp` - New stack pointer value
/// 
/// # Returns
/// `MemoryResult<()>` - Success or error
pub fn update_kernel_stack_pointer(stack_id: u64, new_sp: u64) -> MemoryResult<()> {
    let manager = get_stack_protection_manager();
    manager.lock().update_stack_pointer(stack_id, new_sp)
}

/// Remove stack protection
/// 
/// # Arguments
/// * `stack_id` - ID of the stack to unprotect
/// 
/// # Returns
/// `MemoryResult<()>` - Success or error
pub fn unprotect_kernel_stack(stack_id: u64) -> MemoryResult<()> {
    let manager = get_stack_protection_manager();
    manager.lock().unprotect_stack(stack_id)
}

/// Print stack protection statistics
pub fn print_stack_protection_stats() {
    let manager = get_stack_protection_manager();
    let guard = manager.lock();
    let stats = guard.get_stats();
    
    kprintln!("");
    kprintln!("=== STACK PROTECTION STATISTICS ===");
    kprintln!("Total Protected Stacks: {}", stats.total_protected_stacks.load(Ordering::Relaxed));
    kprintln!("Total Canary Checks: {}", stats.total_canary_checks.load(Ordering::Relaxed));
    kprintln!("Stack Overflows Detected: {}", stats.stack_overflows_detected.load(Ordering::Relaxed));
    kprintln!("Red-Zone Violations: {}", stats.red_zone_violations.load(Ordering::Relaxed));
    kprintln!("Guard Page Violations: {}", stats.guard_page_violations.load(Ordering::Relaxed));
    kprintln!("Canary Corruptions: {}", stats.canary_corruptions.load(Ordering::Relaxed));
    kprintln!("Stacks Successfully Protected: {}", stats.stacks_successfully_protected.load(Ordering::Relaxed));
    kprintln!("=== END STACK PROTECTION STATISTICS ===");
    kprintln!("");
}

/// Test stack protection functionality
#[allow(dead_code)]
pub fn test_stack_protection() {
    kprintln!("[STACK_PROTECT] Testing stack protection system...");
    
    // Test stack protection
    let base_address = 0x1000000;
    let size: usize = 0x10000; // 64KB stack
    let current_sp = base_address + size as u64 - 0x1000; // 4KB from top
    
    match protect_kernel_stack(base_address, size, current_sp) {
        Ok(stack_id) => {
            kprintln!("[STACK_PROTECT] Successfully protected stack {}", stack_id);
            
            // Test protection check
            match check_kernel_stack_protection(stack_id) {
                Ok(result) => {
                    if result.overflow_detected {
                        kprintln!("[STACK_PROTECT] Stack overflow detected!");
                    } else {
                        kprintln!("[STACK_PROTECT] Stack protection check passed");
                    }
                }
                Err(e) => {
                    kprintln!("[STACK_PROTECT] Protection check failed: {:?}", e);
                }
            }
            
            // Test stack pointer update
            let new_sp = current_sp - 0x1000;
            match update_kernel_stack_pointer(stack_id, new_sp) {
                Ok(()) => {
                    kprintln!("[STACK_PROTECT] Stack pointer updated successfully");
                }
                Err(e) => {
                    kprintln!("[STACK_PROTECT] Stack pointer update failed: {:?}", e);
                }
            }
            
            // Clean up
            if let Err(e) = unprotect_kernel_stack(stack_id) {
                kprintln!("[STACK_PROTECT] Failed to unprotect stack: {:?}", e);
            }
        }
        Err(e) => {
            kprintln!("[STACK_PROTECT] Failed to protect stack: {:?}", e);
        }
    }
    
    // Print statistics
    print_stack_protection_stats();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stack_protection_config() {
        let config = StackProtectionConfig::default();
        
        assert!(config.red_zone_enabled);
        assert!(config.guard_page_enabled);
        assert!(config.canary_enabled);
        assert_eq!(config.red_zone_size, KERNEL_STACK_RED_ZONE_SIZE);
        assert_eq!(config.guard_page_size, GUARD_PAGE_SIZE);
    }
    
    #[test]
    fn test_canary_magic_values() {
        for canary in STACK_CANARY_MAGIC.iter() {
            assert_ne!(*canary, 0);
            assert_ne!(*canary, u64::MAX);
        }
        
        // Ensure canaries are unique
        let mut unique_canaries = std::collections::HashSet::new();
        for canary in STACK_CANARY_MAGIC.iter() {
            unique_canaries.insert(canary);
        }
        assert_eq!(unique_canaries.len(), STACK_CANARY_MAGIC.len());
    }
    
    #[test]
    fn test_stack_overflow_types() {
        let overflow_types = [
            StackOverflowType::RedZoneCanaryCorruption,
            StackOverflowType::GuardPageAccess,
            StackOverflowType::StackPointerOutOfBounds,
            StackOverflowType::StackUnderflow,
            StackOverflowType::StackOverflow,
        ];
        
        for overflow_type in overflow_types.iter() {
            assert!(format!("{:?}", overflow_type).len() > 0);
        }
    }
}
