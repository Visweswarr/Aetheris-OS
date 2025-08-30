//! FFI bindings for libpolycrypto
//! 
//! This crate provides safe Rust bindings to the canonical C cryptographic
//! primitives implemented in libpolycrypto.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ptr;

/// Error codes from the C library
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolyCryptoError {
    /// Operation completed successfully
    Ok = 0,
    /// Invalid input parameters
    InvalidInput = -1,
    /// Cryptographic operation failed
    CryptoFailure = -2,
    /// Buffer too small for operation
    BufferTooSmall = -3,
}

impl From<i32> for PolyCryptoError {
    fn from(code: i32) -> Self {
        match code {
            0 => PolyCryptoError::Ok,
            -1 => PolyCryptoError::InvalidInput,
            -2 => PolyCryptoError::CryptoFailure,
            -3 => PolyCryptoError::BufferTooSmall,
            _ => PolyCryptoError::CryptoFailure,
        }
    }
}

/// Constants from the C library
pub const AETH_KEY_SIZE: usize = 32;
pub const AETH_NONCE_SIZE: usize = 24;
pub const AETH_TAG_SIZE: usize = 16;
pub const AETH_MAX_PAYLOAD_SIZE: usize = 256 * 1024; // 256 KiB

/// Raw FFI functions from the C library
#[link(name = "polycrypto")]
extern "C" {
    /// Seal (encrypt) data using XChaCha20-Poly1305
    pub fn aeth_xchacha20_seal(
        key32: *const u8,
        nonce24: *const u8,
        ad: *const u8,
        ad_len: usize,
        pt: *const u8,
        pt_len: usize,
        ct_out: *mut u8,
        tag16_out: *mut u8,
    ) -> i32;

    /// Open (decrypt) data using XChaCha20-Poly1305
    pub fn aeth_xchacha20_open(
        key32: *const u8,
        nonce24: *const u8,
        ad: *const u8,
        ad_len: usize,
        ct: *const u8,
        ct_len: usize,
        tag16: *const u8,
        pt_out: *mut u8,
    ) -> i32;

    /// Securely zero memory
    pub fn aeth_memzero(p: *mut u8, n: usize);

    /// Validate buffer parameters
    pub fn aeth_validate_buffer(ptr: *const u8, len: usize, max_len: usize) -> bool;

    /// Check for buffer overlap
    pub fn aeth_buffers_overlap(
        ptr1: *const u8,
        len1: usize,
        ptr2: *const u8,
        len2: usize,
    ) -> bool;
}

/// Safe wrapper for XChaCha20-Poly1305 encryption
pub fn xchacha20_seal(
    key: &[u8; AETH_KEY_SIZE],
    nonce: &[u8; AETH_NONCE_SIZE],
    ad: &[u8],
    plaintext: &[u8],
) -> Result<(Vec<u8>, [u8; AETH_TAG_SIZE]), PolyCryptoError> {
    // Input validation
    if plaintext.len() > AETH_MAX_PAYLOAD_SIZE {
        return Err(PolyCryptoError::InvalidInput);
    }

    let mut ciphertext = vec![0u8; plaintext.len()];
    let mut tag = [0u8; AETH_TAG_SIZE];

    let result = unsafe {
        aeth_xchacha20_seal(
            key.as_ptr(),
            nonce.as_ptr(),
            if ad.is_empty() { ptr::null() } else { ad.as_ptr() },
            ad.len(),
            plaintext.as_ptr(),
            plaintext.len(),
            ciphertext.as_mut_ptr(),
            tag.as_mut_ptr(),
        )
    };

    if result == 0 {
        Ok((ciphertext, tag))
    } else {
        Err(PolyCryptoError::from(result))
    }
}

/// Safe wrapper for XChaCha20-Poly1305 decryption
pub fn xchacha20_open(
    key: &[u8; AETH_KEY_SIZE],
    nonce: &[u8; AETH_NONCE_SIZE],
    ad: &[u8],
    ciphertext: &[u8],
    tag: &[u8; AETH_TAG_SIZE],
) -> Result<Vec<u8>, PolyCryptoError> {
    // Input validation
    if ciphertext.len() > AETH_MAX_PAYLOAD_SIZE {
        return Err(PolyCryptoError::InvalidInput);
    }

    let mut plaintext = vec![0u8; ciphertext.len()];

    let result = unsafe {
        aeth_xchacha20_open(
            key.as_ptr(),
            nonce.as_ptr(),
            if ad.is_empty() { ptr::null() } else { ad.as_ptr() },
            ad.len(),
            ciphertext.as_ptr(),
            ciphertext.len(),
            tag.as_ptr(),
            plaintext.as_mut_ptr(),
        )
    };

    if result == 0 {
        Ok(plaintext)
    } else {
        Err(PolyCryptoError::from(result))
    }
}

/// Safe wrapper for secure memory zeroing
pub fn memzero(data: &mut [u8]) {
    unsafe {
        aeth_memzero(data.as_mut_ptr(), data.len());
    }
}

/// Safe wrapper for buffer validation
pub fn validate_buffer(data: &[u8], max_len: usize) -> bool {
    unsafe {
        aeth_validate_buffer(data.as_ptr(), data.len(), max_len)
    }
}

/// Safe wrapper for buffer overlap detection
pub fn buffers_overlap(data1: &[u8], data2: &[u8]) -> bool {
    unsafe {
        aeth_buffers_overlap(
            data1.as_ptr(),
            data1.len(),
            data2.as_ptr(),
            data2.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(AETH_KEY_SIZE, 32);
        assert_eq!(AETH_NONCE_SIZE, 24);
        assert_eq!(AETH_TAG_SIZE, 16);
        assert_eq!(AETH_MAX_PAYLOAD_SIZE, 256 * 1024);
    }

    #[test]
    fn test_buffer_validation() {
        let data = [1u8, 2, 3, 4];
        assert!(validate_buffer(&data, 10));
        assert!(!validate_buffer(&data, 2));
    }

    #[test]
    fn test_buffer_overlap() {
        let data1 = [1u8, 2, 3, 4];
        let data2 = [5u8, 6, 7, 8];
        assert!(!buffers_overlap(&data1, &data2));

        let data3 = [1u8, 2, 3, 4];
        let data4 = [3u8, 4, 5, 6];
        assert!(buffers_overlap(&data3, &data4));
    }

    #[test]
    fn test_memzero() {
        let mut data = [1u8, 2, 3, 4];
        memzero(&mut data);
        assert_eq!(data, [0u8, 0, 0, 0]);
    }
}
