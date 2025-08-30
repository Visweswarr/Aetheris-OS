use crate::intent::planner::{Planner, EchoPlanner};
use crate::intent::schema::{IntentV1, ConstraintV1, ActionV1, PlanV1, PlanPreviewV1};
use crate::intent::whylog::WhyLog;

#[test]
fn test_echo_planner_creation() {
    let planner = EchoPlanner::new();
    
    // Test that planner was created successfully
    // (No assertions needed as this just verifies the constructor doesn't panic)
}

#[test]
fn test_backup_intent_planning() {
    let planner = EchoPlanner::new();
    let mut why_log = WhyLog::new(100);
    
    let intent = IntentV1::new()
        .with_id(123)
        .with_description("Backup database".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_BACKUP)
        .with_priority(crate::intent::schema::PRIORITY_HIGH)
        .with_deadline(86400000)
        .with_capability(1)
        .with_capability(2);
    
    let available_caps = vec![1u64, 2u64, 3u64];
    let vclock = 1000;
    
    let preview = planner.preview(&intent, &available_caps, &mut why_log, vclock);
    
    // Verify plan structure
    assert_eq!(preview.plan.intent_id, 123);
    assert_eq!(preview.plan.actions.len(), 2);
    assert!(preview.plan.total_cost > 0);
    assert!(preview.plan.total_time > 0);
    
    // Verify actions are correct for backup intent
    let actions = &preview.plan.actions;
    assert_eq!(actions[0].kind, crate::intent::schema::ACTION_KIND_SNAPSHOT);
    assert_eq!(actions[1].kind, crate::intent::schema::ACTION_KIND_VERIFY);
    
    // Verify action parameters
    assert_eq!(actions[0].params.get("target"), Some(&"database".to_string()));
    assert_eq!(actions[1].params.get("backup_id"), Some(&"{{snapshot_id}}".to_string()));
    
    // Verify risks and notes
    assert!(!preview.risks.is_empty());
    assert!(!preview.notes.is_empty());
    assert!(preview.confidence > 0);
    
    // Verify why-log entries were created
    let tail = why_log.get_tail();
    assert!(tail.entries_count > 0);
}

#[test]
fn test_update_intent_planning() {
    let planner = EchoPlanner::new();
    let mut why_log = WhyLog::new(100);
    
    let intent = IntentV1::new()
        .with_id(456)
        .with_description("Update system packages".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_UPDATE)
        .with_priority(crate::intent::schema::PRIORITY_NORMAL)
        .with_deadline(60000)
        .with_capability(3);
    
    let available_caps = vec![3u64, 4u64];
    let vclock = 2000;
    
    let preview = planner.preview(&intent, &available_caps, &mut why_log, vclock);
    
    // Verify plan structure
    assert_eq!(preview.plan.intent_id, 456);
    assert_eq!(preview.plan.actions.len(), 2);
    
    // Verify actions are correct for update intent
    let actions = &preview.plan.actions;
    assert_eq!(actions[0].kind, crate::intent::schema::ACTION_KIND_VALIDATE);
    assert_eq!(actions[1].kind, crate::intent::schema::ACTION_KIND_EXECUTE);
    
    // Verify action parameters
    assert_eq!(actions[0].params.get("update_package"), Some(&"{{package_name}}".to_string()));
    assert_eq!(actions[1].params.get("update_command"), Some(&"{{update_script}}".to_string()));
    
    // Verify cost and time estimates
    assert!(actions[0].cost_estimate > 0);
    assert!(actions[1].cost_estimate > 0);
    assert!(actions[0].time_estimate > 0);
    assert!(actions[1].time_estimate > 0);
}

#[test]
fn test_monitor_intent_planning() {
    let planner = EchoPlanner::new();
    let mut why_log = WhyLog::new(100);
    
    let intent = IntentV1::new()
        .with_id(789)
        .with_description("Monitor system health".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_MONITOR)
        .with_priority(crate::intent::schema::PRIORITY_LOW)
        .with_deadline(300000)
        .with_capability(5);
    
    let available_caps = vec![5u64, 6u64];
    let vclock = 3000;
    
    let preview = planner.preview(&intent, &available_caps, &mut why_log, vclock);
    
    // Verify plan structure
    assert_eq!(preview.plan.intent_id, 789);
    assert_eq!(preview.plan.actions.len(), 2);
    
    // Verify actions are correct for monitor intent
    let actions = &preview.plan.actions;
    assert_eq!(actions[0].kind, crate::intent::schema::ACTION_KIND_MONITOR);
    assert_eq!(actions[1].kind, crate::intent::schema::ACTION_KIND_LOG);
    
    // Verify action parameters
    assert_eq!(actions[0].params.get("target"), Some(&"{{monitor_target}}".to_string()));
    assert_eq!(actions[0].params.get("interval"), Some(&"{{monitor_interval}}".to_string()));
    assert_eq!(actions[1].params.get("log_level"), Some(&"info".to_string()));
}

