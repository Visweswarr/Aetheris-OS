use crate::{kprintln, klog, format, lazy_static};
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;

use crate::skills::manifest::SkillManifestV1;
use crate::skills::registry::{SkillRegistry, SkillHandle, SkillInfo, SkillStats};
use crate::skills::exec::PreviewBundle;

pub mod manifest;
pub mod broker;
pub mod wasi;
pub mod exec;
pub mod registry;

pub struct SkillsKernel {
    pub registry: Mutex<SkillRegistry>,
}

impl SkillsKernel {
    pub fn new() -> Self {
        Self {
            registry: Mutex::new(SkillRegistry::new()),
        }
    }
    
    pub fn load_skill(&self, manifest_bytes: &[u8], wasm_bytes: &[u8]) -> Result<SkillHandle, SkillsError> {
        if manifest_bytes.len() > crate::skills::manifest::SKILL_MANIFEST_MAX_SIZE {
            return Err(SkillsError::ManifestTooLarge);
        }
        
        if wasm_bytes.len() > 32 * 1024 * 1024 {
            return Err(SkillsError::WasmTooLarge);
        }
        
        let manifest: SkillManifestV1 = serde_cbor::from_slice(manifest_bytes)
            .map_err(|_| SkillsError::InvalidManifest)?;
        
        if !manifest.validate().is_ok() {
            return Err(SkillsError::InvalidManifest);
        }
        
        let registry = self.registry.lock();
        let handle = registry.load_skill(manifest, wasm_bytes.to_vec())?;
        
        klog!(INFO, "[SKILLS] Loaded skill {} with handle 0x{:X}", handle.name, handle.id);
        
        Ok(handle)
    }
    
    pub fn invoke_preview(&self, handle: u64, input: &[u8]) -> Result<PreviewBundle, SkillsError> {
        if input.len() > 64 * 1024 {
            return Err(SkillsError::InputTooLarge);
        }
        
        let registry = self.registry.lock();
        let bundle = registry.invoke_skill(handle, input)?;
        
        klog!(DEBUG, "[SKILLS] Invoked skill preview: {} actions, {} evidence, {}μs",
              bundle.preview.plan.actions.len(),
              bundle.evidence.len(),
              bundle.metrics.execution_time_us);
        
        Ok(bundle)
    }
    
    pub fn unload_skill(&self, handle: u64) -> Result<(), SkillsError> {
        let registry = self.registry.lock();
        registry.unload_skill(handle)?;
        
        klog!(INFO, "[SKILLS] Unloaded skill with handle 0x{:X}", handle);
        
        Ok(())
    }
    
    pub fn get_skill_info(&self, handle: u64) -> Option<SkillInfo> {
        let registry = self.registry.lock();
        registry.get_skill_info(handle)
    }
    
    pub fn list_skills(&self) -> Vec<SkillInfo> {
        let registry = self.registry.lock();
        registry.list_skills()
    }
    
    pub fn get_stats(&self) -> SkillStats {
        let registry = self.registry.lock();
        registry.get_stats()
    }
    
    pub fn get_loaded_count(&self) -> usize {
        let registry = self.registry.lock();
        registry.get_loaded_skills_count()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SkillsError {
    ManifestTooLarge,
    WasmTooLarge,
    InputTooLarge,
    InvalidManifest,
    SkillNotFound,
    InvalidHandle,
    ExecutionFailed(String),
    RegistryError(String),
}

impl core::fmt::Display for SkillsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SkillsError::ManifestTooLarge => write!(f, "Skill manifest too large"),
            SkillsError::WasmTooLarge => write!(f, "WebAssembly module too large"),
            SkillsError::InputTooLarge => write!(f, "Input data too large"),
            SkillsError::InvalidManifest => write!(f, "Invalid skill manifest"),
            SkillsError::SkillNotFound => write!(f, "Skill not found"),
            SkillsError::InvalidHandle => write!(f, "Invalid skill handle"),
            SkillsError::ExecutionFailed(reason) => write!(f, "Execution failed: {}", reason),
            SkillsError::RegistryError(reason) => write!(f, "Registry error: {}", reason),
        }
    }
}

impl From<crate::skills::registry::RegistryError> for SkillsError {
    fn from(error: crate::skills::registry::RegistryError) -> Self {
        match error {
            crate::skills::registry::RegistryError::TooManySkills => SkillsError::RegistryError("Too many skills".to_string()),
            crate::skills::registry::RegistryError::WasmTooLarge => SkillsError::WasmTooLarge,
            crate::skills::registry::RegistryError::InvalidManifest => SkillsError::InvalidManifest,
            crate::skills::registry::RegistryError::PolicyDenied(reason) => SkillsError::RegistryError(format!("Policy denied: {}", reason)),
            crate::skills::registry::RegistryError::PolicyMutationNotSupported => SkillsError::RegistryError("Policy mutation not supported".to_string()),
            crate::skills::registry::RegistryError::SkillNotFound => SkillsError::SkillNotFound,
            crate::skills::registry::RegistryError::InvalidHandle => SkillsError::InvalidHandle,
            crate::skills::registry::RegistryError::ExecutionFailed(reason) => SkillsError::ExecutionFailed(reason),
            crate::skills::registry::RegistryError::SkillStillReferenced => SkillsError::RegistryError("Skill still referenced".to_string()),
            crate::skills::registry::RegistryError::BrokerError(reason) => SkillsError::RegistryError(format!("Broker error: {}", reason)),
        }
    }
}

lazy_static! {
    pub static ref SKILLS_KERNEL: Mutex<SkillsKernel> = Mutex::new(SkillsKernel::new());
}

pub fn get_skills_kernel() -> &'static Mutex<SkillsKernel> {
    &SKILLS_KERNEL
}

pub fn load_skill(manifest_bytes: &[u8], wasm_bytes: &[u8]) -> Result<SkillHandle, SkillsError> {
    let kernel = get_skills_kernel();
    let skills = kernel.lock();
    skills.load_skill(manifest_bytes, wasm_bytes)
}

pub fn invoke_preview(handle: u64, input: &[u8]) -> Result<PreviewBundle, SkillsError> {
    let kernel = get_skills_kernel();
    let skills = kernel.lock();
    skills.invoke_preview(handle, input)
}

pub fn unload_skill(handle: u64) -> Result<(), SkillsError> {
    let kernel = get_skills_kernel();
    let skills = kernel.lock();
    skills.unload_skill(handle)
}

pub fn get_skill_info(handle: u64) -> Option<SkillInfo> {
    let kernel = get_skills_kernel();
    let skills = kernel.lock();
    skills.get_skill_info(handle)
}

pub fn list_skills() -> Vec<SkillInfo> {
    let kernel = get_skills_kernel();
    let skills = kernel.lock();
    skills.list_skills()
}

pub fn get_skills_stats() -> SkillStats {
    let kernel = get_skills_kernel();
    let skills = kernel.lock();
    skills.get_stats()
}

pub fn get_loaded_skills_count() -> usize {
    let kernel = get_skills_kernel();
    let skills = kernel.lock();
    skills.get_loaded_count()
}
