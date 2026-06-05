//! PQC TLS C FFI header for Aetheris OS
//! 
//! This header defines the C FFI interface for Post-Quantum Cryptography TLS
//! and mutual TLS functionality.

#ifndef AETHERIS_PQC_TLS_V1_H
#define AETHERIS_PQC_TLS_V1_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include <time.h>

#ifdef __cplusplus
extern "C" {
#endif

// Error codes
typedef enum {
    PQC_TLS_SUCCESS = 0,
    PQC_TLS_ERROR_INVALID_PARAMETER = -1,
    PQC_TLS_ERROR_OUT_OF_MEMORY = -2,
    PQC_TLS_ERROR_UNSUPPORTED_ALGORITHM = -3,
    PQC_TLS_ERROR_BUFFER_TOO_SMALL = -4,
    PQC_TLS_ERROR_CERTIFICATE_NOT_FOUND = -5,
    PQC_TLS_ERROR_CERTIFICATE_EXPIRED = -6,
    PQC_TLS_ERROR_CERTIFICATE_REVOKED = -7,
    PQC_TLS_ERROR_SIGNATURE_VERIFICATION_FAILED = -8,
    PQC_TLS_ERROR_HANDSHAKE_FAILED = -9,
    PQC_TLS_ERROR_SESSION_NOT_FOUND = -10,
    PQC_TLS_ERROR_KEY_NOT_FOUND = -11,
    PQC_TLS_ERROR_OPERATION_NOT_SUPPORTED = -12
} pqc_tls_error_t;

// PQC Algorithm types
typedef enum {
    PQC_ALGORITHM_KYBER512 = 0,
    PQC_ALGORITHM_KYBER768 = 1,
    PQC_ALGORITHM_KYBER1024 = 2,
    PQC_ALGORITHM_DILITHIUM2 = 3,
    PQC_ALGORITHM_DILITHIUM3 = 4,
    PQC_ALGORITHM_DILITHIUM5 = 5,
    PQC_ALGORITHM_FALCON512 = 6,
    PQC_ALGORITHM_FALCON1024 = 7,
    PQC_ALGORITHM_SPHINCS_PLUS_128F = 8,
    PQC_ALGORITHM_SPHINCS_PLUS_192F = 9,
    PQC_ALGORITHM_SPHINCS_PLUS_256F = 10
} pqc_algorithm_t;

// PQC Key Usage types
typedef enum {
    PQC_KEY_USAGE_KEY_ENCAPSULATION = 0,
    PQC_KEY_USAGE_DIGITAL_SIGNATURE = 1,
    PQC_KEY_USAGE_BOTH = 2
} pqc_key_usage_t;

// TLS Version types
typedef enum {
    TLS_VERSION_1_2 = 2,
    TLS_VERSION_1_3 = 3
} tls_version_t;

// Cipher Suite types
typedef enum {
    CIPHER_SUITE_AES_128_GCM = 0,
    CIPHER_SUITE_AES_256_GCM = 1,
    CIPHER_SUITE_CHACHA20_POLY1305 = 2,
    CIPHER_SUITE_KYBER512 = 3,
    CIPHER_SUITE_DILITHIUM2 = 4,
    CIPHER_SUITE_FALCON512 = 5,
    CIPHER_SUITE_HYBRID_AES_KYBER = 6,
    CIPHER_SUITE_HYBRID_CHACHA20_DILITHIUM = 7
} cipher_suite_t;

// Forward declarations
typedef struct pqc_tls_manager pqc_tls_manager_t;
typedef struct pqc_key_pair pqc_key_pair_t;
typedef struct did_certificate did_certificate_t;
typedef struct tls_session tls_session_t;

// PQC TLS Manager structure
struct pqc_tls_manager {
    char id[256];
    // Additional fields would be opaque in real implementation
};

// PQC Key Pair structure
struct pqc_key_pair {
    char id[256];
    pqc_algorithm_t algorithm;
    uint8_t *public_key;
    size_t public_key_len;
    char private_key_id[256];
    uint64_t created_at;
    uint64_t expires_at;
    pqc_key_usage_t usage;
};

// DID Certificate structure
struct did_certificate {
    char did[256];
    uint8_t *certificate_data;
    size_t certificate_len;
    uint8_t *pqc_signature;
    size_t pqc_signature_len;
    char verification_method[256];
    uint64_t issued_at;
    uint64_t expires_at;
    bool revoked;
};

// TLS Session structure
struct tls_session {
    char session_id[256];
    uint8_t state;
    did_certificate_t *local_cert;
    did_certificate_t *peer_cert;
    uint64_t created_at;
    uint64_t last_activity;
    uint64_t timeout;
    uint64_t bytes_sent;
    uint64_t bytes_received;
};

// TLS Handshake Result structure
typedef struct {
    bool success;
    const char *error_message;
    uint32_t handshake_time_ms;
    cipher_suite_t cipher_suite;
    tls_version_t tls_version;
} pqc_tls_handshake_result_t;

// TLS Manager Statistics structure
typedef struct {
    uint32_t total_configs;
    uint32_t active_sessions;
    uint64_t total_bytes_sent;
    uint64_t total_bytes_received;
    uint64_t total_handshakes;
} pqc_tls_manager_stats_t;

// Function declarations

/**
 * Initialize PQC TLS manager
 * @param manager Pointer to manager structure
 * @return PQC_TLS_SUCCESS on success, error code on failure
 */
int pqc_tls_manager_init(pqc_tls_manager_t *manager);

/**
 * Cleanup PQC TLS manager
 * @param manager Pointer to manager structure
 * @return PQC_TLS_SUCCESS on success, error code on failure
 */
int pqc_tls_manager_cleanup(pqc_tls_manager_t *manager);

/**
 * Generate PQC key pair
 * @param manager Pointer to manager structure
 * @param algorithm PQC algorithm to use
 * @param usage Key usage (encapsulation, signature, or both)
 * @param key_pair Pointer to key pair structure to populate
 * @return PQC_TLS_SUCCESS on success, error code on failure
 */
int pqc_tls_generate_key_pair(
    pqc_tls_manager_t *manager,
    pqc_algorithm_t algorithm,
    pqc_key_usage_t usage,
    pqc_key_pair_t *key_pair
);

/**
 * Free PQC key pair resources
 * @param key_pair Pointer to key pair structure
 * @return PQC_TLS_SUCCESS on success, error code on failure
 */
int pqc_tls_free_key_pair(pqc_key_pair_t *key_pair);

/**
 * Create DID-bound certificate
 * @param manager Pointer to manager structure
 * @param did DID identifier
 * @param subject Certificate subject
 * @param validity_days Certificate validity in days
 * @param key_pair PQC key pair for signing
 * @param certificate Pointer to certificate structure to populate
 * @return PQC_TLS_SUCCESS on success, error code on failure
 */
int pqc_tls_create_did_certificate(
    pqc_tls_manager_t *manager,
    const char *did,
    const char *subject,
    uint32_t validity_days,
    const pqc_key_pair_t *key_pair,
    did_certificate_t *certificate
);

/**
 * Free DID certificate resources
 * @param certificate Pointer to certificate structure
 * @return PQC_TLS_SUCCESS on success, error code on failure
 */
int pqc_tls_free_did_certificate(did_certificate_t *certificate);

/**
 * Create TLS session
 * @param manager Pointer to manager structure
 * @param session_id Unique session identifier
 * @param session Pointer to session structure to populate
 * @return PQC_TLS_SUCCESS on success, error code on failure
 */
int pqc_tls_create_session(
    pqc_tls_manager_t *manager,
    const char *session_id,
    tls_session_t *session
);

/**
 * Perform mTLS handshake with PQC
 * @param manager Pointer to manager structure
 * @param session_id Session identifier
 * @param client_did Client DID identifier
 * @param server_did Server DID identifier
 * @param result Pointer to handshake result structure
 * @return PQC_TLS_SUCCESS on success, error code on failure
 */
int pqc_tls_mtls_handshake(
    pqc_tls_manager_t *manager,
    const char *session_id,
    const char *client_did,
    const char *server_did,
    pqc_tls_handshake_result_t *result
);

/**
 * Encrypt data using TLS session
 * @param manager Pointer to manager structure
 * @param session_id Session identifier
 * @param data Data to encrypt
 * @param data_len Length of data
 * @param encrypted_data Buffer for encrypted data
 * @param encrypted_len Pointer to encrypted data length
 * @return PQC_TLS_SUCCESS on success, error code on failure
 */
int pqc_tls_encrypt_data(
    pqc_tls_manager_t *manager,
    const char *session_id,
    const uint8_t *data,
    size_t data_len,
    uint8_t *encrypted_data,
    size_t *encrypted_len
);

/**
 * Decrypt data using TLS session
 * @param manager Pointer to manager structure
 * @param session_id Session identifier
 * @param encrypted_data Encrypted data
 * @param encrypted_len Length of encrypted data
 * @param data Buffer for decrypted data
 * @param data_len Pointer to decrypted data length
 * @return PQC_TLS_SUCCESS on success, error code on failure
 */
int pqc_tls_decrypt_data(
    pqc_tls_manager_t *manager,
    const char *session_id,
    const uint8_t *encrypted_data,
    size_t encrypted_len,
    uint8_t *data,
    size_t *data_len
);

/**
 * Rekey session with new PQC algorithm
 * @param manager Pointer to manager structure
 * @param session_id Session identifier
 * @param new_algorithm New PQC algorithm
 * @return PQC_TLS_SUCCESS on success, error code on failure
 */
int pqc_tls_rekey_session(
    pqc_tls_manager_t *manager,
    const char *session_id,
    pqc_algorithm_t new_algorithm
);

/**
 * Close TLS session
 * @param manager Pointer to manager structure
 * @param session_id Session identifier
 * @return PQC_TLS_SUCCESS on success, error code on failure
 */
int pqc_tls_close_session(
    pqc_tls_manager_t *manager,
    const char *session_id
);

/**
 * Get manager statistics
 * @param manager Pointer to manager structure
 * @param stats Pointer to statistics structure
 * @return PQC_TLS_SUCCESS on success, error code on failure
 */
int pqc_tls_get_manager_stats(
    pqc_tls_manager_t *manager,
    pqc_tls_manager_stats_t *stats
);

/**
 * Verify PQC signature
 * @param manager Pointer to manager structure
 * @param data Data that was signed
 * @param data_len Length of data
 * @param signature Signature to verify
 * @param signature_len Length of signature
 * @param public_key Public key for verification
 * @param public_key_len Length of public key
 * @param algorithm PQC algorithm used
 * @param is_valid Pointer to boolean result
 * @return PQC_TLS_SUCCESS on success, error code on failure
 */
int pqc_tls_verify_signature(
    pqc_tls_manager_t *manager,
    const uint8_t *data,
    size_t data_len,
    const uint8_t *signature,
    size_t signature_len,
    const uint8_t *public_key,
    size_t public_key_len,
    pqc_algorithm_t algorithm,
    bool *is_valid
);

/**
 * Sign data with PQC key
 * @param manager Pointer to manager structure
 * @param data Data to sign
 * @param data_len Length of data
 * @param private_key_id Private key identifier
 * @param signature Buffer for signature
 * @param signature_len Pointer to signature length
 * @param algorithm PQC algorithm to use
 * @return PQC_TLS_SUCCESS on success, error code on failure
 */
int pqc_tls_sign_data(
    pqc_tls_manager_t *manager,
    const uint8_t *data,
    size_t data_len,
    const char *private_key_id,
    uint8_t *signature,
    size_t *signature_len,
    pqc_algorithm_t algorithm
);

#ifdef __cplusplus
}
#endif

#endif // AETHERIS_PQC_TLS_V1_H
