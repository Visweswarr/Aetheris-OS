use std::ffi::c_void;
use std::marker::PhantomData;
use std::ops::Drop;
use std::ptr;

use super::bindings::*;
use super::error::LibOqsError;

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

/// Kyber public key
pub struct KyberPublicKey {
    inner: kyber_public_key_t,
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

        let mut inner = kyber_public_key_t {
            key_data: liboqs_buffer_t {
                data: ptr::null_mut(),
                length: 0,
                is_allocated: 0,
            },
            params: params.into(),
        };

        // Allocate and copy key data
        unsafe {
            inner.key_data.data = libc::malloc(key_data.len()) as *mut u8;
            if inner.key_data.data.is_null() {
                return Err(LibOqsError::Memory);
            }
            ptr::copy_nonoverlapping(key_data.as_ptr(), inner.key_data.data, key_data.len());
            inner.key_data.length = key_data.len();
            inner.key_data.is_allocated = 1;
        }

        Ok(Self {
            inner,
            _phantom: PhantomData,
        })
    }

    /// Get the parameter set
    pub fn parameter_set(&self) -> KyberParameterSet {
        match self.inner.params {
            KYBER_512 => KyberParameterSet::Kyber512,
            KYBER_768 => KyberParameterSet::Kyber768,
            KYBER_1024 => KyberParameterSet::Kyber1024,
            _ => unreachable!(),
        }
    }

    /// Get the key data as a slice
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.inner.key_data.data, self.inner.key_data.length)
        }
    }

    /// Get the key data as a vector (copying)
    pub fn to_vec(&self) -> Vec<u8> {
        self.as_slice().to_vec()
    }

    /// Get the key length in bytes
    pub fn len(&self) -> usize {
        self.inner.key_data.length
    }

    /// Check if the key is empty
    pub fn is_empty(&self) -> bool {
        self.inner.key_data.length == 0
    }
}

impl Drop for KyberPublicKey {
    fn drop(&mut self) {
        unsafe {
            if !self.inner.key_data.data.is_null() && self.inner.key_data.is_allocated != 0 {
                kyber_public_key_free(&mut self.inner);
            }
        }
    }
}

impl Clone for KyberPublicKey {
    fn clone(&self) -> Self {
        let key_data = self.to_vec();
        let params = self.parameter_set();
        Self::new(params, key_data).expect("Failed to clone public key")
    }
}

/// Kyber secret key
pub struct KyberSecretKey {
    inner: kyber_secret_key_t,
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

        let mut inner = kyber_secret_key_t {
            key_data: liboqs_buffer_t {
                data: ptr::null_mut(),
                length: 0,
                is_allocated: 0,
            },
            params: params.into(),
        };

        // Allocate and copy key data
        unsafe {
            inner.key_data.data = libc::malloc(key_data.len()) as *mut u8;
            if inner.key_data.data.is_null() {
                return Err(LibOqsError::Memory);
            }
            ptr::copy_nonoverlapping(key_data.as_ptr(), inner.key_data.data, key_data.len());
            inner.key_data.length = key_data.len();
            inner.key_data.is_allocated = 1;
        }

        Ok(Self {
            inner,
            _phantom: PhantomData,
        })
    }

    /// Get the parameter set
    pub fn parameter_set(&self) -> KyberParameterSet {
        match self.inner.params {
            KYBER_512 => KyberParameterSet::Kyber512,
            KYBER_768 => KyberParameterSet::Kyber768,
            KYBER_1024 => KyberParameterSet::Kyber1024,
            _ => unreachable!(),
        }
    }

    /// Get the key data as a slice
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.inner.key_data.data, self.inner.key_data.length)
        }
    }

    /// Get the key data as a vector (copying)
    pub fn to_vec(&self) -> Vec<u8> {
        self.as_slice().to_vec()
    }

    /// Get the key length in bytes
    pub fn len(&self) -> usize {
        self.inner.key_data.length
    }

    /// Check if the key is empty
    pub fn is_empty(&self) -> bool {
        self.inner.key_data.length == 0
    }
}

impl Drop for KyberSecretKey {
    fn drop(&mut self) {
        unsafe {
            if !self.inner.key_data.data.is_null() && self.inner.key_data.is_allocated != 0 {
                kyber_secret_key_free(&mut self.inner);
            }
        }
    }
}

impl Clone for KyberSecretKey {
    fn clone(&self) -> Self {
        let key_data = self.to_vec();
        let params = self.parameter_set();
        Self::new(params, key_data).expect("Failed to clone secret key")
    }
}

