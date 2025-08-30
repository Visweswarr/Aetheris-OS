use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr;

use crate::liboqs::bindings::*;
use crate::liboqs::error::LibOqsError;
use crate::pqc::mem::{SecureMemory, get_memory_config};

/// Kyber parameter sets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KyberParameterSet {
    /// Kyber512 - 128-bit security level
    Kyber512 = 0,
    /// Kyber768 - 192-bit security level
    Kyber768 = 1,
    /// Kyber1024 - 256-bit security level
    Kyber1024 = 2,
}

impl KyberParameterSet {
    /// Get the security level in bits
    pub fn security_level(&self) -> u32 {
        match self {
            KyberParameterSet::Kyber512 => 128,
            KyberParameterSet::Kyber768 => 192,
            KyberParameterSet::Kyber1024 => 256,
        }
    }

    /// Get the algorithm name
    pub fn algorithm_name(&self) -> &'static str {
        match self {
            KyberParameterSet::Kyber512 => "Kyber512",
            KyberParameterSet::Kyber768 => "Kyber768",
            KyberParameterSet::Kyber1024 => "Kyber1024",
        }
    }

    /// Get public key length in bytes
    pub fn public_key_length(&self) -> usize {
        unsafe {
            match self {
                KyberParameterSet::Kyber512 => kyber_get_public_key_length(KYBER_512),
                KyberParameterSet::Kyber768 => kyber_get_public_key_length(KYBER_768),
                KyberParameterSet::Kyber1024 => kyber_get_public_key_length(KYBER_1024),
            }
        }
    }

    /// Get secret key length in bytes
    pub fn secret_key_length(&self) -> usize {
        unsafe {
            match self {
                KyberParameterSet::Kyber512 => kyber_get_secret_key_length(KYBER_512),
                KyberParameterSet::Kyber768 => kyber_get_secret_key_length(KYBER_768),
                KyberParameterSet::Kyber1024 => kyber_get_secret_key_length(KYBER_1024),
            }
        }
    }

    /// Get ciphertext length in bytes
    pub fn ciphertext_length(&self) -> usize {
        unsafe {
            match self {
                KyberParameterSet::Kyber512 => kyber_get_ciphertext_length(KYBER_512),
                KyberParameterSet::Kyber768 => kyber_get_ciphertext_length(KYBER_768),
                KyberParameterSet::Kyber1024 => kyber_get_ciphertext_length(KYBER_1024),
            }
        }
    }

    /// Get shared secret length in bytes
    pub fn shared_secret_length(&self) -> usize {
        unsafe {
            match self {
                KyberParameterSet::Kyber512 => kyber_get_shared_secret_length(KYBER_512),
                KyberParameterSet::Kyber768 => kyber_get_shared_secret_length(KYBER_768),
                KyberParameterSet::Kyber1024 => kyber_get_shared_secret_length(KYBER_1024),
            }
        }
    }
}

impl From<KyberParameterSet> for kyber_parameter_set_t {
    fn from(params: KyberParameterSet) -> Self {
        match params {
            KyberParameterSet::Kyber512 => KYBER_512,
            KyberParameterSet::Kyber768 => KYBER_768,
            KyberParameterSet::Kyber1024 => KYBER_1024,
        }
    }
}

/// Kyber public key with secure memory management
pub struct KyberPublicKey {
    inner: SecureMemory<kyber_public_key_t>,
    params: KyberParameterSet,
    _phantom: PhantomData<*mut c_void>,
}

