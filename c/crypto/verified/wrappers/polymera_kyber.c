/* Polymera Kyber wrapper — see README.md. This file MUST NOT contain
 * cryptographic logic. Every function below is a one-line delegation to
 * HACL*. If you find yourself writing math here, stop and add the math
 * to a verified module instead. */

#include "polymera_kyber.h"

/* The HACL* Kyber headers are produced by F* extraction and live under
 * c/crypto/verified/upstream/. Until that submodule is vendored (see
 * README.md "Vendoring HACL*"), the body of each function below returns
 * -1 to make the unverified state loudly observable in tests rather than
 * silently producing nonsense bytes.
 *
 * After vendoring, replace each return with the corresponding Hacl_Kyber*
 * call. Do not add any pre/post processing — that defeats the proof. */

#if defined(POLYMERA_HACL_KYBER_VENDORED)
#include "Hacl_Kyber.h"

int polymera_kyber512_keygen(uint8_t pk[POLYMERA_KYBER512_PUBLICKEY_BYTES],
                             uint8_t sk[POLYMERA_KYBER512_SECRETKEY_BYTES]) {
    return Hacl_Kyber512_keygen(pk, sk);
}

int polymera_kyber512_encaps(uint8_t ct[POLYMERA_KYBER512_CIPHERTEXT_BYTES],
                             uint8_t ss[POLYMERA_KYBER512_SHAREDSECRET_BYTES],
                             const uint8_t pk[POLYMERA_KYBER512_PUBLICKEY_BYTES]) {
    return Hacl_Kyber512_encaps(ct, ss, pk);
}

int polymera_kyber512_decaps(uint8_t ss[POLYMERA_KYBER512_SHAREDSECRET_BYTES],
                             const uint8_t ct[POLYMERA_KYBER512_CIPHERTEXT_BYTES],
                             const uint8_t sk[POLYMERA_KYBER512_SECRETKEY_BYTES]) {
    return Hacl_Kyber512_decaps(ss, ct, sk);
}

#else  /* !POLYMERA_HACL_KYBER_VENDORED */

int polymera_kyber512_keygen(uint8_t pk[POLYMERA_KYBER512_PUBLICKEY_BYTES],
                             uint8_t sk[POLYMERA_KYBER512_SECRETKEY_BYTES]) {
    (void)pk; (void)sk;
    return -1;
}

int polymera_kyber512_encaps(uint8_t ct[POLYMERA_KYBER512_CIPHERTEXT_BYTES],
                             uint8_t ss[POLYMERA_KYBER512_SHAREDSECRET_BYTES],
                             const uint8_t pk[POLYMERA_KYBER512_PUBLICKEY_BYTES]) {
    (void)ct; (void)ss; (void)pk;
    return -1;
}

int polymera_kyber512_decaps(uint8_t ss[POLYMERA_KYBER512_SHAREDSECRET_BYTES],
                             const uint8_t ct[POLYMERA_KYBER512_CIPHERTEXT_BYTES],
                             const uint8_t sk[POLYMERA_KYBER512_SECRETKEY_BYTES]) {
    (void)ss; (void)ct; (void)sk;
    return -1;
}

#endif
