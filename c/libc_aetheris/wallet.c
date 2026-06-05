/*
 * Aetheris Wallet C Shim
 * Provides C interface for wallet operations in the Aetheris OS wallet system.
 */

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <string.h>
#include <stdlib.h>
#include <time.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Error codes */
typedef enum {
    WALLET_SUCCESS = 0,
    WALLET_ERROR_INVALID_PARAM = -1,
    WALLET_ERROR_INSUFFICIENT_MEMORY = -2,
    WALLET_ERROR_KEY_NOT_FOUND = -3,
    WALLET_ERROR_INVALID_KEY_TYPE = -4,
    WALLET_ERROR_CAPABILITY_DENIED = -5,
    WALLET_ERROR_RATE_LIMIT_EXCEEDED = -6,
    WALLET_ERROR_CRYPTO_ERROR = -7,
    WALLET_ERROR_SERIALIZATION_ERROR = -8,
    WALLET_ERROR_IO_ERROR = -9,
    WALLET_ERROR_INTERNAL_ERROR = -10
} wallet_error_t;

/* Key types */
typedef enum {
    WALLET_KEY_TYPE_SECP256K1 = 0,    /* Ethereum, Bitcoin */
    WALLET_KEY_TYPE_ED25519 = 1,      /* Solana, Cardano */
    WALLET_KEY_TYPE_SR25519 = 2,      /* Polkadot, Substrate */
    WALLET_KEY_TYPE_X25519 = 3,       /* Key exchange */
    WALLET_KEY_TYPE_KYBER512 = 4,     /* PQC KEM */
    WALLET_KEY_TYPE_KYBER768 = 5,     /* PQC KEM */
    WALLET_KEY_TYPE_KYBER1024 = 6,    /* PQC KEM */
    WALLET_KEY_TYPE_DILITHIUM2 = 7,   /* PQC Signature */
    WALLET_KEY_TYPE_DILITHIUM3 = 8,   /* PQC Signature */
    WALLET_KEY_TYPE_DILITHIUM5 = 9    /* PQC Signature */
} wallet_key_type_t;

/* Key purposes */
typedef enum {
    WALLET_KEY_PURPOSE_SIGN = 0,
    WALLET_KEY_PURPOSE_VERIFY = 1,
    WALLET_KEY_PURPOSE_ENCRYPT = 2,
    WALLET_KEY_PURPOSE_DECRYPT = 3,
    WALLET_KEY_PURPOSE_KEY_ENCAPSULATION = 4,
    WALLET_KEY_PURPOSE_KEY_DECAPSULATION = 5,
    WALLET_KEY_PURPOSE_KEY_EXCHANGE = 6,
    WALLET_KEY_PURPOSE_AUTHENTICATION = 7
} wallet_key_purpose_t;

/* DID methods */
typedef enum {
    WALLET_DID_METHOD_KEY = 0,
    WALLET_DID_METHOD_PKH = 1,
    WALLET_DID_METHOD_WEB = 2
} wallet_did_method_t;

/* Structures */
typedef struct {
    char wallet_id[37];  /* UUID string */
    char did[256];       /* DID string */
    char pdv_location[512]; /* PDV location path */
    uint64_t created_at; /* Unix timestamp */
} wallet_init_result_t;

typedef struct {
    char key_id[37];     /* UUID string */
    wallet_key_type_t key_type;
    uint8_t *public_key; /* Public key data */
    size_t public_key_len;
    char did_binding[256]; /* DID binding string */
    char pdv_item_id[512]; /* PDV item ID */
    char derivation_path[128]; /* Derivation path string */
} wallet_key_gen_result_t;

typedef struct {
    uint8_t *signature;  /* Signature data */
    size_t signature_len;
    uint8_t recovery_id; /* Recovery ID (0-3) */
    uint8_t *pqc_signature; /* PQC signature data */
    size_t pqc_signature_len;
    char algorithm[64];  /* Algorithm name */
} wallet_sign_result_t;

typedef struct {
    char export_type[32]; /* Export type */
    uint8_t *export_data; /* Export data */
    size_t export_data_len;
} wallet_export_result_t;

typedef struct {
    char key_id[37];     /* UUID string */
    wallet_key_type_t key_type;
    uint8_t purposes[8]; /* Array of purposes */
    size_t purposes_count;
    uint64_t created_at; /* Unix timestamp */
    char did_binding[256]; /* DID binding string */
} wallet_key_info_t;

typedef struct {
    wallet_key_info_t *keys; /* Array of key info */
    size_t keys_count;
    size_t total_keys;
} wallet_key_list_t;

/* Configuration structure */
typedef struct {
    char pdv_path[512];  /* PDV path */
    uint32_t max_keys_per_wallet;
    uint32_t keygen_rate_limit;
    uint32_t audit_retention_days;
    bool pqc_enabled;
    bool hsm_enabled;
} wallet_config_t;

/* Function declarations */

