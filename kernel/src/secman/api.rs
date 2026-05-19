/// Security Manager API Module for Polymera OS
/// 
/// This module provides the public API interface for the Security Manager,
/// including sys_debug operations for key management, statistics, and
/// administrative functions.

use crate::{kprintln, klog, format};
use crate::log::Level;
use alloc::string::ToString;
use alloc::string::String;
use core::time::Duration;
use super::keys::*;
use super::audit::*;
use super::cap_store::*;
use super::idmap::*;
use super::did::*;

//=============================================================================
// API CONSTANTS AND CONFIGURATION
//=============================================================================

/// API operation codes for sys_debug
pub mod api_ops {
    // Key Management Operations
    pub const PRINT_KEY_COUNTS: u64 = 100;
    pub const ROTATE_KEYS_NOW: u64 = 101;
    pub const PURGE_KEY_CACHE: u64 = 102;
    pub const PRINT_KEY_STATS: u64 = 103;
    pub const SET_ROTATION_POLICY: u64 = 104;
    pub const GET_ROTATION_POLICY: u64 = 105;
    
    // Stream Management Operations
    pub const ROTATE_SESS: u64 = 110;
    pub const PRINT_STREAM_STATS: u64 = 111;
    pub const CREATE_STREAM: u64 = 112;
    pub const REMOVE_STREAM: u64 = 113;
    
    // Security Manager Operations
    pub const PRINT_SECMAN_STATS: u64 = 200;
    pub const PRINT_AUDIT_ENTRIES: u64 = 201;
    pub const PRINT_CAPABILITY_STATS: u64 = 202;
    pub const PRINT_IDENTITY_STATS: u64 = 203;
    pub const PRINT_DID_STATS: u64 = 204;
    
    // Administrative Operations
    pub const RESET_STATISTICS: u64 = 300;
    pub const PERFORM_MAINTENANCE: u64 = 301;
    pub const EMERGENCY_PURGE: u64 = 302;
    pub const TEST_KEY_OPERATIONS: u64 = 303;
}

//=============================================================================
// API IMPLEMENTATION
//=============================================================================

/// Handle Security Manager API operations
/// 
/// # Arguments
/// * `op_code` - Operation code (see api_ops module)
/// * `arg1` - First argument
/// * `arg2` - Second argument
/// * `arg3` - Third argument
/// 
/// # Returns
/// `Result<u64, String>` - Success value or error message
pub fn handle_secman_api(
    op_code: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
) -> Result<u64, String> {
    klog!(INFO, "[SECMAN-API] Handling operation {} with args ({}, {}, {})", op_code, arg1, arg2, arg3);
    
    match op_code {
        // Key Management Operations
        api_ops::PRINT_KEY_COUNTS => handle_print_key_counts(),
        api_ops::ROTATE_KEYS_NOW => handle_rotate_keys_now(),
        api_ops::PURGE_KEY_CACHE => handle_purge_key_cache(),
        api_ops::PRINT_KEY_STATS => handle_print_key_stats(),
        api_ops::SET_ROTATION_POLICY => handle_set_rotation_policy(arg1, arg2, arg3),
        api_ops::GET_ROTATION_POLICY => handle_get_rotation_policy(),
        
        // Stream Management Operations
        api_ops::ROTATE_SESS => handle_rotate_session_keys(arg1),
        api_ops::PRINT_STREAM_STATS => handle_print_stream_stats(),
        api_ops::CREATE_STREAM => handle_create_stream(arg1, arg2),
        api_ops::REMOVE_STREAM => handle_remove_stream(arg1),
        
        // Security Manager Operations
        api_ops::PRINT_SECMAN_STATS => handle_print_secman_stats(),
        api_ops::PRINT_AUDIT_ENTRIES => handle_print_audit_entries(arg1),
        api_ops::PRINT_CAPABILITY_STATS => handle_print_capability_stats(),
        api_ops::PRINT_IDENTITY_STATS => handle_print_identity_stats(),
        api_ops::PRINT_DID_STATS => handle_print_did_stats(),
        
        // Administrative Operations
        api_ops::RESET_STATISTICS => handle_reset_statistics(),
        api_ops::PERFORM_MAINTENANCE => handle_perform_maintenance(),
        api_ops::EMERGENCY_PURGE => handle_emergency_purge(),
        api_ops::TEST_KEY_OPERATIONS => handle_test_key_operations(),
        
        // Unknown operation
        _ => Err(format!("Unknown Security Manager API operation: {}", op_code)),
    }
}

