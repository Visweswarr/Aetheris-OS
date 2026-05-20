use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use crate::lazy_static;
use crate::policy::schema::{PolicyInputV1, PolicyDecisionV1, serialize_policy_input, deserialize_policy_decision};

/// WASM policy engine configuration
pub struct WasmPolicyEngine {
    /// WASM module instance
    module: Option<WasmModule>,
    /// Instruction meter
    inst_meter: AtomicU64,
    /// Memory limit in bytes
    mem_limit: u32,
    /// Bundle hash for verification
    bundle_hash: [u8; 32],
    /// Whether engine is loaded
    loaded: bool,
}

/// WASM module wrapper
struct WasmModule {
    /// Module data
    data: Vec<u8>,
    /// Module hash
    hash: [u8; 32],
}

/// Policy evaluation result
#[derive(Debug, Clone)]
pub struct PolicyEvalResult {
    /// Whether evaluation succeeded
    pub success: bool,
    /// Decision if successful
    pub decision: Option<PolicyDecisionV1>,
    /// Error message if failed
    pub error: Option<String>,
    /// Instructions executed
    pub instructions: u64,
}

impl WasmPolicyEngine {
    /// Create a new policy engine
    pub fn new(mem_limit: u32) -> Self {
        Self {
            module: None,
            inst_meter: AtomicU64::new(0),
            mem_limit,
            bundle_hash: [0; 32],
            loaded: false,
        }
    }

    /// Load a policy bundle (dev/test mode only)
    pub fn load_bundle(&mut self, wasm_data: &[u8], expected_hash: [u8; 32]) -> Result<(), &'static str> {
        if wasm_data.len() > self.mem_limit as usize {
            return Err("WASM bundle exceeds memory limit");
        }

        let actual_hash = self.compute_hash(wasm_data);
        if actual_hash != expected_hash {
            return Err("WASM bundle hash mismatch");
        }

        self.module = Some(WasmModule {
            data: wasm_data.to_vec(),
            hash: expected_hash,
        });
        self.bundle_hash = expected_hash;
        self.loaded = true;

        Ok(())
    }

    /// Evaluate a policy input
    pub fn eval(&self, input: &PolicyInputV1) -> PolicyEvalResult {
        if !self.loaded {
            return PolicyEvalResult {
                success: false,
                decision: None,
                error: Some("Policy engine not loaded".to_string()),
                instructions: 0,
            };
        }

        let start_inst = self.inst_meter.load(Ordering::SeqCst);
        
        // Serialize input to CBOR
        let input_data = match serialize_policy_input(input) {
            Ok(data) => data,
            Err(_) => return PolicyEvalResult {
                success: false,
                decision: None,
                error: Some("Failed to serialize input".to_string()),
                instructions: 0,
            },
        };

        // Simulate WASM execution (stub implementation)
        let decision = self.simulate_wasm_eval(&input_data);
        
        let end_inst = self.inst_meter.load(Ordering::SeqCst);
        let instructions = end_inst - start_inst;

        match decision {
            Ok(dec) => PolicyEvalResult {
                success: true,
                decision: Some(dec),
                error: None,
                instructions,
            },
            Err(e) => PolicyEvalResult {
                success: false,
                decision: None,
                error: Some(e.to_string()),
                instructions,
            },
        }
    }

    /// Check if engine is loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Get bundle hash
    pub fn get_bundle_hash(&self) -> [u8; 32] {
        self.bundle_hash
    }

    /// Get instruction count
    pub fn get_instruction_count(&self) -> u64 {
        self.inst_meter.load(Ordering::SeqCst)
    }

    /// Reset instruction meter
    pub fn reset_meter(&self) {
        self.inst_meter.store(0, Ordering::SeqCst);
    }

    /// Simulate WASM evaluation (stub implementation)
    fn simulate_wasm_eval(&self, input_data: &[u8]) -> Result<PolicyDecisionV1, &'static str> {
        // This is a stub implementation that simulates policy evaluation
        // In a real implementation, this would execute the WASM module
        
        // Simulate instruction execution
        self.inst_meter.fetch_add(input_data.len() as u64, Ordering::SeqCst);
        
        // Simple policy logic based on input size and content
        if input_data.len() > 1024 {
            return Ok(PolicyDecisionV1 {
                allow: false,
                reasons: vec!["Input too large".to_string()],
                redactions: vec![],
            });
        }

        // Check for dangerous intent types
        let input_str = String::from_utf8_lossy(input_data);
        if input_str.contains("dangerous") || input_str.contains("admin") {
            return Ok(PolicyDecisionV1 {
                allow: false,
                reasons: vec!["Dangerous operation detected".to_string()],
                redactions: vec![],
            });
        }

        // Default allow with minimal redactions
        Ok(PolicyDecisionV1 {
            allow: true,
            reasons: vec!["Policy evaluation passed".to_string()],
            redactions: vec![],
        })
    }

    /// Compute hash of data
    fn compute_hash(&self, data: &[u8]) -> [u8; 32] {
        use blake3::Hasher;
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        hasher.finalize().into()
    }
}

