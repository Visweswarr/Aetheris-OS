/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2024 Polymera OS Contributors
 * 
 * libpolycrypto - Canonical cryptographic primitives for Polymera OS
 * 
 * This header provides constant-time XChaCha20-Poly1305 AEAD encryption
 * with hardened input validation and secure memory zeroing.
 */

#ifndef POLYCRYPTO_H
#define POLYCRYPTO_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Error codes */
#define AETH_OK 0
#define AETH_ERROR_INVALID_INPUT -1
#define AETH_ERROR_CRYPTO_FAILURE -2
#define AETH_ERROR_BUFFER_TOO_SMALL -3

/* Constants */
#define AETH_KEY_SIZE 32
#define AETH_NONCE_SIZE 24
#define AETH_TAG_SIZE 16
#define AETH_MAX_PAYLOAD_SIZE (256 * 1024)  /* 256 KiB */

/* XChaCha20-Poly1305 AEAD encryption */
/**
 * Seal (encrypt) data using XChaCha20-Poly1305
 * 
 * @param key32 32-byte encryption key
 * @param nonce24 24-byte nonce
 * @param ad Associated data (can be NULL if ad_len is 0)
 * @param ad_len Length of associated data
 * @param pt Plaintext data
 * @param pt_len Length of plaintext (must be <= AETH_MAX_PAYLOAD_SIZE)
 * @param ct_out Output buffer for ciphertext (must be pt_len bytes)
 * @param tag16_out Output buffer for authentication tag (16 bytes)
 * 
 * @return AETH_OK on success, negative error code on failure
 * 
 * Security guarantees:
 * - Constant-time execution (no timing-dependent branches on secrets)
 * - Input validation (no overlapping buffers, length checks)
 * - Secure memory handling
 */
int aeth_xchacha20_seal(
    const uint8_t* key32,
    const uint8_t* nonce24,
    const uint8_t* ad,
    size_t ad_len,
    const uint8_t* pt,
    size_t pt_len,
    uint8_t* ct_out,
    uint8_t* tag16_out
);

/* XChaCha20-Poly1305 AEAD decryption */
/**
 * Open (decrypt) data using XChaCha20-Poly1305
 * 
 * @param key32 32-byte decryption key
 * @param nonce24 24-byte nonce
 * @param ad Associated data (must match what was used during encryption)
 * @param ad_len Length of associated data
 * @param ct Ciphertext data
 * @param ct_len Length of ciphertext
 * @param tag16 Authentication tag (16 bytes)
 * @param pt_out Output buffer for plaintext (must be ct_len bytes)
 * 
 * @return AETH_OK on success, negative error code on failure
 * 
 * Security guarantees:
 * - Constant-time execution
 * - Authentication tag verification
 * - Input validation
 */
int aeth_xchacha20_open(
    const uint8_t* key32,
    const uint8_t* nonce24,
    const uint8_t* ad,
    size_t ad_len,
    const uint8_t* ct,
    size_t ct_len,
    const uint8_t* tag16,
    uint8_t* pt_out
);

/* Secure memory zeroing */
/**
 * Securely zero memory
 * 
 * @param p Pointer to memory to zero
 * @param n Number of bytes to zero
 * 
 * This function ensures that the memory is zeroed even if the compiler
 * optimizes away the operation. It should be used for sensitive data
 * like encryption keys.
 */
void aeth_memzero(void* p, size_t n);

/* Input validation helpers */
/**
 * Validate buffer parameters for security
 * 
 * @param ptr Pointer to validate
 * @param len Length to validate
 * @param max_len Maximum allowed length
 * 
 * @return true if valid, false otherwise
 */
bool aeth_validate_buffer(const void* ptr, size_t len, size_t max_len);

/**
 * Check for buffer overlap
 * 
 * @param ptr1 First buffer pointer
 * @param len1 First buffer length
 * @param ptr2 Second buffer pointer
 * @param len2 Second buffer length
 * 
 * @return true if buffers overlap, false otherwise
 */
bool aeth_buffers_overlap(const void* ptr1, size_t len1, const void* ptr2, size_t len2);

#ifdef __cplusplus
}
#endif

#endif /* POLYCRYPTO_H */
