#include "wrapper.h"
#include <oqs/oqs.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>

// =============================================================================
// Common Utility Functions
// =============================================================================

void liboqs_zeroize(uint8_t* ptr, size_t len) {
    if (ptr != NULL && len > 0) {
        volatile uint8_t* volatile_ptr = ptr;
        for (size_t i = 0; i < len; i++) {
            volatile_ptr[i] = 0;
        }
    }
}

liboqs_result_t liboqs_random_bytes(uint8_t* buffer, size_t length) {
    if (buffer == NULL || length == 0) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    if (OQS_randombytes(buffer, length) != OQS_SUCCESS) {
        return LIBOQS_ERROR_CRYPTO;
    }
    
    return LIBOQS_SUCCESS;
}

int liboqs_constant_time_compare(const uint8_t* a, const uint8_t* b, size_t length) {
    if (a == NULL || b == NULL || length == 0) {
        return 0;
    }
    
    int result = 0;
    for (size_t i = 0; i < length; i++) {
        result |= a[i] ^ b[i];
    }
    
    return result == 0;
}

const char* liboqs_get_error_string(liboqs_result_t result) {
    switch (result) {
        case LIBOQS_SUCCESS:
            return "Success";
        case LIBOQS_ERROR:
            return "General error";
        case LIBOQS_ERROR_INVALID_PARAMETER:
            return "Invalid parameter";
        case LIBOQS_ERROR_MEMORY:
            return "Memory allocation error";
        case LIBOQS_ERROR_CRYPTO:
            return "Cryptographic operation failed";
        case LIBOQS_ERROR_VERIFICATION:
            return "Verification failed";
        default:
            return "Unknown error";
    }
}

liboqs_result_t liboqs_get_last_error(char* buffer, size_t buffer_len) {
    if (buffer == NULL || buffer_len == 0) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    // In a real implementation, you might store the last error
    strncpy(buffer, "No error details available", buffer_len - 1);
    buffer[buffer_len - 1] = '\0';
    
    return LIBOQS_SUCCESS;
}

// =============================================================================
// Kyber KEM Implementation
// =============================================================================

const char* kyber_get_algorithm_name(kyber_parameter_set_t params) {
    switch (params) {
        case KYBER_512:
            return "Kyber512";
        case KYBER_768:
            return "Kyber768";
        case KYBER_1024:
            return "Kyber1024";
        default:
            return "Unknown";
    }
}

size_t kyber_get_public_key_length(kyber_parameter_set_t params) {
    switch (params) {
        case KYBER_512:
            return OQS_KEM_kyber_512_length_public_key;
        case KYBER_768:
            return OQS_KEM_kyber_768_length_public_key;
        case KYBER_1024:
            return OQS_KEM_kyber_1024_length_public_key;
        default:
            return 0;
    }
}

size_t kyber_get_secret_key_length(kyber_parameter_set_t params) {
    switch (params) {
        case KYBER_512:
            return OQS_KEM_kyber_512_length_secret_key;
        case KYBER_768:
            return OQS_KEM_kyber_768_length_secret_key;
        case KYBER_1024:
            return OQS_KEM_kyber_1024_length_secret_key;
        default:
            return 0;
    }
}

size_t kyber_get_ciphertext_length(kyber_parameter_set_t params) {
    switch (params) {
        case KYBER_512:
            return OQS_KEM_kyber_512_length_ciphertext;
        case KYBER_768:
            return OQS_KEM_kyber_768_length_ciphertext;
        case KYBER_1024:
            return OQS_KEM_kyber_1024_length_ciphertext;
        default:
            return 0;
    }
}

size_t kyber_get_shared_secret_length(kyber_parameter_set_t params) {
    switch (params) {
        case KYBER_512:
            return OQS_KEM_kyber_512_length_shared_secret;
        case KYBER_768:
            return OQS_KEM_kyber_768_length_shared_secret;
        case KYBER_1024:
            return OQS_KEM_kyber_1024_length_shared_secret;
        default:
            return 0;
    }
}

int kyber_get_security_level(kyber_parameter_set_t params) {
    switch (params) {
        case KYBER_512:
            return 128;
        case KYBER_768:
            return 192;
        case KYBER_1024:
            return 256;
        default:
            return 0;
    }
}

