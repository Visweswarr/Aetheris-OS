use crate::{kprintln, klog};
use crate::syscall::{SyscallArgs, SyscallResult};
use crate::secman::audit_codes::AuditReason;
use crate::secman::cap_store::CapStore;
use crate::secman::cap::CapTokenV2;
use alloc::vec::Vec;
use alloc::string::String;

use crate::world::{WorldModelKernel, schema::{FactV1, SnapshotId, EntityId, PredId, ValueAtom}};
use crate::world::query::{Pattern, Range};

pub const CAP_WM_WRITE: u64 = 1 << 20;
pub const CAP_WM_READ: u64 = 1 << 21;
pub const CAP_WM_EXPORT: u64 = 1 << 22;

pub const CAP_SKILL_LOAD: u64 = 1 << 23;
pub const CAP_SKILL_INVOKE: u64 = 1 << 24;
pub const CAP_SKILL_QUERY: u64 = 1 << 25;

pub const CAP_EVENT_SUB: u64 = 1 << 26;
pub const CAP_EVENT_PUB: u64 = 1 << 27;

pub const CAP_POLICY_LOAD: u64 = 1 << 28;
pub const CAP_POLICY_EVAL: u64 = 1 << 29;
pub const CAP_POLICY_SIMULATE: u64 = 1 << 30;

pub const CAP_LLM_USE: u64 = 1 << 31;
pub const CAP_LLM_DEV: u64 = 1 << 32;

pub fn sys_wm_put(args: &SyscallArgs) -> SyscallResult {
    let (buf_ptr, buf_len) = args.get_two::<*const u8, usize>();
    
    if buf_len > 65536 {
        crate::audit::emit(AuditReason::WorldModelOversize, "WM_PUT_ENOSPC", &[]);
        return Err(crate::error::Errno::E2BIG);
    }
    
    let caps = CapStore::get_current_caps();
    if !has_capability(&caps, CAP_WM_WRITE) {
        crate::audit::emit(AuditReason::WorldModelCapDenied, "WM_PUT_CAP_DENIED", &[]);
        return Err(crate::error::Errno::EPERM);
    }
    
    let facts = match copy_facts_from_user(buf_ptr, buf_len) {
        Ok(facts) => facts,
        Err(_) => {
            crate::audit::emit(AuditReason::WorldModelInvalid, "WM_PUT_INVALID", &[]);
            return Err(crate::error::Errno::EINVAL);
        }
    };
    
    let world_kernel = crate::world::WORLD_MODEL.lock();
    match world_kernel.put_facts(&facts) {
        Ok((ok_count, snapshot_id)) => {
            crate::audit::emit(AuditReason::WorldModelPutOk, "WM_PUT_OK", &[
                ("count", &ok_count.to_string()),
                ("snapshot_id", &snapshot_id.0.to_string()),
            ]);
            
            let response = crate::world::put_response_t {
                ok_count,
                snapshot_id: snapshot_id.0,
            };
            
            Ok(response as isize)
        }
        Err(()) => {
            crate::audit::emit(AuditReason::WorldModelPutEnospc, "WM_PUT_ENOSPC", &[]);
            Err(crate::error::Errno::ENOSPC)
        }
    }
}