/**
 * Initialize wallet service
 * @param config Configuration structure
 * @return WALLET_SUCCESS on success, error code on failure
 */
wallet_error_t wallet_init(const wallet_config_t *config);

/**
 * Shutdown wallet service
 * @return WALLET_SUCCESS on success, error code on failure
 */
wallet_error_t wallet_shutdown(void);

/**
 * Initialize a new wallet
 * @param subject Subject for capability checking
 * @param intent_id Intent ID for audit trail (can be NULL)
 * @param result Output wallet initialization result
 * @return WALLET_SUCCESS on success, error code on failure
 */
wallet_error_t wallet_init_wallet(
    const char *subject,
    const char *intent_id,
    wallet_init_result_t *result
);

/**
 * Generate a new key
 * @param subject Subject for capability checking
 * @param wallet_id Wallet ID
 * @param key_type Key type to generate
 * @param intent_id Intent ID for audit trail (can be NULL)
 * @param result Output key generation result
 * @return WALLET_SUCCESS on success, error code on failure
 */
wallet_error_t wallet_generate_key(
    const char *subject,
    const char *wallet_id,
    wallet_key_type_t key_type,
    const char *intent_id,
    wallet_key_gen_result_t *result
);

/**
 * Sign data with a key
 * @param subject Subject for capability checking
 * @param wallet_id Wallet ID
 * @param key_id Key ID to use for signing
 * @param data Data to sign
 * @param data_len Length of data
 * @param intent_id Intent ID for audit trail (can be NULL)
 * @param hybrid_pqc Use hybrid PQC signing
 * @param result Output signature result
 * @return WALLET_SUCCESS on success, error code on failure
 */
wallet_error_t wallet_sign(
    const char *subject,
    const char *wallet_id,
    const char *key_id,
    const uint8_t *data,
    size_t data_len,
    const char *intent_id,
    bool hybrid_pqc,
    wallet_sign_result_t *result
);

/**
 * Export a key
 * @param subject Subject for capability checking
 * @param wallet_id Wallet ID
 * @param key_id Key ID to export
 * @param intent_id Intent ID for audit trail (can be NULL)
 * @param public_only Export only public key
 * @param result Output export result
 * @return WALLET_SUCCESS on success, error code on failure
 */
wallet_error_t wallet_export_key(
    const char *subject,
    const char *wallet_id,
    const char *key_id,
    const char *intent_id,
    bool public_only,
    wallet_export_result_t *result
);

/**
 * List keys in a wallet
 * @param subject Subject for capability checking
 * @param wallet_id Wallet ID
 * @param intent_id Intent ID for audit trail (can be NULL)
 * @param result Output key list result
 * @return WALLET_SUCCESS on success, error code on failure
 */
wallet_error_t wallet_list_keys(
    const char *subject,
    const char *wallet_id,
    const char *intent_id,
    wallet_key_list_t *result
);

/**
 * Derive a key from a parent key
 * @param subject Subject for capability checking
 * @param wallet_id Wallet ID
 * @param parent_key_id Parent key ID
 * @param derivation_path Derivation path string
 * @param intent_id Intent ID for audit trail (can be NULL)
 * @param result Output key generation result
 * @return WALLET_SUCCESS on success, error code on failure
 */
wallet_error_t wallet_derive_key(
    const char *subject,
    const char *wallet_id,
    const char *parent_key_id,
    const char *derivation_path,
    const char *intent_id,
    wallet_key_gen_result_t *result
);

/**
 * Import an encrypted key
 * @param subject Subject for capability checking
 * @param wallet_id Wallet ID
 * @param encrypted_key Encrypted key data
 * @param encrypted_key_len Length of encrypted key data
 * @param intent_id Intent ID for audit trail (can be NULL)
 * @param result Output key generation result
 * @return WALLET_SUCCESS on success, error code on failure
 */
wallet_error_t wallet_import_key(
    const char *subject,
    const char *wallet_id,
    const uint8_t *encrypted_key,
    size_t encrypted_key_len,
    const char *intent_id,
    wallet_key_gen_result_t *result
);

/**
 * Revoke a key
 * @param subject Subject for capability checking
 * @param wallet_id Wallet ID
 * @param key_id Key ID to revoke
 * @param intent_id Intent ID for audit trail (can be NULL)
 * @return WALLET_SUCCESS on success, error code on failure
 */
wallet_error_t wallet_revoke_key(
    const char *subject,
    const char *wallet_id,
    const char *key_id,
    const char *intent_id
);

/**
 * Create a new DID
 * @param subject Subject for capability checking
 * @param method DID method
 * @param key_id Key ID to bind (can be NULL)
 * @param intent_id Intent ID for audit trail (can be NULL)
 * @param did Output DID string
 * @param did_len Length of DID buffer
 * @return WALLET_SUCCESS on success, error code on failure
 */
