use crate::{kprintln, klog};
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

use crate::skills::manifest::SkillManifestV1;
use crate::skills::exec::Skill;
use crate::skills::broker::{CapabilityBroker, BrokerSession};

pub const MAX_LOADED_SKILLS: usize = 16;
pub const SKILL_HANDLE_MASK: u64 = 0x8000000000000000;

#[derive(Debug, Clone)]
pub struct SkillHandle {
    pub id: u64,
    pub name: String,
    pub version: u32,
    pub loaded_at: u64,
}

impl SkillHandle {
    pub fn new(id: u64, name: String, version: u32) -> Self {
        Self {
            id,
            name,
            version,
            loaded_at: crate::time::get_current_time_ms(),
        }
    }
    
    pub fn is_valid(&self) -> bool {
        self.id & SKILL_HANDLE_MASK != 0
    }
}

#[derive(Debug)]
pub struct LoadedSkill {
    pub handle: SkillHandle,
    pub skill: Skill,
    pub broker_session: BrokerSession,
    pub load_count: u32,
}

impl LoadedSkill {
    pub fn new(handle: SkillHandle, skill: Skill, broker_session: BrokerSession) -> Self {
        Self {
            handle,
            skill,
            broker_session,
            load_count: 1,
        }
    }
    
    pub fn increment_load_count(&mut self) {
        self.load_count += 1;
    }
    
    pub fn decrement_load_count(&mut self) -> bool {
        if self.load_count > 0 {
            self.load_count -= 1;
        }
        self.load_count == 0
    }
}

#[derive(Debug)]
pub struct SkillCounters {
    pub skills_loaded: AtomicU64,
    pub skills_unloaded: AtomicU64,
    pub skills_invoked: AtomicU64,
    pub skills_failed: AtomicU64,
    pub total_instructions: AtomicU64,
    pub total_memory_bytes: AtomicU64,
    pub total_execution_time_us: AtomicU64,
}

impl SkillCounters {
    pub const fn new() -> Self {
        Self {
            skills_loaded: AtomicU64::new(0),
            skills_unloaded: AtomicU64::new(0),
            skills_invoked: AtomicU64::new(0),
            skills_failed: AtomicU64::new(0),
            total_instructions: AtomicU64::new(0),
            total_memory_bytes: AtomicU64::new(0),
            total_execution_time_us: AtomicU64::new(0),
        }
    }
    
    pub fn increment_loaded(&self) {
        self.skills_loaded.fetch_add(1, Ordering::SeqCst);
    }
    
    pub fn increment_unloaded(&self) {
        self.skills_unloaded.fetch_add(1, Ordering::SeqCst);
    }
    
    pub fn increment_invoked(&self) {
        self.skills_invoked.fetch_add(1, Ordering::SeqCst);
    }
    
    pub fn increment_failed(&self) {
        self.skills_failed.fetch_add(1, Ordering::SeqCst);
    }
    
    pub fn add_instructions(&self, count: u64) {
        self.total_instructions.fetch_add(count, Ordering::SeqCst);
    }
    
    pub fn add_memory(&self, bytes: usize) {
        self.total_memory_bytes.fetch_add(bytes as u64, Ordering::SeqCst);
    }
    
    pub fn add_execution_time(&self, us: u64) {
        self.total_execution_time_us.fetch_add(us, Ordering::SeqCst);
    }
    