/// Kyber ciphertext
pub struct KyberCiphertext {
    inner: kyber_ciphertext_t,
    _phantom: PhantomData<*mut c_void>,
}

impl KyberCiphertext {
    /// Create a new ciphertext from raw bytes
    pub fn new(params: KyberParameterSet, ciphertext_data: Vec<u8>) -> Result<Self, LibOqsError> {
        let expected_len = params.ciphertext_length();
        if ciphertext_data.len() != expected_len {
            return Err(LibOqsError::InvalidParameter(format!(
                "Invalid ciphertext length: expected {}, got {}",
                expected_len,
                ciphertext_data.len()
            )));
        }

        let mut inner = kyber_ciphertext_t {
            ciphertext: liboqs_buffer_t {
                data: ptr::null_mut(),
                length: 0,
                is_allocated: 0,
            },
            params: params.into(),
        };

        // Allocate and copy ciphertext data
        unsafe {
            inner.ciphertext.data = libc::malloc(ciphertext_data.len()) as *mut u8;
            if inner.ciphertext.data.is_null() {
                return Err(LibOqsError::Memory);
            }
            ptr::copy_nonoverlapping(ciphertext_data.as_ptr(), inner.ciphertext.data, ciphertext_data.len());
            inner.ciphertext.length = ciphertext_data.len();
            inner.ciphertext.is_allocated = 1;
        }

        Ok(Self {
            inner,
            _phantom: PhantomData,
        })
    }

    /// Get the parameter set
    pub fn parameter_set(&self) -> KyberParameterSet {
        match self.inner.params {
            KYBER_512 => KyberParameterSet::Kyber512,
            KYBER_768 => KyberParameterSet::Kyber768,
            KYBER_1024 => KyberParameterSet::Kyber1024,
            _ => unreachable!(),
        }
    }

    /// Get the ciphertext data as a slice
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.inner.ciphertext.data, self.inner.ciphertext.length)
        }
    }

    /// Get the ciphertext data as a vector (copying)
    pub fn to_vec(&self) -> Vec<u8> {
        self.as_slice().to_vec()
    }

    /// Get the ciphertext length in bytes
    pub fn len(&self) -> usize {
        self.inner.ciphertext.length
    }

    /// Check if the ciphertext is empty
    pub fn is_empty(&self) -> bool {
        self.inner.ciphertext.length == 0
    }
}

impl Drop for KyberCiphertext {
    fn drop(&mut self) {
        unsafe {
            if !self.inner.ciphertext.data.is_null() && self.inner.ciphertext.is_allocated != 0 {
                kyber_ciphertext_free(&mut self.inner);
            }
        }
    }
}

impl Clone for KyberCiphertext {
    fn clone(&self) -> Self {
        let ciphertext_data = self.to_vec();
        let params = self.parameter_set();
        Self::new(params, ciphertext_data).expect("Failed to clone ciphertext")
    }
}

/// Kyber shared secret
pub struct KyberSharedSecret {
    inner: kyber_shared_secret_t,
    _phantom: PhantomData<*mut c_void>,
}

impl KyberSharedSecret {
    /// Create a new shared secret from raw bytes
    pub fn new(params: KyberParameterSet, secret_data: Vec<u8>) -> Result<Self, LibOqsError> {
        let expected_len = params.shared_secret_length();
        if secret_data.len() != expected_len {
            return Err(LibOqsError::InvalidParameter(format!(
                "Invalid shared secret length: expected {}, got {}",
                expected_len,
                secret_data.len()
            )));
        }

        let mut inner = kyber_shared_secret_t {
            shared_secret: liboqs_buffer_t {
                data: ptr::null_mut(),
                length: 0,
                is_allocated: 0,
            },
            params: params.into(),
        };

        // Allocate and copy secret data
        unsafe {
            inner.shared_secret.data = libc::malloc(secret_data.len()) as *mut u8;
            if inner.shared_secret.data.is_null() {
                return Err(LibOqsError::Memory);
            }
            ptr::copy_nonoverlapping(secret_data.as_ptr(), inner.shared_secret.data, secret_data.len());
            inner.shared_secret.length = secret_data.len();
            inner.shared_secret.is_allocated = 1;
        }

        Ok(Self {
            inner,
            _phantom: PhantomData,
        })
    }

    /// Get the parameter set
    pub fn parameter_set(&self) -> KyberParameterSet {
        match self.inner.params {
            KYBER_512 => KyberParameterSet::Kyber512,
            KYBER_768 => KyberParameterSet::Kyber768,
            KYBER_1024 => KyberParameterSet::Kyber1024,
            _ => unreachable!(),
        }
    }

