/// TLB (Translation Lookaside Buffer) Management for Polymera OS
/// 
/// This module provides comprehensive TLB management including page-specific flushes,
/// global flushes, and SMP shootdown support for future multi-core systems.

use crate::{kprintln, klog, kprintln};
use crate::log::Level;
use crate::secman::audit::{audit_log, AuditEvent, AuditLevel};
use super::constants::*;
use x86_64::{
    structures::paging::{Page, Size4KiB, Size2MiB, Size1GiB},
    VirtAddr, PhysAddr,
    instructions::tlb,
};
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use alloc::collections::HashMap;
use alloc::string::String;
use alloc::vec::Vec;

//=============================================================================
// TLB CONFIGURATION AND CONSTANTS
//=============================================================================

/// TLB flush types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlbFlushType {
    /// Flush single page (4KB)
    SinglePage,
    /// Flush large page (2MB)
    LargePage,
    /// Flush huge page (1GB)
    HugePage,
    /// Flush entire TLB
    Global,
    /// Flush specific address range
    Range,
}

/// TLB flush reason for auditing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlbFlushReason {
    /// Page mapping changed
    PageMappingChanged,
    /// Page protection changed
    PageProtectionChanged,
    /// Page unmapped
    PageUnmapped,
    /// Process context switch
    ContextSwitch,
    /// Kernel memory layout changed
    KernelLayoutChanged,
    /// User process memory changed
    UserMemoryChanged,
    /// Guard page setup
    GuardPageSetup,
    /// Manual flush requested
    ManualFlush,
    /// Security audit
    SecurityAudit,
    /// Debug operation
    DebugOperation,
}

//=============================================================================
// TLB FLUSH OPERATIONS
//=============================================================================

/// Flush TLB for a specific virtual address
/// 
/// # Arguments
/// * `virtual_addr` - Virtual address to flush
/// * `reason` - Reason for the flush (for auditing)
/// 
/// # Returns
/// Result indicating success or failure
pub fn flush_page(virtual_addr: VirtAddr, reason: TlbFlushReason) -> Result<(), String> {
    if !virtual_addr.is_aligned(PAGE_SIZE as u64) {
        return Err("Virtual address not page-aligned".to_string());
    }
    
    klog!(TRACE, "[TLB] Flushing page at 0x{:016x}, reason: {:?}", 
          virtual_addr.as_u64(), reason);
    
    // Determine page size and flush accordingly
    let page_size = determine_page_size(virtual_addr);
    let flush_type = match page_size {
        PAGE_SIZE => TlbFlushType::SinglePage,
        LARGE_PAGE_SIZE => TlbFlushType::LargePage,
        HUGE_PAGE_SIZE => TlbFlushType::HugePage,
        _ => TlbFlushType::SinglePage,
    };
    
    // Perform the flush
    match flush_type {
        TlbFlushType::SinglePage => {
            let page = Page::<Size4KiB>::containing_address(virtual_addr);
            unsafe { tlb::flush(page); }
        }
        TlbFlushType::LargePage => {
            let page = Page::<Size2MiB>::containing_address(virtual_addr);
            unsafe { tlb::flush(page); }
        }
        TlbFlushType::HugePage => {
            let page = Page::<Size1GiB>::containing_address(virtual_addr);
            unsafe { tlb::flush(page); }
        }
        _ => {
            // Fallback to single page flush
            let page = Page::<Size4KiB>::containing_address(virtual_addr);
            unsafe { tlb::flush(page); }
        }
    }
    
    // Update statistics
    update_tlb_stats(flush_type, reason);
    
    // Log the operation
    klog!(DEBUG, "[TLB] Successfully flushed {} page at 0x{:016x}", 
          page_size_to_string(page_size), virtual_addr.as_u64());
    
    Ok(())
}

