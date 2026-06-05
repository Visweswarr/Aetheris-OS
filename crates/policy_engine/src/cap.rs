use chrono::Utc;
use lru::LruCache;
use std::{num::NonZeroUsize, sync::Mutex, fs};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("bad token: {0}")] BadToken(String),
    #[error("signature invalid")] BadSignature,
    #[error("aud mismatch")] Audience,
    #[error("expired or not yet valid")] TimeWindow,
    #[error("replay detected")] Replay,
    #[error("jwks load error: {0}")] Jwks(String),
    #[error("issuer load error: {0}")] Issuer(String),
    #[error("internal: {0}")] Internal(String),
    #[error("lock poison error: {0}")] LockPoison(String),
}

#[derive(Clone, Debug, Deserialize)]
pub struct IssuerCfg {
    pub jwks_path: String,
    pub audience: String,
    #[serde(default = "default_skew")] pub clock_skew_s: i64,
}
fn default_skew() -> i64 { 120 }

#[derive(Clone, Debug)]
pub struct Claims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    pub nbf: Option<i64>,
    pub iat: Option<i64>,
    pub jti_hex: String,
    pub scopes: Vec<String>,
}

pub struct CapVerifier {
    cfg: IssuerCfg,
    nonce_lru: Mutex<LruCache<String, i64>>,
    jwks_bytes: Mutex<Vec<u8>>,
    last_reload: Mutex<i64>,
}

impl CapVerifier {
    pub fn from_env() -> Result<Self, VerifyError> {
        let p = std::env::var("AETHERIS_ISSUER").map_err(|e| VerifyError::Issuer(e.to_string()))?;
        let cfg_bytes = fs::read(&p).map_err(|e| VerifyError::Issuer(e.to_string()))?;
        let cfg: IssuerCfg = serde_json::from_slice(&cfg_bytes).map_err(|e| VerifyError::Issuer(e.to_string()))?;
        
        let cache_size = NonZeroUsize::new(4096).ok_or_else(|| VerifyError::Internal("Invalid cache size".into()))?;
        
        Ok(Self {
            cfg,
            nonce_lru: Mutex::new(LruCache::new(cache_size)),
            jwks_bytes: Mutex::new(Vec::new()),
            last_reload: Mutex::new(0),
        })
    }

    pub fn check_and_remember_nonce(&self, jti_hex: &str, now: i64) -> Result<(), VerifyError> {
        let mut lru = self.nonce_lru.lock().map_err(|e| VerifyError::LockPoison(e.to_string()))?;
        if lru.put(jti_hex.to_string(), now).is_some() {
            return Err(VerifyError::Replay);
        }
        Ok(())
    }

    fn maybe_reload_jwks(&self) -> Result<(), VerifyError> {
        let now = Utc::now().timestamp();
        let mut last = self.last_reload.lock().map_err(|e| VerifyError::LockPoison(e.to_string()))?;
        if now - *last < 60 { return Ok(()); }
        let jwks = fs::read(&self.cfg.jwks_path).map_err(|e| VerifyError::Jwks(e.to_string()))?;
        *self.jwks_bytes.lock().map_err(|e| VerifyError::LockPoison(e.to_string()))? = jwks;
        *last = now;
        Ok(())
    }

    pub fn verify(&self, cose_sign1: &[u8]) -> Result<Claims, VerifyError> {
        self.maybe_reload_jwks()?;
        let sign1 = coset::CoseSign1::from_slice(cose_sign1).map_err(|e| VerifyError::BadToken(e.to_string()))?;
        // Only EdDSA/Ed25519
        if sign1.protected.header.alg.as_ref().cloned() != Some(coset::iana::Algorithm::EdDSA) {
            return Err(VerifyError::BadToken("alg != EdDSA".into()));
        }
        let kid = sign1.protected.header.kid.clone().ok_or_else(|| VerifyError::BadToken("missing kid".into()))?;
        
        let jwks_bytes = self.jwks_bytes.lock().map_err(|e| VerifyError::LockPoison(e.to_string()))?;
        let jwks = serde_json::from_slice::<serde_json::Value>(&jwks_bytes)
            .map_err(|e| VerifyError::Jwks(e.to_string()))?;
            
        let keys = jwks.get("keys").and_then(|k| k.as_array()).ok_or_else(|| VerifyError::Jwks("keys[]".into()))?;
        let mut matched_key: Option<Vec<u8>> = None;
        for k in keys {
            if k.get("kty").and_then(|v| v.as_str()) != Some("OKP") { continue; }
            if k.get("crv").and_then(|v| v.as_str()) != Some("Ed25519") { continue; }
            if let Some(kid_s) = k.get("kid").and_then(|v| v.as_str()) {
                if kid_s.as_bytes() != kid.as_slice() { continue; }
            } else {
                continue;
            }
            if let Some(x_b64) = k.get("x").and_then(|v| v.as_str()) {
                let x = base64::engine::general_purpose::URL_SAFE_NO_PAD
                    .decode(x_b64.as_bytes()).map_err(|e| VerifyError::Jwks(e.to_string()))?;
                matched_key = Some(x);
                break;
            }
        }
        let pk = matched_key.ok_or_else(|| VerifyError::Jwks("kid not found".into()))?;
        let vk = ed25519_dalek::VerifyingKey::from_bytes(&pk[..32]).map_err(|_| VerifyError::BadToken("bad pubkey".into()))?;
        sign1.verify_signature(&vk).map_err(|_| VerifyError::BadSignature)?;
        let payload = sign1.payload.as_ref().ok_or_else(|| VerifyError::BadToken("missing payload".into()))?;

        #[derive(Deserialize)]
        struct RawClaims { iss:String, sub:String, aud:String, iat:Option<i64>, nbf:Option<i64>, exp:i64, jti:Vec<u8>, scopes:Vec<String> }
        let raw: RawClaims = ciborium::de::from_reader(payload.as_slice()).map_err(|e| VerifyError::BadToken(e.to_string()))?;
        // Time checks
        let now = Utc::now().timestamp();
        let skew = self.cfg.clock_skew_s;
        if raw.nbf.map(|n| now + skew < n).unwrap_or(false) { return Err(VerifyError::TimeWindow); }
        if now - skew > raw.exp { return Err(VerifyError::TimeWindow); }
        if raw.aud != self.cfg.audience && raw.aud != "aetheris-kernel" { return Err(VerifyError::Audience); }
        // Replay protection (in-proc)
        let jti_hex = bytes_to_hex(&raw.jti);
        {
            let mut lru = self.nonce_lru.lock().map_err(|e| VerifyError::LockPoison(e.to_string()))?;
            if lru.put(jti_hex.clone(), now).is_some() { return Err(VerifyError::Replay); }
        }
        Ok(Claims{ iss: raw.iss, sub: raw.sub, aud: raw.aud, exp: raw.exp, nbf: raw.nbf, iat: raw.iat, jti_hex, scopes: raw.scopes })
    }

    pub fn require(scopes: &[String], need: &str) -> bool {
        scopes.iter().any(|s| s == need)
    }
}

fn bytes_to_hex(b: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(b.len()*2);
    for &x in b {
        out.push(HEX[(x >> 4) as usize] as char);
        out.push(HEX[(x & 0x0f) as usize] as char);
    }
    out
}