    /// Get the secret data as a slice
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.inner.shared_secret.data, self.inner.shared_secret.length)
        }
    }

    /// Get the secret data as a vector (copying)
    pub fn to_vec(&self) -> Vec<u8> {
        self.as_slice().to_vec()
    }

    /// Get the secret length in bytes
    pub fn len(&self) -> usize {
        self.inner.shared_secret.length
    }

    /// Check if the secret is empty
    pub fn is_empty(&self) -> bool {
        self.inner.shared_secret.length == 0
    }
}

impl Drop for KyberSharedSecret {
    fn drop(&mut self) {
        unsafe {
            if !self.inner.shared_secret.data.is_null() && self.inner.shared_secret.is_allocated != 0 {
                kyber_shared_secret_free(&mut self.inner);
            }
        }
    }
}

impl Clone for KyberSharedSecret {
    fn clone(&self) -> Self {
        let secret_data = self.to_vec();
        let params = self.parameter_set();
        Self::new(params, secret_data).expect("Failed to clone shared secret")
    }
}

/// Kyber KEM implementation
pub struct KyberKem;

impl KyberKem {
    /// Generate a new keypair
    pub fn generate_keypair(
        params: KyberParameterSet,
    ) -> Result<(KyberPublicKey, KyberSecretKey), LibOqsError> {
        let mut public_key = kyber_public_key_t {
            key_data: liboqs_buffer_t {
                data: ptr::null_mut(),
                length: 0,
                is_allocated: 0,
            },
            params: params.into(),
        };

        let mut secret_key = kyber_secret_key_t {
            key_data: liboqs_buffer_t {
                data: ptr::null_mut(),
                length: 0,
                is_allocated: 0,
            },
            params: params.into(),
        };

        let result = unsafe {
            kyber_keypair_generate(params.into(), &mut public_key, &mut secret_key)
        };

        if result != LIBOQS_SUCCESS {
            return Err(LibOqsError::from(result));
        }

        let public_key = KyberPublicKey {
            inner: public_key,
            _phantom: PhantomData,
        };

        let secret_key = KyberSecretKey {
            inner: secret_key,
            _phantom: PhantomData,
        };

        Ok((public_key, secret_key))
    }

    /// Encapsulate a shared secret using a public key
    pub fn encapsulate(
        public_key: &KyberPublicKey,
    ) -> Result<(KyberCiphertext, KyberSharedSecret), LibOqsError> {
        let mut ciphertext = kyber_ciphertext_t {
            ciphertext: liboqs_buffer_t {
                data: ptr::null_mut(),
                length: 0,
                is_allocated: 0,
            },
            params: public_key.inner.params,
        };

        let mut shared_secret = kyber_shared_secret_t {
            shared_secret: liboqs_buffer_t {
                data: ptr::null_mut(),
                length: 0,
                is_allocated: 0,
            },
            params: public_key.inner.params,
        };

        let result = unsafe {
            kyber_encapsulate(&public_key.inner, &mut ciphertext, &mut shared_secret)
        };

        if result != LIBOQS_SUCCESS {
            return Err(LibOqsError::from(result));
        }

        let ciphertext = KyberCiphertext {
            inner: ciphertext,
            _phantom: PhantomData,
        };

        let shared_secret = KyberSharedSecret {
            inner: shared_secret,
            _phantom: PhantomData,
        };

        Ok((ciphertext, shared_secret))
    }

    /// Decapsulate a shared secret using a secret key and ciphertext
    pub fn decapsulate(
        secret_key: &KyberSecretKey,
        ciphertext: &KyberCiphertext,
    ) -> Result<KyberSharedSecret, LibOqsError> {
        let mut shared_secret = kyber_shared_secret_t {
            shared_secret: liboqs_buffer_t {
                data: ptr::null_mut(),
                length: 0,
                is_allocated: 0,
            },
            params: secret_key.inner.params,
        };

        let result = unsafe {
            kyber_decapsulate(&secret_key.inner, &ciphertext.inner, &mut shared_secret)
        };

        if result != LIBOQS_SUCCESS {
            return Err(LibOqsError::from(result));
        }

        let shared_secret = KyberSharedSecret {
            inner: shared_secret,
            _phantom: PhantomData,
        };

        Ok(shared_secret)
    }