liboqs_result_t kyber_keypair_generate(
    kyber_parameter_set_t params,
    kyber_public_key_t* public_key,
    kyber_secret_key_t* secret_key
) {
    if (public_key == NULL || secret_key == NULL) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    // Get algorithm name
    const char* alg_name = kyber_get_algorithm_name(params);
    if (strcmp(alg_name, "Unknown") == 0) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    // Get key lengths
    size_t pub_len = kyber_get_public_key_length(params);
    size_t sec_len = kyber_get_secret_key_length(params);
    
    if (pub_len == 0 || sec_len == 0) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    // Allocate memory
    public_key->key_data.data = malloc(pub_len);
    secret_key->key_data.data = malloc(sec_len);
    
    if (public_key->key_data.data == NULL || secret_key->key_data.data == NULL) {
        // Clean up on allocation failure
        if (public_key->key_data.data) {
            free(public_key->key_data.data);
            public_key->key_data.data = NULL;
        }
        if (secret_key->key_data.data) {
            free(secret_key->key_data.data);
            secret_key->key_data.data = NULL;
        }
        return LIBOQS_ERROR_MEMORY;
    }
    
    // Set metadata
    public_key->key_data.length = pub_len;
    public_key->key_data.is_allocated = 1;
    public_key->params = params;
    
    secret_key->key_data.length = sec_len;
    secret_key->key_data.is_allocated = 1;
    secret_key->params = params;
    
    // Generate keypair using liboqs
    OQS_KEM* kem = OQS_KEM_new(alg_name);
    if (kem == NULL) {
        kyber_public_key_free(public_key);
        kyber_secret_key_free(secret_key);
        return LIBOQS_ERROR_CRYPTO;
    }
    
    int result = OQS_KEM_keypair(kem, 
                                 public_key->key_data.data,
                                 secret_key->key_data.data);
    
    OQS_KEM_free(kem);
    
    if (result != OQS_SUCCESS) {
        kyber_public_key_free(public_key);
        kyber_secret_key_free(secret_key);
        return LIBOQS_ERROR_CRYPTO;
    }
    
    return LIBOQS_SUCCESS;
}

liboqs_result_t kyber_encapsulate(
    const kyber_public_key_t* public_key,
    kyber_ciphertext_t* ciphertext,
    kyber_shared_secret_t* shared_secret
) {
    if (public_key == NULL || ciphertext == NULL || shared_secret == NULL) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    if (public_key->key_data.data == NULL || public_key->key_data.length == 0) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    // Get algorithm name and lengths
    const char* alg_name = kyber_get_algorithm_name(public_key->params);
    size_t cipher_len = kyber_get_ciphertext_length(public_key->params);
    size_t secret_len = kyber_get_shared_secret_length(public_key->params);
    
    if (cipher_len == 0 || secret_len == 0) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    // Allocate memory
    ciphertext->ciphertext.data = malloc(cipher_len);
    shared_secret->shared_secret.data = malloc(secret_len);
    
    if (ciphertext->ciphertext.data == NULL || shared_secret->shared_secret.data == NULL) {
        // Clean up on allocation failure
        if (ciphertext->ciphertext.data) {
            free(ciphertext->ciphertext.data);
            ciphertext->ciphertext.data = NULL;
        }
        if (shared_secret->shared_secret.data) {
            free(shared_secret->shared_secret.data);
            shared_secret->shared_secret.data = NULL;
        }
        return LIBOQS_ERROR_MEMORY;
    }
    
    // Set metadata
    ciphertext->ciphertext.length = cipher_len;
    ciphertext->ciphertext.is_allocated = 1;
    ciphertext->params = public_key->params;
    
    shared_secret->shared_secret.length = secret_len;
    shared_secret->shared_secret.is_allocated = 1;
    shared_secret->params = public_key->params;
    
    // Perform encapsulation using liboqs
    OQS_KEM* kem = OQS_KEM_new(alg_name);
    if (kem == NULL) {
        kyber_ciphertext_free(ciphertext);
        kyber_shared_secret_free(shared_secret);
        return LIBOQS_ERROR_CRYPTO;
    }
    
    int result = OQS_KEM_encaps(kem,
                                ciphertext->ciphertext.data,
                                shared_secret->shared_secret.data,
                                public_key->key_data.data);
    
    OQS_KEM_free(kem);
    
    if (result != OQS_SUCCESS) {
        kyber_ciphertext_free(ciphertext);
        kyber_shared_secret_free(shared_secret);
        return LIBOQS_ERROR_CRYPTO;
    }
    
    return LIBOQS_SUCCESS;
}

