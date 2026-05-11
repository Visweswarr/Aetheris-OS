//! Phase 5-A audit trail.

use std::sync::Arc;

use tokio::sync::RwLock;

use crate::contracts::{to_canonical_cbor, AuditEvent};
use crate::error::Result;

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
}
