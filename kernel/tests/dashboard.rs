//! Tests for ASCII Dashboard Module
//! 
//! These tests verify that the dashboard displays system information
//! correctly in various formats.

use crate::dashboard::*;

/// Test dashboard configuration functionality
pub fn test_dashboard_config() -> Result<(), &'static str> {
    crate::kprintln!("Testing dashboard configuration...");
    
    // Test default configuration
    let config = DashboardConfig::default();
    if config.detailed {
        return Err("Default config should not be detailed");
    }
    if !config.show_performance {
        return Err("Default config should show performance");
    }
    if !config.show_memory {
        return Err("Default config should show memory");
    }
    
    // Test custom configuration
    let custom_config = DashboardConfig {
        detailed: true,
        show_performance: false,
        show_memory: false,
    };
    
    if !custom_config.detailed {
        return Err("Custom config should be detailed");
    }
    if custom_config.show_performance {
        return Err("Custom config should not show performance");
    }
    if custom_config.show_memory {
        return Err("Custom config should not show memory");
    }
    
    crate::kprintln!("✅ Dashboard configuration tests passed!");
    Ok(())
}

/// Test system information collection
pub fn test_system_info_collection() -> Result<(), &'static str> {
    crate::kprintln!("Testing system information collection...");
    
    // Test default system info
    let info = SystemInfo::default();
    if info.version.is_empty() {
        return Err("Version should not be empty");
    }
    if info.target.is_empty() {
        return Err("Target should not be empty");
    }
    if info.tick_count != 0 {
        return Err("Default tick count should be 0");
    }
    if info.runqueue_size != 0 {
        return Err("Default runqueue size should be 0");
    }
    
    // Test system info collection
    let collected_info = collect_system_info();
    if collected_info.version.is_empty() {
        return Err("Collected version should not be empty");
    }
    if collected_info.target.is_empty() {
        return Err("Collected target should not be empty");
    }
    if collected_info.memory_info.total_physical == 0 {
        return Err("Collected memory info should have non-zero total");
    }
    
    crate::kprintln!("✅ System information collection tests passed!");
    Ok(())
}

/// Test memory information structures
pub fn test_memory_info_structures() -> Result<(), &'static str> {
    crate::kprintln!("Testing memory information structures...");
    
    // Test memory info creation
    let memory = MemoryInfo {
        total_physical: 16 * 1024 * 1024 * 1024, // 16 GB
        used_memory: 8 * 1024 * 1024 * 1024,     // 8 GB
        available_memory: 8 * 1024 * 1024 * 1024, // 8 GB
        kernel_memory: 128 * 1024 * 1024,         // 128 MB
    };
    
    if memory.total_physical != 16 * 1024 * 1024 * 1024 {
        return Err("Memory total physical should be 16 GB");
    }
    if memory.used_memory != 8 * 1024 * 1024 * 1024 {
        return Err("Memory used should be 8 GB");
    }
    if memory.available_memory != 8 * 1024 * 1024 * 1024 {
        return Err("Memory available should be 8 GB");
    }
    if memory.kernel_memory != 128 * 1024 * 1024 {
        return Err("Memory kernel should be 128 MB");
    }
    
    // Test memory calculations
    let total_mb = memory.total_physical / (1024 * 1024);
    let used_mb = memory.used_memory / (1024 * 1024);
    let available_mb = memory.available_memory / (1024 * 1024);
    let kernel_mb = memory.kernel_memory / (1024 * 1024);
    
    if total_mb != 16384 {
        return Err("Total memory in MB should be 16384");
    }
    if used_mb != 8192 {
        return Err("Used memory in MB should be 8192");
    }
    if available_mb != 8192 {
        return Err("Available memory in MB should be 8192");
    }
    if kernel_mb != 128 {
        return Err("Kernel memory in MB should be 128");
    }
    
    crate::kprintln!("✅ Memory information structure tests passed!");
    Ok(())
}

/// Test dashboard display functions
pub fn test_dashboard_display() -> Result<(), &'static str> {
    crate::kprintln!("Testing dashboard display functions...");
    
    // Test that display functions don't panic
    // Note: We can't easily capture stdout in tests, so we just verify they run
    
    // Test full dashboard
    print_dashboard();
    
    // Test compact dashboard
    print_compact_dashboard();
    
    // Test minimal dashboard
    print_minimal_dashboard();
    
    // Test with custom configuration
    let config = DashboardConfig {
        detailed: true,
        show_performance: true,
        show_memory: true,
    };
    print_dashboard_with_config(&config);
    
    crate::kprintln!("✅ Dashboard display tests passed!");
    Ok(())
}

/// Test dashboard integration with syscall
pub fn test_dashboard_syscall_integration() -> Result<(), &'static str> {
    crate::kprintln!("Testing dashboard syscall integration...");
    
    // Test that the dashboard can be called via syscall simulation
    // This verifies the integration without actually calling the syscall
    
    // Simulate the syscall handler logic
    let result = std::panic::catch_unwind(|| {
        crate::dashboard::print_dashboard();
    });
    
    if result.is_err() {
        return Err("Dashboard print should not panic");
    }
    
    crate::kprintln!("✅ Dashboard syscall integration tests passed!");
    Ok(())
}

/// Test dashboard performance
pub fn test_dashboard_performance() -> Result<(), &'static str> {
    crate::kprintln!("Testing dashboard performance...");
    
    // Test that dashboard functions complete in reasonable time
    let start = std::time::Instant::now();
    
    // Collect system info multiple times
    for _ in 0..100 {
        let _info = collect_system_info();
    }
    
    let duration = start.elapsed();
    if duration.as_millis() > 100 {
        return Err("Dashboard info collection should be fast (< 100ms for 100 calls)");
    }
    
    crate::kprintln!("✅ Dashboard performance tests passed!");
    Ok(())
}

/// Run all dashboard tests
pub fn run_all_dashboard_tests() -> Result<(), &'static str> {
    crate::kprintln!("🚀 Running dashboard tests...");
    
    test_dashboard_config()?;
    test_system_info_collection()?;
    test_memory_info_structures()?;
    test_dashboard_display()?;
    test_dashboard_syscall_integration()?;
    test_dashboard_performance()?;
    
    crate::kprintln!("🎉 All dashboard tests completed successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        assert!(test_dashboard_config().is_ok());
    }

    #[test]
    fn test_system_info() {
        assert!(test_system_info_collection().is_ok());
    }

    #[test]
    fn test_memory_structures() {
        assert!(test_memory_info_structures().is_ok());
    }

    #[test]
    fn test_display() {
        assert!(test_dashboard_display().is_ok());
    }

    #[test]
    fn test_syscall_integration() {
        assert!(test_dashboard_syscall_integration().is_ok());
    }

    #[test]
    fn test_performance() {
        assert!(test_dashboard_performance().is_ok());
    }

    #[test]
    fn test_all() {
        assert!(run_all_dashboard_tests().is_ok());
    }
}



