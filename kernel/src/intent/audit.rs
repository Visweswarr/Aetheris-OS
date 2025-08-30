use crate::intent::Intent;
use crate::process::ProcessId;
use crate::secman::audit::{audit_log, AuditCode};
use alloc::string::String;

pub struct IntentAudit;

impl IntentAudit {
    pub fn new() -> Self {
        Self
    }
    
    pub fn log_accept(&self, pid: ProcessId, intent: &Intent) {
        audit_log(
            AuditCode::INTENT_ACCEPT,
            pid,
            &format!("Intent {} accepted", intent.header.intent_id)
        );
    }
    
    pub fn log_deny(&self, pid: ProcessId, intent: &Intent, reason: &str) {
        audit_log(
            AuditCode::INTENT_DENY,
            pid,
            &format!("Intent {} denied: {}", intent.header.intent_id, reason)
        );
    }
    
    pub fn log_oversize(&self, pid: ProcessId, intent: &Intent) {
        audit_log(
            AuditCode::INTENT_OVERSIZE,
            pid,
            &format!("Intent {} payload exceeds limits", intent.header.intent_id)
        );
    }
    
    pub fn log_rate_limit(&self, pid: ProcessId, intent: &Intent) {
        audit_log(
            AuditCode::INTENT_RATE,
            pid,
            &format!("Intent {} rate limit exceeded", intent.header.intent_id)
        );
    }
    
    pub fn log_schema_error(&self, pid: ProcessId, intent: &Intent) {
        audit_log(
            AuditCode::INTENT_SCHEMA,
            pid,
            &format!("Intent {} schema validation failed", intent.header.intent_id)
        );
    }
    
    pub fn log_policy_error(&self, pid: ProcessId, intent: &Intent, error: &str) {
        audit_log(
            AuditCode::INTENT_POLICY_ERROR,
            pid,
            &format!("Intent {} policy error: {}", intent.header.intent_id, error)
        );
    }
    
    pub fn log_queue_full(&self, pid: ProcessId, intent: &Intent) {
        audit_log(
            AuditCode::INTENT_QUEUE_FULL,
            pid,
            &format!("Intent {} queue full for lane {}", 
                    intent.header.intent_id, intent.header.lane)
        );
    }
    
    pub fn log_cancelled(&self, pid: ProcessId, intent: &Intent) {
        audit_log(
            AuditCode::INTENT_CANCELLED,
            pid,
            &format!("Intent {} cancelled by owner", intent.header.intent_id)
        );
    }
    
    pub fn log_failed(&self, pid: ProcessId, intent: &Intent, error: &str) {
        audit_log(
            AuditCode::INTENT_FAILED,
            pid,
            &format!("Intent {} failed: {}", intent.header.intent_id, error)
        );
    }
    
    pub fn log_running(&self, pid: ProcessId, intent: &Intent) {
        audit_log(
            AuditCode::INTENT_ACCEPT, // Reuse accept code for running
            pid,
            &format!("Intent {} started execution", intent.header.intent_id)
        );
    }
    
    pub fn log_expired(&self, pid: ProcessId, intent: &Intent) {
        audit_log(
            AuditCode::INTENT_FAILED, // Use failed code for expired
            pid,
            &format!("Intent {} expired (TTL: {}ms)", 
                    intent.header.intent_id, intent.header.ttl_ms)
        );
    }
    
    pub fn log_tool_call(&self, pid: ProcessId, intent: &Intent, tool_name: &str) {
        audit_log(
            AuditCode::INTENT_ACCEPT, // Reuse accept code for tool calls
            pid,
            &format!("Intent {} tool call: {}", intent.header.intent_id, tool_name)
        );
    }
    
    pub fn log_completed(&self, pid: ProcessId, intent: &Intent) {
        audit_log(
            AuditCode::INTENT_ACCEPT, // Reuse accept code for completion
            pid,
            &format!("Intent {} completed successfully", intent.header.intent_id)
        );
    }
    
    pub fn log_scope_mutation(&self, pid: ProcessId, intent: &Intent, additional_scopes: u64) {
        audit_log(
            AuditCode::INTENT_ACCEPT, // Reuse accept code for scope mutation
            pid,
            &format!("Intent {} scopes mutated: +0x{:x}", 
                    intent.header.intent_id, additional_scopes)
        );
    }
    
