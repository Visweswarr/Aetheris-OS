use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr;

use crate::liboqs::bindings::*;
use crate::liboqs::error::LibOqsError;
use crate::pqc::mem::{SecureMemory, get_memory_config};

/// Dilithium parameter sets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DilithiumParameterSet {
    /// Dilithium2 - 128-bit security level
    Dilithium2 = 0,
    /// Dilithium3 - 192-bit security level
    Dilithium3 = 1,
    /// Dilithium5 - 256-bit security level
    Dilithium5 = 2,
}

impl DilithiumParameterSet {
    /// Get the security level in bits
    pub fn security_level(&self) -> u32 {
        match self {
            DilithiumParameterSet::Dilithium2 => 128,
            DilithiumParameterSet::Dilithium3 => 192,
            DilithiumParameterSet::Dilithium5 => 256,
        }
    }

    /// Get the algorithm name
    pub fn algorithm_name(&self) -> &'static str {
        match self {
            DilithiumParameterSet::Dilithium2 => "Dilithium2",
            DilithiumParameterSet::Dilithium3 => "Dilithium3",
            DilithiumParameterSet::Dilithium5 => "Dilithium5",
        }
    }

    /// Get public key length in bytes
    pub fn public_key_length(&self) -> usize {
        unsafe {
            match self {
                DilithiumParameterSet::Dilithium2 => dilithium_get_public_key_length(DILITHIUM_2),
                DilithiumParameterSet::Dilithium3 => dilithium_get_public_key_length(DILITHIUM_3),
                DilithiumParameterSet::Dilithium5 => dilithium_get_public_key_length(DILITHIUM_5),
            }
        }
    }

    /// Get secret key length in bytes
    pub fn secret_key_length(&self) -> usize {
        unsafe {
            match self {
                DilithiumParameterSet::Dilithium2 => dilithium_get_secret_key_length(DILITHIUM_2),
                DilithiumParameterSet::Dilithium3 => dilithium_get_secret_key_length(DILITHIUM_3),
                DilithiumParameterSet::Dilithium5 => dilithium_get_secret_key_length(DILITHIUM_5),
            }
        }
    }

    /// Get signature length in bytes
    pub fn signature_length(&self) -> usize {
        unsafe {
            match self {
                DilithiumParameterSet::Dilithium2 => dilithium_get_signature_length(DILITHIUM_2),
                DilithiumParameterSet::Dilithium3 => dilithium_get_signature_length(DILITHIUM_3),
                DilithiumParameterSet::Dilithium5 => dilithium_get_signature_length(DILITHIUM_5),
            }
        }
    }
}

impl From<DilithiumParameterSet> for dilithium_parameter_set_t {
    fn from(params: DilithiumParameterSet) -> Self {
        match params {
            DilithiumParameterSet::Dilithium2 => DILITHIUM_2,
            DilithiumParameterSet::Dilithium3 => DILITHIUM_3,
            DilithiumParameterSet::Dilithium5 => DILITHIUM_5,
        }
    }
}

/// Dilithium public key with secure memory management
pub struct DilithiumPublicKey {
    inner: SecureMemory<dilithium_public_key_t>,
    params: DilithiumParameterSet,
    _phantom: PhantomData<*mut c_void>,
}

impl DilithiumPublicKey {
    /// Create a new public key from raw bytes
    pub fn new(params: DilithiumParameterSet, key_data: Vec<u8>) -> Result<Self, LibOqsError> {
        let expected_len = params.public_key_length();
        if key_data.len() != expected_len {
            return Err(LibOqsError::InvalidParameter(format!(
                "Invalid public key length: expected {}, got {}",
                expected_len,
                key_data.len()
            )));
        }

        unsafe {
            let mut key = ptr::null_mut();
            let result = dilithium_public_key_new(params.into(), &mut key);
            
            if result != 0 {
                return Err(LibOqsError::from(result));
            }

            if key.is_null() {
                return Err(LibOqsError::NullPointer("Failed to create public key"));
            }

            // Import key data
            let import_result = dilithium_public_key_import(key, key_data.as_ptr(), key_data.len());
            if import_result != 0 {
                dilithium_public_key_free(key);
                return Err(LibOqsError::from(import_result));
            }

            Ok(Self {
                inner: SecureMemory::new(key),
                params,
                _phantom: PhantomData,
            })
        }
    }

