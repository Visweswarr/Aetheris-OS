/// Page Table Audit Tool for Polymera OS
/// 
/// This module provides a debug-only page table audit tool that walks page table
/// mappings and reports suspicious combinations (e.g., writable+executable pages).
/// It helps ensure MMU hygiene and security policy compliance.

use crate::{kprintln, klog, kprintln};
use crate::log::Level;
use crate::secman::audit::{audit_log, AuditEvent, AuditLevel};
use super::paging::{PageFlags, get_paging_stats, flush_tlb};
use super::constants::*;
use x86_64::{
    structures::paging::{PageTable, PageTableFlags, Page, Size4KiB, Mapper, OffsetPageTable},
    registers::control::Cr3,
    VirtAddr, PhysAddr,
};
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use alloc::collections::HashMap;
use alloc::string::String;
use alloc::vec::Vec;

//=============================================================================
// AUDIT CONFIGURATION AND CONSTANTS
//=============================================================================

/// Maximum number of suspicious mappings to report before stopping
const MAX_SUSPICIOUS_REPORTS: usize = 1000;

/// Maximum number of pages to audit before stopping (safety limit)
const MAX_PAGES_TO_AUDIT: usize = 1_000_000;

/// Memory regions to audit
const AUDIT_REGIONS: &[(u64, u64, &str)] = &[
    (KERNEL_VIRT_START, KERNEL_VIRT_START + 0x1000000, "kernel-code"),
    (KERNEL_VIRT_START + 0x1000000, KERNEL_VIRT_START + 0x2000000, "kernel-data"),
    (KERNEL_VIRT_START + 0x2000000, KERNEL_VIRT_START + 0x3000000, "kernel-heap"),
    (0x400000, 0x800000, "user-space"),
];

//=============================================================================
// AUDIT RESULTS AND STATISTICS
//=============================================================================

/// Result of a page table audit
#[derive(Debug, Clone)]
pub struct AuditResult {
    /// Total pages audited
    pub total_pages: u64,
    
    /// Pages with suspicious flags
    pub suspicious_pages: u64,
    
    /// Pages with W+X (writable and executable)
    pub writable_executable_pages: u64,
    
    /// Pages with user-accessible kernel memory
    pub user_accessible_kernel_pages: u64,
    
    /// Pages with unexpected combinations
    pub unexpected_combinations: u64,
    
    /// Audit completion status
    pub completed: bool,
    
    /// Error message if audit failed
    pub error_message: Option<String>,
}

impl AuditResult {
    /// Create new audit result
    pub const fn new() -> Self {
        Self {
            total_pages: 0,
            suspicious_pages: 0,
            writable_executable_pages: 0,
            user_accessible_kernel_pages: 0,
            unexpected_combinations: 0,
            completed: false,
            error_message: None,
        }
    }
    
    /// Check if audit found any issues
    pub fn has_issues(&self) -> bool {
        self.suspicious_pages > 0
    }
    
    /// Get summary string
    pub fn summary(&self) -> String {
        if self.has_issues() {
            format!(
                "AUDIT ISSUES: {} suspicious pages ({} W+X, {} user-kernel, {} unexpected)",
                self.suspicious_pages,
                self.writable_executable_pages,
                self.user_accessible_kernel_pages,
                self.unexpected_combinations
            )
        } else {
            format!("AUDIT CLEAN: {} pages audited, no issues found", self.total_pages)
        }
    }
}

/// Detailed information about a suspicious page
#[derive(Debug, Clone)]
pub struct SuspiciousPage {
    /// Virtual address of the page
    pub virtual_address: u64,
    
    /// Physical address of the page
    pub physical_address: u64,
    
    /// Page flags that caused the suspicion
    pub flags: PageFlags,
    
    /// Reason for suspicion
    pub reason: String,
    
    /// Memory region context
    pub region: String,
    
    /// Severity level
    pub severity: AuditLevel,
}

impl SuspiciousPage {
    /// Create new suspicious page entry
    pub fn new(vaddr: u64, paddr: u64, flags: PageFlags, reason: String, region: String) -> Self {
        let severity = if flags.writable && flags.executable {
            AuditLevel::Critical
        } else if flags.user_accessible && vaddr >= KERNEL_VIRT_START {
            AuditLevel::High
        } else {
            AuditLevel::Medium
        };
        
        Self {
            virtual_address: vaddr,
            physical_address: paddr,
            flags,
            reason,
            region,
            severity,
        }
    }
}