    pub fn get_stats(&self) -> SkillStats {
        SkillStats {
            skills_loaded: self.skills_loaded.load(Ordering::SeqCst),
            skills_unloaded: self.skills_unloaded.load(Ordering::SeqCst),
            skills_invoked: self.skills_invoked.load(Ordering::SeqCst),
            skills_failed: self.skills_failed.load(Ordering::SeqCst),
            total_instructions: self.total_instructions.load(Ordering::SeqCst),
            total_memory_bytes: self.total_memory_bytes.load(Ordering::SeqCst),
            total_execution_time_us: self.total_execution_time_us.load(Ordering::SeqCst),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SkillStats {
    pub skills_loaded: u64,
    pub skills_unloaded: u64,
    pub skills_invoked: u64,
    pub skills_failed: u64,
    pub total_instructions: u64,
    pub total_memory_bytes: u64,
    pub total_execution_time_us: u64,
}

pub struct SkillRegistry {
    pub next_skill_id: AtomicU64,
    pub loaded_skills: Mutex<Vec<LoadedSkill>>,
    pub broker: CapabilityBroker,
    pub counters: SkillCounters,
}

impl SkillRegistry {
    pub const fn new() -> Self {
        Self {
            next_skill_id: AtomicU64::new(1),
            loaded_skills: Mutex::new(Vec::new()),
            broker: CapabilityBroker::new(),
            counters: SkillCounters::new(),
        }
    }
    
    pub fn load_skill(&self, manifest: SkillManifestV1, wasm_bytes: Vec<u8>) -> Result<SkillHandle, RegistryError> {
        if self.loaded_skills.lock().len() >= MAX_LOADED_SKILLS {
            return Err(RegistryError::TooManySkills);
        }
        
        if wasm_bytes.len() > manifest.mem_limit_bytes as usize {
            return Err(RegistryError::WasmTooLarge);
        }
        
        if !manifest.validate().is_ok() {
            return Err(RegistryError::InvalidManifest);
        }
        
        let policy_decision = self.broker.validate_policy(&manifest)?;
        match policy_decision {
            crate::skills::broker::PolicyDecision::Allow => {}
            crate::skills::broker::PolicyDecision::Deny(reason) => {
                return Err(RegistryError::PolicyDenied(reason));
            }
            crate::skills::broker::PolicyDecision::MutateScopes(_) => {
                return Err(RegistryError::PolicyMutationNotSupported);
            }
        }
        
        let broker_session = self.broker.create_session(&manifest)?;
        let skill_id = self.next_skill_id.fetch_add(1, Ordering::SeqCst);
        let handle_id = SKILL_HANDLE_MASK | skill_id;
        
        let handle = SkillHandle::new(handle_id, manifest.name.clone(), manifest.version);
        let skill = Skill::new(skill_id, manifest, wasm_bytes);
        
        let loaded_skill = LoadedSkill::new(handle.clone(), skill, broker_session);
        
        {
            let mut skills = self.loaded_skills.lock();
            skills.push(loaded_skill);
        }
        
        self.counters.increment_loaded();
        
        klog!(INFO, "[REGISTRY] Loaded skill {} (v{}) with handle 0x{:X}", 
              handle.name, handle.version, handle.id);
        
        Ok(handle)
    }
    
    pub fn invoke_skill(&self, handle: u64, input: &[u8]) -> Result<crate::skills::exec::PreviewBundle, RegistryError> {
        let mut skills = self.loaded_skills.lock();
        let loaded_skill = skills.iter_mut()
            .find(|s| s.handle.id == handle)
            .ok_or(RegistryError::SkillNotFound)?;
        
        if !loaded_skill.handle.is_valid() {
            return Err(RegistryError::InvalidHandle);
        }
        
        self.counters.increment_invoked();
        
        match loaded_skill.skill.run_preview(input) {
            Ok(bundle) => {
                self.counters.add_instructions(bundle.metrics.instructions_executed);
                self.counters.add_memory(bundle.metrics.memory_used_bytes);
                self.counters.add_execution_time(bundle.metrics.execution_time_us);
                
                klog!(DEBUG, "[REGISTRY] Skill {} invoked successfully: {} instructions, {} bytes, {}μs",
                      loaded_skill.handle.name,
                      bundle.metrics.instructions_executed,
                      bundle.metrics.memory_used_bytes,
                      bundle.metrics.execution_time_us);
                
                Ok(bundle)
            }
            Err(e) => {
                self.counters.increment_failed();
                
                klog!(ERROR, "[REGISTRY] Skill {} invocation failed: {:?}",
                      loaded_skill.handle.name, e);
                
                Err(RegistryError::ExecutionFailed(e.to_string()))
            }
        }
    }
    
    pub fn unload_skill(&self, handle: u64) -> Result<(), RegistryError> {
        let mut skills = self.loaded_skills.lock();
        
        if let Some(pos) = skills.iter().position(|s| s.handle.id == handle) {
            let mut loaded_skill = skills.remove(pos);
            
            if loaded_skill.decrement_load_count() {
                self.counters.increment_unloaded();
                
                klog!(INFO, "[REGISTRY] Unloaded skill {} (v{})", 
                      loaded_skill.handle.name, loaded_skill.handle.version);
                
                Ok(())
            } else {
                skills.push(loaded_skill);
                Err(RegistryError::SkillStillReferenced)
            }
        } else {
            Err(RegistryError::SkillNotFound)
        }
    }
    
    pub fn get_loaded_skills_count(&self) -> usize {
        self.loaded_skills.lock().len()
    }
    
    pub fn get_skill_info(&self, handle: u64) -> Option<SkillInfo> {
        let skills = self.loaded_skills.lock();
        
        for loaded_skill in skills.iter() {
            if loaded_skill.handle.id == handle {
                return Some(SkillInfo {
                    name: loaded_skill.handle.name.clone(),
                    version: loaded_skill.handle.version,
                    loaded_at: loaded_skill.handle.loaded_at,
                    load_count: loaded_skill.load_count,
                    memory_limit: loaded_skill.skill.manifest.mem_limit_bytes,
                    time_slice: loaded_skill.skill.manifest.time_slice_ms,
                });
            }
        }
        
        None
    }
    
    pub fn list_skills(&self) -> Vec<SkillInfo> {
        let skills = self.loaded_skills.lock();
        
        skills.iter().map(|loaded_skill| SkillInfo {
            name: loaded_skill.handle.name.clone(),
            version: loaded_skill.handle.version,
            loaded_at: loaded_skill.handle.loaded_at,
            load_count: loaded_skill.load_count,
            memory_limit: loaded_skill.skill.manifest.mem_limit_bytes,
            time_slice: loaded_skill.skill.manifest.time_slice_ms,
        }).collect()
    }
    
    pub fn get_stats(&self) -> SkillStats {
        self.counters.get_stats()
    }
}

#[derive(Debug, Clone)]
pub struct SkillInfo {
    pub name: String,
    pub version: u32,
    pub loaded_at: u64,
    pub load_count: u32,
    pub memory_limit: u32,
    pub time_slice: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegistryError {
    TooManySkills,
    WasmTooLarge,
    InvalidManifest,
    PolicyDenied(String),
    PolicyMutationNotSupported,
    SkillNotFound,
    InvalidHandle,
    ExecutionFailed(String),
    SkillStillReferenced,
    BrokerError(String),
}

impl core::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RegistryError::TooManySkills => write!(f, "Too many skills loaded"),
            RegistryError::WasmTooLarge => write!(f, "WebAssembly module too large"),
            RegistryError::InvalidManifest => write!(f, "Invalid skill manifest"),
            RegistryError::PolicyDenied(reason) => write!(f, "Policy denied: {}", reason),
            RegistryError::PolicyMutationNotSupported => write!(f, "Policy mutation not supported"),
            RegistryError::SkillNotFound => write!(f, "Skill not found"),
            RegistryError::InvalidHandle => write!(f, "Invalid skill handle"),
            RegistryError::ExecutionFailed(reason) => write!(f, "Execution failed: {}", reason),
            RegistryError::SkillStillReferenced => write!(f, "Skill still referenced"),
            RegistryError::BrokerError(reason) => write!(f, "Broker error: {}", reason),
        }
    }
}

impl From<crate::skills::broker::BrokerError> for RegistryError {
    fn from(error: crate::skills::broker::BrokerError) -> Self {
        RegistryError::BrokerError(error.to_string())
    }
}