impl KyberPublicKey {
    /// Create a new public key from raw bytes
    pub fn new(params: KyberParameterSet, key_data: Vec<u8>) -> Result<Self, LibOqsError> {
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
            let result = kyber_public_key_new(params.into(), &mut key);
            
            if result != 0 {
                return Err(LibOqsError::from(result));
            }

            if key.is_null() {
                return Err(LibOqsError::NullPointer("Failed to create public key"));
            }

            // Import key data
            let import_result = kyber_public_key_import(key, key_data.as_ptr(), key_data.len());
            if import_result != 0 {
                kyber_public_key_free(key);
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
    pub fn parameter_set(&self) -> KyberParameterSet {
        self.params
    }

    /// Export public key to bytes
    pub fn export(&self) -> Result<Vec<u8>, LibOqsError> {
        unsafe {
            let key = *self.inner.as_ref();
            let mut len = 0;
            let result = kyber_public_key_export_len(key, &mut len);
            
            if result != 0 {
                return Err(LibOqsError::from(result));
            }

            let mut buffer = vec![0u8; len];
            let export_result = kyber_public_key_export(key, buffer.as_mut_ptr(), len);
            
            if export_result != 0 {
                return Err(LibOqsError::from(export_result));
            }

            Ok(buffer)
        }
    }

    /// Get the underlying key pointer (for FFI calls)
    pub fn as_ptr(&self) -> kyber_public_key_t {
        *self.inner.as_ref()
    }
}

impl Drop for KyberPublicKey {
    fn drop(&mut self) {
        unsafe {
            let key = *self.inner.as_ref();
            if !key.is_null() {
                kyber_public_key_free(key);
            }
        }
    }
}

/// Kyber secret key with secure memory management and zeroization
pub struct KyberSecretKey {
    inner: SecureMemory<kyber_secret_key_t>,
    params: KyberParameterSet,
    _phantom: PhantomData<*mut c_void>,
}

impl KyberSecretKey {
    /// Create a new secret key from raw bytes
    pub fn new(params: KyberParameterSet, key_data: Vec<u8>) -> Result<Self, LibOqsError> {
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
            let result = kyber_secret_key_new(params.into(), &mut key);
            
            if result != 0 {
                return Err(LibOqsError::from(result));
            }

            if key.is_null() {
                return Err(LibOqsError::NullPointer("Failed to create secret key"));
            }

            // Import key data
            let import_result = kyber_secret_key_import(key, key_data.as_ptr(), key_data.len());
            if import_result != 0 {
                kyber_secret_key_free(key);
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
    pub fn parameter_set(&self) -> KyberParameterSet {
        self.params
    }

    /// Export secret key to bytes
    pub fn export(&self) -> Result<Vec<u8>, LibOqsError> {
        unsafe {
            let key = *self.inner.as_ref();
            let mut len = 0;
            let result = kyber_secret_key_export_len(key, &mut len);
            
            if result != 0 {
                return Err(LibOqsError::from(result));
            }

            let mut buffer = vec![0u8; len];
            let export_result = kyber_secret_key_export(key, buffer.as_mut_ptr(), len);
            
            if export_result != 0 {
                return Err(LibOqsError::from(export_result));
            }

            Ok(buffer)
        }
    }

    /// Get the underlying key pointer (for FFI calls)
    pub fn as_ptr(&self) -> kyber_secret_key_t {
        *self.inner.as_ref()
    }
}

impl Drop for KyberSecretKey {
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
                
                kyber_secret_key_free(key);
            }
        }
    }
}

/// Kyber ciphertext with secure memory management
pub struct KyberCiphertext {
    inner: SecureMemory<kyber_ciphertext_t>,
    params: KyberParameterSet,
    _phantom: PhantomData<*mut c_void>,
}

impl KyberCiphertext {
    /// Create a new ciphertext from raw bytes
    pub fn new(params: KyberParameterSet, data: Vec<u8>) -> Result<Self, LibOqsError> {
        let expected_len = params.ciphertext_length();
        if data.len() != expected_len {
            return Err(LibOqsError::InvalidParameter(format!(
                "Invalid ciphertext length: expected {}, got {}",
                expected_len,
                data.len()
            )));
        }

        unsafe {
            let mut ct = ptr::null_mut();
            let result = kyber_ciphertext_new(params.into(), &mut ct);
            
            if result != 0 {
                return Err(LibOqsError::from(result));
            }

            if ct.is_null() {
                return Err(LibOqsError::NullPointer("Failed to create ciphertext"));
            }

            // Import ciphertext data
            let import_result = kyber_ciphertext_import(ct, data.as_ptr(), data.len());
            if import_result != 0 {
                kyber_ciphertext_free(ct);
                return Err(LibOqsError::from(import_result));
            }

            Ok(Self {
                inner: SecureMemory::new(ct),
                params,
                _phantom: PhantomData,
            })
        }
    }

    /// Get the parameter set
    pub fn parameter_set(&self) -> KyberParameterSet {
        self.params
    }