liboqs_result_t kyber_decapsulate(
    const kyber_secret_key_t* secret_key,
    const kyber_ciphertext_t* ciphertext,
    kyber_shared_secret_t* shared_secret
) {
    if (secret_key == NULL || ciphertext == NULL || shared_secret == NULL) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    if (secret_key->key_data.data == NULL || secret_key->key_data.length == 0) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    if (ciphertext->ciphertext.data == NULL || ciphertext->ciphertext.length == 0) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    // Get algorithm name and secret length
    const char* alg_name = kyber_get_algorithm_name(secret_key->params);
    size_t secret_len = kyber_get_shared_secret_length(secret_key->params);
    
    if (secret_len == 0) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    // Allocate memory for shared secret
    shared_secret->shared_secret.data = malloc(secret_len);
    if (shared_secret->shared_secret.data == NULL) {
        return LIBOQS_ERROR_MEMORY;
    }
    
    // Set metadata
    shared_secret->shared_secret.length = secret_len;
    shared_secret->shared_secret.is_allocated = 1;
    shared_secret->params = secret_key->params;
    
    // Perform decapsulation using liboqs
    OQS_KEM* kem = OQS_KEM_new(alg_name);
    if (kem == NULL) {
        kyber_shared_secret_free(shared_secret);
        return LIBOQS_ERROR_CRYPTO;
    }
    
    int result = OQS_KEM_decaps(kem,
                                shared_secret->shared_secret.data,
                                ciphertext->ciphertext.data,
                                secret_key->key_data.data);
    
    OQS_KEM_free(kem);
    
    if (result != OQS_SUCCESS) {
        kyber_shared_secret_free(shared_secret);
        return LIBOQS_ERROR_CRYPTO;
    }
    
    return LIBOQS_SUCCESS;
}

void kyber_public_key_free(kyber_public_key_t* key) {
    if (key != NULL && key->key_data.data != NULL && key->key_data.is_allocated) {
        liboqs_zeroize(key->key_data.data, key->key_data.length);
        free(key->key_data.data);
        key->key_data.data = NULL;
        key->key_data.length = 0;
        key->key_data.is_allocated = 0;
    }
}

void kyber_secret_key_free(kyber_secret_key_t* key) {
    if (key != NULL && key->key_data.data != NULL && key->key_data.is_allocated) {
        liboqs_zeroize(key->key_data.data, key->key_data.length);
        free(key->key_data.data);
        key->key_data.data = NULL;
        key->key_data.length = 0;
        key->key_data.is_allocated = 0;
    }
}

void kyber_ciphertext_free(kyber_ciphertext_t* ciphertext) {
    if (ciphertext != NULL && ciphertext->ciphertext.data != NULL && ciphertext->ciphertext.is_allocated) {
        free(ciphertext->ciphertext.data);
        ciphertext->ciphertext.data = NULL;
        ciphertext->ciphertext.length = 0;
        ciphertext->ciphertext.is_allocated = 0;
    }
}

void kyber_shared_secret_free(kyber_shared_secret_t* shared_secret) {
    if (shared_secret != NULL && shared_secret->shared_secret.data != NULL && shared_secret->shared_secret.is_allocated) {
        liboqs_zeroize(shared_secret->shared_secret.data, shared_secret->shared_secret.length);
        free(shared_secret->shared_secret.data);
        shared_secret->shared_secret.data = NULL;
        shared_secret->shared_secret.length = 0;
        shared_secret->shared_secret.is_allocated = 0;
    }
}

// =============================================================================
// Dilithium Digital Signature Implementation
// =============================================================================

const char* dilithium_get_algorithm_name(dilithium_parameter_set_t params) {
    switch (params) {
        case DILITHIUM_2:
            return "Dilithium2";
        case DILITHIUM_3:
            return "Dilithium3";
        case DILITHIUM_5:
            return "Dilithium5";
        default:
            return "Unknown";
    }
}

size_t dilithium_get_public_key_length(dilithium_parameter_set_t params) {
    switch (params) {
        case DILITHIUM_2:
            return OQS_SIG_dilithium_2_length_public_key;
        case DILITHIUM_3:
            return OQS_SIG_dilithium_3_length_public_key;
        case DILITHIUM_5:
            return OQS_SIG_dilithium_5_length_public_key;
        default:
            return 0;
    }
}

