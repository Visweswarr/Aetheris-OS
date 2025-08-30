use crate::did::{Did, DidDocument, DidVerificationMethod};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use blake3::Hasher;

/// Anchor priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnchorPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for AnchorPriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl std::fmt::Display for AnchorPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Normal => write!(f, "normal"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Anchor status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnchorStatus {
    Pending,
    Submitted,
    Confirmed,
    Failed,
    Expired,
}

impl Default for AnchorStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for AnchorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Submitted => write!(f, "submitted"),
            Self::Confirmed => write!(f, "confirmed"),
            Self::Failed => write!(f, "failed"),
            Self::Expired => write!(f, "expired"),
        }
    }
}

/// Anchor metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub priority: AnchorPriority,
    pub batch_id: Option<String>,
    pub expires_at: Option<u64>,
    pub custom_fields: HashMap<String, serde_cbor::Value>,
}

impl Default for AnchorMetadata {
    fn default() -> Self {
        Self {
            description: None,
            tags: Vec::new(),
            priority: AnchorPriority::default(),
            batch_id: None,
            expires_at: None,
            custom_fields: HashMap::new(),
        }
    }
}

/// Core anchor structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorV1 {
    pub snap: Vec<u8>,                    // NGFS snapshot CID (blake3-256)
    pub did: String,                      // DID that created this anchor
    pub time: u64,                        // Unix timestamp when anchor was created
    pub chain: String,                    // Target blockchain identifier
    pub version: String,                  // Schema version
    pub metadata: AnchorMetadata,         // Additional anchor metadata
    pub signature: Vec<u8>,               // DID signature of anchor data
    pub gas_used: u64,                    // Gas consumed for anchoring
    pub block_number: Option<u64>,        // Block number where anchor was recorded
    pub transaction_hash: Option<Vec<u8>>, // Transaction hash of anchor submission
    pub status: AnchorStatus,             // Current status of the anchor
}

impl AnchorV1 {
    /// Create a new anchor for a snapshot
    pub fn new(
        snapshot_cid: &[u8],
        did: &str,
        chain: &str,
        metadata: AnchorMetadata,
    ) -> Self {
        let now = Utc::now().timestamp() as u64;
        
        Self {
            snap: snapshot_cid.to_vec(),
            did: did.to_string(),
            time: now,
            chain: chain.to_string(),
            version: "1.0".to_string(),
            metadata,
            signature: Vec::new(), // Will be set by sign_anchor
            gas_used: 0,           // Will be set after submission
            block_number: None,    // Will be set after confirmation
            transaction_hash: None, // Will be set after submission
            status: AnchorStatus::Pending,
        }
    }

    /// Get the anchor hash (for blockchain submission)
    pub fn anchor_hash(&self) -> Vec<u8> {
        let mut hasher = Hasher::new();
        hasher.update(&self.snap);
        hasher.update(self.did.as_bytes());
        hasher.update(&self.time.to_be_bytes());
        hasher.update(self.chain.as_bytes());
        hasher.update(self.version.as_bytes());
        
        // Include metadata in hash
        if let Some(desc) = &self.metadata.description {
            hasher.update(desc.as_bytes());
        }
        for tag in &self.metadata.tags {
            hasher.update(tag.as_bytes());
        }
        hasher.update(self.metadata.priority.to_string().as_bytes());
        
        hasher.finalize().as_bytes().to_vec()
    }

    /// Get the size of the anchor data (must be ≤128 bytes for gas constraints)
    pub fn size(&self) -> usize {
        self.snap.len() + 
        self.did.len() + 
        8 + // time (u64)
        self.chain.len() + 
        self.version.len() +
        self.metadata.description.as_ref().map_or(0, |d| d.len()) +
        self.metadata.tags.iter().map(|t| t.len()).sum::<usize>() +
        1 // priority
    }

    /// Check if anchor size is within gas constraints
    pub fn is_within_gas_constraints(&self) -> bool {
        self.size() <= 128
    }

    /// Update anchor status
    pub fn update_status(&mut self, status: AnchorStatus) {
        self.status = status;
    }

    /// Set blockchain confirmation data
    pub fn set_confirmation(&mut self, block_number: u64, transaction_hash: Vec<u8>) {
        self.block_number = Some(block_number);
        self.transaction_hash = Some(transaction_hash);
        self.status = AnchorStatus::Confirmed;
    }

