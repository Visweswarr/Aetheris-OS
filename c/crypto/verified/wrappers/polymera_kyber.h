/* Polymera Kyber wrapper.
 *
 * Thin C-ABI surface delegating to HACL* verified Kyber. This header is the
 * stable contract; the implementation in polymera_kyber.c MUST NOT contain
 * any cryptographic logic beyond argument forwarding (see README.md). */

#ifndef POLYMERA_KYBER_H
#define POLYMERA_KYBER_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Kyber-512 sizes. Constants mirror NIST FIPS 203. */
#define POLYMERA_KYBER512_PUBLICKEY_BYTES   800
#define POLYMERA_KYBER512_SECRETKEY_BYTES   1632
#define POLYMERA_KYBER512_CIPHERTEXT_BYTES  768
#define POLYMERA_KYBER512_SHAREDSECRET_BYTES 32

/* Returns 0 on success, nonzero on failure. */
int polymera_kyber512_keygen(uint8_t pk[POLYMERA_KYBER512_PUBLICKEY_BYTES],
                             uint8_t sk[POLYMERA_KYBER512_SECRETKEY_BYTES]);

int polymera_kyber512_encaps(uint8_t ct[POLYMERA_KYBER512_CIPHERTEXT_BYTES],
                             uint8_t ss[POLYMERA_KYBER512_SHAREDSECRET_BYTES],
                             const uint8_t pk[POLYMERA_KYBER512_PUBLICKEY_BYTES]);

int polymera_kyber512_decaps(uint8_t ss[POLYMERA_KYBER512_SHAREDSECRET_BYTES],
                             const uint8_t ct[POLYMERA_KYBER512_CIPHERTEXT_BYTES],
                             const uint8_t sk[POLYMERA_KYBER512_SECRETKEY_BYTES]);

#ifdef __cplusplus
}
#endif

#endif /* POLYMERA_KYBER_H */
