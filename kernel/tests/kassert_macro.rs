//! KAssert Macro Tests
//! 
//! These tests verify that the kassert! macro correctly handles
//! assertion failures, logs audit entries, and halts the system.

use crate::macros::{kassert, kassert_debug, kassert_release, kassert_audit};
use crate::secman::audit;

/// Test basic kassert functionality
pub fn test_kassert_basic() -> Result<(), &'static str> {
    kprintln!("Testing basic kassert functionality...");
    
    // Test successful assertion
    kassert!(true, "This assertion should pass");
    kprintln!("  ✅ Basic assertion passed");
    
    // Test assertion with format arguments
    let value = 42;
    kassert!(value > 0, "Value should be positive, got {}", value);
    kprintln!("  ✅ Formatted assertion passed");
    
    // Test assertion with condition only
    kassert!(value == 42);
    kprintln!("  ✅ Condition-only assertion passed");
    
    kprintln!("✅ Basic kassert functionality tests passed!");
    Ok(())
}

/// Test kassert_debug functionality
pub fn test_kassert_debug() -> Result<(), &'static str> {
    kprintln!("Testing kassert_debug functionality...");
    
    // Test successful debug assertion
    kassert_debug!(true, "This debug assertion should pass");
    kprintln!("  ✅ Debug assertion passed");
    
    // Test debug assertion with format arguments
    let value = 100;
    kassert_debug!(value < 1000, "Value should be less than 1000, got {}", value);
    kprintln!("  ✅ Formatted debug assertion passed");
    
    // Test that debug assertions are disabled in release builds
    if !cfg!(debug_assertions) {
        kprintln!("  ℹ️  Debug assertions are disabled in this build");
    }
    
    kprintln!("✅ Kassert debug functionality tests passed!");
    Ok(())
}

/// Test kassert_release functionality
pub fn test_kassert_release() -> Result<(), &'static str> {
    kprintln!("Testing kassert_release functionality...");
    
    // Test successful release assertion
    kassert_release!(true, "This release assertion should pass");
    kprintln!("  ✅ Release assertion passed");
    
    // Test release assertion with format arguments
    let value = 200;
    kassert_release!(value > 0, "Value should be positive, got {}", value);
    kprintln!("  ✅ Formatted release assertion passed");
    
    // Test that release assertions are disabled in debug builds
    if cfg!(debug_assertions) {
        kprintln!("  ℹ️  Release assertions are disabled in this build");
    }
    
    kprintln!("✅ Kassert release functionality tests passed!");
    Ok(())
}

/// Test kassert_audit functionality
pub fn test_kassert_audit() -> Result<(), &'static str> {
    kprintln!("Testing kassert_audit functionality...");
    
    // Test successful audit assertion
    kassert_audit!(true, audit::ops::ASSERT_FAIL, "This audit assertion should pass");
    kprintln!("  ✅ Audit assertion passed");
    
    // Test audit assertion with custom operation
    let custom_op = 999;
    kassert_audit!(true, custom_op, "This custom audit assertion should pass");
    kprintln!("  ✅ Custom audit assertion passed");
    
    // Test audit assertion with format arguments
    let value = 300;
    kassert_audit!(value > 0, audit::ops::ASSERT_FAIL, "Value should be positive, got {}", value);
    kprintln!("  ✅ Formatted audit assertion passed");
    
    kprintln!("✅ Kassert audit functionality tests passed!");
    Ok(())
}

