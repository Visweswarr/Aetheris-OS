#![no_main]

use libfuzzer_sys::fuzz_target;
use polymera_caps::*;

/// Fuzzer target for capability parser
/// 
/// This fuzzer tests the capability parser with various malformed inputs
/// to ensure it handles abuse attempts gracefully without crashes or
/// security vulnerabilities.
fuzz_target!(|data: &[u8]| {
    // Skip empty inputs
    if data.is_empty() {
        return;
    }
    
    // Test 1: Raw byte parsing
    test_raw_capability_parsing(data);
    
    // Test 2: Base64 decoding
    test_base64_capability_parsing(data);
    
    // Test 3: JSON parsing
    test_json_capability_parsing(data);
    
    // Test 4: Binary format parsing
    test_binary_capability_parsing(data);
    
    // Test 5: Boundary conditions
    test_boundary_conditions(data);
    
    // Test 6: Malformed header parsing
    test_malformed_header_parsing(data);
    
    // Test 7: Invalid signature parsing
    test_invalid_signature_parsing(data);
    
    // Test 8: Corrupted data parsing
    test_corrupted_data_parsing(data);
});

/// Test raw capability parsing with arbitrary bytes
fn test_raw_capability_parsing(data: &[u8]) {
    // Try to parse as raw capability data
    // This should never crash, even with completely random data
    
    // Test with different slice lengths
    for len in [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024] {
        if data.len() >= len {
            let slice = &data[..len];
            
            // Test capability token parsing
            match CapTokenV2::from_bytes(slice) {
                Ok(_) => {
                    // Valid capability parsed successfully
                    // This is fine - we're testing with real capability data
                }
                Err(_) => {
                    // Invalid capability data - this is expected and should not crash
                    // Verify that the error is handled gracefully
                    assert!(slice.len() < CapTokenV2::MIN_SIZE || 
                           slice.len() > CapTokenV2::MAX_SIZE ||
                           !is_valid_capability_format(slice));
                }
            }
        }
    }
}

/// Test base64 capability parsing
fn test_base64_capability_parsing(data: &[u8]) {
    // Try to decode as base64 and then parse as capability
    if let Ok(decoded) = base64::decode(data) {
        // Test the decoded data
        test_raw_capability_parsing(&decoded);
    }
    
    // Test with base64-encoded capability strings
    if let Ok(encoded) = String::from_utf8(data.to_vec()) {
        if let Ok(decoded) = base64::decode(&encoded) {
            test_raw_capability_parsing(&decoded);
        }
    }
}

/// Test JSON capability parsing
fn test_json_capability_parsing(data: &[u8]) {
    // Try to parse as JSON capability representation
    if let Ok(json_str) = String::from_utf8(data.to_vec()) {
        // Test JSON parsing - should not crash even with malformed JSON
        match serde_json::from_str::<serde_json::Value>(&json_str) {
            Ok(json_value) => {
                // Try to extract capability data from JSON
                if let Some(cap_data) = json_value.get("capability") {
                    if let Some(cap_str) = cap_data.as_str() {
                        if let Ok(decoded) = base64::decode(cap_str) {
                            test_raw_capability_parsing(&decoded);
                        }
                    }
                }
            }
            Err(_) => {
                // Malformed JSON - this is expected and should not crash
            }
        }
    }
}

/// Test binary capability format parsing
fn test_binary_capability_parsing(data: &[u8]) {
    // Test with various binary format assumptions
    // This tests the parser's resilience to format confusion attacks
    
    if data.len() >= 4 {
        // Test with different magic number interpretations
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        
        // Test with swapped endianness
        let magic_be = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        
        // Test with different size interpretations
        if data.len() >= 8 {
            let size_le = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
            let size_be = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
            
            // Test size validation
            if size_le > 0 && size_le <= 0x10000 {
                test_with_assumed_size(data, size_le as usize);
            }
            
            if size_be > 0 && size_be <= 0x10000 {
                test_with_assumed_size(data, size_be as usize);
            }
        }
    }
}

