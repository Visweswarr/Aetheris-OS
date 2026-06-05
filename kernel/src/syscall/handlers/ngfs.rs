use crate::policy::engine::caps::CapVerifier;
use crate::policy::audit::{write as audit_write, AuditEntry};

pub fn ngfs_guard_read(token: Option<&[u8]>, path: &str) -> Result<(), &'static str> {
    if let Some(tok) = token {
        let pe = CapVerifier::from_env();
        let claims = pe.verify(tok).map_err(|_| "deny")?;
        pe.require(&claims.scopes, "ngfs.snapshot.read").map_err(|_| "deny")?;
        let _ = audit_write(&AuditEntry { ts90k: crate::time::get_time_90khz(), action: "ngfs.read".into(), result: "allow".into(), reason: None, iss: claims.iss, sub: claims.sub, aud: claims.aud, jti_hex: hex::encode(claims.jti), scopes: claims.scopes, endpoint: Some(path.into()) });
        Ok(())
    } else {
        if core::option_env!("AETHERIS_REQUIRE_TOKEN") == Some("1") { Err("deny") } else { Ok(()) }
    }
}

pub fn ngfs_guard_write(token: Option<&[u8]>, path: &str) -> Result<(), &'static str> {
    if let Some(tok) = token {
        let pe = CapVerifier::from_env();
        let claims = pe.verify(tok).map_err(|_| "deny")?;
        pe.require(&claims.scopes, "ngfs.snapshot.write").map_err(|_| "deny")?;
        let _ = audit_write(&AuditEntry { ts90k: crate::time::get_time_90khz(), action: "ngfs.write".into(), result: "allow".into(), reason: None, iss: claims.iss, sub: claims.sub, aud: claims.aud, jti_hex: hex::encode(claims.jti), scopes: claims.scopes, endpoint: Some(path.into()) });
        Ok(())
    } else {
        if core::option_env!("AETHERIS_REQUIRE_TOKEN") == Some("1") { Err("deny") } else { Ok(()) }
    }
}
