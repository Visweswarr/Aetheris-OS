use chrono::{Utc, Datelike};
use serde::Serialize;
use std::{path::PathBuf, fs, io::Write};

#[derive(Serialize, Clone, Debug)]
pub struct AuditEntry {
    pub ts90k: u64,
    pub action: String,
    pub result: String,
    #[serde(skip_serializing_if="Option::is_none")] pub reason: Option<String>,
    pub iss: String, pub sub: String, pub aud: String,
    pub jti_hex: String,
    pub scopes: Vec<String>,
    #[serde(skip_serializing_if="Option::is_none")] pub endpoint: Option<String>,
}

pub struct AuditWriter {
    root: PathBuf,
}

fn ts90k_now() -> u64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0));
    (now.as_secs() as u64) * 90_000 + (now.subsec_nanos() as u64) * 90_000 / 1_000_000_000
}

impl AuditWriter {
    pub fn from_env() -> Self {
        let root = std::env::var("NGFS_ROOT").unwrap_or_else(|_| "./data/ngfs".into());
        Self { root: root.into() }
    }

    pub fn path_for_today(&self) -> PathBuf {
        let now = Utc::now();
        let mut p = self.root.clone();
        p.push("audit"); p.push("captoken");
        fs::create_dir_all(&p).ok();
        p.push(format!("{:04}{:02}{:02}.cborl", now.year(), now.month(), now.day()));
        p
    }

    pub fn write(&self, entry: &AuditEntry) -> std::io::Result<()> {
        let path = self.path_for_today();
        let mut f = fs::OpenOptions::new().create(true).append(true).open(&path)?;
        let mut buf = Vec::new();
        ciborium::ser::into_writer(entry, &mut buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        buf.push(b'\n');
        f.write_all(&buf)?;
        Ok(())
    }

    pub fn allow(action: &str, iss: &str, sub: &str, aud: &str, jti_hex: &str, scopes: &[String], endpoint: Option<String>, ts90k: u64) -> AuditEntry {
        AuditEntry { ts90k, action: action.into(), result: "allow".into(), reason: None, iss: iss.into(), sub: sub.into(), aud: aud.into(), jti_hex: jti_hex.into(), scopes: scopes.to_vec(), endpoint }
    }
    pub fn deny(action: &str, iss: &str, sub: &str, aud: &str, jti_hex: &str, scopes: &[String], reason: &str, endpoint: Option<String>, ts90k: u64) -> AuditEntry {
        AuditEntry { ts90k, action: action.into(), result: "deny".into(), reason: Some(reason.into()), iss: iss.into(), sub: sub.into(), aud: aud.into(), jti_hex: jti_hex.into(), scopes: scopes.to_vec(), endpoint }
    }

    pub fn now_ts90k() -> u64 { ts90k_now() }
}
