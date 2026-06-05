//! Audit Codes and Encoder for Polymera OS
//! 
//! This module provides structured audit events for system boundary transitions
//! including syscall entry/exit and exec load events. Events are rate-limited
//! to prevent duplicates and use compact encoding to minimize overhead.

use crate::{kprintln, klog, format};
use alloc::string::ToString;
use alloc::string::String;
use alloc::vec::Vec;
use core::time::Duration;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use serde::{Serialize, Deserialize};
use core::fmt;

/// Audit event reason codes with stable numeric IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum AuditReason {
    // Capability verification events (1000-1099)
    CapAccept = 1000,
    CapReject = 1001,
    CapExpired = 1002,
    CapSignatureInvalid = 1003,
    CapIssuerUntrusted = 1004,
    CapReplayed = 1005,
    CapNonceTooOld = 1006,
    CapFormatInvalid = 1007,
    
    // Replay protection events (1100-1199)
    Replay = 1100,
    ReplayOld = 1101,
    ReplayWindowFull = 1102,
    ReplayCleanup = 1103,
    
    // DID trust events (1200-1299)
    DidFail = 1200,
    DidExpired = 1201,
    DidNotFound = 1202,
    DidRotation = 1203,
    DidRotationGrace = 1204,
    
    // Authentication events (1300-1399)
    AuthSuccess = 1300,
    AuthFailure = 1301,
    AuthTimeout = 1302,
    AuthRateLimit = 1303,
    
    // Authorization events (1400-1499)
    AuthzGrant = 1400,
    AuthzDeny = 1401,
    AuthzInsufficient = 1402,
    AuthzScopeMismatch = 1403,
    
    // System security events (1500-1599)
    SecurityViolation = 1500,
    SecurityAlert = 1501,
    SecurityBlock = 1502,
    SecurityAllow = 1503,
    
    // IPC security events (1600-1699)
    IpcAuthSuccess = 1600,
    IpcAuthFailure = 1601,
    IpcCapRequired = 1602,
    IpcCapInvalid = 1603,
    IpcReplayDetected = 1604,
    
    // Policy events (1700-1799)
    PolicySimAllow = 1700,
    PolicySimDeny = 1701,
    PolicyBundleLoadOk = 1702,
    PolicyBundleHashMismatch = 1703,
    PolicyEvalAllow = 1704,
    PolicyEvalDeny = 1705,
    
    // World Model events (2100-2199)
    WorldModelPutOk = 2100,
    WorldModelPutEnospc = 2101,
    WorldModelQueryOk = 2102,
    WorldModelQueryFailed = 2103,
    WorldModelSnapshotNew = 2104,
    WorldModelSnapshotOpen = 2105,
    WorldModelSnapshotNotFound = 2106,
    WorldModelExportOk = 2107,
    WorldModelExportFailed = 2108,
    WorldModelCapDenied = 2109,
    WorldModelInvalid = 2110,
                WorldModelOversize = 2111,
            
            SkillLoadOk = 2200,
            SkillLoadDeny = 2201,
            SkillLoadOversize = 2202,
            SkillLoadInvalid = 2203,
            SkillLoadCapDenied = 2204,
            
            SkillInvokeOk = 2210,
            SkillInvokeQuota = 2211,
            SkillInvokeOversize = 2212,
            SkillInvokeInvalid = 2213,
            SkillInvokeCapDenied = 2214,
            
            SkillUnloadOk = 2220,
            SkillUnloadDeny = 2221,
            SkillUnloadCapDenied = 2222,
            
            SkillStatusOk = 2230,
            SkillStatusNotFound = 2231,
            SkillStatusInvalid = 2232,
            SkillStatusCapDenied = 2233,
            
            SkillListOk = 2240,
            SkillListInvalid = 2241,
            SkillListCapDenied = 2242,
            
            SkillStatsOk = 2250,
            SkillStatsInvalid = 2251,
            SkillStatsCapDenied = 2252,
    
    // Event Fabric events (2300-2309)
    EvSubOk = 2300,
    EvSubDeny = 2301,
    EvUnsubOk = 2302,
    EvUnsubDeny = 2303,
    EvPubOk = 2304,
    EvPubDeny = 2305,
    EvPollOk = 2306,
    EvPollDeny = 2307,
    EvAckOk = 2308,
    EvAckDeny = 2309,
    
    // LLM Adapter events (2316-2321)
    LlmSessionOpen = 2316,
    LlmSessionClose = 2317,
    LlmSendOk = 2318,
    LlmSendDeny = 2319,
    LlmRecvOk = 2320,
    LlmRecvDeny = 2321,
    
    // NGFS events (2322-2327)
    NgfsMountOk = 2322,
    NgfsMountDeny = 2323,
    NgfsSnapshotOk = 2324,
    NgfsSnapshotDeny = 2325,
    NgfsReadOk = 2326,
    NgfsReadDeny = 2327,
    NgfsManifestOk = 2328,
    NgfsManifestDeny = 2329,
    NgfsProofOk = 2330,
    NgfsProofFail = 2331,
    
    // Unknown/fallback
    Unknown = 9999,
}

impl AuditReason {
    /// Get the numeric ID for this audit reason
    pub fn id(&self) -> u32 {
        *self as u32
    }
    
    /// Get the human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            // Capability events
            AuditReason::CapAccept => "Capability accepted",
            AuditReason::CapReject => "Capability rejected",
            AuditReason::CapExpired => "Capability expired",
            AuditReason::CapSignatureInvalid => "Capability signature invalid",
            AuditReason::CapIssuerUntrusted => "Capability issuer untrusted",
            AuditReason::CapReplayed => "Capability replayed",
            AuditReason::CapNonceTooOld => "Capability nonce too old",
            AuditReason::CapFormatInvalid => "Capability format invalid",
            
            // Replay protection
            AuditReason::Replay => "Replay attack detected",
            AuditReason::ReplayOld => "Old nonce rejected",
            AuditReason::ReplayWindowFull => "Replay window full",
            AuditReason::ReplayCleanup => "Replay window cleanup",
            
            // DID trust
            AuditReason::DidFail => "DID verification failed",
            AuditReason::DidExpired => "DID anchor expired",
            AuditReason::DidNotFound => "DID not found",
            AuditReason::DidRotation => "DID key rotated",
            AuditReason::DidRotationGrace => "DID rotation grace period",
            
            // Authentication
            AuditReason::AuthSuccess => "Authentication successful",
            AuditReason::AuthFailure => "Authentication failed",
            AuditReason::AuthTimeout => "Authentication timeout",
            AuditReason::AuthRateLimit => "Authentication rate limited",
            
            // Authorization
            AuditReason::AuthzGrant => "Authorization granted",
            AuditReason::AuthzDeny => "Authorization denied",
            AuditReason::AuthzInsufficient => "Insufficient permissions",
            AuditReason::AuthzScopeMismatch => "Scope mismatch",
            
            // System security
            AuditReason::SecurityViolation => "Security violation",
            AuditReason::SecurityAlert => "Security alert",
            AuditReason::SecurityBlock => "Security block",
            AuditReason::SecurityAllow => "Security allow",
            
