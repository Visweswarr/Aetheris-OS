use polymera_kernel::policy::simulate::*;
use polymera_kernel::policy::schema::*;
use std::collections::BTreeMap;

#[test]
fn test_simulation_engine_creation() {
    let engine = SimulationEngine::new(1024);
    assert_eq!(engine.max_diff_size, 1024);
    assert!(!engine.verbose);
}

#[test]
fn test_simulation_engine_verbose_mode() {
    let mut engine = SimulationEngine::new(1024);
    engine.set_verbose(true);
    assert!(engine.verbose);
    
    engine.set_verbose(false);
    assert!(!engine.verbose);
}

#[test]
fn test_plan_diff_size_calculation() {
    let diff = PlanDiffV1 {
        adds: vec![ActionV1 { kind: 1, params: BTreeMap::new() }],
        removes: vec![0, 1],
        edits: vec![],
    };
    
    assert!(!diff.is_empty());
    assert_eq!(diff.change_count(), 3);
    assert!(diff.total_size() > 0);
}

#[test]
fn test_plan_diff_empty() {
    let diff = PlanDiffV1 {
        adds: vec![],
        removes: vec![],
        edits: vec![],
    };
    
    assert!(diff.is_empty());
    assert_eq!(diff.change_count(), 0);
    assert_eq!(diff.total_size(), 0);
}

#[test]
fn test_action_sanitization() {
    let mut params = BTreeMap::new();
    params.insert("normal".to_string(), "value".to_string());
    params.insert("secret".to_string(), "password123".to_string());
    
    let action = ActionV1 { kind: 1, params };
    let sanitized = SimulationEngine::new(1024).sanitize_params(&action.params);
    
    assert_eq!(sanitized.get("normal"), Some(&"value".to_string()));
    assert_eq!(sanitized.get("secret"), Some(&"[REDACTED]".to_string()));
}

#[test]
fn test_action_sanitization_password() {
    let mut params = BTreeMap::new();
    params.insert("password".to_string(), "secret123".to_string());
    params.insert("token".to_string(), "abc123".to_string());
    
    let action = ActionV1 { kind: 1, params };
    let sanitized = SimulationEngine::new(1024).sanitize_params(&action.params);
    
    assert_eq!(sanitized.get("password"), Some(&"[REDACTED]".to_string()));
    assert_eq!(sanitized.get("token"), Some(&"abc123".to_string()));
}

#[test]
fn test_path_parsing() {
    let engine = SimulationEngine::new(1024);
    
    let result = engine.parse_path("actions[5].params.secret");
    assert_eq!(result, Some((5, "params.secret")));
    
    let result = engine.parse_path("actions[0].kind");
    assert_eq!(result, Some((0, "kind")));
    
    let result = engine.parse_path("invalid_path");
    assert_eq!(result, None);
    
    let result = engine.parse_path("actions[abc].field");
    assert_eq!(result, None);
}

#[test]
fn test_redaction_application() {
    let mut diff = PlanDiffV1 {
        adds: vec![ActionV1 {
            kind: 1,
            params: {
                let mut params = BTreeMap::new();
                params.insert("secret".to_string(), "value".to_string());
                params.insert("normal".to_string(), "data".to_string());
                params
            },
        }],
        removes: vec![],
        edits: vec![],
    };
    
    let redactions = vec![PathSpec { path: "actions[0].params.secret".to_string() }];
    let engine = SimulationEngine::new(1024);
    
    engine.apply_redactions(&mut diff, &redactions);
    
    assert_eq!(diff.adds[0].params.get("secret"), Some(&"[REDACTED]".to_string()));
    assert_eq!(diff.adds[0].params.get("normal"), Some(&"data".to_string()));
}

#[test]
fn test_redaction_application_multiple() {
    let mut diff = PlanDiffV1 {
        adds: vec![
            ActionV1 {
                kind: 1,
                params: {
                    let mut params = BTreeMap::new();
                    params.insert("secret".to_string(), "value1".to_string());
                    params
                },
            },
            ActionV1 {
                kind: 2,
                params: {
                    let mut params = BTreeMap::new();
                    params.insert("password".to_string(), "value2".to_string());
                    params
                },
            },
        ],
        removes: vec![],
        edits: vec![],
    };
    
    let redactions = vec![
        PathSpec { path: "actions[0].params.secret".to_string() },
        PathSpec { path: "actions[1].params.password".to_string() },
    ];
    let engine = SimulationEngine::new(1024);
    
    engine.apply_redactions(&mut diff, &redactions);
    
    assert_eq!(diff.adds[0].params.get("secret"), Some(&"[REDACTED]".to_string()));
    assert_eq!(diff.adds[1].params.get("password"), Some(&"[REDACTED]".to_string()));
}