/// Page table audit statistics
#[derive(Debug, Clone, Default)]
pub struct AuditStats {
    /// Total audits performed
    pub total_audits: u64,
    
    /// Total pages audited across all audits
    pub total_pages_audited: u64,
    
    /// Total suspicious pages found
    pub total_suspicious_pages: u64,
    
    /// Total W+X violations found
    pub total_wx_violations: u64,
    
    /// Last audit timestamp
    pub last_audit_timestamp: u64,
    
    /// Last audit result
    pub last_audit_result: Option<AuditResult>,
}

//=============================================================================
// PAGE TABLE WALKER
//=============================================================================

/// Page table walker for auditing
pub struct PageTableWalker {
    /// Current page table being walked
    page_table: Option<OffsetPageTable<'static>>,
    
    /// Current audit results
    results: AuditResult,
    
    /// Suspicious pages found
    suspicious_pages: Vec<SuspiciousPage>,
    
    /// Audit progress tracking
    pages_processed: u64,
    
    /// Whether to continue walking
    continue_walking: bool,
}

impl PageTableWalker {
    /// Create new page table walker
    pub fn new() -> Self {
        Self {
            page_table: None,
            results: AuditResult::new(),
            suspicious_pages: Vec::new(),
            pages_processed: 0,
            continue_walking: true,
        }
    }
    
    /// Initialize page table walker with current page table
    pub fn init(&mut self) -> Result<(), String> {
        // Get current page table from CR3
        let (level_4_table_frame, _) = Cr3::read();
        let level_4_table_ptr = level_4_table_frame.start_address().as_u64();
        
        // Safety check: ensure we're accessing valid kernel memory
        if !is_kernel_address(level_4_table_ptr) {
            return Err("Invalid page table address".to_string());
        }
        
        // Create offset page table reference
        let level_4_table = unsafe {
            &mut *(level_4_table_ptr as *mut PageTable)
        };
        
        let offset_table = unsafe {
            OffsetPageTable::new(level_4_table, VirtAddr::new(0))
        };
        
        self.page_table = Some(offset_table);
        Ok(())
    }
    
    /// Walk page table and audit all mappings
    pub fn walk_and_audit(&mut self) -> Result<AuditResult, String> {
        let page_table = self.page_table.as_mut()
            .ok_or("Page table not initialized")?;
        
        klog!(INFO, "[AUDIT] Starting page table walk and audit");
        
        // Walk through each audit region
        for (start, end, region_name) in AUDIT_REGIONS {
            if !self.continue_walking {
                break;
            }
            
            klog!(DEBUG, "[AUDIT] Auditing region {}: 0x{:016x} - 0x{:016x}", 
                  region_name, start, end);
            
            self.audit_region(page_table, *start, *end, region_name)?;
        }
        
        self.results.completed = true;
        self.results.total_pages = self.pages_processed;
        
        klog!(INFO, "[AUDIT] Page table audit completed: {} pages processed", 
              self.pages_processed);
        
        Ok(self.results.clone())
    }
    
    /// Audit a specific memory region
    fn audit_region(&mut self, page_table: &mut OffsetPageTable, 
                   start: u64, end: u64, region_name: &str) -> Result<(), String> {
        let start_page = Page::<Size4KiB>::containing_address(VirtAddr::new(start));
        let end_page = Page::<Size4KiB>::containing_address(VirtAddr::new(end));
        
        let mut current_page = start_page;
        
        while current_page < end_page && self.continue_walking {
            if self.pages_processed >= MAX_PAGES_TO_AUDIT as u64 {
                klog!(WARN, "[AUDIT] Reached maximum page limit, stopping audit");
                break;
            }
            
            // Check if page is mapped
            if let Ok((frame, flags)) = page_table.translate_page(current_page) {
                self.audit_page(
                    current_page.start_address().as_u64(),
                    frame.start_address().as_u64(),
                    PageFlags::from_x86_64_flags(flags),
                    region_name
                );
            }
            
            self.pages_processed += 1;
            current_page += 1;
        }
        
        Ok(())
    }
    