pub fn sys_wm_query(args: &SyscallArgs) -> SyscallResult {
    let (pattern_ptr, range_ptr, limit, offset) = args.get_four::<*const u8, *const u8, u16, u32>();
    
    let caps = CapStore::get_current_caps();
    if !has_capability(&caps, CAP_WM_READ) {
        crate::audit::emit(AuditReason::WorldModelCapDenied, "WM_QUERY_CAP_DENIED", &[]);
        return Err(crate::error::Errno::EPERM);
    }
    
    if limit > 1024 {
        return Err(crate::error::Errno::EINVAL);
    }
    
    if offset > 1048576 {
        return Err(crate::error::Errno::EINVAL);
    }
    
    let pattern = match copy_pattern_from_user(pattern_ptr) {
        Ok(pattern) => pattern,
        Err(_) => {
            crate::audit::emit(AuditReason::WorldModelInvalid, "WM_QUERY_INVALID_PATTERN", &[]);
            return Err(crate::error::Errno::EINVAL);
        }
    };
    
    let range = match copy_range_from_user(range_ptr) {
        Ok(range) => range,
        Err(_) => {
            crate::audit::emit(AuditReason::WorldModelInvalid, "WM_QUERY_INVALID_RANGE", &[]);
            return Err(crate::error::Errno::EINVAL);
        }
    };
    
    let world_kernel = crate::world::WORLD_MODEL.lock();
    match world_kernel.query(&pattern, &range, limit, offset) {
        Ok(rows) => {
            crate::audit::emit(AuditReason::WorldModelQueryOk, "WM_QUERY_OK", &[
                ("count", &rows.facts.len().to_string()),
                ("limit", &limit.to_string()),
                ("offset", &offset.to_string()),
            ]);
            
            let response = crate::world::query_response_t {
                rows: rows.facts.as_ptr() as *mut crate::world::schema::fact_v1_t,
                count: rows.facts.len() as u32,
                next_offset: rows.next_offset,
            };
            
            Ok(response as isize)
        }
        Err(()) => {
            crate::audit::emit(AuditReason::WorldModelQueryFailed, "WM_QUERY_FAILED", &[]);
            Err(crate::error::Errno::EIO)
        }
    }
}

pub fn sys_wm_snapshot(args: &SyscallArgs) -> SyscallResult {
    let (op_ptr, id_ptr) = args.get_two::<*const u8, *mut u64>();
    
    let caps = CapStore::get_current_caps();
    if !has_capability(&caps, CAP_WM_READ) {
        crate::audit::emit(AuditReason::WorldModelCapDenied, "WM_SNAPSHOT_CAP_DENIED", &[]);
        return Err(crate::error::Errno::EPERM);
    }
    
    let op = match copy_snapshot_op_from_user(op_ptr) {
        Ok(op) => op,
        Err(_) => {
            crate::audit::emit(AuditReason::WorldModelInvalid, "WM_SNAPSHOT_INVALID_OP", &[]);
            return Err(crate::error::Errno::EINVAL);
        }
    };
    
    let world_kernel = crate::world::WORLD_MODEL.lock();
    
    match op.as_str() {
        "create" => {
            let snapshot_id = world_kernel.snapshot_create();
            let bytes_estimate = world_kernel.world.lock().segments.iter()
                .map(|s| s.bytes as usize)
                .sum::<usize>() as u32;
            
            crate::audit::emit(AuditReason::WorldModelSnapshotNew, "WM_SNAPSHOT_NEW", &[
                ("id", &snapshot_id.0.to_string()),
                ("bytes", &bytes_estimate.to_string()),
            ]);
            
            let response = crate::world::snapshot_response_t {
                id: snapshot_id.0,
                created: true,
                bytes_estimate,
            };
            
            Ok(response as isize)
        }
        "open" => {
            let id = unsafe { *id_ptr };
            let snapshot_id = SnapshotId(id);
            
            if let Some(_view) = world_kernel.snapshot_open(snapshot_id) {
                let bytes_estimate = world_kernel.world.lock().segments.iter()
                    .map(|s| s.bytes as usize)
                    .sum::<usize>() as u32;
                
                crate::audit::emit(AuditReason::WorldModelSnapshotOpen, "WM_SNAPSHOT_OPEN", &[
                    ("id", &id.to_string()),
                    ("bytes", &bytes_estimate.to_string()),
                ]);
                
                let response = crate::world::snapshot_response_t {
                    id,
                    created: false,
                    bytes_estimate,
                };
                
                Ok(response as isize)
            } else {
                crate::audit::emit(AuditReason::WorldModelSnapshotNotFound, "WM_SNAPSHOT_NOT_FOUND", &[
                    ("id", &id.to_string()),
                ]);
                Err(crate::error::Errno::ESRCH)
            }
        }
        _ => {
            crate::audit::emit(AuditReason::WorldModelInvalid, "WM_SNAPSHOT_UNKNOWN_OP", &[("op", &op)]);
            Err(crate::error::Errno::EINVAL)
        }
    }
}

