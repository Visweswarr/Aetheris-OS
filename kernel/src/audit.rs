use crate::process::ProcessId;

/// Audit codes for system operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum AuditCode {
    // Intent-related audit codes
    INTENT_SUBMIT = 2000,
    INTENT_PREVIEW = 2001,
    INTENT_WHYLOG_TAIL = 2002,
    INTENT_STATS = 2003,
    
    // Legacy intent bus codes (from P2-AI1)
    INTENT_ACCEPT = 2004,
    INTENT_DENY = 2005,
    INTENT_OVERSIZE = 2006,
    INTENT_RATE = 2007,
    INTENT_SCHEMA = 2008,
    INTENT_POLICY_ERROR = 2009,
    INTENT_QUEUE_FULL = 2010,
    INTENT_CANCELLED = 2011,
    INTENT_FAILED = 2012,
}

impl AuditCode {
    /// Get the numeric ID for this audit code
    pub fn id(&self) -> u32 {
        *self as u32
    }
    
    /// Get the human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            // Intent kernel codes
            AuditCode::INTENT_SUBMIT => "Intent submitted",
            AuditCode::INTENT_PREVIEW => "Intent preview generated",
            AuditCode::INTENT_WHYLOG_TAIL => "Why-log tail retrieved",
            AuditCode::INTENT_STATS => "Intent statistics retrieved",
            
            // Legacy intent bus codes
            AuditCode::INTENT_ACCEPT => "Intent accepted",
            AuditCode::INTENT_DENY => "Intent denied",
            AuditCode::INTENT_OVERSIZE => "Intent payload oversized",
            AuditCode::INTENT_RATE => "Intent rate limit exceeded",
            AuditCode::INTENT_SCHEMA => "Intent schema validation failed",
            AuditCode::INTENT_POLICY_ERROR => "Intent policy error",
            AuditCode::INTENT_QUEUE_FULL => "Intent queue full",
            AuditCode::INTENT_CANCELLED => "Intent cancelled",
            AuditCode::INTENT_FAILED => "Intent failed",
        }
    }
}

/// Emit an audit event
pub fn emit(code: AuditCode, pid: ProcessId, message: &str) {
    // For now, just log the audit event
    // In a full implementation, this would write to the audit log
    crate::klog!(INFO, "[AUDIT] {}: {} - {}", code.description(), pid, message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_code_ids() {
        assert_eq!(AuditCode::INTENT_SUBMIT.id(), 2000);
        assert_eq!(AuditCode::INTENT_PREVIEW.id(), 2001);
        assert_eq!(AuditCode::INTENT_WHYLOG_TAIL.id(), 2002);
        assert_eq!(AuditCode::INTENT_STATS.id(), 2003);
    }

    #[test]
    fn test_audit_code_descriptions() {
        assert_eq!(AuditCode::INTENT_SUBMIT.description(), "Intent submitted");
        assert_eq!(AuditCode::INTENT_PREVIEW.description(), "Intent preview generated");
        assert_eq!(AuditCode::INTENT_WHYLOG_TAIL.description(), "Why-log tail retrieved");
        assert_eq!(AuditCode::INTENT_STATS.description(), "Intent statistics retrieved");
    }

    #[test]
    fn test_audit_code_equality() {
        let code1 = AuditCode::INTENT_SUBMIT;
        let code2 = AuditCode::INTENT_SUBMIT;
        let code3 = AuditCode::INTENT_PREVIEW;
        
        assert_eq!(code1, code2);
        assert_ne!(code1, code3);
    }
}
