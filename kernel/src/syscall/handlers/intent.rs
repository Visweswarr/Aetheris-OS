use crate::intent::{IntentKernel, Errno};
use crate::process::ProcessId;
use crate::syscall::SyscallContext;
use crate::memory::UserBuffer;
use crate::intent::schema::{IntentV1, PlanPreviewV1, IntentStatusV1, WhyRecordV1};

pub static mut INTENT_KERNEL: Option<IntentKernel> = None;

pub fn init_intent_kernel() {
    unsafe {
        INTENT_KERNEL = Some(IntentKernel::new());
    }
}

pub fn get_intent_kernel() -> &'static IntentKernel {
    unsafe {
        INTENT_KERNEL.as_ref().expect("Intent kernel not initialized")
    }
}

#[derive(serde::Deserialize)]
struct IntentEnvelopeV1<'a> {
    v: u8,
    #[serde(borrow)]
    captoken: &'a [u8],
    #[serde(borrow)]
    payload: &'a [u8],
}

// SYS_INTENT_SUBMIT: Submit an intent and get a preview
pub fn sys_intent_submit(ctx: &mut SyscallContext) -> Result<isize, Errno> {
    let kernel = get_intent_kernel();
    
    // Get user buffer from syscall arguments
    let user_buffer = ctx.get_user_buffer(0)?;
    let buffer_size = user_buffer.len();
    
    // Validate buffer size
    if buffer_size > crate::intent::schema::INTENT_MAX_SIZE {
        crate::audit::emit(
            crate::audit::AuditCode::INTENT_SCHEMA,
            ctx.process_id,
            &format!("Intent buffer too large: {} bytes", buffer_size),
        );
        return Err(Errno::E2BIG);
    }
    
// Copy data from user space
    let mut in_bytes = vec![0u8; buffer_size];
    user_buffer.copy_to(&mut in_bytes)?;

    // Try envelope first
    let (token_opt, intent_bytes): (Option<alloc::vec::Vec<u8>>, alloc::vec::Vec<u8>) = match serde_cbor::from_slice::<IntentEnvelopeV1>(&in_bytes) {
        Ok(env) if env.v == 1 => (Some(env.captoken.to_vec()), env.payload.to_vec()),
        _ => (None, in_bytes.clone()),
    };

    // Verify token if present or required
    if let Some(tok) = token_opt.clone() {
        let pe = crate::policy::engine::caps::CapVerifier::from_env();
        match pe.verify(&tok) {
            Ok(claims) => {
                if let Err(_) = pe.require(&claims.scopes, "intent.emit") {
                    let _ = crate::policy::audit::write(&crate::policy::audit::AuditEntry{
                        ts90k: crate::time::get_time_90khz(), action: "intent.submit".into(), result: "deny".into(), reason: Some("scope".into()),
                        iss: claims.iss, sub: claims.sub, aud: claims.aud, jti_hex: hex::encode(claims.jti), scopes: claims.scopes, endpoint: None,
                    });
                    return Err(Errno::EPERM);
                }
                let _ = crate::policy::audit::write(&crate::policy::audit::AuditEntry{
                    ts90k: crate::time::get_time_90khz(), action: "intent.submit".into(), result: "allow".into(), reason: None,
                    iss: claims.iss, sub: claims.sub, aud: claims.aud, jti_hex: hex::encode(claims.jti), scopes: claims.scopes, endpoint: None,
                });
            }
            Err(_) => { return Err(Errno::EPERM); }
        }
    } else {
        // legacy path: allow only if AETHERIS_REQUIRE_TOKEN != 1
        if core::option_env!("AETHERIS_REQUIRE_TOKEN") == Some("1") { return Err(Errno::EPERM); }
    }

    // Parse CBOR intent
    let intent = match IntentV1::from_cbor(&intent_bytes) {
        Ok(intent) => intent,
        Err(_) => {
            crate::audit::emit(
                crate::audit::AuditCode::INTENT_SCHEMA,
                ctx.process_id,
                "Failed to parse CBOR intent",
            );
            return Err(Errno::EINVAL);
        }
    };
    
    // Submit intent and get preview
    let preview_handle = kernel.submit(&intent_data)?;
    
    // Log audit event
    crate::audit::emit(
        crate::audit::AuditCode::INTENT_SUBMIT,
        ctx.process_id,
        &format!("Intent submitted: id={}, type={}", intent.id, intent.intent_type),
    );
    
    // Return intent ID
    Ok(intent.id as isize)
}

