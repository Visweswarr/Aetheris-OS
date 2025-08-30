use crate::intent::whylog::{WhyLog, WhyLogEntry, WhyLogTail};
use crate::intent::schema::{EvidenceV1, WhyRecordV1};
use blake3::Hasher;

#[test]
fn test_whylog_creation() {
    let whylog = WhyLog::new(100);
    
    assert_eq!(whylog.len(), 0);
    assert!(whylog.is_empty());
    assert_eq!(whylog.capacity(), 100);
    assert_eq!(whylog.get_sequence_number(), 0);
}

#[test]
fn test_whylog_append_single_entry() {
    let mut whylog = WhyLog::new(100);
    
    let evidence = vec![
        EvidenceV1::new("intent_id".to_string(), "123".to_string()),
        EvidenceV1::new("intent_type".to_string(), "1".to_string()),
    ];
    
    let hash = whylog.append("Intent submitted".to_string(), evidence.clone(), 1000);
    
    assert_eq!(whylog.len(), 1);
    assert!(!whylog.is_empty());
    assert_eq!(whylog.get_sequence_number(), 1);
    
    // Verify the entry was added correctly
    let entries = whylog.get_entries_since([0u8; 32]);
    assert_eq!(entries.len(), 1);
    
    let entry = &entries[0];
    assert_eq!(entry.seq, 0);
    assert_eq!(entry.ts_vclock, 1000);
    assert_eq!(entry.reason, "Intent submitted");
    assert_eq!(entry.evidence, evidence);
    assert_eq!(entry.actor, "kernel");
    assert_eq!(entry.event, "intent_operation");
    assert_eq!(entry.prev_hash, [0u8; 32]);
    assert_eq!(entry.entry_hash, hash);
    
    // Verify signature
    assert_eq!(entry.signature.len(), crate::intent::whylog::DILITHIUM_SIGNATURE_SIZE);
}

#[test]
fn test_whylog_append_multiple_entries() {
    let mut whylog = WhyLog::new(100);
    
    let evidence1 = vec![EvidenceV1::new("step".to_string(), "1".to_string())];
    let evidence2 = vec![EvidenceV1::new("step".to_string(), "2".to_string())];
    let evidence3 = vec![EvidenceV1::new("step".to_string(), "3".to_string())];
    
    let hash1 = whylog.append("Step 1".to_string(), evidence1, 1000);
    let hash2 = whylog.append("Step 2".to_string(), evidence2, 1001);
    let hash3 = whylog.append("Step 3".to_string(), evidence3, 1002);
    
    assert_eq!(whylog.len(), 3);
    assert_eq!(whylog.get_sequence_number(), 3);
    
    // Verify chain continuity
    let entries = whylog.get_entries_since([0u8; 32]);
    assert_eq!(entries.len(), 3);
    
    assert_eq!(entries[0].seq, 0);
    assert_eq!(entries[0].entry_hash, hash1);
    assert_eq!(entries[0].prev_hash, [0u8; 32]);
    
    assert_eq!(entries[1].seq, 1);
    assert_eq!(entries[1].entry_hash, hash2);
    assert_eq!(entries[1].prev_hash, hash1);
    
    assert_eq!(entries[2].seq, 2);
    assert_eq!(entries[2].entry_hash, hash3);
    assert_eq!(entries[2].prev_hash, hash2);
}

#[test]
fn test_whylog_ring_buffer_behavior() {
    let mut whylog = WhyLog::new(3); // Small capacity to test ring behavior
    
    // Add 5 entries (should trigger ring buffer behavior)
    for i in 0..5 {
        let evidence = vec![EvidenceV1::new("step".to_string(), i.to_string())];
        whylog.append(format!("Step {}", i), evidence, 1000 + i);
    }
    
    // Should only keep the last 3 entries
    assert_eq!(whylog.len(), 3);
    assert_eq!(whylog.get_sequence_number(), 5);
    
    // Get all entries (should be the last 3)
    let entries = whylog.get_entries_since([0u8; 32]);
    assert_eq!(entries.len(), 3);
    
    // Verify sequence numbers are 2, 3, 4 (0 and 1 were dropped)
    assert_eq!(entries[0].seq, 2);
    assert_eq!(entries[1].seq, 3);
    assert_eq!(entries[2].seq, 4);
}