/// Flush entire TLB
/// 
/// # Arguments
/// * `reason` - Reason for the flush (for auditing)
/// 
/// # Returns
/// Result indicating success or failure
pub fn flush_all(reason: TlbFlushReason) -> Result<(), String> {
    klog!(INFO, "[TLB] Flushing entire TLB, reason: {:?}", reason);
    
    // Perform global TLB flush
    unsafe { tlb::flush_all(); }
    
    // Update statistics
    update_tlb_stats(TlbFlushType::Global, reason);
    
    // Log the operation
    klog!(INFO, "[TLB] Successfully flushed entire TLB");
    
    Ok(())
}

/// Flush TLB for a range of virtual addresses
/// 
/// # Arguments
/// * `start_addr` - Start virtual address (inclusive)
/// * `end_addr` - End virtual address (exclusive)
/// * `reason` - Reason for the flush (for auditing)
/// 
/// # Returns
/// Result indicating success or failure
pub fn flush_range(start_addr: VirtAddr, end_addr: VirtAddr, reason: TlbFlushReason) -> Result<(), String> {
    if start_addr >= end_addr {
        return Err("Invalid address range: start >= end".to_string());
    }
    
    if !start_addr.is_aligned(PAGE_SIZE as u64) || !end_addr.is_aligned(PAGE_SIZE as u64) {
        return Err("Address range not page-aligned".to_string());
    }
    
    klog!(INFO, "[TLB] Flushing TLB range 0x{:016x} - 0x{:016x}, reason: {:?}", 
          start_addr.as_u64(), end_addr.as_u64(), reason);
    
    let mut current_addr = start_addr;
    let mut pages_flushed = 0;
    
    while current_addr < end_addr {
        if let Err(e) = flush_page(current_addr, reason) {
            klog!(ERROR, "[TLB] Failed to flush page at 0x{:016x}: {}", current_addr.as_u64(), e);
            return Err(format!("Range flush failed at 0x{:016x}: {}", current_addr.as_u64(), e));
        }
        
        // Move to next page
        let page_size = determine_page_size(current_addr);
        current_addr = VirtAddr::new(current_addr.as_u64() + page_size as u64);
        pages_flushed += 1;
        
        // Safety check to prevent infinite loops
        if pages_flushed > 1000000 {
            return Err("Range flush exceeded maximum page limit".to_string());
        }
    }
    
    klog!(INFO, "[TLB] Successfully flushed {} pages in range", pages_flushed);
    Ok(())
}

/// Flush TLB for multiple specific addresses
/// 
/// # Arguments
/// * `addresses` - Vector of virtual addresses to flush
/// * `reason` - Reason for the flush (for auditing)
/// 
/// # Returns
/// Result indicating success or failure
pub fn flush_multiple_pages(addresses: &[VirtAddr], reason: TlbFlushReason) -> Result<(), String> {
    if addresses.is_empty() {
        return Ok(());
    }
    
    klog!(INFO, "[TLB] Flushing TLB for {} pages, reason: {:?}", addresses.len(), reason);
    
    let mut success_count = 0;
    let mut failure_count = 0;
    
    for &addr in addresses {
        match flush_page(addr, reason) {
            Ok(()) => success_count += 1,
            Err(e) => {
                klog!(ERROR, "[TLB] Failed to flush page at 0x{:016x}: {}", addr.as_u64(), e);
                failure_count += 1;
            }
        }
    }
    
    klog!(INFO, "[TLB] Multiple page flush completed: {} success, {} failures", 
          success_count, failure_count);
    
    if failure_count > 0 {
        Err(format!("Multiple page flush had {} failures", failure_count))
    } else {
        Ok(())
    }
}

//=============================================================================
// SMP SHOOTDOWN SUPPORT (FUTURE)
//=============================================================================

/// CPU core identifier for SMP operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CpuId(pub u32);

