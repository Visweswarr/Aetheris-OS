#include "libaeth_ipfs.h"
#include <string.h>

int aeth_ipfs_multihash_sha256(const uint8_t* data, size_t len, uint8_t* out, size_t out_cap) {
    if (!data || !out) {
        return AETH_IPFS_ERROR_INVALID_INPUT;
    }

    // Multihash format: [code, length, digest]
    // For SHA2-256: code=0x12, length=32, digest=32 bytes
    size_t required_len = 2 + 32; // code + length + digest
    
    if (out_cap < required_len) {
        return AETH_IPFS_ERROR_BUFFER_TOO_SMALL;
    }

    // Hash algorithm code
    out[0] = AETH_MULTIHASH_SHA2_256;
    
    // Hash length
    out[1] = 32;
    
    // Compute SHA2-256 hash
    int result = aeth_ipfs_sha256(data, len, &out[2]);
    if (result != AETH_IPFS_OK) {
        return result;
    }
    
    return (int)required_len;
}