    /// Audit a single page
    fn audit_page(&mut self, vaddr: u64, paddr: u64, flags: PageFlags, region: &str) {
        let mut issues = Vec::new();
        
        // Check for W+X (writable and executable) - critical security issue
        if flags.writable && flags.executable {
            issues.push("Writable and executable (W+X) - CRITICAL SECURITY VIOLATION".to_string());
            self.results.writable_executable_pages += 1;
        }
        
        // Check for user-accessible kernel memory
        if flags.user_accessible && vaddr >= KERNEL_VIRT_START {
            issues.push("User-accessible kernel memory - HIGH SECURITY RISK".to_string());
            self.results.user_accessible_kernel_pages += 1;
        }
        
        // Check for unexpected flag combinations
        if self.is_unexpected_combination(&flags, region) {
            issues.push("Unexpected flag combination for region".to_string());
            self.results.unexpected_combinations += 1;
        }
        
        // If issues found, add to suspicious pages
        if !issues.is_empty() {
            let reason = issues.join("; ");
            let suspicious_page = SuspiciousPage::new(vaddr, paddr, flags, reason, region.to_string());
            
            self.suspicious_pages.push(suspicious_page);
            self.results.suspicious_pages += 1;
            
            // Log the issue
            klog!(WARN, "[AUDIT] Suspicious page at 0x{:016x}: {}", vaddr, reason);
            
            // Stop if we've found too many suspicious pages
            if self.suspicious_pages.len() >= MAX_SUSPICIOUS_REPORTS {
                klog!(ERROR, "[AUDIT] Too many suspicious pages, stopping audit");
                self.continue_walking = false;
            }
        }
    }
    
    /// Check if flags represent an unexpected combination for the region
    fn is_unexpected_combination(&self, flags: &PageFlags, region: &str) -> bool {
        match region {
            "kernel-code" => {
                // Kernel code should be present, executable, not writable, not user-accessible
                !flags.present || !flags.executable || flags.writable || flags.user_accessible
            }
            "kernel-data" => {
                // Kernel data should be present, writable, not executable, not user-accessible
                !flags.present || flags.executable || !flags.writable || flags.user_accessible
            }
            "kernel-heap" => {
                // Kernel heap should be present, writable, not executable, not user-accessible
                !flags.present || flags.executable || !flags.writable || flags.user_accessible
            }
            "user-space" => {
                // User space can have various combinations, but not W+X
                flags.writable && flags.executable
            }
            _ => false
        }
    }
    
    /// Get suspicious pages found during audit
    pub fn get_suspicious_pages(&self) -> &[SuspiciousPage] {
        &self.suspicious_pages
    }
    
    /// Stop the audit walk
    pub fn stop(&mut self) {
        self.continue_walking = false;
    }
}

//=============================================================================
// AUDIT MANAGER
//=============================================================================

/// Global audit manager
pub struct AuditManager {
    /// Audit statistics
    stats: AuditStats,
    
    /// Current walker instance
    current_walker: Option<PageTableWalker>,
}

impl AuditManager {
    /// Create new audit manager
    pub fn new() -> Self {
        Self {
            stats: AuditStats::default(),
            current_walker: None,
        }
    }
    
    /// Run a complete page table audit
    pub fn run_audit(&mut self) -> Result<AuditResult, String> {
        klog!(INFO, "[AUDIT] Starting page table audit");
        
        // Create new walker
        let mut walker = PageTableWalker::new();
        walker.init()?;
        
        // Run the audit
        let result = walker.walk_and_audit()?;
        
        // Update statistics
        self.stats.total_audits += 1;
        self.stats.total_pages_audited += result.total_pages;
        self.stats.total_suspicious_pages += result.suspicious_pages;
        self.stats.total_wx_violations += result.writable_executable_pages;
        self.stats.last_audit_result = Some(result.clone());
        
        // Log audit completion
        if result.has_issues() {
            klog!(WARN, "[AUDIT] Audit completed with issues: {}", result.summary());
            
            // Log suspicious pages
            for page in walker.get_suspicious_pages() {
                audit_log(AuditEvent::PageTableAudit, page.severity, 
                         &format!("Suspicious page: 0x{:016x} - {}", 
                                 page.virtual_address, page.reason));
            }
        } else {
            klog!(INFO, "[AUDIT] Audit completed successfully: {}", result.summary());
        }
        
        Ok(result)
    }
    
    /// Get audit statistics
    pub fn get_stats(&self) -> &AuditStats {
        &self.stats
    }
    
    /// Reset audit statistics
    pub fn reset_stats(&mut self) {
        self.stats = AuditStats::default();
        klog!(INFO, "[AUDIT] Audit statistics reset");
    }
}

//=============================================================================
// GLOBAL INSTANCES AND PUBLIC API
//=============================================================================