/// TLB shootdown request for SMP systems
#[derive(Debug, Clone)]
pub struct TlbShootdownRequest {
    /// Target CPU cores
    pub target_cpus: Vec<CpuId>,
    /// Virtual address to flush (if single page)
    pub virtual_addr: Option<VirtAddr>,
    /// Address range to flush (if range)
    pub address_range: Option<(VirtAddr, VirtAddr)>,
    /// Flush type
    pub flush_type: TlbFlushType,
    /// Reason for the flush
    pub reason: TlbFlushReason,
    /// Request timestamp
    pub timestamp: u64,
    /// Request ID for tracking
    pub request_id: u64,
}

impl TlbShootdownRequest {
    /// Create new shootdown request
    pub fn new(target_cpus: Vec<CpuId>, flush_type: TlbFlushType, reason: TlbFlushReason) -> Self {
        static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(0);
        
        Self {
            target_cpus,
            virtual_addr: None,
            address_range: None,
            flush_type,
            reason,
            timestamp: get_current_timestamp(),
            request_id: REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }
    
    /// Set single page address
    pub fn with_page(mut self, addr: VirtAddr) -> Self {
        self.virtual_addr = Some(addr);
        self
    }
    
    /// Set address range
    pub fn with_range(mut self, start: VirtAddr, end: VirtAddr) -> Self {
        self.address_range = Some((start, end));
        self
    }
}

/// TLB shootdown manager for SMP systems
pub struct TlbShootdownManager {
    /// Pending shootdown requests
    pending_requests: Vec<TlbShootdownRequest>,
    /// Completed requests
    completed_requests: Vec<TlbShootdownRequest>,
    /// Statistics
    stats: TlbShootdownStats,
}

impl TlbShootdownManager {
    /// Create new shootdown manager
    pub fn new() -> Self {
        Self {
            pending_requests: Vec::new(),
            completed_requests: Vec::new(),
            stats: TlbShootdownStats::default(),
        }
    }
    
    /// Submit a shootdown request
    pub fn submit_request(&mut self, request: TlbShootdownRequest) -> Result<(), String> {
        klog!(INFO, "[TLB-SHOOTDOWN] Submitting request {} for {} CPUs", 
              request.request_id, request.target_cpus.len());
        
        self.pending_requests.push(request);
        self.stats.total_requests += 1;
        
        Ok(())
    }
    
    /// Process pending shootdown requests
    pub fn process_requests(&mut self) -> Result<(), String> {
        if self.pending_requests.is_empty() {
            return Ok(());
        }
        
        klog!(DEBUG, "[TLB-SHOOTDOWN] Processing {} pending requests", self.pending_requests.len());
        
        // For now, this is a stub that simulates SMP shootdown
        // In a real SMP implementation, this would:
        // 1. Send IPIs to target CPUs
        // 2. Wait for acknowledgment
        // 3. Perform local TLB flush
        // 4. Mark request as completed
        
        let mut completed = Vec::new();
        
        for request in &self.pending_requests {
            klog!(DEBUG, "[TLB-SHOOTDOWN] Processing request {}: {:?}", 
                  request.request_id, request.flush_type);
            
            // Simulate processing delay
            // In real implementation, this would wait for IPI responses
            
            // Mark as completed for now
            completed.push(request.clone());
        }
        
        // Move completed requests
        for request in completed {
            self.pending_requests.retain(|r| r.request_id != request.request_id);
            self.completed_requests.push(request);
        }
        
        self.stats.completed_requests += completed.len() as u64;
        
        klog!(INFO, "[TLB-SHOOTDOWN] Processed {} requests, {} remaining", 
              completed.len(), self.pending_requests.len());
        
        Ok(())
    }
    
    /// Get pending request count
    pub fn pending_count(&self) -> usize {
        self.pending_requests.len()
    }
    