    /// Get the parameter set
    pub fn parameter_set(&self) -> DilithiumParameterSet {
        self.params
    }

    /// Export public key to bytes
    pub fn export(&self) -> Result<Vec<u8>, LibOqsError> {
        unsafe {
            let key = *self.inner.as_ref();
            let mut len = 0;
            let result = dilithium_public_key_export_len(key, &mut len);
            
            if result != 0 {
                return Err(LibOqsError::from(result));
            }

            let mut buffer = vec![0u8; len];
            let export_result = dilithium_public_key_export(key, buffer.as_mut_ptr(), len);
            
            if export_result != 0 {
                return Err(LibOqsError::from(export_result));
            }

            Ok(buffer)
        }
    }

    /// Get the underlying key pointer (for FFI calls)
    pub fn as_ptr(&self) -> dilithium_public_key_t {
        *self.inner.as_ref()
    }
}

impl Drop for DilithiumPublicKey {
    fn drop(&mut self) {
        unsafe {
            let key = *self.inner.as_ref();
            if !key.is_null() {
                dilithium_public_key_free(key);
            }
        }
    }
}

/// Dilithium secret key with secure memory management and zeroization
pub struct DilithiumSecretKey {
    inner: SecureMemory<dilithium_secret_key_t>,
    params: DilithiumParameterSet,
    _phantom: PhantomData<*mut c_void>,
}

impl DilithiumSecretKey {
    /// Create a new secret key from raw bytes
    pub fn new(params: DilithiumParameterSet, key_data: Vec<u8>) -> Result<Self, LibOqsError> {
        let expected_len = params.secret_key_length();
        if key_data.len() != expected_len {
            return Err(LibOqsError::InvalidParameter(format!(
                "Invalid secret key length: expected {}, got {}",
                expected_len,
                key_data.len()
            )));
        }

        unsafe {
            let mut key = ptr::null_mut();
            let result = dilithium_secret_key_new(params.into(), &mut key);
            
            if result != 0 {
                return Err(LibOqsError::from(result));
            }

            if key.is_null() {
                return Err(LibOqsError::NullPointer("Failed to create secret key"));
            }

            // Import key data
            let import_result = dilithium_secret_key_import(key, key_data.as_ptr(), key_data.len());
            if import_result != 0 {
                dilithium_secret_key_free(key);
                return Err(LibOqsError::from(import_result));
            }

            Ok(Self {
                inner: SecureMemory::new(key),
                params,
                _phantom: PhantomData,
            })
        }
    }

    /// Get the parameter set
    pub fn parameter_set(&self) -> DilithiumParameterSet {
        self.params
    }

    /// Export secret key to bytes
    pub fn export(&self) -> Result<Vec<u8>, LibOqsError> {
        unsafe {
            let key = *self.inner.as_ref();
            let mut len = 0;
            let result = dilithium_secret_key_export_len(key, &mut len);
            
            if result != 0 {
                return Err(LibOqsError::from(result));
            }

            let mut buffer = vec![0u8; len];
            let export_result = dilithium_secret_key_export(key, buffer.as_mut_ptr(), len);
            
            if export_result != 0 {
                return Err(LibOqsError::from(export_result));
            }

            Ok(buffer)
        }
    }

    /// Get the underlying key pointer (for FFI calls)
    pub fn as_ptr(&self) -> dilithium_secret_key_t {
        *self.inner.as_ref()
    }
}

impl Drop for DilithiumSecretKey {
    fn drop(&mut self) {
        unsafe {
            let key = *self.inner.as_ref();
            if !key.is_null() {
                // Zeroize key data before freeing
                if get_memory_config().zeroize_on_drop {
                    let key_ptr = key as *mut u8;
                    let key_size = self.params.secret_key_length();
                    for i in 0..key_size {
                        ptr::write_volatile(key_ptr.add(i), 0u8);
                    }
                    std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
                }
                
                dilithium_secret_key_free(key);
            }
        }
    }
}

/// Dilithium signature with secure memory management
pub struct DilithiumSignature {
    inner: SecureMemory<Vec<u8>>,
    params: DilithiumParameterSet,
}

