use super::envelope::{DtnEnvelope, SecurityInfo, EncryptionInfo, SecurityLevel};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use sha2::{Sha256, Digest};
use rand::Rng;
use thiserror::Error;
use tracing::{debug, info, warn, error};

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Signature error: {0}")]
    SignatureError(String),
    
    #[error("Verification error: {0}")]
    VerificationError(String),
    
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    #[error("Decryption error: {0}")]
    DecryptionError(String),
    
    #[error("Key generation error: {0}")]
    KeyGenerationError(String),
    
    #[error("Hash error: {0}")]
    HashError(String),
    
    #[error("Invalid security level: {0}")]
    InvalidSecurityLevel(String),
}

pub struct DtnSecurity {
    signing_key: Option<SigningKey>,
    verifying_keys: std::collections::HashMap<String, VerifyingKey>,
}

impl DtnSecurity {
    /// Create a new DTN security instance
    pub fn new() -> Self {
        Self {
            signing_key: None,
            verifying_keys: std::collections::HashMap::new(),
        }
    }
    
    /// Generate a new signing key
    pub fn generate_signing_key(&mut self) -> Result<Vec<u8>, SecurityError> {
        let mut rng = rand::thread_rng();
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();
        
        // Store the signing key
        self.signing_key = Some(signing_key);
        
        // Store the verifying key with its own ID
        let key_id = self.generate_key_id(&verifying_key.to_bytes());
        self.verifying_keys.insert(key_id.clone(), verifying_key);
        
        Ok(verifying_key.to_bytes().to_vec())
    }
    
    /// Import a verifying key
    pub fn import_verifying_key(&mut self, key_bytes: &[u8], key_id: &str) -> Result<(), SecurityError> {
        let verifying_key = VerifyingKey::from_bytes(key_bytes)
            .map_err(|e| SecurityError::KeyGenerationError(e.to_string()))?;
        
        self.verifying_keys.insert(key_id.to_string(), verifying_key);
        Ok(())
    }
    
    /// Sign an envelope
    pub fn sign_envelope(&self, envelope: &mut DtnEnvelope) -> Result<(), SecurityError> {
        let signing_key = self.signing_key
            .as_ref()
            .ok_or_else(|| SecurityError::SignatureError("No signing key available".to_string()))?;
        
        // Create signature message
        let message = self.create_signature_message(envelope);
        
        // Sign the message
        let signature = signing_key.sign(&message);
        
        // Update envelope security info
        envelope.security.signature = signature.to_bytes().to_vec();
        envelope.security.signer_public_key = signing_key.verifying_key().to_bytes().to_vec();
        envelope.security.signature_algorithm = "ed25519".to_string();
        
        // Update payload hash
        envelope.security.payload_hash = self.hash_payload(&envelope.payload);
        envelope.security.hash_algorithm = "sha256".to_string();
        
        info!("Signed envelope: {} with ed25519", envelope.id);
        Ok(())
    }
    
    /// Verify envelope signature
    pub fn verify_envelope_signature(&self, envelope: &DtnEnvelope) -> Result<bool, SecurityError> {
        if envelope.security.signature.is_empty() || envelope.security.signer_public_key.is_empty() {
            return Ok(false);
        }
        
        // Get the verifying key
        let verifying_key = VerifyingKey::from_bytes(&envelope.security.signer_public_key)
            .map_err(|e| SecurityError::VerificationError(e.to_string()))?;
        
        // Get the signature
        let signature = Signature::from_bytes(&envelope.security.signature)
            .map_err(|e| SecurityError::VerificationError(e.to_string()))?;
        
        // Create signature message
        let message = self.create_signature_message(envelope);
        
        // Verify the signature
        match verifying_key.verify(&message, &signature) {
            Ok(_) => {
                debug!("Signature verified for envelope: {}", envelope.id);
                Ok(true)
            }
            Err(e) => {
                warn!("Signature verification failed for envelope {}: {}", envelope.id, e);
                Ok(false)
            }
        }
    }
    
    /// Encrypt envelope payload
    pub fn encrypt_payload(
        &self,
        envelope: &mut DtnEnvelope,
        encryption_key: &[u8],
        algorithm: &str,
    ) -> Result<(), SecurityError> {
        // For simplicity, we'll use a basic XOR encryption
        // In practice, you would use proper encryption libraries like AES
        let encrypted_payload = self.xor_encrypt(&envelope.payload, encryption_key);
        
        // Generate IV (initialization vector)
        let mut rng = rand::thread_rng();
        let mut iv = vec![0u8; 16];
        rng.fill(&mut iv);
        
        // Update envelope security info
        envelope.security.encryption = Some(EncryptionInfo {
            encrypted: true,
            algorithm: algorithm.to_string(),
            iv,
            encrypted_key: encryption_key.to_vec(),
        });
        
        // Replace payload with encrypted version
        envelope.payload = encrypted_payload;
        
        info!("Encrypted payload for envelope: {} with {}", envelope.id, algorithm);
        Ok(())
    }
    