/// Global policy engine instance
lazy_static! {
    pub static ref POLICY_ENGINE: Mutex<WasmPolicyEngine> = Mutex::new(WasmPolicyEngine::new(32 * 1024));
}

/// Load the default embedded policy bundle
pub fn load_default_bundle() -> Result<(), &'static str> {
    // In a real implementation, this would load the embedded policy bundle
    // For now, we'll create a minimal stub bundle
    
    let mut engine = POLICY_ENGINE.lock();
    
    // Create a minimal stub bundle
    let stub_bundle = b"stub_policy_bundle";
    let stub_hash = {
        use blake3::Hasher;
        let mut hasher = blake3::Hasher::new();
        hasher.update(stub_bundle);
        hasher.finalize().into()
    };
    
    engine.load_bundle(stub_bundle, stub_hash)
}

/// Evaluate policy input using the global engine
pub fn eval_policy(input: &PolicyInputV1) -> PolicyEvalResult {
    let engine = POLICY_ENGINE.lock();
    engine.eval(input)
}

/// Check if policy engine is available
pub fn is_policy_available() -> bool {
    let engine = POLICY_ENGINE.lock();
    engine.is_loaded()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::schema::{IntentV1, PlanPreviewV1, ActionV1, CapRef};
    use alloc::collections::BTreeMap;

    #[test]
    fn test_policy_engine_creation() {
        let engine = WasmPolicyEngine::new(64 * 1024);
        assert_eq!(engine.mem_limit, 64 * 1024);
        assert!(!engine.is_loaded());
    }

    #[test]
    fn test_policy_bundle_loading() {
        let mut engine = WasmPolicyEngine::new(1024);
        let bundle_data = b"test_policy";
        let hash = {
            use blake3::Hasher;
            let mut hasher = blake3::Hasher::new();
            hasher.update(bundle_data);
            hasher.finalize().into()
        };
        
        assert!(engine.load_bundle(bundle_data, hash).is_ok());
        assert!(engine.is_loaded());
        assert_eq!(engine.get_bundle_hash(), hash);
    }

    #[test]
    fn test_policy_bundle_hash_mismatch() {
        let mut engine = WasmPolicyEngine::new(1024);
        let bundle_data = b"test_policy";
        let wrong_hash = [0u8; 32];
        
        assert!(engine.load_bundle(bundle_data, wrong_hash).is_err());
        assert!(!engine.is_loaded());
    }

    #[test]
    fn test_policy_evaluation() {
        let mut engine = WasmPolicyEngine::new(1024);
        let bundle_data = b"test_policy";
        let hash = {
            use blake3::Hasher;
            let mut hasher = blake3::Hasher::new();
            hasher.update(bundle_data);
            hasher.finalize().into()
        };
        
        engine.load_bundle(bundle_data, hash).unwrap();
        
        let input = PolicyInputV1 {
            intent: IntentV1 {
                id: 123,
                description: "test".to_string(),
                intent_type: 1,
                priority: 1,
                requested_caps: vec![],
                metadata: BTreeMap::new(),
            },
            preview: PlanPreviewV1 {
                plan_id: 456,
                actions: vec![ActionV1 { kind: 1, params: BTreeMap::new() }],
                cost: 10,
                risks: vec![],
                notes: vec![],
            },
            wm_snapshot: 789,
            caps: vec![],
            features: 0,
        };
        
        let result = engine.eval(&input);
        assert!(result.success);
        assert!(result.decision.is_some());
        assert!(result.instructions > 0);
    }

    #[test]
    fn test_policy_evaluation_unloaded() {
        let engine = WasmPolicyEngine::new(1024);
        let input = PolicyInputV1 {
            intent: IntentV1 {
                id: 123,
                description: "test".to_string(),
                intent_type: 1,
                priority: 1,
                requested_caps: vec![],
                metadata: BTreeMap::new(),
            },
            preview: PlanPreviewV1 {
                plan_id: 456,
                actions: vec![],
                cost: 10,
                risks: vec![],
                notes: vec![],
            },
            wm_snapshot: 789,
            caps: vec![],
            features: 0,
        };
        
        let result = engine.eval(&input);
        assert!(!result.success);
        assert!(result.decision.is_none());
        assert!(result.error.is_some());
    }
}
