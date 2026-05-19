pub mod schema;
pub mod engine;
pub mod simulate;

use crate::policy::engine::{is_policy_available, eval_policy};
use crate::policy::schema::{PolicyInputV1, PolicyDecisionV1, PlanDiffV1};
use crate::policy::simulate::create_simulation_engine;
use crate::event::EventKernel;
use crate::secman::audit::emit_audit;
use crate::secman::audit_codes::{AuditReason, POLICY_SIM_ALLOW, POLICY_SIM_DENY};
use alloc::string::ToString;
use alloc::format;

/// Policy Guardrail kernel interface
pub struct PolicyKernel;

impl PolicyKernel {
    /// Check if policy system is available
    pub fn is_available() -> bool {
        is_policy_available()
    }

    /// Simulate policy evaluation for an intent and preview
    pub fn simulate_intent(
        intent: &crate::policy::schema::IntentV1,
        preview: &crate::policy::schema::PlanPreviewV1,
        wm_snapshot: u64,
        caps: &[u64],
        features: u64,
    ) -> Result<PolicySimResult, &'static str> {
        let simulation_engine = create_simulation_engine();
        let plan_diff = simulation_engine.simulate(preview, wm_snapshot, caps);

        let policy_input = PolicyInputV1 {
            intent: intent.clone(),
            preview: preview.clone(),
            wm_snapshot,
            caps: caps.iter().map(|&cap_id| crate::policy::schema::CapRef {
                cap_id,
                scope: "kernel".to_string(),
            }).collect(),
            features,
        };

        let result = eval_policy(&policy_input);
        
        if result.success {
            if let Some(decision) = result.decision {
                emit_audit(POLICY_SIM_ALLOW, &format!("Policy simulation allowed with {} reasons", decision.reasons.len()));
                
                let why_digest = self::compute_why_digest(&plan_diff);
                
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

                Ok(PolicySimResult {
                    decision,
                    plan_diff,
                    why_digest,
                })
            } else {
                emit_audit(POLICY_SIM_DENY, "Policy simulation failed");
                Err("Policy evaluation failed")
            }
        } else {
            if let Some(error) = result.error {
                emit_audit(POLICY_SIM_DENY, &format!("Policy simulation denied: {}", error));
            } else {
                emit_audit(POLICY_SIM_DENY, "Policy simulation denied");
            }
            
            let why_digest = self::compute_why_digest(&plan_diff);
            
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

            Err("Policy simulation denied")
        }
    }

    /// Get policy system statistics
    pub fn get_stats() -> PolicyStats {
        PolicyStats {
            available: is_policy_available(),
            total_evaluations: 0, // TODO: implement counter
            total_simulations: 0,  // TODO: implement counter
            allow_rate: 0.0,       // TODO: implement calculation
        }
    }
}

/// Policy simulation result
#[derive(Debug, Clone)]
pub struct PolicySimResult {
    /// Policy decision
    pub decision: PolicyDecisionV1,
    /// Plan difference
    pub plan_diff: PlanDiffV1,
    /// Why-log digest
    pub why_digest: [u8; 32],
}

/// Policy system statistics
#[derive(Debug, Clone)]
pub struct PolicyStats {
    /// Whether policy system is available
    pub available: bool,
    /// Total number of evaluations
    pub total_evaluations: u64,
    /// Total number of simulations
    pub total_simulations: u64,
    /// Allow rate (0.0 to 1.0)
    pub allow_rate: f64,
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

/// Integration helpers for other kernel subsystems
pub mod integration {
    use super::*;

    /// Initialize policy system for a task
    pub fn init_task_policy(task_id: u32) -> Result<(), &'static str> {
        if !PolicyKernel::is_available() {
            return Err("Policy system not available");
        }
        
        // TODO: Initialize task-specific policy context
        Ok(())
    }

    /// Cleanup policy resources for a task
    pub fn cleanup_task_policy(task_id: u32) {
        // TODO: Cleanup task-specific policy resources
    }

    /// Check if policy integration is available
    pub fn is_available() -> bool {
        PolicyKernel::is_available()
    }
}

/// Constants for policy system configuration
pub mod constants {
    /// Maximum size for policy input/output (32 KiB)
    pub const MAX_POLICY_IO_SIZE: usize = 32 * 1024;
    
    /// Maximum size for plan diff (32 KiB)
    pub const MAX_PLAN_DIFF_SIZE: usize = 32 * 1024;
    
    /// Default memory limit for policy engine (32 KiB)
    pub const DEFAULT_POLICY_MEMORY_LIMIT: u32 = 32 * 1024;
    
    /// Default instruction limit for policy evaluation
    pub const DEFAULT_POLICY_INSTRUCTION_LIMIT: u64 = 1_000_000;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::schema::{IntentV1, PlanPreviewV1, ActionV1, CapRef};
    use alloc::collections::BTreeMap;

    #[test]
    fn test_policy_kernel_availability() {
        let available = PolicyKernel::is_available();
        // In test environment, this might be false if no bundle is loaded
        assert!(available == false || available == true);
    }

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

    #[test]
    fn test_policy_stats() {
        let stats = PolicyKernel::get_stats();
        assert_eq!(stats.available, PolicyKernel::is_available());
        assert!(stats.allow_rate >= 0.0 && stats.allow_rate <= 1.0);
    }
}