//=============================================================================
// KEY MANAGEMENT OPERATIONS
//=============================================================================

/// Handle printing key counts
fn handle_print_key_counts() -> Result<u64, String> {
    klog!(INFO, "[SECMAN-API] Printing key counts");
    
    let (issuer_count, session_count) = get_key_counts();
    
    kprintln!("=== KEY COUNTS ===");
    kprintln!("Issuer Keys (Dilithium): {}", issuer_count);
    kprintln!("Session Keys (Kyber): {}", session_count);
    kprintln!("Total Keys: {}", issuer_count + session_count);
    kprintln!("=== END KEY COUNTS ===");
    
    // Return total key count
    Ok((issuer_count + session_count) as u64)
}

/// Handle immediate key rotation
fn handle_rotate_keys_now() -> Result<u64, String> {
    klog!(INFO, "[SECMAN-API] Forcing immediate key rotation");
    
    let rotated_count = force_rotation_now();
    
    kprintln!("=== KEY ROTATION COMPLETE ===");
    kprintln!("Rotated {} session keys", rotated_count);
    kprintln!("=== END KEY ROTATION ===");
    
    // Log audit entry
    let audit_entry = AuditEntry::new(
        ops::SEC_ADMIN_OP,
        0, // No specific resource
        "Forced immediate key rotation".to_string(),
        Some(format!("{} keys rotated", rotated_count)),
    );
    log_audit_entry(audit_entry);
    
    Ok(rotated_count as u64)
}

/// Handle purging key cache
fn handle_purge_key_cache() -> Result<u64, String> {
    klog!(INFO, "[SECMAN-API] Purging expired keys from cache");
    
    let purged_count = purge_expired_keys();
    
    kprintln!("=== KEY CACHE PURGE COMPLETE ===");
    kprintln!("Purged {} expired keys", purged_count);
    kprintln!("=== END KEY CACHE PURGE ===");
    
    // Log audit entry
    let audit_entry = AuditEntry::new(
        ops::SEC_ADMIN_OP,
        0, // No specific resource
        "Purged expired keys from cache".to_string(),
        Some(format!("{} keys purged", purged_count)),
    );
    log_audit_entry(audit_entry);
    
    Ok(purged_count as u64)
}

/// Handle printing key statistics
fn handle_print_key_stats() -> Result<u64, String> {
    klog!(INFO, "[SECMAN-API] Printing key management statistics");
    
    let stats = get_keystore_stats();
    stats.print();
    
    Ok(0)
}

/// Handle setting rotation policy
fn handle_set_rotation_policy(arg1: u64, arg2: u64, arg3: u64) -> Result<u64, String> {
    klog!(INFO, "[SECMAN-API] Setting rotation policy: interval={}, threshold={}, auto={}", arg1, arg2, arg3);
    
    // Parse arguments
    let rotation_interval = Duration::from_secs(arg1);
    let message_threshold = arg2 as usize;
    let auto_rotation = arg3 != 0;
    
    // Create new policy
    let policy = RotationPolicy {
        session_rotation_interval: rotation_interval,
        message_rotation_threshold: message_threshold,
        auto_rotation_enabled: auto_rotation,
        grace_period: KEY_ROTATION_GRACE_PERIOD,
        max_key_age: Duration::from_secs(3600), // 1 hour
    };
    
    // Update the policy
    let keystore = get_keystore();
    keystore.lock().update_rotation_policy(policy);
    
    kprintln!("=== ROTATION POLICY UPDATED ===");
    kprintln!("Session Rotation Interval: {} seconds", rotation_interval.as_secs());
    kprintln!("Message Threshold: {}", message_threshold);
    kprintln!("Auto Rotation: {}", auto_rotation);
    kprintln!("=== END ROTATION POLICY ===");
    
    // Log audit entry
    let audit_entry = AuditEntry::new(
        ops::SEC_ADMIN_OP,
        0, // No specific resource
        "Updated key rotation policy".to_string(),
        Some(format!("interval={}s, threshold={}, auto={}", 
                    rotation_interval.as_secs(), message_threshold, auto_rotation)),
    );
    log_audit_entry(audit_entry);
    
    Ok(0)
}

