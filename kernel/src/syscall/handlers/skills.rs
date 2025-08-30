use crate::{kprintln, klog};
use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::skills::{load_skill, invoke_preview, unload_skill, get_skill_info, list_skills, get_skills_stats};
use crate::skills::manifest::SkillManifestV1;
use crate::secman::audit_codes::AuditReason;
use crate::secman::cap::CapStore;
use crate::secman::cap_flags::*;

pub const SKILL_MANIFEST_MAX_SIZE: usize = 32 * 1024; // 32 KiB
pub const SKILL_WASM_MAX_SIZE: usize = 32 * 1024 * 1024; // 32 MiB
pub const SKILL_INPUT_MAX_SIZE: usize = 64 * 1024; // 64 KiB

pub fn sys_skill_load(manifest_ptr: usize, manifest_len: usize, wasm_ptr: usize, wasm_len: usize) -> Result<u64, i32> {
    if !CapStore::has_capability(CAP_SKILL_LOAD) {
        crate::secman::audit::emit(AuditReason::SkillLoadCapDenied, 0, 0, 0);
        return Err(crate::errno::EPERM);
    }
    
    if manifest_len > SKILL_MANIFEST_MAX_SIZE {
        crate::secman::audit::emit(AuditReason::SkillLoadOversize, manifest_len as u64, 0, 0);
        return Err(crate::errno::E2BIG);
    }
    
    if wasm_len > SKILL_WASM_MAX_SIZE {
        crate::secman::audit::emit(AuditReason::SkillLoadOversize, wasm_len as u64, 0, 0);
        return Err(crate::errno::E2BIG);
    }
    
    let manifest_bytes = match crate::syscall::copy_from_user(manifest_ptr, manifest_len) {
        Ok(bytes) => bytes,
        Err(_) => {
            crate::secman::audit::emit(AuditReason::SkillLoadInvalid, 0, 0, 0);
            return Err(crate::errno::EFAULT);
        }
    };
    
    let wasm_bytes = match crate::syscall::copy_from_user(wasm_ptr, wasm_len) {
        Ok(bytes) => bytes,
        Err(_) => {
            crate::secman::audit::emit(AuditReason::SkillLoadInvalid, 0, 0, 0);
            return Err(crate::errno::EFAULT);
        }
    };
    
    match load_skill(&manifest_bytes, &wasm_bytes) {
        Ok(handle) => {
            crate::secman::audit::emit(AuditReason::SkillLoadOk, handle.id, manifest_len as u64, wasm_len as u64);
            Ok(handle.id)
        }
        Err(e) => {
            let error_code = match e {
                crate::skills::SkillsError::ManifestTooLarge => crate::errno::E2BIG,
                crate::skills::SkillsError::WasmTooLarge => crate::errno::E2BIG,
                crate::skills::SkillsError::InvalidManifest => crate::errno::EINVAL,
                crate::skills::SkillsError::RegistryError(_) => crate::errno::EINVAL,
                _ => crate::errno::EINVAL,
            };
            
            crate::secman::audit::emit(AuditReason::SkillLoadDeny, error_code as u64, 0, 0);
            Err(error_code)
        }
    }
}

pub fn sys_skill_invoke_preview(handle: u64, input_ptr: usize, input_len: usize) -> Result<u64, i32> {
    if !CapStore::has_capability(CAP_SKILL_INVOKE) {
        crate::secman::audit::emit(AuditReason::SkillInvokeCapDenied, handle, 0, 0);
        return Err(crate::errno::EPERM);
    }
    
    if input_len > SKILL_INPUT_MAX_SIZE {
        crate::secman::audit::emit(AuditReason::SkillInvokeOversize, input_len as u64, 0, 0);
        return Err(crate::errno::E2BIG);
    }
    
    let input_bytes = if input_len > 0 {
        match crate::syscall::copy_from_user(input_ptr, input_len) {
            Ok(bytes) => bytes,
            Err(_) => {
                crate::secman::audit::emit(AuditReason::SkillInvokeInvalid, 0, 0, 0);
                return Err(crate::errno::EFAULT);
            }
        }
    } else {
        Vec::new()
    };
    
    match invoke_preview(handle, &input_bytes) {
        Ok(bundle) => {
            let metrics = serde_json::to_string(&bundle.metrics)
                .unwrap_or_else(|_| "{}".to_string());
            
            crate::secman::audit::emit(
                AuditReason::SkillInvokeOk,
                handle,
                bundle.metrics.instructions_executed,
                bundle.metrics.execution_time_us
            );
            
            klog!("[SKILLS] Invoke preview success: {} actions, {}μs, {} instructions",
                  bundle.preview.plan.actions.len(),
                  bundle.metrics.execution_time_us,
                  bundle.metrics.instructions_executed);
            
            Ok(0)
        }
        Err(e) => {
            let error_code = match e {
                crate::skills::SkillsError::SkillNotFound => crate::errno::ESRCH,
                crate::skills::SkillsError::InvalidHandle => crate::errno::EINVAL,
                crate::skills::SkillsError::InputTooLarge => crate::errno::E2BIG,
                crate::skills::SkillsError::ExecutionFailed(_) => crate::errno::EAGAIN,
                _ => crate::errno::EINVAL,
            };
            
            crate::secman::audit::emit(AuditReason::SkillInvokeQuota, error_code as u64, 0, 0);
            Err(error_code)
        }
    }
}