    /// Set gas usage
    pub fn set_gas_used(&mut self, gas_used: u64) {
        self.gas_used = gas_used;
    }

    /// Check if anchor has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.metadata.expires_at {
            let now = Utc::now().timestamp() as u64;
            now > expires_at
        } else {
            false
        }
    }
}

/// Anchor submission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorRequest {
    pub snapshot_cid: Vec<u8>,
    pub target_chain: String,
    pub priority: AnchorPriority,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub expires_at: Option<u64>,
    pub gas_limit: u64,
    pub max_fee_per_gas: Option<u64>,
    pub nonce: Option<u64>,
}

impl AnchorRequest {
    /// Create a new anchor request
    pub fn new(snapshot_cid: Vec<u8>, target_chain: String) -> Self {
        Self {
            snapshot_cid,
            target_chain,
            priority: AnchorPriority::Normal,
            description: None,
            tags: Vec::new(),
            expires_at: None,
            gas_limit: 100_000, // Default gas limit
            max_fee_per_gas: None,
            nonce: None,
        }
    }

    /// Set priority level
    pub fn with_priority(mut self, priority: AnchorPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set expiration
    pub fn with_expiration(mut self, expires_at: u64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Set gas limit
    pub fn with_gas_limit(mut self, gas_limit: u64) -> Self {
        self.gas_limit = gas_limit;
        self
    }
}

/// Anchor submission result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorResult {
    pub success: bool,
    pub anchor: Option<AnchorV1>,
    pub error: Option<AnchorError>,
    pub gas_used: u64,
    pub transaction_hash: Option<Vec<u8>>,
    pub block_number: Option<u64>,
    pub confirmation_time: Option<u64>,
    pub chain_response: Option<ChainResponse>,
}

impl AnchorResult {
    /// Create a successful result
    pub fn success(anchor: AnchorV1, gas_used: u64) -> Self {
        Self {
            success: true,
            anchor: Some(anchor),
            error: None,
            gas_used,
            transaction_hash: None,
            block_number: None,
            confirmation_time: None,
            chain_response: None,
        }
    }

    /// Create a failed result
    pub fn failure(error: AnchorError, gas_used: u64) -> Self {
        Self {
            success: false,
            anchor: None,
            error: Some(error),
            gas_used,
            transaction_hash: None,
            block_number: None,
            confirmation_time: None,
            chain_response: None,
        }
    }
}

/// Anchor error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorError {
    pub code: ErrorCode,
    pub message: String,
    pub details: Option<String>,
    pub chain_error: Option<String>,
    pub retry_after: Option<u64>,
}

impl AnchorError {
    /// Create a new error
    pub fn new(code: ErrorCode, message: String) -> Self {
        Self {
            code,
            message,
            details: None,
            chain_error: None,
            retry_after: None,
        }
    }

    /// Add details
    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    /// Add chain error
    pub fn with_chain_error(mut self, chain_error: String) -> Self {
        self.chain_error = Some(chain_error);
        self
    }

    /// Set retry after timestamp
    pub fn with_retry_after(mut self, retry_after: u64) -> Self {
        self.retry_after = Some(retry_after);
        self
    }
}

/// Error codes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorCode {
    InsufficientGas,
    InvalidSignature,
    ChainUnavailable,
    TransactionFailed,
    InsufficientFunds,
    NonceError,
    ChainError,
    Timeout,
    ValidationFailed,
    Unknown,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientGas => write!(f, "insufficient_gas"),
            Self::InvalidSignature => write!(f, "invalid_signature"),
            Self::ChainUnavailable => write!(f, "chain_unavailable"),
            Self::TransactionFailed => write!(f, "transaction_failed"),
            Self::InsufficientFunds => write!(f, "insufficient_funds"),
            Self::NonceError => write!(f, "nonce_error"),
            Self::ChainError => write!(f, "chain_error"),
            Self::Timeout => write!(f, "timeout"),
            Self::ValidationFailed => write!(f, "validation_failed"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Chain response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainResponse {
    pub chain_id: u64,
    pub block_hash: Vec<u8>,
    pub gas_price: u64,
    pub effective_gas_price: u64,
    pub cumulative_gas_used: u64,
    pub logs: Vec<LogEntry>,
    pub events: Vec<EventData>,
    pub status: u32,
}

/// Log entry from blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub address: Vec<u8>,
    pub topics: Vec<Vec<u8>>,
    pub data: Vec<u8>,
    pub block_number: u64,
    pub transaction_hash: Vec<u8>,
    pub log_index: u32,
    pub removed: bool,
}