size_t dilithium_get_secret_key_length(dilithium_parameter_set_t params) {
    switch (params) {
        case DILITHIUM_2:
            return OQS_SIG_dilithium_2_length_secret_key;
        case DILITHIUM_3:
            return OQS_SIG_dilithium_3_length_secret_key;
        case DILITHIUM_5:
            return OQS_SIG_dilithium_5_length_secret_key;
        default:
            return 0;
    }
}

size_t dilithium_get_signature_length(dilithium_parameter_set_t params) {
    switch (params) {
        case DILITHIUM_2:
            return OQS_SIG_dilithium_2_length_signature;
        case DILITHIUM_3:
            return OQS_SIG_dilithium_3_length_signature;
        case DILITHIUM_5:
            return OQS_SIG_dilithium_5_length_signature;
        default:
            return 0;
    }
}

int dilithium_get_security_level(dilithium_parameter_set_t params) {
    switch (params) {
        case DILITHIUM_2:
            return 128;
        case DILITHIUM_3:
            return 192;
        case DILITHIUM_5:
            return 256;
        default:
            return 0;
    }
}

liboqs_result_t dilithium_keypair_generate(
    dilithium_parameter_set_t params,
    dilithium_public_key_t* public_key,
    dilithium_secret_key_t* secret_key
) {
    if (public_key == NULL || secret_key == NULL) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    // Get algorithm name
    const char* alg_name = dilithium_get_algorithm_name(params);
    if (strcmp(alg_name, "Unknown") == 0) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    // Get key lengths
    size_t pub_len = dilithium_get_public_key_length(params);
    size_t sec_len = dilithium_get_secret_key_length(params);
    
    if (pub_len == 0 || sec_len == 0) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    // Allocate memory
    public_key->key_data.data = malloc(pub_len);
    secret_key->key_data.data = malloc(sec_len);
    
    if (public_key->key_data.data == NULL || secret_key->key_data.data == NULL) {
        // Clean up on allocation failure
        if (public_key->key_data.data) {
            free(public_key->key_data.data);
            public_key->key_data.data = NULL;
        }
        if (secret_key->key_data.data) {
            free(secret_key->key_data.data);
            secret_key->key_data.data = NULL;
        }
        return LIBOQS_ERROR_MEMORY;
    }
    
    // Set metadata
    public_key->key_data.length = pub_len;
    public_key->key_data.is_allocated = 1;
    public_key->params = params;
    
    secret_key->key_data.length = sec_len;
    secret_key->key_data.is_allocated = 1;
    secret_key->params = params;
    
    // Generate keypair using liboqs
    OQS_SIG* sig = OQS_SIG_new(alg_name);
    if (sig == NULL) {
        dilithium_public_key_free(public_key);
        dilithium_secret_key_free(secret_key);
        return LIBOQS_ERROR_CRYPTO;
    }
    
    int result = OQS_SIG_keypair(sig,
                                 public_key->key_data.data,
                                 secret_key->key_data.data);
    
    OQS_SIG_free(sig);
    
    if (result != OQS_SUCCESS) {
        dilithium_public_key_free(public_key);
        dilithium_secret_key_free(secret_key);
        return LIBOQS_ERROR_CRYPTO;
    }
    
    return LIBOQS_SUCCESS;
}

