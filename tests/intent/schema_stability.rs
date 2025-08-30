use crate::intent::schema::*;
use serde_cbor;

#[test]
fn test_schema_version_consistency() {
    // Verify schema version is consistent
    assert_eq!(SCHEMA_VERSION, 1);
    assert_eq!(SCHEMA_HASH, "1a2b3c4d5e6f7a8b9cadbecfd0e1f2031425364758697a8b9cadbecfd0e1f203");
}

#[test]
fn test_intent_type_constants() {
    // Verify intent type constants are stable
    assert_eq!(INTENT_TYPE_BACKUP, 1);
    assert_eq!(INTENT_TYPE_UPDATE, 2);
    assert_eq!(INTENT_TYPE_MONITOR, 3);
    assert_eq!(INTENT_TYPE_DEPLOY, 4);
    assert_eq!(INTENT_TYPE_SCALE, 5);
    assert_eq!(INTENT_TYPE_SECURE, 6);
    assert_eq!(INTENT_TYPE_ANALYZE, 7);
    assert_eq!(INTENT_TYPE_OPTIMIZE, 8);
}

#[test]
fn test_action_kind_constants() {
    // Verify action kind constants are stable
    assert_eq!(ACTION_KIND_SNAPSHOT, 1);
    assert_eq!(ACTION_KIND_VERIFY, 2);
    assert_eq!(ACTION_KIND_TRANSFER, 3);
    assert_eq!(ACTION_KIND_VALIDATE, 4);
    assert_eq!(ACTION_KIND_EXECUTE, 5);
    assert_eq!(ACTION_KIND_MONITOR, 6);
    assert_eq!(ACTION_KIND_LOG, 7);
    assert_eq!(ACTION_KIND_NOTIFY, 8);
}

#[test]
fn test_constraint_type_constants() {
    // Verify constraint type constants are stable
    assert_eq!(CONSTRAINT_TYPE_MAX_COST, 1);
    assert_eq!(CONSTRAINT_TYPE_MAX_TIME, 2);
    assert_eq!(CONSTRAINT_TYPE_SECURITY_LEVEL, 3);
    assert_eq!(CONSTRAINT_TYPE_RESOURCE_LIMIT, 4);
    assert_eq!(CONSTRAINT_TYPE_DEPENDENCY, 5);
    assert_eq!(CONSTRAINT_TYPE_COMPLIANCE, 6);
}

#[test]
fn test_priority_constants() {
    // Verify priority constants are stable
    assert_eq!(PRIORITY_CRITICAL, 0);
    assert_eq!(PRIORITY_HIGH, 1);
    assert_eq!(PRIORITY_NORMAL, 2);
    assert_eq!(PRIORITY_LOW, 3);
    assert_eq!(PRIORITY_BACKGROUND, 4);
}

#[test]
fn test_intent_state_constants() {
    // Verify intent state constants are stable
    assert_eq!(INTENT_STATE_SUBMITTED, 0);
    assert_eq!(INTENT_STATE_QUEUED, 1);
    assert_eq!(INTENT_STATE_PLANNED, 2);
    assert_eq!(INTENT_STATE_RUNNING, 3);
    assert_eq!(INTENT_STATE_COMPLETED, 4);
    assert_eq!(INTENT_STATE_CANCELLED, 5);
    assert_eq!(INTENT_STATE_ERROR, 6);
}

#[test]
fn test_size_limit_constants() {
    // Verify size limit constants are stable
    assert_eq!(INTENT_MAX_SIZE, 8192);
    assert_eq!(PLAN_PREVIEW_MAX_SIZE, 16384);
    assert_eq!(WHY_RECORD_MAX_SIZE, 1024);
    assert_eq!(METADATA_MAX_ENTRIES, 32);
    assert_eq!(CONSTRAINTS_MAX_COUNT, 16);
    assert_eq!(ACTIONS_MAX_COUNT, 64);
}

#[test]
fn test_cbor_round_trip_intent() {
    // Test CBOR serialization/deserialization round-trip for IntentV1
    let intent = IntentV1::new()
        .with_id(123456789)
        .with_description("Test backup intent".to_string())
        .with_intent_type(INTENT_TYPE_BACKUP)
        .with_priority(PRIORITY_HIGH)
        .with_deadline(86400000)
        .with_capability(1)
        .with_capability(2);
    
    // Serialize to CBOR
    let cbor_data = intent.to_cbor().expect("Failed to serialize intent to CBOR");
    
    // Verify size limits
    assert!(cbor_data.len() <= INTENT_MAX_SIZE);
    
    // Deserialize from CBOR
    let deserialized_intent = IntentV1::from_cbor(&cbor_data).expect("Failed to deserialize intent from CBOR");
    
    // Verify round-trip equality
    assert_eq!(intent, deserialized_intent);
}