    pub fn log_lane_assignment(&self, pid: ProcessId, intent: &Intent, lane: u8) {
        audit_log(
            AuditCode::INTENT_ACCEPT, // Reuse accept code for lane assignment
            pid,
            &format!("Intent {} assigned to lane {}", intent.header.intent_id, lane)
        );
    }
    
    pub fn log_capability_check(&self, pid: ProcessId, intent: &Intent, required_caps: u64) {
        audit_log(
            AuditCode::INTENT_ACCEPT, // Reuse accept code for capability checks
            pid,
            &format!("Intent {} requires capabilities: 0x{:x}", 
                    intent.header.intent_id, required_caps)
        );
    }
}

impl Default for IntentAudit {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::wire::IntentHeaderV1;
    use crate::intent::Intent;
    
    fn create_test_intent() -> Intent {
        let mut header = IntentHeaderV1::default();
        header.intent_id = 12345;
        header.lane = 1;
        header.ttl_ms = 5000;
        
        Intent::new(
            header,
            vec![0u8; 100],
            vec![],
            vec![],
            vec![],
            456,
        )
    }
    
    #[test]
    fn test_audit_creation() {
        let audit = IntentAudit::new();
        assert!(audit is IntentAudit);
    }
    
    #[test]
    fn test_audit_default() {
        let audit = IntentAudit::default();
        assert!(audit is IntentAudit);
    }
    
    #[test]
    fn test_log_accept() {
        let audit = IntentAudit::new();
        let intent = create_test_intent();
        
        audit.log_accept(123, &intent);
    }
    
    #[test]
    fn test_log_deny() {
        let audit = IntentAudit::new();
        let intent = create_test_intent();
        
        audit.log_deny(123, &intent, "Test denial reason");
    }
    
    #[test]
    fn test_log_oversize() {
        let audit = IntentAudit::new();
        let intent = create_test_intent();
        
        audit.log_oversize(123, &intent);
    }
    
    #[test]
    fn test_log_rate_limit() {
        let audit = IntentAudit::new();
        let intent = create_test_intent();
        
        audit.log_rate_limit(123, &intent);
    }
    
    #[test]
    fn test_log_schema_error() {
        let audit = IntentAudit::new();
        let intent = create_test_intent();
        
        audit.log_schema_error(123, &intent);
    }
    
    #[test]
    fn test_log_policy_error() {
        let audit = IntentAudit::new();
        let intent = create_test_intent();
        
        audit.log_policy_error(123, &intent, "Test policy error");
    }
    
    #[test]
    fn test_log_queue_full() {
        let audit = IntentAudit::new();
        let intent = create_test_intent();
        
        audit.log_queue_full(123, &intent);
    }
    
    #[test]
    fn test_log_cancelled() {
        let audit = IntentAudit::new();
        let intent = create_test_intent();
        
        audit.log_cancelled(123, &intent);
    }
    
    #[test]
    fn test_log_failed() {
        let audit = IntentAudit::new();
        let intent = create_test_intent();
        
        audit.log_failed(123, &intent, "Test failure reason");
    }
    
    #[test]
    fn test_log_running() {
        let audit = IntentAudit::new();
        let intent = create_test_intent();
        
        audit.log_running(123, &intent);
    }
    
    #[test]
    fn test_log_expired() {
        let audit = IntentAudit::new();
        let intent = create_test_intent();
        
        audit.log_expired(123, &intent);
    }
    
    #[test]
    fn test_log_tool_call() {
        let audit = IntentAudit::new();
        let intent = create_test_intent();
        
        audit.log_tool_call(123, &intent, "test_tool");
    }
    
    #[test]
    fn test_log_completed() {
        let audit = IntentAudit::new();
        let intent = create_test_intent();
        
        audit.log_completed(123, &intent);
    }
    
    #[test]
    fn test_log_scope_mutation() {
        let audit = IntentAudit::new();
        let intent = create_test_intent();
        
        audit.log_scope_mutation(123, &intent, 0x42);
    }
    
    #[test]
    fn test_log_lane_assignment() {
        let audit = IntentAudit::new();
        let intent = create_test_intent();
        
        audit.log_lane_assignment(123, &intent, 2);
    }
    
    #[test]
    fn test_log_capability_check() {
        let audit = IntentAudit::new();
        let intent = create_test_intent();
        
        audit.log_capability_check(123, &intent, 0x100);
    }
}