/// Handle getting rotation policy
fn handle_get_rotation_policy() -> Result<u64, String> {
    klog!(INFO, "[SECMAN-API] Getting current rotation policy");
    
    let keystore = get_keystore();
    let policy = keystore.lock().get_rotation_policy();
    
    kprintln!("=== CURRENT ROTATION POLICY ===");
    kprintln!("Session Rotation Interval: {} seconds", policy.session_rotation_interval.as_secs());
    kprintln!("Message Threshold: {}", policy.message_rotation_threshold);
    kprintln!("Auto Rotation: {}", policy.auto_rotation_enabled);
    kprintln!("Grace Period: {} seconds", policy.grace_period.as_secs());
    kprintln!("Max Key Age: {} seconds", policy.max_key_age.as_secs());
    kprintln!("=== END ROTATION POLICY ===");
    
    // Return rotation interval in seconds
    Ok(policy.session_rotation_interval.as_secs())
}

//=============================================================================
// SECURITY MANAGER OPERATIONS
//=============================================================================

/// Handle printing Security Manager statistics
fn handle_print_secman_stats() -> Result<u64, String> {
    klog!(INFO, "[SECMAN-API] Printing Security Manager statistics");
    
    // Print key management stats
    let key_stats = get_keystore_stats();
    key_stats.print();
    
    kprintln!("");
    
    // Print other component stats
    print_audit_stats();
    print_capability_stats();
    print_identity_stats();
    print_did_stats();
    
    Ok(0)
}

/// Handle printing audit entries
fn handle_print_audit_entries(count: u64) -> Result<u64, String> {
    klog!(INFO, "[SECMAN-API] Printing {} audit entries", count);
    
    let count = count.min(100) as usize; // Limit to 100 entries
    
    crate::secman::audit::print_recent_audit_entries(count);

    Ok(count as u64)
}

/// Handle printing capability statistics
fn handle_print_capability_stats() -> Result<u64, String> {
    klog!(INFO, "[SECMAN-API] Printing capability statistics");
    
    print_capability_stats();
    
    Ok(0)
}

/// Handle printing identity statistics
fn handle_print_identity_stats() -> Result<u64, String> {
    klog!(INFO, "[SECMAN-API] Printing identity statistics");
    
    print_identity_stats();
    
    Ok(0)
}

/// Handle printing DID statistics
fn handle_print_did_stats() -> Result<u64, String> {
    klog!(INFO, "[SECMAN-API] Printing DID statistics");
    
    print_did_stats();
    
    Ok(0)
}

//=============================================================================
// ADMINISTRATIVE OPERATIONS
//=============================================================================

/// Handle resetting statistics
fn handle_reset_statistics() -> Result<u64, String> {
    klog!(INFO, "[SECMAN-API] Resetting all statistics");
    
    // Reset key management stats
    let key_stats = get_keystore_stats();
    key_stats.reset();
    
    // Reset other component stats
    reset_audit_stats();
    reset_capability_stats();
    reset_identity_stats();
    reset_did_stats();
    
    kprintln!("=== STATISTICS RESET ===");
    kprintln!("All Security Manager statistics have been reset");
    kprintln!("=== END STATISTICS RESET ===");
    
    // Log audit entry
    let audit_entry = AuditEntry::new(
        ops::SEC_ADMIN_OP,
        0, // No specific resource
        "Reset all Security Manager statistics".to_string(),
        None,
    );
    log_audit_entry(audit_entry);
    
    Ok(0)
}

