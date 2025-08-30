use crate::intent::schema::{ActionV1, ConstraintV1, IntentV1, PlanV1, PlanPreviewV1};
use crate::intent::whylog::WhyLog;
use std::collections::HashMap;

// Evidence structure for why-log entries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceV1 {
    pub key: String,
    pub value: String,
}

impl EvidenceV1 {
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

pub trait Planner: Send + Sync {
    fn preview(
        &self,
        intent: &IntentV1,
        caps: &[u64],
        why_log: &mut WhyLog,
        vclock: u64,
    ) -> PlanPreviewV1;
}

pub struct EchoPlanner {
    intent_type_actions: HashMap<u16, Vec<ActionV1>>,
    constraint_handlers: HashMap<u16, Box<dyn Fn(&ConstraintV1, &mut PlanV1) + Send + Sync>>,
}

impl EchoPlanner {
    pub fn new() -> Self {
        let mut planner = Self {
            intent_type_actions: HashMap::new(),
            constraint_handlers: HashMap::new(),
        };
        
        planner.setup_default_actions();
        planner.setup_default_constraints();
        
        planner
    }
    
    fn setup_default_actions(&mut self) {
        // BACKUP intent type
        let backup_actions = vec![
            ActionV1::new(crate::intent::schema::ACTION_KIND_SNAPSHOT)
                .with_param("target".to_string(), "database".to_string())
                .with_cost_estimate(50)
                .with_time_estimate(5000),
            ActionV1::new(crate::intent::schema::ACTION_KIND_VERIFY)
                .with_param("backup_id".to_string(), "{{snapshot_id}}".to_string())
                .with_cost_estimate(20)
                .with_time_estimate(2000),
        ];
        self.intent_type_actions.insert(crate::intent::schema::INTENT_TYPE_BACKUP, backup_actions);
        
        // UPDATE intent type
        let update_actions = vec![
            ActionV1::new(crate::intent::schema::ACTION_KIND_VALIDATE)
                .with_param("update_package".to_string(), "{{package_name}}".to_string())
                .with_cost_estimate(30)
                .with_time_estimate(3000),
            ActionV1::new(crate::intent::schema::ACTION_KIND_EXECUTE)
                .with_param("update_command".to_string(), "{{update_script}}".to_string())
                .with_cost_estimate(40)
                .with_time_estimate(4000),
        ];
        self.intent_type_actions.insert(crate::intent::schema::INTENT_TYPE_UPDATE, update_actions);
        
        // MONITOR intent type
        let monitor_actions = vec![
            ActionV1::new(crate::intent::schema::ACTION_KIND_MONITOR)
                .with_param("target".to_string(), "{{monitor_target}}".to_string())
                .with_param("interval".to_string(), "{{monitor_interval}}".to_string())
                .with_cost_estimate(10)
                .with_time_estimate(1000),
            ActionV1::new(crate::intent::schema::ACTION_KIND_LOG)
                .with_param("log_level".to_string(), "info".to_string())
                .with_cost_estimate(5)
                .with_time_estimate(500),
        ];
        self.intent_type_actions.insert(crate::intent::schema::INTENT_TYPE_MONITOR, monitor_actions);
        
        // DEPLOY intent type
        let deploy_actions = vec![
            ActionV1::new(crate::intent::schema::ACTION_KIND_VALIDATE)
                .with_param("deployment_config".to_string(), "{{config_file}}".to_string())
                .with_cost_estimate(60)
                .with_time_estimate(6000),
            ActionV1::new(crate::intent::schema::ACTION_KIND_EXECUTE)
                .with_param("deployment_script".to_string(), "{{deploy_script}}".to_string())
                .with_cost_estimate(80)
                .with_time_estimate(8000),
            ActionV1::new(crate::intent::schema::ACTION_KIND_VERIFY)
                .with_param("health_check".to_string(), "{{health_endpoint}}".to_string())
                .with_cost_estimate(25)
                .with_time_estimate(2500),
        ];
        self.intent_type_actions.insert(crate::intent::schema::INTENT_TYPE_DEPLOY, deploy_actions);
    }
    
    fn setup_default_constraints(&mut self) {
        // MAX_COST constraint handler
        self.constraint_handlers.insert(
            crate::intent::schema::CONSTRAINT_TYPE_MAX_COST,
            Box::new(|constraint, plan| {
                if let Some(value) = constraint.value_scalar {
                    if plan.total_cost > value {
                        // Filter out actions that exceed cost limit
                        plan.actions.retain(|action| action.cost_estimate <= value);
                        plan.total_cost = plan.actions.iter().map(|a| a.cost_estimate).sum();
                    }
                }
            }),
        );
        
        // MAX_TIME constraint handler
        self.constraint_handlers.insert(
            crate::intent::schema::CONSTRAINT_TYPE_MAX_TIME,
            Box::new(|constraint, plan| {
                if let Some(value) = constraint.value_scalar {
                    if plan.total_time > value {
                        // Filter out actions that exceed time limit
                        plan.actions.retain(|action| action.time_estimate <= value);
                        plan.total_time = plan.actions.iter().map(|a| a.time_estimate).sum();
                    }
                }
            }),
        );
        
        // SECURITY_LEVEL constraint handler
        self.constraint_handlers.insert(
            crate::intent::schema::CONSTRAINT_TYPE_SECURITY_LEVEL,
            Box::new(|constraint, plan| {
                if let Some(security_level) = constraint.value_scalar {
                    if security_level >= 3 {
                        // Add security validation action for high security levels
                        let security_action = ActionV1::new(crate::intent::schema::ACTION_KIND_VALIDATE)
                            .with_param("security_scan".to_string(), "enabled".to_string())
                            .with_cost_estimate(15)
                            .with_time_estimate(1500);
                        plan.actions.push(security_action);
                        plan.total_cost += 15;
                        plan.total_time += 1500;
                    }
                }
            }),
        );
    }
    
