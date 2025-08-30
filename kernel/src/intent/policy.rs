use crate::intent::Intent;
use crate::secman::caps::CapabilitySet;
use crate::process::ProcessId;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug)]
pub enum PolicyDecision {
    Allow,
    Deny(String),
    MutateScopes(u64),
}

#[derive(Debug)]
pub enum PolicyError {
    EvaluationFailed(String),
    InvalidPolicy,
    MissingCapabilities,
}

pub struct IntentPolicy {
    policy_wasm: Option<Vec<u8>>,
    dev_mode: bool,
}

impl IntentPolicy {
    pub fn new() -> Self {
        Self {
            policy_wasm: None,
            dev_mode: cfg!(debug_assertions),
        }
    }
    
    pub fn load_policy(&mut self, wasm_blob: Vec<u8>) -> Result<(), PolicyError> {
        if wasm_blob.is_empty() {
            return Err(PolicyError::InvalidPolicy);
        }
        
        self.policy_wasm = Some(wasm_blob);
        Ok(())
    }
    
    pub fn evaluate_intent(
        &self,
        intent: &Intent,
        pid: ProcessId,
    ) -> Result<PolicyDecision, PolicyError> {
        if self.dev_mode {
            return self.evaluate_dev_mode(intent, pid);
        }
        
        if let Some(ref wasm) = self.policy_wasm {
            self.evaluate_wasm_policy(intent, pid, wasm)
        } else {
            Err(PolicyError::InvalidPolicy)
        }
    }
    
    fn evaluate_dev_mode(
        &self,
        intent: &Intent,
        _pid: ProcessId,
    ) -> Result<PolicyDecision, PolicyError> {
        // Development mode: simple policy rules
        if intent.header.scope_flags & 0x8000_0000_0000_0000 != 0 {
            return Ok(PolicyDecision::Deny("Development mode: restricted scope".to_string()));
        }
        
        if intent.header.ttl_ms > 300_000 {
            return Ok(PolicyDecision::Deny("Development mode: TTL too long".to_string()));
        }
        
        if intent.header.text_len > 1024 {
            return Ok(PolicyDecision::MutateScopes(0x0000_0000_0000_0001));
        }
        
        Ok(PolicyDecision::Allow)
    }
    
    fn evaluate_wasm_policy(
        &self,
        intent: &Intent,
        pid: ProcessId,
        wasm: &[u8],
    ) -> Result<PolicyDecision, PolicyError> {
        // In a real implementation, this would:
        // 1. Load the WASM module
        // 2. Set up the execution environment
        // 3. Pass intent data and process context
        // 4. Execute the policy
        // 5. Return the decision
        
        // For now, we'll simulate the policy evaluation
        let policy_input = self.build_policy_input(intent, pid);
        
        match self.simulate_policy_evaluation(&policy_input) {
            Ok(decision) => Ok(decision),
            Err(e) => Err(PolicyError::EvaluationFailed(e)),
        }
    }
    
    fn build_policy_input(&self, intent: &Intent, pid: ProcessId) -> PolicyInput {
        PolicyInput {
            intent_id: intent.header.intent_id,
            lane: intent.header.lane,
            scope_flags: intent.header.scope_flags,
            ttl_ms: intent.header.ttl_ms,
            text_len: intent.header.text_len,
            plan_len: intent.header.plan_len,
            preview_len: intent.header.preview_len,
            whylog_len: intent.header.whylog_len,
            owner_pid: pid,
            timestamp: crate::time::MonotonicTime::now().ticks(),
        }
    }
    
    fn simulate_policy_evaluation(&self, input: &PolicyInput) -> Result<PolicyDecision, String> {
        // Simulate policy evaluation logic
        if input.scope_flags & 0x4000_0000_0000_0000 != 0 {
            return Ok(PolicyDecision::Deny("Restricted scope not allowed".to_string()));
        }
        
        if input.ttl_ms > 600_000 {
            return Ok(PolicyDecision::Deny("TTL exceeds maximum allowed".to_string()));
        }
        
        if input.text_len > 8192 {
            return Ok(PolicyDecision::MutateScopes(0x0000_0000_0000_0002));
        }
        
        if input.lane == 0 && input.scope_flags & 0x0000_0000_0000_0001 == 0 {
            return Ok(PolicyDecision::Deny("RT lane requires elevated capabilities".to_string()));
        }
        
        Ok(PolicyDecision::Allow)
    }
    
    pub fn set_dev_mode(&mut self, enabled: bool) {
        self.dev_mode = enabled;
    }
    
    pub fn is_dev_mode(&self) -> bool {
        self.dev_mode
    }
}

