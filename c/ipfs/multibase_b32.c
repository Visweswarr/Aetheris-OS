#include "libaeth_ipfs.h"
#include <string.h>

// Base32 alphabet (RFC 4648, lowercase)
static const char base32_alphabet[] = "abcdefghijklmnopqrstuvwxyz234567";

int aeth_ipfs_multibase_b32lower(const uint8_t* data, size_t len, char* out, size_t out_cap) {
    if (!data || !out) {
        return AETH_IPFS_ERROR_INVALID_INPUT;
    }

    // Calculate required output size: prefix + encoded data
    size_t encoded_len = ((len * 8) + 4) / 5; // 5 bits per character
    size_t total_len = 1 + encoded_len; // 'b' prefix + encoded data
    
    if (total_len > out_cap) {
        return AETH_IPFS_ERROR_BUFFER_TOO_SMALL;
    }

    // Add multibase prefix
    out[0] = AETH_MULTIBASE_B32LOWER;
    
    if (len == 0) {
        out[1] = '\0';
        return 1;
    }

    // Encode data in base32
    size_t bit_pos = 0;
    size_t out_pos = 1;
    
    for (size_t i = 0; i < len; i++) {
        // Process each byte
        uint8_t byte = data[i];
        
        // Handle remaining bits from previous byte
        if (bit_pos > 0) {
            uint8_t prev_bits = (byte << (8 - bit_pos)) & 0x1F;
            out[out_pos++] = base32_alphabet[prev_bits];
            bit_pos = (bit_pos + 5) % 8;
        }
        
        // Process current byte
        while (bit_pos < 8) {
            uint8_t bits = (byte >> (8 - bit_pos - 5)) & 0x1F;
            if (bit_pos + 5 <= 8) {
                out[out_pos++] = base32_alphabet[bits];
                bit_pos += 5;
            } else {
                break;
            }
        }
    }
    
    // Handle padding if needed
    if (bit_pos > 0) {
        uint8_t remaining_bits = (data[len - 1] << (8 - bit_pos)) & 0x1F;
        out[out_pos++] = base32_alphabet[remaining_bits];
    }
    
    // Add null terminator
    out[out_pos] = '\0';
    
    return (int)out_pos;
}