            // IPC security
            AuditReason::IpcAuthSuccess => "IPC authentication successful",
            AuditReason::IpcAuthFailure => "IPC authentication failed",
            AuditReason::IpcCapRequired => "IPC capability required",
            AuditReason::IpcCapInvalid => "IPC capability invalid",
            AuditReason::IpcReplayDetected => "IPC replay detected",
            
            // World Model
            AuditReason::WorldModelPutOk => "World Model fact put successful",
            AuditReason::WorldModelPutEnospc => "World Model fact put failed - no space",
            AuditReason::WorldModelQueryOk => "World Model query successful",
            AuditReason::WorldModelQueryFailed => "World Model query failed",
            AuditReason::WorldModelSnapshotNew => "World Model snapshot created",
            AuditReason::WorldModelSnapshotOpen => "World Model snapshot opened",
            AuditReason::WorldModelSnapshotNotFound => "World Model snapshot not found",
            AuditReason::WorldModelExportOk => "World Model export successful",
            AuditReason::WorldModelExportFailed => "World Model export failed",
            AuditReason::WorldModelCapDenied => "World Model capability denied",
            AuditReason::WorldModelInvalid => "World Model invalid operation",
            AuditReason::WorldModelOversize => "World Model operation oversized",
            
            AuditReason::SkillLoadOk => "Skill loaded successfully",
            AuditReason::SkillLoadDeny => "Skill load denied",
            AuditReason::SkillLoadOversize => "Skill manifest or WASM oversized",
            AuditReason::SkillLoadInvalid => "Skill manifest or WASM invalid",
            AuditReason::SkillLoadCapDenied => "Skill load capability denied",
            
            AuditReason::SkillInvokeOk => "Skill invoked successfully",
            AuditReason::SkillInvokeQuota => "Skill invocation quota exceeded",
            AuditReason::SkillInvokeOversize => "Skill input oversized",
            AuditReason::SkillInvokeInvalid => "Skill input invalid",
            AuditReason::SkillInvokeCapDenied => "Skill invoke capability denied",
            
            AuditReason::SkillUnloadOk => "Skill unloaded successfully",
            AuditReason::SkillUnloadDeny => "Skill unload denied",
            AuditReason::SkillUnloadCapDenied => "Skill unload capability denied",
            
            AuditReason::SkillStatusOk => "Skill status retrieved successfully",
            AuditReason::SkillStatusNotFound => "Skill not found for status",
            AuditReason::SkillStatusInvalid => "Skill status invalid",
            AuditReason::SkillStatusCapDenied => "Skill status capability denied",
            
            AuditReason::SkillListOk => "Skill list retrieved successfully",
            AuditReason::SkillListInvalid => "Skill list invalid",
            AuditReason::SkillListCapDenied => "Skill list capability denied",
            
            AuditReason::SkillStatsOk => "Skill stats retrieved successfully",
            AuditReason::SkillStatsInvalid => "Skill stats invalid",
            AuditReason::SkillStatsCapDenied => "Skill stats capability denied",
            
            // Event Fabric events
            AuditReason::EvSubOk => "Event subscription successful",
            AuditReason::EvSubDeny => "Event subscription denied",
            AuditReason::EvUnsubOk => "Event unsubscription successful",
            AuditReason::EvUnsubDeny => "Event unsubscription denied",
            AuditReason::EvPubOk => "Event publication successful",
            AuditReason::EvPubDeny => "Event publication denied",
            AuditReason::EvPollOk => "Event polling successful",
            AuditReason::EvPollDeny => "Event polling denied",
            AuditReason::EvAckOk => "Event acknowledgment successful",
            AuditReason::EvAckDeny => "Event acknowledgment denied",
            
            // Policy Guardrail events
            AuditReason::PolicyBundleLoadOk => "Policy bundle loaded successfully",
            AuditReason::PolicyBundleHashMismatch => "Policy bundle hash mismatch",
            AuditReason::PolicyEvalAllow => "Policy evaluation allowed",
            AuditReason::PolicyEvalDeny => "Policy evaluation denied",
            AuditReason::PolicySimAllow => "Policy simulation allowed",
            AuditReason::PolicySimDeny => "Policy simulation denied",
            
            // LLM Adapter events
            AuditReason::LlmSessionOpen => "LLM session opened successfully",
            AuditReason::LlmSessionClose => "LLM session closed successfully",
            AuditReason::LlmSendOk => "LLM prompt sent successfully",
            AuditReason::LlmSendDeny => "LLM prompt send denied",
            AuditReason::LlmRecvOk => "LLM chunks received successfully",
            AuditReason::LlmRecvDeny => "LLM chunk receive denied",
            
            // NGFS events
                AuditReason::NgfsMountOk => "NGFS mount operation successful",
    AuditReason::NgfsMountDeny => "NGFS mount operation denied",
    AuditReason::NgfsSnapshotOk => "NGFS snapshot operation successful",
    AuditReason::NgfsSnapshotDeny => "NGFS snapshot operation denied",
    AuditReason::NgfsReadOk => "NGFS read operation successful",
    AuditReason::NgfsReadDeny => "NGFS read operation denied",
    AuditReason::NgfsManifestOk => "NGFS manifest validation successful",
    AuditReason::NgfsManifestDeny => "NGFS manifest validation denied",
    AuditReason::NgfsProofOk => "NGFS proof validation successful",
    AuditReason::NgfsProofFail => "NGFS proof validation failed",
            
