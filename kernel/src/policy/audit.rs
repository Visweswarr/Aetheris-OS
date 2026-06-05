use alloc::string::String;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AuditEntry {
    pub ts90k: u64,
    pub action: String,
    pub result: String,
    pub reason: Option<String>,
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub jti_hex: String,
    pub scopes: Vec<String>,
    pub endpoint: Option<String>,
}

pub fn write(entry: &AuditEntry) -> core::result::Result<(), &'static str> {
    let ngfs_root = core::option_env!("NGFS_ROOT").unwrap_or("./data/ngfs");
    let date = crate::time::date_yyyymmdd();
    let path = crate::kformat!("{}/audit/captoken/{}.cborl", ngfs_root, date);
    if let Some(parent) = crate::fs::parent_dir(&path) { let _ = crate::fs::create_dir_all(parent); }
    let mut line = Vec::new();
    ciborium::ser::into_writer(entry, &mut line).map_err(|_| "ser")?;
    let mut with_nl = line;
    with_nl.push(b'\n');
    crate::fs::append(&path, &with_nl).map_err(|_| "append")
}
