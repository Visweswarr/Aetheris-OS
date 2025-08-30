use crate::skills::*;
use crate::skills::manifest::*;
use crate::skills::exec::*;

#[test]
fn test_instruction_limit_enforcement() {
    let manifest = SkillManifestV1::new("test_skill".to_string(), 1)
        .with_memory_limit(1024 * 1024)
        .with_time_slice(1000);
    
    let manifest_bytes = serde_cbor::to_vec(&manifest).unwrap();
    let wasm_bytes = create_infinite_loop_wasm();
    
    let handle = load_skill(&manifest_bytes, &wasm_bytes).unwrap();
    
    let input = b"test";
    let result = invoke_preview(handle.id, input);
    
    match result {
        Ok(_) => panic!("Expected instruction limit to be exceeded"),
        Err(e) => {
            assert!(matches!(e, SkillsError::ExecutionFailed(_)));
        }
    }
    
    unload_skill(handle.id).unwrap();
}

#[test]
fn test_memory_limit_enforcement() {
    let manifest = SkillManifestV1::new("test_skill".to_string(), 1)
        .with_memory_limit(1024)
        .with_time_slice(1000);
    
    let manifest_bytes = serde_cbor::to_vec(&manifest).unwrap();
    let wasm_bytes = create_memory_hog_wasm();
    
    let handle = load_skill(&manifest_bytes, &wasm_bytes).unwrap();
    
    let input = b"test";
    let result = invoke_preview(handle.id, input);
    
    match result {
        Ok(_) => panic!("Expected memory limit to be exceeded"),
        Err(e) => {
            assert!(matches!(e, SkillsError::ExecutionFailed(_)));
        }
    }
    
    unload_skill(handle.id).unwrap();
}

#[test]
fn test_time_limit_enforcement() {
    let manifest = SkillManifestV1::new("test_skill".to_string(), 1)
        .with_memory_limit(1024 * 1024)
        .with_time_slice(1);
    
    let manifest_bytes = serde_cbor::to_vec(&manifest).unwrap();
    let wasm_bytes = create_slow_wasm();
    
    let handle = load_skill(&manifest_bytes, &wasm_bytes).unwrap();
    
    let input = b"test";
    let result = invoke_preview(handle.id, input);
    
    match result {
        Ok(_) => panic!("Expected time limit to be exceeded"),
        Err(e) => {
            assert!(matches!(e, SkillsError::ExecutionFailed(_)));
        }
    }
    
    unload_skill(handle.id).unwrap();
}

#[test]
fn test_combined_quota_enforcement() {
    let manifest = SkillManifestV1::new("test_skill".to_string(), 1)
        .with_memory_limit(512)
        .with_time_slice(50);
    
    let manifest_bytes = serde_cbor::to_vec(&manifest).unwrap();
    let wasm_bytes = create_quota_test_wasm();
    
    let handle = load_skill(&manifest_bytes, &wasm_bytes).unwrap();
    
    let input = b"test";
    let result = invoke_preview(handle.id, input);
    
    match result {
        Ok(_) => panic!("Expected quota to be exceeded"),
        Err(e) => {
            assert!(matches!(e, SkillsError::ExecutionFailed(_)));
        }
    }
    
    unload_skill(handle.id).unwrap();
}

#[test]
fn test_quota_metrics_collection() {
    let manifest = SkillManifestV1::new("test_skill".to_string(), 1)
        .with_memory_limit(1024 * 1024)
        .with_time_slice(1000);
    
    let manifest_bytes = serde_cbor::to_vec(&manifest).unwrap();
    let wasm_bytes = create_simple_wasm();
    
    let handle = load_skill(&manifest_bytes, &wasm_bytes).unwrap();
    
    let input = b"test";
    let bundle = invoke_preview(handle.id, input).unwrap();
    
    let metrics = bundle.metrics;
    assert!(metrics.instructions_executed > 0);
    assert!(metrics.memory_used_bytes > 0);
    assert!(metrics.execution_time_us > 0);
    assert!(metrics.hostcalls_made >= 0);
    
    unload_skill(handle.id).unwrap();
}

