use crate::{kprintln, klog};
use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

use crate::skills::manifest::HostcallId;
use crate::intent::schema::{PlanPreviewV1, ActionV1, EvidenceV1};
use crate::world::query::{Pattern, Range};

pub const HOSTCALL_BUFFER_MAX_SIZE: usize = 64 * 1024; // 64 KiB
pub const LOG_RATE_LIMIT: u64 = 100; // 100 logs per second

#[derive(Debug)]
pub struct HostcallContext {
    pub skill_id: u64,
    pub plan_actions: Vec<ActionV1>,
    pub evidence: Vec<EvidenceV1>,
    pub world_model_queries: Vec<WorldModelQuery>,
    pub log_count: AtomicU64,
    pub last_log_reset: AtomicU64,
}

impl HostcallContext {
    pub fn new(skill_id: u64) -> Self {
        Self {
            skill_id,
            plan_actions: Vec::new(),
            evidence: Vec::new(),
            world_model_queries: Vec::new(),
            log_count: AtomicU64::new(0),
            last_log_reset: AtomicU64::new(0),
        }
    }
    
    pub fn add_action(&mut self, action: ActionV1) {
        self.plan_actions.push(action);
    }
    
    pub fn add_evidence(&mut self, evidence: EvidenceV1) {
        self.evidence.push(evidence);
    }
    
    pub fn add_world_query(&mut self, query: WorldModelQuery) {
        self.world_model_queries.push(query);
    }
    
    pub fn can_log(&self) -> bool {
        let now = crate::time::get_current_time_ms();
        let last_reset = self.last_log_reset.load(Ordering::SeqCst);
        
        if now - last_reset > 1000 {
            self.last_log_reset.store(now, Ordering::SeqCst);
            self.log_count.store(0, Ordering::SeqCst);
        }
        
        self.log_count.load(Ordering::SeqCst) < LOG_RATE_LIMIT
    }
    
    pub fn increment_log_count(&self) {
        self.log_count.fetch_add(1, Ordering::SeqCst);
    }
}

