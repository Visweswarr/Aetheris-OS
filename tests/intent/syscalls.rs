use crate::intent::{IntentKernel, Errno};
use crate::intent::schema::{IntentV1, ConstraintV1, PlanPreviewV1, IntentStatusV1};
use serde_cbor;

#[test]
fn test_intent_kernel_initialization() {
    let kernel = IntentKernel::new();
    let counters = kernel.get_counters();
    
    assert_eq!(counters.intent_submit_ok, 0);
    assert_eq!(counters.preview_ok, 0);
    assert_eq!(counters.schema_fail, 0);
    assert_eq!(counters.policy_deny_sim, 0);
    assert_eq!(counters.intent_cancelled, 0);
    assert_eq!(counters.intent_status_queries, 0);
    assert_eq!(counters.whylog_streams, 0);
}

#[test]
fn test_sys_intent_submit_success() {
    let kernel = IntentKernel::new();
    
    // Create a valid intent
    let intent = IntentV1::new()
        .with_id(123456789)
        .with_description("Test backup intent".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_BACKUP)
        .with_priority(crate::intent::schema::PRIORITY_HIGH)
        .with_deadline(86400000)
        .with_capability(1)
        .with_capability(2);
    
    // Serialize to CBOR
    let intent_data = intent.to_cbor().expect("Failed to serialize intent");
    
    // Submit intent
    let result = kernel.submit(&intent_data);
    assert!(result.is_ok());
    
    let handle = result.unwrap();
    assert_eq!(handle.intent_id, 123456789);
    
    // Verify counters
    let counters = kernel.get_counters();
    assert_eq!(counters.intent_submit_ok, 1);
    assert_eq!(counters.schema_fail, 0);
    assert_eq!(counters.policy_deny_sim, 0);
    
    // Verify intent was stored
    let stored_intent = kernel.get_intent(123456789);
    assert!(stored_intent.is_some());
    assert_eq!(stored_intent.unwrap().description, "Test backup intent");
}

#[test]
fn test_sys_intent_submit_oversize() {
    let kernel = IntentKernel::new();
    
    // Create an intent with a very long description that exceeds size limits
    let mut intent = IntentV1::new()
        .with_id(987654321)
        .with_intent_type(crate::intent::schema::INTENT_TYPE_MONITOR)
        .with_priority(crate::intent::schema::PRIORITY_NORMAL)
        .with_deadline(300000)
        .with_capability(5);
    
    // Add a very long description
    intent.description = "A".repeat(crate::intent::schema::INTENT_MAX_SIZE + 1000);
    
    // Serialize to CBOR
    let intent_data = intent.to_cbor().expect("Failed to serialize intent");
    
    // Submit should fail due to size
    let result = kernel.submit(&intent_data);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Errno::EINVAL);
    
    // Verify counters
    let counters = kernel.get_counters();
    assert_eq!(counters.intent_submit_ok, 0);
    assert_eq!(counters.schema_fail, 1);
}

#[test]
fn test_sys_intent_submit_invalid_cbor() {
    let kernel = IntentKernel::new();
    
    // Invalid CBOR data
    let invalid_data = vec![0xDE, 0xAD, 0xBE, 0xEF];
    
    // Submit should fail due to invalid CBOR
    let result = kernel.submit(&invalid_data);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Errno::EINVAL);
    
    // Verify counters
    let counters = kernel.get_counters();
    assert_eq!(counters.intent_submit_ok, 0);
    assert_eq!(counters.schema_fail, 1);
}

#[test]
fn test_sys_intent_submit_policy_denial() {
    let kernel = IntentKernel::new();
    
    // Create an intent that should be denied by policy (reserved type 0)
    let intent = IntentV1::new()
        .with_id(111111111)
        .with_description("Reserved intent type".to_string())
        .with_intent_type(0) // Reserved type
        .with_priority(crate::intent::schema::PRIORITY_NORMAL)
        .with_deadline(86400000)
        .with_capability(1);
    
    // Serialize to CBOR
    let intent_data = intent.to_cbor().expect("Failed to serialize intent");
    
    // Submit should fail due to policy
    let result = kernel.submit(&intent_data);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Errno::EPERM);
    
    // Verify counters
    let counters = kernel.get_counters();
    assert_eq!(counters.intent_submit_ok, 0);
    assert_eq!(counters.policy_deny_sim, 1);
}

