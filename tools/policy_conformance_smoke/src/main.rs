use std::{fs, io::{Read, Write}, path::PathBuf, process::{Command, Stdio}, thread, time::Duration};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct EnvelopeV1 { v: u8, captoken: Vec<u8>, payload: Vec<u8> }

#[derive(Serialize, Deserialize, Debug)]
struct IntentResponse { status: String, #[serde(default)] preview_id: Option<String>, #[serde(default)] commit_id: Option<String>, #[serde(default)] message: Option<String> }

fn manifest(path: &str) -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    root.join(path).to_string_lossy().to_string()
}

fn mint_token(workspace_root: &str, scopes: &str, exp_secs: i64, tmpdir: &tempfile::TempDir) -> (PathBuf, PathBuf, PathBuf) {
    let key = tmpdir.path().join("ed25519.sk");
    let tok = tmpdir.path().join("test.token");
    let jwks = tmpdir.path().join("jwks.json");
    fs::write(&key, vec![7u8; 32]).unwrap();
    let status = Command::new("cargo")
        .args([
            "run","-q","--manifest-path", &manifest("tools/captoken_minter/Cargo.toml"), "--",
            "--iss","did:dev:aetheris","--sub","did:dev:smoke","--aud","aetheris-kernel",
            "--exp", &format!("{}", exp_secs),
            "--scopes", scopes, "--key", key.to_str().unwrap(),
            "--out", tok.to_str().unwrap(), "--kid", "dev-1", "--out-jwks", jwks.to_str().unwrap()
        ])
        .status().unwrap();
    assert!(status.success());
    (key, tok, jwks)
}

fn write_issuer(jwks: &PathBuf, tmp: &tempfile::TempDir) -> PathBuf {
    let issuer = tmp.path().join("issuer.json");
    fs::write(&issuer, format!(
        r#"{{"jwks_path":"{}","audience":"aetheris-kernel","clock_skew_s":120}}"#,
        jwks.display()
    )).unwrap();
    issuer
}

fn send_envelope_unix(sock_path: &str, token_path: &PathBuf) -> IntentResponse {
    let mut stream = std::os::unix::net::UnixStream::connect(sock_path).unwrap();
    let token = fs::read(token_path).unwrap();
    let env = EnvelopeV1 { v: 1, captoken: token, payload: vec![0xA0] };
    let mut buf = Vec::new();
    serde_cbor::to_writer(&mut buf, &env).unwrap();
    let len = (buf.len() as u32).to_be_bytes();
    stream.write_all(&len).unwrap();
    stream.write_all(&buf).unwrap();
    let mut l = [0u8; 4];
    stream.read_exact(&mut l).unwrap();
    let blen = u32::from_be_bytes(l) as usize;
    let mut b = vec![0u8; blen];
    stream.read_exact(&mut b).unwrap();
    serde_cbor::from_slice(&b).unwrap()
}

fn main() {
    // Start echo server
    let sock = "/tmp/aetheris-intent.sock";
    let _ = std::fs::remove_file(sock);
    let tmp = tempfile::TempDir::new().unwrap();
    let (_k_allow, tok_allow, jwks) = mint_token(".", "intent.emit", 900, &tmp);
    let (_k_deny, tok_deny, _jwks2) = mint_token(".", "", 900, &tmp);
    let issuer = write_issuer(&jwks, &tmp);

    std::env::set_var("AETHERIS_ISSUER", &issuer);
    let ngfs_root = std::env::var("NGFS_ROOT").unwrap_or_else(|_| "./data/ngfs".into());
    std::fs::create_dir_all(format!("{}/audit/captoken", ngfs_root)).ok();

    let server = Command::new("./target/debug/intent-echo-server")
        .args(["--unix", sock])
        .env("AETHERIS_ISSUER", issuer.as_os_str())
        .env("NGFS_ROOT", &ngfs_root)
        .stdout(Stdio::null()).stderr(Stdio::null())
        .spawn()
        .expect("start server");
    // Give server a moment
    thread::sleep(Duration::from_millis(500));

    // Allow
    let r1 = send_envelope_unix(sock, &tok_allow);
    assert_eq!(r1.status, "ok", "expected ok for allow token: {:?}", r1);
    // Deny
    let r2 = send_envelope_unix(sock, &tok_deny);
    assert_eq!(r2.status, "deny", "expected deny for no-scope token: {:?}", r2);

    // Read audit lines
    let today = chrono::Utc::now().format("%Y%m%d").to_string();
    let audit_path = format!("{}/audit/captoken/{}.cborl", ngfs_root, today);
    let data = fs::read(&audit_path).expect("audit file present");
    let count = data.split(|b| *b == b'\n').filter(|l| !l.is_empty()).count();
    assert!(count >= 2, "expected at least 2 audit lines, got {}", count);

    // Cleanup server
    let _ = Command::new("pkill").args(["-f", "intent-echo-server"]).status();
}
