# Phase 5 — CapToken v2 (COSE/CBOR) Online Verification

Status: Draft (Phase 5-D)

## Token Format (COSE_Sign1 + CBOR claims)
- Protected alg: EdDSA; no unprotected alg override.
- Claims (CBOR map): iss, sub, aud, iat, nbf, exp, jti(16 bytes), scopes([tstr]), ctx(optional map).
- Envelope: COSE_Sign1 with payload=CBOR(claims); signature=Ed25519.

## JWKS & Issuer
- JWKS (JSON): [{kty:"OKP", crv:"Ed25519", kid, x: base64url(pubkey)}]
- Issuer config (JSON): { jwks_path, audience, clock_skew_s }
- Env: AETHERIS_ISSUER=<path/to/issuer.json>

## Scopes Table
- intent.preview/commit → intent.emit
- ngfs.snapshot.write → ngfs.snapshot.write
- ngfs.snapshot.read  → ngfs.snapshot.read
- ai.memory.read/write → ai.memory.read / ai.memory.write (future)

## Replay Protection & Clock Skew
- Nonce jti (16 bytes) unique per token; cache in LRU with TTL 15m.
- Time checks with ±clock_skew_s tolerance for iat/nbf/exp.

## Audit Logs
- Path: $NGFS_ROOT/audit/captoken/YYYYMMDD.cborl
- Record: { ts90k, iss, sub, jti(hex), scopes, action, result, reason?, endpoint }

## Dev Quickstart
- Generate dev token:
```bash
cargo build --manifest-path tools/captoken_minter/Cargo.toml --verbose
# write 32-byte ed25519.sk
head -c 32 /dev/urandom > policy/captoken_v2/dev/ed25519.sk
# mint token + jwks
./target/debug/captoken-minter --iss did:dev:aetheris --sub did:dev:me --aud aetheris-kernel --exp 900 \
  --scopes intent.emit,ngfs.snapshot.write --key policy/captoken_v2/dev/ed25519.sk \
  --out policy/captoken_v2/dev/test.token --kid dev-1 --out-jwks policy/captoken_v2/dev/jwks.json
# issuer.json
echo '{"jwks_path":"policy/captoken_v2/dev/jwks.json","audience":"aetheris-kernel","clock_skew_s":120}' > policy/captoken_v2/dev/issuer.json
```
- Run echo server with verification:
```bash
AETHERIS_ISSUER=policy/captoken_v2/dev/issuer.json ./target/debug/intent-echo-server --unix /tmp/aetheris-intent.sock
```

## Kernel Enforcement

Note: Echo server path uses the same CBOR envelope and writes audits; CI transport tests are authoritative for allow/deny behavior. This path will import kernel PolicyEngine & Audit when promoted to kernelspace.

### Syscall Envelope & Back-compat
- New path: versioned CBOR envelope for Intent syscalls
  - { v:1, captoken: bstr (COSE_Sign1), payload: bstr (CBOR Intent) }
- Legacy path: bare CBOR Intent (only allowed if AETHERIS_REQUIRE_TOKEN!=1 and caller sends no token)
- NGFS shim: ngfs_guard_read/ngfs_guard_write enforce ngfs.snapshot.* scopes.
- Policy enforcement and audit in userland now provided by `policy_engine::{cap,audit}` shared crate (used by echo server).
- Kernel syscall edges remain unchanged for now; we will wire `policy_engine` in a follow-up if desired.
- Env: AETHERIS_ISSUER points to issuer.json with jwks_path/audience/clock_skew_s
- Audit path: $NGFS_ROOT/audit/captoken/YYYYMMDD.cborl (CBOR per line)
- Scopes mapping remains as documented; intent preview/commit require intent.emit.

## CI Coverage
- Linux: success case (intent.emit present) returns status=ok; missing scope returns status=deny.
- Windows: success path validated over named pipe.

## Shared policy crate tests & conformance
- Run unit tests: `cargo test -p policy_engine --all-features`
- Conformance smoke (userland): starts echo server, mints allow and deny tokens, sends envelopes, and verifies audits. Triggered in CI (Linux) after CBOR transport smokes.

## Security Notes
- Key rotation via kid: update JWKS with new key; server reloads on demand (launcher can signal or poll mtime).
- Always deny on alg≠EdDSA, audience mismatch, expired/nbf violations, or replayed jti.
