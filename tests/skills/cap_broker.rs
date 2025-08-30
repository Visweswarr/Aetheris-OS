use crate::skills::broker::*;
use crate::skills::manifest::*;
use crate::intent::schema::CapRef;

#[test]
fn test_broker_session_creation() {
    let broker = CapabilityBroker::new();
    let manifest = SkillManifestV1::new("test_skill".to_string(), 1)
        .with_capabilities(vec![
            CapRef { scope_flags: 1 << 10 },
            CapRef { scope_flags: 1 << 11 },
        ]);
    
    let session = broker.create_session(&manifest).unwrap();
    assert_eq!(session.skill_id, 1);
    assert!(session.preview_only);
    assert_eq!(session.caps.len(), 2);
    
    for cap in &session.caps {
        assert!(cap.scope_flags & PREVIEW_SCOPE_FLAG != 0);
        assert!(cap.scope_flags & SKILL_SCOPE_FLAG != 0);
    }
}

#[test]
fn test_broker_policy_validation() {
    let broker = CapabilityBroker::new();
    
    let valid_manifest = SkillManifestV1::new("test".to_string(), 1)
        .with_memory_limit(16 * 1024 * 1024)
        .with_time_slice(250);
    
    let policy_decision = broker.validate_policy(&valid_manifest).unwrap();
    assert!(matches!(policy_decision, PolicyDecision::Allow));
    
    let oversized_memory = SkillManifestV1::new("test".to_string(), 1)
        .with_memory_limit(64 * 1024 * 1024);
    
    let policy_decision = broker.validate_policy(&oversized_memory).unwrap();
    match policy_decision {
        PolicyDecision::Deny(reason) => {
            assert!(reason.contains("Memory limit too high"));
        }
        _ => panic!("Expected deny decision"),
    }
    
    let oversized_time = SkillManifestV1::new("test".to_string(), 1)
        .with_time_slice(2000);
    
    let policy_decision = broker.validate_policy(&oversized_time).unwrap();
    match policy_decision {
        PolicyDecision::Deny(reason) => {
            assert!(reason.contains("Time slice too high"));
        }
        _ => panic!("Expected deny decision"),
    }
    
    let non_deterministic = SkillManifestV1 {
        name: "test".to_string(),
        version: 1,
        hostcalls: vec![],
        requested_caps: vec![],
        mem_limit_bytes: 16 * 1024 * 1024,
        time_slice_ms: 250,
        deterministic: false,
        entry_point: "main".to_string(),
    };
    
    let policy_decision = broker.validate_policy(&non_deterministic).unwrap();
    match policy_decision {
        PolicyDecision::Deny(reason) => {
            assert!(reason.contains("Non-deterministic skills not allowed"));
        }
        _ => panic!("Expected deny decision"),
    }
}

#[test]
fn test_broker_hostcall_validation() {
    let broker = CapabilityBroker::new();
    
    let valid_hostcalls = SkillManifestV1::new("test".to_string(), 1)
        .with_hostcalls(vec![
            HostcallId::WmQueryReadonly,
            HostcallId::EmitPlanAction,
            HostcallId::EmitEvidence,
            HostcallId::LogDebug,
            HostcallId::RngDeterministic,
        ]);
    
    let policy_decision = broker.validate_policy(&valid_hostcalls).unwrap();
    assert!(matches!(policy_decision, PolicyDecision::Allow));
    
    let too_many_hostcalls = SkillManifestV1::new("test".to_string(), 1)
        .with_hostcalls(vec![HostcallId::WmQueryReadonly; 20]);
    
    let policy_decision = broker.validate_policy(&too_many_hostcalls).unwrap();
    match policy_decision {
        PolicyDecision::Deny(reason) => {
            assert!(reason.contains("Too many hostcalls requested"));
        }
        _ => panic!("Expected deny decision"),
    }
}

#[test]
fn test_broker_capability_limits() {
    let broker = CapabilityBroker::new();
    
    let too_many_caps = SkillManifestV1::new("test".to_string(), 1)
        .with_capabilities(vec![CapRef { scope_flags: 1 }; 40]);
    
    let policy_decision = broker.validate_policy(&too_many_caps).unwrap();
    match policy_decision {
        PolicyDecision::Deny(reason) => {
            assert!(reason.contains("Too many capabilities requested"));
        }
        _ => panic!("Expected deny decision"),
    }
}

#[test]
fn test_broker_session_management() {
    let broker = CapabilityBroker::new();
    
    let manifest = SkillManifestV1::new("test".to_string(), 1);
    let session1 = broker.create_session(&manifest).unwrap();
    let session2 = broker.create_session(&manifest).unwrap();
    
    assert_eq!(session1.skill_id, 1);
    assert_eq!(session2.skill_id, 2);
    
    let retrieved1 = broker.get_session(1);
    let retrieved2 = broker.get_session(2);
    
    assert!(retrieved1.is_some());
    assert!(retrieved2.is_some());
    assert_eq!(retrieved1.unwrap().skill_id, 1);
    assert_eq!(retrieved2.unwrap().skill_id, 2);
    
    assert!(broker.remove_session(1));
    assert!(broker.remove_session(2));
    
    let retrieved1 = broker.get_session(1);
    let retrieved2 = broker.get_session(2);
    
    assert!(retrieved1.is_none());
    assert!(retrieved2.is_none());
}

#[test]
fn test_broker_session_zeroization() {
    let broker = CapabilityBroker::new();
    let manifest = SkillManifestV1::new("test".to_string(), 1)
        .with_capabilities(vec![CapRef { scope_flags: 1 << 10 }]);
    
    let session = broker.create_session(&manifest).unwrap();
    assert_eq!(session.caps.len(), 1);
    
    let cap_scope = session.caps[0].scope_flags;
    assert!(cap_scope & PREVIEW_SCOPE_FLAG != 0);
    assert!(cap_scope & SKILL_SCOPE_FLAG != 0);
    
    broker.remove_session(session.skill_id);
    
    let retrieved = broker.get_session(session.skill_id);
    assert!(retrieved.is_none());
}

#[test]
fn test_broker_active_sessions_count() {
    let broker = CapabilityBroker::new();
    
    assert_eq!(broker.get_active_sessions_count(), 0);
    
    let manifest = SkillManifestV1::new("test".to_string(), 1);
    
    let _session1 = broker.create_session(&manifest).unwrap();
    assert_eq!(broker.get_active_sessions_count(), 1);
    
    let _session2 = broker.create_session(&manifest).unwrap();
    assert_eq!(broker.get_active_sessions_count(), 2);
    
    broker.remove_session(1);
    assert_eq!(broker.get_active_sessions_count(), 1);
    
    broker.remove_session(2);
    assert_eq!(broker.get_active_sessions_count(), 0);
}

#[test]
fn test_broker_nonce_generation() {
    let broker = CapabilityBroker::new();
    let manifest = SkillManifestV1::new("test".to_string(), 1);
    
    let session1 = broker.create_session(&manifest).unwrap();
    let session2 = broker.create_session(&manifest).unwrap();
    
    let cap1 = &session1.caps[0];
    let cap2 = &session2.caps[0];
    
    assert_ne!(cap1.nonce, cap2.nonce);
    assert!(cap1.nonce > 0);
    assert!(cap2.nonce > 0);
}
