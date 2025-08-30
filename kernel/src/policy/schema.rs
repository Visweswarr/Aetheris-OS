use serde::{Deserialize, Serialize};
use serde_cbor;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;

/// Policy Input schema for evaluation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct PolicyInputV1 {
    /// Intent being evaluated
    pub intent: IntentV1,
    /// Plan preview from intent
    pub preview: PlanPreviewV1,
    /// World Model snapshot reference
    pub wm_snapshot: u64,
    /// Available capabilities
    pub caps: Vec<CapRef>,
    /// Kernel feature bits
    pub features: u64,
}

/// Policy decision output
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct PolicyDecisionV1 {
    /// Whether the action is allowed
    pub allow: bool,
    /// Human-readable reasons for decision
    pub reasons: Vec<String>,
    /// Paths to redact from the plan
    pub redactions: Vec<PathSpec>,
}

/// Path specification for redactions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct PathSpec {
    /// JSONPath-like specification (e.g., "actions[2].params.secret")
    pub path: String,
}

/// Intent schema (simplified for policy evaluation)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct IntentV1 {
    /// Intent identifier
    pub id: u128,
    /// Intent description
    pub description: String,
    /// Intent type
    pub intent_type: u16,
    /// Priority level
    pub priority: u8,
    /// Requested capabilities
    pub requested_caps: Vec<CapRef>,
    /// Metadata
    pub metadata: BTreeMap<String, String>,
}

/// Plan preview schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct PlanPreviewV1 {
    /// Plan identifier
    pub plan_id: u128,
    /// List of actions
    pub actions: Vec<ActionV1>,
    /// Estimated cost
    pub cost: u64,
    /// Risk assessment
    pub risks: Vec<String>,
    /// Notes
    pub notes: Vec<String>,
}

/// Action schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct ActionV1 {
    /// Action kind
    pub kind: u16,
    /// Action parameters
    pub params: BTreeMap<String, String>,
}

/// Capability reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct CapRef {
    /// Capability identifier
    pub cap_id: u64,
    /// Capability scope
    pub scope: String,
}

/// Plan difference for simulation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct PlanDiffV1 {
    /// Actions to add
    pub adds: Vec<ActionV1>,
    /// Actions to remove (by index)
    pub removes: Vec<u32>,
    /// Actions to edit
    pub edits: Vec<EditSpec>,
}

/// Edit specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct EditSpec {
    /// Action index to edit
    pub action_index: u32,
    /// New action data
    pub new_action: ActionV1,
}

/// Policy simulation result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub struct PolicySimResultV1 {
    /// Policy decision
    pub decision: PolicyDecisionV1,
    /// Plan difference
    pub plan_diff: PlanDiffV1,
    /// Why-log digest
    pub why_digest: [u8; 32],
}

/// Schema version constant
pub const SCHEMA_VERSION: u16 = 1;

/// Compute schema hash for policy schemas
pub fn schema_hash() -> [u8; 32] {
    use blake3::Hasher;
    
    let mut hasher = blake3::Hasher::new();
    
    // Hash the schema structure deterministically
    let schema_def = r#"
        PolicyInputV1: intent, preview, wm_snapshot, caps, features
        PolicyDecisionV1: allow, reasons, redactions
        PathSpec: path
        IntentV1: id, description, intent_type, priority, requested_caps, metadata
        PlanPreviewV1: plan_id, actions, cost, risks, notes
        ActionV1: kind, params
        CapRef: cap_id, scope
        PlanDiffV1: adds, removes, edits
        EditSpec: action_index, new_action
        PolicySimResultV1: decision, plan_diff, why_digest
    "#;
    
    hasher.update(schema_def.as_bytes());
    hasher.finalize().into()
}

/// Serialize policy input to CBOR
pub fn serialize_policy_input(input: &PolicyInputV1) -> Result<Vec<u8>, &'static str> {
    serde_cbor::to_vec(input).map_err(|_| "Failed to serialize policy input")
}

/// Deserialize policy input from CBOR
pub fn deserialize_policy_input(data: &[u8]) -> Result<PolicyInputV1, &'static str> {
    serde_cbor::from_slice(data).map_err(|_| "Failed to deserialize policy input")
}

/// Serialize policy decision to CBOR
pub fn serialize_policy_decision(decision: &PolicyDecisionV1) -> Result<Vec<u8>, &'static str> {
    serde_cbor::to_vec(decision).map_err(|_| "Failed to serialize policy decision")
}

/// Deserialize policy decision from CBOR
pub fn deserialize_policy_decision(data: &[u8]) -> Result<PolicyDecisionV1, &'static str> {
    serde_cbor::from_slice(data).map_err(|_| "Failed to deserialize policy decision")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_hash_stability() {
        let hash1 = schema_hash();
        let hash2 = schema_hash();
        assert_eq!(hash1, hash2, "Schema hash should be stable across calls");
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
                metadata: BTreeMap::new(),
            },
            preview: PlanPreviewV1 {
                plan_id: 67890,
                actions: vec![ActionV1 { kind: 1, params: BTreeMap::new() }],
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
}
