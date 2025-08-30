#include "libaeth_ipfs.h"

int aeth_ipfs_varint_encode(uint64_t value, uint8_t* out, size_t out_cap) {
    if (!out) {
        return AETH_IPFS_ERROR_INVALID_INPUT;
    }

    size_t written = 0;
    
    do {
        if (written >= out_cap) {
            return AETH_IPFS_ERROR_BUFFER_TOO_SMALL;
        }
        
        uint8_t byte = (uint8_t)(value & 0x7F);
        value >>= 7;
        
        if (value != 0) {
            byte |= 0x80;
        }
        
        out[written++] = byte;
    } while (value != 0);
    
    return (int)written;
}