wallet_error_t wallet_create_did(
    const char *subject,
    wallet_did_method_t method,
    const char *key_id,
    const char *intent_id,
    char *did,
    size_t did_len
);

/**
 * Resolve a DID
 * @param subject Subject for capability checking
 * @param did DID to resolve
 * @param intent_id Intent ID for audit trail (can be NULL)
 * @param did_document Output DID document JSON
 * @param did_document_len Length of DID document buffer
 * @return WALLET_SUCCESS on success, error code on failure
 */
wallet_error_t wallet_resolve_did(
    const char *subject,
    const char *did,
    const char *intent_id,
    char *did_document,
    size_t did_document_len
);

/**
 * List vault items
 * @param subject Subject for capability checking
 * @param wallet_id Wallet ID
 * @param intent_id Intent ID for audit trail (can be NULL)
 * @param items Output array of item IDs
 * @param items_count Number of items
 * @param max_items Maximum number of items to return
 * @return WALLET_SUCCESS on success, error code on failure
 */
wallet_error_t wallet_list_vault_items(
    const char *subject,
    const char *wallet_id,
    const char *intent_id,
    char **items,
    size_t *items_count,
    size_t max_items
);

/**
 * Get audit log entries
 * @param subject Subject for capability checking
 * @param limit Maximum number of entries to return (0 for all)
 * @param entries Output array of audit entries
 * @param entries_count Number of entries
 * @param max_entries Maximum number of entries to return
 * @return WALLET_SUCCESS on success, error code on failure
 */
wallet_error_t wallet_get_audit_log(
    const char *subject,
    size_t limit,
    char **entries,
    size_t *entries_count,
    size_t max_entries
);

/**
 * Free memory allocated by wallet functions
 * @param ptr Pointer to free
 */
void wallet_free(void *ptr);

/**
 * Get error message for error code
 * @param error Error code
 * @return Error message string
 */
const char *wallet_error_message(wallet_error_t error);

/**
 * Get key type name
 * @param key_type Key type
 * @return Key type name string
 */
const char *wallet_key_type_name(wallet_key_type_t key_type);

/**
 * Get expected public key size for key type
 * @param key_type Key type
 * @return Expected public key size in bytes
 */
size_t wallet_key_type_size(wallet_key_type_t key_type);

/**
 * Validate key type and purpose compatibility
 * @param key_type Key type
 * @param purpose Key purpose
 * @return true if compatible, false otherwise
 */
bool wallet_validate_key_purpose(wallet_key_type_t key_type, wallet_key_purpose_t purpose);

/**
 * Compute address from public key
 * @param key_type Key type
 * @param public_key Public key data
 * @param public_key_len Length of public key data
 * @param address Output address string
 * @param address_len Length of address buffer
 * @return WALLET_SUCCESS on success, error code on failure
 */
wallet_error_t wallet_compute_address(
    wallet_key_type_t key_type,
    const uint8_t *public_key,
    size_t public_key_len,
    char *address,
    size_t address_len
);

/**
 * Validate derivation path
 * @param path Derivation path string
 * @return true if valid, false otherwise
 */
bool wallet_validate_derivation_path(const char *path);

/**
 * Check if derivation path should be persisted
 * @param path Derivation path string
 * @return true if should be persisted, false otherwise
 */
bool wallet_should_persist_derivation_path(const char *path);

/* Utility functions */

/**
 * Generate UUID string
 * @param uuid Output UUID string buffer
 * @param uuid_len Length of UUID buffer (should be at least 37)
 * @return WALLET_SUCCESS on success, error code on failure
 */
wallet_error_t wallet_generate_uuid(char *uuid, size_t uuid_len);

/**
 * Convert bytes to hex string
 * @param data Data to convert
 * @param data_len Length of data
 * @param hex Output hex string buffer
 * @param hex_len Length of hex buffer
 * @return WALLET_SUCCESS on success, error code on failure
 */
wallet_error_t wallet_bytes_to_hex(
    const uint8_t *data,
    size_t data_len,
    char *hex,
    size_t hex_len
);

/**
 * Convert hex string to bytes
 * @param hex Hex string to convert
 * @param data Output data buffer
 * @param data_len Length of data buffer
 * @param actual_len Actual length of converted data
 * @return WALLET_SUCCESS on success, error code on failure
 */
wallet_error_t wallet_hex_to_bytes(
    const char *hex,
    uint8_t *data,
    size_t data_len,
    size_t *actual_len
);

/**
 * Get current timestamp
 * @return Current Unix timestamp
 */
uint64_t wallet_get_timestamp(void);

/**
 * Validate DID format
 * @param did DID string to validate
 * @return true if valid, false otherwise
 */
bool wallet_validate_did_format(const char *did);

/**
 * Validate address format
 * @param key_type Key type
 * @param address Address string to validate
 * @return true if valid, false otherwise
 */
bool wallet_validate_address_format(wallet_key_type_t key_type, const char *address);

#ifdef __cplusplus
}
#endif