impl DilithiumSignature {
    /// Create a new signature from raw bytes
    pub fn new(params: DilithiumParameterSet, data: Vec<u8>) -> Result<Self, LibOqsError> {
        let expected_len = params.signature_length();
        if data.len() != expected_len {
            return Err(LibOqsError::InvalidParameter(format!(
                "Invalid signature length: expected {}, got {}",
                expected_len,
                data.len()
            )));
        }

        Ok(Self {
            inner: SecureMemory::new(data),
            params,
        })
    }

    /// Get the parameter set
    pub fn parameter_set(&self) -> DilithiumParameterSet {
        self.params
    }

    /// Get the signature bytes
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_ref()
    }

    /// Consume and return the signature bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.inner.into_inner()
    }
}

impl Drop for DilithiumSignature {
    fn drop(&mut self) {
        // SecureMemory will handle zeroization
    }
}

/// Main Dilithium signature implementation
pub struct Dilithium;

impl Dilithium {
    /// Generate a new Dilithium keypair
    pub fn keygen(params: DilithiumParameterSet) -> Result<(DilithiumPublicKey, DilithiumSecretKey), LibOqsError> {
        unsafe {
            let mut pk = ptr::null_mut();
            let mut sk = ptr::null_mut();
            
            let result = dilithium_keygen(params.into(), &mut pk, &mut sk);
            
            if result != 0 {
                return Err(LibOqsError::from(result));
            }

            if pk.is_null() || sk.is_null() {
                if !pk.is_null() {
                    dilithium_public_key_free(pk);
                }
                if !sk.is_null() {
                    dilithium_secret_key_free(sk);
                }
                return Err(LibOqsError::NullPointer("Key generation failed"));
            }

            let public_key = DilithiumPublicKey {
                inner: SecureMemory::new(pk),
                params,
                _phantom: PhantomData,
            };

            let secret_key = DilithiumSecretKey {
                inner: SecureMemory::new(sk),
                params,
                _phantom: PhantomData,
            };

            Ok((public_key, secret_key))
        }
    }

    /// Sign a message using a secret key
    pub fn sign(message: &[u8], sk: &DilithiumSecretKey) -> Result<DilithiumSignature, LibOqsError> {
        unsafe {
            let mut sig = ptr::null_mut();
            let mut sig_len = 0;
            
            let result = dilithium_sign(
                sk.as_ptr(),
                message.as_ptr(),
                message.len(),
                &mut sig,
                &mut sig_len
            );
            
            if result != 0 {
                return Err(LibOqsError::from(result));
            }

            if sig.is_null() {
                return Err(LibOqsError::NullPointer("Signing failed"));
            }

            let signature = DilithiumSignature {
                inner: SecureMemory::new({
                    let mut buffer = vec![0u8; sig_len];
                    let export_result = dilithium_signature_export(sig, buffer.as_mut_ptr(), sig_len);
                    if export_result != 0 {
                        dilithium_signature_free(sig);
                        return Err(LibOqsError::from(export_result));
                    }
                    buffer
                }),
                params: sk.parameter_set(),
            };

            // Free the liboqs signature object
            dilithium_signature_free(sig);

            Ok(signature)
        }
    }