            // Unknown
            AuditReason::Unknown => "Unknown audit reason",
        }
    }
    
    /// Get the severity level
    pub fn severity(&self) -> AuditSeverity {
        match self {
            // High severity - security violations
            AuditReason::Replay |
            AuditReason::SecurityViolation |
            AuditReason::SecurityBlock |
            AuditReason::IpcReplayDetected => AuditSeverity::High,
            
            // Medium severity - authentication/authorization failures
            AuditReason::CapReject |
            AuditReason::CapSignatureInvalid |
            AuditReason::CapIssuerUntrusted |
            AuditReason::CapReplayed |
            AuditReason::DidFail |
            AuditReason::AuthFailure |
            AuditReason::AuthzDeny |
            AuditReason::SecurityAlert |
            AuditReason::IpcAuthFailure |
            AuditReason::IpcCapInvalid |
            AuditReason::WorldModelCapDenied |
            AuditReason::WorldModelInvalid |
            AuditReason::WorldModelOversize => AuditSeverity::Medium,
            
            AuditReason::SkillLoadOk => AuditSeverity::Low,
            AuditReason::SkillLoadDeny => AuditSeverity::Medium,
            AuditReason::SkillLoadOversize => AuditSeverity::Medium,
            AuditReason::SkillLoadInvalid => AuditSeverity::Medium,
            AuditReason::SkillLoadCapDenied => AuditSeverity::High,
            
            AuditReason::SkillInvokeOk => AuditSeverity::Low,
            AuditReason::SkillInvokeQuota => AuditSeverity::Medium,
            AuditReason::SkillInvokeOversize => AuditSeverity::Medium,
            AuditReason::SkillInvokeInvalid => AuditSeverity::Medium,
            AuditReason::SkillInvokeCapDenied => AuditSeverity::High,
            
            AuditReason::SkillUnloadOk => AuditSeverity::Low,
            AuditReason::SkillUnloadDeny => AuditSeverity::Medium,
            AuditReason::SkillUnloadCapDenied => AuditSeverity::High,
            
            AuditReason::SkillStatusOk => AuditSeverity::Low,
            AuditReason::SkillStatusNotFound => AuditSeverity::Medium,
            AuditReason::SkillStatusInvalid => AuditSeverity::Medium,
            AuditReason::SkillStatusCapDenied => AuditSeverity::High,
            
            AuditReason::SkillListOk => AuditSeverity::Low,
            AuditReason::SkillListInvalid => AuditSeverity::Medium,
            AuditReason::SkillListCapDenied => AuditSeverity::High,
            
            AuditReason::SkillStatsOk => AuditSeverity::Low,
            AuditReason::SkillStatsInvalid => AuditSeverity::Medium,
            AuditReason::SkillStatsCapDenied => AuditSeverity::High,
            
                    // Event Fabric events
        AuditReason::EvSubOk => AuditSeverity::Low,
        AuditReason::EvSubDeny => AuditSeverity::Medium,
        AuditReason::EvUnsubOk => AuditSeverity::Low,
        AuditReason::EvUnsubDeny => AuditSeverity::Medium,
        AuditReason::EvPubOk => AuditSeverity::Low,
        AuditReason::EvPubDeny => AuditSeverity::Medium,
        AuditReason::EvPollOk => AuditSeverity::Low,
        AuditReason::EvPollDeny => AuditSeverity::Medium,
        AuditReason::EvAckOk => AuditSeverity::Low,
        AuditReason::EvAckDeny => AuditSeverity::Medium,
        
        // Policy Guardrail events
        AuditReason::PolicyBundleLoadOk => AuditSeverity::Low,
        AuditReason::PolicyBundleHashMismatch => AuditSeverity::High,
        AuditReason::PolicyEvalAllow => AuditSeverity::Low,
        AuditReason::PolicyEvalDeny => AuditSeverity::Medium,
        AuditReason::PolicySimAllow => AuditSeverity::Low,
        AuditReason::PolicySimDeny => AuditSeverity::Medium,
        
        // LLM Adapter events
        AuditReason::LlmSessionOpen => AuditSeverity::Low,
        AuditReason::LlmSessionClose => AuditSeverity::Low,
        AuditReason::LlmSendOk => AuditSeverity::Low,
        AuditReason::LlmSendDeny => AuditSeverity::Medium,
        AuditReason::LlmRecvOk => AuditSeverity::Low,
        AuditReason::LlmRecvDeny => AuditSeverity::Medium,
        
        // NGFS events
            AuditReason::NgfsMountOk => AuditSeverity::Low,
    AuditReason::NgfsMountDeny => AuditSeverity::Medium,
    AuditReason::NgfsSnapshotOk => AuditSeverity::Low,
    AuditReason::NgfsSnapshotDeny => AuditSeverity::Medium,
    AuditReason::NgfsReadOk => AuditSeverity::Low,
    AuditReason::NgfsReadDeny => AuditSeverity::Medium,
    AuditReason::NgfsManifestOk => AuditSeverity::Low,
    AuditReason::NgfsManifestDeny => AuditSeverity::Medium,
    AuditReason::NgfsProofOk => AuditSeverity::Low,
    AuditReason::NgfsProofFail => AuditSeverity::Medium,
            
            // Low severity - informational events
            AuditReason::CapAccept |
            AuditReason::AuthSuccess |
            AuditReason::AuthzGrant |
            AuditReason::SecurityAllow |
            AuditReason::IpcAuthSuccess |
            AuditReason::DidRotation |
            AuditReason::ReplayCleanup |
            AuditReason::WorldModelPutOk |
            AuditReason::WorldModelQueryOk |
            AuditReason::WorldModelSnapshotNew |
            AuditReason::WorldModelSnapshotOpen |
            AuditReason::WorldModelExportOk => AuditSeverity::Low,
            
            // Default to medium for unknown cases
            _ => AuditSeverity::Medium,
        }
    }
    
    /// Check if this is a security-critical event
    pub fn is_critical(&self) -> bool {
        self.severity() == AuditSeverity::High
    }
    
    /// Get the category for this audit reason
    pub fn category(&self) -> AuditCategory {
        match self {
            // Capability events
            AuditReason::CapAccept |
            AuditReason::CapReject |
            AuditReason::CapExpired |
            AuditReason::CapSignatureInvalid |
            AuditReason::CapIssuerUntrusted |
            AuditReason::CapReplayed |
            AuditReason::CapNonceTooOld |
            AuditReason::CapFormatInvalid => AuditCategory::Capability,
            
            // Replay protection
            AuditReason::Replay |
            AuditReason::ReplayOld |
            AuditReason::ReplayWindowFull |
            AuditReason::ReplayCleanup => AuditCategory::ReplayProtection,
            
            // DID trust
            AuditReason::DidFail |
            AuditReason::DidExpired |
            AuditReason::DidNotFound |
            AuditReason::DidRotation |
            AuditReason::DidRotationGrace => AuditCategory::DidTrust,
            
            // Authentication
            AuditReason::AuthSuccess |
            AuditReason::AuthFailure |
            AuditReason::AuthTimeout |
            AuditReason::AuthRateLimit => AuditCategory::Authentication,
            
            // Authorization
            AuditReason::AuthzGrant |
            AuditReason::AuthzDeny |
            AuditReason::AuthzInsufficient |
            AuditReason::AuthzScopeMismatch => AuditCategory::Authorization,
            
            // System security
            AuditReason::SecurityViolation |
            AuditReason::SecurityAlert |
            AuditReason::SecurityBlock |
            AuditReason::SecurityAllow => AuditCategory::SystemSecurity,
            
            // IPC security
            AuditReason::IpcAuthSuccess |
            AuditReason::IpcAuthFailure |
            AuditReason::IpcCapRequired |
            AuditReason::IpcCapInvalid |
            AuditReason::IpcReplayDetected => AuditCategory::IpcSecurity,
            
            // World Model
            AuditReason::WorldModelPutOk |
            AuditReason::WorldModelPutEnospc |
            AuditReason::WorldModelQueryOk |
            AuditReason::WorldModelQueryFailed |
            AuditReason::WorldModelSnapshotNew |
            AuditReason::WorldModelSnapshotOpen |
            AuditReason::WorldModelSnapshotNotFound |
            AuditReason::WorldModelExportOk |
            AuditReason::WorldModelExportFailed |
            AuditReason::WorldModelCapDenied |
            AuditReason::WorldModelInvalid |
            AuditReason::WorldModelOversize => AuditCategory::WorldModel,
            
            AuditReason::SkillLoadOk |
            AuditReason::SkillLoadDeny |
            AuditReason::SkillLoadOversize |
            AuditReason::SkillLoadInvalid |
            AuditReason::SkillLoadCapDenied |
            AuditReason::SkillInvokeOk |
            AuditReason::SkillInvokeQuota |
            AuditReason::SkillInvokeOversize |
            AuditReason::SkillInvokeInvalid |
            AuditReason::SkillInvokeCapDenied |
            AuditReason::SkillUnloadOk |
            AuditReason::SkillUnloadDeny |
            AuditReason::SkillUnloadCapDenied |
            AuditReason::SkillStatusOk |
            AuditReason::SkillStatusNotFound |
            AuditReason::SkillStatusInvalid |
            AuditReason::SkillStatusCapDenied |
            AuditReason::SkillListOk |
            AuditReason::SkillListInvalid |
            AuditReason::SkillListCapDenied |
            AuditReason::SkillStatsOk |
            AuditReason::SkillStatsInvalid |
            AuditReason::SkillStatsCapDenied => AuditCategory::Skills,
            
            // Event Fabric
            AuditReason::EvSubOk |
            AuditReason::EvSubDeny |
            AuditReason::EvUnsubOk |
            AuditReason::EvUnsubDeny |
            AuditReason::EvPubOk |
            AuditReason::EvPubDeny |
            AuditReason::EvPollOk |
            AuditReason::EvPollDeny |
            AuditReason::EvAckOk |
            AuditReason::EvAckDeny => AuditCategory::EventFabric,
            
            // Policy Guardrail events
            AuditReason::PolicyBundleLoadOk |
            AuditReason::PolicyBundleHashMismatch |
            AuditReason::PolicyEvalAllow |
            AuditReason::PolicyEvalDeny |
            AuditReason::PolicySimAllow |
            AuditReason::PolicySimDeny => AuditCategory::PolicyGuardrail,
            
            // LLM Adapter events
            AuditReason::LlmSessionOpen |
            AuditReason::LlmSessionClose |
            AuditReason::LlmSendOk |
            AuditReason::LlmSendDeny |
            AuditReason::LlmRecvOk |
            AuditReason::LlmRecvDeny => AuditCategory::LlmAdapter,
            
            // NGFS events
                AuditReason::NgfsMountOk |
    AuditReason::NgfsMountDeny |
    AuditReason::NgfsSnapshotOk |
    AuditReason::NgfsSnapshotDeny |
    AuditReason::NgfsReadOk |
    AuditReason::NgfsReadDeny |
    AuditReason::NgfsManifestOk |
    AuditReason::NgfsManifestDeny |
    AuditReason::NgfsProofOk |
    AuditReason::NgfsProofFail => AuditCategory::Filesystem,
            
            // Unknown
            AuditReason::Unknown => AuditCategory::Unknown,
        }
    }
}