    /// Decrypt envelope payload
    pub fn decrypt_payload(
        &self,
        envelope: &mut DtnEnvelope,
        decryption_key: &[u8],
    ) -> Result<(), SecurityError> {
        let encryption_info = envelope.security.encryption
            .as_ref()
            .ok_or_else(|| SecurityError::DecryptionError("Payload not encrypted".to_string()))?;
        
        if !encryption_info.encrypted {
            return Ok(());
        }
        
        // For simplicity, we'll use basic XOR decryption
        // In practice, you would use proper decryption
        let decrypted_payload = self.xor_encrypt(&envelope.payload, decryption_key);
        
        // Replace payload with decrypted version
        envelope.payload = decrypted_payload;
        
        // Mark as not encrypted
        if let Some(ref mut enc_info) = envelope.security.encryption {
            enc_info.encrypted = false;
        }
        
        info!("Decrypted payload for envelope: {}", envelope.id);
        Ok(())
    }
    
    /// Check security level compliance
    pub fn check_security_level(
        &self,
        envelope: &DtnEnvelope,
        required_level: SecurityLevel,
    ) -> Result<bool, SecurityError> {
        let current_level = self.calculate_security_level(envelope);
        
        Ok(current_level >= required_level)
    }
    
    /// Calculate current security level
    fn calculate_security_level(&self, envelope: &DtnEnvelope) -> SecurityLevel {
        let mut score = 0;
        
        // Check signature
        if !envelope.security.signature.is_empty() {
            score += 2;
        }
        
        // Check encryption
        if let Some(enc_info) = &envelope.security.encryption {
            if enc_info.encrypted {
                score += 2;
            }
        }
        
        // Check hash
        if !envelope.security.payload_hash.is_empty() {
            score += 1;
        }
        
        // Map score to security level
        match score {
            0 => SecurityLevel::None,
            1 => SecurityLevel::Low,
            2 => SecurityLevel::Medium,
            3 => SecurityLevel::High,
            _ => SecurityLevel::Critical,
        }
    }
    
    /// Create signature message
    fn create_signature_message(&self, envelope: &DtnEnvelope) -> Vec<u8> {
        let mut hasher = Sha256::new();
        
        // Include all critical fields in signature
        hasher.update(envelope.id.as_bytes());
        hasher.update(envelope.source.as_bytes());
        hasher.update(envelope.destination.as_bytes());
        hasher.update(envelope.ttl.to_le_bytes());
        hasher.update(envelope.created_at.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs().to_le_bytes());
        hasher.update(&envelope.nonce);
        hasher.update(envelope.priority.to_le_bytes());
        hasher.update(envelope.message_type.as_bytes());
        hasher.update(&envelope.payload);
        
        hasher.finalize().to_vec()
    }
    
    /// Hash payload
    fn hash_payload(&self, payload: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(payload);
        hasher.finalize().to_vec()
    }
    
    /// Generate key ID
    fn generate_key_id(&self, key_bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key_bytes);
        let hash = hasher.finalize();
        format!("key_{:x}", hash)
    }
    
    /// Simple XOR encryption (for demonstration - not secure for production)
    fn xor_encrypt(&self, data: &[u8], key: &[u8]) -> Vec<u8> {
        data.iter()
            .zip(key.iter().cycle())
            .map(|(d, k)| d ^ k)
            .collect()
    }
    
    /// Get available signing key
    pub fn get_signing_key(&self) -> Option<&SigningKey> {
        self.signing_key.as_ref()
    }
    
    /// Get verifying key by ID
    pub fn get_verifying_key(&self, key_id: &str) -> Option<&VerifyingKey> {
        self.verifying_keys.get(key_id)
    }
    
    /// List all verifying keys
    pub fn list_verifying_keys(&self) -> Vec<String> {
        self.verifying_keys.keys().cloned().collect()
    }
    
    /// Remove verifying key
    pub fn remove_verifying_key(&mut self, key_id: &str) -> bool {
        self.verifying_keys.remove(key_id).is_some()
    }
    
    /// Clear all keys
    pub fn clear_keys(&mut self) {
        self.signing_key = None;
        self.verifying_keys.clear();
    }
}

impl Default for DtnSecurity {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::envelope::{DtnEnvelope, SecurityLevel};
    
    #[test]
    fn test_security_creation() {
        let security = DtnSecurity::new();
        assert!(security.get_signing_key().is_none());
        assert!(security.list_verifying_keys().is_empty());
    }
    
