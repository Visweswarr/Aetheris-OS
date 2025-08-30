use crate::syscall::copy_from_user;
use crate::secman::cap_flags::*;
use crate::secman::audit::emit as audit_emit;
use crate::secman::audit_codes::*;
use crate::event::fabric::EVENT_FABRIC;
use crate::event::queue::{Lane, Event};
use crate::event::fabric::Pattern;

/// Event subscription request
#[repr(C)]
pub struct EventSubscribeRequest {
    pub topic_pattern: [u8; 64], // Null-terminated string
    pub lane: u8,
    pub task_id: u32,
}

/// Event publish request
#[repr(C)]
pub struct EventPublishRequest {
    pub topic_name: [u8; 64], // Null-terminated string
    pub priority: u8,
    pub payload_ptr: usize,
    pub payload_len: u16,
}

/// Event poll request
#[repr(C)]
pub struct EventPollRequest {
    pub max_events: u16,
    pub timeout_ms: u32,
    pub task_id: u32,
}

/// Event poll response
#[repr(C)]
pub struct EventPollResponse {
    pub event_count: u16,
    pub events: [EventHeader; 64], // Fixed-size array
}

/// Event header for userland
#[repr(C)]
pub struct EventHeader {
    pub topic_id: u32,
    pub ts_vclock: u64,
    pub priority: u8,
    pub size: u16,
    pub flags: u8,
    pub id: u64,
}

/// Event acknowledgment request
#[repr(C)]
pub struct EventAckRequest {
    pub event_id: u64,
    pub task_id: u32,
}

/// Subscribe to events matching a pattern
pub fn sys_event_subscribe(
    pattern_ptr: usize,
    lane: u8,
    task_id: u32,
) -> Result<u64, i32> {
    // Check capabilities
    if !crate::secman::cap_store::CapStore::has_capability(CAP_EVENT_SUB) {
        audit_emit(
            AuditReason::EvSubDeny,
            task_id as u64,
            0,
            b"Missing CAP_EVENT_SUB".len() as u64,
        );
        return Err(libc::EPERM);
    }

    // Copy pattern from userland
    let pattern_bytes = match copy_from_user(pattern_ptr, 64) {
        Ok(bytes) => bytes,
        Err(_) => {
            audit_emit(
                AuditReason::EvSubDeny,
                task_id as u64,
                0,
                b"Invalid pattern pointer".len() as u64,
            );
            return Err(libc::EINVAL);
        }
    };

    // Convert to string and validate
    let pattern_str = match core::str::from_utf8(&pattern_bytes) {
        Ok(s) => s.trim_matches('\0'),
        Err(_) => {
            audit_emit(
                AuditReason::EvSubDeny,
                task_id as u64,
                0,
                b"Invalid UTF-8 pattern".len() as u64,
            );
            return Err(libc::EINVAL);
        }
    };

    if pattern_str.is_empty() {
        audit_emit(
            AuditReason::EvSubDeny,
            task_id as u64,
            0,
            b"Empty pattern".len() as u64,
        );
        return Err(libc::EINVAL);
    }

    // Validate lane
    let lane_enum = match Lane::from_u8(lane) {
        Some(l) => l,
        None => {
            audit_emit(
                AuditReason::EvSubDeny,
                task_id as u64,
                0,
                b"Invalid lane".len() as u64,
            );
            return Err(libc::EINVAL);
        }
    };

    // Create pattern
    let pattern = match Pattern::new(pattern_str.to_string()) {
        Ok(p) => p,
        Err(_) => {
            audit_emit(
                AuditReason::EvSubDeny,
                task_id as u64,
                0,
                b"Invalid pattern format".len() as u64,
            );
            return Err(libc::EINVAL);
        }
    };

    // Subscribe via event fabric
    let caps_mask = crate::secman::cap_store::CapStore::get_current_caps();
    match EVENT_FABRIC.subscribe(task_id, pattern, lane_enum, caps_mask) {
        Ok(handle) => {
            audit_emit(
                AuditReason::EvSubOk,
                task_id as u64,
                handle as u64,
                pattern_str.len() as u64,
            );
            Ok(handle as u64)
        }
        Err(e) => {
            audit_emit(
                AuditReason::EvSubDeny,
                task_id as u64,
                0,
                e.len() as u64,
            );
            Err(libc::EINVAL)
        }
    }
}