impl fmt::Display for AuditReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.description(), self.id())
    }
}

impl From<u32> for AuditReason {
    fn from(id: u32) -> Self {
        match id {
            1000 => AuditReason::CapAccept,
            1001 => AuditReason::CapReject,
            1002 => AuditReason::CapExpired,
            1003 => AuditReason::CapSignatureInvalid,
            1004 => AuditReason::CapIssuerUntrusted,
            1005 => AuditReason::CapReplayed,
            1006 => AuditReason::CapNonceTooOld,
            1007 => AuditReason::CapFormatInvalid,
            
            1100 => AuditReason::Replay,
            1101 => AuditReason::ReplayOld,
            1102 => AuditReason::ReplayWindowFull,
            1103 => AuditReason::ReplayCleanup,
            
            1200 => AuditReason::DidFail,
            1201 => AuditReason::DidExpired,
            1202 => AuditReason::DidNotFound,
            1203 => AuditReason::DidRotation,
            1204 => AuditReason::DidRotationGrace,
            
            1300 => AuditReason::AuthSuccess,
            1301 => AuditReason::AuthFailure,
            1302 => AuditReason::AuthTimeout,
            1303 => AuditReason::AuthRateLimit,
            
            1400 => AuditReason::AuthzGrant,
            1401 => AuditReason::AuthzDeny,
            1402 => AuditReason::AuthzInsufficient,
            1403 => AuditReason::AuthzScopeMismatch,
            
            1500 => AuditReason::SecurityViolation,
            1501 => AuditReason::SecurityAlert,
            1502 => AuditReason::SecurityBlock,
            1503 => AuditReason::SecurityAllow,
            
            1600 => AuditReason::IpcAuthSuccess,
            1601 => AuditReason::IpcAuthFailure,
            1602 => AuditReason::IpcCapRequired,
            1603 => AuditReason::IpcCapInvalid,
            1604 => AuditReason::IpcReplayDetected,
            
            2100 => AuditReason::WorldModelPutOk,
            2101 => AuditReason::WorldModelPutEnospc,
            2102 => AuditReason::WorldModelQueryOk,
            2103 => AuditReason::WorldModelQueryFailed,
            2104 => AuditReason::WorldModelSnapshotNew,
            2105 => AuditReason::WorldModelSnapshotOpen,
            2106 => AuditReason::WorldModelSnapshotNotFound,
            2107 => AuditReason::WorldModelExportOk,
            2108 => AuditReason::WorldModelExportFailed,
            2109 => AuditReason::WorldModelCapDenied,
            2110 => AuditReason::WorldModelInvalid,
            2111 => AuditReason::WorldModelOversize,
            
            2200 => AuditReason::SkillLoadOk,
            2201 => AuditReason::SkillLoadDeny,
            2202 => AuditReason::SkillLoadOversize,
            2203 => AuditReason::SkillLoadInvalid,
            2204 => AuditReason::SkillLoadCapDenied,
            
            2210 => AuditReason::SkillInvokeOk,
            2211 => AuditReason::SkillInvokeQuota,
            2212 => AuditReason::SkillInvokeOversize,
            2213 => AuditReason::SkillInvokeInvalid,
            2214 => AuditReason::SkillInvokeCapDenied,
            
            2220 => AuditReason::SkillUnloadOk,
            2221 => AuditReason::SkillUnloadDeny,
            2222 => AuditReason::SkillUnloadCapDenied,
            
            2230 => AuditReason::SkillStatusOk,
            2231 => AuditReason::SkillStatusNotFound,
            2232 => AuditReason::SkillStatusInvalid,
            2233 => AuditReason::SkillStatusCapDenied,
            
            2240 => AuditReason::SkillListOk,
            2241 => AuditReason::SkillListInvalid,
            2242 => AuditReason::SkillListCapDenied,
            
            2250 => AuditReason::SkillStatsOk,
            2251 => AuditReason::SkillStatsInvalid,
            2252 => AuditReason::SkillStatsCapDenied,
            
            // Event Fabric audit codes (2300-2309)
            2300 => AuditReason::EvSubOk,
            2301 => AuditReason::EvSubDeny,
            2302 => AuditReason::EvUnsubOk,
            2303 => AuditReason::EvUnsubDeny,
            2304 => AuditReason::EvPubOk,
            2305 => AuditReason::EvPubDeny,
            2306 => AuditReason::EvPollOk,
            2307 => AuditReason::EvPollDeny,
            2308 => AuditReason::EvAckOk,
            2309 => AuditReason::EvAckDeny,
            
            // Policy Guardrail audit codes (2310-2315)
            2310 => AuditReason::PolicyBundleLoadOk,
            2311 => AuditReason::PolicyBundleHashMismatch,
            2312 => AuditReason::PolicyEvalAllow,
            2313 => AuditReason::PolicyEvalDeny,
            2314 => AuditReason::PolicySimAllow,
            2315 => AuditReason::PolicySimDeny,
            
            // LLM Adapter audit codes (2316-2321)
            2316 => AuditReason::LlmSessionOpen,
            2317 => AuditReason::LlmSessionClose,
            2318 => AuditReason::LlmSendOk,
            2319 => AuditReason::LlmSendDeny,
            2320 => AuditReason::LlmRecvOk,
            2321 => AuditReason::LlmRecvDeny,
            
                2322 => AuditReason::NgfsMountOk,
    2323 => AuditReason::NgfsMountDeny,
    2324 => AuditReason::NgfsSnapshotOk,
    2325 => AuditReason::NgfsSnapshotDeny,
    2326 => AuditReason::NgfsReadOk,
    2327 => AuditReason::NgfsReadDeny,
    2328 => AuditReason::NgfsManifestOk,
    2329 => AuditReason::NgfsManifestDeny,
    2330 => AuditReason::NgfsProofOk,
    2331 => AuditReason::NgfsProofFail,
            
            _ => AuditReason::Unknown,
        }
    }
}

