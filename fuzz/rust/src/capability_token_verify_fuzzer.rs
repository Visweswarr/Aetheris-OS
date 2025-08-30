#![no_main]

use libfuzzer_sys::fuzz_target;
use polymera_caps::verify::{TokenValidator, CapabilityTokenVerifyConfig, MockTokenValidator};
use polymera_caps::format::{CapabilityToken, CapabilityTokenBuilder, CapabilityTokenFormatter};
use polymera_caps::sign::{SignatureAlgorithm, MockSigner, Signature};
use serde_json::json;
use std::collections::HashMap;

fuzz_target!(|data: &[u8]| {
    // Test with various data sizes
    if data.len() > 0 {
        // Create a mock validator
        let config = CapabilityTokenVerifyConfig::default();
        let mut validator = TokenValidator::new(config);
        let mock_validator = MockTokenValidator::new();

        // Test with valid tokens
        if let Ok(token) = create_test_token() {
            let _ = mock_validator.validate_token(&token);
        }

        // Test with corrupted tokens
        if data.len() > 100 {
            if let Ok(mut token) = create_test_token() {
                // Corrupt the token purpose
                token.payload.purpose = std::str::from_utf8(data).unwrap_or("corrupted").to_string();
                let _ = mock_validator.validate_token(&token);
            }
        }

        // Test with malformed tokens
        if data.len() > 50 {
            if let Ok(mut token) = create_test_token() {
                // Corrupt the token type
                token.header.typ = std::str::from_utf8(data).unwrap_or("corrupted").to_string();
                let _ = mock_validator.validate_token(&token);
            }
        }

        // Test with expired tokens
        if let Ok(mut token) = create_test_token() {
            token.header.exp = 0; // Expired
            let _ = mock_validator.validate_token(&token);
        }

        // Test with future tokens
        if let Ok(mut token) = create_test_token() {
            token.header.nbf = std::u64::MAX; // Far in the future
            let _ = mock_validator.validate_token(&token);
        }

        // Test with empty claims
        if let Ok(mut token) = create_test_token() {
            token.payload.claims.clear();
            let _ = mock_validator.validate_token(&token);
        }

        // Test with corrupted claims
        if data.len() > 20 {
            if let Ok(mut token) = create_test_token() {
                if token.payload.claims.len() > 0 {
                    // Corrupt the first claim
                    if let Ok(corrupted_str) = std::str::from_utf8(data) {
                        token.payload.claims[0].claim_type = 
                            polymera_caps::format::CapabilityClaimType::Custom(corrupted_str.to_string());
                    }
                }
                let _ = mock_validator.validate_token(&token);
            }
        }

        // Test with various token levels
        for level in [0, 1, 10, 100, std::u32::MAX] {
            if let Ok(mut token) = create_test_token() {
                token.payload.level = level;
                let _ = mock_validator.validate_token(&token);
            }
        }

        // Test with corrupted timestamps
        if let Ok(mut token) = create_test_token() {
            token.header.iat = std::u64::MAX;
            token.header.exp = 0;
            let _ = mock_validator.validate_token(&token);
        }

        // Test with corrupted scope
        if data.len() > 10 {
            if let Ok(mut token) = create_test_token() {
                token.payload.scope = std::str::from_utf8(data).unwrap_or("corrupted").to_string();
                let _ = mock_validator.validate_token(&token);
            }
        }

        // Test with corrupted hierarchy
        if data.len() > 20 {
            if let Ok(mut token) = create_test_token() {
                let corrupted_str = std::str::from_utf8(data).unwrap_or("corrupted").to_string();
                token.payload.hierarchy = vec![corrupted_str.clone(), corrupted_str];
                let _ = mock_validator.validate_token(&token);
            }
        }
    }

    // Test with edge cases
    if data.len() == 0 {
        let mock_validator = MockTokenValidator::new();
        // This should handle empty data gracefully
    }

    if data.len() == 1 {
        let mock_validator = MockTokenValidator::new();
        // This should handle single byte data gracefully
    }
});

fn create_test_token() -> Result<CapabilityToken, Box<dyn std::error::Error>> {
    let token = CapabilityTokenBuilder::new(
        "test-issuer",
        "test-subject",
        "test-audience",
        "test-purpose",
        "test-scope",
        1,
    )
    .with_resource_capability(
        "file",
        "test-file-123",
        "/path/to/file",
        vec!["read".to_string()],
    )
    .build();

    Ok(token)
}