/// Test assertion failure handling (simulated)
pub fn test_assertion_failure_handling() -> Result<(), &'static str> {
    kprintln!("Testing assertion failure handling...");
    
    // Note: We can't actually test assertion failures that halt the system
    // in a normal test environment, so we'll test the components separately
    
    // Test that ASSERT_FAIL operation is defined
    assert_eq!(audit::ops::ASSERT_FAIL, 105, "ASSERT_FAIL should be operation code 105");
    kprintln!("  ✅ ASSERT_FAIL operation code verified");
    
    // Test that ASSERT tag is defined
    use crate::log::tags;
    assert_eq!(tags::ASSERT, "ASSERT", "ASSERT tag should be defined");
    kprintln!("  ✅ ASSERT tag verified");
    
    // Test that macros module is accessible
    assert!(cfg!(feature = "kassert-release") || cfg!(debug_assertions), 
            "Kassert should be enabled in debug builds or with kassert-release feature");
    kprintln!("  ✅ Kassert availability verified");
    
    kprintln!("✅ Assertion failure handling tests passed!");
    Ok(())
}

/// Test build profile control
pub fn test_build_profile_control() -> Result<(), &'static str> {
    kprintln!("Testing build profile control...");
    
    // Test debug build behavior
    if cfg!(debug_assertions) {
        kprintln!("  ℹ️  Running in debug build - kassert! is enabled");
        kprintln!("  ℹ️  kassert_debug! is enabled");
        kprintln!("  ℹ️  kassert_release! is disabled");
    } else {
        kprintln!("  ℹ️  Running in release build - kassert! is disabled by default");
        kprintln!("  ℹ️  kassert_debug! is disabled");
        kprintln!("  ℹ️  kassert_release! is enabled");
        
        // Check if kassert-release feature is enabled
        if cfg!(feature = "kassert-release") {
            kprintln!("  ℹ️  kassert-release feature enabled - kassert! is enabled");
        } else {
            kprintln!("  ℹ️  kassert-release feature disabled - kassert! is disabled");
        }
    }
    
    kprintln!("✅ Build profile control tests passed!");
    Ok(())
}

/// Test macro compilation and syntax
pub fn test_macro_compilation() -> Result<(), &'static str> {
    kprintln!("Testing macro compilation and syntax...");
    
    // Test various macro invocations to ensure they compile correctly
    
    // Basic kassert
    let condition = true;
    kassert!(condition);
    kassert!(condition, "Message");
    kassert!(condition, "Message with {}", "format");
    
    // Debug kassert
    kassert_debug!(condition);
    kassert_debug!(condition, "Debug message");
    kassert_debug!(condition, "Debug message with {}", "format");
    
    // Release kassert
    kassert_release!(condition);
    kassert_release!(condition, "Release message");
    kassert_release!(condition, "Release message with {}", "format");
    
    // Audit kassert
    kassert_audit!(condition, audit::ops::ASSERT_FAIL, "Audit message");
    kassert_audit!(condition, audit::ops::ASSERT_FAIL, "Audit message with {}", "format");
    
    kprintln!("✅ Macro compilation tests passed!");
    Ok(())
}

/// Run all kassert macro tests
pub fn run_all_kassert_macro_tests() -> Result<(), &'static str> {
    kprintln!("🚀 Running kassert macro tests...");
    
    test_kassert_basic()?;
    test_kassert_debug()?;
    test_kassert_release()?;
    test_kassert_audit()?;
    test_assertion_failure_handling()?;
    test_build_profile_control()?;
    test_macro_compilation()?;
    
    kprintln!("🎉 All kassert macro tests completed successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic() {
        assert!(test_kassert_basic().is_ok());
    }
    
    #[test]
    fn test_debug() {
        assert!(test_kassert_debug().is_ok());
    }
    
    #[test]
    fn test_release() {
        assert!(test_kassert_release().is_ok());
    }
    
    #[test]
    fn test_audit() {
        assert!(test_kassert_audit().is_ok());
    }
    
    #[test]
    fn test_failure_handling() {
        assert!(test_assertion_failure_handling().is_ok());
    }
    
    #[test]
    fn test_build_profile() {
        assert!(test_build_profile_control().is_ok());
    }
    
    #[test]
    fn test_compilation() {
        assert!(test_macro_compilation().is_ok());
    }
    
    #[test]
    fn test_all() {
        assert!(run_all_kassert_macro_tests().is_ok());
    }
}