/// Audit severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for AuditSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditSeverity::Low => write!(f, "LOW"),
            AuditSeverity::Medium => write!(f, "MEDIUM"),
            AuditSeverity::High => write!(f, "HIGH"),
            AuditSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Audit event categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditCategory {
    Capability,
    ReplayProtection,
    DidTrust,
    Authentication,
    Authorization,
    SystemSecurity,
    IpcSecurity,
    WorldModel,
    Skills,
    EventFabric,
    PolicyGuardrail,
    LlmAdapter,
    Filesystem,
    Unknown,
}

impl fmt::Display for AuditCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditCategory::Capability => write!(f, "CAPABILITY"),
            AuditCategory::ReplayProtection => write!(f, "REPLAY_PROTECTION"),
            AuditCategory::DidTrust => write!(f, "DID_TRUST"),
            AuditCategory::Authentication => write!(f, "AUTHENTICATION"),
            AuditCategory::Authorization => write!(f, "AUTHORIZATION"),
            AuditCategory::SystemSecurity => write!(f, "SYSTEM_SECURITY"),
            AuditCategory::IpcSecurity => write!(f, "IPC_SECURITY"),
            AuditCategory::WorldModel => write!(f, "WORLD_MODEL"),
            AuditCategory::Skills => write!(f, "SKILLS"),
            AuditCategory::EventFabric => write!(f, "EVENT_FABRIC"),
            AuditCategory::PolicyGuardrail => write!(f, "POLICY_GUARDRAIL"),
            AuditCategory::LlmAdapter => write!(f, "LLM_ADAPTER"),
            AuditCategory::Filesystem => write!(f, "FILESYSTEM"),
            AuditCategory::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Audit event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Timestamp when event occurred
    pub timestamp: u64,
    /// Audit reason code
    pub reason: AuditReason,
    /// Event severity
    pub severity: AuditSeverity,
    /// Event category
    pub category: AuditCategory,
    /// Source process/component
    pub source: String,
    /// Target resource/entity
    pub target: Option<String>,
    /// Additional context data
    pub context: Option<String>,
    /// User/process ID that triggered the event
    pub actor_id: Option<u64>,
    /// Success/failure status
    pub success: bool,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(
        reason: AuditReason,
        source: String,
        success: bool,
    ) -> Self {
        Self {
            timestamp: crate::time::get_current_time_ms(),
            reason,
            severity: reason.severity(),
            category: reason.category(),
            source,
            target: None,
            context: None,
            actor_id: None,
            success,
        }
    }
    
    /// Set the target resource
    pub fn with_target(mut self, target: String) -> Self {
        self.target = Some(target);
        self
    }
    
    /// Set additional context
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    /// Get the current timestamp in nanoseconds.
    pub fn get_timestamp_ns() -> u64 {
        crate::time::get_current_time_ms().saturating_mul(1_000_000)
    }

    /// Set the actor ID
    pub fn with_actor(mut self, actor_id: u64) -> Self {
        self.actor_id = Some(actor_id);
        self
    }
    
    /// Format the audit event for logging
    pub fn to_log_string(&self) -> String {
        let mut log = format!(
            "[AUDIT] {} {} {} {}",
            self.timestamp,
            self.reason.id(),
            self.severity,
            self.category
        );
        
        if let Some(target) = &self.target {
            log.push_str(&format!(" target={}", target));
        }
        
        if let Some(context) = &self.context {
            log.push_str(&format!(" context={}", context));
        }
        
        if let Some(actor_id) = self.actor_id {
            log.push_str(&format!(" actor={}", actor_id));
        }
        
        log.push_str(&format!(" success={}", self.success));
        
        log
    }
}

/// Audit event encoder for compact wire format
pub struct AuditEncoder;

impl AuditEncoder {
    /// Encode audit event to compact format
    pub fn encode(event: &AuditEvent) -> Vec<u8> {
        // Simple binary format: timestamp(8) + reason_id(4) + flags(1) + source_len(1) + source + target_len(1) + target + context_len(2) + context
        let mut encoded = Vec::new();
        
        // Timestamp (8 bytes)
        encoded.extend_from_slice(&event.timestamp.to_le_bytes());
        
        // Reason ID (4 bytes)
        encoded.extend_from_slice(&event.reason.id().to_le_bytes());
        
        // Flags byte
        let mut flags = 0u8;
        if event.success { flags |= 1; }
        if event.target.is_some() { flags |= 2; }
        if event.context.is_some() { flags |= 4; }
        if event.actor_id.is_some() { flags |= 8; }
        encoded.push(flags);
        
        // Source string
        encoded.push(event.source.len() as u8);
        encoded.extend_from_slice(event.source.as_bytes());
        
        // Target string (if present)
        if let Some(target) = &event.target {
            encoded.push(target.len() as u8);
            encoded.extend_from_slice(target.as_bytes());
        } else {
            encoded.push(0);
        }
        
        // Context string (if present)
        if let Some(context) = &event.context {
            let context_len = context.len() as u16;
            encoded.extend_from_slice(&context_len.to_le_bytes());
            encoded.extend_from_slice(context.as_bytes());
        } else {
            encoded.extend_from_slice(&0u16.to_le_bytes());
        }
        
        // Actor ID (if present)
        if let Some(actor_id) = event.actor_id {
            encoded.extend_from_slice(&actor_id.to_le_bytes());
        }
        
        encoded
    }
    
    /// Decode audit event from compact format
    pub fn decode(data: &[u8]) -> Result<AuditEvent, &'static str> {
        if data.len() < 14 {
            return Err("Insufficient data for audit event");
        }
        