/// Parsed event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    pub name: String,
    pub signature: String,
    pub parameters: Vec<EventParameter>,
    pub raw_log: LogEntry,
}

/// Event parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventParameter {
    pub name: String,
    pub value: serde_cbor::Value,
    pub param_type: String,
    pub indexed: bool,
}

/// Chain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub name: String,
    pub rpc_url: String,
    pub contract_address: Vec<u8>,
    pub gas_limit: u64,
    pub max_fee_per_gas: u64,
    pub priority_fee: u64,
    pub confirmations: u64,
    pub timeout: u64,
    pub retry_attempts: u64,
    pub enabled: bool,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            chain_id: 1337, // Local devnet
            name: "local".to_string(),
            rpc_url: "http://localhost:8545".to_string(),
            contract_address: vec![0u8; 20], // Placeholder
            gas_limit: 100_000,
            max_fee_per_gas: 20_000_000_000, // 20 gwei
            priority_fee: 1_000_000_000,     // 1 gwei
            confirmations: 1,
            timeout: 300, // 5 minutes
            retry_attempts: 3,
            enabled: true,
        }
    }
}

/// Anchor statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorStats {
    pub total_anchors: u64,
    pub successful_anchors: u64,
    pub failed_anchors: u64,
    pub total_gas_used: u64,
    pub average_gas_per_anchor: f64,
    pub total_cost: u64,
    pub chains_used: Vec<String>,
    pub last_anchor_time: Option<u64>,
    pub pending_anchors: u64,
}

impl Default for AnchorStats {
    fn default() -> Self {
        Self {
            total_anchors: 0,
            successful_anchors: 0,
            failed_anchors: 0,
            total_gas_used: 0,
            average_gas_per_anchor: 0.0,
            total_cost: 0,
            chains_used: Vec::new(),
            last_anchor_time: None,
            pending_anchors: 0,
        }
    }
}

/// Main anchoring service
pub struct AnchorService {
    config: ChainConfig,
    stats: Arc<RwLock<AnchorStats>>,
    anchors: Arc<RwLock<HashMap<Vec<u8>, AnchorV1>>>,
}

impl AnchorService {
    /// Create a new anchor service
    pub fn new(config: ChainConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(AnchorStats::default())),
            anchors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create an anchor for a snapshot
    pub async fn create_anchor(
        &self,
        request: AnchorRequest,
        did_document: &DidDocument,
    ) -> Result<AnchorV1, AnchorError> {
        // Validate snapshot CID
        if request.snapshot_cid.len() != 32 {
            return Err(AnchorError::new(
                ErrorCode::ValidationFailed,
                "Invalid snapshot CID length".to_string(),
            ));
        }

        // Create anchor metadata
        let metadata = AnchorMetadata {
            description: request.description,
            tags: request.tags,
            priority: request.priority,
            batch_id: None,
            expires_at: request.expires_at,
            custom_fields: HashMap::new(),
        };

        // Create anchor
        let mut anchor = AnchorV1::new(
            &request.snapshot_cid,
            &did_document.id,
            &request.target_chain,
            metadata,
        );

        // Validate anchor size
        if !anchor.is_within_gas_constraints() {
            return Err(AnchorError::new(
                ErrorCode::ValidationFailed,
                format!("Anchor size {} exceeds 128 byte limit", anchor.size()),
            ));
        }

        // Sign the anchor
        self.sign_anchor(&mut anchor, did_document).await?;

        // Store anchor
        let anchor_hash = anchor.anchor_hash();
        let mut anchors = self.anchors.write().await;
        anchors.insert(anchor_hash.clone(), anchor.clone());

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_anchors += 1;
        stats.pending_anchors += 1;
        stats.last_anchor_time = Some(anchor.time);

        Ok(anchor)
    }