    /// Export ciphertext to bytes
    pub fn export(&self) -> Result<Vec<u8>, LibOqsError> {
        unsafe {
            let ct = *self.inner.as_ref();
            let mut len = 0;
            let result = kyber_ciphertext_export_len(ct, &mut len);
            
            if result != 0 {
                return Err(LibOqsError::from(result));
            }

            let mut buffer = vec![0u8; len];
            let export_result = kyber_ciphertext_export(ct, buffer.as_mut_ptr(), len);
            
            if export_result != 0 {
                return Err(LibOqsError::from(export_result));
            }

            Ok(buffer)
        }
    }

    /// Get the underlying ciphertext pointer (for FFI calls)
    pub fn as_ptr(&self) -> kyber_ciphertext_t {
        *self.inner.as_ref()
    }
}

impl Drop for KyberCiphertext {
    fn drop(&mut self) {
        unsafe {
            let ct = *self.inner.as_ref();
            if !ct.is_null() {
                kyber_ciphertext_free(ct);
            }
        }
    }
}

/// Kyber shared secret with secure memory management and zeroization
pub struct KyberSharedSecret {
    inner: SecureMemory<Vec<u8>>,
    params: KyberParameterSet,
}

impl KyberSharedSecret {
    /// Create a new shared secret
    pub fn new(params: KyberParameterSet, data: Vec<u8>) -> Result<Self, LibOqsError> {
        let expected_len = params.shared_secret_length();
        if data.len() != expected_len {
            return Err(LibOqsError::InvalidParameter(format!(
                "Invalid shared secret length: expected {}, got {}",
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
    pub fn parameter_set(&self) -> KyberParameterSet {
        self.params
    }

    /// Get the shared secret bytes
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_ref()
    }

    /// Consume and return the shared secret bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.inner.into_inner()
    }
}

impl Drop for KyberSharedSecret {
    fn drop(&mut self) {
        // SecureMemory will handle zeroization
    }
}

/// Main Kyber KEM implementation
pub struct Kyber;

impl Kyber {
    /// Generate a new Kyber keypair
    pub fn keygen(params: KyberParameterSet) -> Result<(KyberPublicKey, KyberSecretKey), LibOqsError> {
        unsafe {
            let mut pk = ptr::null_mut();
            let mut sk = ptr::null_mut();
            
            let result = kyber_keygen(params.into(), &mut pk, &mut sk);
            
            if result != 0 {
                return Err(LibOqsError::from(result));
            }

            if pk.is_null() || sk.is_null() {
                if !pk.is_null() {
                    kyber_public_key_free(pk);
                }
                if !sk.is_null() {
                    kyber_secret_key_free(sk);
                }
                return Err(LibOqsError::NullPointer("Key generation failed"));
            }

            let public_key = KyberPublicKey {
                inner: SecureMemory::new(pk),
                params,
                _phantom: PhantomData,
            };

            let secret_key = KyberSecretKey {
                inner: SecureMemory::new(sk),
                params,
                _phantom: PhantomData,
            };

            Ok((public_key, secret_key))
        }
    }

    /// Encapsulate a shared secret using a public key
    pub fn encapsulate(pk: &KyberPublicKey) -> Result<(KyberCiphertext, KyberSharedSecret), LibOqsError> {
        unsafe {
            let mut ct = ptr::null_mut();
            let mut ss = ptr::null_mut();
            
            let result = kyber_encaps(pk.as_ptr(), &mut ct, &mut ss);
            
            if result != 0 {
                return Err(LibOqsError::from(result));
            }

            if ct.is_null() || ss.is_null() {
                if !ct.is_null() {
                    kyber_ciphertext_free(ct);
                }
                if !ss.is_null() {
                    kyber_shared_secret_free(ss);
                }
                return Err(LibOqsError::NullPointer("Encapsulation failed"));
            }

            let ciphertext = KyberCiphertext {
                inner: SecureMemory::new(ct),
                params: pk.parameter_set(),
                _phantom: PhantomData,
            };

            let shared_secret = KyberSharedSecret {
                inner: SecureMemory::new(unsafe {
                    let mut len = 0;
                    let len_result = kyber_shared_secret_export_len(ss, &mut len);
                    if len_result != 0 {
                        return Err(LibOqsError::from(len_result));
                    }
                    
                    let mut buffer = vec![0u8; len];
                    let export_result = kyber_shared_secret_export(ss, buffer.as_mut_ptr(), len);
                    if export_result != 0 {
                        return Err(LibOqsError::from(export_result));
                    }
                    
                    buffer
                }),
                params: pk.parameter_set(),
            };

            // Free the liboqs shared secret object
            kyber_shared_secret_free(ss);

            Ok((ciphertext, shared_secret))
        }
    }

    /// Decapsulate a shared secret using a secret key
    pub fn decapsulate(ct: &KyberCiphertext, sk: &KyberSecretKey) -> Result<KyberSharedSecret, LibOqsError> {
        if ct.parameter_set() != sk.parameter_set() {
            return Err(LibOqsError::InvalidParameter("Parameter set mismatch"));
        }

        unsafe {
            let mut ss = ptr::null_mut();
            
            let result = kyber_decaps(ct.as_ptr(), sk.as_ptr(), &mut ss);
            
            if result != 0 {
                return Err(LibOqsError::from(result));
            }

            if ss.is_null() {
                return Err(LibOqsError::NullPointer("Decapsulation failed"));
            }

            let shared_secret = KyberSharedSecret {
                inner: SecureMemory::new({
                    let mut len = 0;
                    let len_result = kyber_shared_secret_export_len(ss, &mut len);
                    if len_result != 0 {
                        kyber_shared_secret_free(ss);
                        return Err(LibOqsError::from(len_result));
                    }
                    
                    let mut buffer = vec![0u8; len];
                    let export_result = kyber_shared_secret_export(ss, buffer.as_mut_ptr(), len);
                    if export_result != 0 {
                        kyber_shared_secret_free(ss);
                        return Err(LibOqsError::from(export_result));
                    }
                    
                    buffer
                }),
                params: ct.parameter_set(),
            };

            // Free the liboqs shared secret object
            kyber_shared_secret_free(ss);

            Ok(shared_secret)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kyber_roundtrip() {
        let params = KyberParameterSet::Kyber768;
        
        // Generate keypair
        let (pk, sk) = Kyber::keygen(params).unwrap();
        
        // Encapsulate
        let (ct, ss1) = Kyber::encapsulate(&pk).unwrap();
        
        // Decapsulate
        let ss2 = Kyber::decapsulate(&ct, &sk).unwrap();
        
        // Verify shared secrets match
        assert_eq!(ss1.as_bytes(), ss2.as_bytes());
    }

    #[test]
    fn test_kyber_parameter_sets() {
        let test_cases = [
            KyberParameterSet::Kyber512,
            KyberParameterSet::Kyber768,
            KyberParameterSet::Kyber1024,
        ];

        for params in test_cases {
            let (pk, sk) = Kyber::keygen(params).unwrap();
            
            // Test key lengths
            assert_eq!(pk.export().unwrap().len(), params.public_key_length());
            assert_eq!(sk.export().unwrap().len(), params.secret_key_length());
            
            // Test roundtrip
            let (ct, ss1) = Kyber::encapsulate(&pk).unwrap();
            let ss2 = Kyber::decapsulate(&ct, &sk).unwrap();
            assert_eq!(ss1.as_bytes(), ss2.as_bytes());
            
            // Test ciphertext length
            assert_eq!(ct.export().unwrap().len(), params.ciphertext_length());
            
            // Test shared secret length
            assert_eq!(ss1.as_bytes().len(), params.shared_secret_length());
        }
    }

    #[test]
    fn test_kyber_zeroization() {
        let params = KyberParameterSet::Kyber768;
        let (pk, sk) = Kyber::keygen(params).unwrap();
        
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
    fn test_kyber_performance() {
        let params = KyberParameterSet::Kyber768;
        let iterations = 100;
        
        let start = std::time::Instant::now();
        
        for _ in 0..iterations {
            let (pk, sk) = Kyber::keygen(params).unwrap();
            let (ct, _) = Kyber::encapsulate(&pk).unwrap();
            let _ = Kyber::decapsulate(&ct, &sk).unwrap();
        }
        
        let duration = start.elapsed();
        let avg_time = duration / iterations;
        
        // Verify performance targets (relaxed for testing)
        assert!(avg_time.as_millis() < 50); // < 50ms average for testing
    }
}
