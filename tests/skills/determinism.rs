use crate::skills::*;
use crate::skills::manifest::*;
use crate::skills::exec::*;
use std::collections::HashMap;

#[test]
fn test_skill_deterministic_execution() {
    let manifest = SkillManifestV1::new("test_skill".to_string(), 1)
        .with_hostcalls(vec![HostcallId::RngDeterministic])
        .with_memory_limit(1024 * 1024)
        .with_time_slice(1000);
    
    let manifest_bytes = serde_cbor::to_vec(&manifest).unwrap();
    let wasm_bytes = create_deterministic_wasm();
    
    let handle = load_skill(&manifest_bytes, &wasm_bytes).unwrap();
    
    let input = b"deterministic_test";
    
    let bundle1 = invoke_preview(handle.id, input).unwrap();
    let bundle2 = invoke_preview(handle.id, input).unwrap();
    let bundle3 = invoke_preview(handle.id, input).unwrap();
    
    assert_eq!(bundle1.preview.plan.actions.len(), bundle2.preview.plan.actions.len());
    assert_eq!(bundle2.preview.plan.actions.len(), bundle3.preview.plan.actions.len());
    
    for i in 0..bundle1.preview.plan.actions.len() {
        assert_eq!(bundle1.preview.plan.actions[i].kind, bundle2.preview.plan.actions[i].kind);
        assert_eq!(bundle2.preview.plan.actions[i].kind, bundle3.preview.plan.actions[i].kind);
        
        assert_eq!(bundle1.preview.plan.actions[i].params, bundle2.preview.plan.actions[i].params);
        assert_eq!(bundle2.preview.plan.actions[i].params, bundle3.preview.plan.actions[i].params);
    }
    
    assert_eq!(bundle1.evidence.len(), bundle2.evidence.len());
    assert_eq!(bundle2.evidence.len(), bundle3.evidence.len());
    
    for i in 0..bundle1.evidence.len() {
        assert_eq!(bundle1.evidence[i].key, bundle2.evidence[i].key);
        assert_eq!(bundle2.evidence[i].key, bundle3.evidence[i].key);
        assert_eq!(bundle1.evidence[i].value, bundle2.evidence[i].value);
        assert_eq!(bundle2.evidence[i].value, bundle3.evidence[i].value);
    }
    
    unload_skill(handle.id).unwrap();
}

#[test]
fn test_skill_deterministic_rng() {
    let manifest = SkillManifestV1::new("test_skill".to_string(), 1)
        .with_hostcalls(vec![HostcallId::RngDeterministic])
        .with_memory_limit(1024 * 1024)
        .with_time_slice(1000);
    
    let manifest_bytes = serde_cbor::to_vec(&manifest).unwrap();
    let wasm_bytes = create_rng_test_wasm();
    
    let handle = load_skill(&manifest_bytes, &wasm_bytes).unwrap();
    
    let input = b"rng_test";
    
    let bundle1 = invoke_preview(handle.id, input).unwrap();
    let bundle2 = invoke_preview(handle.id, input).unwrap();
    
    let rng_evidence1 = bundle1.evidence.iter()
        .find(|e| e.key == "rng_output")
        .unwrap();
    let rng_evidence2 = bundle2.evidence.iter()
        .find(|e| e.key == "rng_output")
        .unwrap();
    
    assert_eq!(rng_evidence1.value, rng_evidence2.value);
    
    unload_skill(handle.id).unwrap();
}

