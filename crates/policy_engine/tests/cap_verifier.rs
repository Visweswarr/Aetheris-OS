use std::{fs, path::PathBuf, process::Command};
use tempfile::TempDir;
use policy_engine::cap::CapVerifier;

fn manifest(path: &str) -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    root.join(path).to_string_lossy().to_string()
}

fn mint_token(tmp: &TempDir, scopes: &str, exp_secs: i64) -> (PathBuf, PathBuf, PathBuf) {
    let key = tmp.path().join("ed25519.sk");
    let tok = tmp.path().join("test.token");
    let jwks = tmp.path().join("jwks.json");
    fs::write(&key, vec![7u8; 32]).unwrap();
    let status = Command::new("cargo")
        .args([
            "run","-q","--manifest-path", &manifest("../../tools/captoken_minter/Cargo.toml"), "--",
            "--iss","did:dev:aetheris","--sub","did:dev:me","--aud","aetheris-kernel",
            "--exp", &format!("{}", exp_secs),
            "--scopes", scopes, "--key", key.to_str().unwrap(),
            "--out", tok.to_str().unwrap(), "--kid", "dev-1", "--out-jwks", jwks.to_str().unwrap()
        ])
        .status().unwrap();
    assert!(status.success());
    (key, tok, jwks)
}

fn write_issuer(tmp: &TempDir, jwks: &PathBuf, skew: i64) -> PathBuf {
    let issuer = tmp.path().join("issuer.json");
    fs::write(&issuer, format!(
        r#"{{"jwks_path":"{}","audience":"aetheris-kernel","clock_skew_s":{}}}"#,
        jwks.display(), skew
    )).unwrap();
    issuer
}

#[test]
fn happy_path_and_scope() {
    let tmp = TempDir::new().unwrap();
    let (_k, tok, jwks) = mint_token(&tmp, "intent.emit,ngfs.snapshot.write", 900);
    let issuer = write_issuer(&tmp, &jwks, 120);
    std::env::set_var("AETHERIS_ISSUER", issuer.as_os_str());
    let v = CapVerifier::from_env().expect("verifier");
    let bytes = fs::read(&tok).unwrap();
    let claims = v.verify(&bytes).expect("verify ok");
    assert!(policy_engine::cap::CapVerifier::require(&claims.scopes, "intent.emit"));
}

#[test]
fn bad_audience() {
    let tmp = TempDir::new().unwrap();
    let (_k, tok, jwks) = mint_token(&tmp, "intent.emit", 900);
    let issuer = tmp.path().join("issuer.json");
    fs::write(&issuer, format!(
        r#"{{"jwks_path":"{}","audience":"wrong-aud","clock_skew_s":0}}"#,
        jwks.display()
    )).unwrap();
    std::env::set_var("AETHERIS_ISSUER", issuer.as_os_str());
    let v = CapVerifier::from_env().unwrap();
    let bytes = fs::read(&tok).unwrap();
    assert!(v.verify(&bytes).is_err());
}

#[test]
fn expired_token_respected_with_skew() {
    let tmp = TempDir::new().unwrap();
    let (_k, tok, jwks) = mint_token(&tmp, "intent.emit", -1);
    let issuer = write_issuer(&tmp, &jwks, 0);
    std::env::set_var("AETHERIS_ISSUER", issuer.as_os_str());
    let v = CapVerifier::from_env().unwrap();
    let bytes = fs::read(&tok).unwrap();
    assert!(v.verify(&bytes).is_err());
}

#[test]
fn replay_protection() {
    let tmp = TempDir::new().unwrap();
    let (_k, tok, jwks) = mint_token(&tmp, "intent.emit", 900);
    let issuer = write_issuer(&tmp, &jwks, 120);
    std::env::set_var("AETHERIS_ISSUER", issuer.as_os_str());
    let v = CapVerifier::from_env().unwrap();
    let bytes = fs::read(&tok).unwrap();
    let c1 = v.verify(&bytes).unwrap();
    assert!(v.check_and_remember_nonce(&c1.jti_hex, 0).is_err());
}

#[test]
fn bad_signature_rejected() {
    let tmp = TempDir::new().unwrap();
    let (_k, tok, jwks) = mint_token(&tmp, "intent.emit", 900);
    let issuer = write_issuer(&tmp, &jwks, 120);
    std::env::set_var("AETHERIS_ISSUER", issuer.as_os_str());
    let v = CapVerifier::from_env().unwrap();
    let mut bytes = fs::read(&tok).unwrap();
    if let Some(last) = bytes.last_mut() { *last ^= 0x01; }
    assert!(v.verify(&bytes).is_err());
}
