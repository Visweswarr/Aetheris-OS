use std::ffi::c_void;
use std::marker::PhantomData;
use std::ops::Drop;
use std::ptr;

use super::bindings::*;
use super::error::LibOqsError;

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

/// Dilithium public key
pub struct DilithiumPublicKey {
    inner: dilithium_public_key_t,
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

        let mut inner = dilithium_public_key_t {
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
    pub fn parameter_set(&self) -> DilithiumParameterSet {
        match self.inner.params {
            DILITHIUM_2 => DilithiumParameterSet::Dilithium2,
            DILITHIUM_3 => DilithiumParameterSet::Dilithium3,
            DILITHIUM_5 => DilithiumParameterSet::Dilithium5,
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

impl Drop for DilithiumPublicKey {
    fn drop(&mut self) {
        unsafe {
            if !self.inner.key_data.data.is_null() && self.inner.key_data.is_allocated != 0 {
                dilithium_public_key_free(&mut self.inner);
            }
        }
    }
}

impl Clone for DilithiumPublicKey {
    fn clone(&self) -> Self {
        let key_data = self.to_vec();
        let params = self.parameter_set();
        Self::new(params, key_data).expect("Failed to clone public key")
    }
}

/// Dilithium secret key
pub struct DilithiumSecretKey {
    inner: dilithium_secret_key_t,
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

        let mut inner = dilithium_secret_key_t {
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
    pub fn parameter_set(&self) -> DilithiumParameterSet {
        match self.inner.params {
            DILITHIUM_2 => DilithiumParameterSet::Dilithium2,
            DILITHIUM_3 => DilithiumParameterSet::Dilithium3,
            DILITHIUM_5 => DilithiumParameterSet::Dilithium5,
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

impl Drop for DilithiumSecretKey {
    fn drop(&mut self) {
        unsafe {
            if !self.inner.key_data.data.is_null() && self.inner.key_data.is_allocated != 0 {
                dilithium_secret_key_free(&mut self.inner);
            }
        }
    }
}

impl Clone for DilithiumSecretKey {
    fn clone(&self) -> Self {
        let key_data = self.to_vec();
        let params = self.parameter_set();
        Self::new(params, key_data).expect("Failed to clone secret key")
    }
}

/// Dilithium signature
pub struct DilithiumSignature {
    inner: dilithium_signature_t,
    _phantom: PhantomData<*mut c_void>,
}

impl DilithiumSignature {
    /// Create a new signature from raw bytes
    pub fn new(params: DilithiumParameterSet, signature_data: Vec<u8>) -> Result<Self, LibOqsError> {
        let expected_len = params.signature_length();
        if signature_data.len() != expected_len {
            return Err(LibOqsError::InvalidParameter(format!(
                "Invalid signature length: expected {}, got {}",
                expected_len,
                signature_data.len()
            )));
        }

        let mut inner = dilithium_signature_t {
            signature: liboqs_buffer_t {
                data: ptr::null_mut(),
                length: 0,
                is_allocated: 0,
            },
            params: params.into(),
        };

        // Allocate and copy signature data
        unsafe {
            inner.signature.data = libc::malloc(signature_data.len()) as *mut u8;
            if inner.signature.data.is_null() {
                return Err(LibOqsError::Memory);
            }
            ptr::copy_nonoverlapping(signature_data.as_ptr(), inner.signature.data, signature_data.len());
            inner.signature.length = signature_data.len();
            inner.signature.is_allocated = 1;
        }

        Ok(Self {
            inner,
            _phantom: PhantomData,
        })
    }

    /// Get the parameter set
    pub fn parameter_set(&self) -> DilithiumParameterSet {
        match self.inner.params {
            DILITHIUM_2 => DilithiumParameterSet::Dilithium2,
            DILITHIUM_3 => DilithiumParameterSet::Dilithium3,
            DILITHIUM_5 => DilithiumParameterSet::Dilithium5,
            _ => unreachable!(),
        }
    }

    /// Get the signature data as a slice
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.inner.signature.data, self.inner.signature.length)
        }
    }

    /// Get the signature data as a vector (copying)
    pub fn to_vec(&self) -> Vec<u8> {
        self.as_slice().to_vec()
    }

    /// Get the signature length in bytes
    pub fn len(&self) -> usize {
        self.inner.signature.length
    }

    /// Check if the signature is empty
    pub fn is_empty(&self) -> bool {
        self.inner.signature.length == 0
    }
}

impl Drop for DilithiumSignature {
    fn drop(&mut self) {
        unsafe {
            if !self.inner.signature.data.is_null() && self.inner.signature.is_allocated != 0 {
                dilithium_signature_free(&mut self.inner);
            }
        }
    }
}

impl Clone for DilithiumSignature {
    fn clone(&self) -> Self {
        let signature_data = self.to_vec();
        let params = self.parameter_set();
        Self::new(params, signature_data).expect("Failed to clone signature")
    }
}

/// Dilithium digital signature implementation
pub struct Dilithium;

impl Dilithium {
    /// Generate a new keypair
    pub fn generate_keypair(
        params: DilithiumParameterSet,
    ) -> Result<(DilithiumPublicKey, DilithiumSecretKey), LibOqsError> {
        let mut public_key = dilithium_public_key_t {
            key_data: liboqs_buffer_t {
                data: ptr::null_mut(),
                length: 0,
                is_allocated: 0,
            },
            params: params.into(),
        };

        let mut secret_key = dilithium_secret_key_t {
            key_data: liboqs_buffer_t {
                data: ptr::null_mut(),
                length: 0,
                is_allocated: 0,
            },
            params: params.into(),
        };

        let result = unsafe {
            dilithium_keypair_generate(params.into(), &mut public_key, &mut secret_key)
        };

        if result != LIBOQS_SUCCESS {
            return Err(LibOqsError::from(result));
        }

        let public_key = DilithiumPublicKey {
            inner: public_key,
            _phantom: PhantomData,
        };

        let secret_key = DilithiumSecretKey {
            inner: secret_key,
            _phantom: PhantomData,
        };

        Ok((public_key, secret_key))
    }

    /// Sign a message using a secret key
    pub fn sign(
        secret_key: &DilithiumSecretKey,
        message: &[u8],
    ) -> Result<DilithiumSignature, LibOqsError> {
        let mut signature = dilithium_signature_t {
            signature: liboqs_buffer_t {
                data: ptr::null_mut(),
                length: 0,
                is_allocated: 0,
            },
            params: secret_key.inner.params,
        };

        let result = unsafe {
            dilithium_sign(
                &secret_key.inner,
                message.as_ptr(),
                message.len(),
                &mut signature,
            )
        };

        if result != LIBOQS_SUCCESS {
            return Err(LibOqsError::from(result));
        }

        let signature = DilithiumSignature {
            inner: signature,
            _phantom: PhantomData,
        };

        Ok(signature)
    }

    /// Verify a signature using a public key
    pub fn verify(
        public_key: &DilithiumPublicKey,
        message: &[u8],
        signature: &DilithiumSignature,
    ) -> Result<bool, LibOqsError> {
        let result = unsafe {
            dilithium_verify(
                &public_key.inner,
                message.as_ptr(),
                message.len(),
                &signature.inner,
            )
        };

        if result == LIBOQS_SUCCESS {
            Ok(true)
        } else if result == LIBOQS_ERROR_VERIFICATION {
            Ok(false)
        } else {
            Err(LibOqsError::from(result))
        }
    }

    /// Get test vectors for known-answer tests
    pub fn get_test_vectors(
        params: DilithiumParameterSet,
        message: &[u8],
    ) -> Result<TestVectors, LibOqsError> {
        let pub_len = params.public_key_length();
        let sec_len = params.secret_key_length();
        let sig_len = params.signature_length();

        let mut public_key = vec![0u8; pub_len];
        let mut secret_key = vec![0u8; sec_len];
        let mut signature = vec![0u8; sig_len];

        let result = unsafe {
            dilithium_get_test_vectors(
                params.into(),
                public_key.as_mut_ptr(),
                secret_key.as_mut_ptr(),
                message.as_ptr(),
                message.len(),
                signature.as_mut_ptr(),
            )
        };

        if result != LIBOQS_SUCCESS {
            return Err(LibOqsError::from(result));
        }

        Ok(TestVectors {
            public_key,
            secret_key,
            message: message.to_vec(),
            signature,
            params,
        })
    }
}

/// Test vectors for known-answer tests
pub struct TestVectors {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
    pub message: Vec<u8>,
    pub signature: Vec<u8>,
    pub params: DilithiumParameterSet,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dilithium_parameter_sets() {
        let params = DilithiumParameterSet::Dilithium2;
        assert_eq!(params.security_level(), 128);
        assert_eq!(params.algorithm_name(), "Dilithium2");
        assert!(params.public_key_length() > 0);
        assert!(params.secret_key_length() > 0);
        assert!(params.signature_length() > 0);

        let params = DilithiumParameterSet::Dilithium3;
        assert_eq!(params.security_level(), 192);
        assert_eq!(params.algorithm_name(), "Dilithium3");

        let params = DilithiumParameterSet::Dilithium5;
        assert_eq!(params.security_level(), 256);
        assert_eq!(params.algorithm_name(), "Dilithium5");
    }

    #[test]
    fn test_dilithium_keypair_generation() {
        let params = DilithiumParameterSet::Dilithium2;
        let (public_key, secret_key) = Dilithium::generate_keypair(params).unwrap();

        assert_eq!(public_key.parameter_set(), params);
        assert_eq!(secret_key.parameter_set(), params);
        assert_eq!(public_key.len(), params.public_key_length());
        assert_eq!(secret_key.len(), params.secret_key_length());
        assert!(!public_key.is_empty());
        assert!(!secret_key.is_empty());
    }

    #[test]
    fn test_dilithium_sign_verify() {
        let params = DilithiumParameterSet::Dilithium2;
        let (public_key, secret_key) = Dilithium::generate_keypair(params).unwrap();

        let message = b"Hello, Polymera OS!";
        let signature = Dilithium::sign(&secret_key, message).unwrap();

        assert_eq!(signature.parameter_set(), params);
        assert_eq!(signature.len(), params.signature_length());
        assert!(!signature.is_empty());

        // Verify the signature
        let is_valid = Dilithium::verify(&public_key, message, &signature).unwrap();
        assert!(is_valid);

        // Verify with wrong message
        let wrong_message = b"Wrong message";
        let is_valid = Dilithium::verify(&public_key, wrong_message, &signature).unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_dilithium_clone() {
        let params = DilithiumParameterSet::Dilithium2;
        let (public_key, secret_key) = Dilithium::generate_keypair(params).unwrap();

        let public_key_clone = public_key.clone();
        let secret_key_clone = secret_key.clone();

        assert_eq!(public_key.as_slice(), public_key_clone.as_slice());
        assert_eq!(secret_key.as_slice(), secret_key_clone.as_slice());
        assert_eq!(public_key.parameter_set(), public_key_clone.parameter_set());
        assert_eq!(secret_key.parameter_set(), secret_key_clone.parameter_set());
    }

    #[test]
    fn test_dilithium_invalid_parameters() {
        // Test with invalid key lengths
        let params = DilithiumParameterSet::Dilithium2;
        let invalid_key = vec![0u8; 10]; // Wrong length

        assert!(DilithiumPublicKey::new(params, invalid_key.clone()).is_err());
        assert!(DilithiumSecretKey::new(params, invalid_key.clone()).is_err());
        assert!(DilithiumSignature::new(params, invalid_key).is_err());
    }
}