liboqs_result_t dilithium_sign(
    const dilithium_secret_key_t* secret_key,
    const uint8_t* message,
    size_t message_len,
    dilithium_signature_t* signature
) {
    if (secret_key == NULL || message == NULL || signature == NULL) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    if (secret_key->key_data.data == NULL || secret_key->key_data.length == 0) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    if (message_len == 0) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    // Get algorithm name and signature length
    const char* alg_name = dilithium_get_algorithm_name(secret_key->params);
    size_t sig_len = dilithium_get_signature_length(secret_key->params);
    
    if (sig_len == 0) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    // Allocate memory for signature
    signature->signature.data = malloc(sig_len);
    if (signature->signature.data == NULL) {
        return LIBOQS_ERROR_MEMORY;
    }
    
    // Set metadata
    signature->signature.length = sig_len;
    signature->signature.is_allocated = 1;
    signature->params = secret_key->params;
    
    // Perform signing using liboqs
    OQS_SIG* sig = OQS_SIG_new(alg_name);
    if (sig == NULL) {
        dilithium_signature_free(signature);
        return LIBOQS_ERROR_CRYPTO;
    }
    
    size_t sig_len_out = 0;
    int result = OQS_SIG_sign(sig,
                              signature->signature.data,
                              &sig_len_out,
                              message,
                              message_len,
                              secret_key->key_data.data);
    
    OQS_SIG_free(sig);
    
    if (result != OQS_SUCCESS) {
        dilithium_signature_free(signature);
        return LIBOQS_ERROR_CRYPTO;
    }
    
    // Verify signature length
    if (sig_len_out != sig_len) {
        dilithium_signature_free(signature);
        return LIBOQS_ERROR_CRYPTO;
    }
    
    return LIBOQS_SUCCESS;
}

liboqs_result_t dilithium_verify(
    const dilithium_public_key_t* public_key,
    const uint8_t* message,
    size_t message_len,
    const dilithium_signature_t* signature
) {
    if (public_key == NULL || message == NULL || signature == NULL) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    if (public_key->key_data.data == NULL || public_key->key_data.length == 0) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    if (signature->signature.data == NULL || signature->signature.length == 0) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    if (message_len == 0) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    // Get algorithm name
    const char* alg_name = dilithium_get_algorithm_name(public_key->params);
    
    // Perform verification using liboqs
    OQS_SIG* sig = OQS_SIG_new(alg_name);
    if (sig == NULL) {
        return LIBOQS_ERROR_CRYPTO;
    }
    
    int result = OQS_SIG_verify(sig,
                                message,
                                message_len,
                                signature->signature.data,
                                signature->signature.length,
                                public_key->key_data.data);
    
    OQS_SIG_free(sig);
    
    if (result != OQS_SUCCESS) {
        return LIBOQS_ERROR_VERIFICATION;
    }
    
    return LIBOQS_SUCCESS;
}

void dilithium_public_key_free(dilithium_public_key_t* key) {
    if (key != NULL && key->key_data.data != NULL && key->key_data.is_allocated) {
        free(key->key_data.data);
        key->key_data.data = NULL;
        key->key_data.length = 0;
        key->key_data.is_allocated = 0;
    }
}

void dilithium_secret_key_free(dilithium_secret_key_t* key) {
    if (key != NULL && key->key_data.data != NULL && key->key_data.is_allocated) {
        liboqs_zeroize(key->key_data.data, key->key_data.length);
        free(key->key_data.data);
        key->key_data.data = NULL;
        key->key_data.length = 0;
        key->key_data.is_allocated = 0;
    }
}

void dilithium_signature_free(dilithium_signature_t* signature) {
    if (signature != NULL && signature->signature.data != NULL && signature->signature.is_allocated) {
        free(signature->signature.data);
        signature->signature.data = NULL;
        signature->signature.length = 0;
        signature->signature.is_allocated = 0;
    }
}

// =============================================================================
// Test Vector Functions
// =============================================================================

liboqs_result_t kyber_get_test_vectors(
    kyber_parameter_set_t params,
    uint8_t* public_key,
    uint8_t* secret_key,
    uint8_t* ciphertext,
    uint8_t* shared_secret
) {
    // This is a placeholder for test vectors
    // In a real implementation, you would return known test vectors
    // from NIST or other standardization bodies
    
    if (public_key == NULL || secret_key == NULL || 
        ciphertext == NULL || shared_secret == NULL) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    // For now, return error indicating test vectors not implemented
    return LIBOQS_ERROR_CRYPTO;
}

liboqs_result_t dilithium_get_test_vectors(
    dilithium_parameter_set_t params,
    uint8_t* public_key,
    uint8_t* secret_key,
    uint8_t* message,
    size_t message_len,
    uint8_t* signature
) {
    // This is a placeholder for test vectors
    // In a real implementation, you would return known test vectors
    // from NIST or other standardization bodies
    
    if (public_key == NULL || secret_key == NULL || 
        message == NULL || signature == NULL) {
        return LIBOQS_ERROR_INVALID_PARAMETER;
    }
    
    // For now, return error indicating test vectors not implemented
    return LIBOQS_ERROR_CRYPTO;
}