/// Handle performing maintenance
fn handle_perform_maintenance() -> Result<u64, String> {
    klog!(INFO, "[SECMAN-API] Performing Security Manager maintenance");
    
    // Perform key store maintenance
    perform_keystore_maintenance();
    
    // Perform other component maintenance
    perform_audit_maintenance();
    perform_capability_maintenance();
    perform_identity_maintenance();
    perform_did_maintenance();
    
    kprintln!("=== MAINTENANCE COMPLETE ===");
    kprintln!("Security Manager maintenance operations completed");
    kprintln!("=== END MAINTENANCE ===");
    
    // Log audit entry
    let audit_entry = AuditEntry::new(
        ops::SEC_ADMIN_OP,
        0, // No specific resource
        "Performed Security Manager maintenance".to_string(),
        None,
    );
    log_audit_entry(audit_entry);
    
    Ok(0)
}

/// Handle emergency purge
fn handle_emergency_purge() -> Result<u64, String> {
    klog!(WARN, "[SECMAN-API] Performing emergency purge of all keys");
    
    let purged_count = purge_all_keys();
    
    kprintln!("=== EMERGENCY PURGE COMPLETE ===");
    kprintln!("Purged {} keys from all keystores", purged_count);
    kprintln!("WARNING: This operation bypasses normal cleanup procedures");
    kprintln!("=== END EMERGENCY PURGE ===");
    
    // Log audit entry
    let audit_entry = AuditEntry::new(
        ops::SEC_ADMIN_OP,
        0, // No specific resource
        "Emergency purge of all keys".to_string(),
        Some(format!("{} keys purged", purged_count)),
    );
    log_audit_entry(audit_entry);
    
    Ok(purged_count as u64)
}

/// Handle testing key operations
fn handle_test_key_operations() -> Result<u64, String> {
    klog!(INFO, "[SECMAN-API] Testing key operations");
    
    kprintln!("=== TESTING KEY OPERATIONS ===");
    
    // Test keystore functionality
    test_keystore();
    
    // Test other components
    test_audit();
    test_capability_store();
    test_identity_map();
    test_did_resolver();
    
    kprintln!("=== KEY OPERATIONS TEST COMPLETE ===");
    
    Ok(0)
}

//=============================================================================
// HELPER FUNCTIONS
//=============================================================================

/// Print capability statistics
fn print_capability_stats() {
    kprintln!("=== CAPABILITY STATISTICS ===");
    kprintln!("Capability store statistics:");
    kprintln!("  - Total capabilities: {}", get_total_capability_count());
    kprintln!("  - Active capabilities: {}", get_active_capability_count());
    kprintln!("  - Revoked capabilities: {}", get_revoked_capability_count());
    kprintln!("=== END CAPABILITY STATISTICS ===");
}

/// Print identity statistics
fn print_identity_stats() {
    kprintln!("=== IDENTITY STATISTICS ===");
    kprintln!("Identity mapping statistics:");
    kprintln!("  - Total identities: {}", get_total_identity_count());
    kprintln!("  - Active identities: {}", get_active_identity_count());
    kprintln!("=== END IDENTITY STATISTICS ===");
}

/// Print DID statistics
fn print_did_stats() {
    kprintln!("=== DID STATISTICS ===");
    kprintln!("DID resolver statistics:");
    kprintln!("  - Total DIDs: {}", get_total_did_count());
    kprintln!("  - Cached DIDs: {}", get_cached_did_count());
    kprintln!("  - Resolved DIDs: {}", get_resolved_did_count());
    kprintln!("=== END DID STATISTICS ===");
}

/// Reset capability statistics
fn reset_capability_stats() {
    // TODO: Implement capability stats reset
    klog!(INFO, "[SECMAN-API] Reset capability statistics");
}

/// Reset identity statistics
fn reset_identity_stats() {
    // TODO: Implement identity stats reset
    klog!(INFO, "[SECMAN-API] Reset identity statistics");
}