#[test]
fn test_deploy_intent_planning() {
    let planner = EchoPlanner::new();
    let mut why_log = WhyLog::new(100);
    
    let intent = IntentV1::new()
        .with_id(101)
        .with_description("Deploy new application".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_DEPLOY)
        .with_priority(crate::intent::schema::PRIORITY_CRITICAL)
        .with_deadline(180000)
        .with_capability(7);
    
    let available_caps = vec![7u64, 8u64];
    let vclock = 4000;
    
    let preview = planner.preview(&intent, &available_caps, &mut why_log, vclock);
    
    // Verify plan structure
    assert_eq!(preview.plan.intent_id, 101);
    assert_eq!(preview.plan.actions.len(), 3);
    
    // Verify actions are correct for deploy intent
    let actions = &preview.plan.actions;
    assert_eq!(actions[0].kind, crate::intent::schema::ACTION_KIND_VALIDATE);
    assert_eq!(actions[1].kind, crate::intent::schema::ACTION_KIND_EXECUTE);
    assert_eq!(actions[2].kind, crate::intent::schema::ACTION_KIND_VERIFY);
    
    // Verify action parameters
    assert_eq!(actions[0].params.get("deployment_config"), Some(&"{{config_file}}".to_string()));
    assert_eq!(actions[1].params.get("deployment_script"), Some(&"{{deploy_script}}".to_string()));
    assert_eq!(actions[2].params.get("health_check"), Some(&"{{health_endpoint}}".to_string()));
}

#[test]
fn test_constraint_application_max_cost() {
    let planner = EchoPlanner::new();
    let mut why_log = WhyLog::new(100);
    
    let constraint = ConstraintV1::new(crate::intent::schema::CONSTRAINT_TYPE_MAX_COST)
        .with_scalar_value(50);
    
    let intent = IntentV1::new()
        .with_id(202)
        .with_description("Cost-constrained backup".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_BACKUP)
        .with_priority(crate::intent::schema::PRIORITY_NORMAL)
        .with_deadline(60000)
        .with_capability(1)
        .with_constraint(constraint);
    
    let available_caps = vec![1u64, 2u64];
    let vclock = 5000;
    
    let preview = planner.preview(&intent, &available_caps, &mut why_log, vclock);
    
    // Verify constraint was applied
    assert!(!preview.plan.constraints_applied.is_empty());
    assert!(preview.plan.total_cost <= 50);
    
    // Verify why-log contains constraint application
    let tail = why_log.get_tail();
    assert!(tail.entries_count > 0);
}

#[test]
fn test_constraint_application_max_time() {
    let planner = EchoPlanner::new();
    let mut why_log = WhyLog::new(100);
    
    let constraint = ConstraintV1::new(crate::intent::schema::CONSTRAINT_TYPE_MAX_TIME)
        .with_scalar_value(3000);
    
    let intent = IntentV1::new()
        .with_id(303)
        .with_description("Time-constrained update".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_UPDATE)
        .with_priority(crate::intent::schema::PRIORITY_HIGH)
        .with_deadline(10000)
        .with_capability(3)
        .with_constraint(constraint);
    
    let available_caps = vec![3u64, 4u64];
    let vclock = 6000;
    
    let preview = planner.preview(&intent, &available_caps, &mut why_log, vclock);
    
    // Verify constraint was applied
    assert!(!preview.plan.constraints_applied.is_empty());
    assert!(preview.plan.total_time <= 3000);
}

