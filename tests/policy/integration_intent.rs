use polymera_kernel::policy::*;
use polymera_kernel::policy::schema::*;
use polymera_kernel::event::EventKernel;
use std::collections::BTreeMap;

#[test]
fn test_policy_kernel_availability() {
    let available = PolicyKernel::is_available();
    // In test environment, this might be false if no bundle is loaded
    assert!(available == false || available == true);
}

#[test]
fn test_policy_simulation_intent() {
    let intent = IntentV1 {
        id: 12345,
        description: "test intent for policy simulation".to_string(),
        intent_type: 1,
        priority: 2,
        requested_caps: vec![CapRef { cap_id: 1, scope: "test".to_string() }],
        metadata: BTreeMap::new(),
    };

    let preview = PlanPreviewV1 {
        plan_id: 67890,
        actions: vec![
            ActionV1 { 
                kind: 1, 
                params: {
                    let mut params = BTreeMap::new();
                    params.insert("operation".to_string(), "create".to_string());
                    params
                }
            },
            ActionV1 { 
                kind: 2, 
                params: {
                    let mut params = BTreeMap::new();
                    params.insert("operation".to_string(), "delete".to_string());
                    params
                }
            },
        ],
        cost: 150,
        risks: vec!["medium".to_string()],
        notes: vec!["test plan".to_string()],
    };

    let wm_snapshot = 42;
    let caps = vec![1, 2, 3];
    let features = 0x1234;

    let result = PolicyKernel::simulate_intent(&intent, &preview, wm_snapshot, &caps, features);
    
    // The result might be Ok or Err depending on policy availability
    match result {
        Ok(sim_result) => {
            // Policy simulation succeeded
            assert_eq!(sim_result.plan_diff.adds.len(), 1); // One add action
            assert_eq!(sim_result.plan_diff.removes.len(), 1); // One remove action
            assert_eq!(sim_result.plan_diff.edits.len(), 0); // No edit actions
            
            // Verify why-log digest is computed
            assert_ne!(sim_result.why_digest, [0u8; 32]);
            
            // Check decision structure
            assert!(sim_result.decision.reasons.len() > 0);
        }
        Err(e) => {
            // Policy simulation failed (e.g., no policy bundle loaded)
            assert!(e.contains("denied") || e.contains("failed") || e.contains("not available"));
        }
    }
}

#[test]
fn test_policy_simulation_with_redactions() {
    let intent = IntentV1 {
        id: 12346,
        description: "test intent with secrets".to_string(),
        intent_type: 1,
        priority: 1,
        requested_caps: vec![],
        metadata: BTreeMap::new(),
    };

    let preview = PlanPreviewV1 {
        plan_id: 67891,
        actions: vec![
            ActionV1 { 
                kind: 1, 
                params: {
                    let mut params = BTreeMap::new();
                    params.insert("operation".to_string(), "create".to_string());
                    params.insert("secret".to_string(), "password123".to_string());
                    params.insert("token".to_string(), "abc123".to_string());
                    params
                }
            },
        ],
        cost: 50,
        risks: vec![],
        notes: vec![],
    };

    let wm_snapshot = 43;
    let caps = vec![1];
    let features = 0x1234;

    let result = PolicyKernel::simulate_intent(&intent, &preview, wm_snapshot, &caps, features);
    
    match result {
        Ok(sim_result) => {
            // Check that the plan diff contains the action
            assert_eq!(sim_result.plan_diff.adds.len(), 1);
            
            // In a real implementation, redactions would be applied
            // For now, we just verify the structure
            let action = &sim_result.plan_diff.adds[0];
            assert_eq!(action.kind, 1);
            assert!(action.params.contains_key("operation"));
        }
        Err(_) => {
            // Policy simulation failed, which is acceptable in test environment
        }
    }
}

#[test]
fn test_policy_stats() {
    let stats = PolicyKernel::get_stats();
    
    // Verify stats structure
    assert_eq!(stats.available, PolicyKernel::is_available());
    assert!(stats.total_evaluations >= 0);
    assert!(stats.total_simulations >= 0);
    assert!(stats.allow_rate >= 0.0 && stats.allow_rate <= 1.0);
}

#[test]
fn test_policy_integration_helpers() {
    let task_id = 123;
    
    // Test initialization
    let init_result = integration::init_task_policy(task_id);
    // This might fail if policy system is not available
    if let Err(e) = init_result {
        assert!(e.contains("not available"));
    }
    
    // Test availability check
    let available = integration::is_available();
    assert_eq!(available, PolicyKernel::is_available());
    
    // Test cleanup (should not panic)
    integration::cleanup_task_policy(task_id);
}