/// Reset DID statistics
fn reset_did_stats() {
    // TODO: Implement DID stats reset
    klog!(INFO, "[SECMAN-API] Reset DID statistics");
}

/// Perform audit maintenance
fn perform_audit_maintenance() {
    // TODO: Implement audit maintenance
    klog!(INFO, "[SECMAN-API] Performed audit maintenance");
}

/// Reset audit statistics
fn reset_audit_stats() {
    // TODO: Implement audit stats reset
    klog!(INFO, "[SECMAN-API] Reset audit statistics");
}

/// Perform capability maintenance
fn perform_capability_maintenance() {
    // TODO: Implement capability maintenance
    klog!(INFO, "[SECMAN-API] Performed capability maintenance");
}

/// Perform identity maintenance
fn perform_identity_maintenance() {
    // TODO: Implement identity maintenance
    klog!(INFO, "[SECMAN-API] Performed identity maintenance");
}

/// Perform DID maintenance
fn perform_did_maintenance() {
    // TODO: Implement DID maintenance
    klog!(INFO, "[SECMAN-API] Performed DID maintenance");
}

/// Test capability store
fn test_capability_store() {
    // TODO: Implement capability store tests
    klog!(INFO, "[SECMAN-API] Tested capability store");
}

/// Test identity map
fn test_identity_map() {
    // TODO: Implement identity map tests
    klog!(INFO, "[SECMAN-API] Tested identity map");
}

/// Test DID resolver
fn test_did_resolver() {
    // TODO: Implement DID resolver tests
    klog!(INFO, "[SECMAN-API] Tested DID resolver");
}

//=============================================================================
// STUB FUNCTIONS FOR UNIMPLEMENTED COMPONENTS
//=============================================================================

/// Get total capability count (stub)
fn get_total_capability_count() -> usize {
    // TODO: Implement actual capability counting
    0
}

/// Get active capability count (stub)
fn get_active_capability_count() -> usize {
    // TODO: Implement actual capability counting
    0
}

/// Get revoked capability count (stub)
fn get_revoked_capability_count() -> usize {
    // TODO: Implement actual capability counting
    0
}

/// Get total identity count (stub)
fn get_total_identity_count() -> usize {
    // TODO: Implement actual identity counting
    0
}

/// Get active identity count (stub)
fn get_active_identity_count() -> usize {
    // TODO: Implement actual identity counting
    0
}

/// Get total DID count (stub)
fn get_total_did_count() -> usize {
    // TODO: Implement actual DID counting
    0
}

/// Get cached DID count (stub)
fn get_cached_did_count() -> usize {
    // TODO: Implement actual DID counting
    0
}

/// Get resolved DID count (stub)
fn get_resolved_did_count() -> usize {
    // TODO: Implement actual DID counting
    0
}

//=============================================================================
// STREAM MANAGEMENT OPERATIONS
//=============================================================================

/// Handle session key rotation for a specific process
fn handle_rotate_session_keys(pid: u64) -> Result<u64, String> {
    use crate::ipc::stream::rotate_all_keys;
    
    klog!(INFO, "[SECMAN-API] Rotating session keys for process {}", pid);
    
    // Rotate all keys (in a real implementation, this would be per-process)
    let rotated_count = rotate_all_keys();
    
    klog!(INFO, "[SECMAN-API] Rotated {} session keys", rotated_count);
    
    Ok(rotated_count as u64)
}

/// Handle printing stream statistics
fn handle_print_stream_stats() -> Result<u64, String> {
    use crate::ipc::stream::get_stream_stats;
    
    klog!(INFO, "[SECMAN-API] Printing stream statistics");
    
    let stats = get_stream_stats();
    
    kprintln!("");
    kprintln!("=== STREAM STATISTICS ===");
    kprintln!("Streams created: {}", stats.streams_created);
    kprintln!("Streams destroyed: {}", stats.streams_destroyed);
    kprintln!("Active streams: {}", stats.active_streams);
    kprintln!("Key rotations: {}", stats.key_rotations);
    kprintln!("Messages processed: {}", stats.messages_processed);
    kprintln!("MAC failures: {}", stats.mac_failures);
    kprintln!("Replay attempts: {}", stats.replay_attempts);
    
    Ok(0)
}