    /// Sign an anchor with DID credentials
    async fn sign_anchor(
        &self,
        anchor: &mut AnchorV1,
        did_document: &DidDocument,
    ) -> Result<(), AnchorError> {
        // Get verification method
        let verification_method = did_document
            .verification_methods
            .iter()
            .find(|vm| vm.verify_type == "Ed25519VerificationKey2020")
            .ok_or_else(|| {
                AnchorError::new(
                    ErrorCode::InvalidSignature,
                    "No suitable verification method found".to_string(),
                )
            })?;

        // Create signature data (anchor hash)
        let signature_data = anchor.anchor_hash();

        // Sign the data (placeholder implementation)
        // In production, this would use the actual private key
        let signature = self.mock_sign(&signature_data, verification_method)?;
        anchor.signature = signature;

        Ok(())
    }

    /// Mock signature generation (placeholder)
    fn mock_sign(
        &self,
        data: &[u8],
        _verification_method: &DidVerificationMethod,
    ) -> Result<Vec<u8>, AnchorError> {
        // This is a placeholder - in production, you would:
        // 1. Retrieve the private key associated with the verification method
        // 2. Use Ed25519 or similar to sign the data
        // 3. Return the signature
        
        // For now, return a mock signature
        let mut mock_signature = Vec::new();
        mock_signature.extend_from_slice(data);
        mock_signature.extend_from_slice(b"_mock_signature");
        
        Ok(mock_signature)
    }

    /// Get anchor by hash
    pub async fn get_anchor(&self, anchor_hash: &[u8]) -> Option<AnchorV1> {
        let anchors = self.anchors.read().await;
        anchors.get(anchor_hash).cloned()
    }

    /// List all anchors
    pub async fn list_anchors(&self) -> Vec<AnchorV1> {
        let anchors = self.anchors.read().await;
        anchors.values().cloned().collect()
    }

    /// Get anchor statistics
    pub async fn get_stats(&self) -> AnchorStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Update anchor status
    pub async fn update_anchor_status(
        &self,
        anchor_hash: &[u8],
        status: AnchorStatus,
    ) -> Result<(), AnchorError> {
        let mut anchors = self.anchors.write().await;
        
        if let Some(anchor) = anchors.get_mut(anchor_hash) {
            anchor.update_status(status);
            
            // Update statistics
            let mut stats = self.stats.write().await;
            match status {
                AnchorStatus::Confirmed => {
                    stats.successful_anchors += 1;
                    stats.pending_anchors = stats.pending_anchors.saturating_sub(1);
                }
                AnchorStatus::Failed => {
                    stats.failed_anchors += 1;
                    stats.pending_anchors = stats.pending_anchors.saturating_sub(1);
                }
                _ => {}
            }
            
            Ok(())
        } else {
            Err(AnchorError::new(
                ErrorCode::ValidationFailed,
                "Anchor not found".to_string(),
            ))
        }
    }

    /// Verify anchor signature
    pub async fn verify_anchor_signature(
        &self,
        anchor: &AnchorV1,
        did_document: &DidDocument,
    ) -> Result<bool, AnchorError> {
        // Get verification method
        let verification_method = did_document
            .verification_methods
            .iter()
            .find(|vm| vm.verify_type == "Ed25519VerificationKey2020")
            .ok_or_else(|| {
                AnchorError::new(
                    ErrorCode::InvalidSignature,
                    "No suitable verification method found".to_string(),
                )
            })?;

        // Verify signature (placeholder implementation)
        // In production, this would use the actual public key to verify
        let signature_data = anchor.anchor_hash();
        let expected_signature = self.mock_sign(&signature_data, verification_method)?;
        
        Ok(anchor.signature == expected_signature)
    }