#[test]
fn test_constraint_application_security_level() {
    let planner = EchoPlanner::new();
    let mut why_log = WhyLog::new(100);
    
    let constraint = ConstraintV1::new(crate::intent::schema::CONSTRAINT_TYPE_SECURITY_LEVEL)
        .with_scalar_value(4);
    
    let intent = IntentV1::new()
        .with_id(404)
        .with_description("High-security deployment".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_DEPLOY)
        .with_priority(crate::intent::schema::PRIORITY_CRITICAL)
        .with_deadline(120000)
        .with_capability(7)
        .with_constraint(constraint);
    
    let available_caps = vec![7u64, 8u64];
    let vclock = 7000;
    
    let preview = planner.preview(&intent, &available_caps, &mut why_log, vclock);
    
    // Verify constraint was applied
    assert!(!preview.plan.constraints_applied.is_empty());
    
    // Verify security action was added
    let security_actions: Vec<&ActionV1> = preview.plan.actions
        .iter()
        .filter(|action| action.params.get("security_scan") == Some(&"enabled".to_string()))
        .collect();
    
    assert!(!security_actions.is_empty());
    
    // Verify security action has correct parameters
    let security_action = security_actions[0];
    assert_eq!(security_action.kind, crate::intent::schema::ACTION_KIND_VALIDATE);
    assert_eq!(security_action.params.get("security_scan"), Some(&"enabled".to_string()));
}

#[test]
fn test_multiple_constraints() {
    let planner = EchoPlanner::new();
    let mut why_log = WhyLog::new(100);
    
    let cost_constraint = ConstraintV1::new(crate::intent::schema::CONSTRAINT_TYPE_MAX_COST)
        .with_scalar_value(100);
    
    let time_constraint = ConstraintV1::new(crate::intent::schema::CONSTRAINT_TYPE_MAX_TIME)
        .with_scalar_value(5000);
    
    let intent = IntentV1::new()
        .with_id(505)
        .with_description("Multi-constrained operation".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_ANALYZE)
        .with_priority(crate::intent::schema::PRIORITY_NORMAL)
        .with_deadline(30000)
        .with_capability(9)
        .with_constraint(cost_constraint)
        .with_constraint(time_constraint);
    
    let available_caps = vec![9u64, 10u64];
    let vclock = 8000;
    
    let preview = planner.preview(&intent, &available_caps, &mut why_log, vclock);
    
    // Verify both constraints were applied
    assert_eq!(preview.plan.constraints_applied.len(), 2);
    assert!(preview.plan.total_cost <= 100);
    assert!(preview.plan.total_time <= 5000);
}

#[test]
fn test_unknown_intent_type() {
    let planner = EchoPlanner::new();
    let mut why_log = WhyLog::new(100);
    
    let intent = IntentV1::new()
        .with_id(606)
        .with_description("Unknown operation".to_string())
        .with_intent_type(999) // Unknown type
        .with_priority(crate::intent::schema::PRIORITY_LOW)
        .with_deadline(60000)
        .with_capability(11);
    
    let available_caps = vec![11u64];
    let vclock = 9000;
    
    let preview = planner.preview(&intent, &available_caps, &mut why_log, vclock);
    
    // Verify plan was still generated (with no actions)
    assert_eq!(preview.plan.intent_id, 606);
    assert_eq!(preview.plan.actions.len(), 0);
    assert_eq!(preview.plan.total_cost, 0);
    assert_eq!(preview.plan.total_time, 0);
}

#[test]
fn test_risk_assessment() {
    let planner = EchoPlanner::new();
    let mut why_log = WhyLog::new(100);
    
    // Test critical priority risk
    let critical_intent = IntentV1::new()
        .with_id(707)
        .with_description("Critical operation".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_SECURE)
        .with_priority(crate::intent::schema::PRIORITY_CRITICAL)
        .with_deadline(60000)
        .with_capability(12);
    
    let available_caps = vec![12u64];
    let vclock = 10000;
    
    let preview = planner.preview(&critical_intent, &available_caps, &mut why_log, vclock);
    
    // Verify risks were identified
    assert!(!preview.risks.is_empty());
    assert!(preview.risks.iter().any(|risk| risk.contains("Critical priority")));
    
    // Test short deadline risk
    let short_deadline_intent = IntentV1::new()
        .with_id(808)
        .with_description("Quick operation".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_OPTIMIZE)
        .with_priority(crate::intent::schema::PRIORITY_HIGH)
        .with_deadline(30000) // Less than 1 minute
        .with_capability(13);
    
    let preview2 = planner.preview(&short_deadline_intent, &available_caps, &mut why_log, vclock);
    
    // Verify deadline risk was identified
    assert!(!preview2.risks.is_empty());
    assert!(preview2.risks.iter().any(|risk| risk.contains("short deadline")));
}