#[test]
fn test_whylog_chain_verification() {
    let mut whylog = WhyLog::new(100);
    
    // Add several entries
    for i in 0..5 {
        let evidence = vec![EvidenceV1::new("step".to_string(), i.to_string())];
        whylog.append(format!("Step {}", i), evidence, 1000 + i);
    }
    
    // Verify chain integrity
    assert!(whylog.verify_chain());
    
    // Get tail information
    let tail = whylog.get_tail();
    assert_eq!(tail.entries_count, 5);
    assert_eq!(tail.last_seq, 4);
    assert!(tail.chain_verified);
    assert!(!tail.truncated);
}

#[test]
fn test_whylog_get_entries_since() {
    let mut whylog = WhyLog::new(100);
    
    // Add several entries
    for i in 0..5 {
        let evidence = vec![EvidenceV1::new("step".to_string(), i.to_string())];
        whylog.append(format!("Step {}", i), evidence, 1000 + i);
    }
    
    // Get entries since the beginning
    let all_entries = whylog.get_entries_since([0u8; 32]);
    assert_eq!(all_entries.len(), 5);
    
    // Get entries since the second entry
    let entries_since_second = whylog.get_entries_since(all_entries[1].entry_hash);
    assert_eq!(entries_since_second.len(), 4); // Should get entries 1, 2, 3, 4
    
    // Get entries since the last entry
    let entries_since_last = whylog.get_entries_since(all_entries[4].entry_hash);
    assert_eq!(entries_since_last.len(), 0); // No entries after the last one
    
    // Get entries since a non-existent hash
    let entries_since_nonexistent = whylog.get_entries_since([0xFFu8; 32]);
    assert_eq!(entries_since_nonexistent.len(), 0);
}

#[test]
fn test_whylog_export_digest() {
    let mut whylog = WhyLog::new(100);
    
    // Add some entries
    for i in 0..3 {
        let evidence = vec![EvidenceV1::new("step".to_string(), i.to_string())];
        whylog.append(format!("Step {}", i), evidence, 1000 + i);
    }
    
    let digest = whylog.export_digest();
    
    // Digest should contain: tail_hash (32) + entries_count (8) + next_seq (8) = 48 bytes
    assert_eq!(digest.len(), 48);
    
    // Verify digest components
    let tail = whylog.get_tail();
    let expected_entries_count = (tail.entries_count as u64).to_le_bytes();
    let expected_next_seq = whylog.get_sequence_number().to_le_bytes();
    
    assert_eq!(&digest[32..40], &expected_entries_count);
    assert_eq!(&digest[40..48], &expected_next_seq);
}

#[test]
fn test_whylog_clear() {
    let mut whylog = WhyLog::new(100);
    
    // Add some entries
    for i in 0..3 {
        let evidence = vec![EvidenceV1::new("step".to_string(), i.to_string())];
        whylog.append(format!("Step {}", i), evidence, 1000 + i);
    }
    
    assert_eq!(whylog.len(), 3);
    assert_eq!(whylog.get_sequence_number(), 3);
    
    // Clear the why-log
    whylog.clear();
    
    assert_eq!(whylog.len(), 0);
    assert!(whylog.is_empty());
    assert_eq!(whylog.get_sequence_number(), 0);
    
    // Verify chain is still valid (empty chain is valid)
    assert!(whylog.verify_chain());
}

