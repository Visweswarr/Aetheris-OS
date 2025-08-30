#![no_main]

use libfuzzer_sys::fuzz_target;
use uuid::{Uuid, Version, Variant};

fuzz_target!(|data: &[u8]| {
    // Test UUID generation
    let uuid = Uuid::new_v4();
    
    // Test UUID parsing from string
    let uuid_str = uuid.to_string();
    let _ = Uuid::parse_str(&uuid_str);
    
    // Test with malformed UUID strings
    if data.len() > 0 {
        // Test with truncated data
        if data.len() > 1 {
            let truncated = &data[..data.len() - 1];
            if let Ok(truncated_str) = std::str::from_utf8(truncated) {
                let _ = Uuid::parse_str(truncated_str);
            }
        }
        
        // Test with corrupted data
        let mut corrupted = data.to_vec();
        if corrupted.len() > 10 {
            // Flip some bits
            for i in 0..10 {
                corrupted[i] = corrupted[i].wrapping_add(1);
            }
            if let Ok(corrupted_str) = std::str::from_utf8(&corrupted) {
                let _ = Uuid::parse_str(corrupted_str);
            }
        }
        
        // Test with invalid characters
        let mut invalid_chars = data.to_vec();
        for byte in &mut invalid_chars {
            if *byte == b'-' {
                *byte = b'*'; // Invalid UUID character
            } else if *byte == b'f' {
                *byte = b'g'; // Invalid hex character
            }
        }
        if let Ok(invalid_str) = std::str::from_utf8(&invalid_chars) {
            let _ = Uuid::parse_str(invalid_str);
        }
    }
    
    // Test with various data sizes
    if data.len() == 0 {
        // This should handle empty data gracefully
    }
    
    if data.len() == 1 {
        if let Ok(single_char) = std::str::from_utf8(data) {
            let _ = Uuid::parse_str(single_char);
        }
    }
    
    if data.len() > 36 {
        // Standard UUID length is 36 characters
        let long_uuid = std::str::from_utf8(data).unwrap_or("invalid");
        let _ = Uuid::parse_str(long_uuid);
    }
    
    // Test with malformed UUID format
    if data.len() > 0 {
        // Test with missing hyphens
        let no_hyphens = std::str::from_utf8(data)
            .unwrap_or("invalid")
            .replace("-", "");
        let _ = Uuid::parse_str(&no_hyphens);
        
        // Test with extra hyphens
        let extra_hyphens = std::str::from_utf8(data)
            .unwrap_or("invalid")
            .replace("-", "--");
        let _ = Uuid::parse_str(&extra_hyphens);
        
        // Test with wrong hyphen positions
        let wrong_hyphens = format!(
            "{}-{}-{}-{}-{}",
            std::str::from_utf8(&data[..std::cmp::min(8, data.len())]).unwrap_or(""),
            std::str::from_utf8(&data[std::cmp::min(8, data.len())..std::cmp::min(12, data.len())]).unwrap_or(""),
            std::str::from_utf8(&data[std::cmp::min(12, data.len())..std::cmp::min(16, data.len())]).unwrap_or(""),
            std::str::from_utf8(&data[std::cmp::min(16, data.len())..std::cmp::min(20, data.len())]).unwrap_or(""),
            std::str::from_utf8(&data[std::cmp::min(20, data.len())..std::cmp::min(32, data.len())]).unwrap_or("")
        );
        let _ = Uuid::parse_str(&wrong_hyphens);
    }
    
    // Test with non-hex characters
    if data.len() > 0 {
        let non_hex = std::str::from_utf8(data)
            .unwrap_or("invalid")
            .chars()
            .map(|c| if c.is_ascii_hexdigit() { c } else { 'x' })
            .collect::<String>();
        let _ = Uuid::parse_str(&non_hex);
    }
    
    // Test with mixed case
    if data.len() > 0 {
        let mixed_case = std::str::from_utf8(data)
            .unwrap_or("invalid")
            .chars()
            .map(|c| if c.is_ascii_uppercase() { c.to_ascii_lowercase() } else { c.to_ascii_uppercase() })
            .collect::<String>();
        let _ = Uuid::parse_str(&mixed_case);
    }
    
    // Test with whitespace
    if data.len() > 0 {
        let with_whitespace = std::str::from_utf8(data)
            .unwrap_or("invalid")
            .chars()
            .enumerate()
            .map(|(i, c)| if i % 5 == 0 { format!(" {} ", c) } else { c.to_string() })
            .collect::<String>();
        let _ = Uuid::parse_str(&with_whitespace);
    }
    
    // Test UUID properties
    if let Ok(parsed_uuid) = Uuid::parse_str(&uuid_str) {
        let _ = parsed_uuid.get_version();
        let _ = parsed_uuid.get_variant();
        let _ = parsed_uuid.as_bytes();
        let _ = parsed_uuid.to_hyphenated().to_string();
        let _ = parsed_uuid.to_simple().to_string();
        let _ = parsed_uuid.to_urn().to_string();
    }
    
    // Test with known UUID variants
    let nil_uuid = Uuid::nil();
    let _ = nil_uuid.get_version();
    let _ = nil_uuid.get_variant();
    
    let max_uuid = Uuid::max();
    let _ = max_uuid.get_version();
    let _ = max_uuid.get_variant();
});
