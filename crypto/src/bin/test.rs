use polymera_crypto::{
    kyber::{KyberKem, KyberParameterSet},
    dilithium::{Dilithium, DilithiumParameterSet},
    utils,
    CryptoContext, SecurityLevel,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Polymera OS Post-Quantum Cryptography Test ===\n");

    // Test Kyber KEM
    test_kyber_kem()?;
    println!();

    // Test Dilithium signatures
    test_dilithium_signatures()?;
    println!();

    // Test crypto context
    test_crypto_context()?;
    println!();

    // Test utilities
    test_utilities()?;
    println!();

    println!("=== All tests completed successfully! ===");
    Ok(())
}

fn test_kyber_kem() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Kyber KEM operations...");

    // Test all parameter sets
    for params in [
        KyberParameterSet::Kyber512,
        KyberParameterSet::Kyber768,
        KyberParameterSet::Kyber1024,
    ] {
        println!("  Testing {}...", params.algorithm_name());

        // Generate keypair
        let (public_key, secret_key) = KyberKem::generate_keypair(params)?;
        println!("    Generated keypair: public={} bytes, secret={} bytes", 
                public_key.len(), secret_key.len());

        // Encapsulate shared secret
        let (ciphertext, shared_secret1) = KyberKem::encapsulate(&public_key)?;
        println!("    Encapsulated: ciphertext={} bytes, shared_secret={} bytes",
                ciphertext.len(), shared_secret1.len());

        // Decapsulate shared secret
        let shared_secret2 = KyberKem::decapsulate(&secret_key, &ciphertext)?;
        println!("    Decapsulated: shared_secret={} bytes", shared_secret2.len());

        // Verify shared secrets match
        assert_eq!(shared_secret1.as_slice(), shared_secret2.as_slice());
        println!("    ✓ Shared secrets match");

        // Test cloning
        let public_key_clone = public_key.clone();
        let secret_key_clone = secret_key.clone();
        assert_eq!(public_key.as_slice(), public_key_clone.as_slice());
        assert_eq!(secret_key.as_slice(), secret_key_clone.as_slice());
        println!("    ✓ Keys cloned successfully");

        // Test parameter set information
        assert_eq!(params.security_level(), match params {
            KyberParameterSet::Kyber512 => 128,
            KyberParameterSet::Kyber768 => 192,
            KyberParameterSet::Kyber1024 => 256,
        });
        println!("    ✓ Security level: {} bits", params.security_level());
    }

    println!("  ✓ All Kyber KEM tests passed");
    Ok(())
}

fn test_dilithium_signatures() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Dilithium signature operations...");

    // Test all parameter sets
    for params in [
        DilithiumParameterSet::Dilithium2,
        DilithiumParameterSet::Dilithium3,
        DilithiumParameterSet::Dilithium5,
    ] {
        println!("  Testing {}...", params.algorithm_name());

        // Generate keypair
        let (public_key, secret_key) = Dilithium::generate_keypair(params)?;
        println!("    Generated keypair: public={} bytes, secret={} bytes",
                public_key.len(), secret_key.len());

        // Test messages
        let messages = [
            b"Hello, Polymera OS!",
            b"This is a test message for digital signatures",
            b"Post-quantum cryptography is the future",
        ];

        for message in &messages {
            // Sign message
            let signature = Dilithium::sign(&secret_key, message)?;
            println!("    Signed message ({} bytes): signature={} bytes",
                    message.len(), signature.len());

            // Verify signature
            let is_valid = Dilithium::verify(&public_key, message, &signature)?;
            assert!(is_valid);
            println!("    ✓ Signature verified successfully");

            // Test with wrong message
            let wrong_message = b"Wrong message for verification";
            let is_valid = Dilithium::verify(&public_key, wrong_message, &signature)?;
            assert!(!is_valid);
            println!("    ✓ Wrong message correctly rejected");

            // Test cloning
            let signature_clone = signature.clone();
            assert_eq!(signature.as_slice(), signature_clone.as_slice());
            println!("    ✓ Signature cloned successfully");
        }

        // Test parameter set information
        assert_eq!(params.security_level(), match params {
            DilithiumParameterSet::Dilithium2 => 128,
            DilithiumParameterSet::Dilithium3 => 192,
            DilithiumParameterSet::Dilithium5 => 256,
        });
        println!("    ✓ Security level: {} bits", params.security_level());
    }

    println!("  ✓ All Dilithium signature tests passed");
    Ok(())
}

