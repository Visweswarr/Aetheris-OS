#ifndef AETH_IPFS_H
#define AETH_IPFS_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Multicodec values for IPFS
#define AETH_MULTICODEC_DAG_CBOR 0x71
#define AETH_MULTICODEC_RAW 0x55

// Multihash algorithm values
#define AETH_MULTIHASH_SHA2_256 0x12

// Multibase encoding
#define AETH_MULTIBASE_B32LOWER 'b'

// Maximum CIDv1 string length (including null terminator)
#define AETH_MAX_CIDV1_LENGTH 128

// Maximum varint length for LEB128 encoding
#define AETH_MAX_VARINT_LENGTH 9

// Error codes
#define AETH_IPFS_OK 0
#define AETH_IPFS_ERROR_INVALID_INPUT -1
#define AETH_IPFS_ERROR_BUFFER_TOO_SMALL -2
#define AETH_IPFS_ERROR_INVALID_MULTICODEC -3

// Build CIDv1 string for DAG-CBOR data
// Returns AETH_IPFS_OK on success, error code on failure
int aeth_ipfs_cidv1_dag_cbor_sha256(
    const uint8_t* data, 
    size_t len,
    char* out_str, 
    size_t out_cap
);

// Build CIDv1 string for RAW block data
// Returns AETH_IPFS_OK on success, error code on failure
int aeth_ipfs_cidv1_raw_sha256(
    const uint8_t* data, 
    size_t len,
    char* out_str, 
    size_t out_cap
);

// Lower-level helpers for testing and advanced usage

// Encode LEB128 varint
// Returns number of bytes written, or negative error code
int aeth_ipfs_varint_encode(uint64_t value, uint8_t* out, size_t out_cap);

// Compute SHA2-256 hash
// Returns AETH_IPFS_OK on success, error code on failure
int aeth_ipfs_sha256(
    const uint8_t* data, 
    size_t len, 
    uint8_t* hash_out
);

// Encode multibase base32-lowercase
// Returns number of bytes written, or negative error code
int aeth_ipfs_multibase_b32lower(
    const uint8_t* data, 
    size_t len, 
    char* out, 
    size_t out_cap
);

// Build multihash for SHA2-256
// Returns number of bytes written, or negative error code
int aeth_ipfs_multihash_sha256(
    const uint8_t* data, 
    size_t len, 
    uint8_t* out, 
    size_t out_cap
);

// Build CIDv1 from multicodec and multihash
// Returns number of bytes written, or negative error code
int aeth_ipfs_cidv1_build(
    uint64_t multicodec,
    const uint8_t* multihash,
    size_t multihash_len,
    char* out_str,
    size_t out_cap
);

#ifdef __cplusplus
}
#endif

#endif // AETH_IPFS_H
