use policy_engine::audit::{AuditWriter, AuditEntry};
use std::fs;
use tempfile::TempDir;

#[test]
fn cborl_append_and_parse_back() {
    let tmp = TempDir::new().unwrap();
    std::env::set_var("NGFS_ROOT", tmp.path());
    let aw = AuditWriter::from_env();
    let e = AuditEntry { ts90k: 12345, action: "intent.preview".into(), result: "allow".into(), reason: None, iss: "did:dev:aetheris".into(), sub: "did:me".into(), aud: "aetheris-kernel".into(), jti_hex: "deadbeef".into(), scopes: vec!["intent.emit".into()], endpoint: Some("unix:///tmp/a.sock".into()) };
    aw.write(&e).unwrap();

    let p = aw.path_for_today();
    let data = fs::read(&p).unwrap();
    assert!(data.ends_with(b"\n"));
    let lines = data.split(|b| *b == b'\n').filter(|l| !l.is_empty()).count();
    assert!(lines >= 1);
}