// SYS_INTENT_PREVIEW: Generate preview without submitting
pub fn sys_intent_preview(ctx: &mut SyscallContext) -> Result<isize, Errno> {
    let kernel = get_intent_kernel();
    
    // Get user buffer from syscall arguments
    let user_buffer = ctx.get_user_buffer(0)?;
    let buffer_size = user_buffer.len();
    
    // Validate buffer size
    if buffer_size > crate::intent::schema::INTENT_MAX_SIZE {
        crate::audit::emit(
            crate::audit::AuditCode::INTENT_SCHEMA,
            ctx.process_id,
            &format!("Preview buffer too large: {} bytes", buffer_size),
        );
        return Err(Errno::E2BIG);
    }
    
// Copy user bytes
    let mut in_bytes = vec![0u8; buffer_size];
    user_buffer.copy_to(&mut in_bytes)?;

    // Try envelope
    let (token_opt, intent_bytes): (Option<alloc::vec::Vec<u8>>, alloc::vec::Vec<u8>) = match serde_cbor::from_slice::<IntentEnvelopeV1>(&in_bytes) {
        Ok(env) if env.v == 1 => (Some(env.captoken.to_vec()), env.payload.to_vec()),
        _ => (None, in_bytes.clone()),
    };

    if let Some(tok) = token_opt.clone() {
        let pe = crate::policy::engine::caps::CapVerifier::from_env();
        match pe.verify(&tok) {
            Ok(claims) => {
                if let Err(_) = pe.require(&claims.scopes, "intent.emit") { return Err(Errno::EPERM); }
                let _ = crate::policy::audit::write(&crate::policy::audit::AuditEntry{
                    ts90k: crate::time::get_time_90khz(), action: "intent.preview".into(), result: "allow".into(), reason: None,
                    iss: claims.iss, sub: claims.sub, aud: claims.aud, jti_hex: hex::encode(claims.jti), scopes: claims.scopes, endpoint: None,
                });
            }
            Err(_) => { return Err(Errno::EPERM); }
        }
    } else { if core::option_env!("AETHERIS_REQUIRE_TOKEN") == Some("1") { return Err(Errno::EPERM); } }

    // Generate preview
    let preview = kernel.preview(&intent_bytes)?;
    
    // Log audit event
    crate::audit::emit(
        crate::audit::AuditCode::INTENT_PREVIEW,
        ctx.process_id,
        "Intent preview generated",
    );
    
    // Return preview size (caller should use SYS_READ to get the actual preview)
    Ok(preview.serialized_size()? as isize)
}

// SYS_INTENT_STATUS: Get status of an intent
pub fn sys_intent_status(ctx: &mut SyscallContext) -> Result<isize, Errno> {
    let kernel = get_intent_kernel();
    
    // Get intent ID from syscall arguments
    let intent_id = ctx.get_arg::<u128>(0)?;
    
    // Get intent status
    let status = kernel.get_intent_status(intent_id)?;
    
    // Log audit event
    crate::audit::emit(
        crate::audit::AuditCode::INTENT_STATUS,
        ctx.process_id,
        &format!("Intent status retrieved: id={}", intent_id),
    );
    
    // Return status (caller should use SYS_READ to get the actual status)
    Ok(0)
}

// SYS_INTENT_CANCEL: Cancel an intent
pub fn sys_intent_cancel(ctx: &mut SyscallContext) -> Result<isize, Errno> {
    let kernel = get_intent_kernel();
    
    // Get intent ID from syscall arguments
    let intent_id = ctx.get_arg::<u128>(0)?;
    
    // Cancel intent
    let result = kernel.cancel_intent(intent_id)?;
    
    // Log audit event
    crate::audit::emit(
        crate::audit::AuditCode::INTENT_CANCEL,
        ctx.process_id,
        &format!("Intent cancelled: id={}", intent_id),
    );
    
    // Return result
    Ok(result as isize)
}

// SYS_WHYLOG_STREAM: Stream why-log records
pub fn sys_whylog_stream(ctx: &mut SyscallContext) -> Result<isize, Errno> {
    let kernel = get_intent_kernel();
    
    // Get cursor and max records from syscall arguments
    let cursor = ctx.get_arg::<u64>(0)?;
    let max_records = ctx.get_arg::<u32>(1)?;
    
    // Get why-log stream
    let stream_data = kernel.get_whylog_stream(cursor, max_records)?;
    
    // Log audit event
    crate::audit::emit(
        crate::audit::AuditCode::INTENT_WHYLOG_TAIL,
        ctx.process_id,
        &format!("Why-log stream requested: cursor={}, max={}", cursor, max_records),
    );
    
    // Return stream size (caller should use SYS_READ to get the actual stream)
    Ok(stream_data.len() as isize)
}

// SYS_INTENT_WHYLOG_TAIL: Get why-log tail (legacy)
pub fn sys_intent_whylog_tail(ctx: &mut SyscallContext) -> Result<isize, Errno> {
    let kernel = get_intent_kernel();
    
    // Get why-log tail
    let tail = kernel.get_whylog_tail();
    
    // Log audit event
    crate::audit::emit(
        crate::audit::AuditCode::INTENT_WHYLOG_TAIL,
        ctx.process_id,
        "Why-log tail retrieved",
    );
    
    // Return tail hash as integer (simplified for now)
    let hash_value = u64::from_le_bytes([
        tail.tail_hash[0], tail.tail_hash[1], tail.tail_hash[2], tail.tail_hash[3],
        tail.tail_hash[4], tail.tail_hash[5], tail.tail_hash[6], tail.tail_hash[7],
    ]);
    
    Ok(hash_value as isize)
}

// SYS_INTENT_STATS: Get intent statistics
pub fn sys_intent_stats(ctx: &mut SyscallContext) -> Result<isize, Errno> {
    let kernel = get_intent_kernel();
    
    // Get statistics
    let stats = kernel.get_counters();
    
    // Log audit event
    crate::audit::emit(
        crate::audit::AuditCode::INTENT_STATS,
        ctx.process_id,
        "Intent statistics retrieved",
    );
    
    // Return total operations count
    let total_ops = stats.intent_submit_ok + stats.preview_ok + stats.schema_fail + stats.policy_deny_sim;
    Ok(total_ops as isize)
}