#[test]
fn test_skill_deterministic_metrics() {
    let manifest = SkillManifestV1::new("test_skill".to_string(), 1)
        .with_memory_limit(1024 * 1024)
        .with_time_slice(1000);
    
    let manifest_bytes = serde_cbor::to_vec(&manifest).unwrap();
    let wasm_bytes = create_simple_wasm();
    
    let handle = load_skill(&manifest_bytes, &wasm_bytes).unwrap();
    
    let input = b"metrics_test";
    
    let bundle1 = invoke_preview(handle.id, input).unwrap();
    let bundle2 = invoke_preview(handle.id, input).unwrap();
    
    assert_eq!(bundle1.metrics.instructions_executed, bundle2.metrics.instructions_executed);
    assert_eq!(bundle1.metrics.memory_used_bytes, bundle2.metrics.memory_used_bytes);
    assert_eq!(bundle1.metrics.hostcalls_made, bundle2.metrics.hostcalls_made);
    
    unload_skill(handle.id).unwrap();
}

#[test]
fn test_skill_deterministic_hostcalls() {
    let manifest = SkillManifestV1::new("test_skill".to_string(), 1)
        .with_hostcalls(vec![
            HostcallId::EmitPlanAction,
            HostcallId::EmitEvidence,
            HostcallId::LogDebug,
        ])
        .with_memory_limit(1024 * 1024)
        .with_time_slice(1000);
    
    let manifest_bytes = serde_cbor::to_vec(&manifest).unwrap();
    let wasm_bytes = create_hostcall_test_wasm();
    
    let handle = load_skill(&manifest_bytes, &wasm_bytes).unwrap();
    
    let input = b"hostcall_test";
    
    let bundle1 = invoke_preview(handle.id, input).unwrap();
    let bundle2 = invoke_preview(handle.id, input).unwrap();
    
    assert_eq!(bundle1.preview.plan.actions.len(), bundle2.preview.plan.actions.len());
    assert_eq!(bundle1.evidence.len(), bundle2.evidence.len());
    
    for i in 0..bundle1.preview.plan.actions.len() {
        let action1 = &bundle1.preview.plan.actions[i];
        let action2 = &bundle2.preview.plan.actions[i];
        
        assert_eq!(action1.kind, action2.kind);
        assert_eq!(action1.params, action2.params);
    }
    
    for i in 0..bundle1.evidence.len() {
        let evidence1 = &bundle1.evidence[i];
        let evidence2 = &bundle2.evidence[i];
        
        assert_eq!(evidence1.key, evidence2.key);
        assert_eq!(evidence1.value, evidence2.value);
    }
    
    unload_skill(handle.id).unwrap();
}

#[test]
fn test_skill_deterministic_memory_layout() {
    let manifest = SkillManifestV1::new("test_skill".to_string(), 1)
        .with_memory_limit(1024 * 1024)
        .with_time_slice(1000);
    
    let manifest_bytes = serde_cbor::to_vec(&manifest).unwrap();
    let wasm_bytes = create_memory_test_wasm();
    
    let handle = load_skill(&manifest_bytes, &wasm_bytes).unwrap();
    
    let input = b"memory_test";
    
    let bundle1 = invoke_preview(handle.id, input).unwrap();
    let bundle2 = invoke_preview(handle.id, input).unwrap();
    
    assert_eq!(bundle1.metrics.memory_used_bytes, bundle2.metrics.memory_used_bytes);
    
    unload_skill(handle.id).unwrap();
}

#[test]
fn test_skill_deterministic_execution_order() {
    let manifest = SkillManifestV1::new("test_skill".to_string(), 1)
        .with_hostcalls(vec![HostcallId::EmitPlanAction])
        .with_memory_limit(1024 * 1024)
        .with_time_slice(1000);
    
    let manifest_bytes = serde_cbor::to_vec(&manifest).unwrap();
    let wasm_bytes = create_order_test_wasm();
    
    let handle = load_skill(&manifest_bytes, &wasm_bytes).unwrap();
    
    let input = b"order_test";
    
    let bundle1 = invoke_preview(handle.id, input).unwrap();
    let bundle2 = invoke_preview(handle.id, input).unwrap();
    
    assert_eq!(bundle1.preview.plan.actions.len(), bundle2.preview.plan.actions.len());
    
    for i in 0..bundle1.preview.plan.actions.len() {
        let action1 = &bundle1.preview.plan.actions[i];
        let action2 = &bundle2.preview.plan.actions[i];
        
        assert_eq!(action1.kind, action2.kind);
        assert_eq!(action1.params.get("order"), action2.params.get("order"));
    }
    
    unload_skill(handle.id).unwrap();
}