impl Clone for HostcallContext {
    fn clone(&self) -> Self {
        Self {
            skill_id: self.skill_id,
            plan_actions: self.plan_actions.clone(),
            evidence: self.evidence.clone(),
            world_model_queries: self.world_model_queries.clone(),
            log_count: AtomicU64::new(self.log_count.load(Ordering::Relaxed)),
            last_log_reset: AtomicU64::new(self.last_log_reset.load(Ordering::Relaxed)),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorldModelQuery {
    pub pattern: Pattern,
    pub range: Range,
    pub limit: u16,
    pub offset: u32,
}

#[derive(Debug)]
pub struct WasiHost {
    pub context: Mutex<HostcallContext>,
    pub deterministic_rng: Mutex<DeterministicRng>,
}

impl WasiHost {
    pub fn new(skill_id: u64) -> Self {
        Self {
            context: Mutex::new(HostcallContext::new(skill_id)),
            deterministic_rng: Mutex::new(DeterministicRng::new(skill_id)),
        }
    }
    
    pub fn handle_hostcall(&self, hostcall_id: u32, params: &[u32]) -> Result<u32, HostcallError> {
        let hostcall = HostcallId::from_u32(hostcall_id)
            .ok_or(HostcallError::InvalidHostcall)?;
        
        match hostcall {
            HostcallId::WmQueryReadonly => self.hc_wm_query_readonly(params),
            HostcallId::EmitPlanAction => self.hc_emit_plan_action(params),
            HostcallId::EmitEvidence => self.hc_emit_evidence(params),
            HostcallId::LogDebug => self.hc_log_debug(params),
            HostcallId::RngDeterministic => self.hc_rng_deterministic(params),
        }
    }
    
    fn hc_wm_query_readonly(&self, params: &[u32]) -> Result<u32, HostcallError> {
        if params.len() < 4 {
            return Err(HostcallError::InvalidParameters);
        }
        
        let input_ptr = params[0] as usize;
        let input_len = params[1] as usize;
        let output_ptr = params[2] as usize;
        let output_len = params[3] as usize;
        
        if input_len > HOSTCALL_BUFFER_MAX_SIZE || output_len > HOSTCALL_BUFFER_MAX_SIZE {
            return Err(HostcallError::BufferTooLarge);
        }
        
        let input_data = self.read_memory(input_ptr, input_len)?;
        let query: WorldModelQuery = serde_cbor::from_slice(&input_data)
            .map_err(|_| HostcallError::InvalidInput)?;
        
        let mut context = self.context.lock();
        context.add_world_query(query);
        
        Ok(0)
    }
    
    fn hc_emit_plan_action(&self, params: &[u32]) -> Result<u32, HostcallError> {
        if params.len() < 2 {
            return Err(HostcallError::InvalidParameters);
        }
        
        let action_ptr = params[0] as usize;
        let action_len = params[1] as usize;
        
        if action_len > HOSTCALL_BUFFER_MAX_SIZE {
            return Err(HostcallError::BufferTooLarge);
        }
        
        let action_data = self.read_memory(action_ptr, action_len)?;
        let action: ActionV1 = serde_cbor::from_slice(&action_data)
            .map_err(|_| HostcallError::InvalidInput)?;
        
        let mut context = self.context.lock();
        context.add_action(action);
        
        Ok(0)
    }
    
    fn hc_emit_evidence(&self, params: &[u32]) -> Result<u32, HostcallError> {
        if params.len() < 2 {
            return Err(HostcallError::InvalidParameters);
        }
        
        let evidence_ptr = params[0] as usize;
        let evidence_len = params[1] as usize;
        
        if evidence_len > HOSTCALL_BUFFER_MAX_SIZE {
            return Err(HostcallError::BufferTooLarge);
        }
        
        let evidence_data = self.read_memory(evidence_ptr, evidence_len)?;
        let evidence: EvidenceV1 = serde_cbor::from_slice(&evidence_data)
            .map_err(|_| HostcallError::InvalidInput)?;
        
        let mut context = self.context.lock();
        context.add_evidence(evidence);
        
        Ok(0)
    }
    
    fn hc_log_debug(&self, params: &[u32]) -> Result<u32, HostcallError> {
        if params.len() < 2 {
            return Err(HostcallError::InvalidParameters);
        }
        
        let log_ptr = params[0] as usize;
        let log_len = params[1] as usize;
        
        if log_len > 1024 {
            return Err(HostcallError::BufferTooLarge);
        }
        
        let context = self.context.lock();
        if !context.can_log() {
            return Err(HostcallError::RateLimited);
        }
        
        let log_data = self.read_memory(log_ptr, log_len)?;
        let log_message = String::from_utf8(log_data)
            .map_err(|_| HostcallError::InvalidInput)?;
        
        klog!(DEBUG, "[SKILL-{}] {}", context.skill_id, log_message);
        context.increment_log_count();
        
        Ok(0)
    }
    
    fn hc_rng_deterministic(&self, params: &[u32]) -> Result<u32, HostcallError> {
        if params.len() < 2 {
            return Err(HostcallError::InvalidParameters);
        }
        
        let output_ptr = params[0] as usize;
        let output_len = params[1] as usize;
        
        if output_len > 1024 {
            return Err(HostcallError::BufferTooLarge);
        }
        
        let mut rng = self.deterministic_rng.lock();
        let random_bytes = rng.generate_bytes(output_len);
        
        self.write_memory(output_ptr, &random_bytes)?;
        
        Ok(0)
    }
    
    fn read_memory(&self, ptr: usize, len: usize) -> Result<Vec<u8>, HostcallError> {
        if ptr == 0 || len == 0 {
            return Err(HostcallError::InvalidPointer);
        }
        
        let mut data = Vec::with_capacity(len);
        for i in 0..len {
            let byte = unsafe { *(ptr as *const u8).add(i) };
            data.push(byte);
        }
        
        Ok(data)
    }
    
    fn write_memory(&self, ptr: usize, data: &[u8]) -> Result<(), HostcallError> {
        if ptr == 0 {
            return Err(HostcallError::InvalidPointer);
        }
        
        for (i, &byte) in data.iter().enumerate() {
            unsafe { *(ptr as *mut u8).add(i) = byte };
        }
        
        Ok(())
    }
    
    pub fn get_context(&self) -> HostcallContext {
        self.context.lock().clone()
    }
}

#[derive(Debug)]
pub struct DeterministicRng {
    pub seed: u64,
    pub counter: u64,
}

impl DeterministicRng {
    pub fn new(skill_id: u64) -> Self {
        Self {
            seed: skill_id as u64,
            counter: 0,
        }
    }
    
    pub fn generate_bytes(&mut self, len: usize) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(len);
        
        for _ in 0..len {
            let random_byte = self.next_byte();
            bytes.push(random_byte);
        }
        
        bytes
    }
    
    fn next_byte(&mut self) -> u8 {
        self.counter += 1;
        let combined = self.seed.wrapping_add(self.counter);
        let hash = self.simple_hash(combined);
        (hash & 0xFF) as u8
    }
    
    fn simple_hash(&self, input: u64) -> u64 {
        let mut hash = input;
        hash = hash.wrapping_mul(0x517cc1b727220a95);
        hash = hash.rotate_right(32);
        hash = hash.wrapping_mul(0x9e3779b97f4a7c15);
        hash
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HostcallError {
    InvalidHostcall,
    InvalidParameters,
    InvalidPointer,
    BufferTooLarge,
    InvalidInput,
    RateLimited,
    MemoryAccessFailed,
}

impl core::fmt::Display for HostcallError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HostcallError::InvalidHostcall => write!(f, "Invalid hostcall ID"),
            HostcallError::InvalidParameters => write!(f, "Invalid parameters"),
            HostcallError::InvalidPointer => write!(f, "Invalid memory pointer"),
            HostcallError::BufferTooLarge => write!(f, "Buffer too large"),
            HostcallError::InvalidInput => write!(f, "Invalid input data"),
            HostcallError::RateLimited => write!(f, "Rate limited"),
            HostcallError::MemoryAccessFailed => write!(f, "Memory access failed"),
        }
    }
}
