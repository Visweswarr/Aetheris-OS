use polymera_kernel::policy::engine::*;
use polymera_kernel::policy::schema::*;
use std::collections::BTreeMap;

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
fn test_policy_bundle_size_limit() {
    let mut engine = WasmPolicyEngine::new(100);
    let bundle_data = b"this bundle is too large for the memory limit";
    let hash = [0u8; 32];
    
    assert!(engine.load_bundle(bundle_data, hash).is_err());
    assert!(!engine.is_loaded());
}

#[test]
fn test_policy_evaluation_loaded() {
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

#[test]
fn test_policy_evaluation_large_input() {
    let mut engine = WasmPolicyEngine::new(1024);
    let bundle_data = b"test_policy";
    let hash = {
        use blake3::Hasher;
        let mut hasher = blake3::Hasher::new();
        hasher.update(bundle_data);
        hasher.finalize().into()
    };
    
    engine.load_bundle(bundle_data, hash).unwrap();
    
    let large_description = "x".repeat(2000); // Large input
    let input = PolicyInputV1 {
        intent: IntentV1 {
            id: 123,
            description: large_description,
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
    // Should be denied due to large input
    assert!(!result.success);
    assert!(result.decision.is_some());
    if let Some(decision) = result.decision {
        assert!(!decision.allow);
        assert!(decision.reasons.iter().any(|r| r.contains("too large")));
    }
}

#[test]
fn test_policy_evaluation_dangerous_intent() {
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
            description: "dangerous operation".to_string(),
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
    // Should be denied due to dangerous content
    assert!(!result.success);
    assert!(result.decision.is_some());
    if let Some(decision) = result.decision {
        assert!(!decision.allow);
        assert!(decision.reasons.iter().any(|r| r.contains("Dangerous")));
    }
}

#[test]
fn test_policy_engine_instruction_metering() {
    let mut engine = WasmPolicyEngine::new(1024);
    let bundle_data = b"test_policy";
    let hash = {
        use blake3::Hasher;
        let mut hasher = blake3::Hasher::new();
        hasher.update(bundle_data);
        hasher.finalize().into()
    };
    
    engine.load_bundle(bundle_data, hash).unwrap();
    
    let initial_count = engine.get_instruction_count();
    
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
    
    let _result = engine.eval(&input);
    let final_count = engine.get_instruction_count();
    
    assert!(final_count > initial_count, "Instruction count should increase after evaluation");
}

#[test]
fn test_policy_engine_reset_meter() {
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
            actions: vec![],
            cost: 10,
            risks: vec![],
            notes: vec![],
        },
        wm_snapshot: 789,
        caps: vec![],
        features: 0,
    };
    
    let _result = engine.eval(&input);
    let count_before_reset = engine.get_instruction_count();
    
    engine.reset_meter();
    let count_after_reset = engine.get_instruction_count();
    
    assert!(count_before_reset > 0, "Instruction count should be non-zero before reset");
    assert_eq!(count_after_reset, 0, "Instruction count should be zero after reset");
}
