//! AI inference backends

pub mod onnx;
pub mod whisper;

use std::collections::HashMap;
use crate::error::AiResult;

/// Backend trait for AI inference
#[async_trait::async_trait]
pub trait InferenceBackend: Send + Sync {
    /// Initialize the backend
    async fn initialize(&mut self) -> AiResult<()>;
    
    /// Run inference on input data
    async fn infer(&self, input: &[f32]) -> AiResult<Vec<f32>>;
    
    /// Get backend information
    fn get_info(&self) -> BackendInfo;
    
    /// Check if backend is available
    fn is_available(&self) -> bool;
}

/// Backend information
#[derive(Debug, Clone)]
pub struct BackendInfo {
    /// Backend name
    pub name: String,
    /// Backend version
    pub version: String,
    /// Supported models
    pub supported_models: Vec<String>,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// Performance info
    pub performance: PerformanceInfo,
}

/// Performance information
#[derive(Debug, Clone)]
pub struct PerformanceInfo {
    /// Average inference time (ms)
    pub avg_inference_time_ms: f64,
    /// Memory usage (MB)
    pub memory_usage_mb: f64,
    /// CPU usage (%)
    pub cpu_usage_percent: f32,
    /// GPU usage (%)
    pub gpu_usage_percent: Option<f32>,
}

/// Backend registry
pub struct BackendRegistry {
    /// Registered backends
    backends: HashMap<String, Box<dyn InferenceBackend>>,
}

impl BackendRegistry {
    /// Create a new backend registry
    pub fn new() -> Self {
        Self {
            backends: HashMap::new(),
        }
    }
    
    /// Register a backend
    pub fn register(&mut self, name: String, backend: Box<dyn InferenceBackend>) {
        self.backends.insert(name, backend);
    }
    
    /// Get a backend by name
    pub fn get(&self, name: &str) -> Option<&dyn InferenceBackend> {
        self.backends.get(name).map(|b| b.as_ref())
    }
    
    /// Get all available backends
    pub fn get_available(&self) -> Vec<&dyn InferenceBackend> {
        self.backends.values()
            .filter(|b| b.is_available())
            .map(|b| b.as_ref())
            .collect()
    }
    
    /// List all backend names
    pub fn list_names(&self) -> Vec<String> {
        self.backends.keys().cloned().collect()
    }
}

impl Default for BackendRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Model information
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Model name
    pub name: String,
    /// Model path
    pub path: String,
    /// Model type
    pub model_type: ModelType,
    /// Input shape
    pub input_shape: Vec<usize>,
    /// Output shape
    pub output_shape: Vec<usize>,
    /// Model size (bytes)
    pub size_bytes: u64,
    /// Supported backends
    pub supported_backends: Vec<String>,
}

/// Model type
#[derive(Debug, Clone)]
pub enum ModelType {
    Onnx,
    Whisper,
    TensorFlow,
    PyTorch,
    Custom,
}

/// Model registry
pub struct ModelRegistry {
    /// Registered models
    models: HashMap<String, ModelInfo>,
}

impl ModelRegistry {
    /// Create a new model registry
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }
    
    /// Register a model
    pub fn register(&mut self, model: ModelInfo) {
        self.models.insert(model.name.clone(), model);
    }
    
    /// Get a model by name
    pub fn get(&self, name: &str) -> Option<&ModelInfo> {
        self.models.get(name)
    }
    
    /// List all models
    pub fn list(&self) -> Vec<&ModelInfo> {
        self.models.values().collect()
    }
    
    /// List models by type
    pub fn list_by_type(&self, model_type: &ModelType) -> Vec<&ModelInfo> {
        self.models.values()
            .filter(|m| std::mem::discriminant(&m.model_type) == std::mem::discriminant(model_type))
            .collect()
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_registry() {
        let mut registry = BackendRegistry::new();
        assert!(registry.get("nonexistent").is_none());
        assert!(registry.get_available().is_empty());
    }

    #[test]
    fn test_model_registry() {
        let mut registry = ModelRegistry::new();
        assert!(registry.get("nonexistent").is_none());
        assert!(registry.list().is_empty());
    }
}
