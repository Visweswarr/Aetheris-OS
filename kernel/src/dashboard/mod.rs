//! ASCII Dashboard Module
//! 
//! This module provides a debug screen that displays system information
//! in an ASCII box format, including version, target, tick counter,
//! runqueue size, and IPC counters.

use crate::{klog, format};
use crate::log::{Level, tags};
use crate::ipc::IpcStats;
use crate::sched::runqueue::RunQueue;
use crate::hal::x86_64::X64Hal;
use alloc::string::{String, ToString};

/// Dashboard configuration
pub struct DashboardConfig {
    /// Whether to show detailed information
    pub detailed: bool,
    /// Whether to show performance metrics
    pub show_performance: bool,
    /// Whether to show memory information
    pub show_memory: bool,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            detailed: false,
            show_performance: true,
            show_memory: true,
        }
    }
}

/// System information for dashboard display
pub struct SystemInfo {
    /// Kernel version
    pub version: String,
    /// Target architecture
    pub target: String,
    /// Current tick counter
    pub tick_count: u64,
    /// Runqueue size
    pub runqueue_size: usize,
    /// IPC statistics
    pub ipc_stats: IpcStats,
    /// System uptime in milliseconds
    pub uptime_ms: u64,
    /// Memory usage information
    pub memory_info: MemoryInfo,
}

/// Memory usage information
pub struct MemoryInfo {
    /// Total physical memory in bytes
    pub total_physical: u64,
    /// Used memory in bytes
    pub used_memory: u64,
    /// Available memory in bytes
    pub available_memory: u64,
    /// Kernel memory usage in bytes
    pub kernel_memory: u64,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            target: crate::TARGET_ARCH.to_string(),
            tick_count: 0,
            runqueue_size: 0,
            ipc_stats: IpcStats::default(),
            uptime_ms: 0,
            memory_info: MemoryInfo {
                total_physical: 0,
                used_memory: 0,
                available_memory: 0,
                kernel_memory: 0,
            },
        }
    }
}

/// Collect current system information
pub fn collect_system_info() -> SystemInfo {
    let mut info = SystemInfo::default();
    
    // Get tick count
    info.tick_count = X64Hal::get_tick_count();
    
    // Get uptime
    info.uptime_ms = X64Hal::get_uptime_ms();
    
    // Get runqueue size
    info.runqueue_size = RunQueue::get_current_size();
    
    // Get IPC statistics
    info.ipc_stats = crate::ipc::get_stats();
    
    // Get memory information (stub for now)
    info.memory_info = MemoryInfo {
        total_physical: 8 * 1024 * 1024 * 1024, // 8 GB
        used_memory: 2 * 1024 * 1024 * 1024,    // 2 GB
        available_memory: 6 * 1024 * 1024 * 1024, // 6 GB
        kernel_memory: 64 * 1024 * 1024,         // 64 MB
    };
    
    info
}

/// Print the ASCII dashboard
pub fn print_dashboard() {
    print_dashboard_with_config(&DashboardConfig::default());
}

/// Print the ASCII dashboard with custom configuration
pub fn print_dashboard_with_config(config: &DashboardConfig) {
    let info = collect_system_info();
    
    // Print header
    println_dashboard_header();
    
    // Print version and target
    println_dashboard_section("SYSTEM INFO", &format!("Version: {}", info.version));
    println_dashboard_section("", &format!("Target: {}", info.target));
    
    // Print uptime and tick information
    println_dashboard_section("TIMING", &format!("Uptime: {} ms", info.uptime_ms));
    println_dashboard_section("", &format!("Ticks: {}", info.tick_count));
    
    // Print scheduler information
    println_dashboard_section("SCHEDULER", &format!("Runqueue Size: {}", info.runqueue_size));
    
    // Print IPC information
    println_dashboard_section("IPC STATISTICS", &format!("Messages Sent: {}", info.ipc_stats.messages_sent));
    println_dashboard_section("", &format!("Messages Received: {}", info.ipc_stats.messages_received));
    println_dashboard_section("", &format!("Channels Created: {}", info.ipc_stats.active_channels));
    println_dashboard_section("", &format!("Active Channels: {}", info.ipc_stats.active_channels));
    
    if config.show_performance {
        let trace_stats = crate::trace::get_system_stats();
        // Print performance metrics
        println_dashboard_section("PERFORMANCE", &format!("IPC Latency (avg): {} μs", trace_stats.ipc_latency_mean_us));
        println_dashboard_section("", &format!("Context Switches: {}", trace_stats.ctx_switches));
    }
    
    if config.show_memory {
        // Print memory information
        println_dashboard_section("MEMORY", &format!("Total: {} MB", info.memory_info.total_physical / (1024 * 1024)));
        println_dashboard_section("", &format!("Used: {} MB", info.memory_info.used_memory / (1024 * 1024)));
        println_dashboard_section("", &format!("Available: {} MB", info.memory_info.available_memory / (1024 * 1024)));
        println_dashboard_section("", &format!("Kernel: {} MB", info.memory_info.kernel_memory / (1024 * 1024)));
    }
    
    if config.detailed {
        // Print detailed information
        println_dashboard_section("DETAILED INFO", &format!("Build Date: {}", option_env!("VERGEN_BUILD_TIMESTAMP").unwrap_or("Unknown")));
        println_dashboard_section("", &format!("Git Commit: {}", option_env!("VERGEN_GIT_SHA_SHORT").unwrap_or("Unknown")));
        println_dashboard_section("", &format!("Rust Version: {}", option_env!("VERGEN_RUSTC_SEMVER").unwrap_or("Unknown")));
    }
    
    // Print footer
    println_dashboard_footer();
}