#[test]
fn test_whylog_helper_functions() {
    let mut whylog = WhyLog::new(100);
    
    // Test intent submission logging
    let intent = crate::intent::schema::IntentV1::new()
        .with_id(123)
        .with_intent_type(crate::intent::schema::INTENT_TYPE_BACKUP)
        .with_priority(crate::intent::schema::PRIORITY_HIGH);
    
    let hash1 = whylog.log_intent_submitted(&intent, 1000);
    assert_eq!(whylog.len(), 1);
    
    // Test constraint application logging
    let constraint = crate::intent::schema::ConstraintV1::new(
        crate::intent::schema::CONSTRAINT_TYPE_MAX_COST
    ).with_scalar_value(100);
    
    let hash2 = whylog.log_constraint_applied(&constraint, 1001);
    assert_eq!(whylog.len(), 2);
    
    // Test action selection logging
    let action = crate::intent::schema::ActionV1::new(
        crate::intent::schema::ACTION_KIND_SNAPSHOT
    ).with_cost_estimate(50).with_time_estimate(5000);
    
    let hash3 = whylog.log_action_selected(&action, 1002);
    assert_eq!(whylog.len(), 3);
    
    // Test policy decision logging
    let hash4 = whylog.log_policy_decision("allow", "capabilities_sufficient", 1003);
    assert_eq!(whylog.len(), 4);
    
    // Verify all entries were logged
    let entries = whylog.get_entries_since([0u8; 32]);
    assert_eq!(entries.len(), 4);
    
    assert_eq!(entries[0].event, "intent_operation");
    assert_eq!(entries[1].event, "intent_operation");
    assert_eq!(entries[2].event, "intent_operation");
    assert_eq!(entries[3].event, "intent_operation");
}

#[test]
fn test_whylog_kernel_key() {
    let whylog = WhyLog::new(100);
    let kernel_key = whylog.get_kernel_key();
    
    // Verify kernel key has expected pattern (stubbed for now)
    assert_eq!(kernel_key[0], 0x1a);
    assert_eq!(kernel_key[1], 0x2b);
    assert_eq!(kernel_key[2], 0x3c);
    assert_eq!(kernel_key[3], 0x4d);
    
    // Verify key size
    assert_eq!(kernel_key.len(), crate::intent::whylog::DILITHIUM_PUBLIC_KEY_SIZE);
}

#[test]
fn test_whylog_signature_verification() {
    let mut whylog = WhyLog::new(100);
    
    // Add an entry
    let evidence = vec![EvidenceV1::new("test".to_string(), "value".to_string())];
    whylog.append("Test entry".to_string(), evidence, 1000);
    
    // Verify chain integrity (which includes signature verification)
    assert!(whylog.verify_chain());
    
    // Get the entry and verify its signature
    let entries = whylog.get_entries_since([0u8; 32]);
    let entry = &entries[0];
    
    // Verify signature size
    assert_eq!(entry.signature.len(), crate::intent::whylog::DILITHIUM_SIGNATURE_SIZE);
    
    // Verify signature is deterministic (stubbed implementation)
    let expected_signature = {
        let mut hasher = Hasher::new();
        hasher.update(&entry.entry_hash);
        hasher.update(&whylog.get_kernel_key());
        hasher.update(&entry.seq.to_le_bytes());
        let hash_result = hasher.finalize();
        let hash_bytes: [u8; 32] = hash_result.into();
        
        let mut signature = Vec::with_capacity(crate::intent::whylog::DILITHIUM_SIGNATURE_SIZE);
        signature.extend_from_slice(&hash_bytes);
        signature.extend_from_slice(&vec![0u8; crate::intent::whylog::DILITHIUM_SIGNATURE_SIZE - 32]);
        signature
    };
    
    assert_eq!(entry.signature, expected_signature);
}

#[test]
fn test_whylog_large_capacity() {
    let mut whylog = WhyLog::new(10000);
    
    // Add many entries to test performance
    for i in 0..1000 {
        let evidence = vec![EvidenceV1::new("step".to_string(), i.to_string())];
        whylog.append(format!("Step {}", i), evidence, 1000 + i);
    }
    
    assert_eq!(whylog.len(), 1000);
    assert_eq!(whylog.get_sequence_number(), 1000);
    
    // Verify chain integrity with large number of entries
    assert!(whylog.verify_chain());
    
    // Test performance of getting entries
    let start = std::time::Instant::now();
    let entries = whylog.get_entries_since([0u8; 32]);
    let duration = start.elapsed();
    
    assert_eq!(entries.len(), 1000);
    
    // Should complete within reasonable time (less than 1ms)
    assert!(duration.as_millis() < 1);
}
