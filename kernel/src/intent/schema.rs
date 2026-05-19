use serde::{Deserialize, Serialize};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

pub const SCHEMA_VERSION: u16 = 1;
pub const INTENT_MAX_SIZE: usize = 8192;
pub const PLAN_PREVIEW_MAX_SIZE: usize = 16384;

// Intent state lifecycle constants
pub const INTENT_STATE_SUBMITTED: u8 = 0;
pub const INTENT_STATE_RUNNING: u8 = 1;
pub const INTENT_STATE_COMPLETED: u8 = 2;
pub const INTENT_STATE_FAILED: u8 = 3;
pub const INTENT_STATE_CANCELLED: u8 = 4;

// Intent type tags
pub const INTENT_TYPE_BACKUP: u16 = 1;
pub const INTENT_TYPE_UPDATE: u16 = 2;
pub const INTENT_TYPE_MONITOR: u16 = 3;
pub const INTENT_TYPE_DEPLOY: u16 = 4;

// Action kinds
pub const ACTION_KIND_VALIDATE: u16 = 1;
pub const ACTION_KIND_EXECUTE: u16 = 2;
pub const ACTION_KIND_VERIFY: u16 = 3;
pub const ACTION_KIND_SNAPSHOT: u16 = 4;
pub const ACTION_KIND_MONITOR: u16 = 5;
pub const ACTION_KIND_LOG: u16 = 6;

// Constraint kinds
pub const CONSTRAINT_TYPE_MAX_TIME: u16 = 1;
pub const CONSTRAINT_TYPE_MAX_COST: u16 = 2;
pub const CONSTRAINT_TYPE_SECURITY_LEVEL: u16 = 3;

// Priority levels
pub const PRIORITY_LOW: u8 = 0;
pub const PRIORITY_NORMAL: u8 = 1;
pub const PRIORITY_HIGH: u8 = 2;
pub const PRIORITY_CRITICAL: u8 = 3;