#[test]
fn test_redaction_application_edits() {
    let mut diff = PlanDiffV1 {
        adds: vec![],
        removes: vec![],
        edits: vec![EditSpec {
            action_index: 0,
            new_action: ActionV1 {
                kind: 1,
                params: {
                    let mut params = BTreeMap::new();
                    params.insert("secret".to_string(), "value".to_string());
                    params
                },
            },
        }],
    };
    
    let redactions = vec![PathSpec { path: "actions[0].params.secret".to_string() }];
    let engine = SimulationEngine::new(1024);
    
    engine.apply_redactions(&mut diff, &redactions);
    
    assert_eq!(diff.edits[0].new_action.params.get("secret"), Some(&"[REDACTED]".to_string()));
}

#[test]
fn test_simulation_action_kinds() {
    let engine = SimulationEngine::new(1024);
    
    // Test create action (kind 1)
    let action = ActionV1 { kind: 1, params: BTreeMap::new() };
    let result = engine.simulate_action(&action, 0, &[]);
    assert!(matches!(result, ActionSimResult::Add));
    
    // Test delete action (kind 2)
    let action = ActionV1 { kind: 2, params: BTreeMap::new() };
    let result = engine.simulate_action(&action, 0, &[]);
    assert!(matches!(result, ActionSimResult::Remove));
    
    // Test edit action (kind 3)
    let action = ActionV1 { kind: 3, params: BTreeMap::new() };
    let result = engine.simulate_action(&action, 0, &[]);
    assert!(matches!(result, ActionSimResult::Edit(_)));
    
    // Test unknown action (kind 0)
    let action = ActionV1 { kind: 0, params: BTreeMap::new() };
    let result = engine.simulate_action(&action, 0, &[]);
    assert!(matches!(result, ActionSimResult::Keep));
}

#[test]
fn test_simulation_with_caps() {
    let engine = SimulationEngine::new(1024);
    
    let preview = PlanPreviewV1 {
        plan_id: 123,
        actions: vec![
            ActionV1 { kind: 1, params: BTreeMap::new() },
            ActionV1 { kind: 2, params: BTreeMap::new() },
        ],
        cost: 100,
        risks: vec![],
        notes: vec![],
    };
    
    let caps = vec![1, 2, 3];
    let diff = engine.simulate(&preview, 42, &caps);
    
    assert_eq!(diff.adds.len(), 1); // One add action
    assert_eq!(diff.removes.len(), 1); // One remove action
    assert_eq!(diff.edits.len(), 0); // No edit actions
}

#[test]
fn test_diff_truncation() {
    let engine = SimulationEngine::new(100); // Small size limit
    
    let preview = PlanPreviewV1 {
        plan_id: 123,
        actions: vec![
            ActionV1 { 
                kind: 1, 
                params: {
                    let mut params = BTreeMap::new();
                    params.insert("large_key".to_string(), "large_value".repeat(50));
                    params
                }
            },
        ],
        cost: 100,
        risks: vec![],
        notes: vec![],
    };
    
    let diff = engine.simulate(&preview, 0, &[]);
    
    // Diff should be truncated to fit size limit
    assert!(diff.total_size() <= 100);
}

#[test]
fn test_action_total_size() {
    let action = ActionV1 {
        kind: 1,
        params: {
            let mut params = BTreeMap::new();
            params.insert("key".to_string(), "value".to_string());
            params.insert("another".to_string(), "data".to_string());
            params
        },
    };
    
    let size = action.total_size();
    assert!(size > 0);
    assert!(size >= 2); // At least kind size
}

#[test]
fn test_edit_spec_total_size() {
    let edit = EditSpec {
        action_index: 5,
        new_action: ActionV1 { kind: 1, params: BTreeMap::new() },
    };
    
    let size = edit.total_size();
    assert!(size > 0);
    assert!(size >= 4); // At least action_index size
}