#[test]
fn test_sys_intent_preview_success() {
    let kernel = IntentKernel::new();
    
    // Create a valid intent
    let intent = IntentV1::new()
        .with_id(222222222)
        .with_description("Test preview intent".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_UPDATE)
        .with_priority(crate::intent::schema::PRIORITY_NORMAL)
        .with_deadline(60000)
        .with_capability(3);
    
    // Serialize to CBOR
    let intent_data = intent.to_cbor().expect("Failed to serialize intent");
    
    // Generate preview
    let result = kernel.preview(&intent_data);
    assert!(result.is_ok());
    
    let preview = result.unwrap();
    assert_eq!(preview.plan.intent_id, 222222222);
    assert!(!preview.plan.actions.is_empty());
    assert!(preview.confidence > 0);
    
    // Verify counters
    let counters = kernel.get_counters();
    assert_eq!(counters.preview_ok, 1);
    assert_eq!(counters.schema_fail, 0);
}

#[test]
fn test_sys_intent_preview_with_constraints() {
    let kernel = IntentKernel::new();
    
    // Create an intent with constraints
    let constraint = ConstraintV1::new(crate::intent::schema::CONSTRAINT_TYPE_MAX_COST)
        .with_scalar_value(100);
    
    let intent = IntentV1::new()
        .with_id(333333333)
        .with_description("Constrained intent".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_DEPLOY)
        .with_priority(crate::intent::schema::PRIORITY_HIGH)
        .with_deadline(120000)
        .with_capability(4)
        .with_constraint(constraint);
    
    // Serialize to CBOR
    let intent_data = intent.to_cbor().expect("Failed to serialize intent");
    
    // Generate preview
    let result = kernel.preview(&intent_data);
    assert!(result.is_ok());
    
    let preview = result.unwrap();
    assert_eq!(preview.plan.intent_id, 333333333);
    
    // Verify constraints were applied
    assert!(!preview.plan.constraints_applied.is_empty());
    assert!(preview.plan.total_cost <= 100);
    
    // Verify counters
    let counters = kernel.get_counters();
    assert_eq!(counters.preview_ok, 1);
}

#[test]
fn test_sys_intent_status() {
    let kernel = IntentKernel::new();
    
    // First submit an intent
    let intent = IntentV1::new()
        .with_id(444444444)
        .with_description("Status test intent".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_MONITOR)
        .with_priority(crate::intent::schema::PRIORITY_LOW)
        .with_deadline(30000)
        .with_capability(6);
    
    let intent_data = intent.to_cbor().expect("Failed to serialize intent");
    let _handle = kernel.submit(&intent_data).expect("Failed to submit intent");
    
    // Get status
    let result = kernel.get_intent_status(444444444);
    assert!(result.is_ok());
    
    let status = result.unwrap();
    assert_eq!(status.intent_id, 444444444);
    assert_eq!(status.state, crate::intent::schema::INTENT_STATE_SUBMITTED);
    assert_eq!(status.progress, 0);
    assert_eq!(status.error_code, 0);
    
    // Verify counters
    let counters = kernel.get_counters();
    assert_eq!(counters.intent_status_queries, 1);
}

#[test]
fn test_sys_intent_status_not_found() {
    let kernel = IntentKernel::new();
    
    // Try to get status for non-existent intent
    let result = kernel.get_intent_status(999999999);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Errno::ESRCH);
}