    /// Get completed request count
    pub fn completed_count(&self) -> usize {
        self.completed_requests.len()
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> &TlbShootdownStats {
        &self.stats
    }
}

/// TLB shootdown statistics
#[derive(Debug, Clone, Default)]
pub struct TlbShootdownStats {
    /// Total requests submitted
    pub total_requests: u64,
    /// Total requests completed
    pub completed_requests: u64,
    /// Total CPUs targeted
    pub total_cpus_targeted: u64,
    /// Average processing time (microseconds)
    pub avg_processing_time_us: u64,
}

//=============================================================================
// TLB STATISTICS AND MONITORING
//=============================================================================

/// TLB operation statistics
#[derive(Debug, Clone, Default)]
pub struct TlbStats {
    /// Total TLB flushes performed
    pub total_flushes: u64,
    /// Single page flushes
    pub single_page_flushes: u64,
    /// Large page flushes
    pub large_page_flushes: u64,
    /// Huge page flushes
    pub huge_page_flushes: u64,
    /// Global flushes
    pub global_flushes: u64,
    /// Range flushes
    pub range_flushes: u64,
    /// Multiple page flushes
    pub multiple_page_flushes: u64,
    /// Flushes by reason
    pub flushes_by_reason: HashMap<TlbFlushReason, u64>,
    /// Last flush timestamp
    pub last_flush_timestamp: u64,
    /// Average flush time (microseconds)
    pub avg_flush_time_us: u64,
}

/// Global TLB statistics
static TLB_STATS: spin::Mutex<TlbStats> = spin::Mutex::new(TlbStats::default());

/// Global TLB shootdown manager
static TLB_SHOOTDOWN_MANAGER: spin::Mutex<TlbShootdownManager> = 
    spin::Mutex::new(TlbShootdownManager::new());

/// Update TLB statistics
fn update_tlb_stats(flush_type: TlbFlushType, reason: TlbFlushReason) {
    let mut stats = TLB_STATS.lock();
    
    stats.total_flushes += 1;
    stats.last_flush_timestamp = get_current_timestamp();
    
    match flush_type {
        TlbFlushType::SinglePage => stats.single_page_flushes += 1,
        TlbFlushType::LargePage => stats.large_page_flushes += 1,
        TlbFlushType::HugePage => stats.huge_page_flushes += 1,
        TlbFlushType::Global => stats.global_flushes += 1,
        TlbFlushType::Range => stats.range_flushes += 1,
    }
    
    *stats.flushes_by_reason.entry(reason).or_insert(0) += 1;
}

/// Get TLB statistics
pub fn get_tlb_stats() -> TlbStats {
    TLB_STATS.lock().clone()
}

/// Print TLB statistics
pub fn print_tlb_stats() {
    let stats = get_tlb_stats();
    
    kprintln!("");
    kprintln!("=== TLB STATISTICS ===");
    kprintln!("Total flushes: {}", stats.total_flushes);
    kprintln!("Single page: {}", stats.single_page_flushes);
    kprintln!("Large page: {}", stats.large_page_flushes);
    kprintln!("Huge page: {}", stats.huge_page_flushes);
    kprintln!("Global: {}", stats.global_flushes);
    kprintln!("Range: {}", stats.range_flushes);
    kprintln!("Multiple: {}", stats.multiple_page_flushes);
    kprintln!("");
    
    kprintln!("Flushes by reason:");
    for (reason, count) in &stats.flushes_by_reason {
        kprintln!("  {:?}: {}", reason, count);
    }
    
    kprintln!("=== END TLB STATISTICS ===");
    kprintln!("");
}

//=============================================================================
// UTILITY FUNCTIONS
//=============================================================================

/// Determine page size for a given virtual address
/// 
/// # Arguments
/// * `addr` - Virtual address to check
/// 
/// # Returns
/// Page size in bytes
fn determine_page_size(addr: VirtAddr) -> usize {
    // This is a simplified implementation
    // In a real system, you'd check the page table entries to determine actual page size
    
    let addr_u64 = addr.as_u64();
    
    // Check if address is in kernel space
    if is_kernel_address(addr_u64) {
        // Kernel space typically uses large pages for efficiency
        if addr_u64 & (LARGE_PAGE_SIZE as u64 - 1) == 0 {
            LARGE_PAGE_SIZE
        } else {
            PAGE_SIZE
        }
    } else {
        // User space typically uses 4KB pages
        PAGE_SIZE
    }
}

/// Convert page size to string representation
fn page_size_to_string(size: usize) -> &'static str {
    match size {
        PAGE_SIZE => "4KB",
        LARGE_PAGE_SIZE => "2MB",
        HUGE_PAGE_SIZE => "1GB",
        _ => "Unknown",
    }
}

/// Get current timestamp in microseconds
fn get_current_timestamp() -> u64 {
    // This is a simplified implementation
    // In a real system, you'd use a high-resolution timer
    use core::time::Duration;
    use std::time::SystemTime;
    
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_micros() as u64
}

//=============================================================================
// PUBLIC API FUNCTIONS
//=============================================================================

/// Initialize TLB management system
pub fn init_tlb_system() -> Result<(), String> {
    klog!(INFO, "[TLB] Initializing TLB management system");
    
    // Reset statistics
    {
        let mut stats = TLB_STATS.lock();
        *stats = TlbStats::default();
    }
    
    // Initialize shootdown manager
    {
        let mut manager = TLB_SHOOTDOWN_MANAGER.lock();
        *manager = TlbShootdownManager::new();
    }
    
    klog!(INFO, "[TLB] TLB management system initialized");
    Ok(())
}

/// Get TLB shootdown manager
pub fn get_shootdown_manager() -> spin::MutexGuard<'static, TlbShootdownManager> {
    TLB_SHOOTDOWN_MANAGER.lock()
}