    /// Get test vectors for known-answer tests
    pub fn get_test_vectors(
        params: KyberParameterSet,
    ) -> Result<TestVectors, LibOqsError> {
        let pub_len = params.public_key_length();
        let sec_len = params.secret_key_length();
        let cipher_len = params.ciphertext_length();
        let secret_len = params.shared_secret_length();

        let mut public_key = vec![0u8; pub_len];
        let mut secret_key = vec![0u8; sec_len];
        let mut ciphertext = vec![0u8; cipher_len];
        let mut shared_secret = vec![0u8; secret_len];

        let result = unsafe {
            kyber_get_test_vectors(
                params.into(),
                public_key.as_mut_ptr(),
                secret_key.as_mut_ptr(),
                ciphertext.as_mut_ptr(),
                shared_secret.as_mut_ptr(),
            )
        };

        if result != LIBOQS_SUCCESS {
            return Err(LibOqsError::from(result));
        }

        Ok(TestVectors {
            public_key,
            secret_key,
            ciphertext,
            shared_secret,
            params,
        })
    }
}

/// Test vectors for known-answer tests
pub struct TestVectors {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub shared_secret: Vec<u8>,
    pub params: KyberParameterSet,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kyber_parameter_sets() {
        let params = KyberParameterSet::Kyber512;
        assert_eq!(params.security_level(), 128);
        assert_eq!(params.algorithm_name(), "Kyber512");
        assert!(params.public_key_length() > 0);
        assert!(params.secret_key_length() > 0);
        assert!(params.ciphertext_length() > 0);
        assert!(params.shared_secret_length() > 0);

        let params = KyberParameterSet::Kyber768;
        assert_eq!(params.security_level(), 192);
        assert_eq!(params.algorithm_name(), "Kyber768");

        let params = KyberParameterSet::Kyber1024;
        assert_eq!(params.security_level(), 256);
        assert_eq!(params.algorithm_name(), "Kyber1024");
    }

    #[test]
    fn test_kyber_keypair_generation() {
        let params = KyberParameterSet::Kyber512;
        let (public_key, secret_key) = KyberKem::generate_keypair(params).unwrap();

        assert_eq!(public_key.parameter_set(), params);
        assert_eq!(secret_key.parameter_set(), params);
        assert_eq!(public_key.len(), params.public_key_length());
        assert_eq!(secret_key.len(), params.secret_key_length());
        assert!(!public_key.is_empty());
        assert!(!secret_key.is_empty());
    }

    #[test]
    fn test_kyber_encapsulation_decapsulation() {
        let params = KyberParameterSet::Kyber512;
        let (public_key, secret_key) = KyberKem::generate_keypair(params).unwrap();

        // Encapsulate
        let (ciphertext, shared_secret1) = KyberKem::encapsulate(&public_key).unwrap();

        assert_eq!(ciphertext.parameter_set(), params);
        assert_eq!(shared_secret1.parameter_set(), params);
        assert_eq!(ciphertext.len(), params.ciphertext_length());
        assert_eq!(shared_secret1.len(), params.shared_secret_length());

        // Decapsulate
        let shared_secret2 = KyberKem::decapsulate(&secret_key, &ciphertext).unwrap();

        assert_eq!(shared_secret2.parameter_set(), params);
        assert_eq!(shared_secret2.len(), params.shared_secret_length());

        // Verify shared secrets match
        assert_eq!(shared_secret1.as_slice(), shared_secret2.as_slice());
    }

    #[test]
    fn test_kyber_clone() {
        let params = KyberParameterSet::Kyber512;
        let (public_key, secret_key) = KyberKem::generate_keypair(params).unwrap();

        let public_key_clone = public_key.clone();
        let secret_key_clone = secret_key.clone();

        assert_eq!(public_key.as_slice(), public_key_clone.as_slice());
        assert_eq!(secret_key.as_slice(), secret_key_clone.as_slice());
        assert_eq!(public_key.parameter_set(), public_key_clone.parameter_set());
        assert_eq!(secret_key.parameter_set(), secret_key_clone.parameter_set());
    }

    #[test]
    fn test_kyber_invalid_parameters() {
        // Test with invalid key lengths
        let params = KyberParameterSet::Kyber512;
        let invalid_key = vec![0u8; 10]; // Wrong length

        assert!(KyberPublicKey::new(params, invalid_key.clone()).is_err());
        assert!(KyberSecretKey::new(params, invalid_key.clone()).is_err());
        assert!(KyberCiphertext::new(params, invalid_key.clone()).is_err());
        assert!(KyberSharedSecret::new(params, invalid_key).is_err());
    }
}
