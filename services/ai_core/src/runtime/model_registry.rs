//! Local model registry metadata.
//!
//! The registry tracks reviewed Hugging Face-compatible model artifacts without
//! committing weights. Execution backends use this metadata for auditability and
//! fail closed when an entry is missing, unreviewed, or license-incompatible.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

use crate::error::{AiCoreError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelRegistry {
    pub models: Vec<ModelRegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelRegistryEntry {
    pub id: String,
    pub source: String,
    pub repo: String,
    pub file: String,
    pub license: String,
    pub backend: String,
    pub max_bytes: u64,
    pub revision: String,
    pub sha256: String,
    pub status: ModelRegistryStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelRegistryStatus {
    Reviewed,
    ReviewRequired,
    Blocked,
}

impl ModelRegistry {
    pub fn from_toml_str(input: &str) -> Result<Self> {
        let registry: Self = toml::from_str(input)
            .map_err(|err| AiCoreError::ConfigValidationError(format!("invalid model registry: {err}")))?;
        registry.validate()?;
        Ok(registry)
    }

    pub async fn load(path: &Path) -> Result<Self> {
        let text = tokio::fs::read_to_string(path)
            .await
            .map_err(|err| AiCoreError::IoError(format!("failed to read model registry {}: {err}", path.display())))?;
        Self::from_toml_str(&text)
    }

    pub fn get_reviewed(&self, id: &str) -> Result<&ModelRegistryEntry> {
        let entry = self
            .models
            .iter()
            .find(|entry| entry.id == id)
            .ok_or_else(|| AiCoreError::NotFound(format!("model registry entry '{id}' not found")))?;
        if entry.status != ModelRegistryStatus::Reviewed {
            return Err(AiCoreError::ConfigValidationError(format!(
                "model registry entry '{}' is not reviewed",
                entry.id
            )));
        }
        if !is_permissive_license(&entry.license) {
            return Err(AiCoreError::ConfigValidationError(format!(
                "model registry entry '{}' has unsupported license '{}'",
                entry.id, entry.license
            )));
        }
        Ok(entry)
    }

    fn validate(&self) -> Result<()> {
        let mut seen = BTreeMap::new();
        for entry in &self.models {
            if entry.id.trim().is_empty() {
                return Err(AiCoreError::ConfigValidationError(
                    "model registry entry id must not be empty".to_string(),
                ));
            }
            if seen.insert(entry.id.clone(), true).is_some() {
                return Err(AiCoreError::ConfigValidationError(format!(
                    "duplicate model registry id '{}'",
                    entry.id
                )));
            }
            if entry.max_bytes == 0 {
                return Err(AiCoreError::ConfigValidationError(format!(
                    "model registry entry '{}' must set max_bytes",
                    entry.id
                )));
            }
            if entry.sha256.trim().is_empty() {
                return Err(AiCoreError::ConfigValidationError(format!(
                    "model registry entry '{}' must set sha256 or a placeholder",
                    entry.id
                )));
            }
        }
        Ok(())
    }
}

pub fn is_permissive_license(license: &str) -> bool {
    matches!(
        license.to_ascii_lowercase().as_str(),
        "apache-2.0" | "mit" | "bsd-2-clause" | "bsd-3-clause"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const REGISTRY: &str = r#"
[[models]]
id = "smollm2"
source = "huggingface"
repo = "HuggingFaceTB/SmolLM2-135M-Instruct"
file = "model.safetensors"
license = "apache-2.0"
backend = "llama-cpp"
max_bytes = 300000000
revision = "main"
sha256 = "manual-download-required"
status = "reviewed"
"#;

    #[test]
    fn registry_accepts_reviewed_permissive_entry() {
        let registry = ModelRegistry::from_toml_str(REGISTRY).unwrap();
        assert_eq!(registry.get_reviewed("smollm2").unwrap().license, "apache-2.0");
    }

    #[test]
    fn registry_rejects_unknown_entry() {
        let registry = ModelRegistry::from_toml_str(REGISTRY).unwrap();
        assert!(registry.get_reviewed("unknown").is_err());
    }
}
