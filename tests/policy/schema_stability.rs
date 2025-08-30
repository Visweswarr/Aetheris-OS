use polymera_kernel::policy::schema::*;

#[test]
fn test_schema_hash_stability() {
    let hash1 = schema_hash();
    let hash2 = schema_hash();
    assert_eq!(hash1, hash2, "Schema hash should be stable across calls");
    
    // Verify hash is not all zeros
    assert_ne!(hash1, [0u8; 32], "Schema hash should not be all zeros");
}

#[test]
fn test_policy_input_roundtrip() {
    let input = PolicyInputV1 {
        intent: IntentV1 {
            id: 12345,
            description: "test intent".to_string(),
            intent_type: 1,
            priority: 2,
            requested_caps: vec![CapRef { cap_id: 1, scope: "test".to_string() }],
            metadata: std::collections::BTreeMap::new(),
        },
        preview: PlanPreviewV1 {
            plan_id: 67890,
            actions: vec![ActionV1 { kind: 1, params: std::collections::BTreeMap::new() }],
            cost: 100,
            risks: vec!["low".to_string()],
            notes: vec!["test note".to_string()],
        },
        wm_snapshot: 42,
        caps: vec![CapRef { cap_id: 1, scope: "test".to_string() }],
        features: 0x1234,
    };

    let serialized = serialize_policy_input(&input).unwrap();
    let deserialized = deserialize_policy_input(&serialized).unwrap();
    
    assert_eq!(input, deserialized, "Policy input should roundtrip correctly");
}

#[test]
fn test_policy_decision_roundtrip() {
    let decision = PolicyDecisionV1 {
        allow: true,
        reasons: vec!["test reason".to_string()],
        redactions: vec![PathSpec { path: "actions[0].params.secret".to_string() }],
    };

    let serialized = serialize_policy_decision(&decision).unwrap();
    let deserialized = deserialize_policy_decision(&serialized).unwrap();
    
    assert_eq!(decision, deserialized, "Policy decision should roundtrip correctly");
}

#[test]
fn test_plan_diff_roundtrip() {
    let diff = PlanDiffV1 {
        adds: vec![ActionV1 { 
            kind: 1, 
            params: {
                let mut params = std::collections::BTreeMap::new();
                params.insert("key".to_string(), "value".to_string());
                params
            }
        }],
        removes: vec![0, 1],
        edits: vec![EditSpec {
            action_index: 2,
            new_action: ActionV1 { kind: 3, params: std::collections::BTreeMap::new() },
        }],
    };

    let serialized = serde_cbor::to_vec(&diff).unwrap();
    let deserialized: PlanDiffV1 = serde_cbor::from_slice(&serialized).unwrap();
    
    assert_eq!(diff, deserialized, "Plan diff should roundtrip correctly");
}

#[test]
fn test_schema_version_constant() {
    assert_eq!(SCHEMA_VERSION, 1, "Schema version should be 1");
}

#[test]
fn test_cap_ref_ordering() {
    let cap1 = CapRef { cap_id: 1, scope: "a".to_string() };
    let cap2 = CapRef { cap_id: 2, scope: "b".to_string() };
    let cap3 = CapRef { cap_id: 1, scope: "c".to_string() };
    
    assert!(cap1 < cap2, "CapRef should be ordered by cap_id first");
    assert!(cap1 < cap3, "CapRef should be ordered by scope when cap_id is equal");
}

#[test]
fn test_path_spec_ordering() {
    let path1 = PathSpec { path: "actions[0].params".to_string() };
    let path2 = PathSpec { path: "actions[1].params".to_string() };
    let path3 = PathSpec { path: "actions[0].params.secret".to_string() };
    
    assert!(path1 < path2, "PathSpec should be ordered by path string");
    assert!(path1 < path3, "PathSpec should be ordered by path string length");
}

#[test]
fn test_serialization_size_limits() {
    let large_string = "x".repeat(64 * 1024); // 64 KiB
    
    let input = PolicyInputV1 {
        intent: IntentV1 {
            id: 12345,
            description: large_string.clone(),
            intent_type: 1,
            priority: 2,
            requested_caps: vec![],
            metadata: std::collections::BTreeMap::new(),
        },
        preview: PlanPreviewV1 {
            plan_id: 67890,
            actions: vec![],
            cost: 100,
            risks: vec![],
            notes: vec![],
        },
        wm_snapshot: 42,
        caps: vec![],
        features: 0x1234,
    };

    let serialized = serialize_policy_input(&input).unwrap();
    assert!(serialized.len() <= 32 * 1024, "Serialized policy input should fit within 32 KiB limit");
}
