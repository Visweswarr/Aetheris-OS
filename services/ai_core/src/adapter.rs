//! LoRA/adapter manifest and version hooks.
//!
//! Real tensor updates are delegated to feature-gated runtime backends. The
//! registry here enforces adapter lifecycle, versioning, and auditability.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::contracts::{sha256_hex, AdapterManifest, AdapterState};
use crate::error::{AiCoreError, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum AdapterBackend {
    Mock,
    External(String),
}

#[derive(Debug, Clone)]
pub struct AdapterRegistry {
    backend: AdapterBackend,
    adapters: Arc<RwLock<HashMap<String, AdapterManifest>>>,
    versions: Arc<RwLock<HashMap<String, Vec<AdapterManifest>>>>,
}

impl AdapterRegistry {
    pub fn new(backend: AdapterBackend) -> Self {
        Self {
            backend,
            adapters: Arc::new(RwLock::new(HashMap::new())),
            versions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn attach(&self, mut manifest: AdapterManifest) -> Result<AdapterManifest> {
        self.validate_manifest(&manifest)?;
        manifest.state = AdapterState::Attached;
        self.adapters
            .write()
            .await
            .insert(manifest.adapter_id.clone(), manifest.clone());
        self.record_version(manifest.clone()).await;
        Ok(manifest)
    }

    pub async fn train(&self, adapter_id: &str, training_digest: &[u8]) -> Result<AdapterManifest> {
        let mut adapters = self.adapters.write().await;
        let adapter = adapters
            .get_mut(adapter_id)
            .ok_or_else(|| AiCoreError::NotFoundError(format!("adapter {} not found", adapter_id)))?;

        match &self.backend {
            AdapterBackend::Mock => {
                adapter.state = AdapterState::Ready;
                adapter.sha256 = Some(sha256_hex(training_digest));
                adapter.version = next_patch_version(&adapter.version);
                let snapshot = adapter.clone();
                drop(adapters);
                self.record_version(snapshot.clone()).await;
                Ok(snapshot)
            }
            AdapterBackend::External(name) => Err(AiCoreError::FeatureNotAvailable(format!(
                "adapter backend {} is not wired in this build",
                name
            ))),
        }
    }

    pub async fn detach(&self, adapter_id: &str) -> Result<AdapterManifest> {
        let mut adapters = self.adapters.write().await;
        let adapter = adapters
            .get_mut(adapter_id)
            .ok_or_else(|| AiCoreError::NotFoundError(format!("adapter {} not found", adapter_id)))?;
        adapter.state = AdapterState::Detached;
        let snapshot = adapter.clone();
        drop(adapters);
        self.record_version(snapshot.clone()).await;
        Ok(snapshot)
    }

    pub async fn versions(&self, adapter_id: &str) -> Vec<AdapterManifest> {
        self.versions
            .read()
            .await
            .get(adapter_id)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn get(&self, adapter_id: &str) -> Option<AdapterManifest> {
        self.adapters.read().await.get(adapter_id).cloned()
    }

    fn validate_manifest(&self, manifest: &AdapterManifest) -> Result<()> {
        if manifest.adapter_id.trim().is_empty() {
            return Err(AiCoreError::ValidationError("adapter_id is required".to_string()));
        }
        if manifest.base_model_id.trim().is_empty() {
            return Err(AiCoreError::ValidationError("base_model_id is required".to_string()));
        }
        if manifest.rank == 0 || manifest.rank > 256 {
            return Err(AiCoreError::ValidationError("adapter rank must be 1..=256".to_string()));
        }
        if manifest.target_modules.is_empty() {
            return Err(AiCoreError::ValidationError("target_modules cannot be empty".to_string()));
        }
        Ok(())
    }

    async fn record_version(&self, manifest: AdapterManifest) {
        self.versions
            .write()
            .await
            .entry(manifest.adapter_id.clone())
            .or_default()
            .push(manifest);
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new(AdapterBackend::Mock)
    }
}

fn next_patch_version(version: &str) -> String {
    let mut parts: Vec<u64> = version
        .split('.')
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect();
    while parts.len() < 3 {
        parts.push(0);
    }
    parts[2] += 1;
    format!("{}.{}.{}", parts[0], parts[1], parts[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> AdapterManifest {
        AdapterManifest {
            adapter_id: "adapter-1".to_string(),
            base_model_id: "model-1".to_string(),
            rank: 8,
            alpha: 16.0,
            target_modules: vec!["q_proj".to_string()],
            version: "1.0.0".to_string(),
            sha256: None,
            state: AdapterState::Registered,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn records_adapter_versions() {
        let registry = AdapterRegistry::default();
        registry.attach(manifest()).await.unwrap();
        registry.train("adapter-1", b"training").await.unwrap();
        registry.detach("adapter-1").await.unwrap();

        let versions = registry.versions("adapter-1").await;
        assert_eq!(versions.len(), 3);
        assert_eq!(versions[1].version, "1.0.1");
    }
}