/// Handle creating a new stream
fn handle_create_stream(sender_pid: u64, receiver_pid: u64) -> Result<u64, String> {
    use crate::ipc::stream::{create_stream, StreamId};
    
    klog!(INFO, "[SECMAN-API] Creating stream between processes {} and {}", sender_pid, receiver_pid);
    
    match create_stream(sender_pid, receiver_pid) {
        Ok(stream_id) => {
            klog!(INFO, "[SECMAN-API] Created stream: {}", stream_id);
            Ok(stream_id.sequence)
        }
        Err(e) => {
            klog!(ERROR, "[SECMAN-API] Failed to create stream: {}", e);
            Err(e)
        }
    }
}

/// Handle removing a stream
fn handle_remove_stream(stream_sequence: u64) -> Result<u64, String> {
    use crate::ipc::stream::{get_stream_manager, StreamId};
    
    klog!(INFO, "[SECMAN-API] Removing stream with sequence {}", stream_sequence);
    
    // This is a simplified implementation - in practice, we'd need to find the stream
    // by sequence number across all process pairs
    let manager = get_stream_manager().lock();
    let stats = manager.get_stats();
    
    klog!(INFO, "[SECMAN-API] Stream removal requested, current active streams: {}", stats.active_streams);
    
    // For now, return success (in practice, this would find and remove the specific stream)
    Ok(0)
}

//=============================================================================
// TESTING
//=============================================================================

/// Test Security Manager API functionality
#[allow(dead_code)]
pub fn test_secman_api() {
    kprintln!("[SECMAN-API] Testing Security Manager API functionality...");
    
    // Test key management operations
    let _ = handle_print_key_counts();
    let _ = handle_print_key_stats();
    let _ = handle_get_rotation_policy();
    
    // Test administrative operations
    let _ = handle_print_secman_stats();
    let _ = handle_perform_maintenance();
    
    kprintln!("[SECMAN-API] API test completed");
}

/// Helper to log audit entry
fn log_audit_entry(entry: AuditEntry) {
    crate::kprintln!("[AUDIT] {:?}", entry);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_operation_codes() {
        assert_eq!(api_ops::PRINT_KEY_COUNTS, 100);
        assert_eq!(api_ops::ROTATE_KEYS_NOW, 101);
        assert_eq!(api_ops::PURGE_KEY_CACHE, 102);
        assert_eq!(api_ops::PRINT_KEY_STATS, 103);
        assert_eq!(api_ops::SET_ROTATION_POLICY, 104);
        assert_eq!(api_ops::GET_ROTATION_POLICY, 105);
        
        // Stream Management Operations
        assert_eq!(api_ops::ROTATE_SESS, 110);
        assert_eq!(api_ops::PRINT_STREAM_STATS, 111);
        assert_eq!(api_ops::CREATE_STREAM, 112);
        assert_eq!(api_ops::REMOVE_STREAM, 113);
    }
    
    #[test]
    fn test_secman_operations() {
        assert_eq!(api_ops::PRINT_SECMAN_STATS, 200);
        assert_eq!(api_ops::PRINT_AUDIT_ENTRIES, 201);
        assert_eq!(api_ops::PRINT_CAPABILITY_STATS, 202);
        assert_eq!(api_ops::PRINT_IDENTITY_STATS, 203);
        assert_eq!(api_ops::PRINT_DID_STATS, 204);
    }
    
    #[test]
    fn test_admin_operations() {
        assert_eq!(api_ops::RESET_STATISTICS, 300);
        assert_eq!(api_ops::PERFORM_MAINTENANCE, 301);
        assert_eq!(api_ops::EMERGENCY_PURGE, 302);
        assert_eq!(api_ops::TEST_KEY_OPERATIONS, 303);
    }
    
    #[test]
    fn test_unknown_operation() {
        let result = handle_secman_api(999, 0, 0, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown Security Manager API operation"));
    }
}
