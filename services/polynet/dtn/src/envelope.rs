use crate::proto::dtn::{
    envelope::*, routing_info::*, ack_requirements::*, security_info::*,
    routing_strategy::*, ack_type::*, ack_status::*, security_level::*,
    envelope_status::*, eviction_policy::*, interface_status::*,
    Envelope as ProtoEnvelope, RoutingInfo as ProtoRoutingInfo,
    AckRequirements as ProtoAckRequirements, SecurityInfo as ProtoSecurityInfo,
    NodeInfo as ProtoNodeInfo, StoreInfo as ProtoStoreInfo,
};
use prost_types::{Timestamp, Any};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::{debug, info, warn, error};
use uuid::Uuid;
use sha2::{Sha256, Digest};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::Rng;

pub mod store;
pub mod routing;
pub mod security;

// Include generated protobuf code
pub mod proto {
    tonic::include_proto!("dtn");
}

#[derive(Debug, Error)]
pub enum EnvelopeError {
    #[error("Invalid envelope: {0}")]
    InvalidEnvelope(String),
    
    #[error("Envelope expired: {0}")]
    EnvelopeExpired(String),
    
    #[error("Duplicate envelope: {0}")]
    DuplicateEnvelope(String),
    
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Routing error: {0}")]
    RoutingError(String),
    
    #[error("Acknowledgment error: {0}")]
    AcknowledgmentError(String),
    
