use crate::secman::audit_codes::AuditReason;
use crate::secman::audit_codes::{
    POLICY_BUNDLE_LOAD_OK, POLICY_BUNDLE_HASH_MISMATCH, POLICY_EVAL_ALLOW, POLICY_EVAL_DENY,
    POLICY_SIM_ALLOW, POLICY_SIM_DENY,
};
use crate::policy::engine::{load_default_bundle, eval_policy, is_policy_available};
use crate::policy::schema::{PolicyInputV1, PolicyDecisionV1, serialize_policy_input, deserialize_policy_decision};
use crate::policy::simulate::{create_simulation_engine, PlanDiffV1};
use crate::event::EventKernel;
use crate::secman::cap_store::CapStore;
use crate::secman::audit::emit_audit;
use crate::syscall::handlers::world::{CAP_POLICY_LOAD, CAP_POLICY_EVAL, CAP_POLICY_SIMULATE};

/// Policy load bundle request
#[derive(Debug)]
struct PolicyLoadBundleRequest {
    /// WASM bundle data pointer
    wasm_ptr: usize,
    /// WASM bundle length
    wasm_len: usize,
    /// Expected hash
    expected_hash: [u8; 32],
}

/// Policy evaluation request
#[derive(Debug)]
struct PolicyEvalRequest {
    /// Input data pointer
    input_ptr: usize,
    /// Input data length
    input_len: usize,
}

/// Policy simulation request
#[derive(Debug)]
struct PolicySimRequest {
    /// Intent data pointer
    intent_ptr: usize,
    /// Intent data length
    intent_len: usize,
    /// Preview data pointer
    preview_ptr: usize,
    /// Preview data length
    preview_len: usize,
}

/// Policy simulation response
#[derive(Debug)]
struct PolicySimResponse {
    /// Whether simulation was allowed
    allowed: bool,
    /// Plan difference
    plan_diff: PlanDiffV1,
    /// Why-log digest
    why_digest: [u8; 32],
}

/// Load a policy bundle (dev/test mode only)
pub fn sys_policy_load_bundle(
    wasm_ptr: usize,
    wasm_len: usize,
    hash_ptr: usize,
) -> Result<u64, i32> {
    if !CapStore::has_capability(CAP_POLICY_LOAD) {
        emit_audit(POLICY_BUNDLE_HASH_MISMATCH, "Missing CAP_POLICY_LOAD capability");
        return Err(libc::EPERM);
    }

    if wasm_len > 32 * 1024 {
        return Err(libc::E2BIG);
    }

    let mut hash = [0u8; 32];
    if !copy_from_user(hash_ptr, &mut hash) {
        return Err(libc::EFAULT);
    }

    let mut wasm_data = vec![0u8; wasm_len];
    if !copy_from_user(wasm_ptr, &mut wasm_data) {
        return Err(libc::EFAULT);
    }

    match load_default_bundle() {
        Ok(()) => {
            emit_audit(POLICY_BUNDLE_LOAD_OK, "Policy bundle loaded successfully");
            Ok(0)
        }
        Err(e) => {
            emit_audit(POLICY_BUNDLE_HASH_MISMATCH, &format!("Failed to load bundle: {}", e));
            Err(libc::EINVAL)
        }
    }
}

/// Evaluate a policy input
pub fn sys_policy_eval(
    input_ptr: usize,
    input_len: usize,
) -> Result<u64, i32> {
    if !CapStore::has_capability(CAP_POLICY_EVAL) {
        emit_audit(POLICY_EVAL_DENY, "Missing CAP_POLICY_EVAL capability");
        return Err(libc::EPERM);
    }

    if input_len > 32 * 1024 {
        return Err(libc::E2BIG);
    }

    if !is_policy_available() {
        return Err(libc::ENODEV);
    }

    let mut input_data = vec![0u8; input_len];
    if !copy_from_user(input_ptr, &mut input_data) {
        return Err(libc::EFAULT);
    }

    let input = match deserialize_policy_input(&input_data) {
        Ok(input) => input,
        Err(_) => return Err(libc::EINVAL),
    };

    let result = eval_policy(&input);
    
    if result.success {
        if let Some(decision) = result.decision {
            emit_audit(POLICY_EVAL_ALLOW, &format!("Policy evaluation allowed with {} reasons", decision.reasons.len()));
            Ok(1) // Allowed
        } else {
            emit_audit(POLICY_EVAL_DENY, "Policy evaluation failed");
            Err(libc::EINVAL)
        }
    } else {
        if let Some(error) = result.error {
            emit_audit(POLICY_EVAL_DENY, &format!("Policy evaluation denied: {}", error));
        } else {
            emit_audit(POLICY_EVAL_DENY, "Policy evaluation denied");
        }
        Ok(0) // Denied
    }
}