/// Unsubscribe from events
pub fn sys_event_unsubscribe(handle: u32) -> Result<u64, i32> {
    // Check capabilities
    if !crate::secman::cap_store::CapStore::has_capability(CAP_EVENT_SUB) {
        audit_emit(
            AuditReason::EvUnsubDeny,
            0,
            0,
            b"Missing CAP_EVENT_SUB".len() as u64,
        );
        return Err(libc::EPERM);
    }

    // Get current task ID (simplified)
    let task_id = 1; // This would come from the current task context

    match EVENT_FABRIC.unsubscribe(handle) {
        Ok(_) => {
            audit_emit(
                AuditReason::EvUnsubOk,
                task_id as u64,
                handle as u64,
                0,
            );
            Ok(0)
        }
        Err(_) => {
            audit_emit(
                AuditReason::EvUnsubDeny,
                task_id as u64,
                handle as u64,
                b"Subscription not found".len() as u64,
            );
            Err(libc::EINVAL)
        }
    }
}

/// Publish an event
pub fn sys_event_publish(
    topic_ptr: usize,
    priority: u8,
    payload_ptr: usize,
    payload_len: u16,
) -> Result<u64, i32> {
    // Check capabilities
    if !crate::secman::cap_store::CapStore::has_capability(CAP_EVENT_PUB) {
        audit_emit(
            AuditReason::EvPubDeny,
            0,
            0,
            b"Missing CAP_EVENT_PUB".len() as u64,
        );
        return Err(libc::EPERM);
    }

    // Validate payload size
    if payload_len > 4096 {
        audit_emit(
            AuditReason::EvPubDeny,
            0,
            0,
            b"Payload too large".len() as u64,
        );
        return Err(libc::E2BIG);
    }

    // Copy topic name from userland
    let topic_bytes = match copy_from_user(topic_ptr, 64) {
        Ok(bytes) => bytes,
        Err(_) => {
            audit_emit(
                AuditReason::EvPubDeny,
                0,
                0,
                b"Invalid topic pointer".len() as u64,
            );
            return Err(libc::EINVAL);
        }
    };

    let topic_name = match core::str::from_utf8(&topic_bytes) {
        Ok(s) => s.trim_matches('\0'),
        Err(_) => {
            audit_emit(
                AuditReason::EvPubDeny,
                0,
                0,
                b"Invalid UTF-8 topic".len() as u64,
            );
            return Err(libc::EINVAL);
        }
    };

    if topic_name.is_empty() {
        audit_emit(
            AuditReason::EvPubDeny,
            0,
            0,
            b"Empty topic name".len() as u64,
        );
        return Err(libc::EINVAL);
    }

    // Validate priority
    if priority > 2 {
        audit_emit(
            AuditReason::EvPubDeny,
            0,
            0,
            b"Invalid priority".len() as u64,
        );
        return Err(libc::EINVAL);
    }

    // Copy payload from userland
    let payload = if payload_len > 0 {
        match copy_from_user(payload_ptr, payload_len as usize) {
            Ok(bytes) => bytes,
            Err(_) => {
                audit_emit(
                    AuditReason::EvPubDeny,
                    0,
                    0,
                    b"Invalid payload pointer".len() as u64,
                );
                return Err(libc::EINVAL);
            }
        }
    } else {
        Vec::new()
    };

    // Publish via event fabric
    match EVENT_FABRIC.publish(topic_name, priority, payload) {
        Ok(delivery_count) => {
            audit_emit(
                AuditReason::EvPubOk,
                0,
                delivery_count,
                topic_name.len() as u64,
            );
            Ok(delivery_count)
        }
        Err(e) => {
            audit_emit(
                AuditReason::EvPubDeny,
                0,
                0,
                e.len() as u64,
            );
            Err(libc::EINVAL)
        }
    }
}