    /// Verify a signature using a public key
    pub fn verify(message: &[u8], signature: &DilithiumSignature, pk: &DilithiumPublicKey) -> Result<bool, LibOqsError> {
        if signature.parameter_set() != pk.parameter_set() {
            return Err(LibOqsError::InvalidParameter("Parameter set mismatch"));
        }

        unsafe {
            let mut sig = ptr::null_mut();
            let result = dilithium_signature_new(pk.parameter_set().into(), &mut sig);
            
            if result != 0 {
                return Err(LibOqsError::from(result));
            }

            if sig.is_null() {
                return Err(LibOqsError::NullPointer("Failed to create signature object"));
            }

            // Import signature data
            let import_result = dilithium_signature_import(sig, signature.as_bytes().as_ptr(), signature.as_bytes().len());
            if import_result != 0 {
                dilithium_signature_free(sig);
                return Err(LibOqsError::from(import_result));
            }

            // Verify signature
            let verify_result = dilithium_verify(
                sig,
                message.as_ptr(),
                message.len(),
                pk.as_ptr()
            );

            // Free the signature object
            dilithium_signature_free(sig);

            if verify_result != 0 {
                return Err(LibOqsError::from(verify_result));
            }

            Ok(true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dilithium_roundtrip() {
        let params = DilithiumParameterSet::Dilithium2;
        let message = b"Hello, Post-Quantum World!";
        
        // Generate keypair
        let (pk, sk) = Dilithium::keygen(params).unwrap();
        
        // Sign message
        let signature = Dilithium::sign(message, &sk).unwrap();
        
        // Verify signature
        let is_valid = Dilithium::verify(message, &signature, &pk).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_dilithium_parameter_sets() {
        let test_cases = [
            DilithiumParameterSet::Dilithium2,
            DilithiumParameterSet::Dilithium3,
            DilithiumParameterSet::Dilithium5,
        ];

        let message = b"Test message for parameter sets";

        for params in test_cases {
            let (pk, sk) = Dilithium::keygen(params).unwrap();
            
            // Test key lengths
            assert_eq!(pk.export().unwrap().len(), params.public_key_length());
            assert_eq!(sk.export().unwrap().len(), params.secret_key_length());
            
            // Test signing and verification
            let signature = Dilithium::sign(message, &sk).unwrap();
            assert_eq!(signature.as_bytes().len(), params.signature_length());
            
            let is_valid = Dilithium::verify(message, &signature, &pk).unwrap();
            assert!(is_valid);
        }
    }

    #[test]
    fn test_dilithium_zeroization() {
        let params = DilithiumParameterSet::Dilithium2;
        let (pk, sk) = Dilithium::keygen(params).unwrap();
        
        // Get secret key bytes
        let sk_bytes = sk.export().unwrap();
        let sk_ptr = sk_bytes.as_ptr() as usize;
        let sk_size = sk_bytes.len();
        
        // Drop secret key (should zeroize)
        drop(sk);
        
        // Verify memory is zeroized (this is unsafe and test-only)
        unsafe {
            let memory_region = std::slice::from_raw_parts(sk_ptr as *const u8, sk_size);
            assert!(memory_region.iter().all(|&b| b == 0));
        }
    }

    #[test]
    fn test_dilithium_performance() {
        let params = DilithiumParameterSet::Dilithium2;
        let message = b"Performance test message";
        let iterations = 100;
        
        let start = std::time::Instant::now();
        
        for _ in 0..iterations {
            let (pk, sk) = Dilithium::keygen(params).unwrap();
            let signature = Dilithium::sign(message, &sk).unwrap();
            let _ = Dilithium::verify(message, &signature, &pk).unwrap();
        }
        
        let duration = start.elapsed();
        let avg_time = duration / iterations;
        
        // Verify performance targets (relaxed for testing)
        assert!(avg_time.as_millis() < 100); // < 100ms average for testing
    }

    #[test]
    fn test_dilithium_invalid_signature() {
        let params = DilithiumParameterSet::Dilithium2;
        let message = b"Test message";
        let (pk, sk) = Dilithium::keygen(params).unwrap();
        
        // Create signature
        let signature = Dilithium::sign(message, &sk).unwrap();
        
        // Verify with wrong message
        let wrong_message = b"Wrong message";
        let is_valid = Dilithium::verify(wrong_message, &signature, &pk).unwrap();
        assert!(!is_valid);
        
        // Verify with wrong public key
        let (wrong_pk, _) = Dilithium::keygen(params).unwrap();
        let is_valid = Dilithium::verify(message, &signature, &wrong_pk).unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_dilithium_parameter_mismatch() {
        let params1 = DilithiumParameterSet::Dilithium2;
        let params2 = DilithiumParameterSet::Dilithium3;
        
        let (pk1, sk1) = Dilithium::keygen(params1).unwrap();
        let (pk2, sk2) = Dilithium::keygen(params2).unwrap();
        
        let message = b"Test message";
        let signature = Dilithium::sign(message, &sk1).unwrap();
        
        // Try to verify with mismatched parameters
        let result = Dilithium::verify(message, &signature, &pk2);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LibOqsError::InvalidParameter(_)));
    }
}
