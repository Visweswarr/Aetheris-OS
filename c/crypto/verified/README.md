# Polymera OS — Verified Cryptography

Initiative 5 of the polyglot roadmap. This directory holds the **verified**
cryptography surface used by Polymera. Code here is either:

1. **Extracted-from-F\*** C, typically from [HACL\*](https://github.com/hacl-star/hacl-star).
   Treat as read-only — never hand-edit. Re-extract from the upstream proof.
2. **Polymera-specific wrappers** that adapt HACL\* to our `services/wallet`,
   `services/chain`, and crypto-bus surfaces. Wrappers must contain *no* cryptographic
   logic; they only marshal arguments and forward to HACL\*. Otherwise the verification
   guarantee is lost.

## Why F\* / HACL\*

We claim "post-quantum, quantum-era OS." That claim is only defensible if the PQC
implementations are *proven* correct against their specifications, not merely fuzzed.
HACL\* gives us:

- Functional-correctness proofs (the C code matches the algorithm's mathematical spec).
- Memory-safety proofs (no buffer overflows, no UB).
- Secret-independence proofs (no value-dependent branches → constant-time guarantees).

Adopting HACL\* is *cheaper* than writing our own F\* proofs and gives the same guarantees
for the algorithms HACL\* already covers (Ed25519, Curve25519, SHA-2, ChaCha20-Poly1305).

For NIST PQC primitives (Kyber, Dilithium), HACL\* coverage is partial as of writing —
use the unverified `pqcrypto-*` Rust crates with an explicit "**unverified**" annotation
in `LANGUAGES.md` until either (a) HACL\* gains coverage or (b) we write our own F\* proof.

## Layout

```
c/crypto/verified/
├── README.md                  # this file
├── upstream/                  # HACL* extraction (vendored, do not edit)
│   └── .gitkeep
├── wrappers/                  # thin Polymera wrappers around HACL*
│   ├── polymera_kyber.h
│   └── polymera_kyber.c       # delegates to upstream/Hacl_Kyber*.{h,c}
└── proofs/                    # Polymera-specific F* specs (if any)
    └── .gitkeep
```

## Vendoring HACL\*

```bash
# From repo root.
git submodule add https://github.com/hacl-star/hacl-star.git c/crypto/verified/upstream
cd c/crypto/verified/upstream
git checkout <pinned-tag>           # never `main` — pin a release
```

Re-vendor only on a release-cadence schedule. Every re-vendor commit must reference
the upstream HACL\* tag and the SHA256 of the extraction artefact.

## Verifying our wrappers

The wrappers themselves are *not* verified by HACL\* — only the algorithms they call.
We require:

1. The wrapper performs no transform on inputs/outputs other than copying.
2. CI runs the wrapper against HACL\*'s test vectors (see `c/crypto/verified/tests/`).
3. A reviewer signs off that the wrapper does not introduce timing variance.

For our own algorithms (e.g. zkVM verification gadgets), write F\* specs in
`proofs/<module>.fst`, run `fstar --codegen c`, and check the extracted C into
`upstream-polymera/` alongside the proof.

## CI

A verification CI job is scaffolded at `.github/workflows/verification.yml` (planned —
see roadmap Initiative 5). It will:

- Install `opam install fstar` in a cached Docker layer.
- Re-verify all `.fst` files in `proofs/`.
- Diff the extracted C against the checked-in `upstream-polymera/` and fail on drift.

## What is **not** in this directory

- `pqcrypto-kyber` and `pqcrypto-dilithium` Rust crates (unverified, used by
  `crypto/src/crypto/liboqs/`). Those remain in place until HACL\* PQC coverage matures.
- Anything in `c/crypto/*.c` outside this `verified/` subtree — that code is assumed
  unverified and must not be used for new attack-surface paths.