    #[error("Security error: {0}")]
    SecurityError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtnEnvelope {
    pub id: String,
    pub source: String,
    pub destination: String,
    pub ttl: u64,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
    pub nonce: Vec<u8>,
    pub priority: u32,
    pub message_type: String,
    pub payload: Vec<u8>,
    pub routing: RoutingInfo,
    pub ack_requirements: AckRequirements,
    pub security: SecurityInfo,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingInfo {
    pub intermediate_nodes: Vec<String>,
    pub max_hops: u32,
    pub hop_count: u32,
    pub strategy: RoutingStrategy,
    pub geo_constraints: Option<GeographicConstraints>,
    pub net_constraints: Option<NetworkConstraints>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicConstraints {
    pub min_latitude: f64,
    pub max_latitude: f64,
    pub min_longitude: f64,
    pub max_longitude: f64,
    pub max_distance: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConstraints {
    pub required_capabilities: Vec<String>,
    pub min_bandwidth: u64,
    pub max_latency: u64,
    pub min_security_level: SecurityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckRequirements {
    pub required: bool,
    pub ack_type: AckType,
    pub ack_timeout: u64,
    pub max_retries: u32,
    pub required_ack_nodes: Vec<String>,
    pub ack_records: Vec<AckRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckRecord {
    pub ack_node: String,
    pub ack_type: AckType,
    pub ack_time: SystemTime,
    pub status: AckStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInfo {
    pub signature: Vec<u8>,
    pub signer_public_key: Vec<u8>,
    pub signature_algorithm: String,
    pub payload_hash: Vec<u8>,
    pub hash_algorithm: String,
    pub encryption: Option<EncryptionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionInfo {
    pub encrypted: bool,
    pub algorithm: String,
    pub iv: Vec<u8>,
    pub encrypted_key: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingStrategy {
    Flood,
    Directed,
    Opportunistic,
    Epidemic,
    SprayAndWait,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AckType {
    Delivery,
    Forward,
    Processing,
    Storage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AckStatus {
    Success,
    Failure,
    Partial,
    Timeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl DtnEnvelope {
    /// Create a new DTN envelope
    pub fn new(
        source: String,
        destination: String,
        ttl: u64,
        message_type: String,
        payload: Vec<u8>,
        priority: u32,
    ) -> Self {
        let now = SystemTime::now();
        let expires_at = now + Duration::from_secs(ttl);
        let nonce = Self::generate_nonce();
        let id = Uuid::new_v4().to_string();
        
        let routing = RoutingInfo {
            intermediate_nodes: Vec::new(),
            max_hops: 10,
            hop_count: 0,
            strategy: RoutingStrategy::Directed,
            geo_constraints: None,
            net_constraints: None,
        };
        
        let ack_requirements = AckRequirements {
            required: false,
            ack_type: AckType::Delivery,
            ack_timeout: 300, // 5 minutes
            max_retries: 3,
            required_ack_nodes: Vec::new(),
            ack_records: Vec::new(),
        };
        
        let security = SecurityInfo {
            signature: Vec::new(),
            signer_public_key: Vec::new(),
            signature_algorithm: "ed25519".to_string(),
            payload_hash: Self::hash_payload(&payload),
            hash_algorithm: "sha256".to_string(),
            encryption: None,
        };
        
        Self {
            id,
            source,
            destination,
            ttl,
            created_at: now,
            expires_at,
            nonce,
            priority,
            message_type,
            payload,
            routing,
            ack_requirements,
            security,
            metadata: HashMap::new(),
        }
    }
    
    /// Check if envelope is expired
    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }
    
    /// Get remaining TTL in seconds
    pub fn remaining_ttl(&self) -> u64 {
        let now = SystemTime::now();
        if now > self.expires_at {
            0
        } else {
            self.expires_at.duration_since(now)
                .unwrap_or(Duration::from_secs(0))
                .as_secs()
        }
    }
    
    /// Increment hop count
    pub fn increment_hop_count(&mut self) -> Result<(), EnvelopeError> {
        if self.routing.hop_count >= self.routing.max_hops {
            return Err(EnvelopeError::RoutingError(
                "Maximum hop count exceeded".to_string()
            ));
        }
        self.routing.hop_count += 1;
        Ok(())
    }
    
    /// Add intermediate node
    pub fn add_intermediate_node(&mut self, node: String) {
        if !self.routing.intermediate_nodes.contains(&node) {
            self.routing.intermediate_nodes.push(node);
        }
    }
    
    /// Set routing strategy
    pub fn set_routing_strategy(&mut self, strategy: RoutingStrategy) {
        self.routing.strategy = strategy;
    }
    
    /// Set geographic constraints
    pub fn set_geographic_constraints(&mut self, constraints: GeographicConstraints) {
        self.routing.geo_constraints = Some(constraints);
    }
    
    /// Set network constraints
    pub fn set_network_constraints(&mut self, constraints: NetworkConstraints) {
        self.routing.net_constraints = Some(constraints);
    }
    
    /// Enable acknowledgments
    pub fn enable_acknowledgments(&mut self, ack_type: AckType, timeout: u64) {
        self.ack_requirements.required = true;
        self.ack_requirements.ack_type = ack_type;
        self.ack_requirements.ack_timeout = timeout;
    }
    
    /// Add required acknowledgment node
    pub fn add_required_ack_node(&mut self, node: String) {
        if !self.ack_requirements.required_ack_nodes.contains(&node) {
            self.ack_requirements.required_ack_nodes.push(node);
        }
    }
    
    /// Record acknowledgment
    pub fn record_acknowledgment(
        &mut self,
        ack_node: String,
        ack_type: AckType,
        status: AckStatus,
        message: String,
    ) {
        let ack_record = AckRecord {
            ack_node,
            ack_type,
            ack_time: SystemTime::now(),
            status,
            message,
        };
        self.ack_requirements.ack_records.push(ack_record);
    }
    
    /// Check if all required acknowledgments are received
    pub fn all_acks_received(&self) -> bool {
        if !self.ack_requirements.required {
            return true;
        }
        
        let required_nodes = &self.ack_requirements.required_ack_nodes;
        let received_nodes: std::collections::HashSet<_> = self.ack_requirements.ack_records
            .iter()
            .filter(|ack| ack.status == AckStatus::Success)
            .map(|ack| &ack.ack_node)
            .collect();
        
        required_nodes.iter().all(|node| received_nodes.contains(node))
    }
    
    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Sign the envelope
    pub fn sign(&mut self, signing_key: &SigningKey) -> Result<(), EnvelopeError> {
        let message = self.create_signature_message();
        let signature = signing_key.sign(&message);
        
        self.security.signature = signature.to_bytes().to_vec();
        self.security.signer_public_key = signing_key.verifying_key().to_bytes().to_vec();
        
        Ok(())
    }
    
    /// Verify the envelope signature
    pub fn verify_signature(&self) -> Result<bool, EnvelopeError> {
        if self.security.signature.is_empty() || self.security.signer_public_key.is_empty() {
            return Ok(false);
        }
        
        let verifying_key = VerifyingKey::from_bytes(&self.security.signer_public_key)
            .map_err(|e| EnvelopeError::SecurityError(e.to_string()))?;
        
        let signature = Signature::from_bytes(&self.security.signature)
            .map_err(|e| EnvelopeError::SecurityError(e.to_string()))?;
        
        let message = self.create_signature_message();
        
        Ok(verifying_key.verify(&message, &signature).is_ok())
    }
    
    /// Create signature message for signing
    fn create_signature_message(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(self.id.as_bytes());
        hasher.update(self.source.as_bytes());
        hasher.update(self.destination.as_bytes());
        hasher.update(self.ttl.to_le_bytes());
        hasher.update(self.created_at.duration_since(UNIX_EPOCH).unwrap().as_secs().to_le_bytes());
        hasher.update(&self.nonce);
        hasher.update(self.priority.to_le_bytes());
        hasher.update(self.message_type.as_bytes());
        hasher.update(&self.payload);
        hasher.update(&self.security.payload_hash);
        hasher.finalize().to_vec()
    }
    
    /// Generate a unique nonce
    fn generate_nonce() -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let mut nonce = vec![0u8; 16];
        rng.fill(&mut nonce);
        nonce
    }
    
    /// Hash payload for integrity
    fn hash_payload(payload: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(payload);
        hasher.finalize().to_vec()
    }
    
    /// Convert to protobuf
    pub fn to_proto(&self) -> ProtoEnvelope {
        ProtoEnvelope {
            id: self.id.clone(),
            source: self.source.clone(),
            destination: self.destination.clone(),
            ttl: self.ttl,
            created_at: Some(self.created_at.into()),
            expires_at: Some(self.expires_at.into()),
            nonce: self.nonce.clone(),
            priority: self.priority,
            message_type: self.message_type.clone(),
            payload: Some(Any {
                type_url: self.message_type.clone(),
                value: self.payload.clone(),
            }),
            routing: Some(self.routing.to_proto()),
            ack_requirements: Some(self.ack_requirements.to_proto()),
            security: Some(self.security.to_proto()),
            metadata: self.metadata.clone(),
        }
    }
    
    /// Create from protobuf
    pub fn from_proto(proto: ProtoEnvelope) -> Result<Self, EnvelopeError> {
        let created_at = proto.created_at
            .ok_or_else(|| EnvelopeError::InvalidEnvelope("Missing created_at".to_string()))?
            .try_into()
            .map_err(|_| EnvelopeError::InvalidEnvelope("Invalid created_at".to_string()))?;
        
        let expires_at = proto.expires_at
            .ok_or_else(|| EnvelopeError::InvalidEnvelope("Missing expires_at".to_string()))?
            .try_into()
            .map_err(|_| EnvelopeError::InvalidEnvelope("Invalid expires_at".to_string()))?;
        
        let payload = proto.payload
            .map(|any| any.value)
            .unwrap_or_default();
        
        let routing = proto.routing
            .map(RoutingInfo::from_proto)
            .transpose()?
            .unwrap_or_default();
        
        let ack_requirements = proto.ack_requirements
            .map(AckRequirements::from_proto)
            .transpose()?
            .unwrap_or_default();
        
        let security = proto.security
            .map(SecurityInfo::from_proto)
            .transpose()?
            .unwrap_or_default();
        
        Ok(Self {
            id: proto.id,
            source: proto.source,
            destination: proto.destination,
            ttl: proto.ttl,
            created_at,
            expires_at,
            nonce: proto.nonce,
            priority: proto.priority,
            message_type: proto.message_type,
            payload,
            routing,
            ack_requirements,
            security,
            metadata: proto.metadata,
        })
    }
}

impl RoutingInfo {
    pub fn to_proto(&self) -> ProtoRoutingInfo {
        ProtoRoutingInfo {
            intermediate_nodes: self.intermediate_nodes.clone(),
            max_hops: self.max_hops,
            hop_count: self.hop_count,
            strategy: self.strategy.to_proto(),
            geo_constraints: self.geo_constraints.as_ref().map(|g| g.to_proto()),
            net_constraints: self.net_constraints.as_ref().map(|n| n.to_proto()),
        }
    }
    
    pub fn from_proto(proto: ProtoRoutingInfo) -> Result<Self, EnvelopeError> {
        Ok(Self {
            intermediate_nodes: proto.intermediate_nodes,
            max_hops: proto.max_hops,
            hop_count: proto.hop_count,
            strategy: RoutingStrategy::from_proto(proto.strategy)?,
            geo_constraints: proto.geo_constraints.map(GeographicConstraints::from_proto).transpose()?,
            net_constraints: proto.net_constraints.map(NetworkConstraints::from_proto).transpose()?,
        })
    }
}

impl AckRequirements {
    pub fn to_proto(&self) -> ProtoAckRequirements {
        ProtoAckRequirements {
            required: self.required,
            ack_type: self.ack_type.to_proto(),
            ack_timeout: self.ack_timeout,
            max_retries: self.max_retries,
            required_ack_nodes: self.required_ack_nodes.clone(),
            ack_records: self.ack_records.iter().map(|r| r.to_proto()).collect(),
        }
    }
    
    pub fn from_proto(proto: ProtoAckRequirements) -> Result<Self, EnvelopeError> {
        Ok(Self {
            required: proto.required,
            ack_type: AckType::from_proto(proto.ack_type)?,
            ack_timeout: proto.ack_timeout,
            max_retries: proto.max_retries,
            required_ack_nodes: proto.required_ack_nodes,
            ack_records: proto.ack_records.into_iter().map(AckRecord::from_proto).collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl SecurityInfo {
    pub fn to_proto(&self) -> ProtoSecurityInfo {
        ProtoSecurityInfo {
            signature: self.signature.clone(),
            signer_public_key: self.signer_public_key.clone(),
            signature_algorithm: self.signature_algorithm.clone(),
            payload_hash: self.payload_hash.clone(),
            hash_algorithm: self.hash_algorithm.clone(),
            encryption: self.encryption.as_ref().map(|e| e.to_proto()),
        }
    }
    
    pub fn from_proto(proto: ProtoSecurityInfo) -> Result<Self, EnvelopeError> {
        Ok(Self {
            signature: proto.signature,
            signer_public_key: proto.signer_public_key,
            signature_algorithm: proto.signature_algorithm,
            payload_hash: proto.payload_hash,
            hash_algorithm: proto.hash_algorithm,
            encryption: proto.encryption.map(EncryptionInfo::from_proto).transpose()?,
        })
    }
}

impl RoutingStrategy {
    pub fn to_proto(&self) -> i32 {
        match self {
            RoutingStrategy::Flood => 1,
            RoutingStrategy::Directed => 2,
            RoutingStrategy::Opportunistic => 3,
            RoutingStrategy::Epidemic => 4,
            RoutingStrategy::SprayAndWait => 5,
        }
    }
    
    pub fn from_proto(proto: i32) -> Result<Self, EnvelopeError> {
        match proto {
            1 => Ok(RoutingStrategy::Flood),
            2 => Ok(RoutingStrategy::Directed),
            3 => Ok(RoutingStrategy::Opportunistic),
            4 => Ok(RoutingStrategy::Epidemic),
            5 => Ok(RoutingStrategy::SprayAndWait),
            _ => Err(EnvelopeError::InvalidEnvelope(
                format!("Unknown routing strategy: {}", proto)
            )),
        }
    }
}

impl AckType {
    pub fn to_proto(&self) -> i32 {
        match self {
            AckType::Delivery => 1,
            AckType::Forward => 2,
            AckType::Processing => 3,
            AckType::Storage => 4,
        }
    }
    
    pub fn from_proto(proto: i32) -> Result<Self, EnvelopeError> {
        match proto {
            1 => Ok(AckType::Delivery),
            2 => Ok(AckType::Forward),
            3 => Ok(AckType::Processing),
            4 => Ok(AckType::Storage),
            _ => Err(EnvelopeError::InvalidEnvelope(
                format!("Unknown ack type: {}", proto)
            )),
        }
    }
}

impl AckStatus {
    pub fn to_proto(&self) -> i32 {
        match self {
            AckStatus::Success => 1,
            AckStatus::Failure => 2,
            AckStatus::Partial => 3,
            AckStatus::Timeout => 4,
        }
    }
    
    pub fn from_proto(proto: i32) -> Result<Self, EnvelopeError> {
        match proto {
            1 => Ok(AckStatus::Success),
            2 => Ok(AckStatus::Failure),
            3 => Ok(AckStatus::Partial),
            4 => Ok(AckStatus::Timeout),
            _ => Err(EnvelopeError::InvalidEnvelope(
                format!("Unknown ack status: {}", proto)
            )),
        }
    }
}

impl GeographicConstraints {
    pub fn to_proto(&self) -> crate::proto::dtn::GeographicConstraints {
        crate::proto::dtn::GeographicConstraints {
            min_latitude: self.min_latitude,
            max_latitude: self.max_latitude,
            min_longitude: self.min_longitude,
            max_longitude: self.max_longitude,
            max_distance: self.max_distance,
        }
    }
    
    pub fn from_proto(proto: crate::proto::dtn::GeographicConstraints) -> Result<Self, EnvelopeError> {
        Ok(Self {
            min_latitude: proto.min_latitude,
            max_latitude: proto.max_latitude,
            min_longitude: proto.min_longitude,
            max_longitude: proto.max_longitude,
            max_distance: proto.max_distance,
        })
    }
}

impl NetworkConstraints {
    pub fn to_proto(&self) -> crate::proto::dtn::NetworkConstraints {
        crate::proto::dtn::NetworkConstraints {
            required_capabilities: self.required_capabilities.clone(),
            min_bandwidth: self.min_bandwidth,
            max_latency: self.max_latency,
            min_security_level: self.min_security_level.to_proto(),
        }
    }
    
    pub fn from_proto(proto: crate::proto::dtn::NetworkConstraints) -> Result<Self, EnvelopeError> {
        Ok(Self {
            required_capabilities: proto.required_capabilities,
            min_bandwidth: proto.min_bandwidth,
            max_latency: proto.max_latency,
            min_security_level: SecurityLevel::from_proto(proto.min_security_level)?,
        })
    }
}

impl SecurityLevel {
    pub fn to_proto(&self) -> i32 {
        match self {
            SecurityLevel::None => 1,
            SecurityLevel::Low => 2,
            SecurityLevel::Medium => 3,
            SecurityLevel::High => 4,
            SecurityLevel::Critical => 5,
        }
    }
    
    pub fn from_proto(proto: i32) -> Result<Self, EnvelopeError> {
        match proto {
            1 => Ok(SecurityLevel::None),
            2 => Ok(SecurityLevel::Low),
            3 => Ok(SecurityLevel::Medium),
            4 => Ok(SecurityLevel::High),
            5 => Ok(SecurityLevel::Critical),
            _ => Err(EnvelopeError::InvalidEnvelope(
                format!("Unknown security level: {}", proto)
            )),
        }
    }
}

impl EncryptionInfo {
    pub fn to_proto(&self) -> crate::proto::dtn::EncryptionInfo {
        crate::proto::dtn::EncryptionInfo {
            encrypted: self.encrypted,
            algorithm: self.algorithm.clone(),
            iv: self.iv.clone(),
            encrypted_key: self.encrypted_key.clone(),
        }
    }
    
    pub fn from_proto(proto: crate::proto::dtn::EncryptionInfo) -> Result<Self, EnvelopeError> {
        Ok(Self {
            encrypted: proto.encrypted,
            algorithm: proto.algorithm,
            iv: proto.iv,
            encrypted_key: proto.encrypted_key,
        })
    }
}

impl AckRecord {
    pub fn to_proto(&self) -> crate::proto::dtn::AckRecord {
        crate::proto::dtn::AckRecord {
            ack_node: self.ack_node.clone(),
            ack_type: self.ack_type.to_proto(),
            ack_time: Some(self.ack_time.into()),
            status: self.status.to_proto(),
            message: self.message.clone(),
        }
    }
    
    pub fn from_proto(proto: crate::proto::dtn::AckRecord) -> Result<Self, EnvelopeError> {
        let ack_time = proto.ack_time
            .ok_or_else(|| EnvelopeError::InvalidEnvelope("Missing ack_time".to_string()))?
            .try_into()
            .map_err(|_| EnvelopeError::InvalidEnvelope("Invalid ack_time".to_string()))?;
        
        Ok(Self {
            ack_node: proto.ack_node,
            ack_type: AckType::from_proto(proto.ack_type)?,
            ack_time,
            status: AckStatus::from_proto(proto.status)?,
            message: proto.message,
        })
    }
}

impl Default for RoutingInfo {
    fn default() -> Self {
        Self {
            intermediate_nodes: Vec::new(),
            max_hops: 10,
            hop_count: 0,
            strategy: RoutingStrategy::Directed,
            geo_constraints: None,
            net_constraints: None,
        }
    }
}

impl Default for AckRequirements {
    fn default() -> Self {
        Self {
            required: false,
            ack_type: AckType::Delivery,
            ack_timeout: 300,
            max_retries: 3,
            required_ack_nodes: Vec::new(),
            ack_records: Vec::new(),
        }
    }
}

impl Default for SecurityInfo {
    fn default() -> Self {
        Self {
            signature: Vec::new(),
            signer_public_key: Vec::new(),
            signature_algorithm: "ed25519".to_string(),
            payload_hash: Vec::new(),
            hash_algorithm: "sha256".to_string(),
            encryption: None,
        }
    }
}

// Conversion traits for SystemTime <-> Timestamp
impl From<SystemTime> for Timestamp {
    fn from(time: SystemTime) -> Self {
        let duration = time.duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0));
        Timestamp {
            seconds: duration.as_secs() as i64,
            nanos: duration.subsec_nanos() as i32,
        }
    }
}

impl TryFrom<Timestamp> for SystemTime {
    type Error = EnvelopeError;
    
    fn try_from(timestamp: Timestamp) -> Result<Self, Self::Error> {
        let duration = Duration::from_secs(timestamp.seconds as u64)
            + Duration::from_nanos(timestamp.nanos as u64);
        Ok(UNIX_EPOCH + duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_envelope_creation() {
        let envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        assert_eq!(envelope.source, "did:polynet:source");
        assert_eq!(envelope.destination, "did:polynet:dest");
        assert_eq!(envelope.ttl, 3600);
        assert_eq!(envelope.message_type, "test_message");
        assert_eq!(envelope.payload, b"test payload");
        assert_eq!(envelope.priority, 100);
        assert!(!envelope.is_expired());
    }
    
    #[test]
    fn test_envelope_expiration() {
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            1, // 1 second TTL
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        // Wait for expiration
        std::thread::sleep(Duration::from_secs(2));
        
        assert!(envelope.is_expired());
        assert_eq!(envelope.remaining_ttl(), 0);
    }
    
    #[test]
    fn test_envelope_routing() {
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        envelope.set_routing_strategy(RoutingStrategy::Flood);
        assert_eq!(envelope.routing.strategy, RoutingStrategy::Flood);
        
        envelope.add_intermediate_node("did:polynet:intermediate".to_string());
        assert_eq!(envelope.routing.intermediate_nodes.len(), 1);
        
        envelope.increment_hop_count().unwrap();
        assert_eq!(envelope.routing.hop_count, 1);
    }
    
    #[test]
    fn test_envelope_acknowledgments() {
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        envelope.enable_acknowledgments(AckType::Delivery, 60);
        assert!(envelope.ack_requirements.required);
        assert_eq!(envelope.ack_requirements.ack_type, AckType::Delivery);
        assert_eq!(envelope.ack_requirements.ack_timeout, 60);
        
        envelope.add_required_ack_node("did:polynet:ack1".to_string());
        envelope.add_required_ack_node("did:polynet:ack2".to_string());
        assert_eq!(envelope.ack_requirements.required_ack_nodes.len(), 2);
        
        // Initially no acks received
        assert!(!envelope.all_acks_received());
        
        // Record acknowledgments
        envelope.record_acknowledgment(
            "did:polynet:ack1".to_string(),
            AckType::Delivery,
            AckStatus::Success,
            "Ack 1".to_string(),
        );
        envelope.record_acknowledgment(
            "did:polynet:ack2".to_string(),
            AckType::Delivery,
            AckStatus::Success,
            "Ack 2".to_string(),
        );
        
        // Now all acks received
        assert!(envelope.all_acks_received());
    }
    
    #[test]
    fn test_envelope_metadata() {
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        envelope.add_metadata("key1".to_string(), "value1".to_string());
        envelope.add_metadata("key2".to_string(), "value2".to_string());
        
        assert_eq!(envelope.get_metadata("key1"), Some(&"value1".to_string()));
        assert_eq!(envelope.get_metadata("key2"), Some(&"value2".to_string()));
        assert_eq!(envelope.get_metadata("key3"), None);
    }
    
    #[test]
    fn test_envelope_protobuf_conversion() {
        let original = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        let proto = original.to_proto();
        let converted = DtnEnvelope::from_proto(proto).unwrap();
        
        assert_eq!(original.id, converted.id);
        assert_eq!(original.source, converted.source);
        assert_eq!(original.destination, converted.destination);
        assert_eq!(original.ttl, converted.ttl);
        assert_eq!(original.message_type, converted.message_type);
        assert_eq!(original.payload, converted.payload);
        assert_eq!(original.priority, converted.priority);
    }
}
