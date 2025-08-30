#![no_main]

use libfuzzer_sys::fuzz_target;
use base64::{Engine, engine::general_purpose};

fuzz_target!(|data: &[u8]| {
    // Test base64 encoding
    let encoded = general_purpose::STANDARD.encode(data);
    
    // Test base64 decoding
    let _ = general_purpose::STANDARD.decode(&encoded);
    
    // Test with malformed base64
    if data.len() > 0 {
        // Test with truncated data
        if data.len() > 1 {
            let truncated = &data[..data.len() - 1];
            let _ = general_purpose::STANDARD.decode(truncated);
        }
        
        // Test with corrupted data
        let mut corrupted = data.to_vec();
        if corrupted.len() > 10 {
            // Flip some bits
            for i in 0..10 {
                corrupted[i] = corrupted[i].wrapping_add(1);
            }
            let _ = general_purpose::STANDARD.decode(&corrupted);
        }
        
        // Test with invalid characters
        let mut invalid_chars = data.to_vec();
        for byte in &mut invalid_chars {
            if *byte == b'+' {
                *byte = b'*'; // Invalid base64 character
            } else if *byte == b'/' {
                *byte = b'@'; // Invalid base64 character
            }
        }
        let _ = general_purpose::STANDARD.decode(&invalid_chars);
    }
    
    // Test with various data sizes
    if data.len() == 0 {
        let _ = general_purpose::STANDARD.decode(data);
    }
    
    if data.len() == 1 {
        let _ = general_purpose::STANDARD.decode(data);
    }
    
    if data.len() > 1024 {
        let _ = general_purpose::STANDARD.decode(data);
    }
    
    // Test with padding variations
    if data.len() > 0 {
        let encoded = general_purpose::STANDARD.encode(data);
        
        // Test with missing padding
        let mut no_padding = encoded.clone();
        while no_padding.ends_with('=') {
            no_padding.pop();
        }
        let _ = general_purpose::STANDARD.decode(&no_padding);
        
        // Test with extra padding
        let extra_padding = format!("{}===", encoded);
        let _ = general_purpose::STANDARD.decode(&extra_padding);
    }
    
    // Test with mixed case
    if data.len() > 0 {
        let encoded = general_purpose::STANDARD.encode(data);
        let mixed_case = encoded.chars()
            .map(|c| if c.is_ascii_uppercase() { c.to_ascii_lowercase() } else { c })
            .collect::<String>();
        let _ = general_purpose::STANDARD.decode(&mixed_case);
    }
    
    // Test with whitespace
    if data.len() > 0 {
        let encoded = general_purpose::STANDARD.encode(data);
        let with_whitespace = encoded.chars()
            .enumerate()
            .map(|(i, c)| if i % 3 == 0 { format!(" {} ", c) } else { c.to_string() })
            .collect::<String>();
        let _ = general_purpose::STANDARD.decode(&with_whitespace);
    }
    
    // Test with URL-safe encoding
    if data.len() > 0 {
        let url_encoded = general_purpose::URL_SAFE.encode(data);
        let _ = general_purpose::URL_SAFE.decode(&url_encoded);
        
        // Test URL-safe with standard decoder
        let _ = general_purpose::STANDARD.decode(&url_encoded);
    }
    
    // Test with no-padding encoding
    if data.len() > 0 {
        let no_padding_encoded = general_purpose::STANDARD_NO_PAD.encode(data);
        let _ = general_purpose::STANDARD_NO_PAD.decode(&no_padding_encoded);
        
        // Test no-padding with standard decoder
        let _ = general_purpose::STANDARD.decode(&no_padding_encoded);
    }
});