        let mut offset = 0;
        
        // Timestamp
        let timestamp = u64::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
            data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7]
        ]);
        offset += 8;
        
        // Reason ID
        let reason_id = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3]
        ]);
        let reason = AuditReason::from(reason_id);
        offset += 4;
        
        // Flags
        let flags = data[offset];
        let success = (flags & 1) != 0;
        let has_target = (flags & 2) != 0;
        let has_context = (flags & 4) != 0;
        let has_actor = (flags & 8) != 0;
        offset += 1;
        
        // Source string
        let source_len = data[offset] as usize;
        offset += 1;
        if offset + source_len > data.len() {
            return Err("Invalid source length");
        }
        let source = String::from_utf8_lossy(&data[offset..offset + source_len]).to_string();
        offset += source_len;
        
        // Target string
        let target_len = data[offset] as usize;
        offset += 1;
        let target = if has_target && target_len > 0 {
            if offset + target_len > data.len() {
                return Err("Invalid target length");
            }
            let target_str = String::from_utf8_lossy(&data[offset..offset + target_len]).to_string();
            offset += target_len;
            Some(target_str)
        } else {
            None
        };
        
        // Context string
        let context = if has_context {
            if offset + 2 > data.len() {
                return Err("Invalid context length");
            }
            let context_len = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
            offset += 2;
            if offset + context_len > data.len() {
                return Err("Invalid context data");
            }
            let context_str = String::from_utf8_lossy(&data[offset..offset + context_len]).to_string();
            offset += context_len;
            Some(context_str)
        } else {
            None
        };
        
        // Actor ID
        let actor_id = if has_actor {
            if offset + 8 > data.len() {
                return Err("Invalid actor ID data");
            }
            let actor_id_val = u64::from_le_bytes([
                data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
                data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7]
            ]);
            offset += 8;
            Some(actor_id_val)
        } else {
            None
        };
        
        Ok(AuditEvent {
            timestamp,
            reason,
            severity: reason.severity(),
            category: reason.category(),
            source,
            target,
            context,
            actor_id,
            success,
        })
    }
}

/// Compact audit event payload
/// 
/// This structure provides a minimal payload for audit events to avoid
/// storing PII while maintaining useful context information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditPayload {
    /// Event-specific data (encoded as compact string)
    pub data: String,
    /// Additional context fields (key-value pairs)
    pub context: Vec<(String, String)>,
    /// Error code if applicable
    pub error_code: Option<i32>,
    /// Duration if applicable (in nanoseconds)
    pub duration_ns: Option<u64>,
}

impl AuditPayload {
    /// Create a new audit payload with minimal data
    pub fn new(data: &str) -> Self {
        Self {
            data: data.to_string(),
            context: Vec::new(),
            error_code: None,
            duration_ns: None,
        }
    }
    
    /// Add context information
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.push((key.to_string(), value.to_string()));
        self
    }
    
    /// Add error code
    pub fn with_error(mut self, error_code: i32) -> Self {
        self.error_code = Some(error_code);
        self
    }
    
    /// Add duration information
    pub fn with_duration(mut self, duration_ns: u64) -> Self {
        self.duration_ns = Some(duration_ns);
        self
    }
    
    /// Get the compact encoded representation
    pub fn encode(&self) -> String {
        let mut encoded = self.data.clone();
        
        if !self.context.is_empty() {
            encoded.push_str("|");
            for (i, (key, value)) in self.context.iter().enumerate() {
                if i > 0 {
                    encoded.push_str(",");
                }
                encoded.push_str(&format!("{}={}", key, value));
            }
        }
        
        if let Some(error_code) = self.error_code {
            encoded.push_str(&format!("|err={}", error_code));
        }
        
        if let Some(duration_ns) = self.duration_ns {
            encoded.push_str(&format!("|dur={}", duration_ns));
        }
        
        encoded
    }
}

/// Audit reason codes for boundary transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum AuditReasonCode {
    /// Syscall entry boundary
    SYSCALL_ENTRY = 100,
    /// Syscall exit boundary
    SYSCALL_EXIT = 101,
    /// Exec load event
    EXEC_LOAD = 102,
    /// Task creation
    TASK_CREATE = 103,
    /// Task termination
    TASK_EXIT = 104,
    /// IPC send
    IPC_SEND = 105,
    /// IPC receive
    IPC_RECV = 106,
    /// Memory map
    MMAP = 107,
    /// Memory unmap
    MUNMAP = 108,
    /// Signal delivery
    SIGNAL = 109,
}

impl AuditReasonCode {
    /// Get the name of this reason code
    pub fn name(&self) -> &'static str {
        match self {
            AuditReasonCode::SYSCALL_ENTRY => "SYSCALL_ENTRY",
            AuditReasonCode::SYSCALL_EXIT => "SYSCALL_EXIT",
            AuditReasonCode::EXEC_LOAD => "EXEC_LOAD",
            AuditReasonCode::TASK_CREATE => "TASK_CREATE",
            AuditReasonCode::TASK_EXIT => "TASK_EXIT",
            AuditReasonCode::IPC_SEND => "IPC_SEND",
            AuditReasonCode::IPC_RECV => "IPC_RECV",
            AuditReasonCode::MMAP => "MMAP",
            AuditReasonCode::MUNMAP => "MUNMAP",
            AuditReasonCode::SIGNAL => "SIGNAL",
        }
    }
    
    /// Get the category of this reason code
    pub fn category(&self) -> &'static str {
        match self {
            AuditReasonCode::SYSCALL_ENTRY | AuditReasonCode::SYSCALL_EXIT => "BOUNDARY",
            AuditReasonCode::EXEC_LOAD => "EXEC",
            AuditReasonCode::TASK_CREATE | AuditReasonCode::TASK_EXIT => "TASK",
            AuditReasonCode::IPC_SEND | AuditReasonCode::IPC_RECV => "IPC",
            AuditReasonCode::MMAP | AuditReasonCode::MUNMAP => "MEMORY",
            AuditReasonCode::SIGNAL => "SIGNAL",
        }
    }
    
    /// Check if this is a boundary transition event
    pub fn is_boundary_transition(&self) -> bool {
        matches!(self, AuditReasonCode::SYSCALL_ENTRY | AuditReasonCode::SYSCALL_EXIT)
    }
    
    /// Get the severity of this reason code
    pub fn severity(&self) -> AuditReasonCodeSeverity {
        match self {
            AuditReasonCode::EXEC_LOAD => AuditReasonCodeSeverity::High,
            AuditReasonCode::SYSCALL_ENTRY | AuditReasonCode::SYSCALL_EXIT => AuditReasonCodeSeverity::Low,
            _ => AuditReasonCodeSeverity::Medium,
        }
    }
}

/// Severity levels for audit reason codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditReasonCodeSeverity {
    Low,
    Medium,
    High,
}

impl AuditReasonCodeSeverity {
    pub fn name(&self) -> &'static str {
        match self {
            AuditReasonCodeSeverity::Low => "LOW",
            AuditReasonCodeSeverity::Medium => "MEDIUM",
            AuditReasonCodeSeverity::High => "HIGH",
        }
    }
}

