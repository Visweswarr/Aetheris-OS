use serde::{Serialize, Deserialize};
use alloc::string::ToString;
use alloc::string::String;
use alloc::vec::Vec;

use crate::intent::schema::CapRef;

pub const SKILL_SCHEMA_VERSION: u16 = 1;
pub const SKILL_SCHEMA_HASH: &[u8; 32] = &[/* TODO: computed at build time */];

pub const SKILL_MANIFEST_MAX_SIZE: usize = 32 * 1024; // 32 KiB
pub const SKILL_NAME_MAX_LEN: usize = 64;
pub const SKILL_HOSTCALLS_MAX: usize = 16;
pub const SKILL_CAPS_MAX: usize = 32;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct SkillManifestV1 {
    #[serde(rename = "name")]
    pub name: String,
    
    #[serde(rename = "version")]
    pub version: u32,
    
    #[serde(rename = "hostcalls")]
    pub hostcalls: Vec<HostcallId>,
    
    #[serde(rename = "requested_caps")]
    pub requested_caps: Vec<CapRef>,
    
    #[serde(rename = "mem_limit_bytes")]
    pub mem_limit_bytes: u32,
    
    #[serde(rename = "time_slice_ms")]
    pub time_slice_ms: u32,
    
    #[serde(rename = "deterministic")]
    pub deterministic: bool,
    
    #[serde(rename = "entry_point")]
    pub entry_point: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(C)]
pub enum HostcallId {
    #[serde(rename = "wm_query_readonly")]
    WmQueryReadonly = 1,
    
    #[serde(rename = "emit_plan_action")]
    EmitPlanAction = 2,
    
    #[serde(rename = "emit_evidence")]
    EmitEvidence = 3,
    
    #[serde(rename = "log_debug")]
    LogDebug = 4,
    
    #[serde(rename = "rng_deterministic")]
    RngDeterministic = 5,
}

impl HostcallId {
    pub fn from_u32(id: u32) -> Option<Self> {
        match id {
            1 => Some(HostcallId::WmQueryReadonly),
            2 => Some(HostcallId::EmitPlanAction),
            3 => Some(HostcallId::EmitEvidence),
            4 => Some(HostcallId::LogDebug),
            5 => Some(HostcallId::RngDeterministic),
            _ => None,
        }
    }
    
    pub fn to_u32(&self) -> u32 {
        match self {
            HostcallId::WmQueryReadonly => 1,
            HostcallId::EmitPlanAction => 2,
            HostcallId::EmitEvidence => 3,
            HostcallId::LogDebug => 4,
            HostcallId::RngDeterministic => 5,
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            HostcallId::WmQueryReadonly => "wm_query_readonly",
            HostcallId::EmitPlanAction => "emit_plan_action",
            HostcallId::EmitEvidence => "emit_evidence",
            HostcallId::LogDebug => "log_debug",
            HostcallId::RngDeterministic => "rng_deterministic",
        }
    }
}

impl SkillManifestV1 {
    pub fn new(name: String, version: u32) -> Self {
        Self {
            name,
            version,
            hostcalls: Vec::new(),
            requested_caps: Vec::new(),
            mem_limit_bytes: 32 * 1024 * 1024, // 32 MiB default
            time_slice_ms: 250, // 250ms default
            deterministic: true,
            entry_point: "main".to_string(),
        }
    }
    
    pub fn with_hostcalls(mut self, hostcalls: Vec<HostcallId>) -> Self {
        self.hostcalls = hostcalls;
        self
    }
    
    pub fn with_capabilities(mut self, caps: Vec<CapRef>) -> Self {
        self.requested_caps = caps;
        self
    }
    
    pub fn with_memory_limit(mut self, mem_limit: u32) -> Self {
        self.mem_limit_bytes = mem_limit;
        self
    }
    
    pub fn with_time_slice(mut self, time_slice: u32) -> Self {
        self.time_slice_ms = time_slice;
        self
    }
    
    pub fn with_entry_point(mut self, entry: String) -> Self {
        self.entry_point = entry;
        self
    }
    
    pub fn validate(&self) -> Result<(), SkillManifestError> {
        if self.name.len() > SKILL_NAME_MAX_LEN {
            return Err(SkillManifestError::NameTooLong);
        }
        
        if self.hostcalls.len() > SKILL_HOSTCALLS_MAX {
            return Err(SkillManifestError::TooManyHostcalls);
        }
        
        if self.requested_caps.len() > SKILL_CAPS_MAX {
            return Err(SkillManifestError::TooManyCapabilities);
        }
        
        if self.mem_limit_bytes > 32 * 1024 * 1024 {
            return Err(SkillManifestError::MemoryLimitExceeded);
        }
        
        if self.time_slice_ms > 1000 {
            return Err(SkillManifestError::TimeSliceExceeded);
        }
        
        if !self.deterministic {
            return Err(SkillManifestError::NonDeterministicNotAllowed);
        }
        
        Ok(())
    }
    
    pub fn serialized_size(&self) -> Result<usize, serde_cbor::Error> {
        let data = serde_cbor::to_vec(self)?;
        Ok(data.len())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SkillManifestError {
    NameTooLong,
    TooManyHostcalls,
    TooManyCapabilities,
    MemoryLimitExceeded,
    TimeSliceExceeded,
    NonDeterministicNotAllowed,
    SerializationFailed,
}

impl core::fmt::Display for SkillManifestError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SkillManifestError::NameTooLong => write!(f, "Skill name too long"),
            SkillManifestError::TooManyHostcalls => write!(f, "Too many hostcalls requested"),
            SkillManifestError::TooManyCapabilities => write!(f, "Too many capabilities requested"),
            SkillManifestError::MemoryLimitExceeded => write!(f, "Memory limit exceeded"),
            SkillManifestError::TimeSliceExceeded => write!(f, "Time slice exceeded"),
            SkillManifestError::NonDeterministicNotAllowed => write!(f, "Non-deterministic skills not allowed"),
            SkillManifestError::SerializationFailed => write!(f, "Serialization failed"),
        }
    }
}

pub fn schema_hash() -> [u8; 32] {
    *SKILL_SCHEMA_HASH
}

pub const HOSTCALL_NAMES: &[(u32, &str)] = &[
    (1, "wm_query_readonly"),
    (2, "emit_plan_action"),
    (3, "emit_evidence"),
    (4, "log_debug"),
    (5, "rng_deterministic"),
];
