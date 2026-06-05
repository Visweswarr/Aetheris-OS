//! Secure model loader with hash, signature, policy, and audit gates.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use base64::{engine::general_purpose, Engine as _};
use ed25519_dalek::{Signature as Ed25519Signature, Verifier, VerifyingKey};
use p256::ecdsa::{Signature as EcdsaSignature, VerifyingKey as EcdsaVerifyingKey};
use tokio::sync::RwLock;

use crate::audit::AuditTrail;
use crate::contracts::{
    sha256_hex, timestamp_90khz, AuditEvent, ModelManifest, SignatureAlgorithm, VerificationResult,
};
use crate::error::{AiCoreError, Result};

#[derive(Debug, Clone)]
pub struct ModelPolicy {
    pub require_signature: bool,
    pub allowed_licenses: HashSet<String>,
    pub trusted_signers: HashSet<String>,
    pub max_model_bytes: u64,
}

impl Default for ModelPolicy {
    fn default() -> Self {
        Self {
            require_signature: true,
            allowed_licenses: ["MIT", "Apache-2.0", "BSD-3-Clause", "Proprietary-Approved"]
                .into_iter()
                .map(String::from)
                .collect(),
            trusted_signers: HashSet::new(),
            max_model_bytes: 8 * 1024 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SecureModelLoader {
    policy: ModelPolicy,
    loaded: Arc<RwLock<HashMap<String, ModelManifest>>>,
    audit: AuditTrail,
}

impl SecureModelLoader {
    pub fn new(policy: ModelPolicy, audit: AuditTrail) -> Self {
        Self {
            policy,
            loaded: Arc::new(RwLock::new(HashMap::new())),
            audit,
        }
    }

    pub async fn load(
        &self,
        manifest: ModelManifest,
        actor: &str,
        trace_id: Option<String>,
    ) -> Result<VerificationResult> {
        let verification = self.verify(&manifest).await?;
        let allowed = verification.is_allowed();

        self.audit
            .record(AuditEvent {
                event_id: format!("audit-model-load-{}", manifest.model_id),
                actor: actor.to_string(),
                action: "model.load".to_string(),
                subject: manifest.model_id.clone(),
                allowed,
                reason: if allowed {
                    "model verified".to_string()
                } else {
                    verification.errors.join("; ")
                },
                timestamp_90khz: timestamp_90khz(),
                trace_id,
                metadata: HashMap::from([
                    ("sha256".to_string(), manifest.sha256.clone()),
                    ("license".to_string(), manifest.license.clone()),
                ]),
            })
            .await;

        if !allowed {
            return Err(AiCoreError::AuthorizationError(format!(
                "model {} failed verification: {}",
                manifest.model_id,
                verification.errors.join("; ")
            )));
        }

        self.loaded
            .write()
            .await
            .insert(manifest.model_id.clone(), manifest);
        Ok(verification)
    }

    pub async fn verify(&self, manifest: &ModelManifest) -> Result<VerificationResult> {
        let mut errors = Vec::new();

        let hash_verified = match self.read_model_bytes(manifest).await {
            Ok(bytes) => {
                let actual = sha256_hex(&bytes);
                if actual.eq_ignore_ascii_case(&manifest.sha256) {
                    true
                } else {
                    errors.push(format!(
                        "hash mismatch: expected {}, got {}",
                        manifest.sha256, actual
                    ));
                    false
                }
            }
            Err(error) => {
                errors.push(error.to_string());
                false
            }
        };

        let signature_verified = self.verify_signatures(manifest).map_err(|error| {
            AiCoreError::AuthorizationError(format!("signature verification failed: {}", error))
        })?;
        if !signature_verified && self.policy.require_signature {
            errors.push("no trusted signature verified".to_string());
        }

        let policy_allowed = self.policy_allows(manifest, &mut errors);

        Ok(VerificationResult {
            hash_verified,
            signature_verified: signature_verified || !self.policy.require_signature,
            policy_allowed,
            errors,
        })
    }

    pub async fn unload(&self, model_id: &str, actor: &str) -> bool {
        let removed = self.loaded.write().await.remove(model_id).is_some();
        self.audit
            .record(AuditEvent {
                event_id: format!("audit-model-unload-{}", model_id),
                actor: actor.to_string(),
                action: "model.unload".to_string(),
                subject: model_id.to_string(),
                allowed: removed,
                reason: if removed {
                    "model unloaded"
                } else {
                    "model not loaded"
                }
                .to_string(),
                timestamp_90khz: timestamp_90khz(),
                trace_id: None,
                metadata: HashMap::new(),
            })
            .await;
        removed
    }

    pub async fn list(&self) -> Vec<ModelManifest> {
        self.loaded.read().await.values().cloned().collect()
    }

    pub async fn get(&self, model_id: &str) -> Option<ModelManifest> {
        self.loaded.read().await.get(model_id).cloned()
    }

    async fn read_model_bytes(&self, manifest: &ModelManifest) -> Result<Vec<u8>> {
        let path = Path::new(&manifest.path);
        let metadata = tokio::fs::metadata(path).await.map_err(|error| {
            AiCoreError::ModelError(format!("cannot stat model {}: {}", manifest.path, error))
        })?;

        if metadata.len() != manifest.size_bytes {
            return Err(AiCoreError::ModelError(format!(
                "size mismatch: expected {} bytes, got {} bytes",
                manifest.size_bytes,
                metadata.len()
            )));
        }

        if metadata.len() > self.policy.max_model_bytes {
            return Err(AiCoreError::ResourceError(format!(
                "model size {} exceeds policy limit {}",
                metadata.len(),
                self.policy.max_model_bytes
            )));
        }

        tokio::fs::read(path).await.map_err(|error| {
            AiCoreError::ModelError(format!("cannot read model {}: {}", manifest.path, error))
        })
    }

    fn verify_signatures(&self, manifest: &ModelManifest) -> std::result::Result<bool, String> {
        if manifest.signatures.is_empty() {
            return Ok(false);
        }

        for signature in &manifest.signatures {
            if !self.policy.trusted_signers.is_empty()
                && !self.policy.trusted_signers.contains(&signature.signer)
            {
                continue;
            }

            match signature.algorithm {
                SignatureAlgorithm::Ed25519 => {
                    if self.verify_ed25519(manifest, signature)? {
                        return Ok(true);
                    }
                }
                SignatureAlgorithm::EcdsaP256Sha256 => {
                    if self.verify_ecdsa_p256_sha256(manifest, signature)? {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    fn verify_ed25519(
        &self,
        manifest: &ModelManifest,
        signature: &crate::contracts::ModelSignature,
    ) -> std::result::Result<bool, String> {
        let public_key = general_purpose::STANDARD
            .decode(&signature.public_key_b64)
            .map_err(|error| error.to_string())?;
        let signature_bytes = general_purpose::STANDARD
            .decode(&signature.signature_b64)
            .map_err(|error| error.to_string())?;

        let public_key: [u8; 32] = public_key
            .try_into()
            .map_err(|_| "ed25519 public key must be 32 bytes".to_string())?;
        let verifying_key =
            VerifyingKey::from_bytes(&public_key).map_err(|error| error.to_string())?;
        let signature =
            Ed25519Signature::from_slice(&signature_bytes).map_err(|error| error.to_string())?;

        let signed_payload = self.signature_payload(manifest);
        Ok(verifying_key
            .verify(signed_payload.as_bytes(), &signature)
            .is_ok())
    }

    fn verify_ecdsa_p256_sha256(
        &self,
        manifest: &ModelManifest,
        signature: &crate::contracts::ModelSignature,
    ) -> std::result::Result<bool, String> {
        let public_key = general_purpose::STANDARD
            .decode(&signature.public_key_b64)
            .map_err(|error| error.to_string())?;
        let signature_bytes = general_purpose::STANDARD
            .decode(&signature.signature_b64)
            .map_err(|error| error.to_string())?;

        let verifying_key =
            EcdsaVerifyingKey::from_sec1_bytes(&public_key).map_err(|error| error.to_string())?;
        let signature = EcdsaSignature::from_der(&signature_bytes)
            .or_else(|_| EcdsaSignature::try_from(signature_bytes.as_slice()))
            .map_err(|error| error.to_string())?;

        let signed_payload = self.signature_payload(manifest);
        Ok(verifying_key
            .verify(signed_payload.as_bytes(), &signature)
            .is_ok())
    }

    fn signature_payload(&self, manifest: &ModelManifest) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            manifest.model_id,
            manifest.version,
            manifest.sha256,
            manifest.size_bytes,
            manifest.license
        )
    }

    fn policy_allows(&self, manifest: &ModelManifest, errors: &mut Vec<String>) -> bool {
        let mut allowed = true;

        if !self.policy.allowed_licenses.contains(&manifest.license) {
            errors.push(format!("license {} is not allowed", manifest.license));
            allowed = false;
        }

        if manifest.size_bytes > self.policy.max_model_bytes {
            errors.push(format!(
                "model size {} exceeds policy limit {}",
                manifest.size_bytes, self.policy.max_model_bytes
            ));
            allowed = false;
        }

        allowed
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;
    use tokio::io::AsyncWriteExt;

    use crate::contracts::{Modality, ModelFormat, ModelSignature};

    use super::*;

    async fn temp_model(contents: &[u8]) -> (NamedTempFile, ModelManifest) {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();
        let mut tokio_file = tokio::fs::File::create(&path).await.unwrap();
        tokio_file.write_all(contents).await.unwrap();
        tokio_file.flush().await.unwrap();

        let manifest = ModelManifest {
            model_id: "model-test".to_string(),
            path: path.display().to_string(),
            format: ModelFormat::Onnx,
            modality: Modality::Text,
            version: "1.0.0".to_string(),
            sha256: sha256_hex(contents),
            size_bytes: contents.len() as u64,
            license: "MIT".to_string(),
            signatures: Vec::new(),
            metadata: HashMap::new(),
        };

        (file, manifest)
    }

    #[tokio::test]
    async fn denies_unsigned_model_when_policy_requires_signature() {
        let (_file, manifest) = temp_model(b"model").await;
        let loader = SecureModelLoader::new(ModelPolicy::default(), AuditTrail::new());

        let verification = loader.verify(&manifest).await.unwrap();
        assert!(!verification.is_allowed());
        assert!(verification
            .errors
            .iter()
            .any(|error| error.contains("signature")));
    }

    #[tokio::test]
    async fn can_load_unsigned_model_when_policy_allows_ci_mock() {
        let (_file, manifest) = temp_model(b"model").await;
        let mut policy = ModelPolicy::default();
        policy.require_signature = false;
        let loader = SecureModelLoader::new(policy, AuditTrail::new());

        let verification = loader.load(manifest, "test", None).await.unwrap();
        assert!(verification.is_allowed());
        assert_eq!(loader.list().await.len(), 1);
    }

    #[tokio::test]
    async fn verifies_ecdsa_p256_signature_when_trusted() {
        use p256::ecdsa::{signature::Signer, SigningKey};
        use rand::rngs::OsRng;

        let (_file, mut manifest) = temp_model(b"model").await;
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = signing_key.verifying_key().to_encoded_point(false);
        let loader = SecureModelLoader::new(ModelPolicy::default(), AuditTrail::new());
        let payload = loader.signature_payload(&manifest);
        let signature: p256::ecdsa::Signature = signing_key.sign(payload.as_bytes());

        manifest.signatures.push(ModelSignature {
            algorithm: SignatureAlgorithm::EcdsaP256Sha256,
            public_key_b64: general_purpose::STANDARD.encode(verifying_key.as_bytes()),
            signature_b64: general_purpose::STANDARD.encode(signature.to_der().as_bytes()),
            signer: "ci-signer".to_string(),
        });

        let mut policy = ModelPolicy::default();
        policy.trusted_signers.insert("ci-signer".to_string());
        let loader = SecureModelLoader::new(policy, AuditTrail::new());

        let verification = loader.verify(&manifest).await.unwrap();
        assert!(verification.is_allowed());
    }
}
