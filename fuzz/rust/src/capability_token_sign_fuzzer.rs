#![no_main]

use libfuzzer_sys::fuzz_target;
use polymera_caps::sign::{Signature, SignatureAlgorithm, DilithiumSigner, MockSigner, Signer};
use polymera_caps::format::CapabilityToken;

fuzz_target!(|data: &[u8]| {
    // Test with various data sizes
    if data.len() > 0 {
        // Test Dilithium signer with different algorithms
        for algorithm in &[
            SignatureAlgorithm::Dilithium2,
            SignatureAlgorithm::Dilithium3,
            SignatureAlgorithm::Dilithium5,
        ] {
            if let Ok(signer) = DilithiumSigner::new(algorithm.clone()) {
                // Test signing
                let _ = signer.sign(data);
                
                // Test public key retrieval
                let _ = signer.get_public_key();
                let _ = signer.get_key_id();
            }
        }

        // Test MockSigner
        let mock_signer = MockSigner::new();
        let _ = mock_signer.sign(data);
        let _ = mock_signer.get_public_key();
        let _ = mock_signer.get_key_id();
    }

    // Test with edge case data sizes
    if data.len() == 0 {
        let mock_signer = MockSigner::new();
        let _ = mock_signer.sign(data);
    }

    if data.len() == 1 {
        let mock_signer = MockSigner::new();
        let _ = mock_signer.sign(data);
    }

    // Test with very large data
    if data.len() > 1024 {
        let mock_signer = MockSigner::new();
        let _ = mock_signer.sign(data);
    }

    // Test with corrupted signature data
    if data.len() > 32 {
        let mock_signer = MockSigner::new();
        if let Ok(signature) = mock_signer.sign(data) {
            // Create corrupted signature data
            let mut corrupted_data = data.to_vec();
            for i in 0..std::cmp::min(32, data.len()) {
                corrupted_data[i] = corrupted_data[i].wrapping_add(1);
            }
            let _ = mock_signer.sign(&corrupted_data);
        }
    }

    // Test signature creation with various algorithms
    if data.len() > 0 {
        let mock_signer = MockSigner::new();
        if let Ok(signature) = mock_signer.sign(data) {
            // Test signature properties
            assert!(signature.id.len() > 0);
            assert!(signature.public_key.len() > 0);
            assert!(signature.signature_data.len() > 0);
            assert!(signature.data_hash.len() > 0);
        }
    }
});