#[test]
fn test_sys_intent_cancel() {
    let kernel = IntentKernel::new();
    
    // First submit an intent
    let intent = IntentV1::new()
        .with_id(555555555)
        .with_description("Cancel test intent".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_BACKUP)
        .with_priority(crate::intent::schema::PRIORITY_NORMAL)
        .with_deadline(60000)
        .with_capability(7);
    
    let intent_data = intent.to_cbor().expect("Failed to serialize intent");
    let _handle = kernel.submit(&intent_data).expect("Failed to submit intent");
    
    // Cancel intent
    let result = kernel.cancel_intent(555555555);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), crate::intent::schema::INTENT_STATE_CANCELLED);
    
    // Verify counters
    let counters = kernel.get_counters();
    assert_eq!(counters.intent_cancelled, 1);
    
    // Verify state was updated
    let status = kernel.get_intent_status(555555555).expect("Failed to get status");
    assert_eq!(status.state, crate::intent::schema::INTENT_STATE_CANCELLED);
}

#[test]
fn test_sys_intent_cancel_not_found() {
    let kernel = IntentKernel::new();
    
    // Try to cancel non-existent intent
    let result = kernel.cancel_intent(888888888);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Errno::ESRCH);
}

#[test]
fn test_sys_whylog_stream() {
    let kernel = IntentKernel::new();
    
    // First submit an intent to generate some why-log entries
    let intent = IntentV1::new()
        .with_id(666666666)
        .with_description("Why-log test intent".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_ANALYZE)
        .with_priority(crate::intent::schema::PRIORITY_BACKGROUND)
        .with_deadline(180000)
        .with_capability(8);
    
    let intent_data = intent.to_cbor().expect("Failed to serialize intent");
    let _handle = kernel.submit(&intent_data).expect("Failed to submit intent");
    
    // Get why-log stream from the beginning
    let result = kernel.get_whylog_stream(0, 10);
    assert!(result.is_ok());
    
    let stream_data = result.unwrap();
    assert!(!stream_data.is_empty());
    
    // Verify counters
    let counters = kernel.get_counters();
    assert_eq!(counters.whylog_streams, 1);
    
    // Try to parse the CBOR stream data
    let records: Result<Vec<crate::intent::schema::WhyRecordV1>, _> = serde_cbor::from_slice(&stream_data);
    assert!(records.is_ok());
    
    let records = records.unwrap();
    assert!(!records.is_empty());
}

#[test]
fn test_sys_whylog_stream_with_cursor() {
    let kernel = IntentKernel::new();
    
    // Submit multiple intents to generate more why-log entries
    for i in 0..3 {
        let intent = IntentV1::new()
            .with_id(700000000 + i)
            .with_description(format!("Why-log cursor test {}", i))
            .with_intent_type(crate::intent::schema::INTENT_TYPE_OPTIMIZE)
            .with_priority(crate::intent::schema::PRIORITY_LOW)
            .with_deadline(90000)
            .with_capability(9);
        
        let intent_data = intent.to_cbor().expect("Failed to serialize intent");
        let _handle = kernel.submit(&intent_data).expect("Failed to submit intent");
    }
    
    // Get why-log stream from cursor 1
    let result = kernel.get_whylog_stream(1, 5);
    assert!(result.is_ok());
    
    let stream_data = result.unwrap();
    assert!(!stream_data.is_empty());
    
    // Parse and verify we get entries from sequence 1 onwards
    let records: Result<Vec<crate::intent::schema::WhyRecordV1>, _> = serde_cbor::from_slice(&stream_data);
    assert!(records.is_ok());
    
    let records = records.unwrap();
    assert!(!records.is_empty());
    
    // All records should have sequence >= 1
    for record in &records {
        assert!(record.seq >= 1);
    }
}