    #[test]
    fn test_key_generation() {
        let mut security = DtnSecurity::new();
        let public_key = security.generate_signing_key().unwrap();
        
        assert!(security.get_signing_key().is_some());
        assert_eq!(security.list_verifying_keys().len(), 1);
        assert_eq!(public_key.len(), 32); // ed25519 public key size
    }
    
    #[test]
    fn test_envelope_signing() {
        let mut security = DtnSecurity::new();
        security.generate_signing_key().unwrap();
        
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        security.sign_envelope(&mut envelope).unwrap();
        
        assert!(!envelope.security.signature.is_empty());
        assert!(!envelope.security.signer_public_key.is_empty());
        assert_eq!(envelope.security.signature_algorithm, "ed25519");
        assert_eq!(envelope.security.hash_algorithm, "sha256");
    }
    
    #[test]
    fn test_signature_verification() {
        let mut security = DtnSecurity::new();
        security.generate_signing_key().unwrap();
        
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        // Sign the envelope
        security.sign_envelope(&mut envelope).unwrap();
        
        // Verify the signature
        let is_valid = security.verify_envelope_signature(&envelope).unwrap();
        assert!(is_valid);
    }
    
    #[test]
    fn test_signature_verification_failure() {
        let mut security = DtnSecurity::new();
        security.generate_signing_key().unwrap();
        
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        // Sign the envelope
        security.sign_envelope(&mut envelope).unwrap();
        
        // Tamper with the payload
        envelope.payload = b"tampered payload".to_vec();
        
        // Verification should fail
        let is_valid = security.verify_envelope_signature(&envelope).unwrap();
        assert!(!is_valid);
    }
    
    #[test]
    fn test_payload_encryption() {
        let security = DtnSecurity::new();
        
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        let original_payload = envelope.payload.clone();
        let encryption_key = b"secret_key_32_bytes_long_key_here";
        
        security.encrypt_payload(&mut envelope, encryption_key, "xor").unwrap();
        
        assert!(envelope.security.encryption.is_some());
        assert_ne!(envelope.payload, original_payload);
        
        let enc_info = envelope.security.encryption.as_ref().unwrap();
        assert!(enc_info.encrypted);
        assert_eq!(enc_info.algorithm, "xor");
    }
    
    #[test]
    fn test_payload_decryption() {
        let security = DtnSecurity::new();
        
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        let original_payload = envelope.payload.clone();
        let encryption_key = b"secret_key_32_bytes_long_key_here";
        
        // Encrypt
        security.encrypt_payload(&mut envelope, encryption_key, "xor").unwrap();
        assert_ne!(envelope.payload, original_payload);
        
        // Decrypt
        security.decrypt_payload(&mut envelope, encryption_key).unwrap();
        assert_eq!(envelope.payload, original_payload);
    }
    
    #[test]
    fn test_security_level_calculation() {
        let mut security = DtnSecurity::new();
        security.generate_signing_key().unwrap();
        
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        // Initially no security
        let level = security.calculate_security_level(&envelope);
        assert_eq!(level, SecurityLevel::None);
        
        // After signing
        security.sign_envelope(&mut envelope).unwrap();
        let level = security.calculate_security_level(&envelope);
        assert_eq!(level, SecurityLevel::Medium);
        
        // After encryption
        let encryption_key = b"secret_key_32_bytes_long_key_here";
        security.encrypt_payload(&mut envelope, encryption_key, "xor").unwrap();
        let level = security.calculate_security_level(&envelope);
        assert_eq!(level, SecurityLevel::High);
    }
    
    #[test]
    fn test_security_level_compliance() {
        let mut security = DtnSecurity::new();
        security.generate_signing_key().unwrap();
        
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        // Check compliance with different levels
        assert!(security.check_security_level(&envelope, SecurityLevel::None).unwrap());
        assert!(!security.check_security_level(&envelope, SecurityLevel::Low).unwrap());
        
        // Sign the envelope
        security.sign_envelope(&mut envelope).unwrap();
        assert!(security.check_security_level(&envelope, SecurityLevel::Medium).unwrap());
        assert!(!security.check_security_level(&envelope, SecurityLevel::High).unwrap());
    }
    
    #[test]
    fn test_key_import_export() {
        let mut security = DtnSecurity::new();
        
        // Generate a key
        let public_key = security.generate_signing_key().unwrap();
        let key_id = security.list_verifying_keys()[0].clone();
        
        // Remove the key
        security.remove_verifying_key(&key_id);
        assert!(security.list_verifying_keys().is_empty());
        
        // Import the key back
        security.import_verifying_key(&public_key, &key_id).unwrap();
        assert_eq!(security.list_verifying_keys().len(), 1);
        assert!(security.get_verifying_key(&key_id).is_some());
    }
}
