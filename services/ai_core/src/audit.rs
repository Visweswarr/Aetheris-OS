//! Phase 5-A audit trail.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::contracts::{sha256_hex, timestamp_90khz, to_canonical_cbor, AuditEvent};
use crate::error::{AiCoreError, Result};

#[derive(Debug, Clone, Default)]
pub struct AuditTrail {
    events: Arc<RwLock<Vec<AuditEvent>>>,
}

impl AuditTrail {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn record(&self, event: AuditEvent) {
        self.events.write().await.push(event);
    }

    pub async fn list(&self) -> Vec<AuditEvent> {
        self.events.read().await.clone()
    }

    pub async fn export_cbor(&self) -> Result<Vec<u8>> {
        to_canonical_cbor(&*self.events.read().await)
    }

    pub async fn clear(&self) {
        self.events.write().await.clear();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditChainEntry {
    pub sequence: u64,
    pub timestamp_90khz: u64,
    pub principal: String,
    pub event: String,
    pub previous_hash: String,
    pub hash: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TamperEvidentAuditLog {
    pub entries: Vec<AuditChainEntry>,
}

impl TamperEvidentAuditLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, principal: impl Into<String>, event: impl Into<String>) -> Result<()> {
        let sequence = self.entries.len() as u64;
        let timestamp = timestamp_90khz();
        let principal = principal.into();
        let event = event.into();
        let previous_hash = self
            .entries
            .last()
            .map(|entry| entry.hash.clone())
            .unwrap_or_else(|| "0".to_string());
        let hash = chain_hash(sequence, timestamp, &principal, &event, &previous_hash)?;
        self.entries.push(AuditChainEntry {
            sequence,
            timestamp_90khz: timestamp,
            principal,
            event,
            previous_hash,
            hash,
        });
        Ok(())
    }

    pub fn verify(&self) -> Result<()> {
        let mut previous_hash = "0".to_string();
        for entry in &self.entries {
            if entry.previous_hash != previous_hash {
                return Err(AiCoreError::DataCorruptionError(format!(
                    "audit hash-chain break at sequence {}",
                    entry.sequence
                )));
            }
            let expected = chain_hash(
                entry.sequence,
                entry.timestamp_90khz,
                &entry.principal,
                &entry.event,
                &entry.previous_hash,
            )?;
            if expected != entry.hash {
                return Err(AiCoreError::DataCorruptionError(format!(
                    "audit entry hash mismatch at sequence {}",
                    entry.sequence
                )));
            }
            previous_hash = entry.hash.clone();
        }
        Ok(())
    }
}

fn chain_hash(
    sequence: u64,
    timestamp_90khz: u64,
    principal: &str,
    event: &str,
    previous_hash: &str,
) -> Result<String> {
    let bytes = serde_json::to_vec(&serde_json::json!({
        "sequence": sequence,
        "timestamp_90khz": timestamp_90khz,
        "principal": principal,
        "event": event,
        "previous_hash": previous_hash,
    }))
    .map_err(|error| AiCoreError::SerializationError(error.to_string()))?;
    Ok(sha256_hex(&bytes))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::contracts::timestamp_90khz;

    use super::*;

    #[tokio::test]
    async fn audit_export_is_cbor() {
        let audit = AuditTrail::new();
        audit
            .record(AuditEvent {
                event_id: "audit-1".to_string(),
                actor: "test".to_string(),
                action: "model.load".to_string(),
                subject: "model-1".to_string(),
                allowed: true,
                reason: "ok".to_string(),
                timestamp_90khz: timestamp_90khz(),
                trace_id: None,
                metadata: HashMap::new(),
            })
            .await;

        assert!(!audit.export_cbor().await.unwrap().is_empty());
    }

    #[test]
    fn audit_chain_detects_modified_entry() {
        let mut audit = TamperEvidentAuditLog::new();
        audit.append("cli", "plan generated").unwrap();
        audit.append("cli", "tool admitted").unwrap();
        audit.verify().unwrap();

        audit.entries[1].event = "tool denied".to_string();
        assert!(audit.verify().is_err());
    }
}
