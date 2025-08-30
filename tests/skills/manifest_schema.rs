use crate::skills::manifest::*;
use serde_cbor;

#[test]
fn test_schema_version_and_hash() {
    assert_eq!(SKILL_SCHEMA_VERSION, 1);
    assert_eq!(SKILL_SCHEMA_HASH.len(), 32);
}

#[test]
fn test_manifest_validation() {
    let valid_manifest = SkillManifestV1::new("test_skill".to_string(), 1)
        .with_hostcalls(vec![HostcallId::WmQueryReadonly, HostcallId::EmitPlanAction])
        .with_capabilities(vec![])
        .with_memory_limit(16 * 1024 * 1024)
        .with_time_slice(100);
    
    assert!(valid_manifest.validate().is_ok());
    
    let oversized_name = SkillManifestV1::new(
        "a".repeat(SKILL_NAME_MAX_LEN + 1),
        1
    );
    assert!(oversized_name.validate().is_err());
    
    let too_many_hostcalls = SkillManifestV1::new("test".to_string(), 1)
        .with_hostcalls(vec![HostcallId::WmQueryReadonly; SKILL_HOSTCALLS_MAX + 1]);
    assert!(too_many_hostcalls.validate().is_err());
    
    let too_many_caps = SkillManifestV1::new("test".to_string(), 1)
        .with_capabilities(vec![crate::intent::schema::CapRef { scope_flags: 1 }; SKILL_CAPS_MAX + 1]);
    assert!(too_many_caps.validate().is_err());
    
    let memory_too_high = SkillManifestV1::new("test".to_string(), 1)
        .with_memory_limit(64 * 1024 * 1024);
    assert!(memory_too_high.validate().is_err());
    
    let time_too_high = SkillManifestV1::new("test".to_string(), 1)
        .with_time_slice(2000);
    assert!(time_too_high.validate().is_err());
    
    let non_deterministic = SkillManifestV1 {
        name: "test".to_string(),
        version: 1,
        hostcalls: vec![],
        requested_caps: vec![],
        mem_limit_bytes: 32 * 1024 * 1024,
        time_slice_ms: 250,
        deterministic: false,
        entry_point: "main".to_string(),
    };
    assert!(non_deterministic.validate().is_err());
}

#[test]
fn test_hostcall_id_conversion() {
    assert_eq!(HostcallId::WmQueryReadonly.to_u32(), 1);
    assert_eq!(HostcallId::EmitPlanAction.to_u32(), 2);
    assert_eq!(HostcallId::EmitEvidence.to_u32(), 3);
    assert_eq!(HostcallId::LogDebug.to_u32(), 4);
    assert_eq!(HostcallId::RngDeterministic.to_u32(), 5);
    
    assert_eq!(HostcallId::from_u32(1), Some(HostcallId::WmQueryReadonly));
    assert_eq!(HostcallId::from_u32(2), Some(HostcallId::EmitPlanAction));
    assert_eq!(HostcallId::from_u32(3), Some(HostcallId::EmitEvidence));
    assert_eq!(HostcallId::from_u32(4), Some(HostcallId::LogDebug));
    assert_eq!(HostcallId::from_u32(5), Some(HostcallId::RngDeterministic));
    assert_eq!(HostcallId::from_u32(99), None);
}

#[test]
fn test_hostcall_names() {
    assert_eq!(HostcallId::WmQueryReadonly.name(), "wm_query_readonly");
    assert_eq!(HostcallId::EmitPlanAction.name(), "emit_plan_action");
    assert_eq!(HostcallId::EmitEvidence.name(), "emit_evidence");
    assert_eq!(HostcallId::LogDebug.name(), "log_debug");
    assert_eq!(HostcallId::RngDeterministic.name(), "rng_deterministic");
}

#[test]
fn test_manifest_cbor_roundtrip() {
    let manifest = SkillManifestV1::new("test_skill".to_string(), 1)
        .with_hostcalls(vec![HostcallId::WmQueryReadonly])
        .with_capabilities(vec![])
        .with_memory_limit(16 * 1024 * 1024)
        .with_time_slice(100);
    
    let cbor_data = serde_cbor::to_vec(&manifest).unwrap();
    let deserialized: SkillManifestV1 = serde_cbor::from_slice(&cbor_data).unwrap();
    
    assert_eq!(manifest.name, deserialized.name);
    assert_eq!(manifest.version, deserialized.version);
    assert_eq!(manifest.hostcalls, deserialized.hostcalls);
    assert_eq!(manifest.mem_limit_bytes, deserialized.mem_limit_bytes);
    assert_eq!(manifest.time_slice_ms, deserialized.time_slice_ms);
    assert_eq!(manifest.deterministic, deserialized.deterministic);
    assert_eq!(manifest.entry_point, deserialized.entry_point);
}

#[test]
fn test_manifest_serialized_size() {
    let manifest = SkillManifestV1::new("test_skill".to_string(), 1);
    let size = manifest.serialized_size().unwrap();
    assert!(size > 0);
    assert!(size <= SKILL_MANIFEST_MAX_SIZE);
}

#[test]
fn test_manifest_constants() {
    assert_eq!(SKILL_MANIFEST_MAX_SIZE, 32 * 1024);
    assert_eq!(SKILL_NAME_MAX_LEN, 64);
    assert_eq!(SKILL_HOSTCALLS_MAX, 16);
    assert_eq!(SKILL_CAPS_MAX, 32);
    
    assert_eq!(HOSTCALL_NAMES.len(), 5);
    assert_eq!(HOSTCALL_NAMES[0].0, 1);
    assert_eq!(HOSTCALL_NAMES[0].1, "wm_query_readonly");
}

#[test]
fn test_manifest_builder_pattern() {
    let manifest = SkillManifestV1::new("test".to_string(), 1)
        .with_hostcalls(vec![HostcallId::WmQueryReadonly])
        .with_capabilities(vec![])
        .with_memory_limit(16 * 1024 * 1024)
        .with_time_slice(100)
        .with_entry_point("custom_main".to_string());
    
    assert_eq!(manifest.name, "test");
    assert_eq!(manifest.version, 1);
    assert_eq!(manifest.hostcalls.len(), 1);
    assert_eq!(manifest.requested_caps.len(), 0);
    assert_eq!(manifest.mem_limit_bytes, 16 * 1024 * 1024);
    assert_eq!(manifest.time_slice_ms, 100);
    assert_eq!(manifest.deterministic, true);
    assert_eq!(manifest.entry_point, "custom_main");
}