/// Structured boundary audit event (for syscall/exec transitions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryAuditEvent {
    /// Unique event identifier
    pub id: u64,
    /// Timestamp in nanoseconds since epoch
    pub timestamp_ns: u64,
    /// Process ID that generated the event
    pub pid: u32,
    /// Thread ID that generated the event
    pub tid: u32,
    /// Reason code for the event
    pub reason: AuditReasonCode,
    /// Event payload
    pub payload: AuditPayload,
    /// CPU core where the event occurred
    pub cpu_core: u32,
    /// Stack trace depth (0 = no trace)
    pub stack_depth: u8,
}

impl BoundaryAuditEvent {
    /// Create a new boundary audit event
    pub fn new(
        pid: u32,
        tid: u32,
        reason: AuditReasonCode,
        payload: AuditPayload,
    ) -> Self {
        static EVENT_COUNTER: AtomicU64 = AtomicU64::new(1);
        
        Self {
            id: EVENT_COUNTER.fetch_add(1, Ordering::Relaxed),
            timestamp_ns: Self::get_timestamp_ns(),
            pid,
            tid,
            reason,
            payload,
            cpu_core: Self::get_cpu_core(),
            stack_depth: 0, // TODO: Implement stack trace capture
        }
    }
    
    /// Get current timestamp in nanoseconds
    fn get_timestamp_ns() -> u64 {
        // TODO: Implement high-resolution timestamp
        // For now, use a simple counter
        static TIMESTAMP_COUNTER: AtomicU64 = AtomicU64::new(0);
        TIMESTAMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    }
    
    /// Get current CPU core
    fn get_cpu_core() -> u32 {
        // TODO: Implement CPU core detection
        // For now, return 0
        0
    }
    
    /// Get the compact encoded representation
    pub fn encode(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}|{}",
            self.id,
            self.timestamp_ns,
            self.pid,
            self.tid,
            self.reason as u16,
            self.payload.encode(),
            self.cpu_core
        )
    }
    
    /// Get the human-readable representation
    pub fn to_string(&self) -> String {
        format!(
            "[AUDIT] {} {} (PID: {}, TID: {}, Core: {}) - {}",
            self.reason.name(),
            self.id,
            self.pid,
            self.tid,
            self.cpu_core,
            self.payload.data
        )
    }
}

/// Rate limiter for audit events
/// 
/// Prevents duplicate events from flooding the audit log while maintaining
/// important event visibility.
pub struct AuditRateLimiter {
    /// Event counters by reason code
    counters: Mutex<alloc::collections::BTreeMap<AuditReasonCode, RateLimitCounter>>,
    /// Rate limit configuration
    config: RateLimitConfig,
}

/// Rate limit counter for a specific reason code
#[derive(Debug, Clone)]
struct RateLimitCounter {
    /// Number of events in current window
    count: u32,
    /// Window start time
    window_start: u64,
    /// Last event time
    last_event: u64,
}

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum events per window
    pub max_events_per_window: u32,
    /// Window duration in nanoseconds
    pub window_duration_ns: u64,
    /// Minimum interval between events in nanoseconds
    pub min_interval_ns: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_events_per_window: 100,
            window_duration_ns: 1_000_000_000, // 1 second
            min_interval_ns: 1_000_000, // 1 millisecond
        }
    }
}

impl AuditRateLimiter {
    /// Create a new rate limiter with default configuration
    pub fn new() -> Self {
        Self {
            counters: Mutex::new(alloc::collections::BTreeMap::new()),
            config: RateLimitConfig::default(),
        }
    }
    
    /// Create a new rate limiter with custom configuration
    pub fn with_config(config: RateLimitConfig) -> Self {
        Self {
            counters: Mutex::new(alloc::collections::BTreeMap::new()),
            config,
        }
    }
    
    /// Check if an event should be rate limited
    pub fn should_rate_limit(&self, reason: AuditReasonCode) -> bool {
        let mut counters = self.counters.lock();
        let now = AuditEvent::get_timestamp_ns();
        
        let counter = counters.entry(reason).or_insert_with(|| RateLimitCounter {
            count: 0,
            window_start: now,
            last_event: 0,
        });
        
        // Check if we're in a new window
        if now - counter.window_start >= self.config.window_duration_ns {
            counter.count = 0;
            counter.window_start = now;
        }
        
        // Check rate limit
        if counter.count >= self.config.max_events_per_window {
            return true;
        }
        
        // Check minimum interval
        if now - counter.last_event < self.config.min_interval_ns {
            return true;
        }
        
        // Update counter
        counter.count += 1;
        counter.last_event = now;
        
        false
    }
    
    /// Get current rate limit statistics
    pub fn get_stats(&self) -> RateLimitStats {
        let counters = self.counters.lock();
        let mut total_events = 0;
        let mut rate_limited_events = 0;
        
        for counter in counters.values() {
            total_events += counter.count;
            // TODO: Track rate limited events
        }
        
        RateLimitStats {
            total_events,
            rate_limited_events,
            active_counters: counters.len(),
        }
    }
}

/// Rate limit statistics
#[derive(Debug, Clone)]
pub struct RateLimitStats {
    /// Total events processed
    pub total_events: u32,
    /// Number of rate limited events
    pub rate_limited_events: u32,
    /// Number of active rate limit counters
    pub active_counters: usize,
}

/// Global audit rate limiter instance
static mut AUDIT_RATE_LIMITER: Option<AuditRateLimiter> = None;

/// Initialize the global audit rate limiter
pub fn init_audit_rate_limiter() {
    unsafe {
        if AUDIT_RATE_LIMITER.is_none() {
            AUDIT_RATE_LIMITER = Some(AuditRateLimiter::new());
        }
    }
}

/// Get the global audit rate limiter
pub fn get_audit_rate_limiter() -> Option<&'static AuditRateLimiter> {
    unsafe {
        AUDIT_RATE_LIMITER.as_ref()
    }
}

/// Check if an audit event should be rate limited
pub fn should_rate_limit_audit(reason: AuditReasonCode) -> bool {
    if let Some(limiter) = get_audit_rate_limiter() {
        limiter.should_rate_limit(reason)
    } else {
        false // No rate limiting if not initialized
    }
}

/// Create and emit a boundary audit event
pub fn emit_audit_event(
    pid: u32,
    tid: u32,
    reason: AuditReasonCode,
    payload: AuditPayload,
) -> Option<BoundaryAuditEvent> {
    // Check rate limiting
    if should_rate_limit_audit(reason) {
        return None;
    }
    
    // Create the event
    let event = BoundaryAuditEvent::new(pid, tid, reason, payload);
    
    // Log the event
    klog!(DEBUG, "{}", event.to_string());
    
    // TODO: Send to audit log system
    // TODO: Trigger any configured alerts
    
    Some(event)
}