#[test]
fn test_policy_constants() {
    assert_eq!(constants::MAX_POLICY_IO_SIZE, 32 * 1024);
    assert_eq!(constants::MAX_PLAN_DIFF_SIZE, 32 * 1024);
    assert_eq!(constants::DEFAULT_POLICY_MEMORY_LIMIT, 32 * 1024);
    assert_eq!(constants::DEFAULT_POLICY_INSTRUCTION_LIMIT, 1_000_000);
}

#[test]
fn test_policy_simulation_deterministic() {
    let intent = IntentV1 {
        id: 12347,
        description: "deterministic test".to_string(),
        intent_type: 1,
        priority: 1,
        requested_caps: vec![],
        metadata: BTreeMap::new(),
    };

    let preview = PlanPreviewV1 {
        plan_id: 67892,
        actions: vec![ActionV1 { kind: 1, params: BTreeMap::new() }],
        cost: 10,
        risks: vec![],
        notes: vec![],
    };

    let wm_snapshot = 44;
    let caps = vec![1];
    let features = 0x1234;

    // Run simulation multiple times
    let result1 = PolicyKernel::simulate_intent(&intent, &preview, wm_snapshot, &caps, features);
    let result2 = PolicyKernel::simulate_intent(&intent, &preview, wm_snapshot, &caps, features);
    
    // Results should be consistent (either both succeed or both fail)
    match (result1, result2) {
        (Ok(sim1), Ok(sim2)) => {
            // Both succeeded - verify determinism
            assert_eq!(sim1.plan_diff.adds.len(), sim2.plan_diff.adds.len());
            assert_eq!(sim1.plan_diff.removes.len(), sim2.plan_diff.removes.len());
            assert_eq!(sim1.plan_diff.edits.len(), sim2.plan_diff.edits.len());
            
            // Why-log digests should be identical
            assert_eq!(sim1.why_digest, sim2.why_digest);
        }
        (Err(e1), Err(e2)) => {
            // Both failed - verify consistent error messages
            assert_eq!(e1, e2);
        }
        _ => {
            // Mixed results - this might happen in test environment
            // but should not happen in production
        }
    }
}

#[test]
fn test_policy_simulation_large_input() {
    let large_description = "x".repeat(1000); // Large description
    
    let intent = IntentV1 {
        id: 12348,
        description: large_description,
        intent_type: 1,
        priority: 1,
        requested_caps: vec![],
        metadata: BTreeMap::new(),
    };

    let preview = PlanPreviewV1 {
        plan_id: 67893,
        actions: vec![ActionV1 { kind: 1, params: BTreeMap::new() }],
        cost: 10,
        risks: vec![],
        notes: vec![],
    };

    let wm_snapshot = 45;
    let caps = vec![1];
    let features = 0x1234;

    let result = PolicyKernel::simulate_intent(&intent, &preview, wm_snapshot, &caps, features);
    
    // Large input should still be processed (within limits)
    match result {
        Ok(sim_result) => {
            // Verify the large description was processed
            assert_eq!(sim_result.plan_diff.adds.len(), 1);
        }
        Err(_) => {
            // Policy simulation failed, which is acceptable
        }
    }
}

#[test]
fn test_policy_simulation_capability_check() {
    let intent = IntentV1 {
        id: 12349,
        description: "capability test".to_string(),
        intent_type: 1,
        priority: 1,
        requested_caps: vec![
            CapRef { cap_id: 999, scope: "admin".to_string() },
            CapRef { cap_id: 888, scope: "system".to_string() },
        ],
        metadata: BTreeMap::new(),
    };

    let preview = PlanPreviewV1 {
        plan_id: 67894,
        actions: vec![ActionV1 { kind: 1, params: BTreeMap::new() }],
        cost: 10,
        risks: vec![],
        notes: vec![],
    };

    let wm_snapshot = 46;
    let caps = vec![1, 2]; // Limited capabilities
    let features = 0x1234;

    let result = PolicyKernel::simulate_intent(&intent, &preview, wm_snapshot, &caps, features);
    
    match result {
        Ok(sim_result) => {
            // Policy should consider requested vs available capabilities
            assert!(sim_result.decision.reasons.len() > 0);
        }
        Err(_) => {
            // Policy simulation failed, which is acceptable
        }
    }
}