#[derive(Debug)]
struct PolicyInput {
    intent_id: u128,
    lane: u8,
    scope_flags: u64,
    ttl_ms: u32,
    text_len: u32,
    plan_len: u32,
    preview_len: u16,
    whylog_len: u16,
    owner_pid: ProcessId,
    timestamp: u64,
}

impl Default for IntentPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::wire::IntentHeaderV1;
    use crate::intent::Intent;
    
    fn create_test_intent(scope_flags: u64, ttl_ms: u32, text_len: u32) -> Intent {
        let mut header = IntentHeaderV1::default();
        header.intent_id = 1;
        header.scope_flags = scope_flags;
        header.ttl_ms = ttl_ms;
        header.text_len = text_len;
        
        Intent::new(
            header,
            vec![0u8; text_len as usize],
            vec![],
            vec![],
            vec![],
            123,
        )
    }
    
    #[test]
    fn test_dev_mode_policy() {
        let mut policy = IntentPolicy::new();
        policy.set_dev_mode(true);
        
        let intent = create_test_intent(0, 1000, 100);
        let decision = policy.evaluate_intent(&intent, 123).unwrap();
        
        match decision {
            PolicyDecision::Allow => {},
            _ => panic!("Expected Allow decision"),
        }
    }
    
    #[test]
    fn test_dev_mode_restricted_scope() {
        let mut policy = IntentPolicy::new();
        policy.set_dev_mode(true);
        
        let intent = create_test_intent(0x8000_0000_0000_0000, 1000, 100);
        let decision = policy.evaluate_intent(&intent, 123).unwrap();
        
        match decision {
            PolicyDecision::Deny(_) => {},
            _ => panic!("Expected Deny decision"),
        }
    }
    
    #[test]
    fn test_dev_mode_ttl_limit() {
        let mut policy = IntentPolicy::new();
        policy.set_dev_mode(true);
        
        let intent = create_test_intent(0, 400_000, 100);
        let decision = policy.evaluate_intent(&intent, 123).unwrap();
        
        match decision {
            PolicyDecision::Deny(_) => {},
            _ => panic!("Expected Deny decision"),
        }
    }
    
    #[test]
    fn test_dev_mode_mutate_scopes() {
        let mut policy = IntentPolicy::new();
        policy.set_dev_mode(true);
        
        let intent = create_test_intent(0, 1000, 2048);
        let decision = policy.evaluate_intent(&intent, 123).unwrap();
        
        match decision {
            PolicyDecision::MutateScopes(flags) => {
                assert_eq!(flags, 0x0000_0000_0000_0001);
            },
            _ => panic!("Expected MutateScopes decision"),
        }
    }
    
    #[test]
    fn test_simulated_policy_evaluation() {
        let policy = IntentPolicy::new();
        
        let intent = create_test_intent(0, 1000, 100);
        let decision = policy.evaluate_intent(&intent, 123).unwrap();
        
        match decision {
            PolicyDecision::Allow => {},
            _ => panic!("Expected Allow decision"),
        }
    }
    
    #[test]
    fn test_simulated_policy_restricted_scope() {
        let policy = IntentPolicy::new();
        
        let intent = create_test_intent(0x4000_0000_0000_0000, 1000, 100);
        let decision = policy.evaluate_intent(&intent, 123).unwrap();
        
        match decision {
            PolicyDecision::Deny(_) => {},
            _ => panic!("Expected Deny decision"),
        }
    }
    
    #[test]
    fn test_simulated_policy_ttl_limit() {
        let policy = IntentPolicy::new();
        
        let intent = create_test_intent(0, 700_000, 100);
        let decision = policy.evaluate_intent(&intent, 123).unwrap();
        
        match decision {
            PolicyDecision::Deny(_) => {},
            _ => panic!("Expected Deny decision"),
        }
    }
    
    #[test]
    fn test_simulated_policy_mutate_scopes() {
        let policy = IntentPolicy::new();
        
        let intent = create_test_intent(0, 1000, 16384);
        let decision = policy.evaluate_intent(&intent, 123).unwrap();
        
        match decision {
            PolicyDecision::MutateScopes(flags) => {
                assert_eq!(flags, 0x0000_0000_0000_0002);
            },
            _ => panic!("Expected MutateScopes decision"),
        }
    }
    
    #[test]
    fn test_simulated_policy_rt_lane_restriction() {
        let policy = IntentPolicy::new();
        
        let mut header = IntentHeaderV1::default();
        header.intent_id = 1;
        header.lane = 0; // RT lane
        header.scope_flags = 0; // No elevated capabilities
        
        let intent = Intent::new(
            header,
            vec![],
            vec![],
            vec![],
            vec![],
            123,
        );
        
        let decision = policy.evaluate_intent(&intent, 123).unwrap();
        
        match decision {
            PolicyDecision::Deny(_) => {},
            _ => panic!("Expected Deny decision for RT lane without capabilities"),
        }
    }
}
