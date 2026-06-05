#ifndef LIBAETH_WALLET_H
#define LIBAETH_WALLET_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stddef.h>

// FFI result type
typedef struct {
    int success;
    char* error_message;
    void* data;
    size_t data_len;
} aeth_wallet_result_t;

// FFI wallet handle
typedef struct {
    char* wallet_id;
    char* did;
    char* pdv_location;
} aeth_wallet_t;

// FFI key handle
typedef struct {
    char* key_id;
    int key_type;
    void* public_key;
    size_t public_key_len;
    char* did_binding;
} aeth_key_t;

// FFI signature result
typedef struct {
    void* signature;
    size_t signature_len;
    int recovery_id;
    void* pqc_signature;
    size_t pqc_signature_len;
} aeth_signature_t;

// Key types
typedef enum {
    AETH_KEY_TYPE_SECP256K1 = 0,
    AETH_KEY_TYPE_ED25519 = 1,
    AETH_KEY_TYPE_SR25519 = 2,
    AETH_KEY_TYPE_X25519 = 3,
    AETH_KEY_TYPE_KYBER = 4,
    AETH_KEY_TYPE_DILITHIUM = 5
} aeth_key_type_t;

// DID methods
typedef enum {
    AETH_DID_METHOD_KEY = 0,
    AETH_DID_METHOD_PKH = 1,
    AETH_DID_METHOD_WEB = 2
} aeth_did_method_t;

// Initialize the wallet service
int aeth_wallet_init(
    const char* pdv_path,
    aeth_wallet_result_t* result
);

// Create a new wallet
int aeth_wallet_create(
    const char* subject,
    const char* intent_id,
    aeth_wallet_result_t* result
);

// Generate a new key
int aeth_key_generate(
    const char* subject,
    const char* wallet_id,
    aeth_key_type_t key_type,
    const char* intent_id,
    aeth_wallet_result_t* result
);

// Derive a key
int aeth_key_derive(
    const char* subject,
    const char* wallet_id,
    const char* parent_key_id,
    const char* derivation_path,
    const char* intent_id,
    aeth_wallet_result_t* result
);

// Sign data
int aeth_sign(
    const char* subject,
    const char* wallet_id,
    const char* key_id,
    const void* data,
    size_t data_len,
    const char* intent_id,
    int hybrid_pqc,
    aeth_wallet_result_t* result
);

// Resolve a DID
int aeth_did_resolve(
    const char* subject,
    const char* did,
    const char* intent_id,
    aeth_wallet_result_t* result
);

// Export public key
int aeth_export_pubkey(
    const char* subject,
    const char* wallet_id,
    const char* key_id,
    const char* intent_id,
    aeth_wallet_result_t* result
);

// Import encrypted key
int aeth_key_import(
    const char* subject,
    const char* wallet_id,
    const void* encrypted_key,
    size_t encrypted_key_len,
    const char* intent_id,
    aeth_wallet_result_t* result
);

// Revoke a key
int aeth_key_revoke(
    const char* subject,
    const char* wallet_id,
    const char* key_id,
    const char* intent_id,
    aeth_wallet_result_t* result
);

// Create a new DID
int aeth_did_create(
    const char* subject,
    aeth_did_method_t method,
    const char* key_id,
    const char* intent_id,
    aeth_wallet_result_t* result
);

// List vault items
int aeth_vault_list(
    const char* subject,
    const char* wallet_id,
    const char* intent_id,
    aeth_wallet_result_t* result
);

// Get vault item
int aeth_vault_get(
    const char* subject,
    const char* wallet_id,
    const char* item_id,
    const char* intent_id,
    aeth_wallet_result_t* result
);

// Put vault item
int aeth_vault_put(
    const char* subject,
    const char* wallet_id,
    const char* item_id,
    const void* data,
    size_t data_len,
    const char* intent_id,
    aeth_wallet_result_t* result
);

// Remove vault item
int aeth_vault_remove(
    const char* subject,
    const char* wallet_id,
    const char* item_id,
    const char* intent_id,
    aeth_wallet_result_t* result
);

// Get audit log
int aeth_audit_tail(
    const char* subject,
    size_t limit,
    aeth_wallet_result_t* result
);

// Free FFI result data
void aeth_free_result(aeth_wallet_result_t* result);

// Free FFI wallet handle
void aeth_free_wallet(aeth_wallet_t* wallet);

// Free FFI key handle
void aeth_free_key(aeth_key_t* key);

// Free FFI signature
void aeth_free_signature(aeth_signature_t* signature);

#ifdef __cplusplus
}
#endif

#endif // LIBAETH_WALLET_H