/// Test boundary conditions
fn test_boundary_conditions(data: &[u8]) {
    // Test with minimum and maximum sizes
    if data.len() >= CapTokenV2::MIN_SIZE {
        test_raw_capability_parsing(&data[..CapTokenV2::MIN_SIZE]);
    }
    
    if data.len() >= CapTokenV2::MAX_SIZE {
        test_raw_capability_parsing(&data[..CapTokenV2::MAX_SIZE]);
    }
    
    // Test with sizes just below and above boundaries
    if data.len() >= CapTokenV2::MIN_SIZE - 1 {
        test_raw_capability_parsing(&data[..(CapTokenV2::MIN_SIZE - 1)]);
    }
    
    if data.len() >= CapTokenV2::MAX_SIZE + 1 {
        test_raw_capability_parsing(&data[..(CapTokenV2::MAX_SIZE + 1)]);
    }
    
    // Test with zero-sized slices
    test_raw_capability_parsing(&[]);
    
    // Test with single-byte slices
    if !data.is_empty() {
        test_raw_capability_parsing(&data[..1]);
    }
}

/// Test malformed header parsing
fn test_malformed_header_parsing(data: &[u8]) {
    // Test with corrupted header fields
    if data.len() >= 32 {
        // Test with invalid version numbers
        let mut corrupted = data.to_vec();
        if corrupted.len() >= 4 {
            corrupted[0] = 0xFF; // Invalid version
            test_raw_capability_parsing(&corrupted);
        }
        
        // Test with invalid magic numbers
        if corrupted.len() >= 8 {
            corrupted[4..8].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]); // Invalid magic
            test_raw_capability_parsing(&corrupted);
        }
        
        // Test with invalid flags
        if corrupted.len() >= 12 {
            corrupted[8] = 0xFF; // Invalid flags
            test_raw_capability_parsing(&corrupted);
        }
    }
}

/// Test invalid signature parsing
fn test_invalid_signature_parsing(data: &[u8]) {
    // Test with corrupted signature data
    if data.len() >= 64 {
        let mut corrupted = data.to_vec();
        
        // Corrupt signature bytes
        for i in 32..64 {
            if i < corrupted.len() {
                corrupted[i] = corrupted[i].wrapping_add(1);
            }
        }
        
        test_raw_capability_parsing(&corrupted);
        
        // Test with zeroed signature
        for i in 32..64 {
            if i < corrupted.len() {
                corrupted[i] = 0;
            }
        }
        
        test_raw_capability_parsing(&corrupted);
        
        // Test with repeated signature bytes
        for i in 32..64 {
            if i < corrupted.len() {
                corrupted[i] = 0xAA;
            }
        }
        
        test_raw_capability_parsing(&corrupted);
    }
}

/// Test corrupted data parsing
fn test_corrupted_data_parsing(data: &[u8]) {
    // Test with various corruption patterns
    if data.len() >= 16 {
        let mut corrupted = data.to_vec();
        
        // Bit flipping
        for i in 0..corrupted.len().min(16) {
            corrupted[i] ^= 0x01; // Flip LSB
            test_raw_capability_parsing(&corrupted);
            corrupted[i] ^= 0x01; // Restore
        }
        
        // Byte shifting
        if corrupted.len() >= 32 {
            for i in 0..corrupted.len() - 1 {
                let temp = corrupted[i];
                corrupted[i] = corrupted[i + 1];
                corrupted[i + 1] = temp;
                test_raw_capability_parsing(&corrupted);
                // Restore
                corrupted[i + 1] = corrupted[i];
                corrupted[i] = temp;
            }
        }
        
        // Null byte injection
        for i in 0..corrupted.len().min(16) {
            let original = corrupted[i];
            corrupted[i] = 0;
            test_raw_capability_parsing(&corrupted);
            corrupted[i] = original;
        }
    }
}

/// Test with assumed size
fn test_with_assumed_size(data: &[u8], assumed_size: usize) {
    if data.len() >= assumed_size {
        test_raw_capability_parsing(&data[..assumed_size]);
    }
}

/// Check if data appears to be in valid capability format
fn is_valid_capability_format(data: &[u8]) -> bool {
    if data.len() < 8 {
        return false;
    }
    
    // Check for reasonable magic numbers
    let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    if magic == 0 || magic == 0xFFFFFFFF {
        return false;
    }
    
    // Check for reasonable version numbers
    let version = data[4];
    if version > 100 {
        return false;
    }
    
    // Check for reasonable flags
    let flags = data[8];
    if flags > 0x0F {
        return false;
    }
    
    true
}

/// Helper function to safely test capability parsing
fn safe_capability_test<F>(f: F) 
where 
    F: FnOnce() -> Result<(), Box<dyn std::error::Error>>
{
    // This function should never panic or crash
    let _ = std::panic::catch_unwind(|| {
        if let Err(_) = f() {
            // Errors are expected and should not crash
        }
    });
}