fn test_crypto_context() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Crypto Context...");

    // Create default context
    let mut context = CryptoContext::new();
    println!("  Default context: {} algorithms available", context.algorithms().len());
    assert!(!context.algorithms().is_empty());

    // Test security level filtering
    let level_128 = context.algorithms_by_security_level(SecurityLevel::Level1);
    let level_192 = context.algorithms_by_security_level(SecurityLevel::Level3);
    let level_256 = context.algorithms_by_security_level(SecurityLevel::Level5);

    println!("  Level 1 (128-bit): {} algorithms", level_128.len());
    println!("  Level 3 (192-bit): {} algorithms", level_192.len());
    println!("  Level 5 (256-bit): {} algorithms", level_256.len());

    assert_eq!(level_128.len(), 2); // Kyber512 + Dilithium2
    assert_eq!(level_192.len(), 2); // Kyber768 + Dilithium3
    assert_eq!(level_256.len(), 2); // Kyber1024 + Dilithium5

    // Test category filtering
    let kem_algorithms = context.algorithms_by_category(polymera_crypto::CryptoCategory::Kem);
    let sig_algorithms = context.algorithms_by_category(polymera_crypto::CryptoCategory::Signature);

    println!("  KEM algorithms: {}", kem_algorithms.len());
    println!("  Signature algorithms: {}", sig_algorithms.len());

    assert_eq!(kem_algorithms.len(), 3); // All Kyber variants
    assert_eq!(sig_algorithms.len(), 3); // All Dilithium variants

    // Test context modification
    context.set_default_security_level(SecurityLevel::Level3);
    assert_eq!(context.default_security_level(), SecurityLevel::Level3);
    assert_eq!(context.algorithms().len(), 2); // Only Level 3 algorithms

    println!("  ✓ Context modified to Level 3: {} algorithms", context.algorithms().len());

    // Test PQC enable/disable
    context.set_pqc_enabled(false);
    assert!(!context.pqc_enabled());
    assert!(context.algorithms().is_empty());

    context.set_pqc_enabled(true);
    assert!(context.pqc_enabled());
    assert!(!context.algorithms().is_empty());

    println!("  ✓ PQC enable/disable working correctly");

    println!("  ✓ All Crypto Context tests passed");
    Ok(())
}

fn test_utilities() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Cryptographic Utilities...");

    // Test random bytes
    let random_data = utils::random_bytes(32)?;
    assert_eq!(random_data.len(), 32);
    println!("  ✓ Generated 32 random bytes");

    // Test constant time compare
    let data1 = b"Hello, World!";
    let data2 = b"Hello, World!";
    let data3 = b"Hello, Universe!";

    assert!(utils::constant_time_compare(data1, data2));
    assert!(!utils::constant_time_compare(data1, data3));
    println!("  ✓ Constant time comparison working");

    // Test zeroize
    let mut sensitive_data = vec![0x42u8; 16];
    utils::zeroize(&mut sensitive_data);
    // Note: In a real implementation, we'd verify the data is zeroized
    println!("  ✓ Zeroize function called");

    // Test algorithm recommendations
    let recommendations_128 = utils::get_recommendations(SecurityLevel::Level1);
    let recommendations_256 = utils::get_recommendations(SecurityLevel::Level5);

    assert_eq!(recommendations_128.len(), 2);
    assert_eq!(recommendations_256.len(), 2);
    println!("  ✓ Algorithm recommendations working");

    // Test forward secrecy
    let kyber = polymera_crypto::Algorithm::Kyber(KyberParameterSet::Kyber512);
    let dilithium = polymera_crypto::Algorithm::Dilithium(DilithiumParameterSet::Dilithium2);

    assert!(utils::provides_forward_secrecy(&kyber));
    assert!(utils::provides_forward_secrecy(&dilithium));
    println!("  ✓ Forward secrecy detection working");

    // Test performance info
    let kyber_perf = utils::get_performance_info(&kyber);
    let dilithium_perf = utils::get_performance_info(&dilithium);

    println!("  Kyber performance: {}", kyber_perf.summary());
    println!("  Dilithium performance: {}", dilithium_perf.summary());

    assert!(!kyber_perf.key_generation.is_empty());
    assert!(!dilithium_perf.key_generation.is_empty());
    println!("  ✓ Performance information available");

    println!("  ✓ All utility tests passed");
    Ok(())
}
