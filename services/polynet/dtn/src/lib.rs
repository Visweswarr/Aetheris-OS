//! DTN Envelope Service for Polymera OS
//! 
//! This module provides Delay-Tolerant Networking envelope functionality with:
//! - TTL-based expiration
//! - Nonce-based duplicate detection
//! - Acknowledgment tracking
//! - Multiple routing strategies
//! - Cryptographic security
//! - Persistent storage

pub mod envelope;
pub mod store;
pub mod routing;
pub mod security;

// Re-export main types
pub use envelope::{
    DtnEnvelope, RoutingInfo, AckRequirements, SecurityInfo,
    RoutingStrategy, AckType, AckStatus, SecurityLevel,
    EnvelopeError,
};
pub use store::{DtnStore, StoreConfig, StoreStats, EvictionPolicy, StoreError};
pub use routing::{DtnRouter, RoutingNode, RoutingDecision, RoutingError};
pub use security::{DtnSecurity, SecurityError};

// Include generated protobuf code
pub mod proto {
    tonic::include_proto!("dtn");
}

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// Main DTN service that coordinates envelope operations
pub struct DtnService {
    store: Arc<DtnStore>,
    router: Arc<RwLock<DtnRouter>>,
    security: Arc<RwLock<DtnSecurity>>,
}

impl DtnService {
    /// Create a new DTN service
    pub fn new(store_config: StoreConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let store = Arc::new(DtnStore::new(store_config)?);
        let router = Arc::new(RwLock::new(DtnRouter::new()));
        let security = Arc::new(RwLock::new(DtnSecurity::new()));
        
        info!("DTN service initialized");
        
        Ok(Self {
            store,
            router,
            security,
        })
    }
    
    /// Send an envelope
    pub async fn send_envelope(&self, envelope: DtnEnvelope) -> Result<String, Box<dyn std::error::Error>> {
        let envelope_id = envelope.id.clone();
        
        // Check if envelope is expired
        if envelope.is_expired() {
            warn!("Attempting to send expired envelope: {}", envelope_id);
            return Err("Cannot send expired envelope".into());
        }
        
        // Sign the envelope if we have a signing key
        let mut envelope_to_send = envelope;
        if let Ok(mut security) = self.security.write().await {
            if security.get_signing_key().is_some() {
                if let Err(e) = security.sign_envelope(&mut envelope_to_send) {
                    warn!("Failed to sign envelope {}: {}", envelope_id, e);
                }
            }
        }
        
        // Store the envelope
        self.store.store_envelope(envelope_to_send.clone()).await?;
        
        // Route the envelope
        let routing_decisions = {
            let router = self.router.read().await;
            router.route_envelope(&envelope_to_send, &*self.store).await?
        };
        
        info!("Envelope {} sent with {} routing decisions", envelope_id, routing_decisions.len());
        
        Ok(envelope_id)
    }
    
    /// Receive envelopes for a node
    pub async fn receive_envelopes(
        &self,
        node_id: &str,
        max_envelopes: usize,
    ) -> Result<Vec<DtnEnvelope>, Box<dyn std::error::Error>> {
        // Get envelopes destined for this node
        let envelopes = self.store.list_envelopes_by_destination(node_id).await;
        
        // Filter out expired envelopes
        let valid_envelopes: Vec<DtnEnvelope> = envelopes
            .into_iter()
            .filter(|env| !env.is_expired())
            .take(max_envelopes)
            .collect();
        
        info!("Node {} received {} envelopes", node_id, valid_envelopes.len());
        
        Ok(valid_envelopes)
    }
    
    /// Acknowledge envelope receipt
    pub async fn acknowledge_envelope(
        &self,
        envelope_id: &str,
        ack_node: &str,
        ack_type: AckType,
        status: AckStatus,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Retrieve the envelope
        if let Some(mut envelope) = self.store.retrieve_envelope(envelope_id).await? {
            // Record the acknowledgment
            envelope.record_acknowledgment(
                ack_node.to_string(),
                ack_type,
                status,
                message.to_string(),
            );
            
            // Update the envelope in storage
            self.store.remove_envelope(envelope_id).await?;
            self.store.store_envelope(envelope).await?;
            
            info!("Envelope {} acknowledged by {} with status {:?}", envelope_id, ack_node, status);
            Ok(())
        } else {
            Err("Envelope not found".into())
        }
    }
    
    /// Query envelope status
    pub async fn query_envelope(&self, envelope_id: &str) -> Result<Option<DtnEnvelope>, Box<dyn std::error::Error>> {
        let envelope = self.store.retrieve_envelope(envelope_id).await?;
        Ok(envelope)
    }
    
    /// Clean up expired envelopes
    pub async fn cleanup_expired(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let cleaned = self.store.cleanup_expired().await?;
        info!("Cleaned up {} expired envelopes", cleaned);
        Ok(cleaned)
    }
    
    /// Get service statistics
    pub async fn get_stats(&self) -> Result<ServiceStats, Box<dyn std::error::Error>> {
        let store_stats = self.store.get_stats().await;
        let router_stats = self.router.read().await.get_routing_stats();
        
        let stats = ServiceStats {
            store: store_stats,
            routing: router_stats,
        };
        
        Ok(stats)
    }
    
