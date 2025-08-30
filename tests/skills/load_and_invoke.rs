use crate::skills::*;
use crate::skills::manifest::*;
use crate::intent::schema::*;

#[test]
fn test_skill_load_and_invoke() {
    let manifest = SkillManifestV1::new("test_skill".to_string(), 1)
        .with_hostcalls(vec![HostcallId::WmQueryReadonly, HostcallId::EmitPlanAction])
        .with_capabilities(vec![])
        .with_memory_limit(16 * 1024 * 1024)
        .with_time_slice(100);
    
    let manifest_bytes = serde_cbor::to_vec(&manifest).unwrap();
    
    let wasm_bytes = create_test_wasm();
    
    let handle = load_skill(&manifest_bytes, &wasm_bytes).unwrap();
    assert!(handle.is_valid());
    assert_eq!(handle.name, "test_skill");
    assert_eq!(handle.version, 1);
    
    let input = b"test input";
    let bundle = invoke_preview(handle.id, input).unwrap();
    
    assert_eq!(bundle.preview.plan.actions.len(), 1);
    assert_eq!(bundle.preview.plan.actions[0].kind, 1);
    assert_eq!(bundle.preview.plan.actions[0].params.get("test"), Some(&"value".to_string()));
    
    assert_eq!(bundle.evidence.len(), 1);
    assert_eq!(bundle.evidence[0].key, "execution_time");
    assert!(bundle.evidence[0].value.parse::<u64>().is_ok());
    
    assert!(bundle.metrics.instructions_executed > 0);
    assert!(bundle.metrics.memory_used_bytes > 0);
    assert!(bundle.metrics.execution_time_us > 0);
    
    unload_skill(handle.id).unwrap();
}

#[test]
fn test_skill_load_validation() {
    let oversized_manifest = SkillManifestV1::new("test".to_string(), 1)
        .with_memory_limit(64 * 1024 * 1024);
    
    let manifest_bytes = serde_cbor::to_vec(&oversized_manifest).unwrap();
    let wasm_bytes = create_test_wasm();
    
    let result = load_skill(&manifest_bytes, &wasm_bytes);
    assert!(result.is_err());
    
    let oversized_wasm = vec![0u8; 64 * 1024 * 1024 + 1];
    let result = load_skill(&manifest_bytes, &oversized_wasm);
    assert!(result.is_err());
}

#[test]
fn test_skill_invoke_quota_enforcement() {
    let manifest = SkillManifestV1::new("test_skill".to_string(), 1)
        .with_memory_limit(1024)
        .with_time_slice(1);
    
    let manifest_bytes = serde_cbor::to_vec(&manifest).unwrap();
    let wasm_bytes = create_test_wasm();
    
    let handle = load_skill(&manifest_bytes, &wasm_bytes).unwrap();
    
    let large_input = vec![0u8; 128 * 1024];
    let result = invoke_preview(handle.id, &large_input);
    assert!(result.is_err());
    
    unload_skill(handle.id).unwrap();
}

#[test]
fn test_skill_hostcalls() {
    let manifest = SkillManifestV1::new("test_skill".to_string(), 1)
        .with_hostcalls(vec![
            HostcallId::WmQueryReadonly,
            HostcallId::EmitPlanAction,
            HostcallId::EmitEvidence,
            HostcallId::LogDebug,
            HostcallId::RngDeterministic,
        ]);
    
    let manifest_bytes = serde_cbor::to_vec(&manifest).unwrap();
    let wasm_bytes = create_test_wasm();
    
    let handle = load_skill(&manifest_bytes, &wasm_bytes).unwrap();
    
    let input = b"test hostcalls";
    let bundle = invoke_preview(handle.id, input).unwrap();
    
    assert!(bundle.preview.plan.actions.len() > 0);
    assert!(bundle.evidence.len() > 0);
    
    unload_skill(handle.id).unwrap();
}

#[test]
fn test_skill_registry_limits() {
    let mut handles = Vec::new();
    
    for i in 0..20 {
        let manifest = SkillManifestV1::new(format!("skill_{}", i), 1);
        let manifest_bytes = serde_cbor::to_vec(&manifest).unwrap();
        let wasm_bytes = create_test_wasm();
        
        match load_skill(&manifest_bytes, &wasm_bytes) {
            Ok(handle) => handles.push(handle),
            Err(_) => break,
        }
    }
    
    assert!(handles.len() <= 16);
    
    for handle in handles {
        unload_skill(handle.id).unwrap();
    }
}

fn create_test_wasm() -> Vec<u8> {
    let mut wasm = Vec::new();
    
    wasm.extend_from_slice(b"\x00asm"); // Magic
    wasm.extend_from_slice(b"\x01\x00\x00\x00"); // Version
    
    wasm.extend_from_slice(&[0x01]); // Type section
    wasm.extend_from_slice(&[0x07]); // Size
    wasm.extend_from_slice(&[0x01]); // Count
    wasm.extend_from_slice(&[0x60]); // Func type
    wasm.extend_from_slice(&[0x00]); // Params count
    wasm.extend_from_slice(&[0x00]); // Results count
    
    wasm.extend_from_slice(&[0x03]); // Function section
    wasm.extend_from_slice(&[0x02]); // Size
    wasm.extend_from_slice(&[0x01]); // Count
    wasm.extend_from_slice(&[0x00]); // Type index
    
    wasm.extend_from_slice(&[0x0A]); // Code section
    wasm.extend_from_slice(&[0x04]); // Size
    wasm.extend_from_slice(&[0x01]); // Count
    wasm.extend_from_slice(&[0x02]); // Function size
    wasm.extend_from_slice(&[0x00]); // Local count
    wasm.extend_from_slice(&[0x0B]); // End
    
    wasm.extend_from_slice(&[0x0B]); // Data section
    wasm.extend_from_slice(&[0x02]); // Size
    wasm.extend_from_slice(&[0x01]); // Count
    wasm.extend_from_slice(&[0x00]); // Memory index
    wasm.extend_from_slice(&[0x0B]); // Offset expr
    wasm.extend_from_slice(&[0x00]); // End
    wasm.extend_from_slice(&[0x01]); // Data size
    wasm.extend_from_slice(&[0x74]); // 't'
    
    wasm
}