pub fn sys_wm_export(args: &SyscallArgs) -> SyscallResult {
    let (id, subject_filter_ptr) = args.get_two::<u64, *const u8>();
    
    let caps = CapStore::get_current_caps();
    if !has_capability(&caps, CAP_WM_EXPORT) {
        crate::audit::emit(AuditReason::WorldModelCapDenied, "WM_EXPORT_CAP_DENIED", &[]);
        return Err(crate::error::Errno::EPERM);
    }
    
    let snapshot_id = SnapshotId(id);
    let subject_filter = if !subject_filter_ptr.is_null() {
        match copy_entity_id_from_user(subject_filter_ptr) {
            Ok(entity_id) => Some(entity_id),
            Err(_) => {
                crate::audit::emit(AuditReason::WorldModelInvalid, "WM_EXPORT_INVALID_FILTER", &[]);
                return Err(crate::error::Errno::EINVAL);
            }
        }
    } else {
        None
    };
    
    let world_kernel = crate::world::WORLD_MODEL.lock();
    match world_kernel.export_snapshot(snapshot_id, subject_filter) {
        Ok(cbor_data) => {
            crate::audit::emit(AuditReason::WorldModelExportOk, "WM_EXPORT_OK", &[
                ("snapshot_id", &id.to_string()),
                ("bytes", &cbor_data.len().to_string()),
            ]);
            
            let response = crate::world::export_response_t {
                cbor_blob: cbor_data.as_ptr(),
                len: cbor_data.len() as u32,
            };
            
            Ok(response as isize)
        }
        Err(()) => {
            crate::audit::emit(AuditReason::WorldModelExportFailed, "WM_EXPORT_FAILED", &[
                ("snapshot_id", &id.to_string()),
            ]);
            Err(crate::error::Errno::EIO)
        }
    }
}

fn has_capability(caps: &[CapTokenV2], required_cap: u64) -> bool {
    caps.iter().any(|cap| cap.scope_flags & required_cap != 0)
}

fn copy_facts_from_user(buf_ptr: *const u8, buf_len: usize) -> Result<Vec<FactV1>, ()> {
    let mut facts = Vec::new();
    let mut offset = 0;
    
    while offset < buf_len {
        let fact_size = unsafe { *(buf_ptr.add(offset) as *const u32) };
        if fact_size > 1024 || offset + fact_size as usize > buf_len {
            return Err(());
        }
        
        let fact_data = unsafe { core::slice::from_raw_parts(buf_ptr.add(offset + 4), fact_size as usize) };
        let fact: FactV1 = serde_cbor::from_slice(fact_data).map_err(|_| ())?;
        facts.push(fact);
        
        offset += 4 + fact_size as usize;
    }
    
    Ok(facts)
}

fn copy_pattern_from_user(pattern_ptr: *const u8) -> Result<Pattern, ()> {
    if pattern_ptr.is_null() {
        return Ok(Pattern::new());
    }
    
    let pattern_data = unsafe { core::slice::from_raw_parts(pattern_ptr, 64) };
    serde_cbor::from_slice(pattern_data).map_err(|_| ())
}

fn copy_range_from_user(range_ptr: *const u8) -> Result<Range, ()> {
    if range_ptr.is_null() {
        return Ok(Range::new());
    }
    
    let range_data = unsafe { core::slice::from_raw_parts(range_ptr, 32) };
    serde_cbor::from_slice(range_data).map_err(|_| ())
}

fn copy_snapshot_op_from_user(op_ptr: *const u8) -> Result<String, ()> {
    if op_ptr.is_null() {
        return Ok("create".to_string());
    }
    
    let op_data = unsafe { core::slice::from_raw_parts(op_ptr, 16) };
    let op_str = core::str::from_utf8(op_data).map_err(|_| ())?;
    Ok(op_str.trim_matches(char::from(0)).to_string())
}

fn copy_entity_id_from_user(entity_ptr: *const u8) -> Result<EntityId, ()> {
    if entity_ptr.is_null() {
        return Err(());
    }
    
    let entity_data = unsafe { core::slice::from_raw_parts(entity_ptr, 16) };
    let entity_id = u128::from_le_bytes([
        entity_data[0], entity_data[1], entity_data[2], entity_data[3],
        entity_data[4], entity_data[5], entity_data[6], entity_data[7],
        entity_data[8], entity_data[9], entity_data[10], entity_data[11],
        entity_data[12], entity_data[13], entity_data[14], entity_data[15],
    ]);
    
    Ok(EntityId(entity_id))
}