    /// Add a routing node
    pub async fn add_routing_node(&self, node: RoutingNode) -> Result<(), Box<dyn std::error::Error>> {
        let mut router = self.router.write().await;
        router.add_node(node);
        Ok(())
    }
    
    /// Remove a routing node
    pub async fn remove_routing_node(&self, node_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut router = self.router.write().await;
        router.remove_node(node_id);
        Ok(())
    }
    
    /// Generate a new signing key
    pub async fn generate_signing_key(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut security = self.security.write().await;
        let public_key = security.generate_signing_key()?;
        Ok(public_key)
    }
    
    /// Import a verifying key
    pub async fn import_verifying_key(&self, key_bytes: &[u8], key_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut security = self.security.write().await;
        security.import_verifying_key(key_bytes, key_id)?;
        Ok(())
    }
    
    /// Verify envelope signature
    pub async fn verify_envelope_signature(&self, envelope: &DtnEnvelope) -> Result<bool, Box<dyn std::error::Error>> {
        let security = self.security.read().await;
        let is_valid = security.verify_envelope_signature(envelope)?;
        Ok(is_valid)
    }
}

/// Combined service statistics
#[derive(Debug, Clone)]
pub struct ServiceStats {
    pub store: StoreStats,
    pub routing: routing::RoutingStats,
}

/// Configuration for the DTN service
#[derive(Debug, Clone)]
pub struct DtnServiceConfig {
    pub store_config: StoreConfig,
    pub enable_security: bool,
    pub enable_routing: bool,
    pub cleanup_interval: u64,
}

impl Default for DtnServiceConfig {
    fn default() -> Self {
        Self {
            store_config: StoreConfig::default(),
            enable_security: true,
            enable_routing: true,
            cleanup_interval: 300, // 5 minutes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_service_creation() {
        let config = StoreConfig::default();
        let service = DtnService::new(config).unwrap();
        
        let stats = service.get_stats().await.unwrap();
        assert_eq!(stats.store.total_envelopes, 0);
        assert_eq!(stats.routing.total_nodes, 0);
    }
    
    #[tokio::test]
    async fn test_send_receive_envelope() {
        let config = StoreConfig::default();
        let service = DtnService::new(config).unwrap();
        
        let envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        // Send envelope
        let envelope_id = service.send_envelope(envelope).await.unwrap();
        
        // Receive envelopes
        let received = service.receive_envelopes("did:polynet:dest", 10).await.unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].id, envelope_id);
    }
    
    #[tokio::test]
    async fn test_envelope_acknowledgment() {
        let config = StoreConfig::default();
        let service = DtnService::new(config).unwrap();
        
        let envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        // Send envelope
        let envelope_id = service.send_envelope(envelope).await.unwrap();
        
        // Acknowledge envelope
        service.acknowledge_envelope(
            &envelope_id,
            "did:polynet:dest",
            AckType::Delivery,
            AckStatus::Success,
            "Received successfully",
        ).await.unwrap();
        
        // Query envelope to check acknowledgment
        let updated_envelope = service.query_envelope(&envelope_id).await.unwrap().unwrap();
        assert_eq!(updated_envelope.ack_requirements.ack_records.len(), 1);
        assert_eq!(updated_envelope.ack_requirements.ack_records[0].ack_node, "did:polynet:dest");
    }
    
    #[tokio::test]
    async fn test_expired_envelope_cleanup() {
        let config = StoreConfig::default();
        let service = DtnService::new(config).unwrap();
        
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            1, // 1 second TTL
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        // Send envelope
        service.send_envelope(envelope).await.unwrap();
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Cleanup should remove expired envelope
        let cleaned = service.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 1);
        
        let stats = service.get_stats().await.unwrap();
        assert_eq!(stats.store.total_expired, 1);
    }
    
    #[tokio::test]
    async fn test_routing_node_management() {
        let config = StoreConfig::default();
        let service = DtnService::new(config).unwrap();
        
        let node = RoutingNode {
            node_id: "did:polynet:test".to_string(),
            capabilities: std::collections::HashSet::new(),
            location: None,
            interfaces: Vec::new(),
            last_seen: std::time::SystemTime::now(),
            storage_capacity: 1024 * 1024,
            available_storage: 512 * 1024,
        };
        
        // Add node
        service.add_routing_node(node).await.unwrap();
        
        let stats = service.get_stats().await.unwrap();
        assert_eq!(stats.routing.total_nodes, 1);
        
        // Remove node
        service.remove_routing_node("did:polynet:test").await.unwrap();
        
        let stats = service.get_stats().await.unwrap();
        assert_eq!(stats.routing.total_nodes, 0);
    }
    
    #[tokio::test]
    async fn test_security_operations() {
        let config = StoreConfig::default();
        let service = DtnService::new(config).unwrap();
        
        // Generate signing key
        let public_key = service.generate_signing_key().await.unwrap();
        assert_eq!(public_key.len(), 32);
        
        // Import verifying key
        service.import_verifying_key(&public_key, "test_key").await.unwrap();
        
        // Create and sign envelope
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        // Send envelope (should be signed automatically)
        let envelope_id = service.send_envelope(envelope).await.unwrap();
        
        // Query envelope and verify signature
        let retrieved_envelope = service.query_envelope(&envelope_id).await.unwrap().unwrap();
        let is_valid = service.verify_envelope_signature(&retrieved_envelope).await.unwrap();
        assert!(is_valid);
    }
}