/// Global audit manager instance
static AUDIT_MANAGER: spin::Mutex<AuditManager> = spin::Mutex::new(AuditManager::new());

/// Global audit statistics
static AUDIT_STATS: spin::Mutex<AuditStats> = spin::Mutex::new(AuditStats::default());

/// Initialize the page table audit system
pub fn init_audit_system() -> Result<(), String> {
    let mut manager = AUDIT_MANAGER.lock();
    manager.reset_stats();
    
    klog!(INFO, "[AUDIT] Page table audit system initialized");
    Ok(())
}

/// Run a page table audit
pub fn run_page_table_audit() -> Result<AuditResult, String> {
    let mut manager = AUDIT_MANAGER.lock();
    manager.run_audit()
}

/// Get current audit statistics
pub fn get_audit_stats() -> AuditStats {
    AUDIT_STATS.lock().clone()
}

/// Print audit statistics
pub fn print_audit_stats() {
    let stats = get_audit_stats();
    
    kprintln!("");
    kprintln!("=== PAGE TABLE AUDIT STATISTICS ===");
    kprintln!("Total audits performed: {}", stats.total_audits);
    kprintln!("Total pages audited: {}", stats.total_pages_audited);
    kprintln!("Total suspicious pages: {}", stats.total_suspicious_pages);
    kprintln!("Total W+X violations: {}", stats.total_wx_violations);
    
    if let Some(last_result) = &stats.last_audit_result {
        kprintln!("");
        kprintln!("Last audit result:");
        kprintln!("  {}", last_result.summary());
        kprintln!("  Total pages: {}", last_result.total_pages);
        kprintln!("  Completed: {}", last_result.completed);
        
        if let Some(error) = &last_result.error_message {
            kprintln!("  Error: {}", error);
        }
    }
    
    kprintln!("=== END AUDIT STATISTICS ===");
    kprintln!("");
}

/// Print detailed suspicious page information
pub fn print_suspicious_pages() {
    let manager = AUDIT_MANAGER.lock();
    
    if let Some(walker) = &manager.current_walker {
        let suspicious_pages = walker.get_suspicious_pages();
        
        if suspicious_pages.is_empty() {
            kprintln!("No suspicious pages found in current audit");
            return;
        }
        
        kprintln!("");
        kprintln!("=== SUSPICIOUS PAGES ===");
        
        for (i, page) in suspicious_pages.iter().enumerate() {
            kprintln!("Page {}: 0x{:016x} -> 0x{:016x}", i + 1, 
                     page.virtual_address, page.physical_address);
            kprintln!("  Region: {}", page.region);
            kprintln!("  Flags: {:?}", page.flags);
            kprintln!("  Reason: {}", page.reason);
            kprintln!("  Severity: {:?}", page.severity);
            kprintln!("");
        }
        
        kprintln!("=== END SUSPICIOUS PAGES ===");
        kprintln!("");
    } else {
        kprintln!("No current audit walker available");
    }
}

//=============================================================================
// UNIT TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audit_result_creation() {
        let result = AuditResult::new();
        assert_eq!(result.total_pages, 0);
        assert_eq!(result.suspicious_pages, 0);
        assert!(!result.has_issues());
    }
    
    #[test]
    fn test_suspicious_page_creation() {
        let flags = PageFlags {
            present: true,
            writable: true,
            user_accessible: false,
            executable: true,
            accessed: false,
            dirty: false,
            write_through: false,
            cache_disabled: false,
        };
        
        let page = SuspiciousPage::new(
            0x1000, 0x2000, flags, 
            "Test reason".to_string(), 
            "test-region".to_string()
        );
        
        assert_eq!(page.virtual_address, 0x1000);
        assert_eq!(page.physical_address, 0x2000);
        assert_eq!(page.severity, AuditLevel::Critical); // W+X should be critical
    }
    
    #[test]
    fn test_audit_result_summary() {
        let mut result = AuditResult::new();
        result.total_pages = 1000;
        result.suspicious_pages = 5;
        result.writable_executable_pages = 2;
        
        let summary = result.summary();
        assert!(summary.contains("AUDIT ISSUES"));
        assert!(summary.contains("5 suspicious pages"));
        assert!(summary.contains("2 W+X"));
    }
    
    #[test]
    fn test_page_table_walker_creation() {
        let walker = PageTableWalker::new();
        assert_eq!(walker.pages_processed, 0);
        assert!(walker.continue_walking);
        assert!(walker.suspicious_pages.is_empty());
    }
}