fn create_deterministic_wasm() -> Vec<u8> {
    let mut wasm = Vec::new();
    
    wasm.extend_from_slice(b"\x00asm");
    wasm.extend_from_slice(b"\x01\x00\x00\x00");
    
    wasm.extend_from_slice(&[0x01, 0x07, 0x01, 0x60, 0x00, 0x00]);
    wasm.extend_from_slice(&[0x03, 0x02, 0x01, 0x00]);
    wasm.extend_from_slice(&[0x0A, 0x04, 0x01, 0x02, 0x00, 0x0B]);
    
    wasm
}

fn create_rng_test_wasm() -> Vec<u8> {
    let mut wasm = Vec::new();
    
    wasm.extend_from_slice(b"\x00asm");
    wasm.extend_from_slice(b"\x01\x00\x00\x00");
    
    wasm.extend_from_slice(&[0x01, 0x07, 0x01, 0x60, 0x00, 0x00]);
    wasm.extend_from_slice(&[0x03, 0x02, 0x01, 0x00]);
    wasm.extend_from_slice(&[0x0A, 0x04, 0x01, 0x02, 0x00, 0x0B]);
    
    wasm
}

fn create_hostcall_test_wasm() -> Vec<u8> {
    let mut wasm = Vec::new();
    
    wasm.extend_from_slice(b"\x00asm");
    wasm.extend_from_slice(b"\x01\x00\x00\x00");
    
    wasm.extend_from_slice(&[0x01, 0x07, 0x01, 0x60, 0x00, 0x00]);
    wasm.extend_from_slice(&[0x03, 0x02, 0x01, 0x00]);
    wasm.extend_from_slice(&[0x0A, 0x04, 0x01, 0x02, 0x00, 0x0B]);
    
    wasm
}

fn create_memory_test_wasm() -> Vec<u8> {
    let mut wasm = Vec::new();
    
    wasm.extend_from_slice(b"\x00asm");
    wasm.extend_from_slice(b"\x01\x00\x00\x00");
    
    wasm.extend_from_slice(&[0x01, 0x07, 0x01, 0x60, 0x00, 0x00]);
    wasm.extend_from_slice(&[0x03, 0x02, 0x01, 0x00]);
    wasm.extend_from_slice(&[0x0A, 0x04, 0x01, 0x02, 0x00, 0x0B]);
    
    wasm
}

fn create_order_test_wasm() -> Vec<u8> {
    let mut wasm = Vec::new();
    
    wasm.extend_from_slice(b"\x00asm");
    wasm.extend_from_slice(b"\x01\x00\x00\x00");
    
    wasm.extend_from_slice(&[0x01, 0x07, 0x01, 0x60, 0x00, 0x00]);
    wasm.extend_from_slice(&[0x03, 0x02, 0x01, 0x00]);
    wasm.extend_from_slice(&[0x0A, 0x04, 0x01, 0x02, 0x00, 0x0B]);
    
    wasm
}

fn create_simple_wasm() -> Vec<u8> {
    let mut wasm = Vec::new();
    
    wasm.extend_from_slice(b"\x00asm");
    wasm.extend_from_slice(b"\x01\x00\x00\x00");
    
    wasm.extend_from_slice(&[0x01, 0x07, 0x01, 0x60, 0x00, 0x00]);
    wasm.extend_from_slice(&[0x03, 0x02, 0x01, 0x00]);
    wasm.extend_from_slice(&[0x0A, 0x04, 0x01, 0x02, 0x00, 0x0B]);
    
    wasm
}