/// Simulate policy evaluation for an intent and preview
pub fn sys_policy_simulate(
    intent_ptr: usize,
    intent_len: usize,
    preview_ptr: usize,
    preview_len: usize,
) -> Result<u64, i32> {
    if !CapStore::has_capability(CAP_POLICY_SIMULATE) {
        emit_audit(POLICY_SIM_DENY, "Missing CAP_POLICY_SIMULATE capability");
        return Err(libc::EPERM);
    }

    if intent_len > 32 * 1024 || preview_len > 32 * 1024 {
        return Err(libc::E2BIG);
    }

    if !is_policy_available() {
        return Err(libc::ENODEV);
    }

    let mut intent_data = vec![0u8; intent_len];
    let mut preview_data = vec![0u8; preview_len];

    if !copy_from_user(intent_ptr, &mut intent_data) {
        return Err(libc::EFAULT);
    }
    if !copy_from_user(preview_ptr, &mut preview_data) {
        return Err(libc::EFAULT);
    }

    let intent = match deserialize_intent(&intent_data) {
        Ok(intent) => intent,
        Err(_) => return Err(libc::EINVAL),
    };

    let preview = match deserialize_preview(&preview_data) {
        Ok(preview) => preview,
        Err(_) => return Err(libc::EINVAL),
    };

    let simulation_engine = create_simulation_engine();
    let plan_diff = simulation_engine.simulate(&preview, 0, &[]);

    let policy_input = PolicyInputV1 {
        intent,
        preview,
        wm_snapshot: 0,
        caps: vec![],
        features: 0,
    };

    let result = eval_policy(&policy_input);
    
    if result.success {
        if let Some(decision) = result.decision {
            emit_audit(POLICY_SIM_ALLOW, &format!("Policy simulation allowed with {} reasons", decision.reasons.len()));
            
            let why_digest = compute_why_digest(&plan_diff);
            
            EventKernel::publish_sys_event(
                "policy.simulate",
                crate::event::queue::Lane::MED,
                serde_json::json!({
                    "decision": "allow",
                    "reasons_count": decision.reasons.len(),
                    "plan_diff_size": plan_diff.total_size(),
                    "why_digest": hex::encode(why_digest)
                }).to_string().into_bytes(),
            ).ok();

            Ok(1) // Allowed
        } else {
            emit_audit(POLICY_SIM_DENY, "Policy simulation failed");
            Err(libc::EINVAL)
        }
    } else {
        if let Some(error) = result.error {
            emit_audit(POLICY_SIM_DENY, &format!("Policy simulation denied: {}", error));
        } else {
            emit_audit(POLICY_SIM_DENY, "Policy simulation denied");
        }
        
        let why_digest = compute_why_digest(&plan_diff);
        
        EventKernel::publish_sys_event(
            "policy.simulate",
            crate::event::queue::Lane::MED,
            serde_json::json!({
                "decision": "deny",
                "error": result.error.unwrap_or_else(|| "Unknown error".to_string()),
                "plan_diff_size": plan_diff.total_size(),
                "why_digest": hex::encode(why_digest)
            }).to_string().into_bytes(),
        ).ok();

        Ok(0) // Denied
    }
}

/// Copy data from user space
fn copy_from_user<T>(user_ptr: usize, kernel_buf: &mut [u8]) -> bool {
    // This is a simplified implementation
    // In a real kernel, this would perform proper bounds checking and copy
    if user_ptr == 0 {
        return false;
    }
    
    // Simulate successful copy
    true
}

/// Deserialize intent from CBOR
fn deserialize_intent(data: &[u8]) -> Result<crate::policy::schema::IntentV1, &'static str> {
    serde_cbor::from_slice(data).map_err(|_| "Failed to deserialize intent")
}

/// Deserialize preview from CBOR
fn deserialize_preview(data: &[u8]) -> Result<crate::policy::schema::PlanPreviewV1, &'static str> {
    serde_cbor::from_slice(data).map_err(|_| "Failed to deserialize preview")
}

/// Compute why-log digest for plan diff
fn compute_why_digest(plan_diff: &PlanDiffV1) -> [u8; 32] {
    use blake3::Hasher;
    
    let mut hasher = blake3::Hasher::new();
    hasher.update(&plan_diff.total_size().to_le_bytes());
    hasher.update(&plan_diff.change_count().to_le_bytes());
    
    for action in &plan_diff.adds {
        hasher.update(&action.kind.to_le_bytes());
        for (key, value) in &action.params {
            hasher.update(key.as_bytes());
            hasher.update(value.as_bytes());
        }
    }
    
    for remove_idx in &plan_diff.removes {
        hasher.update(&remove_idx.to_le_bytes());
    }
    
    for edit in &plan_diff.edits {
        hasher.update(&edit.action_index.to_le_bytes());
        hasher.update(&edit.new_action.kind.to_le_bytes());
    }
    
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_why_digest_computation() {
        let plan_diff = PlanDiffV1 {
            adds: vec![],
            removes: vec![],
            edits: vec![],
        };
        
        let digest1 = compute_why_digest(&plan_diff);
        let digest2 = compute_why_digest(&plan_diff);
        
        assert_eq!(digest1, digest2, "Why-log digest should be deterministic");
    }
}
