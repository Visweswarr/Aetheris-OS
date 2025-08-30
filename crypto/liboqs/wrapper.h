#ifndef LIBOQS_WRAPPER_H
#define LIBOQS_WRAPPER_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// =============================================================================
// Common Types and Constants
// =============================================================================

/// Result codes for operations
typedef enum {
    LIBOQS_SUCCESS = 0,
    LIBOQS_ERROR = -1,
    LIBOQS_ERROR_INVALID_PARAMETER = -2,
    LIBOQS_ERROR_MEMORY = -3,
    LIBOQS_ERROR_CRYPTO = -4,
    LIBOQS_ERROR_VERIFICATION = -5
} liboqs_result_t;

/// Buffer structure for safe memory management
typedef struct {
    uint8_t* data;
    size_t length;
    int is_allocated;
} liboqs_buffer_t;

// =============================================================================
// Kyber KEM (Key Encapsulation Mechanism)
// =============================================================================

/// Kyber parameter sets
typedef enum {
    KYBER_512 = 0,
    KYBER_768 = 1,
    KYBER_1024 = 2
} kyber_parameter_set_t;

/// Kyber public key structure
typedef struct {
    liboqs_buffer_t key_data;
    kyber_parameter_set_t params;
} kyber_public_key_t;

/// Kyber secret key structure
typedef struct {
    liboqs_buffer_t key_data;
    kyber_parameter_set_t params;
} kyber_secret_key_t;

/// Kyber ciphertext structure
typedef struct {
    liboqs_buffer_t ciphertext;
    kyber_parameter_set_t params;
} kyber_ciphertext_t;

/// Kyber shared secret structure
typedef struct {
    liboqs_buffer_t shared_secret;
    kyber_parameter_set_t params;
} kyber_shared_secret_t;

// Kyber KEM functions
liboqs_result_t kyber_keypair_generate(
    kyber_parameter_set_t params,
    kyber_public_key_t* public_key,
    kyber_secret_key_t* secret_key
);

liboqs_result_t kyber_encapsulate(
    const kyber_public_key_t* public_key,
    kyber_ciphertext_t* ciphertext,
    kyber_shared_secret_t* shared_secret
);

liboqs_result_t kyber_decapsulate(
    const kyber_secret_key_t* secret_key,
    const kyber_ciphertext_t* ciphertext,
    kyber_shared_secret_t* shared_secret
);

// Memory management for Kyber
void kyber_public_key_free(kyber_public_key_t* key);
void kyber_secret_key_free(kyber_secret_key_t* key);
void kyber_ciphertext_free(kyber_ciphertext_t* ciphertext);
void kyber_shared_secret_free(kyber_shared_secret_t* shared_secret);

// =============================================================================
// Dilithium Digital Signatures
// =============================================================================

/// Dilithium parameter sets
typedef enum {
    DILITHIUM_2 = 0,
    DILITHIUM_3 = 1,
    DILITHIUM_5 = 2
} dilithium_parameter_set_t;

/// Dilithium public key structure
typedef struct {
    liboqs_buffer_t key_data;
    dilithium_parameter_set_t params;
} dilithium_public_key_t;

/// Dilithium secret key structure
typedef struct {
    liboqs_buffer_t key_data;
    dilithium_parameter_set_t params;
} dilithium_secret_key_t;

/// Dilithium signature structure
typedef struct {
    liboqs_buffer_t signature;
    dilithium_parameter_set_t params;
} dilithium_signature_t;

// Dilithium signature functions
liboqs_result_t dilithium_keypair_generate(
    dilithium_parameter_set_t params,
    dilithium_public_key_t* public_key,
    dilithium_secret_key_t* secret_key
);

liboqs_result_t dilithium_sign(
    const dilithium_secret_key_t* secret_key,
    const uint8_t* message,
    size_t message_len,
    dilithium_signature_t* signature
);

liboqs_result_t dilithium_verify(
    const dilithium_public_key_t* public_key,
    const uint8_t* message,
    size_t message_len,
    const dilithium_signature_t* signature
);

// Memory management for Dilithium
void dilithium_public_key_free(dilithium_public_key_t* key);
void dilithium_secret_key_free(dilithium_secret_key_t* key);
void dilithium_signature_free(dilithium_signature_t* signature);

// =============================================================================
// Utility Functions
// =============================================================================

/// Get algorithm name for parameter set
const char* kyber_get_algorithm_name(kyber_parameter_set_t params);
const char* dilithium_get_algorithm_name(dilithium_parameter_set_t params);

/// Get key sizes for parameter set
size_t kyber_get_public_key_length(kyber_parameter_set_t params);
size_t kyber_get_secret_key_length(kyber_parameter_set_t params);
size_t kyber_get_ciphertext_length(kyber_parameter_set_t params);
size_t kyber_get_shared_secret_length(kyber_parameter_set_t params);

size_t dilithium_get_public_key_length(dilithium_parameter_set_t params);
size_t dilithium_get_secret_key_length(dilithium_parameter_set_t params);
size_t dilithium_get_signature_length(dilithium_parameter_set_t params);

/// Get security level in bits
int kyber_get_security_level(kyber_parameter_set_t params);
int dilithium_get_security_level(dilithium_parameter_set_t params);

/// Zeroize buffer (secure memory clearing)
void liboqs_zeroize(uint8_t* ptr, size_t len);

/// Generate random bytes
liboqs_result_t liboqs_random_bytes(uint8_t* buffer, size_t length);

/// Compare buffers in constant time
int liboqs_constant_time_compare(
    const uint8_t* a,
    const uint8_t* b,
    size_t length
);

// =============================================================================
// Test Vector Functions
// =============================================================================

/// Get test vectors for known-answer tests
liboqs_result_t kyber_get_test_vectors(
    kyber_parameter_set_t params,
    uint8_t* public_key,
    uint8_t* secret_key,
    uint8_t* ciphertext,
    uint8_t* shared_secret
);

liboqs_result_t dilithium_get_test_vectors(
    dilithium_parameter_set_t params,
    uint8_t* public_key,
    uint8_t* secret_key,
    uint8_t* message,
    size_t message_len,
    uint8_t* signature
);

// =============================================================================
// Error Handling
// =============================================================================

/// Get error string for result code
const char* liboqs_get_error_string(liboqs_result_t result);

/// Get last error details
liboqs_result_t liboqs_get_last_error(char* buffer, size_t buffer_len);

#ifdef __cplusplus
}
#endif

#endif // LIBOQS_WRAPPER_H
