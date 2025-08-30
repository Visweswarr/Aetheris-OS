/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2024 Polymera OS Contributors
 * 
 * XChaCha20-Poly1305 AEAD implementation using libsodium
 * 
 * This provides constant-time encryption with hardened input validation.
 */

#include "polycrypto.h"
#include <sodium.h>
#include <string.h>

/* Ensure libsodium is initialized */
static bool sodium_initialized = false;

static void ensure_sodium_init(void) {
    if (!sodium_initialized) {
        if (sodium_init() < 0) {
            /* This should never happen in a properly configured system */
            abort();
        }
        sodium_initialized = true;
    }
}

/* Input validation helper */
bool aeth_validate_buffer(const void* ptr, size_t len, size_t max_len) {
    if (ptr == NULL && len > 0) {
        return false;
    }
    if (len > max_len) {
        return false;
    }
    return true;
}

/* Buffer overlap detection */
bool aeth_buffers_overlap(const void* ptr1, size_t len1, const void* ptr2, size_t len2) {
    if (ptr1 == NULL || ptr2 == NULL) {
        return false;
    }
    
    const uint8_t* start1 = (const uint8_t*)ptr1;
    const uint8_t* end1 = start1 + len1;
    const uint8_t* start2 = (const uint8_t*)ptr2;
    const uint8_t* end2 = start2 + len2;
    
    /* Check if buffers overlap */
    return !(end1 <= start2 || end2 <= start1);
}

int aeth_xchacha20_seal(
    const uint8_t* key32,
    const uint8_t* nonce24,
    const uint8_t* ad,
    size_t ad_len,
    const uint8_t* pt,
    size_t pt_len,
    uint8_t* ct_out,
    uint8_t* tag16_out
) {
    /* Ensure libsodium is initialized */
    ensure_sodium_init();
    
    /* Input validation */
    if (!aeth_validate_buffer(key32, AETH_KEY_SIZE, AETH_KEY_SIZE)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    if (!aeth_validate_buffer(nonce24, AETH_NONCE_SIZE, AETH_NONCE_SIZE)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    if (!aeth_validate_buffer(pt, pt_len, AETH_MAX_PAYLOAD_SIZE)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    if (!aeth_validate_buffer(ct_out, pt_len, AETH_MAX_PAYLOAD_SIZE)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    if (!aeth_validate_buffer(tag16_out, AETH_TAG_SIZE, AETH_TAG_SIZE)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    
    /* Check for buffer overlaps */
    if (aeth_buffers_overlap(pt, pt_len, ct_out, pt_len)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    if (aeth_buffers_overlap(pt, pt_len, tag16_out, AETH_TAG_SIZE)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    if (aeth_buffers_overlap(ct_out, pt_len, tag16_out, AETH_TAG_SIZE)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    if (ad != NULL && aeth_buffers_overlap(ad, ad_len, ct_out, pt_len)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    if (ad != NULL && aeth_buffers_overlap(ad, ad_len, tag16_out, AETH_TAG_SIZE)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    
    /* Use libsodium's XChaCha20-Poly1305 */
    unsigned long long ciphertext_len;
    int result = crypto_aead_xchacha20poly1305_ietf_encrypt(
        ct_out,
        &ciphertext_len,
        pt,
        pt_len,
        ad,
        ad_len,
        NULL,  /* Additional nonce (not used in IETF variant) */
        nonce24,
        key32
    );
    
    if (result != 0) {
        return AETH_ERROR_CRYPTO_FAILURE;
    }
    
    /* Extract the tag from the end of the ciphertext */
    if (ciphertext_len < AETH_TAG_SIZE) {
        return AETH_ERROR_CRYPTO_FAILURE;
    }
    
    /* Copy the tag */
    memcpy(tag16_out, ct_out + pt_len, AETH_TAG_SIZE);
    
    return AETH_OK;
}

int aeth_xchacha20_open(
    const uint8_t* key32,
    const uint8_t* nonce24,
    const uint8_t* ad,
    size_t ad_len,
    const uint8_t* ct,
    size_t ct_len,
    const uint8_t* tag16,
    uint8_t* pt_out
) {
    /* Ensure libsodium is initialized */
    ensure_sodium_init();
    
    /* Input validation */
    if (!aeth_validate_buffer(key32, AETH_KEY_SIZE, AETH_KEY_SIZE)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    if (!aeth_validate_buffer(nonce24, AETH_NONCE_SIZE, AETH_NONCE_SIZE)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    if (!aeth_validate_buffer(ct, ct_len, AETH_MAX_PAYLOAD_SIZE)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    if (!aeth_validate_buffer(tag16, AETH_TAG_SIZE, AETH_TAG_SIZE)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    if (!aeth_validate_buffer(pt_out, ct_len, AETH_MAX_PAYLOAD_SIZE)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    
    /* Check for buffer overlaps */
    if (aeth_buffers_overlap(ct, ct_len, pt_out, ct_len)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    if (aeth_buffers_overlap(ct, ct_len, tag16, AETH_TAG_SIZE)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    if (aeth_buffers_overlap(pt_out, ct_len, tag16, AETH_TAG_SIZE)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    if (ad != NULL && aeth_buffers_overlap(ad, ad_len, pt_out, ct_len)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    if (ad != NULL && aeth_buffers_overlap(ad, ad_len, tag16, AETH_TAG_SIZE)) {
        return AETH_ERROR_INVALID_INPUT;
    }
    
    /* Reconstruct the full ciphertext with tag */
    uint8_t full_ct[AETH_MAX_PAYLOAD_SIZE + AETH_TAG_SIZE];
    if (ct_len + AETH_TAG_SIZE > sizeof(full_ct)) {
        return AETH_ERROR_BUFFER_TOO_SMALL;
    }
    
    memcpy(full_ct, ct, ct_len);
    memcpy(full_ct + ct_len, tag16, AETH_TAG_SIZE);
    
    /* Use libsodium's XChaCha20-Poly1305 */
    unsigned long long plaintext_len;
    int result = crypto_aead_xchacha20poly1305_ietf_decrypt(
        pt_out,
        &plaintext_len,
        NULL,  /* Additional nonce (not used in IETF variant) */
        full_ct,
        ct_len + AETH_TAG_SIZE,
        ad,
        ad_len,
        nonce24,
        key32
    );
    
    if (result != 0) {
        return AETH_ERROR_CRYPTO_FAILURE;
    }
    
    return AETH_OK;
}
