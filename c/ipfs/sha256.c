#include "libaeth_ipfs.h"
#include <string.h>

// SHA2-256 constants
static const uint32_t K[64] = {
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
    0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
    0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
    0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
    0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
};

// SHA2-256 initial hash values
static const uint32_t H0[8] = {
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
};

// Right rotation macro
#define ROTR(x, n) (((x) >> (n)) | ((x) << (32 - (n))))

// SHA2-256 functions
#define CH(x, y, z) (((x) & (y)) ^ (~(x) & (z)))
#define MAJ(x, y, z) (((x) & (y)) ^ ((x) & (z)) ^ ((y) & (z)))
#define EP0(x) (ROTR(x, 2) ^ ROTR(x, 13) ^ ROTR(x, 22))
#define EP1(x) (ROTR(x, 6) ^ ROTR(x, 11) ^ ROTR(x, 25))
#define SIG0(x) (ROTR(x, 7) ^ ROTR(x, 18) ^ ((x) >> 3))
#define SIG1(x) (ROTR(x, 17) ^ ROTR(x, 19) ^ ((x) >> 10))

// SHA2-256 context
typedef struct {
    uint32_t state[8];
    uint64_t count;
    uint8_t buffer[64];
} sha256_ctx;

// Initialize SHA2-256 context
static void sha256_init(sha256_ctx* ctx) {
    memcpy(ctx->state, H0, sizeof(H0));
    ctx->count = 0;
}

// Process a 512-bit block
static void sha256_transform(sha256_ctx* ctx, const uint8_t* data) {
    uint32_t a, b, c, d, e, f, g, h;
    uint32_t W[64];
    uint32_t T1, T2;
    int t;

    // Prepare message schedule
    for (t = 0; t < 16; t++) {
        W[t] = ((uint32_t)data[t * 4] << 24) |
                ((uint32_t)data[t * 4 + 1] << 16) |
                ((uint32_t)data[t * 4 + 2] << 8) |
                ((uint32_t)data[t * 4 + 3]);
    }
    
    for (t = 16; t < 64; t++) {
        W[t] = SIG1(W[t - 2]) + W[t - 7] + SIG0(W[t - 15]) + W[t - 16];
    }

    // Initialize working variables
    a = ctx->state[0];
    b = ctx->state[1];
    c = ctx->state[2];
    d = ctx->state[3];
    e = ctx->state[4];
    f = ctx->state[5];
    g = ctx->state[6];
    h = ctx->state[7];

    // Main loop
    for (t = 0; t < 64; t++) {
        T1 = h + EP1(e) + CH(e, f, g) + K[t] + W[t];
        T2 = EP0(a) + MAJ(a, b, c);
        h = g;
        g = f;
        f = e;
        e = d + T1;
        d = c;
        c = b;
        b = a;
        a = T1 + T2;
    }

    // Update state
    ctx->state[0] += a;
    ctx->state[1] += b;
    ctx->state[2] += c;
    ctx->state[3] += d;
    ctx->state[4] += e;
    ctx->state[5] += f;
    ctx->state[6] += g;
    ctx->state[7] += h;
}

// Update SHA2-256 context with data
static void sha256_update(sha256_ctx* ctx, const uint8_t* data, size_t len) {
    size_t i;
    uint32_t j;

    j = (uint32_t)((ctx->count >> 3) & 0x3F);
    ctx->count += len << 3;

    if ((j + len) > 63) {
        i = 64 - j;
        memcpy(&ctx->buffer[j], data, i);
        sha256_transform(ctx, ctx->buffer);
        
        for (; i + 63 < len; i += 64) {
            sha256_transform(ctx, &data[i]);
        }
        j = 0;
    } else {
        i = 0;
    }
    
    memcpy(&ctx->buffer[j], &data[i], len - i);
}

// Finalize SHA2-256 hash
static void sha256_final(sha256_ctx* ctx, uint8_t* hash) {
    uint32_t i;
    uint8_t finalcount[8];

    for (i = 0; i < 8; i++) {
        finalcount[i] = (uint8_t)((ctx->count >> ((7 - i) * 8)) & 0xFF);
    }

    sha256_update(ctx, (uint8_t*)"\x80", 1);
    while ((ctx->count & 1023) != 896) {
        sha256_update(ctx, (uint8_t*)"\0", 1);
    }
    sha256_update(ctx, finalcount, 8);

    for (i = 0; i < 32; i++) {
        hash[i] = (uint8_t)((ctx->state[i >> 2] >> ((3 - (i & 3)) * 8)) & 0xFF);
    }
}

int aeth_ipfs_sha256(const uint8_t* data, size_t len, uint8_t* hash_out) {
    if (!data || !hash_out) {
        return AETH_IPFS_ERROR_INVALID_INPUT;
    }

    sha256_ctx ctx;
    sha256_init(&ctx);
    sha256_update(&ctx, data, len);
    sha256_final(&ctx, hash_out);
    
    return AETH_IPFS_OK;
}