// Capacity limits for sub-collections
pub const METADATA_MAX_ENTRIES: usize = 64;
pub const CONSTRAINTS_MAX_COUNT: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct IntentV1 {
    pub version: u16,
    pub id: u128,
    pub description: String,
    pub intent_type: u16,
    pub priority: u8,
    pub constraints: Vec<ConstraintV1>,
    pub metadata: BTreeMap<String, String>,
    pub requested_caps: Vec<CapRef>,
    pub deadline_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct ConstraintV1 {
    pub kind: u16,
    pub value: ConstraintValue,
    /// Denormalised cache of `value` when it is a `Scalar(_)`. Callers in the
    /// planner read this directly instead of pattern-matching the enum.
    /// Kept in sync by `ConstraintV1::new_scalar`.
    #[serde(default)]
    pub value_scalar: Option<u64>,
}

impl ConstraintV1 {
    pub fn new_scalar(kind: u16, scalar: u64) -> Self {
        Self {
            kind,
            value: ConstraintValue::Scalar(scalar),
            value_scalar: Some(scalar),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub enum ConstraintValue {
    Bytes(Vec<u8>),
    Scalar(u64),
    String(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct ActionV1 {
    pub kind: u16,
    pub params: BTreeMap<String, String>,
    #[serde(default)]
    pub cost_estimate: u64,
    /// Planner-estimated wall-clock cost in milliseconds. Used by the
    /// constraint-satisfaction loop to drop actions that violate a
    /// CONSTRAINT_TYPE_MAX_TIME budget.
    #[serde(default)]
    pub time_estimate: u64,
}

impl ActionV1 {
    pub fn new(kind: u16) -> Self {
        Self { kind, params: BTreeMap::new(), cost_estimate: 0, time_estimate: 0 }
    }

    pub fn with_cost_estimate(mut self, cost: u64) -> Self {
        self.cost_estimate = cost;
        self
    }

    pub fn with_time_estimate(mut self, time_ms: u64) -> Self {
        self.time_estimate = time_ms;
        self
    }

    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct PlanV1 {
    pub intent_id: u128,
    pub actions: Vec<ActionV1>,
    pub cost: u64,
    /// Sum of `actions[*].cost_estimate`, computed at plan-build time and
    /// updated whenever the planner mutates `actions`.
    #[serde(default)]
    pub total_cost: u64,
    /// Sum of `actions[*].time_estimate` in milliseconds, kept in sync with
    /// `total_cost`. The constraint loop reads this to enforce time budgets.
    #[serde(default)]
    pub total_time: u64,
    /// Constraints that were satisfied (and so applied) while building this
    /// plan. Surfaced to whylog and the user-facing preview notes.
    #[serde(default)]
    pub constraints_applied: Vec<ConstraintV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct PlanPreviewV1 {
    pub plan: PlanV1,
    pub risks: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct EvidenceV1 {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct CapRef {
    pub cap_id: u64,
    pub scope: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct PreviewHandle {
    pub intent_id: u128,
    pub preview_hash: [u8; 32],
    pub timestamp: u64,
}

pub fn schema_hash() -> [u8; 32] {
    const SCHEMA_HASH: [u8; 32] = [
        0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x7a, 0x8b,
        0x9c, 0xad, 0xbe, 0xcf, 0xd0, 0xe1, 0xf2, 0x03,
        0x14, 0x25, 0x36, 0x47, 0x58, 0x69, 0x7a, 0x8b,
        0x9c, 0xad, 0xbe, 0xcf, 0xd0, 0xe1, 0xf2, 0x03,
    ];
    SCHEMA_HASH
}

impl IntentV1 {
    pub fn new(id: u128, description: String, intent_type: u16) -> Self {
        Self {
            version: SCHEMA_VERSION,
            id,
            description,
            intent_type,
            priority: 0,
            constraints: Vec::new(),
            metadata: BTreeMap::new(),
            requested_caps: Vec::new(),
            deadline_ms: 0,
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_constraints(mut self, constraints: Vec<ConstraintV1>) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn with_metadata(mut self, metadata: BTreeMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_caps(mut self, caps: Vec<CapRef>) -> Self {
        self.requested_caps = caps;
        self
    }

    pub fn with_deadline(mut self, deadline_ms: u64) -> Self {
        self.deadline_ms = deadline_ms;
        self
    }

    pub fn serialized_size(&self) -> Result<usize, serde_json::Error> {
        serde_json::to_vec(self).map(|v| v.len())
    }
}

impl ConstraintV1 {
    /// Build a constraint from a `kind` and a `ConstraintValue`. Keeps
    /// `value_scalar` in sync with the enum: if `value` is `Scalar(n)` we
    /// cache it, otherwise the cache is `None`.
    pub fn new(kind: u16, value: ConstraintValue) -> Self {
        let value_scalar = match &value {
            ConstraintValue::Scalar(n) => Some(*n),
            _ => None,
        };
        Self { kind, value, value_scalar }
    }

    pub fn bytes(kind: u16, data: Vec<u8>) -> Self {
        Self {
            kind,
            value: ConstraintValue::Bytes(data),
            value_scalar: None,
        }
    }

    pub fn scalar(kind: u16, value: u64) -> Self {
        Self {
            kind,
            value: ConstraintValue::Scalar(value),
            value_scalar: Some(value),
        }
    }

    pub fn string(kind: u16, value: String) -> Self {
        Self {
            kind,
            value: ConstraintValue::String(value),
            value_scalar: None,
        }
    }
}

// (impl ActionV1 with `new`/`with_cost_estimate`/`with_time_estimate`/`with_param`
// lives at the top of this file alongside the struct definition.)

impl ActionV1 {
    pub fn with_params(mut self, params: BTreeMap<String, String>) -> Self {
        self.params = params;
        self
    }
}

impl PlanV1 {
    pub fn new(intent_id: u128) -> Self {
        Self {
            intent_id,
            actions: Vec::new(),
            cost: 0,
            total_cost: 0,
            total_time: 0,
            constraints_applied: Vec::new(),
        }
    }

    pub fn with_action(mut self, action: ActionV1) -> Self {
        self.actions.push(action);
        self
    }

    pub fn with_cost(mut self, cost: u64) -> Self {
        self.cost = cost;
        self
    }
}

impl PlanPreviewV1 {
    pub fn new(plan: PlanV1) -> Self {
        Self {
            plan,
            risks: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn with_risk(mut self, risk: String) -> Self {
        self.risks.push(risk);
        self
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.notes.push(note);
        self
    }

    pub fn serialized_size(&self) -> Result<usize, serde_json::Error> {
        serde_json::to_vec(self).map(|v| v.len())
    }
}

impl EvidenceV1 {
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

impl CapRef {
    pub fn new(cap_id: u64, scope: u64) -> Self {
        Self { cap_id, scope }
    }
}

impl PreviewHandle {
    pub fn new(intent_id: u128, preview_hash: [u8; 32], timestamp: u64) -> Self {
        Self {
            intent_id,
            preview_hash,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_creation() {
        let intent = IntentV1::new(123, "test intent".to_string(), 1);
        assert_eq!(intent.version, SCHEMA_VERSION);
        assert_eq!(intent.id, 123);
        assert_eq!(intent.description, "test intent");
        assert_eq!(intent.intent_type, 1);
        assert_eq!(intent.priority, 0);
        assert!(intent.constraints.is_empty());
        assert!(intent.metadata.is_empty());
        assert!(intent.requested_caps.is_empty());
        assert_eq!(intent.deadline_ms, 0);
    }

    #[test]
    fn test_intent_builder_pattern() {
        let intent = IntentV1::new(456, "builder test".to_string(), 2)
            .with_priority(5)
            .with_deadline(1000);
        
        assert_eq!(intent.priority, 5);
        assert_eq!(intent.deadline_ms, 1000);
    }

    #[test]
    fn test_constraint_creation() {
        let bytes_constraint = ConstraintV1::bytes(1, vec![1, 2, 3]);
        let scalar_constraint = ConstraintV1::scalar(2, 42);
        let string_constraint = ConstraintV1::string(3, "test".to_string());

        assert_eq!(bytes_constraint.kind, 1);
        assert_eq!(scalar_constraint.kind, 2);
        assert_eq!(string_constraint.kind, 3);

        match bytes_constraint.value {
            ConstraintValue::Bytes(ref data) => assert_eq!(data, &vec![1, 2, 3]),
            _ => panic!("Expected Bytes"),
        }

        match scalar_constraint.value {
            ConstraintValue::Scalar(value) => assert_eq!(value, 42),
            _ => panic!("Expected Scalar"),
        }

        match string_constraint.value {
            ConstraintValue::String(ref s) => assert_eq!(s, "test"),
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_action_creation() {
        let action = ActionV1::new(1)
            .with_param("key1".to_string(), "value1".to_string())
            .with_param("key2".to_string(), "value2".to_string());

        assert_eq!(action.kind, 1);
        assert_eq!(action.params.len(), 2);
        assert_eq!(action.params.get("key1"), Some(&"value1".to_string()));
        assert_eq!(action.params.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_plan_creation() {
        let action1 = ActionV1::new(1);
        let action2 = ActionV1::new(2);
        
        let plan = PlanV1::new(789)
            .with_action(action1)
            .with_action(action2)
            .with_cost(100);

        assert_eq!(plan.intent_id, 789);
        assert_eq!(plan.actions.len(), 2);
        assert_eq!(plan.cost, 100);
    }

    #[test]
    fn test_plan_preview_creation() {
        let plan = PlanV1::new(999);
        let preview = PlanPreviewV1::new(plan)
            .with_risk("high risk".to_string())
            .with_note("important note".to_string());

        assert_eq!(preview.plan.intent_id, 999);
        assert_eq!(preview.risks.len(), 1);
        assert_eq!(preview.notes.len(), 1);
        assert_eq!(preview.risks[0], "high risk");
        assert_eq!(preview.notes[0], "important note");
    }

    #[test]
    fn test_evidence_creation() {
        let evidence = EvidenceV1::new("key".to_string(), "value".to_string());
        assert_eq!(evidence.key, "key");
        assert_eq!(evidence.value, "value");
    }

    #[test]
    fn test_cap_ref_creation() {
        let cap_ref = CapRef::new(123, 456);
        assert_eq!(cap_ref.cap_id, 123);
        assert_eq!(cap_ref.scope, 456);
    }

    #[test]
    fn test_preview_handle_creation() {
        let hash = [1u8; 32];
        let handle = PreviewHandle::new(111, hash, 1000);
        assert_eq!(handle.intent_id, 111);
        assert_eq!(handle.preview_hash, hash);
        assert_eq!(handle.timestamp, 1000);
    }

    #[test]
    fn test_schema_hash_consistency() {
        let hash1 = schema_hash();
        let hash2 = schema_hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let intent = IntentV1::new(123, "test".to_string(), 1)
            .with_priority(5)
            .with_constraints(vec![
                ConstraintV1::scalar(1, 42),
                ConstraintV1::string(2, "constraint".to_string()),
            ]);

        let serialized = serde_json::to_string(&intent).unwrap();
        let deserialized: IntentV1 = serde_json::from_str(&serialized).unwrap();

        assert_eq!(intent, deserialized);
    }

    #[test]
    fn test_size_limits() {
        let intent = IntentV1::new(123, "test".to_string(), 1);
        let preview = PlanPreviewV1::new(PlanV1::new(123));

        assert!(intent.serialized_size().unwrap() <= INTENT_MAX_SIZE);
        assert!(preview.serialized_size().unwrap() <= PLAN_PREVIEW_MAX_SIZE);
    }
}

// ============================================================================
// Additional types for compatibility
// ============================================================================

/// Intent status snapshot returned to userspace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct IntentStatusV1 {
    pub intent_id: u128,
    pub state: u8,
    pub last_why_log_hash: [u8; 32],
    pub progress: u32,
    pub last_update: u64,
    pub error_code: i32,
}

impl Default for IntentStatusV1 {
    fn default() -> Self {
        Self {
            intent_id: 0,
            state: INTENT_STATE_SUBMITTED,
            last_why_log_hash: [0u8; 32],
            progress: 0,
            last_update: 0,
            error_code: 0,
        }
    }
}

/// Why record for audit trail
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WhyRecordV1 {
    /// Timestamp of the record
    pub timestamp: u64,
    /// Intent ID this record relates to
    pub intent_id: u128,
    /// Vector clock for ordering
    pub vclock: u64,
    /// Justification text
    pub justification: String,
    /// Evidence supporting the decision
    pub evidence: Vec<EvidenceV1>,
    /// Hash of the record for integrity
    pub hash: [u8; 32],
}

impl WhyRecordV1 {
    pub fn new(intent_id: u128, vclock: u64, justification: String) -> Self {
        Self {
            timestamp: crate::log::get_current_time_ms(),
            intent_id,
            vclock,
            justification,
            evidence: Vec::new(),
            hash: [0u8; 32],
        }
    }
    
    pub fn with_evidence(mut self, evidence: Vec<EvidenceV1>) -> Self {
        self.evidence = evidence;
        self
    }
}