/// Print dashboard header with ASCII box
fn println_dashboard_header() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                           POLYMERA OS DASHBOARD                              ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
}

/// Print dashboard section
fn println_dashboard_section(title: &str, content: &str) {
    if title.is_empty() {
        println!("║ {:70} ║", content);
    } else {
        println!("║ {:70} ║", format!("{}: {}", title, content));
    }
}

/// Print dashboard footer
fn println_dashboard_footer() {
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
}

/// Print compact dashboard (single line)
pub fn print_compact_dashboard() {
    let info = collect_system_info();
    
    println!(
        "📊 [v{}] [{}] [Ticks:{}] [RQ:{}] [IPC:{}] [Uptime:{}ms]",
        info.version,
        info.target,
        info.tick_count,
        info.runqueue_size,
        info.ipc_stats.messages_sent,
        info.uptime_ms
    );
}

/// Print minimal dashboard (key metrics only)
pub fn print_minimal_dashboard() {
    let info = collect_system_info();
    
    println!("╔═══ POLYMERA OS STATUS ═══╗");
    println!("║ Version: {:<15} ║", info.version);
    println!("║ Target:  {:<15} ║", info.target);
    println!("║ Ticks:   {:<15} ║", info.tick_count);
    println!("║ RunQ:    {:<15} ║", info.runqueue_size);
    println!("║ IPC:     {:<15} ║", info.ipc_stats.messages_sent);
    println!("╚═══════════════════════════╝");
}

/// Initialize the dashboard module
pub fn init() {
    klog!(Level::INFO, [tags::DASHBOARD], "Dashboard module initialized");
    
    // Print initial dashboard at boot
    print_dashboard();
}

/// Test dashboard functionality
pub fn test_dashboard() {
    klog!(Level::INFO, [tags::DASHBOARD], "Testing dashboard functionality...");
    
    // Test different dashboard formats
    println!("Testing full dashboard:");
    print_dashboard();
    
    println!("Testing compact dashboard:");
    print_compact_dashboard();
    
    println!("Testing minimal dashboard:");
    print_minimal_dashboard();
    
    // Test with custom configuration
    let config = DashboardConfig {
        detailed: true,
        show_performance: true,
        show_memory: true,
    };
    
    println!("Testing detailed dashboard:");
    print_dashboard_with_config(&config);
    
    klog!(Level::INFO, [tags::DASHBOARD], "Dashboard tests completed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_config_default() {
        let config = DashboardConfig::default();
        assert!(!config.detailed);
        assert!(config.show_performance);
        assert!(config.show_memory);
    }

    #[test]
    fn test_system_info_default() {
        let info = SystemInfo::default();
        assert!(!info.version.is_empty());
        assert!(!info.target.is_empty());
        assert_eq!(info.tick_count, 0);
        assert_eq!(info.runqueue_size, 0);
    }

    #[test]
    fn test_memory_info_creation() {
        let memory = MemoryInfo {
            total_physical: 1024 * 1024 * 1024, // 1 GB
            used_memory: 512 * 1024 * 1024,     // 512 MB
            available_memory: 512 * 1024 * 1024, // 512 MB
            kernel_memory: 64 * 1024 * 1024,    // 64 MB
        };
        
        assert_eq!(memory.total_physical, 1024 * 1024 * 1024);
        assert_eq!(memory.used_memory, 512 * 1024 * 1024);
        assert_eq!(memory.available_memory, 512 * 1024 * 1024);
        assert_eq!(memory.kernel_memory, 64 * 1024 * 1024);
    }

    #[test]
    fn test_collect_system_info() {
        let info = collect_system_info();
        assert!(!info.version.is_empty());
        assert!(!info.target.is_empty());
        assert!(info.memory_info.total_physical > 0);
    }
}



