/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2024 Polymera OS Contributors
 * 
 * NGFS Merkle Verifier Library
 * 
 * This header provides C functions for computing NGFS Merkle roots and
 * verifying membership proofs, ensuring cross-language consistency.
 */

#ifndef NGFS_MERKLE_H
#define NGFS_MERKLE_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Error codes */
#define NGFS_OK 0
#define NGFS_ERROR_INVALID_INPUT -1
#define NGFS_ERROR_INVALID_CBOR -2
#define NGFS_ERROR_PROOF_INVALID -3
#define NGFS_ERROR_BUFFER_TOO_SMALL -4

/* Constants */
#define NGFS_BLAKE3_SIZE 32
#define NGFS_MAX_MANIFEST_SIZE (64 * 1024)
#define NGFS_MAX_PROOF_DEPTH 64

/* Content types */
#define NGFS_CONTENT_RAW 0
#define NGFS_CONTENT_DIRECTORY 1
#define NGFS_CONTENT_FILE_MANIFEST 2
#define NGFS_CONTENT_SNAPSHOT 3
#define NGFS_CONTENT_SYMLINK 4
#define NGFS_CONTENT_SPECIAL 5

/* Entry kinds */
#define NGFS_ENTRY_DIRECTORY 0
#define NGFS_ENTRY_FILE 1
#define NGFS_ENTRY_SYMLINK 2

/**
 * Compute Blake3 hash of CBOR data for NGFS manifest
 * 
 * @param cbor Input CBOR data
 * @param cbor_len Length of CBOR data
 * @param out_hash32 Output buffer for 32-byte Blake3 hash
 * 
 * @return NGFS_OK on success, negative error code on failure
 * 
 * This function computes the deterministic Blake3 hash of the CBOR data,
 * which serves as the content identifier (CID) for the manifest.
 */
int aeth_ngfs_root_blake3(
    const uint8_t* cbor,
    size_t cbor_len,
    uint8_t out_hash32[32]
);

/**
 * Verify NGFS membership proof
 * 
 * @param proof_cbor CBOR-encoded proof data
 * @param proof_len Length of proof CBOR data
 * @param root_cid Expected root CID (32 bytes)
 * @param root_cid_len Length of root CID
 * 
 * @return NGFS_OK if proof is valid, negative error code on failure
 * 
 * This function verifies that a membership proof correctly demonstrates
 * that a node is part of a tree with the given root CID.
 */
int aeth_ngfs_verify_proof(
    const uint8_t* proof_cbor,
    size_t proof_len,
    const uint8_t* root_cid,
    size_t root_cid_len
);

/**
 * Validate NGFS manifest structure
 * 
 * @param cbor Input CBOR data
 * @param cbor_len Length of CBOR data
 * @param expected_type Expected content type
 * 
 * @return NGFS_OK if manifest is valid, negative error code on failure
 * 
 * This function performs basic structural validation of NGFS manifests,
 * ensuring they conform to the expected schema.
 */
int aeth_ngfs_validate_manifest(
    const uint8_t* cbor,
    size_t cbor_len,
    uint8_t expected_type
);

/**
 * Extract entry count from directory manifest
 * 
 * @param cbor Input CBOR data
 * @param cbor_len Length of CBOR data
 * @param out_count Output for entry count
 * 
 * @return NGFS_OK on success, negative error code on failure
 * 
 * This function extracts the number of entries from a directory manifest
 * without fully parsing the entire structure.
 */
int aeth_ngfs_dir_entry_count(
    const uint8_t* cbor,
    size_t cbor_len,
    uint32_t* out_count
);

/**
 * Extract chunk count from file manifest
 * 
 * @param cbor Input CBOR data
 * @param cbor_len Length of CBOR data
 * @param out_count Output for chunk count
 * 
 * @return NGFS_OK on success, negative error code on failure
 * 
 * This function extracts the number of chunks from a file manifest
 * without fully parsing the entire structure.
 */
int aeth_ngfs_file_chunk_count(
    const uint8_t* cbor,
    size_t cbor_len,
    uint32_t* out_count
);

/**
 * Check if CBOR data represents a valid NGFS manifest
 * 
 * @param cbor Input CBOR data
 * @param cbor_len Length of CBOR data
 * 
 * @return true if valid NGFS manifest, false otherwise
 * 
 * This function performs a quick validity check without detailed parsing.
 */
bool aeth_ngfs_is_valid_manifest(
    const uint8_t* cbor,
    size_t cbor_len
);

#ifdef __cplusplus
}
#endif

#endif /* NGFS_MERKLE_H */