#[test]
fn test_sys_whylog_tail() {
    let kernel = IntentKernel::new();
    
    // Submit an intent to generate why-log entries
    let intent = IntentV1::new()
        .with_id(777777777)
        .with_description("Why-log tail test".to_string())
        .with_intent_type(crate::intent::schema::INTENT_TYPE_SECURE)
        .with_priority(crate::intent::schema::PRIORITY_HIGH)
        .with_deadline(45000)
        .with_capability(10);
    
    let intent_data = intent.to_cbor().expect("Failed to serialize intent");
    let _handle = kernel.submit(&intent_data).expect("Failed to submit intent");
    
    // Get why-log tail
    let tail = kernel.get_whylog_tail();
    
    assert!(tail.entries_count > 0);
    assert!(tail.last_seq > 0);
    assert!(tail.chain_verified);
    assert!(!tail.truncated);
}

#[test]
fn test_sys_intent_stats() {
    let kernel = IntentKernel::new();
    
    // Submit a few intents to generate some stats
    for i in 0..2 {
        let intent = IntentV1::new()
            .with_id(800000000 + i)
            .with_description(format!("Stats test intent {}", i))
            .with_intent_type(crate::intent::schema::INTENT_TYPE_SCALE)
            .with_priority(crate::intent::schema::PRIORITY_NORMAL)
            .with_deadline(120000)
            .with_capability(11);
        
        let intent_data = intent.to_cbor().expect("Failed to serialize intent");
        let _handle = kernel.submit(&intent_data).expect("Failed to submit intent");
    }
    
    // Get counters
    let counters = kernel.get_counters();
    
    assert_eq!(counters.intent_submit_ok, 2);
    assert_eq!(counters.schema_fail, 0);
    assert_eq!(counters.policy_deny_sim, 0);
    assert_eq!(counters.intent_cancelled, 0);
    assert_eq!(counters.intent_status_queries, 0);
    assert_eq!(counters.whylog_streams, 0);
    
    // Test reset counters
    kernel.reset_counters();
    let reset_counters = kernel.get_counters();
    
    assert_eq!(reset_counters.intent_submit_ok, 0);
    assert_eq!(reset_counters.preview_ok, 0);
    assert_eq!(reset_counters.schema_fail, 0);
    assert_eq!(reset_counters.policy_deny_sim, 0);
}

#[test]
fn test_intent_kernel_list_and_clear() {
    let kernel = IntentKernel::new();
    
    // Submit a few intents
    for i in 0..3 {
        let intent = IntentV1::new()
            .with_id(900000000 + i)
            .with_description(format!("List test intent {}", i))
            .with_intent_type(crate::intent::schema::INTENT_TYPE_MONITOR)
            .with_priority(crate::intent::schema::PRIORITY_LOW)
            .with_deadline(60000)
            .with_capability(12);
        
        let intent_data = intent.to_cbor().expect("Failed to serialize intent");
        let _handle = kernel.submit(&intent_data).expect("Failed to submit intent");
    }
    
    // List intents
    let intent_ids = kernel.list_intents();
    assert_eq!(intent_ids.len(), 3);
    
    // Verify all intents exist
    for &id in &intent_ids {
        let intent = kernel.get_intent(id);
        assert!(intent.is_some());
    }
    
    // Clear intents
    kernel.clear_intents();
    
    // Verify all intents were cleared
    let intent_ids_after_clear = kernel.list_intents();
    assert_eq!(intent_ids_after_clear.len(), 0);
    
    for &id in &intent_ids {
        let intent = kernel.get_intent(id);
        assert!(intent.is_none());
    }
}

#[test]
fn test_virtual_clock() {
    let kernel = IntentKernel::new();
    
    // Set virtual clock
    kernel.set_virtual_clock(1000);
    
    // Get virtual clock (should increment)
    let clock1 = kernel.get_virtual_clock();
    let clock2 = kernel.get_virtual_clock();
    let clock3 = kernel.get_virtual_clock();
    
    assert_eq!(clock1, 1001);
    assert_eq!(clock2, 1002);
    assert_eq!(clock3, 1003);
    
    // Set clock again
    kernel.set_virtual_clock(5000);
    let clock4 = kernel.get_virtual_clock();
    assert_eq!(clock4, 5001);
}
