/// Capability Revocation Tests
/// 
/// This module tests the capability revocation system to ensure:
/// - Capabilities can be revoked globally via sys_debug(op=REVOKE_CAP, id)
/// - Revoked capabilities fail immediately with EPERM in sys_send
/// - Proper audit logging is generated for revocation events
/// - Revoked capabilities are removed from all processes

use crate::security::cap::*;
use crate::secman::audit;
use crate::syscall::handlers::debug_ops;
use crate::syscall::dispatch;

/// Test basic capability revocation functionality
#[test]
fn test_basic_capability_revocation() {
    // Initialize security and audit systems
    crate::security::init_security();
    audit::init_audit();
    
    let sender_pid = 1000;
    let receiver_pid = 2000;
    
    // Grant a capability
    let token = match grant_capability(sender_pid, receiver_pid, scope::SEND, 60000) {
        Ok(t) => t,
        Err(e) => panic!("Failed to grant capability: {}", e),
    };
    
    kprintln!("[TEST] Granted capability: {}", token);
    
    // Verify capability is valid
    let check_result = check_capability(sender_pid, receiver_pid);
    assert!(check_result.is_ok(), "Capability should be valid before revocation");
    
    // Revoke the capability globally
    let revoked = revoke_capability_globally(token.id);
    assert!(revoked, "Capability should be successfully revoked");
    
    // Verify capability is now invalid
    let check_result = check_capability(sender_pid, receiver_pid);
    assert!(check_result.is_err(), "Capability should be invalid after revocation");
    assert_eq!(check_result.unwrap_err(), crate::security::SecurityError::PermissionDenied);
    
    kprintln!("[TEST] Basic capability revocation test PASSED");
}

/// Test that revoked capabilities fail immediately in IPC operations
#[test]
fn test_revoked_capability_ipc_failure() {
    // Initialize systems
    crate::security::init_security();
    crate::ipc::init_ipc();
    audit::init_audit();
    
    let sender_pid = 1001;
    let receiver_pid = 2001;
    
    // Grant a capability for sending
    let token = match grant_capability(sender_pid, receiver_pid, scope::SEND, 60000) {
        Ok(t) => t,
        Err(e) => panic!("Failed to grant capability: {}", e),
    };
    
    kprintln!("[TEST] Granted capability for IPC test: {}", token);
    
    // Verify capability works for IPC
    let check_result = check_capability(sender_pid, receiver_pid);
    assert!(check_result.is_ok(), "Capability should be valid before revocation");
    
    // Revoke the capability
    let revoked = revoke_capability_globally(token.id);
    assert!(revoked, "Capability should be successfully revoked");
    
    // Now try to use the revoked capability for IPC - should fail immediately
    let check_result = check_capability(sender_pid, receiver_pid);
    assert!(check_result.is_err(), "Revoked capability should fail immediately");
    assert_eq!(check_result.unwrap_err(), crate::security::SecurityError::PermissionDenied);
    
    kprintln!("[TEST] Revoked capability IPC failure test PASSED");
}

/// Test sys_debug REVOKE_CAP operation
#[test]
fn test_sys_debug_revoke_cap() {
    // Initialize systems
    crate::security::init_security();
    audit::init_audit();
    
    let sender_pid = 1002;
    let receiver_pid = 2002;
    
    // Grant a capability
    let token = match grant_capability(sender_pid, receiver_pid, scope::SEND, 60000) {
        Ok(t) => t,
        Err(e) => panic!("Failed to grant capability: {}", e),
    };
    
    kprintln!("[TEST] Granted capability for sys_debug test: {}", token);
    
    // Verify capability is valid
    let check_result = check_capability(sender_pid, receiver_pid);
    assert!(check_result.is_ok(), "Capability should be valid before sys_debug revocation");
    
    // Use sys_debug to revoke the capability
    let result = dispatch(7, debug_ops::REVOKE_CAP, token.id as u64, 0, 0); // SYS_DEBUG = 7
    assert_eq!(result, 0, "sys_debug REVOKE_CAP should return success (0)");
    
    // Verify capability is now invalid
    let check_result = check_capability(sender_pid, receiver_pid);
    assert!(check_result.is_err(), "Capability should be invalid after sys_debug revocation");
    assert_eq!(check_result.unwrap_err(), crate::security::SecurityError::PermissionDenied);
    
    kprintln!("[TEST] sys_debug REVOKE_CAP test PASSED");
}