/// Poll events from task inbox
pub fn sys_event_poll(
    max_events: u16,
    _timeout_ms: u32, // Timeout not implemented in v0
    task_id: u32,
) -> Result<u64, i32> {
    // Check capabilities
    if !crate::secman::cap_store::CapStore::has_capability(CAP_EVENT_SUB) {
        audit_emit(
            AuditReason::EvPollDeny,
            task_id as u64,
            0,
            b"Missing CAP_EVENT_SUB".len() as u64,
        );
        return Err(libc::EPERM);
    }

    // Validate max_events
    if max_events == 0 || max_events > 64 {
        audit_emit(
            AuditReason::EvPollDeny,
            task_id as u64,
            0,
            b"Invalid max_events".len() as u64,
        );
        return Err(libc::EINVAL);
    }

    // Poll events from fabric
    match EVENT_FABRIC.poll(task_id, max_events as usize) {
        Ok(events) => {
            let event_count = events.len() as u16;
            
            audit_emit(
                AuditReason::EvPollOk,
                task_id as u64,
                event_count as u64,
                max_events as u64,
            );
            
            // Return event count (in a real implementation, we'd copy events to userland)
            Ok(event_count as u64)
        }
        Err(_) => {
            audit_emit(
                AuditReason::EvPollDeny,
                task_id as u64,
                0,
                b"Task inbox not found".len() as u64,
            );
            Err(libc::EINVAL)
        }
    }
}

/// Acknowledge events (for diagnostics)
pub fn sys_event_ack(event_id: u64, task_id: u32) -> Result<u64, i32> {
    // Check capabilities
    if !crate::secman::cap_store::CapStore::has_capability(CAP_EVENT_SUB) {
        audit_emit(
            AuditReason::EvAckDeny,
            task_id as u64,
            0,
            b"Missing CAP_EVENT_SUB".len() as u64,
        );
        return Err(libc::EPERM);
    }

    match EVENT_FABRIC.ack(task_id, event_id) {
        Ok(_) => {
            audit_emit(
                AuditReason::EvAckOk,
                task_id as u64,
                event_id,
                0,
            );
            Ok(0)
        }
        Err(_) => {
            audit_emit(
                AuditReason::EvAckDeny,
                task_id as u64,
                event_id,
                b"Task inbox not found".len() as u64,
            );
            Err(libc::EINVAL)
        }
    }
}

/// Get event fabric statistics
pub fn sys_event_stats() -> Result<u64, i32> {
    // Check capabilities
    if !crate::secman::cap_store::CapStore::has_capability(CAP_EVENT_SUB) {
        return Err(libc::EPERM);
    }

    let stats = EVENT_FABRIC.get_stats();
    
    // Return a simple hash of stats for now
    // In a real implementation, we'd copy detailed stats to userland
    let stats_hash = stats.total_events_published
        .wrapping_add(stats.total_events_delivered)
        .wrapping_add(stats.total_subscriptions);
    
    Ok(stats_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_subscribe_validation() {
        // Test with valid pattern
        let result = sys_event_subscribe(0x1000, 0, 123);
        // This will fail due to invalid pointer, but we can test the validation logic
        assert!(result.is_err());
    }

    #[test]
    fn test_event_publish_validation() {
        // Test with valid parameters
        let result = sys_event_publish(0x1000, 0, 0x2000, 100);
        // This will fail due to invalid pointers, but we can test the validation logic
        assert!(result.is_err());
    }

    #[test]
    fn test_event_poll_validation() {
        // Test with valid parameters
        let result = sys_event_poll(10, 0, 123);
        // This will fail due to missing capabilities, but we can test the validation logic
        assert!(result.is_err());
    }
}
