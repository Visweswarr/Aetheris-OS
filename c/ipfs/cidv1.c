#include "libaeth_ipfs.h"
#include <string.h>

int aeth_ipfs_cidv1_build(uint64_t multicodec, const uint8_t* multihash, size_t multihash_len, char* out_str, size_t out_cap) {
    if (!multihash || !out_str) {
        return AETH_IPFS_ERROR_INVALID_INPUT;
    }

    // CIDv1 format: [version, multicodec, multihash]
    // version = 1 (varint), multicodec (varint), multihash (bytes)
    
    // Calculate required buffer size for CIDv1 bytes
    uint8_t cid_bytes[128];
    size_t cid_len = 0;
    
    // Version (1)
    int version_len = aeth_ipfs_varint_encode(1, &cid_bytes[cid_len], sizeof(cid_bytes) - cid_len);
    if (version_len < 0) {
        return version_len;
    }
    cid_len += version_len;
    
    // Multicodec
    int codec_len = aeth_ipfs_varint_encode(multicodec, &cid_bytes[cid_len], sizeof(cid_bytes) - cid_len);
    if (codec_len < 0) {
        return codec_len;
    }
    cid_len += codec_len;
    
    // Multihash
    if (cid_len + multihash_len > sizeof(cid_bytes)) {
        return AETH_IPFS_ERROR_BUFFER_TOO_SMALL;
    }
    memcpy(&cid_bytes[cid_len], multihash, multihash_len);
    cid_len += multihash_len;
    
    // Encode to multibase base32-lowercase
    return aeth_ipfs_multibase_b32lower(cid_bytes, cid_len, out_str, out_cap);
}

int aeth_ipfs_cidv1_dag_cbor_sha256(const uint8_t* data, size_t len, char* out_str, size_t out_cap) {
    if (!data || !out_str) {
        return AETH_IPFS_ERROR_INVALID_INPUT;
    }

    // Build multihash for SHA2-256
    uint8_t multihash[34]; // code(1) + length(1) + digest(32)
    int multihash_len = aeth_ipfs_multihash_sha256(data, len, multihash, sizeof(multihash));
    if (multihash_len < 0) {
        return multihash_len;
    }
    
    // Build CIDv1 with dag-cbor multicodec
    return aeth_ipfs_cidv1_build(AETH_MULTICODEC_DAG_CBOR, multihash, multihash_len, out_str, out_cap);
}

int aeth_ipfs_cidv1_raw_sha256(const uint8_t* data, size_t len, char* out_str, size_t out_cap) {
    if (!data || !out_str) {
        return AETH_IPFS_ERROR_INVALID_INPUT;
    }

    // Build multihash for SHA2-256
    uint8_t multihash[34]; // code(1) + length(1) + digest(32)
    int multihash_len = aeth_ipfs_multihash_sha256(data, len, multihash, sizeof(multihash));
    if (multihash_len < 0) {
        return multihash_len;
    }
    
    // Build CIDv1 with raw multicodec
    return aeth_ipfs_cidv1_build(AETH_MULTICODEC_RAW, multihash, multihash_len, out_str, out_cap);
}