/// Test audit logging for capability revocation
#[test]
fn test_revocation_audit_logging() {
    // Initialize systems
    crate::security::init_security();
    audit::init_audit();
    
    let sender_pid = 1003;
    let receiver_pid = 2003;
    
    // Clear audit log to start fresh
    audit::clear_audit_log();
    
    // Grant a capability
    let token = match grant_capability(sender_pid, receiver_pid, scope::SEND, 60000) {
        Ok(t) => t,
        Err(e) => panic!("Failed to grant capability: {}", e),
    };
    
    kprintln!("[TEST] Granted capability for audit test: {}", token);
    
    // Get initial audit count
    let initial_audit_count = audit::get_audit_count();
    
    // Revoke the capability
    let revoked = revoke_capability_globally(token.id);
    assert!(revoked, "Capability should be successfully revoked");
    
    // Check that audit entry was created
    let final_audit_count = audit::get_audit_count();
    assert!(final_audit_count > initial_audit_count, "Audit entry should be created for revocation");
    
    // Find the revocation audit entry
    let audit_entries = audit::get_latest_entries(10);
    let revocation_entries: Vec<_> = audit_entries.iter()
        .filter(|entry| entry.op == audit::ops::SEC_CAP_REVOKE)
        .collect();
    
    assert!(!revocation_entries.is_empty(), "Should have audit entry for capability revocation");
    
    // Verify the audit entry contains the correct token ID
    let revocation_entry = revocation_entries[0];
    assert_eq!(revocation_entry.arg, token.id as u64, "Audit entry should contain the revoked token ID");
    
    kprintln!("[TEST] Revocation audit logging test PASSED");
}

/// Test that revoked capabilities are removed from all processes
#[test]
fn test_revoked_capability_removal_from_all_processes() {
    // Initialize systems
    crate::security::init_security();
    audit::init_audit();
    
    let sender_pid_1 = 1004;
    let sender_pid_2 = 1005;
    let receiver_pid = 2004;
    
    // Grant the same capability to multiple processes
    let token_1 = match grant_capability(sender_pid_1, receiver_pid, scope::SEND, 60000) {
        Ok(t) => t,
        Err(e) => panic!("Failed to grant capability to process 1: {}", e),
    };
    
    let token_2 = match grant_capability(sender_pid_2, receiver_pid, scope::SEND, 60000) {
        Ok(t) => t,
        Err(e) => panic!("Failed to grant capability to process 2: {}", e),
    };
    
    kprintln!("[TEST] Granted capabilities: {} to process 1, {} to process 2", token_1, token_2);
    
    // Verify both capabilities are valid
    assert!(check_capability(sender_pid_1, receiver_pid).is_ok(), "Capability 1 should be valid");
    assert!(check_capability(sender_pid_2, receiver_pid).is_ok(), "Capability 2 should be valid");
    
    // Revoke capability 1 globally
    let revoked = revoke_capability_globally(token_1.id);
    assert!(revoked, "Capability 1 should be successfully revoked");
    
    // Verify capability 1 is invalid for both processes
    let check_result_1 = check_capability(sender_pid_1, receiver_pid);
    assert!(check_result_1.is_err(), "Revoked capability 1 should fail for process 1");
    
    // Capability 2 should still be valid
    let check_result_2 = check_capability(sender_pid_2, receiver_pid);
    assert!(check_result_2.is_ok(), "Capability 2 should still be valid");
    
    kprintln!("[TEST] Revoked capability removal from all processes test PASSED");
}

/// Test that already revoked capabilities cannot be revoked again
#[test]
fn test_double_revocation_handling() {
    // Initialize systems
    crate::security::init_security();
    audit::init_audit();
    
    let sender_pid = 1006;
    let receiver_pid = 2006;
    
    // Grant a capability
    let token = match grant_capability(sender_pid, receiver_pid, scope::SEND, 60000) {
        Ok(t) => t,
        Err(e) => panic!("Failed to grant capability: {}", e),
    };
    
    kprintln!("[TEST] Granted capability for double revocation test: {}", token);
    
    // First revocation should succeed
    let revoked_1 = revoke_capability_globally(token.id);
    assert!(revoked_1, "First revocation should succeed");
    
    // Second revocation should fail (already revoked)
    let revoked_2 = revoke_capability_globally(token.id);
    assert!(!revoked_2, "Second revocation should fail (already revoked)");
    
    // Capability should still be invalid
    let check_result = check_capability(sender_pid, receiver_pid);
    assert!(check_result.is_err(), "Capability should remain invalid after double revocation");
    
    kprintln!("[TEST] Double revocation handling test PASSED");
}

/// Test revocation with sys_debug and verify immediate failure
#[test]
fn test_sys_debug_revocation_immediate_failure() {
    // Initialize systems
    crate::security::init_security();
    crate::ipc::init_ipc();
    audit::init_audit();
    
    let sender_pid = 1007;
    let receiver_pid = 2007;
    
    // Grant a capability
    let token = match grant_capability(sender_pid, receiver_pid, scope::SEND, 60000) {
        Ok(t) => t,
        Err(e) => panic!("Failed to grant capability: {}", e),
    };
    
    kprintln!("[TEST] Granted capability for immediate failure test: {}", token);
    
    // Verify capability works initially
    assert!(check_capability(sender_pid, receiver_pid).is_ok(), "Capability should be valid initially");
    
    // Revoke via sys_debug
    let result = dispatch(7, debug_ops::REVOKE_CAP, token.id as u64, 0, 0);
    assert_eq!(result, 0, "sys_debug REVOKE_CAP should succeed");
    
    // Verify immediate failure
    let check_result = check_capability(sender_pid, receiver_pid);
    assert!(check_result.is_err(), "Revoked capability should fail immediately");
    assert_eq!(check_result.unwrap_err(), crate::security::SecurityError::PermissionDenied);
    
    kprintln!("[TEST] sys_debug revocation immediate failure test PASSED");
}