#[test]
fn test_cbor_round_trip_plan_preview() {
    // Test CBOR serialization/deserialization round-trip for PlanPreviewV1
    let action1 = ActionV1::new(ACTION_KIND_SNAPSHOT)
        .with_param("target".to_string(), "database".to_string())
        .with_cost_estimate(50)
        .with_time_estimate(5000);
    
    let action2 = ActionV1::new(ACTION_KIND_VERIFY)
        .with_param("backup_id".to_string(), "snapshot_123".to_string())
        .with_cost_estimate(20)
        .with_time_estimate(2000);
    
    let plan = PlanV1 {
        intent_id: 123456789,
        actions: vec![action1, action2],
        total_cost: 70,
        total_time: 7000,
        constraints_applied: Vec::new(),
    };
    
    let preview = PlanPreviewV1 {
        plan,
        risks: vec!["Test risk".to_string()],
        notes: vec!["Test note".to_string()],
        confidence: 85,
    };
    
    // Serialize to CBOR
    let cbor_data = preview.to_cbor().expect("Failed to serialize preview to CBOR");
    
    // Verify size limits
    assert!(cbor_data.len() <= PLAN_PREVIEW_MAX_SIZE);
    
    // Deserialize from CBOR
    let deserialized_preview = PlanPreviewV1::from_cbor(&cbor_data).expect("Failed to deserialize preview from CBOR");
    
    // Verify round-trip equality
    assert_eq!(preview, deserialized_preview);
}

#[test]
fn test_cbor_round_trip_why_record() {
    // Test CBOR serialization/deserialization round-trip for WhyRecordV1
    let why_record = WhyRecordV1 {
        seq: 1,
        ts: 1000,
        actor: "kernel".to_string(),
        event: "intent_submitted".to_string(),
        data_hash: [0x1a; 32],
        prev_hash: [0x2b; 32],
        signature: vec![0x3c; 32],
        justification: "Test why record".to_string(),
    };
    
    // Serialize to CBOR
    let cbor_data = why_record.to_cbor().expect("Failed to serialize why record to CBOR");
    
    // Verify size limits
    assert!(cbor_data.len() <= WHY_RECORD_MAX_SIZE);
    
    // Deserialize from CBOR
    let deserialized_record = WhyRecordV1::from_cbor(&cbor_data).expect("Failed to deserialize why record from CBOR");
    
    // Verify round-trip equality
    assert_eq!(why_record, deserialized_record);
}

#[test]
fn test_constraint_value_types() {
    // Test different constraint value types
    let constraint_bytes = ConstraintV1::new(CONSTRAINT_TYPE_MAX_COST)
        .with_bytes_value(vec![0x01, 0x02, 0x03]);
    
    let constraint_scalar = ConstraintV1::new(CONSTRAINT_TYPE_MAX_TIME)
        .with_scalar_value(5000);
    
    let constraint_string = ConstraintV1::new(CONSTRAINT_TYPE_SECURITY_LEVEL)
        .with_string_value("high".to_string());
    
    // Verify value types are set correctly
    assert_eq!(constraint_bytes.value_type, 3); // bytes
    assert_eq!(constraint_scalar.value_type, 4); // scalar
    assert_eq!(constraint_string.value_type, 5); // string
    
    // Verify values are stored correctly
    assert_eq!(constraint_bytes.value_bytes, Some(vec![0x01, 0x02, 0x03]));
    assert_eq!(constraint_scalar.value_scalar, Some(5000));
    assert_eq!(constraint_string.value_string, Some("high".to_string()));
}

#[test]
fn test_builder_pattern() {
    // Test the builder pattern for creating intents
    let intent = IntentV1::new()
        .with_id(987654321)
        .with_description("Test intent with builder".to_string())
        .with_intent_type(INTENT_TYPE_MONITOR)
        .with_priority(PRIORITY_NORMAL)
        .with_deadline(300000) // 5 minutes
        .with_capability(5)
        .with_capability(6)
        .with_metadata("environment".to_string(), "production".to_string())
        .with_metadata("version".to_string(), "1.0.0".to_string());
    
    // Verify all fields are set correctly
    assert_eq!(intent.id, 987654321);
    assert_eq!(intent.description, "Test intent with builder");
    assert_eq!(intent.intent_type, INTENT_TYPE_MONITOR);
    assert_eq!(intent.priority, PRIORITY_NORMAL);
    assert_eq!(intent.deadline_ms, 300000);
    assert_eq!(intent.requested_caps, vec![5, 6]);
    assert_eq!(intent.metadata.get("environment"), Some(&"production".to_string()));
    assert_eq!(intent.metadata.get("version"), Some(&"1.0.0".to_string()));
}

#[test]
fn test_serialized_size_validation() {
    // Test that serialized sizes are within limits
    let intent = IntentV1::new()
        .with_id(111111111)
        .with_description("A".repeat(1000)) // Large description
        .with_intent_type(INTENT_TYPE_DEPLOY)
        .with_priority(PRIORITY_LOW);
    
    let size = intent.serialized_size().expect("Failed to compute serialized size");
    assert!(size <= INTENT_MAX_SIZE);
    
    // Test with many metadata entries
    let mut intent_with_metadata = intent;
    for i in 0..METADATA_MAX_ENTRIES {
        intent_with_metadata = intent_with_metadata.with_metadata(
            format!("key{}", i),
            format!("value{}", i),
        );
    }
    
    let size_with_metadata = intent_with_metadata.serialized_size().expect("Failed to compute serialized size");
    assert!(size_with_metadata <= INTENT_MAX_SIZE);
}

#[test]
fn test_schema_hash_stability() {
    // Test that schema hash is consistent across test runs
    let expected_hash = "1a2b3c4d5e6f7a8b9cadbecfd0e1f2031425364758697a8b9cadbecfd0e1f203";
    assert_eq!(SCHEMA_HASH, expected_hash);
    
    // Verify hash format (32 bytes = 64 hex characters)
    assert_eq!(SCHEMA_HASH.len(), 64);
    
    // Verify hash contains only hex characters
    for c in SCHEMA_HASH.chars() {
        assert!(c.is_ascii_hexdigit());
    }
}