#[test]
fn test_confidence_calculation() {
    let planner = EchoPlanner::new();
    let mut why_log = WhyLog::new(100);
    
    // Test well-defined intent (should have higher confidence)
    let well_defined_intent = IntentV1::new()
        .with_id(909)
        .with_description("Well-defined backup operation".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_BACKUP)
        .with_priority(crate::intent::schema::PRIORITY_NORMAL)
        .with_deadline(120000)
        .with_capability(14);
    
    let available_caps = vec![14u64, 15u64];
    let vclock = 11000;
    
    let preview = planner.preview(&well_defined_intent, &available_caps, &mut why_log, vclock);
    
    // Verify confidence is reasonable
    assert!(preview.confidence >= 70);
    assert!(preview.confidence <= 100);
    
    // Test risky intent (should have lower confidence)
    let risky_intent = IntentV1::new()
        .with_id(1010)
        .with_description("Risky operation".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_DEPLOY)
        .with_priority(crate::intent::schema::PRIORITY_CRITICAL)
        .with_deadline(15000) // Very short deadline
        .with_capability(16);
    
    let preview2 = planner.preview(&risky_intent, &available_caps, &mut why_log, vclock);
    
    // Verify confidence is lower due to risks
    assert!(preview2.confidence < preview.confidence);
}

#[test]
fn test_why_log_integration() {
    let planner = EchoPlanner::new();
    let mut why_log = WhyLog::new(100);
    
    let intent = IntentV1::new()
        .with_id(1111)
        .with_description("Why-log test operation".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_MONITOR)
        .with_priority(crate::intent::schema::PRIORITY_LOW)
        .with_deadline(90000)
        .with_capability(17);
    
    let available_caps = vec![17u64];
    let vclock = 12000;
    
    let initial_entries = why_log.get_tail().entries_count;
    
    let _preview = planner.preview(&intent, &available_caps, &mut why_log, vclock);
    
    // Verify why-log entries were created
    let final_entries = why_log.get_tail().entries_count;
    assert!(final_entries > initial_entries);
    
    // Verify entries contain expected information
    let entries = why_log.get_entries_since([0u8; 32]);
    assert!(!entries.is_empty());
    
    // Should have entries for intent submission, action selection, etc.
    let has_intent_entry = entries.iter().any(|entry| entry.event == "intent_operation");
    let has_action_entry = entries.iter().any(|entry| entry.event == "intent_operation");
    
    assert!(has_intent_entry);
    assert!(has_action_entry);
}

#[test]
fn test_deterministic_behavior() {
    let planner = EchoPlanner::new();
    let mut why_log1 = WhyLog::new(100);
    let mut why_log2 = WhyLog::new(100);
    
    let intent = IntentV1::new()
        .with_id(1212)
        .with_description("Deterministic test".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_BACKUP)
        .with_priority(crate::intent::schema::PRIORITY_NORMAL)
        .with_deadline(60000)
        .with_capability(18);
    
    let available_caps = vec![18u64, 19u64];
    let vclock = 13000;
    
    // Generate preview twice with identical inputs
    let preview1 = planner.preview(&intent, &available_caps, &mut why_log1, vclock);
    let preview2 = planner.preview(&intent, &available_caps, &mut why_log2, vclock);
    
    // Verify results are identical
    assert_eq!(preview1.plan.intent_id, preview2.plan.intent_id);
    assert_eq!(preview1.plan.actions.len(), preview2.plan.actions.len());
    assert_eq!(preview1.plan.total_cost, preview2.plan.total_cost);
    assert_eq!(preview1.plan.total_time, preview2.plan.total_time);
    assert_eq!(preview1.risks.len(), preview2.risks.len());
    assert_eq!(preview1.notes.len(), preview2.notes.len());
    assert_eq!(preview1.confidence, preview2.confidence);
    
    // Verify actions are identical
    for (action1, action2) in preview1.plan.actions.iter().zip(preview2.plan.actions.iter()) {
        assert_eq!(action1.kind, action2.kind);
        assert_eq!(action1.params, action2.params);
        assert_eq!(action1.cost_estimate, action2.cost_estimate);
        assert_eq!(action1.time_estimate, action2.time_estimate);
    }
}