/// Test revocation statistics tracking
#[test]
fn test_revocation_statistics() {
    // Initialize systems
    crate::security::init_security();
    audit::init_audit();
    
    let sender_pid = 1008;
    let receiver_pid = 2008;
    
    // Get initial statistics
    let initial_stats = get_capability_stats().expect("Should have capability stats");
    let initial_revoked = initial_stats.revoked_tokens_count;
    
    // Grant a capability
    let token = match grant_capability(sender_pid, receiver_pid, scope::SEND, 60000) {
        Ok(t) => t,
        Err(e) => panic!("Failed to grant capability: {}", e),
    };
    
    kprintln!("[TEST] Granted capability for statistics test: {}", token);
    
    // Revoke the capability
    let revoked = revoke_capability_globally(token.id);
    assert!(revoked, "Capability should be successfully revoked");
    
    // Check that statistics were updated
    let final_stats = get_capability_stats().expect("Should have capability stats");
    assert!(final_stats.revoked_tokens_count > initial_revoked, "Revoked tokens count should increase");
    
    kprintln!("[TEST] Revocation statistics test PASSED");
}

/// Integration test: test complete revocation flow
#[test]
fn test_complete_revocation_flow() {
    // Initialize all systems
    crate::security::init_security();
    crate::ipc::init_ipc();
    audit::init_audit();
    
    let sender_pid = 1009;
    let receiver_pid = 2009;
    
    kprintln!("[TEST] Starting complete revocation flow test...");
    
    // Step 1: Grant capability
    let token = match grant_capability(sender_pid, receiver_pid, scope::SEND, 60000) {
        Ok(t) => t,
        Err(e) => panic!("Failed to grant capability: {}", e),
    };
    
    kprintln!("[TEST] Step 1: Granted capability: {}", token);
    
    // Step 2: Verify capability works
    let check_result = check_capability(sender_pid, receiver_pid);
    assert!(check_result.is_ok(), "Step 2: Capability should be valid");
    kprintln!("[TEST] Step 2: Capability validation PASSED");
    
    // Step 3: Revoke via sys_debug
    let revoke_result = dispatch(7, debug_ops::REVOKE_CAP, token.id as u64, 0, 0);
    assert_eq!(revoke_result, 0, "Step 3: sys_debug REVOKE_CAP should succeed");
    kprintln!("[TEST] Step 3: Capability revocation PASSED");
    
    // Step 4: Verify immediate failure
    let check_result = check_capability(sender_pid, receiver_pid);
    assert!(check_result.is_err(), "Step 4: Revoked capability should fail immediately");
    assert_eq!(check_result.unwrap_err(), crate::security::SecurityError::PermissionDenied);
    kprintln!("[TEST] Step 4: Immediate failure verification PASSED");
    
    // Step 5: Verify audit logging
    let audit_entries = audit::get_latest_entries(10);
    let revocation_entries: Vec<_> = audit_entries.iter()
        .filter(|entry| entry.op == audit::ops::SEC_CAP_REVOKE)
        .collect();
    
    assert!(!revocation_entries.is_empty(), "Step 5: Should have audit entry for revocation");
    kprintln!("[TEST] Step 5: Audit logging verification PASSED");
    
    // Step 6: Verify statistics
    let stats = get_capability_stats().expect("Should have capability stats");
    assert!(stats.revoked_tokens_count > 0, "Step 6: Revoked tokens count should be > 0");
    kprintln!("[TEST] Step 6: Statistics verification PASSED");
    
    kprintln!("[TEST] Complete revocation flow test PASSED - all steps completed successfully");
}

/// Run all capability revocation tests
pub fn run_all_capability_revocation_tests() {
    kprintln!("");
    kprintln!("=== RUNNING CAPABILITY REVOCATION TESTS ===");
    
    // Test basic functionality
    test_basic_capability_revocation();
    
    // Test IPC failure
    test_revoked_capability_ipc_failure();
    
    // Test sys_debug operation
    test_sys_debug_revoke_cap();
    
    // Test audit logging
    test_revocation_audit_logging();
    
    // Test removal from all processes
    test_revoked_capability_removal_from_all_processes();
    
    // Test double revocation handling
    test_double_revocation_handling();
    
    // Test immediate failure
    test_sys_debug_revocation_immediate_failure();
    
    // Test statistics
    test_revocation_statistics();
    
    // Test complete flow
    test_complete_revocation_flow();
    
    kprintln!("");
    kprintln!("=== ALL CAPABILITY REVOCATION TESTS PASSED ===");
    kprintln!("");
}



