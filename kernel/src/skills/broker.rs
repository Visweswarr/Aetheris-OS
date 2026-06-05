use crate::{kprintln, klog, format, vec};
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
// Note: zeroize not available in no_std, using stub
pub struct Zeroizing<T>(pub T);
impl<T> Zeroizing<T> {
    pub fn new(val: T) -> Self { Self(val) }
}
impl<T> core::ops::Deref for Zeroizing<T> {
    type Target = T;
    fn deref(&self) -> &T { &self.0 }
}

use crate::intent::schema::CapRef;
use crate::security::cap_v2::{
    CapTokenHeader, CapTokenMetadata, CapTokenSignature, CapTokenV2, SignatureAlgorithm,
};

pub const PREVIEW_SCOPE_FLAG: u64 = 1 << 30;
pub const SKILL_SCOPE_FLAG: u64 = 1 << 31;

#[derive(Debug, Clone)]
pub struct BrokerSession {
    pub skill_id: u64,
    pub caps: Vec<CapTokenV2>,
    pub preview_only: bool,
}

impl BrokerSession {
    pub fn new(skill_id: u64) -> Self {
        Self {
            skill_id,
            caps: Vec::new(),
            preview_only: true,
        }
    }
    
    pub fn add_cap(&mut self, cap: CapTokenV2) {
        self.caps.push(cap);
    }
    
    pub fn has_capability(&self, required_cap: u64) -> bool {
        self.caps.iter().any(|cap| cap.header.scope & required_cap != 0)
    }
    
    pub fn get_capabilities(&self) -> &[CapTokenV2] {
        &self.caps
    }
}

impl Drop for BrokerSession {
    fn drop(&mut self) {
        for cap in &mut self.caps {
            cap.zeroize();
        }
        self.caps.clear();
    }
}

pub struct CapabilityBroker {
    pub next_session_id: AtomicU64,
    pub active_sessions: Mutex<Vec<BrokerSession>>,
}

impl CapabilityBroker {
    pub const fn new() -> Self {
        Self {
            next_session_id: AtomicU64::new(1),
            active_sessions: Mutex::new(Vec::new()),
        }
    }
    
    pub fn create_session(&self, manifest: &crate::skills::manifest::SkillManifestV1) -> Result<BrokerSession, BrokerError> {
        let session_id = self.next_session_id.fetch_add(1, Ordering::SeqCst);
        let mut session = BrokerSession::new(session_id);
        
        for cap_ref in &manifest.requested_caps {
            match self.broker_capability(cap_ref) {
                Ok(cap_token) => {
                    session.add_cap(cap_token);
                }
                Err(e) => {
                    klog!(ERROR, "[BROKER] Failed to broker capability {:?}: {:?}", cap_ref, e);
                    return Err(BrokerError::CapabilityBrokerageFailed);
                }
            }
        }
        
        self.active_sessions.lock().push(session.clone());
        Ok(session)
    }
    
    fn broker_capability(&self, cap_ref: &CapRef) -> Result<CapTokenV2, BrokerError> {
        let scope = cap_ref.scope | PREVIEW_SCOPE_FLAG | SKILL_SCOPE_FLAG;
        let header = CapTokenHeader::new(
            self.generate_skill_nonce(),
            0,
            scope,
            crate::time::get_current_time_ms().saturating_add(60_000),
        );
        let signature = CapTokenSignature::new(SignatureAlgorithm::None, Vec::new());
        let metadata = CapTokenMetadata::new(cap_ref.cap_id, Vec::new());
        Ok(CapTokenV2::new(header, signature, metadata))
    }
    
    fn generate_skill_nonce(&self) -> u128 {
        let session_id = self.next_session_id.load(Ordering::SeqCst);
        let timestamp = crate::time::get_current_time_ms();
        ((session_id as u128) << 64) | (timestamp as u128)
    }
    
    pub fn get_session(&self, session_id: u64) -> Option<BrokerSession> {
        let sessions = self.active_sessions.lock();
        sessions.iter().find(|s| s.skill_id == session_id).cloned()
    }
    
    pub fn remove_session(&self, session_id: u64) -> bool {
        let mut sessions = self.active_sessions.lock();
        if let Some(pos) = sessions.iter().position(|s| s.skill_id == session_id) {
            let session = sessions.remove(pos);
            drop(session);
            true
        } else {
            false
        }
    }
    
    pub fn validate_policy(&self, manifest: &crate::skills::manifest::SkillManifestV1) -> Result<PolicyDecision, BrokerError> {
        if manifest.requested_caps.len() > 32 {
            return Ok(PolicyDecision::Deny("Too many capabilities requested".to_string()));
        }
        
        if manifest.mem_limit_bytes > 32 * 1024 * 1024 {
            return Ok(PolicyDecision::Deny("Memory limit too high".to_string()));
        }
        
        if manifest.time_slice_ms > 1000 {
            return Ok(PolicyDecision::Deny("Time slice too high".to_string()));
        }
        
        if !manifest.deterministic {
            return Ok(PolicyDecision::Deny("Non-deterministic skills not allowed".to_string()));
        }
        
        for hostcall in &manifest.hostcalls {
            if !self.is_hostcall_allowed(hostcall) {
                return Ok(PolicyDecision::Deny(format!("Hostcall {} not allowed", hostcall.name())));
            }
        }
        
        Ok(PolicyDecision::Allow)
    }
    
    fn is_hostcall_allowed(&self, hostcall: &crate::skills::manifest::HostcallId) -> bool {
        match hostcall {
            crate::skills::manifest::HostcallId::WmQueryReadonly => true,
            crate::skills::manifest::HostcallId::EmitPlanAction => true,
            crate::skills::manifest::HostcallId::EmitEvidence => true,
            crate::skills::manifest::HostcallId::LogDebug => true,
            crate::skills::manifest::HostcallId::RngDeterministic => true,
        }
    }
    
    pub fn get_active_sessions_count(&self) -> usize {
        self.active_sessions.lock().len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PolicyDecision {
    Allow,
    Deny(String),
    MutateScopes(Vec<CapRef>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrokerError {
    CapabilityNotFound,
    CapabilityBrokerageFailed,
    PolicyDenied,
    SessionNotFound,
    InvalidManifest,
}

impl core::fmt::Display for BrokerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BrokerError::CapabilityNotFound => write!(f, "Capability not found"),
            BrokerError::CapabilityBrokerageFailed => write!(f, "Capability brokerage failed"),
            BrokerError::PolicyDenied => write!(f, "Policy denied"),
            BrokerError::SessionNotFound => write!(f, "Session not found"),
            BrokerError::InvalidManifest => write!(f, "Invalid manifest"),
        }
    }
}