    pub fn register_intent_type(&mut self, intent_type: u16, actions: Vec<ActionV1>) {
        self.intent_type_actions.insert(intent_type, actions);
    }
    
    pub fn register_constraint_handler(
        &mut self,
        constraint_type: u16,
        handler: Box<dyn Fn(&ConstraintV1, &mut PlanV1) + Send + Sync>,
    ) {
        self.constraint_handlers.insert(constraint_type, handler);
    }
}

impl Planner for EchoPlanner {
    fn preview(
        &self,
        intent: &IntentV1,
        caps: &[u64],
        why_log: &mut WhyLog,
        vclock: u64,
    ) -> PlanPreviewV1 {
        // Log intent analysis
        why_log.log_intent_submitted(intent, vclock);
        
        // Get default actions for this intent type
        let mut actions = self.intent_type_actions
            .get(&intent.intent_type)
            .cloned()
            .unwrap_or_else(Vec::new);
        
        // Create initial plan
        let mut plan = PlanV1 {
            intent_id: intent.id,
            actions: actions.clone(),
            total_cost: actions.iter().map(|a| a.cost_estimate).sum(),
            total_time: actions.iter().map(|a| a.time_estimate).sum(),
            constraints_applied: Vec::new(),
        };
        
        // Apply constraints
        for constraint in &intent.constraints {
            if let Some(handler) = self.constraint_handlers.get(&constraint.kind) {
                handler(constraint, &mut plan);
                
                // Log constraint application
                why_log.log_constraint_applied(constraint, vclock);
                
                // Add to applied constraints
                plan.constraints_applied.push(constraint.clone());
            }
        }
        
        // Log action selection
        for action in &plan.actions {
            why_log.log_action_selected(action, vclock);
        }
        
        // Generate risks and notes
        let mut risks = Vec::new();
        let mut notes = Vec::new();
        
        // Risk assessment based on priority and deadline
        if intent.priority <= crate::intent::schema::PRIORITY_CRITICAL {
            risks.push("Critical priority may impact system stability".to_string());
        }
        
        if intent.deadline_ms < 60000 { // Less than 1 minute
            risks.push("Very short deadline may cause timeouts".to_string());
        }
        
        // Risk assessment based on capabilities
        if intent.requested_caps.len() > caps.len() {
            risks.push("Requested capabilities exceed available capabilities".to_string());
        }
        
        // Notes about the plan
        notes.push(format!("Plan generated for intent type: {}", intent.intent_type));
        notes.push(format!("Total estimated cost: {}", plan.total_cost));
        notes.push(format!("Total estimated time: {}ms", plan.total_time));
        
        if !plan.constraints_applied.is_empty() {
            notes.push(format!("Applied {} constraints", plan.constraints_applied.len()));
        }
        
        // Calculate confidence based on various factors
        let mut confidence = 80u8; // Base confidence
        
        // Reduce confidence for high-risk scenarios
        if !risks.is_empty() {
            confidence = confidence.saturating_sub(risks.len() as u8 * 10);
        }
        
        // Increase confidence for well-defined intents
        if !intent.description.is_empty() {
            confidence = confidence.saturating_add(5);
        }
        
        // Ensure confidence is within bounds
        confidence = confidence.clamp(0, 100);
        
        PlanPreviewV1 {
            plan,
            risks,
            notes,
            confidence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::schema::{IntentV1, ConstraintV1, ConstraintValue};

    #[test]
    fn test_echo_planner_creation() {
        let planner = EchoPlanner::new();
        assert!(!planner.intent_type_actions.is_empty());
        assert!(!planner.constraint_handlers.is_empty());
    }

    #[test]
    fn test_backup_intent_planning() {
        let planner = EchoPlanner::new();
        let mut why_log = WhyLog::new(10);
        
        let intent = IntentV1::new(123, "backup system".to_string(), 1); // BACKUP type
        let caps = vec![1, 2, 3];
        
        let preview = planner.preview(&intent, &caps, &mut why_log, 1000);
        
        assert_eq!(preview.plan.intent_id, 123);
        assert_eq!(preview.plan.actions.len(), 3); // snapshot, verify, archive
        assert_eq!(preview.plan.actions[0].params.get("operation"), Some(&"snapshot".to_string()));
        assert_eq!(preview.plan.actions[1].params.get("operation"), Some(&"verify".to_string()));
        assert_eq!(preview.plan.actions[2].params.get("operation"), Some(&"archive".to_string()));
    }

    #[test]
    fn test_constraint_application() {
        let planner = EchoPlanner::new();
        let mut why_log = WhyLog::new(10);
        
        let intent = IntentV1::new(456, "backup with cost limit".to_string(), 1)
            .with_constraints(vec![
                ConstraintV1::scalar(1, 20), // MAX_COST = 20
            ]);
        
        let caps = vec![1, 2, 3];
        let preview = planner.preview(&intent, &caps, &mut why_log, 1000);
        
        // Should have reduced actions due to cost constraint
        assert!(preview.plan.cost <= 20);
        assert!(preview.plan.actions.len() <= 2); // Cost constraint should limit actions
    }

    #[test]
    fn test_unknown_intent_type() {
        let planner = EchoPlanner::new();
        let mut why_log = WhyLog::new(10);
        
        let intent = IntentV1::new(789, "unknown operation".to_string(), 999); // Unknown type
        let caps = vec![1, 2, 3];
        
        let preview = planner.preview(&intent, &caps, &mut why_log, 1000);
        
        assert_eq!(preview.plan.intent_id, 789);
        assert_eq!(preview.plan.actions.len(), 1);
        assert_eq!(preview.plan.actions[0].params.get("operation"), Some(&"unknown_intent".to_string()));
    }

    #[test]
    fn test_capability_check() {
        let planner = EchoPlanner::new();
        let mut why_log = WhyLog::new(10);
        
        let intent = IntentV1::new(111, "capability test".to_string(), 1)
            .with_caps(vec![
                crate::intent::schema::CapRef::new(1, 1),
                crate::intent::schema::CapRef::new(2, 1),
                crate::intent::schema::CapRef::new(3, 1),
            ]);
        
        let caps = vec![1, 2]; // Only 2 caps available
        
        let preview = planner.preview(&intent, &caps, &mut why_log, 1000);
        
        // Should have risk about insufficient capabilities
        assert!(preview.risks.iter().any(|r| r.contains("Insufficient capabilities")));
    }

    #[test]
    fn test_priority_risk_assessment() {
        let planner = EchoPlanner::new();
        let mut why_log = WhyLog::new(10);
        
        let intent = IntentV1::new(222, "high priority".to_string(), 1)
            .with_priority(7); // High priority
        
        let caps = vec![1, 2, 3];
        let preview = planner.preview(&intent, &caps, &mut why_log, 1000);
        
        // Should have risk about high priority
        assert!(preview.risks.iter().any(|r| r.contains("High priority")));
    }

    #[test]
    fn test_deadline_check() {
        let planner = EchoPlanner::new();
        let mut why_log = WhyLog::new(10);
        
        let intent = IntentV1::new(333, "urgent backup".to_string(), 1)
            .with_deadline(50); // Very short deadline
        
        let caps = vec![1, 2, 3];
        let preview = planner.preview(&intent, &caps, &mut why_log, 1000);
        
        // Should have risk about deadline
        assert!(preview.risks.iter().any(|r| r.contains("exceeds deadline")));
    }

    #[test]
    fn test_why_log_integration() {
        let planner = EchoPlanner::new();
        let mut why_log = WhyLog::new(10);
        
        let intent = IntentV1::new(444, "test logging".to_string(), 1);
        let caps = vec![1, 2, 3];
        
        let _preview = planner.preview(&intent, &caps, &mut why_log, 1000);
        
        // Should have logged intent submission and action selection
        assert!(why_log.len() >= 2);
        let tail = why_log.get_tail();
        assert!(tail.entries_count >= 2);
    }

    #[test]
    fn test_planner_registration() {
        let mut planner = EchoPlanner::new();
        
        // Register custom intent type
        let custom_actions = vec![
            ActionV1::new(100).with_param("operation".to_string(), "custom1".to_string()),
            ActionV1::new(101).with_param("operation".to_string(), "custom2".to_string()),
        ];
        planner.register_intent_type(100, custom_actions);
        
        // Register custom constraint handler
        planner.register_constraint_handler(100, Box::new(|_constraint, plan| {
            plan.actions.push(
                ActionV1::new(102).with_param("operation".to_string(), "constraint_action".to_string())
            );
            plan.cost += 100;
        }));
        
        let mut why_log = WhyLog::new(10);
        let intent = IntentV1::new(555, "custom test".to_string(), 100)
            .with_constraints(vec![
                ConstraintV1::new(100, ConstraintValue::Scalar(0)),
            ]);
        
        let caps = vec![1, 2, 3];
        let preview = planner.preview(&intent, &caps, &mut why_log, 1000);
        
        // Should have custom actions and constraint action
        assert_eq!(preview.plan.actions.len(), 3);
        assert!(preview.plan.actions.iter().any(|a| a.params.get("operation") == Some(&"custom1".to_string())));
        assert!(preview.plan.actions.iter().any(|a| a.params.get("operation") == Some(&"custom2".to_string())));
        assert!(preview.plan.actions.iter().any(|a| a.params.get("operation") == Some(&"constraint_action".to_string())));
    }
}