pub fn sys_skill_unload(handle: u64) -> Result<u64, i32> {
    if !CapStore::has_capability(CAP_SKILL_LOAD) {
        crate::secman::audit::emit(AuditReason::SkillUnloadCapDenied, handle, 0, 0);
        return Err(crate::errno::EPERM);
    }
    
    match unload_skill(handle) {
        Ok(_) => {
            crate::secman::audit::emit(AuditReason::SkillUnloadOk, handle, 0, 0);
            Ok(0)
        }
        Err(e) => {
            let error_code = match e {
                crate::skills::SkillsError::SkillNotFound => crate::errno::ESRCH,
                crate::skills::SkillsError::InvalidHandle => crate::errno::EINVAL,
                crate::skills::SkillsError::RegistryError(_) => crate::errno::EINVAL,
                _ => crate::errno::EINVAL,
            };
            
            crate::secman::audit::emit(AuditReason::SkillUnloadDeny, error_code as u64, 0, 0);
            Err(error_code)
        }
    }
}

pub fn sys_skill_status(handle: u64, info_ptr: usize) -> Result<u64, i32> {
    if !CapStore::has_capability(CAP_SKILL_QUERY) {
        crate::secman::audit::emit(AuditReason::SkillStatusCapDenied, handle, 0, 0);
        return Err(crate::errno::EPERM);
    }
    
    let skill_info = match get_skill_info(handle) {
        Some(info) => info,
        None => {
            crate::secman::audit::emit(AuditReason::SkillStatusNotFound, handle, 0, 0);
            return Err(crate::errno::ESRCH);
        }
    };
    
    let info_json = serde_json::to_string(&skill_info)
        .unwrap_or_else(|_| "{}".to_string());
    
    let info_bytes = info_json.as_bytes();
    if info_bytes.len() > 1024 {
        return Err(crate::errno::E2BIG);
    }
    
    match crate::syscall::copy_to_user(info_ptr, info_bytes) {
        Ok(_) => {
            crate::secman::audit::emit(AuditReason::SkillStatusOk, handle, info_bytes.len() as u64, 0);
            Ok(info_bytes.len() as u64)
        }
        Err(_) => {
            crate::secman::audit::emit(AuditReason::SkillStatusInvalid, 0, 0, 0);
            Err(crate::errno::EFAULT)
        }
    }
}

pub fn sys_skill_list(list_ptr: usize, list_len: usize) -> Result<u64, i32> {
    if !CapStore::has_capability(CAP_SKILL_QUERY) {
        crate::secman::audit::emit(AuditReason::SkillListCapDenied, 0, 0, 0);
        return Err(crate::errno::EPERM);
    }
    
    let skills = list_skills();
    let skills_json = serde_json::to_string(&skills)
        .unwrap_or_else(|_| "[]".to_string());
    
    let skills_bytes = skills_json.as_bytes();
    if skills_bytes.len() > list_len {
        return Err(crate::errno::E2BIG);
    }
    
    match crate::syscall::copy_to_user(list_ptr, skills_bytes) {
        Ok(_) => {
            crate::secman::audit::emit(AuditReason::SkillListOk, skills.len() as u64, skills_bytes.len() as u64, 0);
            Ok(skills_bytes.len() as u64)
        }
        Err(_) => {
            crate::secman::audit::emit(AuditReason::SkillListInvalid, 0, 0, 0);
            Err(crate::errno::EFAULT)
        }
    }
}

pub fn sys_skill_stats(stats_ptr: usize, stats_len: usize) -> Result<u64, i32> {
    if !CapStore::has_capability(CAP_SKILL_QUERY) {
        crate::secman::audit::emit(AuditReason::SkillStatsCapDenied, 0, 0, 0);
        return Err(crate::errno::EPERM);
    }
    
    let stats = get_skills_stats();
    let stats_json = serde_json::to_string(&stats)
        .unwrap_or_else(|_| "{}".to_string());
    
    let stats_bytes = stats_json.as_bytes();
    if stats_bytes.len() > stats_len {
        return Err(crate::errno::E2BIG);
    }
    
    match crate::syscall::copy_to_user(stats_ptr, stats_bytes) {
        Ok(_) => {
            crate::secman::audit::emit(AuditReason::SkillStatsOk, stats_bytes.len() as u64, 0, 0);
            Ok(stats_bytes.len() as u64)
        }
        Err(_) => {
            crate::secman::audit::emit(AuditReason::SkillStatsInvalid, 0, 0, 0);
            Err(crate::errno::EFAULT)
        }
    }
}

pub fn copy_from_user(ptr: usize, len: usize) -> Result<Vec<u8>, ()> {
    if ptr == 0 || len == 0 {
        return Ok(Vec::new());
    }
    
    let mut data = Vec::with_capacity(len);
    for i in 0..len {
        let byte = unsafe { *(ptr as *const u8).add(i) };
        data.push(byte);
    }
    
    Ok(data)
}

pub fn copy_to_user(ptr: usize, data: &[u8]) -> Result<(), ()> {
    if ptr == 0 {
        return Err(());
    }
    
    for (i, &byte) in data.iter().enumerate() {
        unsafe { *(ptr as *mut u8).add(i) = byte };
    }
    
    Ok(())
}