    /// Clean up expired anchors
    pub async fn cleanup_expired_anchors(&self) -> u64 {
        let mut anchors = self.anchors.write().await;
        let mut expired_count = 0;
        
        anchors.retain(|_, anchor| {
            if anchor.is_expired() {
                expired_count += 1;
                false
            } else {
                true
            }
        });

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_anchors = stats.total_anchors.saturating_sub(expired_count);
        
        expired_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::did::{DidDocument, DidVerificationMethod};

    fn create_mock_did_document() -> DidDocument {
        DidDocument {
            id: "did:aetheris:test:user".to_string(),
            verification_methods: vec![
                DidVerificationMethod {
                    id: "did:aetheris:test:user#key1".to_string(),
                    verify_type: "Ed25519VerificationKey2020".to_string(),
                    controller: "did:aetheris:test:user".to_string(),
                    public_key_multibase: "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
                }
            ],
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_anchor_creation() {
        let config = ChainConfig::default();
        let service = AnchorService::new(config);
        
        let request = AnchorRequest::new(
            vec![0u8; 32], // Mock CID
            "local".to_string(),
        ).with_priority(AnchorPriority::High)
         .with_description("Test anchor".to_string())
         .with_tags(vec!["test".to_string(), "anchor".to_string()]);
        
        let did_doc = create_mock_did_document();
        let result = service.create_anchor(request, &did_doc).await;
        
        assert!(result.is_ok());
        let anchor = result.unwrap();
        
        assert_eq!(anchor.chain, "local");
        assert_eq!(anchor.metadata.priority, AnchorPriority::High);
        assert_eq!(anchor.metadata.description, Some("Test anchor".to_string()));
        assert_eq!(anchor.metadata.tags, vec!["test", "anchor"]);
        assert!(!anchor.signature.is_empty());
        assert!(anchor.is_within_gas_constraints());
    }

    #[tokio::test]
    async fn test_anchor_size_constraints() {
        let config = ChainConfig::default();
        let service = AnchorService::new(config);
        
        // Create a very large CID (should fail validation)
        let large_cid = vec![0u8; 1000];
        let request = AnchorRequest::new(large_cid, "local".to_string());
        
        let did_doc = create_mock_did_document();
        let result = service.create_anchor(request, &did_doc).await;
        
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.code, ErrorCode::ValidationFailed);
            assert!(error.message.contains("exceeds 128 byte limit"));
        }
    }

    #[tokio::test]
    async fn test_anchor_statistics() {
        let config = ChainConfig::default();
        let service = AnchorService::new(config);
        
        // Create multiple anchors
        for i in 0..3 {
            let request = AnchorRequest::new(
                vec![i; 32], // Unique CID
                "local".to_string(),
            );
            
            let did_doc = create_mock_did_document();
            let _ = service.create_anchor(request, &did_doc).await;
        }
        
        let stats = service.get_stats().await;
        assert_eq!(stats.total_anchors, 3);
        assert_eq!(stats.pending_anchors, 3);
        assert_eq!(stats.successful_anchors, 0);
        assert_eq!(stats.failed_anchors, 0);
    }

    #[tokio::test]
    async fn test_anchor_status_updates() {
        let config = ChainConfig::default();
        let service = AnchorService::new(config);
        
        let request = AnchorRequest::new(vec![0u8; 32], "local".to_string());
        let did_doc = create_mock_did_document();
        let anchor = service.create_anchor(request, &did_doc).await.unwrap();
        
        // Update status to confirmed
        let anchor_hash = anchor.anchor_hash();
        service.update_anchor_status(&anchor_hash, AnchorStatus::Confirmed).await.unwrap();
        
        let stats = service.get_stats().await;
        assert_eq!(stats.successful_anchors, 1);
        assert_eq!(stats.pending_anchors, 0);
    }

    #[tokio::test]
    async fn test_anchor_signature_verification() {
        let config = ChainConfig::default();
        let service = AnchorService::new(config);
        
        let request = AnchorRequest::new(vec![0u8; 32], "local".to_string());
        let did_doc = create_mock_did_document();
        let anchor = service.create_anchor(request, &did_doc).await.unwrap();
        
        // Verify signature
        let is_valid = service.verify_anchor_signature(&anchor, &did_doc).await.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_anchor_expiration() {
        let config = ChainConfig::default();
        let service = AnchorService::new(config);
        
        let expires_at = Utc::now().timestamp() as u64 - 3600; // 1 hour ago
        let request = AnchorRequest::new(vec![0u8; 32], "local".to_string())
            .with_expiration(expires_at);
        
        let did_doc = create_mock_did_document();
        let anchor = service.create_anchor(request, &did_doc).await.unwrap();
        
        // Check if expired
        assert!(anchor.is_expired());
        
        // Cleanup expired anchors
        let expired_count = service.cleanup_expired_anchors().await;
        assert_eq!(expired_count, 1);
        
        let stats = service.get_stats().await;
        assert_eq!(stats.total_anchors, 0);
    }
}