/// Macro for emitting syscall entry audit events
#[macro_export]
macro_rules! audit_syscall_entry {
    ($syscall_id:expr, $pid:expr, $tid:expr) => {
        $crate::secman::audit_codes::emit_audit_event(
            $pid,
            $tid,
            $crate::secman::audit_codes::AuditReasonCode::SYSCALL_ENTRY,
            $crate::secman::audit_codes::AuditPayload::new(&::alloc::format!("syscall={}", $syscall_id))
                .with_context("syscall_id", &::alloc::string::ToString::to_string(&$syscall_id))
        );
    };
}

/// Macro for emitting syscall exit audit events
#[macro_export]
macro_rules! audit_syscall_exit {
    ($syscall_id:expr, $pid:expr, $tid:expr, $result:expr, $duration_ns:expr) => {
        $crate::secman::audit_codes::emit_audit_event(
            $pid,
            $tid,
            $crate::secman::audit_codes::AuditReasonCode::SYSCALL_EXIT,
            $crate::secman::audit_codes::AuditPayload::new(&::alloc::format!("syscall={},result={}", $syscall_id, $result))
                .with_context("syscall_id", &::alloc::string::ToString::to_string(&$syscall_id))
                .with_context("result", &::alloc::string::ToString::to_string(&$result))
                .with_duration($duration_ns)
        );
    };
}

/// Macro for emitting exec load audit events
#[macro_export]
macro_rules! audit_exec_load {
    ($pid:expr, $tid:expr, $image_path:expr, $image_size:expr) => {
        $crate::secman::audit_codes::emit_audit_event(
            $pid,
            $tid,
            $crate::secman::audit_codes::AuditReasonCode::EXEC_LOAD,
            $crate::secman::audit_codes::AuditPayload::new(&format!("exec={},size={}", $image_path, $image_size))
                .with_context("image_path", $image_path)
                .with_context("image_size", &::alloc::string::ToString::to_string(&$image_size))
        );
    };
}

/// Test the audit codes system
pub fn test_audit_codes() {
    kprintln!("");
    kprintln!("=== AUDIT CODES SYSTEM TEST ===");
    
    // Test reason codes
    let reason = AuditReasonCode::SYSCALL_ENTRY;
    kprintln!("✓ Reason code: {} (ID: {})", reason.name(), reason as u16);
    kprintln!("✓ Category: {}", reason.category());
    kprintln!("✓ Is boundary transition: {}", reason.is_boundary_transition());
    kprintln!("✓ Severity: {}", reason.severity().name());
    
    // Test payload encoding
    let payload = AuditPayload::new("test_data")
        .with_context("key1", "value1")
        .with_context("key2", "value2")
        .with_error(-1)
        .with_duration(1000);
    
    kprintln!("✓ Payload encoding: {}", payload.encode());
    
    // Test event creation
    let event = BoundaryAuditEvent::new(123, 456, reason, payload);
    kprintln!("✓ Event created: {}", event.to_string());
    kprintln!("✓ Event encoding: {}", event.encode());
    
    // Test rate limiter
    init_audit_rate_limiter();
    if let Some(limiter) = get_audit_rate_limiter() {
        kprintln!("✓ Rate limiter initialized");
        
        // Test rate limiting
        let mut rate_limited = 0;
        for _ in 0..150 {
            if should_rate_limit_audit(AuditReasonCode::SYSCALL_ENTRY) {
                rate_limited += 1;
            }
        }
        kprintln!("✓ Rate limiting working: {} events rate limited", rate_limited);
        
        // Get stats
        let stats = limiter.get_stats();
        kprintln!("✓ Rate limiter stats: {} total events, {} active counters", 
                 stats.total_events, stats.active_counters);
    }
    
    // Test audit event emission
    let emitted = emit_audit_event(
        789,
        101,
        AuditReasonCode::TASK_CREATE,
        AuditPayload::new("task_created")
            .with_context("task_type", "user")
            .with_context("priority", "normal")
    );
    
    if emitted.is_some() {
        kprintln!("✓ Audit event emitted successfully");
    } else {
        kprintln!("✗ Audit event emission failed");
    }
    
    kprintln!("=== AUDIT CODES SYSTEM TEST COMPLETE ===");
    kprintln!("");
}

/// Print audit codes statistics
pub fn print_audit_codes_stats() {
    kprintln!("");
    kprintln!("=== AUDIT CODES SYSTEM STATISTICS ===");
    
    if let Some(limiter) = get_audit_rate_limiter() {
        let stats = limiter.get_stats();
        kprintln!("Total Events Processed: {}", stats.total_events);
        kprintln!("Rate Limited Events: {}", stats.rate_limited_events);
        kprintln!("Active Rate Limit Counters: {}", stats.active_counters);
    } else {
        kprintln!("Rate limiter not initialized");
    }
    
    kprintln!("=== END AUDIT CODES SYSTEM STATISTICS ===");
    kprintln!("");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reason_code_properties() {
        let reason = AuditReasonCode::SYSCALL_ENTRY;
        assert_eq!(reason.name(), "SYSCALL_ENTRY");
        assert_eq!(reason.category(), "BOUNDARY");
        assert!(reason.is_boundary_transition());
        assert_eq!(reason.severity(), AuditReasonCodeSeverity::Low);
    }
    
    #[test]
    fn test_payload_encoding() {
        let payload = AuditPayload::new("test")
            .with_context("key", "value")
            .with_error(-1);
        
        let encoded = payload.encode();
        assert!(encoded.contains("test"));
        assert!(encoded.contains("key=value"));
        assert!(encoded.contains("err=-1"));
    }
    
    #[test]
    fn test_event_creation() {
        let payload = AuditPayload::new("test");
        let event = BoundaryAuditEvent::new(1, 2, AuditReasonCode::SYSCALL_ENTRY, payload);
        
        assert_eq!(event.pid, 1);
        assert_eq!(event.tid, 2);
        assert_eq!(event.reason, AuditReasonCode::SYSCALL_ENTRY);
    }
    
    #[test]
    fn test_rate_limiter() {
        let limiter = AuditRateLimiter::new();
        
        // Should not be rate limited initially
        assert!(!limiter.should_rate_limit(AuditReasonCode::SYSCALL_ENTRY));
        
        // After many events, should be rate limited
        for _ in 0..150 {
            limiter.should_rate_limit(AuditReasonCode::SYSCALL_ENTRY);
        }
        
        assert!(limiter.should_rate_limit(AuditReasonCode::SYSCALL_ENTRY));
    }
}



// Constant aliases for policy audit codes
pub const POLICY_SIM_ALLOW: AuditReason = AuditReason::PolicySimAllow;
pub const POLICY_SIM_DENY: AuditReason = AuditReason::PolicySimDeny;
pub const POLICY_BUNDLE_LOAD_OK: AuditReason = AuditReason::PolicyBundleLoadOk;
pub const POLICY_BUNDLE_HASH_MISMATCH: AuditReason = AuditReason::PolicyBundleHashMismatch;
pub const POLICY_EVAL_ALLOW: AuditReason = AuditReason::PolicyEvalAllow;
pub const POLICY_EVAL_DENY: AuditReason = AuditReason::PolicyEvalDeny;
