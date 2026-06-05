import { Encoder } from 'cbor-x';
import nacl from 'tweetnacl';
import fs from 'node:fs';
import path from 'node:path';

function base64url(buf) {
  return Buffer.from(buf).toString('base64').replace(/=/g, '').replace(/\+/g, '-').replace(/\//g, '_');
}

const encoder = new Encoder({ useRecords: false, mapsAsObjects: true, structuredClone: true });

function encode(obj) { return Buffer.from(encoder.encode(obj)); }

(async () => {
  const workspace = process.env.GITHUB_WORKSPACE || process.cwd();
  const devDir = path.join(workspace, 'policy', 'captoken_v2', 'dev');
  fs.mkdirSync(devDir, { recursive: true });

  const kp = nacl.sign.keyPair();
  const iss = 'did:dev:aetheris';
  const sub = 'did:dev:smoke';
  const aud = 'aetheris-kernel';
  const now = Math.floor(Date.now() / 1000);
  const exp = now + 900;
  const jti = Buffer.from(kp.publicKey).subarray(0, 16); // deterministic jti from pubkey
  const scopes = ['intent.emit'];

  const claims = new Map();
  claims.set('iss', iss);
  claims.set('sub', sub);
  claims.set('aud', aud);
  claims.set('iat', now);
  claims.set('nbf', now - 5);
  claims.set('exp', exp);
  claims.set('jti', new Uint8Array(jti));
  claims.set('scopes', scopes);

  const payload = encode(claims);
  const protectedMap = new Map();
  protectedMap.set(1, -8); // alg: EdDSA
  protectedMap.set(4, new Uint8Array(Buffer.from('dev-1'))); // kid
  const protectedBytes = encode(protectedMap);
  const sigStructure = ['Signature1', new Uint8Array(protectedBytes), new Uint8Array(0), new Uint8Array(payload)];
  const toSign = encode(sigStructure);
  const sig = nacl.sign.detached(new Uint8Array(toSign), kp.secretKey);

  const coseSign1 = [new Uint8Array(protectedBytes), new Map(), new Uint8Array(payload), new Uint8Array(sig)];
  const tokenBytes = encode(coseSign1);

  // Write token
  const tokenPath = path.join(devDir, 'test.token');
  fs.writeFileSync(tokenPath, tokenBytes);

  // Write jwks.json
  const jwksPath = path.join(devDir, 'jwks.json');
  const jwk = { kty: 'OKP', crv: 'Ed25519', kid: 'dev-1', x: base64url(kp.publicKey) };
  fs.writeFileSync(jwksPath, JSON.stringify({ keys: [jwk] }, null, 2));

  // Write issuer.json
  const issuerPath = path.join(devDir, 'issuer.json');
  fs.writeFileSync(issuerPath, JSON.stringify({ jwks_path: path.relative(workspace, jwksPath), audience: aud, clock_skew_s: 120 }, null, 2));

  console.log(`Minted dev token at ${tokenPath}`);
})();