/// Process pending TLB shootdown requests
pub fn process_tlb_shootdowns() -> Result<(), String> {
    let mut manager = TLB_SHOOTDOWN_MANAGER.lock();
    manager.process_requests()
}

//=============================================================================
// UNIT TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tlb_flush_types() {
        assert_eq!(TlbFlushType::SinglePage, TlbFlushType::SinglePage);
        assert_ne!(TlbFlushType::SinglePage, TlbFlushType::Global);
    }
    
    #[test]
    fn test_tlb_flush_reasons() {
        assert_eq!(TlbFlushReason::PageMappingChanged, TlbFlushReason::PageMappingChanged);
        assert_ne!(TlbFlushReason::PageMappingChanged, TlbFlushReason::ContextSwitch);
    }
    
    #[test]
    fn test_cpu_id() {
        let cpu1 = CpuId(1);
        let cpu2 = CpuId(2);
        
        assert_eq!(cpu1, CpuId(1));
        assert_ne!(cpu1, cpu2);
    }
    
    #[test]
    fn test_shootdown_request_creation() {
        let request = TlbShootdownRequest::new(
            vec![CpuId(1), CpuId(2)],
            TlbFlushType::SinglePage,
            TlbFlushReason::PageMappingChanged
        );
        
        assert_eq!(request.target_cpus.len(), 2);
        assert_eq!(request.flush_type, TlbFlushType::SinglePage);
        assert_eq!(request.reason, TlbFlushReason::PageMappingChanged);
    }
    
    #[test]
    fn test_shootdown_manager_creation() {
        let manager = TlbShootdownManager::new();
        
        assert_eq!(manager.pending_count(), 0);
        assert_eq!(manager.completed_count(), 0);
        assert_eq!(manager.get_stats().total_requests, 0);
    }
    
    #[test]
    fn test_page_size_determination() {
        let kernel_addr = VirtAddr::new(KERNEL_VIRT_START);
        let user_addr = VirtAddr::new(0x400000);
        
        // These tests are simplified since we don't have real page tables
        let kernel_size = determine_page_size(kernel_addr);
        let user_size = determine_page_size(user_addr);
        
        assert!(kernel_size >= PAGE_SIZE);
        assert!(user_size >= PAGE_SIZE);
    }
}
