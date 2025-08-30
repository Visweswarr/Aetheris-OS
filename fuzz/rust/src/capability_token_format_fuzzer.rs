#![no_main]

use libfuzzer_sys::fuzz_target;
use polymera_caps::format::{CapabilityToken, CapabilityTokenFormatter, CapabilityTokenError};
use serde_json::Value;

fuzz_target!(|data: &[u8]| {
    // Try to parse as JSON first
    if let Ok(json_str) = std::str::from_utf8(data) {
        // Test JSON parsing
        if let Ok(json_value) = serde_json::from_str::<Value>(json_str) {
            // Test if it can be parsed as a capability token
            let _ = CapabilityTokenFormatter::from_json(json_str);
        }
    }

    // Test base64 decoding
    if let Ok(base64_str) = std::str::from_utf8(data) {
        let _ = CapabilityTokenFormatter::from_base64(base64_str);
    }

    // Test token size validation with various limits
    if let Ok(json_str) = std::str::from_utf8(data) {
        if let Ok(token) = CapabilityTokenFormatter::from_json(json_str) {
            // Test size validation with different limits
            let _ = CapabilityTokenFormatter::validate_token_size(&token, 1024);
            let _ = CapabilityTokenFormatter::validate_token_size(&token, 64 * 1024);
            let _ = CapabilityTokenFormatter::validate_token_size(&token, 1024 * 1024);
            
            // Test formatting functions
            let _ = CapabilityTokenFormatter::to_json(&token);
            let _ = CapabilityTokenFormatter::to_json_compact(&token);
            let _ = CapabilityTokenFormatter::to_base64(&token);
            let _ = CapabilityTokenFormatter::get_token_size(&token);
        }
    }

    // Test with malformed data
    if data.len() > 0 {
        // Test with truncated data
        let truncated = &data[..data.len() / 2];
        if let Ok(json_str) = std::str::from_utf8(truncated) {
            let _ = CapabilityTokenFormatter::from_json(json_str);
        }

        // Test with corrupted data
        let mut corrupted = data.to_vec();
        if corrupted.len() > 10 {
            // Flip some bits
            for i in 0..10 {
                corrupted[i] = corrupted[i].wrapping_add(1);
            }
            if let Ok(json_str) = std::str::from_utf8(&corrupted) {
                let _ = CapabilityTokenFormatter::from_json(json_str);
            }
        }
    }
});