#[test]
fn test_quota_policy_validation() {
    let oversized_manifest = SkillManifestV1::new("test".to_string(), 1)
        .with_memory_limit(64 * 1024 * 1024)
        .with_time_slice(2000);
    
    let manifest_bytes = serde_cbor::to_vec(&oversized_manifest).unwrap();
    let wasm_bytes = create_simple_wasm();
    
    let result = load_skill(&manifest_bytes, &wasm_bytes);
    assert!(result.is_err());
    
    let non_deterministic_manifest = SkillManifestV1 {
        name: "test".to_string(),
        version: 1,
        hostcalls: vec![],
        requested_caps: vec![],
        mem_limit_bytes: 16 * 1024 * 1024,
        time_slice_ms: 250,
        deterministic: false,
        entry_point: "main".to_string(),
    };
    
    let manifest_bytes = serde_cbor::to_vec(&non_deterministic_manifest).unwrap();
    let result = load_skill(&manifest_bytes, &wasm_bytes);
    assert!(result.is_err());
}

fn create_infinite_loop_wasm() -> Vec<u8> {
    let mut wasm = Vec::new();
    
    wasm.extend_from_slice(b"\x00asm");
    wasm.extend_from_slice(b"\x01\x00\x00\x00");
    
    wasm.extend_from_slice(&[0x01, 0x07, 0x01, 0x60, 0x00, 0x00]);
    wasm.extend_from_slice(&[0x03, 0x02, 0x01, 0x00]);
    wasm.extend_from_slice(&[0x0A, 0x08, 0x01, 0x06, 0x00, 0x03, 0x00, 0x0B, 0x0B]);
    
    wasm
}

fn create_memory_hog_wasm() -> Vec<u8> {
    let mut wasm = Vec::new();
    
    wasm.extend_from_slice(b"\x00asm");
    wasm.extend_from_slice(b"\x01\x00\x00\x00");
    
    wasm.extend_from_slice(&[0x01, 0x07, 0x01, 0x60, 0x00, 0x00]);
    wasm.extend_from_slice(&[0x03, 0x02, 0x01, 0x00]);
    wasm.extend_from_slice(&[0x0A, 0x0A, 0x01, 0x08, 0x00, 0x41, 0x80, 0x80, 0x80, 0x80, 0x0B]);
    
    wasm
}

fn create_slow_wasm() -> Vec<u8> {
    let mut wasm = Vec::new();
    
    wasm.extend_from_slice(b"\x00asm");
    wasm.extend_from_slice(b"\x01\x00\x00\x00");
    
    wasm.extend_from_slice(&[0x01, 0x07, 0x01, 0x60, 0x00, 0x00]);
    wasm.extend_from_slice(&[0x03, 0x02, 0x01, 0x00]);
    wasm.extend_from_slice(&[0x0A, 0x0C, 0x01, 0x0A, 0x00, 0x41, 0x00, 0x41, 0x01, 0x4E, 0x0B, 0x0B]);
    
    wasm
}

fn create_quota_test_wasm() -> Vec<u8> {
    let mut wasm = Vec::new();
    
    wasm.extend_from_slice(b"\x00asm");
    wasm.extend_from_slice(b"\x01\x00\x00\x00");
    
    wasm.extend_from_slice(&[0x01, 0x07, 0x01, 0x60, 0x00, 0x00]);
    wasm.extend_from_slice(&[0x03, 0x02, 0x01, 0x00]);
    wasm.extend_from_slice(&[0x0A, 0x0E, 0x01, 0x0C, 0x00, 0x41, 0x80, 0x80, 0x80, 0x80, 0x41, 0x00, 0x0B, 0x0B]);
    
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
